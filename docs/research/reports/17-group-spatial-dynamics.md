# Report 17. Group Spatial Dynamics, Multi-Tile Entities and Divergence

## 0. Context

This document stands alone. It states the context that it needs.

Cachette is a world simulation engine. The core is Rust. The control plane
is Python. The engine simulates a hex world of 16.7 million tiles and about
one million units. A unit is one individual soldier, not a formation. The
engine runs on AWS Graviton servers with a 64-byte cache line, a NEON
vector unit and a weak memory model.

Four rules bind every recommendation in this document.

1. No floating point in simulated or aggregated state. Fixed point is
   Q16.16.
2. Determinism. Iterate in a defined order. Sort by a stable key. Never
   depend on thread completion order or on hash iteration order.
3. The read and write phase split. A system reads the world and writes
   events. A later phase applies the events.
4. Prefer disjoint outputs to atomic operations.

The engine stores every tile field in blocks. The level 1 pyramid cell is a
block of 256 tiles, so the world holds 65,536 level 1 cells.[^1] A unit's
position is its tile index. There is no sub-tile coordinate. A tile holds
at most 8 units. Movement is a local choice over the six neighbours plus a
sort-then-admit capacity check.[^2]

A formation already exists as an organisational node. A formation row holds
a parent, a commander, a cached strength and a bounding level 1 mask.
Membership is one `formation: u32` column on the unit row, plus a
compressed sparse row reverse index built by a counting sort.[^3]

### 0.1 What this document owns

This document owns spatial group dynamics and multi-tile entities. It does
not own individual intent selection or the group weight vector. It does not
own formation membership or the chain of command.

### 0.2 The five findings

1. **A multi-tile entity is a set of tiles, not a shape.** Store it as a
   sparse per-tile owner column over a dense bitplane. The cost is 4.6 MB.
   A footprint that crosses a storage block boundary then needs no special
   case, because the representation holds no geometry.
2. **Shared-field cohesion is mostly free, but it is not complete.** Drift
   from terrain variance grows as the square root of distance, and reaches
   about 4 tiles over 100 tiles. Drift from a chokepoint grows linearly and
   reaches 31 tiles for a 1,000-soldier regiment at a one-tile bridge.
3. **The chokepoint problem is a waiting problem, not an attraction
   problem.** A rally bias cannot correct it. A hold gate can. Adopt both,
   and give the hold gate the larger role.
4. **Reject shared need state.** The saving is 15.8 MB of storage and 0.03
   to 0.16 wall-ms for each tick. Exact allocation gives each member a
   different integer payment, so the values diverge after the first meal.
   The rule is: share what is configured, never share what is accumulated.
5. **A departed unit loses less than expected.** The flow tile cache is
   keyed on the chunk and the exit portal, not on the formation, so a
   departed unit keeps its cached route at no cost. It loses the group
   weight vector and it loses supply access. Both need an explicit rule.

Total added cost: **0.35 core-ms for each tick**, which is under 0.03
wall-ms on 12 cores. Total added storage: **4.6 MB**.

---

## 1. Terms

| Term | Meaning in this document |
|---|---|
| Site | A multi-tile entity. A camp, a city, a fortress, a wall or a bridge. |
| Footprint | The set of tiles that one site occupies. |
| Anchor | The tile that a site's deterministic order starts from. |
| Formation | An organisational node that owns soldiers and receives orders. |
| Centroid | The integer mean axial coordinate of a formation's members. |
| Leash | A distance in tiles. It bounds how far a member may lag. |
| Hold gate | A rule that stops a leading member until the rear catches up. |
| Rally bias | A small term that pulls a member toward the centroid. |
| Drift | The growth in distance between the leading and the rear member. |
| Divergence | The change in a unit's state when it leaves a formation. |
| Bitplane | One bit for each tile, stored in block-tiled order. |
| Rank-select | A prefix count plus a population count that maps a set bit to a dense payload index. |

---

## 2. Multi-tile entities

### 2.1 The gap

The design holds no representation for an entity with a footprint. Every
structure is per-tile or per-entity-at-a-tile. A camp of 13 tiles, a city,
a fortress, a bridge and a wall have no representation at all.

The owner's example fixes the scale. A company of 100 soldiers at a tile
capacity of 8 needs `ceil(100 / 8) = 13` tiles. So a camp is 13 tiles. A
city is larger. A wall is a long thin set. A bridge is a short thin set.

### 2.2 The three candidates

**Candidate A. Linked per-tile records that share an owner identifier.**
Each occupied tile carries a `site: u32` value. The site row holds the
shared data.

**Candidate B. One entity that holds a footprint mask or a bounding
shape.** The site row holds an origin and a bitmask, or an origin and a
radius.

**Candidate C. A region-tagged aggregate keyed on the storage block or the
level 1 cell.** The site is a property of a level 1 cell.

### 2.3 The four queries, costed

The table gives the cost of each query for a 13-tile camp. `F` is the
footprint tile count. `S` is the live site count. The figures assume a
2.5 GHz Graviton core, a 4-cycle L1 load and a 12-cycle L2 load.

| Query | A, owner column | B, footprint mask | C, region tag |
|---|---|---|---|
| Am I inside a camp | 1 bit test, about 1 to 4 ns | Find candidates, then test. Needs a spatial index. 20 to 100 ns | 1 read, but **wrong**. A 13-tile camp is 5% of one 256-tile cell |
| Destroy this camp | Read the reverse index run, then clear `F` bits. About 30 ns | Iterate the mask, clear `F` entries. About 30 ns | Cannot express. The cell is not the camp |
| What does this camp contain | 1 site row read, 32 bytes | 1 site row read | Reads the whole cell, not the camp |
| Does this footprint overlap that one | `F` bit tests. 13 reads, about 30 ns. **Exact** | Align the two masks, then AND. Needs a shift when origins differ | Cannot express |

Candidate C fails three of the four queries. **Reject candidate C as the
primary representation.** Section 2.7 keeps it as a derived summary.

Candidates A and B are close on cost. The difference is not cost. The
difference is the block boundary.

### 2.4 Why the block boundary decides it

The engine stores every tile field in blocks of 256 tiles.[^1] A footprint
that crosses a block boundary is the awkward case.

Candidate B holds a shape. A shape has an origin, an extent and an
implicit coordinate frame. A mask that crosses a block boundary splits
into two masks with different block origins. The engine must then shift
bits across word boundaries, or hold a second coordinate system beside the
block-tiled one. Both are new code and both are a source of defects.

Candidate A holds no shape. A footprint is a set of tiles, and a tile index
already carries its block. A footprint that crosses a block boundary is
several disjoint runs in the same array. **There is no special case at
all.**

**Recommendation: adopt candidate A. A site is a set of tiles, not a
shape.** The representation holds no geometry, so geometry cannot go wrong.

### 2.5 The storage form, and why a dense column is too large

A dense `site: u32` column over 16.7 million tiles costs 66.8 MB. That is
half of the 134 MB world budget. Reject it. A dense `u16` column costs
33.4 MB and caps the site count at 65,535. Reject that also.

Sites cover a small fraction of tiles. Take 20,000 sites at an average of
17 tiles each. That is 340,000 tiles, which is 2.0% of the world.

Use the rank-select structure that the memory layout report already
describes for sparse tile payloads.[^4] Three arrays hold the whole thing.

| Structure | Type | Size |
|---|---|---|
| `sited` bitplane | 1 bit for each tile | 2.09 MB |
| Block prefix count | `u32` for each 256-tile block | 0.26 MB |
| `site_id` payload | `u32` for each set bit, 340,000 entries | 1.36 MB |
| Site rows | 32 bytes, 20,000 rows | 0.64 MB |
| **Total** | | **4.35 MB** |

The lookup reads one bit. If the bit is clear, the query ends. That is the
answer for 98% of tiles and it costs one load.

```
fn site_at(tile: TileIdx) -> Option<SiteId> {
    let (blk, off) = split_block(tile);          // shift and mask
    let word = sited[blk * 4 + off / 64];
    if word & (1 << (off % 64)) == 0 { return None; }
    let rank = prefix[blk] + (word & ((1 << (off % 64)) - 1)).count_ones();
    Some(site_id[rank])
}
```

The bit test is one `gather`. The rank step is one `gather` plus one
population count. Both are inside the kernel vocabulary.

### 2.6 The site row

```rust
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct SiteRow {
    owner:      u32,   // CharacterId, or u32::MAX
    formation:  u32,   // FormationId, or u32::MAX
    anchor:     u32,   // TileIdx. The deterministic ordering key.
    tile_count: u32,   // Falls as tiles are destroyed.
    l1_lo:      u32,   // Lowest bounding level 1 cell index.
    l1_hi:      u32,   // Highest bounding level 1 cell index.
    kind:       u16,   // SiteKindId, an index into the type table.
    faction:    u16,
    integrity:  u16,   // Q16.16 truncated to 16 bits. 65535 is intact.
    layer:      u8,    // Which bitplane holds this site.
    flags:      u8,
}
```

The row is 32 bytes. It is `Pod` and its padding is declared, so it hashes
without false nondeterminism.

**Partial destruction is exact.** Clear the bit for the destroyed tile.
Decrement `tile_count`. Rebuild the payload at the next structure barrier.
A camp with a hole in it is a correct state, and the representation holds
it without any new field. A bounding-shape representation cannot hold it.

**Overlapping footprints are forbidden inside one layer.** The payload is
single-valued, so a tile belongs to at most one site of a layer. Use a
small fixed layer count when overlap must exist. A wall on a camp tile is
two layers. Each layer costs 2.09 MB for the bitplane plus its own prefix
and payload. Two layers cost 8.7 MB. **Recommend two layers: ground and
overlay.** Do not make the layer count data-driven, because each layer is a
separate memory-resident plane.

### 2.7 The level 1 summary

**A site identifier must not become a pyramid field.** Its cardinality is
20,000, so no aggregate of it is meaningful. This is the same reasoning
that excludes the formation identifier from the pyramid.[^3]

Two derived summaries are cheap and useful.

| Field | Type | Size at 65,536 cells |
|---|---|---|
| `site_tiles` | `u16`, count of sited tiles in the cell | 0.13 MB |
| `site_kind_count[k]` | `u8` for each of up to 8 kinds | 0.52 MB |

Take the counts, not a kind mask. A count is a group under addition, so an
incremental update applies a signed delta and needs no recompute.[^1] A
bitwise OR mask has no inverse, so removing the last fortress from a cell
forces a full recompute of that cell's children. The pyramid report records
this failure for the faction mask, and the same failure applies here.[^1]
The count costs 0.65 MB and removes the recompute.

The site's own `l1_lo` and `l1_hi` pair gives the reverse direction. It
gives the command scheduler a region scope, exactly as the formation
bounding mask does.[^3] A destroy command on a site is therefore a
region-scoped command, and two destroy commands on disjoint sites run in
parallel.

### 2.8 Placement, and the deterministic footprint

A site's footprint must be a fixed function of its inputs. Take the anchor
tile, the required tile count, the terrain and the current occupancy.

Take tiles in **hex spiral ring order** from the anchor. Ring 0 is the
anchor. Ring 1 is the six neighbours in a fixed direction order. Ring 2 is
the twelve tiles at distance 2, again in a fixed order. The order is a
compile-time table. Skip a tile that is impassable. Skip a tile whose bit
is already set in this layer. Stop when the count is reached.

This gives 13 tiles for a 100-soldier company, which matches the owner's
figure exactly. The function reads only simulated state and a constant
table, so it replays identically.

**Two sites may claim the same tile in the same tick.** Resolve it with the
device that movement already uses.[^2] Sort the claims by the key
`(target_tile, anchor, site_id)`. Each target tile then owns one contiguous
segment. Admit the first claim in each segment. Reject the rest, and let
the rejected site take the next tile in its spiral. The sort is the stable
radix sort. The segments are disjoint, so the admission scan needs no
atomic operation.

### 2.9 Where footprint work falls outside the kernel vocabulary

| Step | Kernel | Note |
|---|---|---|
| Test one tile for a site | gather | One bit test, then a rank probe. |
| Stamp a footprint | scatter | Disjoint after the claim sort. |
| Resolve claim conflicts | sort, then scan | Reuses the movement admission rule. |
| Build the site reverse index | sort, counting | 340,000 entries. |
| Overlap test | gather, then reduce | `F` bit tests, then a logical OR. |
| Update the level 1 site counts | scatter, signed delta | A group, so no recompute. |
| Spiral ring order | map over a constant table | Not data-dependent. |

**Every step is inside the vocabulary.** Nothing here needs a graph
traversal and nothing needs a new primitive. This is the direct consequence
of choosing a set over a shape. A shape representation would need a
geometric intersection step, which is not a kernel that the engine has.

---

## 3. Spatial cohesion

### 3.1 The claim to test

The lead claims that cohesion is already free. The reasoning is that every
member follows the same flow field toward the same destination, so
coherence comes from the shared field. The movement report rejected boids
for the same reason: an alignment term and a cohesion term duplicate what
the field supplies, and they oppose it.[^2]

The claim is **mostly correct and incomplete**. Four separate mechanisms
cause drift. The field removes two of them. It does not remove the other
two.

### 3.2 Drift source 1. Speed variance

Members with different speeds separate linearly with distance.

Speed is a function of the unit type and the upgrade set. Members of one
formation are usually configured identically, which is the same property
that gives the effective-stat memoisation a 99.5% to 99.98% hit rate.[^5]
When the configuration is identical, the speed is identical, and this term
is **exactly zero**.

A mixed formation is the exception. Two members whose speeds differ by 10%
separate by 10 tiles over 100 tiles of march. That is unbounded and it is
real, but it is a composition choice, not a physics problem.

**Rule: measure this at formation build time, not at run time.** Report the
slowest member speed as the formation speed. Section 3.6 gives the gate
that enforces it.

### 3.3 Drift source 2. Terrain variance

Members stand on different tiles, so they pay different step costs.

Model the difference in accumulated cost between two members as a random
walk. Let the per-step cost have a mean `c` and a standard deviation
`sigma`. After `N` steps, the difference in accumulated cost between two
members has a standard deviation of about `sigma * sqrt(2N)`. Divide by `c`
to get the separation in tiles.

| Distance `N` | `sigma / c` = 0.15 | `sigma / c` = 0.30 |
|---|---|---|
| 25 tiles | 1.1 tiles | 2.1 tiles |
| 100 tiles | 2.1 tiles | 4.2 tiles |
| 400 tiles | 4.2 tiles | 8.5 tiles |
| 1,000 tiles | 6.7 tiles | 13.4 tiles |

Growth is proportional to the square root of the distance. A 4-tile spread
over a 100-tile march is small against a 13-tile camp footprint.

**Verdict: this term needs no correction.** It is bounded in the range that
matters and its magnitude is below the formation's own footprint. The lead
is correct here.

### 3.4 Drift source 3. The chokepoint. This is the real problem

A chokepoint of one tile passes at most 8 units for each tick, because the
tile capacity is 8. The admission rule rejects the rest.[^2]

| Formation | Tiles to pass | Minimum ticks | Separation at 1 tile per 4 ticks |
|---|---|---|---|
| Squad, 10 | 2 | 2 | 0.5 tiles |
| Company, 100 | 13 | 13 | 3 tiles |
| Battalion, 500 | 63 | 63 | 16 tiles |
| Regiment, 1,000 | 125 | 125 | **31 tiles** |

The separation grows **linearly** with the member count. It is not a
random walk and it does not settle.

**A rally bias cannot correct this.** The rear member is not failing to
move toward the centroid. It is being rejected by the capacity rule. An
attraction term adds nothing, because the member already wants the tile it
cannot enter. Adding a stronger attraction term only makes the rear press
harder against a full tile, which wastes ticks and raises the density
penalty for everyone.

This is why the correction must be a **hold gate** and not a force. Section
3.6 specifies it.

### 3.5 Drift source 4. Divergent flow tiles

The flow tile cache is keyed on the chunk and the exit portal.[^2] Two
members in different chunks read different cached tiles. Both tiles reach
the same goal, but they may route around one obstacle on opposite sides.

This produces a **split**, not a drift. Two coherent halves travel and then
rejoin at the goal. The magnitude is bounded by the obstacle width, which
is at most one chunk of 32 tiles.

**Verdict: acceptable, and the hold gate handles the worst case.** A split
that reunites is a correct behaviour for a real column that meets a lake. A
split that does not reunite means one half found no route, which is a
routing failure and not a cohesion failure.

### 3.6 The recommendation. A hold gate, then a small rally bias

**Adopt the hold gate. It is the term that matters.**

The formation holds one `rear_cost` value: the largest flow cost of any
member, which is the distance-to-goal of the most delayed member. Compute
it in the same reduce that computes the centroid. A member whose own flow
cost is below `rear_cost - LEASH` does not accumulate `progress` this tick.

```
// One extra comparison inside the existing movement map kernel.
let f = formations[unit.formation];
if f.flags & HOLD != 0 && flow_cost[unit.tile] + LEASH < f.rear_cost {
    // Do not add speed to progress. The member waits.
} else {
    progress += speed;
}
```

The cost is one gather, already needed for the rally bias, plus one
comparison. **The hold gate converts drift into waiting**, which is the
behaviour that a real column has. It corrects drift source 3 and the worst
case of drift sources 1 and 5. It cannot deadlock, because `rear_cost`
falls whenever the rear member moves, and the rear member is never gated
against itself.

Set `LEASH` per formation kind. A marching column takes a large leash, near
32 tiles. A formation in line takes a small leash, near 4 tiles.

**Adopt the rally bias as a tie-break only.** Add one term to the existing
local choice.

```
score[n] = flow_cost[n] + W_DENSITY * density[n] + W_RALLY * hex_dist(n, centroid)
```

Set `W_RALLY = 1` when the flow cost is on a scale of 256 for each tile.
The bias then changes the choice only when two neighbours have flow costs
within one unit of each other. That is the whole intent. **The rally bias
must never overcome the flow field**, because that is the failure that the
movement report identified in boids.[^2] A bias that only breaks ties
cannot produce it.

Suppress the rally bias when the unit's tile has fewer than three passable
neighbours. That covers a bridge, a mountain pass and a ford. Without the
suppression the bias pulls members sideways into impassable tiles and
wastes ticks. The test is one population count on the passability bitplane,
which the movement kernel already loads.

### 3.7 The centroid, computed deterministically

The centroid is a reduce over the formation's compressed sparse row run.

```
// Per formation. The CSR run is sorted by unit index.
let mut sq: i64 = 0; let mut sr: i64 = 0; let mut n: i64 = 0;
for u in csr_run(f) {
    let (q, r) = axial_of(units.tile[u]);
    sq += q as i64; sr += r as i64; n += 1;
}
// Truncate toward negative infinity. A fixed rule, not the platform's.
let cq = sq.div_euclid(n); let cr = sr.div_euclid(n);
```

Three properties give determinism. Integer addition is associative, so any
summation order gives the same total. The compressed sparse row run is
sorted by unit index, so the order is fixed in any case. The division uses
`div_euclid`, which is a stated rounding rule and not the platform default.

An axial coordinate of a 4,096 by 4,096 world fits in an `i16`. One million
such values sum to at most `4096 * 10^6`, which is under `2^42`. An `i64`
accumulator has wide margins.

The centroid is one value for each formation and one gather for each
member, exactly as the lead proposed.

### 3.8 The three failure cases

**Terrain that splits a column.** The flow field routes both halves to the
goal. The hold gate stops the leading half when the lagging half exceeds
the leash. Members do not need to know that a split happened.

**A chokepoint under a tile capacity of 8.** Section 3.4 gives the
arithmetic. The hold gate makes the head wait. The formation passes as a
unit at a cost of `ceil(strength / 8)` ticks. **This is a game design
consequence and it should be visible to the player.** A 1,000-soldier
regiment takes 125 ticks to cross a bridge, which is 12.5 seconds at 10 Hz.

**Water or a bridge.** The flow field already handles the routing, because
an impassable tile has no finite flow cost. The rally bias is suppressed on
the bridge by the three-neighbour rule. The hold gate keeps the head of the
column on the far bank until the tail arrives.

### 3.9 Cost

| Step | Kernel | Cost at 300,000 movers |
|---|---|---|
| Centroid and rear cost, reduce over members | reduce | 0.10 core-ms |
| Read the centroid for each member | gather, 80 KB table, L2-resident | 0.15 core-ms |
| Hold gate comparison | map | included above |
| Rally bias term in the score | map | included above |
| **Total** | | **0.25 core-ms** |

The centroid table for 10,000 formations is 80 KB, which is resident in the
level 2 cache of a Graviton core. The gather is therefore a cache hit.

---

## 4. Coordinated multi-cell actions

### 4.1 The state machine

Six states cover the behaviours that the design needs.

| State | Meaning | Movement | Camp footprint |
|---|---|---|---|
| `Marching` | Moving to a destination in column | Yes, hold gate on | None |
| `Holding` | Formed and stationary | No | None |
| `InLine` | Formed for combat, small leash | Yes, leash 4 | None |
| `Camped` | Occupying a camp site | No | Yes |
| `Foraging` | Dispersed to gather, large leash | Yes, leash 64, rally off | None |
| `Besieging` | Static around a target site | No | Yes, a ring |

The state is one `u8`. Add it to the existing formation row in place of two
padding bytes, so the row stays at 24 bytes and the table stays at 240 KB
for 10,000 formations.[^3]

Each state selects a row in a small constant table: the leash, the rally
weight, whether the hold gate applies, and whether members may leave the
footprint. The table is 6 rows of 8 bytes. It is a compile-time constant.

### 4.2 How a member reads the state

A member reads the state by one gather through its `formation` column into
the 240 KB formation table. At one million members this table exceeds the
level 2 cache, but the members are sorted by tile, and members of one
formation are spatially adjacent, so the access is highly clustered.
Measured as an effective cache hit, the cost is about **0.05 core-ms** for
300,000 movers.

**A member holds no copy of the state.** This is the central design choice
of this section and it removes the whole atomicity problem.

### 4.3 A transition is atomic on the row and lazy on the members

**A state transition writes one byte.** It is a scatter into the formation
table. There is no tick in which two members read different states, because
no member holds a copy, and the write lands in the apply phase before any
member reads it in the next execute phase.

**Do not iterate the members on a transition.** A transition that touched
one million rows would cost more than the whole movement step, and it would
gain nothing.

### 4.4 What happens to a member that cannot comply

A member cannot refuse a state that it does not store. Compliance is
therefore not a property of the member. It is **progress** toward the
state, measured by the formation.

The formation holds `strength_present`: the count of members inside the
target footprint, or within the leash of the centroid. Compute it in the
same reduce that computes the centroid, at no extra pass.

- A member that is out of position has simply not arrived. It keeps
  marching toward the camp anchor.
- A member in combat has its own intent, which the individual agency layer
  selects.[^6] It does not arrive until the combat ends.
- A member behind a chokepoint arrives late by the arithmetic of section
  3.4.

None of these is an error state. None needs a per-member flag.

### 4.5 Make camp, and how it produces a site

The transition to `Camped` does two things and they happen at different
times.

**At the transition.** Write the state byte. Write `camp_anchor`, which is
the current centroid tile. Nothing else happens.

**At the materialisation.** On each formation tick, test
`strength_present * 65536 / strength >= CAMP_THRESHOLD`. Take
`CAMP_THRESHOLD = 39322`, which is 60% in Q16.16. When the test passes,
create the site.

The site's tile count is `ceil(strength / 8)`, which is 13 tiles for a
100-soldier company. The footprint is the hex spiral from `camp_anchor` by
the rule of section 2.8. The site row's `formation` field points back to
the formation, and the formation's `flags` gains a `HAS_SITE` bit. The site
identifier lives in the formation row's `l1_mask` field, which the camped
state does not otherwise use.

A camp that is struck clears the bits and retires the site row. The tiles
return to the free state at the next structure barrier.

### 4.6 The ordering rule for a transition across many chunks

The transition itself touches one row, so it needs no ordering rule.

The **materialisation** scatters into the site bitplane across many chunks,
and two formations may camp on the same tile in the same tick. Order it by
the key `(target_tile, camp_anchor, formation_id)` ascending. Sort the
claims with the stable radix sort. Each target tile owns one contiguous
segment. Admit the first claim in each segment.

This is the movement admission rule with a different key.[^2] It reuses the
kernel, it uses disjoint outputs, and it needs no atomic operation. A
rejected formation takes the next tile in its spiral order and retries in
the same tick, bounded by a fixed retry count of 4. After 4 rejections the
formation stays in `Camped` without a site and retries next tick.

At 10,000 formations and a low transition rate the claim count is small.
The sort cost is under **0.01 ms** and it runs at the structure barrier,
not every tick.

---

## 5. State sharing, tested and rejected

### 5.1 The owner's observation

The owner observes that members in a formation likely feed and rest at a
similar rate, and that a member who breaks off becomes unique. That
suggests copy-on-write sharing: one need vector for each formation, split
into a per-member vector on divergence.

### 5.2 The storage arithmetic

A need is an `i32` in Q16.16. There are four needs: sustenance, water, rest
and shelter.[^7] So a need vector is 16 bytes, not 6 bytes.

| Case | Size |
|---|---|
| Per unit, 1,000,000 units | **16.0 MB** |
| Per formation, 10,000 formations | **0.16 MB** |
| Saving | 15.8 MB |

The world is about 134 MB of tile state, plus 16.8 MB for the density
array, plus 12 MB for the formation index, plus 4.4 MB for the site
structures. Call the total about 200 MB. **The saving is 7.9%.** That is
larger than the lead's estimate, because a need is 4 bytes and not 1 byte,
but it is still not decisive.

### 5.3 The compute arithmetic

The decay kernel is one map over four `i32` lanes.

```
need[i] = (need[i] - rate).max(0);
```

Traffic is 16 bytes read plus 16 bytes written for each unit, which is
32 MB for each invocation at one million units. Graviton sustains roughly
20 GB/s for each core on a streaming kernel, so one core takes about
1.6 core-ms. Across 12 cores the memory system limits it, so take 0.3 to
1.6 wall-ms for each invocation.

**The needs kernel does not run every tick.** The needs report stages it on
a period of 10 ticks and multiplies the rate by 10 at bake time.[^7] So the
amortised cost is **0.03 to 0.16 wall-ms for each tick**.

At a 10 Hz tick the frame budget is 100 ms. At 30 Hz it is 33 ms. **The
saving is 0.05% to 0.5% of the frame.** The lead's estimate of 2 ms assumed
a per-tick kernel. The staged cadence makes the real saving ten times
smaller.

### 5.4 The correctness argument, which is decisive

The decay rate is genuinely identical for identical unit types. It is a
bake-time constant for each unit type. So decay alone could be shared.

**Satisfaction cannot be shared.** The satisfaction kernel is:

```
gain = ((paid[i] * quality[i]) >> 16);
need[i] = (need[i] + gain).min(65536);
```

`paid[i]` comes from the conserving `transfer` verb, which uses
largest-remainder apportionment.[^8] Largest-remainder apportionment exists
precisely to give different members different integer amounts, so that the
total is exact. Two members of one formation therefore receive different
payments whenever the total does not divide evenly.

**So the members diverge after the first meal.** A shared value is not
approximately right after that. It is wrong.

A shared baseline plus a per-member delta stores the same 16 bytes for each
member, plus 16 bytes for each formation, plus one addition on every read.
That is per-entity state with extra indirection and extra cost.

You could share the decay half and keep the satisfaction half per member.
But decay is the cheap half, and it is a map over a constant. Satisfaction
is the expensive half, and it needs a gather and a multiply. Sharing saves
the cheap half only, and it adds a materialisation path.

### 5.5 Verdict

**Reject copy-on-write need sharing. Do not build it now.**

Three changes would reverse the verdict, and only one of them makes sharing
correct.

1. The unit count rises above about 10 million. Then the need state is
   160 MB, which is a real fraction of memory. This makes sharing
   **attractive**, not correct.
2. The needs kernel moves to every tick and grows past 8 needs. This makes
   sharing **attractive**, not correct.
3. The apportionment rule is replaced by an equal-share rule that provably
   gives every member the same integer. **This is the only change that
   makes sharing correct.** It also gives up exact conservation, which the
   project has already decided to keep.

### 5.6 The rule that generalises this

**Share what is configured. Never share what is accumulated.**

Homogeneity already pays where it is real. The effective-stat memoisation
reaches a 99.5% to 99.98% hit rate because formation members are configured
identically.[^5] The group weight vector serves ten thousand members from
one decision.[^6] Both share a **configuration**, which is exactly equal by
construction.

A need is an **accumulated** value. It integrates a payment history that
differs for every member by design. It is not shareable, and no amount of
similarity makes it shareable, because sharing needs equality and not
similarity.

Write this rule into the record. It settles the same question for morale,
experience, fatigue and inventory without a separate analysis for each.

---

## 6. Divergence

Divergence matters regardless of the sharing verdict. A unit that leaves a
formation changes behaviour, and each change needs an explicit rule.

### 6.1 What triggers a departure

| Trigger | Rule | Kernel |
|---|---|---|
| An order | A command writes `formation` to a new value or to `u32::MAX`. | scatter |
| Straggling | The member exceeds `LEASH_BREAK` from the centroid for `K` consecutive formation ticks. | map, then a bitset scan |
| Routing | Morale falls below a threshold. Members leave at a fixed rate for each tick. | map, then a bitset scan |
| Commander death | **Does not dissolve the formation.** | see below |
| Death | The row is gone. The next reverse index rebuild does not see it. | none |

**Set `K` larger than the worst chokepoint delay.** Section 3.4 gives 125
ticks for a 1,000-soldier regiment at a one-tile bridge. Take `K = 256`.
Without this, a bridge dissolves every large formation that crosses it,
which is a defect that would be hard to diagnose.

Take `LEASH_BREAK` at four times the state's `LEASH`. Detect it with the
same centroid gather that the rally bias already performs, so detection
costs one comparison and no extra memory traffic.

Use the dense-bitset plus sparse-scan pattern for both straggling and
routing.[^7] A dense branchless map writes one predicate bit for each
member. A barrier follows. A sparse ascending scan then runs the departure
handler for each set bit. The ascending scan order gives determinism.

**Commander death.** The office cascade reassigns the commander from the
chain of command.[^3] Only when the cascade finds no successor does the
formation move to a routed condition, and then members leave individually
by the routing rule. This keeps a large army from dissolving when one
officer dies.

### 6.2 Loss 1. The group weight vector

A departed unit reverts to its own need vector for intent selection.

**Reserve index 0 of the group weight table as the individual default
row.** Then the rule needs no branch.

```
// Reads the weight row. u32::MAX maps to 0 by construction.
let w = weights[if unit.formation == u32::MAX { 0 } else { unit.formation }];
```

Better still, define `u32::MAX` to select row 0 in the index function
itself, so the gather is unconditional. A branchless gather keeps the map
kernel vectorised on NEON.

### 6.3 Loss 2. The shared flow field. This loss is almost nothing

**The task framing overstates this loss. State the correction plainly.**

The flow tile cache is keyed on `(chunk_id, exit_portal_id)`.[^2] It is not
keyed on the formation. A departed unit that keeps the same destination
reads exactly the same cached flow tiles as before, at exactly the same
cost. **Nothing is lost.**

The unit only routes independently when its **destination** changes. Then
it needs a new plan, which is a portal graph search plus a flow tile build.
The flow tile build costs 5 to 20 microseconds and it enters the shared
cache, so the second unit with the same plan pays nothing.

The real cost of departure is therefore **plan diversity**, not per-unit
routing. A formation that dissolves into 1,000 units with 1,000 different
destinations lowers the flow tile cache hit rate. The movement report
already flags cache hit rate as an open risk.[^2] Departure adds to that
risk and does not create a new one.

**Rule: cap plan diversity, not departures.** When the live plan count
exceeds the cache capacity, evict by least recent use and let the evicted
units reuse a coarser level 1 field until a slot frees.

### 6.4 Loss 3. Supply access

The economy pools commodities at the settlement level. A formation draws
from a settlement through its own supply arc.

**Rule: a unit with `formation == u32::MAX` draws only from the settlement
that owns the level 1 cell containing its tile, and only when that
settlement's faction matches the unit's faction. Otherwise it draws
nothing, and its needs decay.**

The lookup is one gather through the existing level 1 index into the
settlement table. It needs no new structure. The faction test is one
comparison.

This gives the behaviour the design wants. A deserter inside friendly
territory survives. A deserter in the field starves. The needs report gives
sustenance a 3-day emptying time at 36 units for each tick, so a unit with
no supply reaches an unmet threshold in about 3 game days.[^7]

### 6.5 Rejoining

**Rejoining is allowed. It is cheap and it needs one restriction.**

The cost is one scatter into the `formation` column, plus the reverse index
rebuild. The rebuild is already scheduled at the structure barrier and
costs about 5 ms at one million units, run twelve times for each simulated
year.[^3] A rejoin adds nothing to that.

**The restriction: a unit may only join a formation whose bounding level 1
range contains the unit's tile.** The test is
`f.l1_lo <= l1_of(unit.tile) && l1_of(unit.tile) <= f.l1_hi`.

Without the restriction, a rejoin at any distance would transfer supply
access instantly across the map. That would turn the formation column into
a teleport for logistics. The restriction is one comparison and it closes
the exploit.

A unit that rejoins loses nothing that it kept. Its needs, morale and
experience are its own and they persist through the departure and the
rejoin. This is a further consequence of rejecting shared state: with no
shared state, there is no materialisation on departure and no merge on
rejoin. **The rejection of sharing makes divergence free.**

---

## 7. Cost and storage summary

### 7.1 Per tick

| Item | Kernel | Cost |
|---|---|---|
| Centroid and rear cost reduce, 300,000 movers | reduce | 0.10 core-ms |
| Centroid gather for the rally bias | gather | 0.15 core-ms |
| Formation state gather | gather | 0.05 core-ms |
| Hold gate and rally bias terms | map | included above |
| Site bit test for units on sited tiles | gather | under 0.05 core-ms |
| **Total for each tick** | | **0.35 core-ms** |
| **Wall time on 12 cores** | | **under 0.03 ms** |

### 7.2 At the structure barrier, not every tick

| Item | Cost | Cadence |
|---|---|---|
| Site reverse index rebuild, 340,000 entries | 0.30 ms | Twelve times for each simulated year |
| Camp claim sort and admission | under 0.01 ms | On a transition only |
| Level 1 site count update, signed delta | under 0.01 ms | On a site change only |
| **Amortised for each tick** | **under 0.01 ms** | |

### 7.3 Storage

| Structure | Size |
|---|---|
| `sited` bitplane, ground layer | 2.09 MB |
| `sited` bitplane, overlay layer | 2.09 MB |
| Block prefix counts, both layers | 0.52 MB |
| `site_id` payload, both layers | 1.63 MB |
| Site rows, 20,000 at 32 bytes | 0.64 MB |
| Level 1 site counts and kind counts | 0.65 MB |
| Formation state byte | 0, reuses padding |
| Centroid and rear cost table, 10,000 formations | 0.16 MB |
| **Total** | **7.78 MB** |

One layer instead of two brings the total to **4.6 MB**. Start with one
layer and add the overlay when a wall on a camp tile is needed.

### 7.4 The row for the running budget

| Subsystem | Cost |
|---|---|
| Group spatial dynamics | 0.35 core-ms, under 0.03 wall-ms on 12 cores |

This is smaller than every existing line. It does not change the frame
budget conclusion. **Every figure here is derived, not measured.** Benchmark
on the target platform before you rely on any of it.

---

## 8. Determinism rules

1. **The centroid uses `div_euclid`.** State the rounding rule. Do not use
   the platform's default division rounding for a negative numerator.
2. **The spiral ring order is a compile-time table.** It is not derived from
   a run-time sort.
3. **Camp tile claims sort by `(target_tile, camp_anchor, formation_id)`.**
   The key is total, so the sort is a total order and stability does not
   matter for correctness.
4. **The departure scan is an ascending sparse scan over a bitset.** The
   order is the unit index order, which is stable.
5. **The site reverse index is built by a counting sort**, which is the same
   device as the formation reverse index. A counting sort has no comparator
   and therefore no comparator inconsistency.
6. **The site row is `Pod` with declared padding.** So is the formation
   state byte, which sits in existing declared padding.
7. **No step in this document uses an atomic operation.** Every scatter is
   disjoint after a sort, or it is a single-row write.
8. **The rally bias and the hold gate read the previous tick's centroid.**
   Compute the centroid in the reduce that follows the movement apply
   phase. Do not read a centroid that is being written.

---

## 9. What this document rejects

| Rejected | Reason |
|---|---|
| A footprint mask or a bounding shape | A shape has an origin and needs a second coordinate system when it crosses a storage block boundary. A tile set has no geometry, so no geometry can go wrong. |
| A region-tagged aggregate as the primary site form | A level 1 cell is 256 tiles. A 13-tile camp is 5% of one cell. It answers "am I inside" wrongly. |
| A dense `site: u32` tile column | 66.8 MB, which is half of the world budget, for a structure that 98% of tiles do not use. |
| A site identifier as a pyramid field | Cardinality 20,000. No aggregate of it is meaningful. |
| A level 1 site kind **mask** | A bitwise OR has no inverse, so removing the last site of a kind forces a recompute. A count is a group. |
| A per-agent cohesion force | It duplicates the flow field and it opposes it, which is the failure that boids showed. |
| A strong rally bias | It cannot correct chokepoint drift, and it makes the rear press against a full tile. |
| Copy-on-write need sharing | 15.8 MB and 0.03 to 0.16 wall-ms saved. Exact apportionment makes the values diverge after the first meal, so a shared value is wrong and not merely stale. |
| A shared baseline plus a per-member delta | It stores the same per-member bytes, plus a formation row, plus an addition on every read. |
| A per-member copy of the formation state | It makes a transition a one-million-row write for no gain. A gather is free. |
| Iterating members on a state transition | Same reason. The transition writes one byte. |
| Dissolving a formation when the commander dies | The office cascade already reassigns the commander. |
| An unrestricted rejoin | It would transfer supply access instantly across the map. |

---

## 10. Ready-to-apply decision block

**This block is ready to apply. It uses D130 to D149. It adds no decision
that conflicts with an existing one. It depends on the tile capacity of 8,
the tile-discrete position rule, and the formation ownership column.**

---

#### D130. A multi-tile entity is a set of tiles, not a shape

A site is a camp, a city, a fortress, a wall or a bridge. **A site is a set
of tiles.** It has no origin, no extent, no rotation and no mask.

The reason is the storage block. Every tile field is stored in blocks of
256 tiles. A shape that crosses a block boundary needs a bit shift across
word boundaries or a second coordinate system. A tile set crossing a block
boundary is several disjoint runs in the same array, and it needs **no
special case at all**.

Reject a bounding shape. Reject a footprint mask. Reject a level 1 region
tag as the primary form; a level 1 cell is 256 tiles and a 13-tile camp is
5% of one cell, so a region tag answers "am I inside a camp" wrongly.

#### D131. Store the site owner as a sparse rank-select column

A dense `site: u32` tile column costs 66.8 MB, which is half of the world
budget, for a structure that 98% of tiles do not use. Reject it.

Use the rank-select structure that D-block storage already defines for
sparse tile payloads.

| Structure | Type | Size |
|---|---|---|
| `sited` bitplane | 1 bit for each tile | 2.09 MB |
| Block prefix count | `u32` for each 256-tile block | 0.26 MB |
| `site_id` payload | `u32` for each set bit, at 2% occupancy | 1.36 MB |
| Site rows | 32 bytes, 20,000 rows | 0.64 MB |
| **Total, one layer** | | **4.35 MB** |

"Am I inside a site" reads one bit. For 98% of tiles the query ends there,
at one load. For the remaining 2% it costs a prefix read, a population
count and a payload read.

Build the site reverse index with a counting sort over the payload, which
is the same device as the formation reverse index. 340,000 entries cost
about 0.30 ms for each rebuild, at the structure barrier only.

#### D132. Footprints do not overlap inside a layer. Use two layers

The payload is single-valued, so a tile belongs to at most one site of one
layer. **Take two layers: ground and overlay.** A wall on a camp tile is
two layers. Two layers cost 7.8 MB in total.

**Do not make the layer count data-driven.** Each layer is a separate
memory-resident bitplane with its own prefix array and payload.

Partial destruction is exact. Clear the bit. Decrement `tile_count`. A camp
with a hole in it is a correct state and needs no new field. Retire the
site row when `tile_count` reaches zero.

#### D133. A site identifier is not a pyramid field. Summarise with counts

A site identifier has a cardinality of 20,000, so no aggregate of it is
meaningful. **Do not put it in the pyramid.**

Add two derived level 1 fields instead.

| Field | Type | Size at 65,536 cells |
|---|---|---|
| `site_tiles` | `u16` count of sited tiles | 0.13 MB |
| `site_kind_count[k]` | `u8` for each of up to 8 kinds | 0.52 MB |

**Take counts, not a bitwise OR mask.** A count is a group under addition,
so an incremental update applies a signed delta. A mask has no inverse, so
removing the last site of a kind from a cell forces a full recompute of
that cell's children. This is the same failure that the faction mask has.

The site row carries `l1_lo` and `l1_hi`, a bounding level 1 cell range.
That gives the command scheduler a region scope, exactly as the formation
bounding mask does. Two destroy commands on disjoint sites run in parallel.

#### D134. A footprint is placed by a hex spiral, and conflicts sort

A footprint is a fixed function of the anchor tile, the required tile
count, the terrain and the current occupancy.

Take tiles in **hex spiral ring order** from the anchor. The order is a
compile-time table. Skip an impassable tile. Skip a tile whose bit is
already set in this layer. Stop at the count.

A company of 100 soldiers at a tile capacity of 8 needs
`ceil(100 / 8) = 13` tiles.

Two sites may claim the same tile in one tick. Sort the claims by the key
`(target_tile, anchor, site_id)` with the stable radix sort. Each target
tile owns one contiguous segment. Admit the first claim in each segment.
Reject the rest, and let a rejected site take the next tile in its spiral,
bounded by 4 retries. **This is the movement admission rule with a
different key. It needs no atomic operation.**

#### D135. Cohesion under a shared flow field is mostly free. Four terms, quantified

Members that follow the same flow field toward the same destination stay
together without any attraction force. **Confirm the claim, and record the
two terms that it does not cover.**

| Drift source | Growth | Magnitude | Correction needed |
|---|---|---|---|
| Speed variance | Linear in distance | Zero when members share a unit type and upgrades | None. Configuration is identical, which is the same property that gives the effective-stat cache a 99.5% hit rate |
| Terrain variance | Square root of distance | 2.1 tiles at 100 tiles, 4.2 tiles at 400 tiles | **None.** It is below the 13-tile camp footprint |
| **Chokepoint** | **Linear in member count** | **31 tiles for 1,000 soldiers at a one-tile bridge** | **A hold gate. See D137** |
| Divergent flow tiles | Bounded by the obstacle width | Up to 32 tiles, and the halves rejoin | The hold gate covers the worst case |

The terrain figure uses a random walk of the accumulated step cost
difference, with a per-step standard deviation of 0.15 to 0.30 of the mean.

**Reject a per-agent cohesion force.** It duplicates what the field
supplies and it opposes it.

#### D136. The rally bias is a tie-break, and it is suppressed at a chokepoint

Add one term to the local movement choice.

```
score[n] = flow_cost[n] + W_DENSITY * density[n] + W_RALLY * hex_dist(n, centroid)
```

**Take `W_RALLY = 1` when the flow cost is on a scale of 256 for each
tile.** The bias then changes the choice only when two neighbours have flow
costs within one unit. **The rally bias must never overcome the flow
field.**

**Suppress the bias when the unit's tile has fewer than three passable
neighbours.** That covers a bridge, a pass and a ford. Without it, the bias
pulls members sideways into impassable tiles. The test is one population
count on the passability bitplane, which the movement kernel already loads.

Compute the centroid as a reduce over the formation's compressed sparse row
run. Accumulate the axial coordinates in `i64`. Divide with `div_euclid`,
which is a stated rounding rule. Integer addition is associative, so any
summation order gives the same total.

One centroid for each formation. One gather for each member. The table is
80 KB at 10,000 formations, so the gather hits the level 2 cache.

#### D137. The hold gate, not the rally bias, is what holds a formation together

A chokepoint of one tile passes at most 8 units for each tick. A
1,000-soldier regiment takes 125 ticks to pass, which is 31 tiles of
separation at a march speed of one tile for each four ticks. **A rally bias
cannot correct this**, because the rear member already wants the tile that
the capacity rule denies it.

The formation holds `rear_cost`: the largest flow cost of any member.
Compute it in the same reduce as the centroid.

```
if f.flags & HOLD != 0 && flow_cost[unit.tile] + LEASH < f.rear_cost {
    // The member waits. Do not add speed to progress.
}
```

The gate converts drift into waiting. It cannot deadlock, because
`rear_cost` falls whenever the rear member moves, and the rear member is
never gated against itself.

Set `LEASH` for each formation state. Marching takes 32 tiles. In line
takes 4 tiles. Foraging takes 64 tiles and turns the rally bias off.

**A player will see this.** A large formation crossing a bridge waits. That
is a game design consequence and it should be visible.

#### D138. The formation state is one byte, and no member holds a copy

Six states: `Marching`, `Holding`, `InLine`, `Camped`, `Foraging`,
`Besieging`. The state is one `u8` in the existing formation row padding,
so the row stays at 24 bytes.

Each state selects a row in a constant table of 6 rows by 8 bytes: the
leash, the rally weight, whether the hold gate applies, and whether members
may leave the footprint.

**A member reads the state by one gather through its `formation` column.**
A member holds no copy. This removes the atomicity problem completely.

**A state transition writes one byte.** It is atomic in the only sense that
matters: no tick exists in which two members read different states. **Do
not iterate the members on a transition.** A transition that touched one
million rows would cost more than the whole movement step and gain nothing.

#### D139. Compliance is a formation-level count, not a per-member flag

A member cannot refuse a state that it does not store. Compliance is
progress toward the state, and the formation measures it.

The formation holds `strength_present`: the count of members inside the
target footprint or within the leash of the centroid. Compute it in the
same reduce as the centroid, at no extra pass.

A member out of position has not arrived. A member in combat has its own
intent. A member behind a chokepoint arrives late by D137's arithmetic.
**None of these is an error state and none needs a flag.**

#### D140. Make camp materialises on a threshold, not on the transition

The transition to `Camped` writes the state byte and the `camp_anchor`,
which is the current centroid tile. Nothing else happens.

On each formation tick, test
`strength_present * 65536 / strength >= CAMP_THRESHOLD`. Take
`CAMP_THRESHOLD = 39322`, which is 60% in Q16.16. When the test passes,
create a site by D134 with `ceil(strength / 8)` tiles.

The site row's `formation` field points back to the formation. Striking
camp clears the bits and retires the site row.

#### D141. Reject copy-on-write need state sharing. Do not build it now

| Case | Size |
|---|---|
| Four needs at `i32` Q16.16, 1,000,000 units | 16.0 MB |
| Shared over 10,000 formations | 0.16 MB |
| Saving against a world of about 200 MB | 15.8 MB, or 7.9% |

The needs decay kernel moves 32 MB for each invocation, which is 0.3 to
1.6 wall-ms. It runs on a period of 10 ticks, so the amortised cost is
**0.03 to 0.16 wall-ms for each tick**. Against a 33 to 100 ms frame that
is **0.05% to 0.5%**.

**The correctness argument is decisive.** The decay rate is a bake-time
constant for each unit type, so it is genuinely identical. But satisfaction
uses `paid[i]`, which comes from largest-remainder apportionment.
Apportionment exists to give members **different** integers so that the
total is exact. **Members therefore diverge after the first meal.** A
shared value is wrong, not merely stale.

A shared baseline plus a per-member delta stores the same per-member bytes,
adds a formation row and adds an addition on every read. Reject it.

Three changes would reverse this decision. Only the third makes sharing
correct.

1. The unit count rises above about 10 million.
2. The needs kernel moves to every tick and grows past 8 needs.
3. Apportionment is replaced by an equal-share rule that provably gives
   every member the same integer. This gives up exact conservation.

#### D142. Share what is configured. Never share what is accumulated

This is the general rule that D141 is one case of.

A **configuration** is exactly equal by construction. The effective-stat
memoisation shares one, and reaches a 99.5% to 99.98% hit rate. The group
weight vector shares one, and serves ten thousand members from one
decision. Both are correct.

An **accumulated** value integrates a history that differs for every member
by design. A need, a morale value, an experience value, a fatigue value and
an inventory are all accumulated. **None of them is shareable, and no
amount of similarity makes one shareable, because sharing needs equality
and not similarity.**

Apply this rule to any future sharing proposal without a separate analysis.

#### D143. Five departure triggers, each with a rule

| Trigger | Rule |
|---|---|
| An order | A command writes the `formation` column. One scatter. |
| Straggling | The member exceeds `LEASH_BREAK` from the centroid for `K` consecutive formation ticks. Take `LEASH_BREAK` at four times the state's `LEASH`. |
| Routing | Morale falls below a threshold. Members leave at a fixed rate for each tick. |
| Commander death | **Does not dissolve the formation.** The office cascade reassigns the commander. Only when no successor exists does the formation rout. |
| Death | The row is gone. The next reverse index rebuild does not see it. |

**Take `K = 256` ticks.** It must exceed the worst chokepoint delay, which
D137 gives as 125 ticks for a 1,000-soldier regiment. A smaller `K` would
dissolve every large formation that crosses a bridge, and that defect would
be hard to diagnose.

Detect straggling with the centroid gather that D136 already performs, so
detection costs one comparison and no extra memory traffic.

Use the dense-bitset plus sparse-scan pattern for straggling and routing. A
branchless map writes one predicate bit for each member. A barrier follows.
An ascending sparse scan runs the handler. The ascending order gives
determinism.

#### D144. A departed unit falls back to group weight row 0

**Reserve index 0 of the group weight table as the individual default
row.** Define the index function so that `u32::MAX` maps to 0. The gather
is then unconditional and the map kernel stays vectorised on NEON.

A departed unit's intent reverts to its own need vector with no branch and
no special case.

#### D145. A departed unit does not lose its route. It loses plan sharing

**The flow tile cache is keyed on the chunk and the exit portal, not on the
formation.** A departed unit that keeps the same destination reads exactly
the same cached flow tiles at exactly the same cost. **Nothing is lost.**

A unit only routes independently when its **destination** changes. That
costs a portal graph search plus a flow tile build of 5 to 20 microseconds,
and the result enters the shared cache.

The real cost of mass departure is **plan diversity**, which lowers the
flow tile cache hit rate. **Cap plan diversity, not departures.** When the
live plan count exceeds the cache capacity, evict by least recent use, and
let the evicted units follow the coarser level 1 field until a slot frees.

#### D146. A unit outside a formation draws supply locally or not at all

**A unit with `formation == u32::MAX` draws only from the settlement that
owns the level 1 cell containing its tile, and only when that settlement's
faction matches the unit's faction. Otherwise it draws nothing and its
needs decay.**

The lookup is one gather through the existing level 1 index. The faction
test is one comparison. No new structure is needed.

A deserter inside friendly territory survives. A deserter in the field
reaches an unmet sustenance threshold in about 3 game days, at a decay rate
of 36 units for each tick.

#### D147. Rejoining is allowed, and it is bounded by the formation's region

The cost is one scatter into the `formation` column. The reverse index
rebuild is already scheduled at the structure barrier.

**A unit may only join a formation whose bounding level 1 range contains
the unit's tile.** The test is
`f.l1_lo <= l1_of(unit.tile) && l1_of(unit.tile) <= f.l1_hi`.

Without the restriction, a rejoin at any distance transfers supply access
instantly across the map. That turns the formation column into a logistics
teleport. The restriction is one comparison.

A rejoining unit keeps its needs, morale and experience. **Because D141
rejects shared state, there is no materialisation on departure and no merge
on rejoin. Rejecting sharing makes divergence free.**

#### D148. The determinism rules for this subsystem

1. The centroid divides with `div_euclid`. State the rounding rule.
2. The hex spiral ring order is a compile-time table.
3. Camp tile claims sort by `(target_tile, camp_anchor, formation_id)`,
   which is a total key.
4. The departure scan is an ascending sparse scan over a bitset.
5. The site reverse index uses a counting sort. It has no comparator.
6. The site row is `Pod` with declared padding.
7. **No step in this subsystem uses an atomic operation.** Every scatter is
   disjoint after a sort, or it is a single-row write.
8. The rally bias and the hold gate read the **previous** tick's centroid.
   Compute the centroid in the reduce after the movement apply phase.

#### D149. The cost row for the running budget

| Item | Cost for each tick |
|---|---|
| Centroid and rear cost reduce, 300,000 movers | 0.10 core-ms |
| Centroid gather | 0.15 core-ms |
| Formation state gather | 0.05 core-ms |
| Site bit test | under 0.05 core-ms |
| Amortised structure barrier work | under 0.01 ms |
| **Total** | **0.35 core-ms, under 0.03 wall-ms on 12 cores** |

Storage: **4.6 MB at one layer, 7.8 MB at two layers.**

Every figure here is derived, not measured. Benchmark on the target
platform before you rely on any of it.

---

## 11. Open questions

**OQ80. How many sites exist at the target scale?** Every storage figure in
section 2 scales with it. This document assumes 20,000 sites at an average
of 17 tiles each, which is 2.0% tile occupancy. If sites cover 10% of
tiles, the payload grows to 6.7 MB and the total to 11 MB. **The owner must
give a figure.**

**OQ81. One layer or two?** Two layers cost 3.4 MB more. The question is
whether a wall may cross a camp tile, and whether a bridge may cross a
river tile that also carries a site. **This is a game design question.**

**OQ82. What is `W_RALLY`?** This document recommends 1 against a flow cost
scale of 256, so the bias only breaks ties. It is a tuning value, not a
structural one, but it must be an integer constant and it must appear in
the replay header if it is ever made configurable.

**OQ83. What are `LEASH` and `LEASH_BREAK` for each formation state?** This
document recommends 32 tiles marching, 4 in line, 64 foraging, and
`LEASH_BREAK` at four times `LEASH`. These are game feel values. Measure
them against the milestone movement demonstration.

**OQ84. Is a 125-tick bridge crossing acceptable game feel?** A
1,000-soldier regiment at a one-tile bridge waits 12.5 seconds at 10 Hz.
The arithmetic is certain. Whether a player accepts it is not. **This
cannot be judged from arithmetic. Build the demonstration and look at it.**

**OQ85. What is `CAMP_THRESHOLD`?** This document recommends 60%. A higher
value delays the camp and makes a damaged formation unable to rest. A lower
value lets a scattered formation camp.

**OQ86. Does a besieging formation's ring footprint need its own placement
rule?** The hex spiral of D134 gives a disc, not a ring. A siege ring needs
a different constant table. This document does not specify it.

**OQ87. What is the maximum site kind count?** D133 assumes 8 kinds at one
`u8` counter each. 16 kinds double the level 1 cost to 1.05 MB.

**OQ88. Does the site structure need an event in the log?** Site creation
and site destruction are discontinuous facts, so they belong in the log by
the existing rule. The per-tile bit writes are derived and must not be
logged. Confirm that the replay reconstructs the bitplane from the site
creation events plus the spiral function.

**OQ89. Does a formation's `l1_mask` field carry the camp site identifier
in the `Camped` state?** This document proposes that reuse, because the
camped state does not otherwise need the bounding mask. It saves 4 bytes on
each formation row. It also makes one field mean two things, which is a
maintenance hazard. **Decide before implementation.**

---

## 12. What remains unverified

Every performance figure in this document is derived from a bandwidth
model, not measured. The Graviton figure of about 20 GB/s for each core on
a streaming kernel is an assumption. The research agenda already flags
benchmarking on the target platform as blocking most conclusions, and this
document adds no exception.

The drift arithmetic in section 3.3 uses a random walk model of terrain
cost variance. The model assumes that neighbouring members' step costs are
independent. In practice they are correlated, because members stand close
together on similar terrain. **Correlation reduces the drift, so the
figures in section 3.3 are an upper bound.**

No game implementation claim appears in this document, so the project's
recurring problem of community-wiki-only sources does not apply here. The
one external result used is the floor field cellular automaton, which is a
peer-reviewed publication with measured pedestrian validation.

---

## 13. Controlling the chokepoint rate

### 13.1 What the owner settled

The owner accepts a 12.5-second crossing for a 1,000-soldier regiment at a
one-tile bridge. A chokepoint that matters is desirable, not tedious. The
hold gate and the linear chokepoint term therefore stand without change.

The owner then asks whether the rate can be controlled. This section gives
the levers, the arithmetic for each, and the effect on every kernel.

### 13.2 A correction to the throughput arithmetic. Read this first

**Section 3.4 mixed two different rates. Correct it before you use any
figure from it.**

Throughput through a chokepoint is not the capacity for each tick. Derive
it from the movement rule. A unit that steps onto the chokepoint tile
spends its accumulated progress, because the rule applies
`progress -= step_cost` on a successful step. To leave the chokepoint the
unit must accumulate the cost of the **next** tile. That takes
`dwell = ceil(step_cost_of_exit / speed)` ticks.

In a steady state a full chokepoint of `C` units releases `C / dwell` units
for each tick. The admission rule refills those slots at once. A unit
waiting in the queue accumulates progress while it waits, so the queue is
never the limit.

> **Throughput = capacity / dwell. The dwell is set by the tile beyond the
> chokepoint, not by the chokepoint itself.**

Corrected figures for 1,000 soldiers at a one-tile crossing of capacity 8.

| Dwell | Case | Ticks to pass | Seconds at 10 Hz |
|---|---|---|---|
| 1 | Speed at or above the exit cost. Cavalry on a road. | 125 | 12.5 |
| 2 | Fast infantry. | 250 | 25 |
| 4 | Marching infantry on ordinary ground. | 500 | 50 |
| 8 | A heavy unit onto rough ground. | 1,000 | 100 |

**The 12.5-second figure that the owner approved is the dwell-1 case.**
Marching infantry onto ordinary ground takes 50 seconds. The mechanism that
the owner approved stands. The range is four times wider than section 3.4
stated.

This makes the hold gate more necessary, not less. At dwell 4 the head of
the column would advance 125 tiles while the tail waits 500 ticks.

**The exit tile's terrain cost is therefore itself a lever, and it already
exists in the game.** Section 13.3 ranks it.

### 13.3 The five levers, ranked

| Lever | Effect on throughput | Build cost | Reach for it when |
|---|---|---|---|
| Terrain and road capacity | Multiplies directly | 64 bytes | You want a class of terrain to throttle or to carry |
| Exit tile terrain cost | Divides, through dwell | **Zero. It exists** | You want to tune a rate now, before any new work |
| Crossing width | Multiplies exactly by the tile count | Zero. Level design | One named place feels wrong |
| An explicit dwell floor | Divides | 1 MB and one comparison | A fast heavy unit must still throttle a crossing |
| Tick rate | Multiplies globally | Zero, but hazardous | **Never.** It is a performance knob |

### 13.4 Lever 1: terrain-specific capacity. Verified, with one correction

The entity economy report specifies a flattened movement cost table indexed
as `terrain_cost[unit_type][terrain][road_tier]`, held as `u8`, plus a
dense 2-bit `road_tier` field over the tiles at 4 MiB.[^5]

**Capacity extends that table, but it must drop the unit type dimension.**
The lead expects the same table shape. That is not correct, and the
difference is structural.

Capacity is a property of the tile. The admission rule admits one
contiguous segment of intents into one target tile against one limit.[^2]
If capacity varied by unit type, a segment holding two unit types would
carry two limits, and the admission scan would no longer be one comparison
for each segment. So the shape is:

```
capacity[terrain][road_tier]      // NOT indexed by unit_type
```

Size: 16 terrain kinds times 4 road tiers times one byte is **64 bytes**.
It lives in the existing type table and it stays in the level 1 cache. It
adds **no new structure and no new bitplane**, because the `road_tier`
plane already exists for the cost lookup.

**Does this disturb sort-then-admit? No. But the mechanism is not the one
the lead expects.**

The lead expects the admit step to gather the terrain at no cost, because
the kernel already gathers terrain. That is not correct. The **intent**
step gathers the terrain. The **admit** step is a separate pass over the
sorted intents. A fresh gather in the admit step would read the terrain and
road planes over every touched block. At 300,000 targets spread over about
20,000 blocks of 256 bytes, that is 5.1 MB of traffic, or about
**0.26 core-ms**. That is 1% of the movement budget. It is affordable, and
it is unnecessary.

**The free mechanism is to carry the limit in the intent record.** The
intent step already gathers `terrain[target]` and `road_tier[target]` to
compute the step cost. Have it also read the 64-byte capacity table, which
is a level 1 cache hit, and write the limit into the intent.

The world holds 16.7 million tiles, which needs 24 bits. The intent's
target tile field is a `u32`, so **8 bits are free**.

```
intent.target = (tile & 0x00FF_FFFF) | ((cap as u32) << 24);
```

The intent record does not grow, so the radix sort moves the same bytes.
**The sort key may stay the full 32 bits.** Capacity is a pure function of
the tile, so equal tiles carry equal capacity bits, and the segments are
identical to those a 24-bit sort produces. No mask is needed and no extra
pass is needed. The admit step reads the limit from the first intent of its
segment, which is already in a register.

**Cost of lever 1: zero extra memory traffic, and one level 1 table read in
a kernel that already performs several.**

**The `u8` occupancy array is unaffected. Confirm this explicitly.** The
array counts units. It does not hold a limit. Only the limit becomes
variable. A `u8` saturates at 255, so a capacity of 16 or 32 keeps 8 to 16
times headroom for transient overflow during sort-then-admit.

### 13.5 The camp footprint under a variable capacity

The rule `ceil(strength / 8)` becomes a scan. **This is required for
consistency, not a flavour choice.**

Take a camp of 100 soldiers on terrain of capacity 4. A fixed 13-tile
footprint would hold 52 soldiers. The admission rule would reject the other
48. The footprint rule and the admission rule would then disagree, and the
camp would never fill.

The corrected rule: take tiles in hex spiral order from the anchor.
Accumulate `capacity[terrain][road_tier]` for each accepted tile. Stop when
the running total reaches the formation strength. That is a `scan`, which
is inside the kernel vocabulary. It costs one extra 64-byte table read for
each candidate tile, which is under 40 reads for a company.

**Recommendation: adopt it. A camp on broken ground sprawls.** It is
required for consistency, it costs nothing, and a player can see and
understand it.

**Add a cap.** Set `MAX_CAMP_TILES` to four times
`ceil(strength / max_world_capacity)`. If the scan passes the cap, the camp
does not materialise. The formation stays in the camped state without a
site, and it must move to rest. **A regiment cannot camp in a swamp.** That
is a good mechanic, and it stops capacity-1 terrain from sprawling a
1,000-soldier camp across 1,000 tiles.

### 13.6 Lever 2: a road carries cost and capacity, and they must stay independent

Recommended values on a plains base.

| Tier | Name | Cost multiplier | Capacity |
|---|---|---|---|
| 0 | None | 1.00 | 8 |
| 1 | Track | 0.75 | 8 |
| 2 | Road | 0.50 | 12 |
| 3 | Paved | 0.33 | 16 |

**Scale cost and capacity independently. Do not couple them.** The two
quantities are physically distinct. Cost is surface quality, which is mud
against paving. Capacity is width. A narrow paved mountain road is fast and
low capacity. A wide dirt drove road is slow and high capacity. A coupled
pair cannot express either.

Independence costs nothing, because the two tables are already separate
arrays over a shared index: `terrain_cost[unit_type][terrain][road_tier]`
and `capacity[terrain][road_tier]`. **Coupling them would be extra work,
not less.**

The strategic consequence, quantified. The rate at which a route delivers
soldiers to a front is the product of speed and throughput, and a road
multiplies both.

| Route | Relative speed | Capacity | Relative delivery rate |
|---|---|---|---|
| Unroaded plains | 1.00 | 8 | **1.0** |
| Track | 1.33 | 8 | **1.3** |
| Road | 2.00 | 12 | **3.0** |
| Paved | 3.00 | 16 | **6.0** |

**A paved road delivers soldiers to a front six times faster than open
ground.** That is not a saving in travel time. It changes where a player
can concentrate force, so it changes the geometry of a campaign. It also
gives road destruction a measurable strategic effect, and it gives a
defender a reason to break a bridge rather than hold it.

Both effects come from the existing 2-bit `road_tier` plane at 4 MiB. **One
bitplane, two functions, no new storage.**

### 13.7 Lever 3: the width of a crossing

Throughput scales exactly linearly in the tile count of the crossing front.
`N` parallel tiles of capacity `C` give `N * C / dwell`.

For 1,000 soldiers at capacity 8 and dwell 4:

| Width | Ticks to pass | Seconds at 10 Hz |
|---|---|---|
| 1 | 500 | 50 |
| 2 | 250 | 25 |
| 3 | 167 | 17 |
| 5 | 100 | 10 |

**This is level design, not a parameter.** It tunes one named place without
touching a global table, so it cannot regress the rest of the world. **It
is the safest lever, and it should be the first response to a complaint
about one specific location.**

### 13.8 Lever 4: dwell time, and whether a heavy unit should dwell longer

Dwell already exists, but it is implicit and it is not independent:
`dwell = ceil(step_cost / speed)`, floored at one tick.

To make dwell independent of speed, add an explicit floor.

```
if progress >= step_cost && ticks_on_tile >= dwell_floor[unit_type] { step }
```

The cost is one `u8` for each unit to hold `ticks_on_tile`, which is 1 MB
at one million units, plus one comparison. The `dwell_floor` table is one
byte for each unit type and stays in the level 1 cache.

**Recommendation: do not build this for version 1.** Levers 1 and 2 already
give throughput control at 64 bytes. The implicit dwell already makes a
slow heavy unit a throughput liability. Build the floor only if the owner
wants a **fast** heavy unit that still throttles a crossing, which is a
narrow case.

**The interaction with the hold gate is real and it needs a statement.** A
member with a long dwell is always the rear member. The hold gate therefore
gates the whole formation on the slowest-dwelling member. That is correct
behaviour, and it matches the rule already stated for speed variance:
report the slowest member's rate as the formation rate. **A formation that
mixes heavy and light units moves at the heavy rate.** Tell the owner,
because it makes a mixed formation mechanically unattractive, and that may
or may not be the intent.

### 13.9 Lever 5: the tick rate

Throughput in units for each second is `capacity * tick_rate / dwell`. A
change from 10 Hz to 30 Hz triples throughput in wall time.

**It also triples every other rate, and one of them is baked.** The needs
decay rate is computed at bake time as `65536 / (D * TICKS_PER_DAY)`.[^7] A
tick rate change without a fresh bake silently changes how fast a soldier
starves.

**Recommendation: never use the tick rate as a game feel lever.** It is a
performance knob. If it changes, re-bake every per-tick rate and re-run the
golden state hash. Record the value in the replay header.

### 13.10 Claim 4: speed does help at a chokepoint, up to a ceiling

**The lead's claim is half right, and the wrong half matters. Correct it
before it reaches the owner again.**

The lead states that throughput is capacity divided by dwell, and concludes
that faster cavalry cross a one-tile bridge at the same rate as infantry.
The formula is right. **The conclusion does not follow from it, because
dwell is itself a function of speed** under the movement rule.

```
throughput = capacity / max(1, ceil(step_cost_of_exit / speed))
```

| Unit | Speed | Dwell | Throughput at capacity 8 |
|---|---|---|---|
| Marching infantry | 64 | 4 | 2 for each tick |
| Fast infantry | 128 | 2 | 4 for each tick |
| Cavalry | 256 | 1 | **8 for each tick** |
| Cavalry, doubled again | 512 | 1 | **8 for each tick** |

**Speed does help at a chokepoint, by a factor of four in this example, and
then it stops helping completely.** Dwell floors at one tile for each tick.
Above `speed >= step_cost` the crossing is capacity-limited, and further
speed gives nothing at all.

The correct statement for the owner:

> Speed and throughput are the same knob until a unit reaches one tile for
> each tick. Above that point they are independent, and only capacity moves
> throughput.

**This is a better mechanic than the flat claim.** It gives cavalry a real
advantage at a crossing, and it gives that advantage a hard ceiling, so no
speed upgrade can make a chokepoint irrelevant.

**Verify it against the admission kernel.** The kernel admits
`capacity - (density - departures)` for each tick. `departures` is exactly
the count of occupants whose progress reached the step cost. In a steady
state that count is `C / dwell`. **The kernel produces this law already.
Nothing needs to be built.**

### 13.11 The straggling threshold under a variable capacity

`K = 256` was chosen to exceed a 125-tick bridge delay. Both the corrected
dwell arithmetic and a variable capacity break that choice. The worst case
is now `strength * dwell_max / capacity_min`. At strength 1,000, dwell 8
and capacity 2 that is 4,000 ticks.

**Do not derive `K` from the world's worst case.** A `K` of 4,000 ticks is
400 seconds. It would disable straggle departure for every large formation,
which is worse than the problem it solves.

**Amend the rule instead. A queued tick is not a straggling tick.**

The movement kernel already maintains a saturating `blocked` counter that
increments when the admission rule rejects a unit.[^2] The hold gate
already knows when it has gated a member.

> Increment the straggle counter only on a tick when the member is neither
> blocked nor gated by the hold gate.

A member waiting at a chokepoint is blocked, so it contributes zero. A
member held by the gate contributes zero. Only a member that is free to
move and still falls behind accumulates the counter. **That is exactly what
straggling means.**

`K` then needs no world constant and no derivation. **Take `K = 64`
ticks.** The rule is robust to any future change in capacity, dwell, tick
rate or crossing width, which is the real point of the owner's question.

The cost is one extra condition in a map kernel that already reads the
`blocked` counter. It is free.

### 13.12 Which kernels change

| Kernel | Change |
|---|---|
| Intent step | Read the 64-byte capacity table. Pack the limit into the free 8 bits of the target field. One level 1 read. |
| Radix sort | **None.** Same key width, same record size, identical segments. |
| Admit step | Compare against the packed limit, not a constant. One shift. |
| Density array | **None.** It counts. Only the limit varies. |
| Camp footprint | A divide becomes a scan. Under 40 table reads for each camp. |
| Hold gate | **None.** It reads the flow cost only. |
| Rally bias and centroid | **None.** |
| Site bitplane and reverse index | **None.** |
| Straggle detection | One extra condition, on a counter it already reads. |

**No kernel leaves the vocabulary. No structure is added. The total added
storage is 64 bytes.**

---

## 14. Amendments from the chokepoint levers

### 14.1 Amendments that take no new number

These change decisions that section 10 already states.

- **Amend D134.** The camp tile count becomes a capacity scan in hex spiral
  order, not a fixed divide by 8. Add `MAX_CAMP_TILES` at four times
  `ceil(strength / max_world_capacity)`. A camp that exceeds the cap does
  not materialise.
- **Amend D137.** Throughput is capacity divided by dwell, and the dwell
  comes from the tile beyond the chokepoint. Replace the table with the
  corrected one in section 13.2. The approved 12.5-second figure is the
  dwell-1 case. Marching infantry take 50 seconds.
- **Amend D143.** The straggle counter does not increment on a tick when
  the member is blocked or gated. `K` falls from 256 to 64 and needs no
  derivation from a world constant.

### 14.2 Three findings, now assigned numbers

**The assigned range D130 to D149 is fully used.** The lead has since
assigned **D170 to D179** for the further decisions in this document.

The three findings that this section raised are now **D170, D171 and
D172**. Section 16 states them in full, with two corrections applied: the
road delivery rate is 4 times and not 6 times, and the track tier gains
through capacity rather than through cost.

### 14.3 An open question number that is now free

**OQ84 is answered and retired.** The owner accepts a long crossing. Reuse
the number for the question that section 13.8 raises.

**OQ84 (replacement). Should a formation that mixes heavy and light units
move at the heavy rate?** The hold gate makes it so, because the
slowest-dwelling member is always the rear member. This makes a mixed
formation mechanically unattractive. That may be the intent, or it may push
every player toward single-type formations. **This is a game design
question and it needs an answer before the leash constants are tuned.**

---
## 15. Calibration for a 12.5-second ordinary crossing

### 15.1 The owner's target

The owner rejects 50 seconds and accepts 12.5 seconds. The target is
therefore: **1,000 marching infantry cross a one-tile bridge onto ordinary
ground in about 125 ticks, which is 12.5 seconds at 10 Hz.**

### 15.2 The coupling that constrains every option

Throughput through a chokepoint is `capacity / dwell`, and
`dwell = ceil(step_cost_of_exit / speed)`. **Dwell is also what sets the
open-ground march rate.** A unit crosses one tile every `dwell` ticks
everywhere, not only at a chokepoint.

> **Chokepoint throughput and open-ground march rate are the same knob when
> you move them through dwell. They are separate knobs only when you move
> them through capacity.**

This is the finding that decides the calibration, and the lead's list of
options does not contain it. Lowering dwell to 1 buys the 12.5-second
bridge and **also multiplies the march rate by four**, because dwell falls
from 4 to 1. At 600 ticks for each game day, a unit covers 150 tiles for
each day at dwell 4 and 600 tiles at dwell 1.

### 15.3 The four candidates, tested

**Candidate A. Set the baseline dwell to 1.** This is the lead's
expectation. It hits the target. It has three costs. The march rate rises
four times. The speed accumulator saturates, so **speed does nothing at all
on ordinary ground**, because a unit steps at most once for each tick.
Cavalry lose their chokepoint advantage on every ordinary tile.

**Candidate B. Lower the step cost of ordinary terrain.** **This is the
same lever as candidate A, expressed differently. Confirm it plainly.**
Only the ratio `step_cost / speed` enters the dwell. Halving the step cost
is identical to doubling the speed in every equation in this document.
There is no case where one works and the other does not.

**Candidate C. Raise the base capacity above 8.** Reject it. The owner
locked capacity at 8. The camp footprint follows capacity, so 100 soldiers
would occupy 7 tiles instead of 13, and every army would shrink visually.

**Candidate D. Raise the capacity of the crossing terrain only.** The
capacity table of section 13.4 is already indexed by terrain, so a bridge
may carry a different capacity from ordinary ground at no cost. **This
raises chokepoint throughput without touching the march rate at all.**

The camp footprint objection does not apply, because a formation does not
camp on a bridge. Ordinary ground keeps capacity 8, so every camp footprint
in this document is unchanged.

**Candidate D is the option the lead's list omits, and it is the one that
preserves the most.**

### 15.4 The recommendation: a dwell-2 baseline plus a capacity-16 crossing

Take neither extreme. Move half the distance through dwell and half through
capacity.

| Quantity | Value |
|---|---|
| Progress scale | 256 for each tile of ordinary ground |
| Marching infantry speed | **128**, so ordinary ground gives dwell 2 |
| Cavalry speed | **256**, so ordinary ground gives dwell 1 |
| Ordinary ground capacity | **8**, unchanged. The owner's lock holds |
| Bridge and ford capacity | **16** |

Check the target: 1,000 infantry, chokepoint capacity 16, exit dwell 2.

```
ticks = strength * dwell / capacity = 1000 * 2 / 16 = 125 ticks = 12.5 s
```

**The target is met exactly.** Four properties make this better than
candidate A alone.

1. The march rate doubles rather than quadruples. A unit covers 300 tiles
   for each game day, not 600.
2. **Speed keeps a meaning on ordinary ground.** One doubling of headroom
   remains between infantry and the dwell-1 floor. Under candidate A there
   is none.
3. **Cavalry keep a 2x chokepoint advantage**, at dwell 1 against dwell 2.
   Section 15.7 states where it lives.
4. Capacity 8 stays the ordinary value, so every camp footprint,
   `MAX_CAMP_TILES` figure and density argument in this document survives
   unchanged.

### 15.5 The terrain ladder, and the spread the owner asked for

Step costs against an infantry speed of 128 and a cavalry speed of 256.

| Terrain | Step cost | Infantry dwell | Cavalry dwell |
|---|---|---|---|
| Paved road | 64 | 1 | 1 |
| Road | 128 | 1 | 1 |
| Track | 256 | 2 | 1 |
| Plains, ordinary | 256 | 2 | 1 |
| Forest | 384 | 3 | 2 |
| Hills | 512 | 4 | 2 |
| Dense forest | 640 | 5 | 3 |
| Marsh | 768 | 6 | 3 |
| Mountain | 1024 | 8 | 4 |

Crossing times for 1,000 infantry at a one-tile bridge of capacity 16,
by the terrain they cross **into**.

| Exit terrain | Ticks | Seconds at 10 Hz |
|---|---|---|
| Road | 63 | **6.3** |
| Plains, ordinary | 125 | **12.5** |
| Forest | 188 | 18.8 |
| Hills | 250 | 25.0 |
| Dense forest | 313 | 31.3 |
| Marsh | 375 | 37.5 |
| Mountain | 500 | **50.0** |

**This reads sensibly, and it places the owner's rejected figure correctly.**
The 50 seconds that the owner rejected for an ordinary bridge is not
deleted. It moves to a mountain crossing, which is where a 50-second delay
is the intended experience. A dwell above 1 now marks difficult terrain
only, exactly as the lead predicted.

### 15.6 A defect that a dwell-1 baseline exposes

**The progress accumulator overflows for any unit whose speed exceeds the
local step cost.** A unit steps at most once for each tick, but the rule
`progress -= step_cost` returns only one step's worth. Cavalry at speed 256
on a paved road at cost 64 bank a surplus of 192 for each tick. The `u16`
accumulator saturates in about 341 ticks.

**Fix: clamp the progress after a successful step.**

```
if progress >= step_cost {
    attempt the step;
    on success: progress = min(progress - step_cost, step_cost - 1);
}
```

The clamp discards surplus that the unit can never spend, because the
one-step-per-tick rule caps the useful stock at one step. It costs one
comparison. **Without it, a fast unit on cheap terrain is a silent
overflow, and an overflow is a determinism defect and not only a movement
defect.** This defect exists in the current rule and it is not created by
this calibration. The calibration only makes it reachable.

### 15.7 Where the cavalry advantage lives now

**It does not evaporate. State this plainly, because the lead expected it
might.**

| Situation | Infantry dwell | Cavalry dwell | Cavalry advantage |
|---|---|---|---|
| Paved road or road | 1 | 1 | **None.** A road is a road |
| Plains, track, ordinary | 2 | 1 | **2x throughput and 2x march rate** |
| Forest | 3 | 2 | 1.5x |
| Hills | 4 | 2 | **2x** |
| Marsh | 6 | 3 | **2x** |
| Mountain | 8 | 4 | **2x** |

Cavalry hold a **2x advantage in throughput and in march rate everywhere
off a road**, including at an ordinary chokepoint. They hold none on a
made road, which is correct: a road exists precisely to remove the terrain
penalty that cavalry otherwise escape.

**Under candidate A the advantage would survive only in difficult
terrain**, because infantry would already sit at the dwell-1 floor on
ordinary ground. The lead's view, that cavalry advantage belongs in
difficult terrain and in distance rather than at a chokepoint, is
defensible. But it is a choice and not a necessity, and candidate A pays a
mechanic for a target that the dwell-2 baseline also reaches. **Recommend
keeping the mechanic.**

### 15.8 Can a unit move more than one tile for each tick? No

The movement rule attempts at most one step for each tick. **A dwell-1
baseline is therefore a hard ceiling, not a soft one.** No amount of speed
moves a unit two tiles in one tick.

This is the whole argument against candidate A. Under candidate A every
unit on ordinary ground sits at the ceiling, so speed becomes a value that
does nothing except on difficult terrain. Under the dwell-2 baseline the
distinction survives on every ordinary tile, which is most of the world.

**A further structural consequence: dwell resolution is poor near the
ceiling.** Dwell is an integer, so between plains at dwell 2 and the floor
at dwell 1 there is no intermediate value. A track that is 25% cheaper than
plains still rounds to dwell 2 and gives **no throughput gain at all**.

**The fix uses the independence established in section 13.6.** Where the
dwell axis has no resolution, move the capacity axis instead.

### 15.9 Corrected road tiers, and a corrected delivery figure

**Section 13.6 contains an arithmetic error. Correct it.** That section
multiplied speed by throughput to get a 6x delivery rate. That double
counts speed, because dwell already contains it. **Sustained flow through a
one-tile-wide route is `capacity / dwell` and nothing else.** Speed sets the
arrival latency of the first unit. It does not raise the sustained rate a
second time.

Corrected tiers under the section 15.4 calibration.

| Tier | Step cost | Infantry dwell | Capacity | Throughput | Relative |
|---|---|---|---|---|---|
| None | 256 | 2 | 8 | 4.0 | **1.0x** |
| Track | 256 | 2 | 10 | 5.0 | **1.25x** |
| Road | 128 | 1 | 12 | 12.0 | **3.0x** |
| Paved | 64 | 1 | 16 | 16.0 | **4.0x** |

**A paved road delivers four times as many soldiers as open ground, not
six.** The strategic conclusion of section 13.6 stands and its magnitude
falls. A 4x delivery rate still changes where a player can concentrate
force, and it still makes breaking a road or a bridge a measurable act.

**The track tier now buys its gain through capacity, not through cost**,
because a cost reduction of 25% cannot cross an integer dwell boundary. The
paved tier's extra cost reduction below the road tier gives infantry
nothing, and it helps only units slower than infantry, such as siege
equipment. That is a legitimate role for the top tier.

### 15.10 Everything that depended on the old numbers

| Item | Status |
|---|---|
| Section 3.4, "125 ticks" and "31 tiles" | **Superseded** by section 13.2 and again here. At capacity 8 and dwell 2 an ordinary one-tile crossing takes 250 ticks. At the bridge capacity of 16 it takes 125. |
| Section 13.2 dwell table | Stands as a general law. The baseline row moves from dwell 4 to dwell 2. |
| Section 13.6, the 6x delivery rate | **Corrected to 4x** in section 15.9. The multiplication double counted speed. |
| Section 13.7, the width table at dwell 4 | Recomputed below. |
| Section 13.10, the cavalry table | Speeds shift. The law is unchanged and section 15.7 restates it. |
| `K = 64` straggle threshold | **Stands, and the fix is what makes it stand.** A queuing unit is blocked and a held unit is gated, so neither increments the counter under any dwell or capacity value. The threshold needs no recalibration now and none after any future change. |
| The capacity-accumulating spiral scan | **Unchanged.** Ordinary ground keeps capacity 8, so a 100-soldier camp still takes 13 tiles. |
| `MAX_CAMP_TILES` | **Needs one correction.** See below. |
| The `u8` density array | Unchanged. Capacity 16 leaves 15 times headroom below 255. |

Corrected width table for 1,000 infantry on ordinary ground at capacity 8
and dwell 2, so `throughput = 4N` for each tick.

| Width | Ticks | Seconds at 10 Hz |
|---|---|---|
| 1 | 250 | 25.0 |
| 2 | 125 | **12.5** |
| 3 | 84 | 8.4 |
| 5 | 50 | 5.0 |

**Note the equivalence: a two-tile crossing at capacity 8 hits the same
12.5-second target as a one-tile crossing at capacity 16.** Both are
available. **Recommend the capacity route**, because it applies to every
bridge in the world from one table entry, while the width route is
per-location level design.

**The `MAX_CAMP_TILES` correction.** Section 14.1 sets the cap at four
times `ceil(strength / max_world_capacity)`. That is backwards. Raising the
world's maximum capacity to 16 would **tighten** the cap from 52 tiles to
28, which is the opposite of the intent. Define it against a named baseline
instead:

```
MAX_CAMP_TILES = 4 * ceil(strength / CAPACITY_ORDINARY)   // CAPACITY_ORDINARY = 8
```

A 100-soldier camp then caps at 52 tiles, independent of what any crossing
terrain carries.

---

## 16. Decision block for D170 to D179

**This block is ready to apply. It uses D170 to D174 and OQ100. The range
D130 to D149 is fully used and unchanged. D150 to D169 belongs to another
report and this document takes no number from it.**

### 16.1 The design rule that these decisions rest on

State this first, because it is useful beyond this one calibration.

> **A chokepoint crossing time and an open-ground march rate are the same
> knob when you move them through dwell. They separate only when you move
> throughput through capacity. So the owner cannot set a crossing time and
> a march rate independently by changing speed alone. Route throughput
> through capacity, and leave dwell to set the march rate.**

Throughput is `capacity / dwell`, and `dwell = ceil(step_cost / speed)`.
Dwell also sets how many ticks a unit spends on every ordinary tile. So a
speed change moves both quantities together. A capacity change moves only
the first.

### 16.2 Amendments to decisions already in section 10

These take no new number.

- **Amend D134.** The camp tile count is a capacity-accumulating scan in
  hex spiral order, not a fixed divide. `MAX_CAMP_TILES` is four times
  `ceil(strength / CAPACITY_ORDINARY)`. **Define it against the named
  ordinary-terrain capacity, never against the world maximum capacity.**
  Defining it against the maximum inverts the intent: raising a bridge to
  capacity 16 would tighten a 100-soldier cap from 52 tiles to 28.
- **Amend D137.** Throughput is capacity divided by dwell, and the dwell
  comes from the tile beyond the chokepoint. Take the section 15.5 ladder.
  The baseline is dwell 2.
- **Amend D143.** The straggle counter does not increment on a tick when
  the member is blocked or gated. `K` falls from 256 to 64. **This is
  confirmed robust under the new calibration**, because the counter no
  longer depends on dwell or on capacity at all.

---

#### D170. Tile capacity is a terrain and road lookup, carried in the intent's spare bits

Capacity becomes `capacity[terrain][road_tier]`, a `u8` table of 16 terrain
kinds by 4 road tiers, which is **64 bytes**. It extends the existing
movement cost table and it adds no structure, because the 2-bit `road_tier`
plane already exists for the cost lookup.

**The table drops the unit type dimension.** The movement cost table is
indexed `[unit_type][terrain][road_tier]`. Capacity must not be, because
the admission rule admits one contiguous segment of intents into one target
tile against **one** limit. A segment holding two unit types would carry
two limits, and the admission scan would stop being one comparison.

**Sort-then-admit is undisturbed, and the mechanism is not a gather in the
admit step.** The intent step gathers the terrain; the admit step is a
separate pass. A fresh gather there would read 5.1 MB over the touched
blocks, at about 0.26 core-ms. Avoid it. The world holds 16.7 million
tiles, which needs 24 bits, so the intent's `u32` target field has **8 free
bits**. The intent step writes the limit there.

```
intent.target = (tile & 0x00FF_FFFF) | ((cap as u32) << 24);
```

The record does not grow. The radix sort key may stay the full 32 bits,
because capacity is a pure function of the tile, so equal tiles carry equal
bits and the segments are identical. **No mask and no extra pass.** Cost:
zero extra memory traffic.

**The `u8` density array is unchanged.** It counts units. Only the limit
varies. Capacity 16 leaves 15 times headroom below 255.

#### D171. A road tier sets traverse cost and capacity independently

Cost is surface quality. Capacity is width. They are physically distinct. A
narrow paved mountain road is fast and low capacity. A wide dirt drove road
is slow and high capacity. A coupled pair cannot express either.

Independence costs nothing. The two tables are already separate arrays over
a shared index.

| Tier | Step cost | Infantry dwell | Capacity | Throughput | Relative |
|---|---|---|---|---|---|
| None | 256 | 2 | 8 | 4.0 | **1.0x** |
| Track | 256 | 2 | 10 | 5.0 | **1.25x** |
| Road | 128 | 1 | 12 | 12.0 | **3.0x** |
| Paved | 64 | 1 | 16 | 16.0 | **4.0x** |

**Correction to an earlier figure in this document. The delivery rate of a
paved road is 4 times that of open ground, not 6 times.** Section 13.6
multiplied speed by throughput. That double counts speed, because dwell
already contains it. **Sustained flow through a one-tile-wide route is
`capacity / dwell` and nothing else.** Speed sets the arrival latency of the
first unit; it does not raise the sustained rate a second time.

The strategic conclusion stands at the reduced magnitude. A 4x delivery
rate still changes where a player can concentrate force, and it still makes
breaking a road or a bridge a measurable act.

**The track tier gains through capacity, not through cost.** Dwell is an
integer, so a 25% cost reduction cannot cross the boundary from 2 to 1 and
buys no throughput at all. Where the dwell axis has no resolution, move the
capacity axis. This is the first practical use of the independence that
this decision establishes.

#### D172. Speed raises throughput until dwell reaches one tick, then stops

```
throughput = capacity / max(1, ceil(step_cost_of_exit / speed))
```

**Speed and throughput are the same knob below one tile for each tick. They
are independent above it.** The movement rule attempts at most one step for
each tick, so dwell floors at 1 and further speed gives nothing.

This gives cavalry a real advantage at a crossing and a hard ceiling on it,
so no speed upgrade makes a chokepoint irrelevant. **The admission kernel
produces this law already and needs no change**, because `departures` is
the count of occupants whose progress reached the step cost, which is
`C / dwell` in a steady state.

#### D173. The movement calibration. Express it against the tile scale, not as absolutes

**Adopt a dwell-2 baseline plus a capacity-16 crossing.** This meets the
owner's target of about 125 ticks for 1,000 marching infantry at a one-tile
bridge onto ordinary ground.

```
ticks = strength * dwell / capacity = 1000 * 2 / 16 = 125 ticks = 12.5 s
```

**The load-bearing part is raising capacity on the crossing terrain only.**
Ordinary ground keeps capacity 8. That is what lets the crossing time fall
without the march rate rising to match, and it is what preserves the
owner's capacity lock and every camp footprint in this document.

**Do not fix the constants as absolutes. The tile scale is still an open
question, so express them parametrically.** Let `TICKS_PER_DAY` be the
ticks in a game day, `L` the tile edge, and `M` the intended march rate for
each game day in the same length unit.

```
dwell_baseline   = TICKS_PER_DAY * L / M
SPEED_INFANTRY   = STEP_ORDINARY / dwell_baseline      // STEP_ORDINARY = 256
CAPACITY_CROSSING = ceil(strength * dwell_baseline / target_ticks)
```

The recommended values are the case `dwell_baseline = 2`:

| Constant | Value at dwell 2 |
|---|---|
| `STEP_ORDINARY` | 256 |
| `SPEED_INFANTRY` | 128 |
| `SPEED_CAVALRY` | 256, giving dwell 1 |
| `CAPACITY_ORDINARY` | **8, unchanged. The owner's lock holds** |
| `CAPACITY_CROSSING` | 16 |

Four properties justify this over a dwell-1 baseline.

1. The march rate doubles rather than quadruples.
2. **Speed keeps a meaning on ordinary ground.** One doubling of headroom
   remains above infantry. A dwell-1 baseline leaves none, because a unit
   can never move more than one tile in one tick.
3. **Cavalry keep a 2x advantage everywhere off a made road**, and none on
   a road, which is the correct shape. A dwell-1 baseline would pay this
   mechanic for a target that dwell 2 also reaches.
4. The terrain ladder of section 15.5 gives 6.3 seconds on a road, 12.5 on
   ordinary ground and 50 on a mountain. **The 50 seconds that the owner
   rejected is not deleted. It moves to a mountain crossing, where a long
   delay is the intended experience.**

#### D174. Clamp the progress accumulator. This is a defect in the existing movement rule

**This is a defect found in the current rule, not one that the calibration
introduces. The calibration only makes it reachable.**

A unit steps at most once for each tick, but `progress -= step_cost`
returns only one step's worth of the accumulator. **Any unit whose speed
exceeds the local step cost therefore banks a surplus that it can never
spend.** Cavalry at speed 256 on a paved road at cost 64 bank 192 for each
tick, and the `u16` accumulator overflows in about 341 ticks.

```
if progress >= step_cost {
    attempt the step;
    on success: progress = min(progress - step_cost, step_cost - 1);
}
```

The clamp discards surplus that the one-step-per-tick rule makes
unspendable. It costs one comparison.

**Treat this as a determinism defect, not only a movement defect.** An
overflowing accumulator is simulated state. It enters the frame state hash,
so it breaks the golden-file test and the thread-count equivalence test.
Add a test that runs a unit at the maximum speed on the cheapest terrain
for 1,000 ticks and asserts that the accumulator stays in range.

---

### 16.3 Open question

**OQ100. Confirm the tile scale and the intended march rate. They fix every
constant in D173.**

The dwell-2 baseline gives 300 tiles for each game day at 600 ticks for
each day. Whether that rate is right depends on the tile edge, and the
world extent is still an unanswered question for the owner.

Work the two cases.

| Tile edge | March rate | Required dwell | Crossing capacity for 12.5 s | Verdict |
|---|---|---|---|---|
| 80 m | 24 km for each day | 2 | 16 | **Consistent.** A world 4,096 tiles wide spans 330 km |
| 1 km | 24 km for each day | 25 | 200 | **Not consistent** |

**A continental tile scale and a 12.5-second crossing are incompatible.** A
one-kilometre tile at a historical march rate needs dwell 25, and holding
the crossing target then needs a capacity of 200 on the bridge tile. That
exceeds any sensible headroom in the `u8` density array and it is visually
absurd.

**So the owner must choose one of three.** Take a small tile of about 80
metres and a regional world of about 330 kilometres. Or take a large tile
and accept a slower crossing. Or shorten the game day, which changes
`TICKS_PER_DAY` and forces a re-bake of every per-tick rate, including the
needs decay rates.

**Do not fix `SPEED_INFANTRY` or `CAPACITY_CROSSING` before this is
answered.** The parametric form in D173 resolves both as soon as it is.

---
## References

[^1]: Report 02, Hex Grid and Level of Detail Pyramid. Sections on the block layout, fanout 16, and the group-with-inverse update rule. `docs/research/reports/02-hex-grid-and-lod-pyramid.md`
[^2]: Report 10, Crowd and Movement. Sections 4 and 11, on tile-discrete positions, the floor field rule, the density array, sort-then-admit, and the flow tile cache key. `docs/research/reports/10-crowd-and-movement.md`
[^3]: Report 14, Character Graph and Inheritance. Section 10, on formations as organisational nodes, the ownership column, the compressed sparse row reverse index, and the bounding level 1 mask. Section 5.3 on the office cascade. `docs/research/reports/14-character-graph-and-inheritance.md`
[^4]: Report 01, Entity Component System and Memory Layout. Sections on block-tiled storage and the rank-select structure for sparse tile payloads. `docs/research/reports/01-ecs-and-memory-layout.md`
[^5]: Report 12, Entity Economy and Modifiers. Section 3.3 and decision D53, on effective-stat memoisation and the 99.5% to 99.98% hit rate. `docs/research/reports/12-entity-economy-and-modifiers.md`
[^6]: Report 16, Individual Agency and Occupations. Owns individual intent selection and the group weight vector. In progress at the time of writing. `docs/research/reports/16-individual-agency-and-occupations.md`
[^7]: Report 15, Needs, Consumption and Economy. Section 6, on the four needs, the Q16.16 scale, the decay rates, and the dense-bitset plus sparse-scan threshold pattern. `docs/research/reports/15-needs-consumption-and-economy.md`
[^8]: Merge Notes for ADR-0001, section 9, on largest-remainder apportionment for the conserving transfer verb and the accepted Alabama paradox. `docs/research/reports/MERGE-NOTES.md`
[^9]: Burstedde, Klauck, Schadschneider and Zittartz, "Simulation of pedestrian dynamics using a two-dimensional cellular automaton", Physica A 295, 2001, pages 507 to 525.
[^10]: Balinski and Young, "Fair Representation: Meeting the Ideal of One Man, One Vote", Yale University Press, 1982.
[^11]: ADR Registry. The omnibus draft this report fed was deleted and its number reclaimed. The hard invariants, the barrier count, and the counter-based random generator rule. `docs/adrs/REGISTRY.md`
