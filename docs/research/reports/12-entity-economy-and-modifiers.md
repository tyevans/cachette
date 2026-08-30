# Entity Economy and Modifiers

Research report 12 for ADR-0001. This report covers resource and stat
**generation, consumption and modification** at the entity level and at the
tile level.

## Scope

This report covers six subjects:

1. Production. A structure yields resources on each tick.
2. Upkeep. A unit or a structure consumes resources on each tick.
3. Growth, decay, research and pollution. These have the same shape as
   production.
4. Effective stats. A base type, its upgrades, terrain, adjacency,
   technology and policy combine into one number that a system reads.
5. Tile upgrades. Roads, irrigation and mines modify tile properties and
   the terrain-cost matrix.
6. Pooled quantities, caps, saturation and threshold crossings.

This report does **not** cover resource transport, trade networks, markets
or flow solvers. Section 11 states the interface that this subsystem
exposes to the transport subsystem.

## Context that this report assumes

Cachette is a world simulation engine. The core is Rust. The control plane
is Python. The world holds about 16.7 million hex tiles and up to one
million units. The deployment target is an AWS Graviton server.[^1]

Five facts from the foundational architecture record govern every
recommendation below.[^2]

- **No floating point in simulated or aggregated state.** The fixed-point
  scale is Q16.16. `Fix32` is `i32`. `Fix64` is `i64`. `Accum` is always
  `i64`.
- **The modifier pipeline has five fixed stages.** The record states them
  as base, flat, percent, multiplier, clamp.
- **The frame loop splits reads from writes.** Phases 1 to 4 read the world
  and write only events. Phases 5 to 8 write the world and read only
  events.
- **Types are data.** `UnitTypeId` is a `u16` index into an immutable stat
  table. `UpgradeSetId` is an interned `u32`. `CapabilityMask` is a `u64`
  derived from the type and the upgrade set.
- **Determinism is bit-exact for one binary at any thread count.** Ordered
  iteration and stable keys are mandatory. Thread completion order is
  forbidden.

Integer addition and bitwise OR are exactly commutative and exactly
associative. A scatter-add and a scatter-or are therefore independent of
order. Minimum, maximum and first-wins are **not** order-independent, and
this report never uses them in a parallel reduce.

---

## 1. The unified rate kernel

### 1.1 The hypothesis

The session lead proposes that production and upkeep are one kernel with
signed rates. This report confirms that proposal. The confirmation is not
trivial, and section 1.4 states the one place where the sign matters.

Each entity contributes a signed rate to a pooled quantity. A positive rate
is production. A negative rate is upkeep. Growth, decay, research and
pollution take the same form. Decay is a negative rate on the pool itself
rather than on an entity, so it is one extra row in the same loop.

### 1.2 The recipe

An entity does not store its rates. The type table stores them. A type
declares a **recipe**: a short list of pairs of a commodity and a signed
base rate.

```rust
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct RecipeSlot {
    commodity: u16,     // CommodityId
    rate:      i32,     // Fix32, Q16.16, per tick, signed
    priority:  u8,      // consumption priority band, 0 is highest
    _pad:      [u8; 1],
}
```

Recipes intern into a deduplicated table, exactly as upgrade sets do. A
`RecipeId` is a `u32`. The recipe table stores a flat array of slots and a
`(start, len)` pair per recipe. The mean recipe length is short. A farm
produces one commodity. A workshop consumes two and produces one. A unit
consumes one or two. This report assumes a mean of 2.2 slots per entity and
a maximum of 8.

Cap the slot count per recipe at 8 and check the cap at bake time. An
unbounded recipe makes the inner loop unpredictable.

### 1.3 The kernel, in the engine vocabulary

The production and upkeep pass is five kernels in sequence.

| Step | Kernel | Input | Output |
|---|---|---|---|
| 1 | map | entity row | `EffKey` per entity |
| 2 | gather | `EffKey` | effective recipe row |
| 3 | map | recipe slot | signed `Accum` delta |
| 4 | segmented reduce | deltas sorted by pool | pool net rate |
| 5 | map | pool net rate | pool stock update |

Step 4 is a **segmented reduce, not a scatter-add**. Section 5.3 explains
why this choice removes every atomic from the pass.

### 1.4 Where the sign matters

Production and upkeep share the kernel up to step 4. They separate at step
5, because a pool has a floor at zero and a ceiling at its capacity.

- A positive rate can exceed the ceiling. The excess is **overflow**.
- A negative rate can fall below the floor. The shortfall is
  **underflow**, and underflow means starvation or bankruptcy.

A naive implementation clamps the pool after the reduce. That is wrong, and
section 7 states why: a clamp discards the information about which
consumers were denied. The kernel therefore reduces production and
consumption into **two separate pool columns**, and section 7 resolves them
against each other.

So the hypothesis holds with one correction. Production and upkeep are one
kernel for steps 1 to 4. They are two accumulators, not one.

---

## 2. The modifier pipeline

### 2.1 What shipped games do

**Paradox.** The Clausewitz engine family applies modifiers within a
category by addition and across categories by multiplication. The community
documentation of the behaviour describes the shape as a base value, one
additive sum and one multiplicative product.[^3] The order is fixed by the
engine, not by the data. A content author cannot reorder the stages.

**RimWorld.** Content is defined in XML definition files that map onto game
classes. The files support abstract bases and inheritance, so common values
are not repeated.[^4] A stat is computed by an ordered list of parts that the
game code defines. Each part transforms a running value through an offset or
a factor, and a clamp runs last. The order is code, not content.

**Factorio.** Effects are strictly additive bonuses on a fixed set of named
fields, such as speed, productivity and energy consumption. There is a
mutable authoring phase, a one-time bake, and then an immutable run-time
table.[^5] Factorio deliberately has no general modifier language. This
keeps the effective value a small fixed computation.

**Civilization VI.** Modifiers are database rows that attach to a
requirement set. The stack is additive for yield changes and multiplicative
for a small named set of percentage effects. The order is a property of the
modifier type, which is schema data. **Verify this claim against current
modding documentation before the record is finalised.**[^6]

The convergent lesson is clear. **Every shipped implementation fixes the
stage order in the schema. None lets the data choose the order.** Order in
data is a determinism hazard and an authoring hazard at the same time.

### 2.2 The recommended schema

The foundational record already fixes the stages.[^2] This report adds the
category rules and the type declarations that make the order a schema
property.

```
stage 0  base       = stat_table[type][field]                     // Fix32
stage 1  flat       = base + sum(flat modifiers)                  // i64
stage 2  percent    = (flat * (65536 + sum(pct modifiers))) >> 16  // i64
stage 3  multiplier = fold(mult categories, in declared order)     // i64
stage 4  clamp      = clamp(result, field_min, field_max)          // Fix32
```

A modifier declares three things at bake time:

```rust
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ModifierDef {
    field:    u16,   // FieldId, index into the stat schema
    op:       u8,    // 0 = Flat, 1 = Pct, 2 = Mult
    category: u8,    // meaningful only when op == Mult
    value:    i32,   // Fix32
}
```

Four rules make the pipeline deterministic and cheap.

1. **Flat and percent modifiers need no order.** Stage 1 and stage 2 sum
   integers. Integer addition is exactly commutative and exactly
   associative, so the sum is identical for any collection order.
2. **Multiplier categories are ordered by a compile-time constant array.**
   The array lists the categories once. A modifier names its category. The
   fold walks the array, not the data. Two modifiers in the same multiplier
   category multiply in ascending source identifier order, and the bake step
   rejects two modifiers with the same category and the same source.
3. **Cap the multiplier categories at 8.** Each extra category is one more
   multiply and one more rounding step in every effective-stat computation.
   Eight covers terrain, stance, morale, weather, technology, policy,
   difficulty and a spare. A ninth category is a content design question,
   not an engine question.
4. **Round once per stage, toward zero, and clamp last.** Each stage
   computes in `i64` and shifts down by 16. A widening `i64` intermediate
   costs nothing on the target, because 64-bit integer arithmetic runs at
   full rate on Neoverse cores.[^1]

### 2.3 The scale conflict is already resolved

An earlier research report proposed a scale of 1/1024 for modifiers, so
that the modifier multiply stays inside `i32`.[^7] The foundational record
rejects that and mandates Q16.16 everywhere, because a widening `i64`
intermediate is free on the target.[^2] **This report follows the record.
The scale is Q16.16 for every modifier and every effective stat.** No part
of this report needs the coarser scale.

One consequence deserves a note. Stage 2 computes
`flat * (65536 + pct_sum)`. If `flat` is a Q16.16 value near the `i32`
maximum of about 32768 and `pct_sum` is large, the product needs 62 bits.
It fits in `i64` with margin. The bake step must still check that
`field_max` for each field keeps the product under 2^62. A compile-time
assertion in the field registry is the right place for that check.

### 2.4 Range overflow at the multiplier stage

Stage 3 folds up to 8 multipliers. Each fold is a multiply and a shift by
16. Eight folds of a value near 2^31 accumulate rounding of at most 8 units
in the last place, which is 8/65536 of one stat point. That is invisible.
The bigger risk is intermediate overflow. Fold in `i64`, clamp the running
value to `field_max` after each category, and the fold cannot overflow.
Clamping between categories changes the result against clamping only at the
end. **State the clamp position in the schema and never change it**, because
a change invalidates every replay.

---

## 3. The effective-stat table

### 3.1 The hypothesis, tested

The session lead proposes that the modifier pipeline runs per configuration
rather than per entity, because `(TypeId, UpgradeSetId)` is the real key for
effective stats.

**The hypothesis holds. It is the single largest optimisation in this
report.** The evidence follows, and section 3.5 states the two conditions
under which it fails.

### 3.2 The key

The pair `(TypeId, UpgradeSetId)` is not sufficient by itself. Technology
and policy also enter the pipeline, and both are faction-scoped. The
correct key is a triple:

```rust
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, bytemuck::Pod, bytemuck::Zeroable)]
struct EffKey {
    unit_type: u16,   // UnitTypeId
    faction:   u16,   // FactionId
    upgrades:  u32,   // UpgradeSetId
}
```

Technology and policy do not enter the key as values. They enter it through
the faction identifier, because the engine holds one modifier list per
faction. A faction's technology change invalidates that faction's rows and
no others.

The table is a per-faction dense array of rows, indexed by a per-faction
`ConfigId(u16)`. A small deduplicating map from `EffKey` to `ConfigId`
builds the index. **Store the `ConfigId` as a column on the entity.** Then
the per-entity work in step 1 of the kernel is a single 2-byte column read,
not a hash lookup.

### 3.3 The hit rate

The hit rate is `1 - K/N`, where `K` is the count of distinct configurations
and `N` is the count of entities.

`K` is bounded by the product of the type count, the faction count and the
distinct upgrade-set count. In practice it is far below that product,
because upgrade sets correlate strongly with type. A siege engine and a
scout do not share an upgrade.

| Scenario | Types | Factions | Distinct upgrade sets | Plausible `K` | Entities `N` | Hit rate |
|---|---|---|---|---|---|---|
| Structures, developed map | 40 | 8 | 12 | 190 | 50,000 | 99.6% |
| Units, mid game | 60 | 8 | 30 | 900 | 400,000 | 99.8% |
| Units, late game, heavy upgrades | 60 | 8 | 200 | 4,800 | 1,000,000 | 99.5% |
| Pathological content, one set per unit | 60 | 8 | 1,000,000 | 1,000,000 | 1,000,000 | 0% |

The first three rows are the design case. The fourth row is the failure
mode that the foundational record already names for the upgrade intern
table itself: if content lets each unit accumulate a different set, the
intern table grows toward one entry per unit.[^2] The same monitor covers
both. **Report the distinct-configuration count each tick and fail a test
above a threshold of 8,192.**

### 3.4 Memory and rebuild cost

A row holds every field the schema declares. Assume 32 fields at `i32`,
which is 128 bytes. The recipe pointer and length add 8 bytes. Round to
**144 bytes per row**.

| `K` | Table size | Fits |
|---|---|---|
| 190 | 27 KiB | L2 cache |
| 900 | 127 KiB | L2 cache |
| 4,800 | 675 KiB | L2 cache on Neoverse V1 and later |
| 8,192 (the cap) | 1.13 MiB | L2 or L3 |

The rebuild cost is the pipeline run `K` times over 32 fields. Each field
costs about 15 ns, because it gathers a short modifier list and runs five
stages. At `K = 4,800` the rebuild costs
`4800 * 32 * 15 ns = 2.3 core-ms`.

Rebuild is **not** per event. Rebuild is per faction per tick, and only when
that faction's modifier set changed. Technology and policy change a few
times per minute, not a few times per tick. At 10 Hz the amortised cost is
under 0.05 core-ms per tick. Rebuild at the frame barrier, in phase 5, in
ascending faction order.

Partial invalidation is possible but not recommended for version 1. A
modifier that names one field could invalidate one column. The bookkeeping
costs more than the 2.3 ms it saves at a frequency of once per minute.
**Rebuild the whole table for the affected faction.**

### 3.5 The break-even, and where the hypothesis fails

Per-entity evaluation costs about 15 ns per field. The table lookup costs
one gather into an L2-resident array, which is about 5 ns, plus the
2-byte `ConfigId` read, which is sequential and effectively free.

The table wins when `K * P < N * P_table_miss + K * P`, which reduces to
`N > K`. The break-even is at `N = K`. Any hit rate above zero favours the
table. **The table is never slower in a realistic case**, because the
rebuild is amortised over many ticks and the lookup is cheaper than the
pipeline even for a single entity.

The hypothesis fails in exactly two ways.

**Failure 1: per-entity state enters the pipeline as a stage input.** If
damage, veterancy or local terrain selects *which* modifiers apply, the
configuration no longer determines the result. This is fatal to the sharing
and the schema must forbid it.

**Failure 2: per-entity state enters as an unbounded continuous factor.**
If effective attack is `base * (health / max_health)` with health as a
continuous value, every entity has a distinct result.

Both failures have the same fix, and it is a schema rule.

> **A per-entity input may only enter the pipeline as a post-stage
> multiplier drawn from a small fixed table.**

The kernel becomes a table lookup followed by up to 3 multiplies:

```
eff = eff_table[config_id][field]
eff = (eff * damage_factor[health_tier(entity)])   >> 16
eff = (eff * veteran_factor[veterancy(entity)])    >> 16
eff = (eff * terrain_factor[terrain(tile)][field]) >> 16
```

`health_tier` quantises health into 8 bands. `veterancy` is already a small
enumeration. `terrain_factor` is a table of 32 terrains by the field count.
All three tables are tiny and stay in L1 cache. The cost is 3 multiplies and
3 shifts, which is under 3 ns per entity per field.

Quantising health into 8 bands is a visible design choice, not an
implementation detail. It means a unit at 51% health and a unit at 60%
health fight identically. That is acceptable and it is what most strategy
games already do. Record it in the decision, so nobody treats it as a bug.

Cap the post-stage multiplier count at 4. Each one is a real cost in the
hot loop.

---

## 4. Adjacency and terrain bonuses

### 4.1 Two different questions

The session lead states two beliefs. This report tests both.

**Belief 1: the L1 terrain histogram answers block-granularity adjacency
for free.** This is **true but it answers a different question than the
designer usually asks.** The L1 summary already declares a terrain
histogram as `Histogram<32>`.[^7] Reading `hist[FOREST]` for a producer's
own block is one array read of 2 bytes, and the pyramid already maintains
it. The cost is genuinely zero.

But "how many forest tiles are in my 256-tile block" is not "how many of my
6 neighbours are forest". The block bonus is a **regional** bonus. It is a
correct and cheap mechanic, and it is what a city-scale yield should use.
It is not radius-1 adjacency.

**Belief 2: true radius-1 adjacency needs a small cached stencil behind the
existing dirty bits.** This is **true, and the caching is essential rather
than optional.** The arithmetic follows.

### 4.2 The cost of uncached radius-1 adjacency

A producer reads its own tile and 6 neighbours. In block-tiled 16x16 order,
a tile at an interior position has all 6 neighbours inside the same 256-byte
block. There are 14 x 14 = 196 interior positions out of 256, so
**76.6% of tiles need one 256-byte span** and their gather stays inside 4
cache lines. The remaining 60 edge positions touch up to 3 further blocks.

Cost per producer, uncached:

| Case | Fraction | Cache lines touched | Cost |
|---|---|---|---|
| Interior tile | 76.6% | 1 to 2 | about 20 ns |
| Edge tile | 23.4% | 3 to 4 | about 90 ns |
| Weighted mean | | | **about 36 ns** |

At 50,000 producers that is 1.8 core-ms per tick. At 1,000,000 entities
with adjacency it is **36 core-ms per tick**, which exceeds the whole
economy budget on its own.

### 4.3 The cost of the cached stencil

Producers are static. A farm does not move. So the adjacency contribution
changes only when a tile within radius 1 of the producer changes terrain or
gains an upgrade.

Cache the contribution as one `Fix32` column on the producer entity. The
column is 4 bytes per producer, which is 200 KiB at 50,000 producers and
4 MiB at 1,000,000.

Recompute is driven by the existing L1 dirty bitset. When a block is dirty
for terrain, rescan the producers in that block and in its 8 surrounding
blocks, because an edge producer reads across the boundary.

| Quantity | Value |
|---|---|
| Terrain writes per tick, plausible | 1,000 tiles |
| Distinct dirty blocks | up to 1,000 |
| Blocks rescanned with the 3x3 halo | up to 9,000 |
| Producers per block at 50,000 over 65,536 blocks | 0.76 |
| Producers rescanned | about 6,800 |
| Cost at 36 ns each | **0.25 core-ms** |

The cached path costs 0.25 core-ms against 1.8 core-ms uncached at 50,000
producers, and against 36 core-ms at 1,000,000. **Cache it.** The saving
grows with the entity count, and the cache is 4 bytes per entity.

Two rules keep the cache correct.

- Recompute in ascending producer index order, in phase 5, after the tile
  writes land and before the pyramid update.
- Mark the halo, not only the dirty block. A producer on a block edge reads
  a tile in the adjacent block. Missing the halo produces a silent
  one-block-wide error that no test finds by accident.

### 4.4 When a producer moves

A mobile entity with an adjacency bonus defeats the cache, because it
invalidates its own entry on every move. Handle it with the same dirty-bit
pattern from the other side: the movement system already knows which units
changed tile, and the foundational record already restricts field-of-view
recomputation to that set.[^2] Reuse that set. Units that changed tile
recompute their stencil. At a plausible 30,000 movers per tick the cost is
1.1 core-ms.

---

## 5. Pooled quantities

### 5.1 Where a stockpile lives

Three placements are possible. Only one is affordable and expressive.

| Placement | Storage at 64 commodities, `i64` | Verdict |
|---|---|---|
| Per tile, 16.7M tiles | 8.6 GB | **Reject.** Two orders of magnitude over budget. |
| Per faction, 8 factions | 4 KiB | **Reject alone.** It cannot express local scarcity, so the transport network has nothing to solve. |
| **Per settlement, 5,000 pools** | **2.56 MB** | **Recommend.** |

**Recommendation: the stockpile lives per settlement.** A settlement is a
named pool with a location. The engine calls it a `Pool`, because a
settlement is a game concept and a pool is an engine concept.

```rust
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PoolRow {
    tile:     u32,   // TileIdx, the pool location for the transport graph
    faction:  u16,   // FactionId
    _pad:     [u8; 2],
}
```

The stock itself is a separate flat array in commodity-major order per pool:
`stock[pool * COMMODITY_COUNT + commodity]` as `i64`. Pool-major order is
correct, because every consumer of this array reads all commodities of one
pool.

Keep a per-faction roll-up as a **derived** array of the same shape, updated
by the same signed delta. It costs 4 KiB and it answers "can this faction
afford it" without a scan. It is a group, so the delta update is exact and
needs no rescan.

### 5.2 The commodity ceiling

**Recommendation: 64 core commodities.**

Four independent reasons give the same number.

1. A per-pool "which commodities are present" mask is one `u64`. That mask
   prunes the resolve pass in section 7 and it aggregates into the pyramid
   as a bitwise OR, which is exactly commutative.
2. 64 commodities at `i64` is 512 bytes per pool, which is exactly 8 cache
   lines. A pool's whole stock is 8 sequential prefetches.
3. The L1 summary budget is a hard 256 bytes per cell.[^2] A commodity
   histogram cannot exceed that budget, so a summary can carry a presence
   mask and a few named totals, not 64 sums.
4. It matches the capability-mask precedent. A ceiling of 64 that is
   enforced by a `u64` is a ceiling that the type system checks.

Content that needs a rare commodity uses a **sparse side table** keyed on
`(PoolId, CommodityId)`, exactly as rare tile data does.[^8] A sparse
commodity does not tick at full rate. It is written by an event and read by
a query.

Reject 128 and 256 commodities. Both break reason 1 and reason 2, and
neither is needed for a strategy game. Established titles ship with well
under 64 distinct tradeable goods.

### 5.3 The reduce, with no atomics

The pass reduces about 2.2 million signed deltas into 5,000 pools. Three
implementations are possible.

**Option A: atomic scatter-add.** Exactly deterministic, because integer
addition commutes. But the target has a weak memory model, and a relaxed
`fetch_add` emits a real barrier on ARM where x86 emits almost nothing.[^1]
2.2 million contended atomics is the wrong shape.

**Option B: per-worker private accumulator arrays, combined in worker
order.** Storage is `workers * pools * commodities * 8` bytes. At 64
workers, 5,000 pools and 64 commodities that is **164 MB**. Reject on
memory.

**Option C, recommended: keep producers sorted by pool identifier and run
a segmented reduce.** Structures are static, so the sort is not a per-tick
cost. Rebuild the order only when a structure is created or destroyed, in
the structural phase that already invalidates every index.

Each parallel task then owns a contiguous span of producers, which covers a
contiguous range of pools. Only the pools at the two ends of a span are
shared with a neighbouring task. Handle those with a boundary-fixup pass of
at most `2 * task_count` entries, combined in ascending task index order.
**No atomic appears anywhere in the pass**, and the outputs are disjoint,
which the record promotes from preferred to required.[^1]

Mobile entities with upkeep do not sort by pool for free. Assign each unit
to a pool by its owning formation or by its home settlement, and store that
`PoolId` as a column. Then units also sort by pool. The unit arena is
already re-sorted by a spatial key on some cadence, so a second sort key is
a known question rather than a new one.[^2]

---

## 6. Tile and terrain upgrades

### 6.1 What fraction of tiles carries an upgrade

This decides dense against sparse, so it needs a number rather than a
guess.

| Upgrade class | Plausible extent | Fraction of 16.7M tiles |
|---|---|---|
| Roads and rails | A connected network between about 5,000 settlements, mean path 60 tiles | 1.8% |
| Irrigation and farms | A radius-3 disc around each settlement, partly filled | 0.6% |
| Mines and wells | Bound to resource deposits, which are rare | 0.1% |
| Walls and forts | Frontier lines only | 0.2% |
| **Total, developed late game** | | **about 2.7%** |

An earlier report gives the rule: below about 5% of tiles, a sparse side
table wins; above it, swap to a rank-select structure with a dense
payload.[^8] 2.7% sits under the threshold but not far under, and a
pathological content set could exceed it.

**Recommendation: use the rank-select form directly and skip the hash
map.** The reasoning is that a tile upgrade is read by the pathing system
on a hot path, and a hash lookup on a hot path is the wrong shape at any
density. The rank-select form has no hashing, gives a dense payload array
and costs one extra `popcount`.

### 6.2 The storage

Three structures, and each answers a different question.

**Structure 1: an `upgrade_present` bitplane.** One bit per tile, which is
**2 MiB**. This is the index. Every bulk query reads it and never touches
the payload: "count upgraded tiles in this block" is a popcount of 4 `u64`
words. It aggregates into L1 as a popcount, which is a group, so the delta
update is exact.

**Structure 2: a rank-select payload.** Keep a per-block prefix count of
set bits as a `u32` array over 65,536 blocks, which is 256 KiB. The dense
payload index is `prefix[block] + popcount(word_prefix)`. The payload is a
`u32` `UpgradeSetId` per present tile. At 2.7% that is 453,000 entries and
**1.8 MB**. The upgrade set interns, exactly as the unit upgrade set does,
so a tile with a road and irrigation holds one identifier, not two fields.

**Structure 3: a dense `road_tier` field, 2 bits per tile.** This is the
exception, and it is deliberate. Roads are read by the pathing system on
every terrain-cost lookup. Routing that read through a rank-select
indirection would put a dependent load in the innermost pathing loop.
2 bits per tile as two bitplanes costs **4 MiB** and gives 4 tiers: none,
track, road and rail.

A dense `u16` upgrade column was considered and is rejected. It costs
33 MiB of memory and 33 MiB of traffic on every full-grid pass, against a
rich tile schema of 160 MiB, for a field that is 97.3% zero.

| Structure | Size | Answers |
|---|---|---|
| `upgrade_present` bitplane | 2 MiB | Bulk counts, pyramid aggregation, selector pruning |
| Rank-select prefix array | 256 KiB | Payload index |
| `UpgradeSetId` payload | 1.8 MB at 2.7% | Which upgrades, for yields and for display |
| `road_tier`, 2 bitplanes | 4 MiB | The pathing cost lookup |
| **Total** | **about 8 MiB** | |

Compare that against 33 MiB for the dense column alone.

### 6.3 How an upgrade modifies the terrain-cost matrix

The type table already holds a flattened terrain-cost matrix indexed by
`[unit_type][terrain]`.[^7] A road must modify it without adding a memory
pass.

**Recommendation: extend the matrix by one dimension rather than
post-processing its result.**

```
cost = terrain_cost[unit_type][terrain][road_tier]
```

At 256 types, 32 terrains and 4 tiers with a `u8` cost, the table is
**32 KiB**. It fits in L1 cache on a Neoverse core. The lookup is one load
with a computed index, exactly as before. A unit type that ignores roads
simply has equal values across the tier dimension, so no branch is needed.

Reject the alternative of applying a multiplier to the base cost. A
multiplier needs a multiply and a shift in the innermost pathing loop, and
it cannot express "this unit type cannot use a rail". The table can.

Two consequences follow, and both are already handled by existing
decisions.

- A `road_tier` write dirties the pathing chunk. The flow-tile cache drops
  every entry whose chunk is dirty.[^2] That mechanism needs no change.
- The intra-chunk portal costs need one Dijkstra per portal for the dirty
  chunk. The record bounds this repair work at K chunks per tick.[^2] Road
  construction is exactly the case that record anticipates.

### 6.4 How an upgrade modifies a yield

An upgrade contributes to the producer's effective stat through the same
modifier pipeline as a unit upgrade. There is no second mechanism.

The producer's `EffKey` gains no new component. The tile upgrade enters as
a **post-stage multiplier** under the section 3.5 rule, drawn from a table
indexed by the tile's `UpgradeSetId`. That keeps the effective-stat sharing
intact. The table is small, because the count of distinct tile upgrade sets
is small.

---

## 7. Saturation, caps and conservation

### 7.1 Why a clamp after a reduce is wrong

A clamp is a `min` or a `max`. Neither has an inverse, so neither supports
a delta update. That alone is a reason for care. The stronger reason is
informational.

Suppose 400 consumers demand 1,000 units of grain and the pool holds 700.
A clamp on the pool total gives one number: the pool is empty. It does not
say which consumers went without. A game needs that answer, because
starvation is a gameplay event with a threshold crossing behind it.

A per-entity loop that pays each consumer in turn is worse. Its result
depends on iteration order, late consumers receive nothing, and a parallel
version can drive the pool negative.[^7]

### 7.2 The resolve pass

Route every cap through the conserving `transfer` verb, which the
foundational record already specifies: sum the demand, compute one scale,
write the floor, then distribute the remainder by the largest-remainder
method in canonical order.[^2] This section states how upkeep uses it.

The pass runs per pool and per commodity. It is a map over
`pool_count * commodity_count` cells, and the presence mask from section
5.2 skips the empty ones.

```
available = stock + inflow                        // i64, exact
demand    = sum of |negative rates| for this pool  // computed in step 4
if available >= demand:
    pay every consumer in full
    surplus = available - demand
    stock   = min(surplus, capacity)
    spilled = surplus - stock                      // may be zero
else:
    run largest_remainder(available, demands, priority_order)
    stock = 0
```

`largest_remainder` works in **priority bands**, not over the whole set. A
consumer declares a `priority` byte in its recipe slot. The resolve pass
walks the bands in ascending order. Each band is paid in full while budget
remains. The first band that cannot be paid in full is split
proportionally by the largest-remainder method. Every band below it
receives zero.

This gives designers the mechanic they expect: feed the army before the
workshops. It is deterministic, because the band order is data and the
tiebreak inside a band is the ascending entity index.

### 7.3 Why integer arithmetic is what makes this exact

The largest-remainder method distributes `available` exactly. The
construction is:

1. Each consumer `i` receives `floor(demand_i * available / total_demand)`.
   All three values are `i64`, so the product needs 63 bits at most for the
   ranges in section 5, and the division is exact integer division.
2. The sum of the floors is at most `available`. The difference `r` is the
   remainder, and `r < consumer_count`.
3. Sort the consumers by the fractional part, descending, breaking ties on
   the ascending entity index. Give one extra unit to the first `r`.

The sum of the payments is then **exactly** `available`. Not approximately.
There is no accumulated drift over a million ticks, because there is no
rounding error to accumulate. A floating-point implementation of the same
algorithm loses or gains fractions of a unit on every tick, and over an
hour at 10 Hz that is 36,000 opportunities to drift.

Step 3 needs a sort. Sorting 400 consumers costs about 4 microseconds. The
sort is only over the band that split, not over the whole set, and only
when a shortfall occurs.

### 7.4 Overflow

Overflow is the mirror case and it uses the same verb. The excess above
capacity routes to a declared **spill target**: a neighbouring pool, a
faction reserve, or nowhere.

Where the target is a set of pools, the split is the same
largest-remainder call with the roles reversed. Where there is no target,
the excess is recorded as a `Spilled` event with the pool, the commodity
and the amount. **Never discard the excess silently.** A silent discard
makes a conservation test impossible, and a conservation test is the
cheapest way to find a bug in this subsystem.

Add that test: sum every pool, every in-flight quantity and every spill
record, and assert that the total equals the previous total plus the total
production minus the total consumption. It is an exact equality, because
every quantity is an integer.

---

## 8. Threshold crossings

### 8.1 The pattern

Construction completion, upgrade completion, starvation, bankruptcy and
structure destruction share one shape. A cheap dense pass computes a
predicate over every entity. A small fraction of entities cross the
threshold. An expensive handler then runs for that fraction only.

The pattern is stated precisely below.

**Phase A: the dense pass.** A map kernel over the entity columns. It
updates the accumulator and computes the crossing predicate as a branchless
comparison. It writes the predicate result into a **dense bitset**.

```rust
// One plane per event class. Sized to the entity capacity.
type CrossPlane = Box<[u64]>;   // ceil(capacity / 64) words

// Inside the span loop, for entity index `i` within span `s`:
let crossed = (accum[i] >= threshold[i]) as u64;
let base    = s * SPAN_LEN;
plane[(base + i) >> 6] |= crossed << ((base + i) & 63);
```

The write is a **plain store, not an atomic**. This is safe by construction,
because the span length is 4096, which is a multiple of 64.[^2] Each span
therefore owns whole `u64` words, and no two workers ever touch one word.
The record's requirement of disjoint outputs is met exactly.

**Barrier.**

**Phase B: the sparse pass.** Scan the plane in **ascending word order**.
For each non-zero word, extract set bits with `trailing_zeros`, which gives
ascending bit order. Run the handler for each entity in that order.

### 8.2 The ordering rule

> **The handler runs in ascending entity index order. Nothing else.**

The order does not depend on the thread count, on the work-stealing order,
or on which worker set which bit. Bitwise OR into disjoint words is exactly
commutative, so the plane's contents are identical at any thread count, and
the scan order is fixed by the index.

Phase B is serial by default. Section 8.4 gives the condition under which
it may be parallel.

### 8.3 Cost

| Item | Value |
|---|---|
| Plane size at 1M entities | 128 KiB |
| Clear cost, one `memset` | about 5 microseconds |
| Phase A predicate and store, per entity | 1 to 2 ns |
| Phase A total at 1M entities | **1 to 2 core-ms** |
| Phase B scan, all words zero | about 4 microseconds |
| Phase B scan, 1% of bits set | about 16 microseconds |
| Phase B handler at 1% and 200 ns each | **2 core-ms** |

Use one plane per event class. Five classes cost 640 KiB and remove every
re-test from the handlers. That is the right trade: 640 KiB is nothing
against a 160 MiB tile grid, and a handler that must re-test its own
condition is a handler that will drift from the predicate.

Popcount the whole plane before phase B to size the work. On the target,
`count_ones` compiles to a NEON sequence rather than one instruction, so
count a whole block at once and accumulate across it rather than counting
word by word.[^1]

### 8.4 When phase B may be parallel

Phase B may run in parallel only when the handler's writes are disjoint per
entity. Construction completion writes the structure's own row and marks
one tile, so it qualifies if the tile writes are collected as events rather
than applied directly. Bankruptcy writes a faction-wide value, so it does
not qualify and it must stay serial.

**Classify each handler at the point it is written**, in the same three
classes the record already defines for invariants: region-local, global
scalar and cross-region.[^2] The classification is cheap now and expensive
later.

### 8.5 Why a work list is worse

A per-thread `Vec` of crossing entities is the obvious alternative. Reject
it. Concatenating per-thread lists is deterministic only for a fixed thread
count. Concatenating per-span lists is deterministic but needs a prefix sum
over the span lengths, and the result is exactly the ascending index order
the bitset already gives. **The bitset is already sorted.** It also costs
128 KiB for any density, where a work list costs 4 bytes per crossing and
needs a capacity guess.

---

## 9. The frame-loop placement

The economy pass is not one block of work. The transport solve sits in the
middle of it, so it splits into two parts around that solve.

| Phase | Work | Reads | Writes |
|---|---|---|---|
| 5a | Effective-stat table rebuild, if a faction's modifiers changed | modifier lists | effective-stat table |
| 5b | Adjacency stencil recompute for dirty producers and movers | tile columns | adjacency column |
| 5c | Rate evaluation and segmented reduce into per-pool inflow and demand | entity columns, effective-stat table | pool inflow, pool demand |
| — | **The transport solve runs here.** | pool supply and demand | delivered quantities |
| 5d | Resolve pass: caps, priority bands, largest-remainder split | pool stock, delivered, demand | pool stock, paid amounts |
| 5e | Threshold pass: phase A dense predicate, phase B sparse handlers | accumulators | crossing planes, events |
| 7 | Pyramid update, unchanged | | |

The whole sequence sits inside the write phase of the frame loop, which is
correct: every step reads events and column values and writes column
values. No step reads a value another step wrote in the same parallel
region.

Step 5a runs in ascending faction order. Step 5b runs in ascending producer
index order. Step 5c runs as a segmented reduce with a boundary fixup in
ascending task order. Step 5d runs in ascending pool order and then
ascending commodity order. Step 5e runs in ascending entity index order.
**Every step has a stated total order.**

Give the economy a period and a phase offset, as the record requires. The
record proposes economy every 10 ticks.[^2] This report agrees, with one
exception: the threshold pass for combat-relevant crossings, such as a
structure destroyed, must run every tick. Split the period by event class,
not by kernel.

---

## 10. Cost model

The figures below assume one million entities with upkeep, 50,000
structures with production, 5,000 pools, 64 commodities and a mean recipe
of 2.2 slots. They give core-milliseconds, which the record identifies as
the reliable column, and wall-milliseconds at 12 cores for comparison with
the existing table.[^2]

### 10.1 Per economy tick

| Kernel | Scale | Core-ms | Wall-ms (12c) |
|---|---|---|---|
| Effective-stat rebuild, amortised | 4,800 rows x 32 fields | 0.05-0.25 | 0.01-0.03 |
| Adjacency stencil recompute | 6,800 static + 30,000 movers | 0.2-1.4 | 0.05-0.2 |
| Config lookup and recipe gather | 1M entities | 3-6 | 0.3-0.6 |
| Rate evaluation and post-stage multipliers | 2.2M slots | 1-2 | 0.1-0.2 |
| Segmented reduce into pools | 2.2M deltas | 2-4 | 0.2-0.4 |
| Resolve pass: caps and transfer | 320,000 pool-commodity cells | 1-3 | 0.1-0.3 |
| Threshold phase A, dense predicate | 1M entities | 1-2 | 0.1-0.2 |
| Threshold phase B, sparse handlers | 1% crossing | 2 | 2 (serial) |
| Pool and faction roll-up | 5,000 pools | 0.1 | 0.02 |
| **Total** | | **10.4-20.8** | **2.9-3.9** |

Phase B dominates the wall-clock column because it is serial. That is the
correct place to spend effort if the budget breaks, and section 8.4 states
the condition under which it parallelises.

### 10.2 Bandwidth check

The pass reads about 12 bytes per entity: the `ConfigId`, the `PoolId`, the
accumulator and the adjacency value. At one million entities that is 12 MB.
The record states that a full-map pass over 16.7 million tiles costs at
least 3.3 ms of pure bandwidth.[^2] This pass reads 12 MB against a tile
pass of 160 MB, so it costs about 0.25 ms of pure bandwidth. **The economy
pass is latency-bound on the gathers, not bandwidth-bound.** Sorting
entities by `ConfigId` inside a span would convert the gather to a
sequential read. Measure before doing it; the entities are already sorted
by pool, and pool correlates with type.

### 10.3 Fit against the existing budget

At a period of 10 ticks, the economy adds **1.0 to 2.1 core-ms and 0.29 to
0.39 wall-ms** to the mean tick. The record's current total is 90 to 360
core-ms and 12 to 46 wall-ms.[^2] The economy is therefore about 1% of the
mean tick, and a 10.4 to 20.8 core-ms spike on every tenth tick.

**Stagger the economy by faction.** Run one eighth of the factions on each
tick with a phase offset. The spike then flattens to 1.3 to 2.6 core-ms per
tick with no change to the total. The record already requires a period and
a phase offset per system, so this needs no new machinery.

Add this row to the record's per-tick cost table:

| Work | Scale | Core-ms | Wall-ms (12c) |
|---|---|---|---|
| Economy: production, upkeep, caps, thresholds | 1M entities, period 10, staggered | 1.3-2.6 | 0.4-0.6 |

---

## 11. The interface to the transport subsystem

Capacity limits on movement of goods are edge capacities in the transport
network. They are not this subsystem's concern. This section states exactly
what this subsystem computes and hands over.

### 11.1 What this subsystem provides

| Array | Shape | Type | Meaning |
|---|---|---|---|
| `pool_tile` | `[pool]` | `u32` | The pool's location, as a tile index |
| `pool_faction` | `[pool]` | `u16` | The pool's owner |
| `pool_supply` | `[pool][commodity]` | `i64` | Net production this tick, after modifiers, before transport |
| `pool_demand` | `[pool][commodity]` | `i64` | Gross consumption demand this tick |
| `pool_demand_band` | `[pool][commodity][band]` | `i64` | Demand split by priority band |
| `pool_stock` | `[pool][commodity]` | `i64` | Stock at the start of the tick |
| `pool_capacity` | `[pool][commodity]` | `i64` | The ceiling |
| `pool_present` | `[pool]` | `u64` | Which commodities are non-zero. A prune mask. |
| `edge_rate_contrib` | `[structure]` | `(u32, i32)` | An entity index and its effective throughput rate as `Fix32` |
| `road_tier` | 2 bitplanes over tiles | bits | The road tier per tile |
| `terrain_cost` | `[type][terrain][tier]` | `u8` | The movement cost table |

`edge_rate_contrib` is the important one. A harbour's unload rate is an
**effective stat of the harbour**, so this subsystem computes it through
the same memoised pipeline as every other stat. The transport subsystem
maps the structure to the edge it serves and sums the contributions. **This
subsystem does not know what an edge is.**

### 11.2 What this subsystem consumes

| Array | Shape | Type | Meaning |
|---|---|---|---|
| `delivered` | `[pool][commodity]` | `i64` | Quantity that arrived, after the flow solve |
| `shipped` | `[pool][commodity]` | `i64` | Quantity that left |

### 11.3 The ordering contract

The economy runs in three parts within one tick, in this order:

1. This subsystem computes `pool_supply`, `pool_demand` and
   `edge_rate_contrib`.
2. The transport subsystem solves the flow and writes `delivered` and
   `shipped`.
3. This subsystem runs the resolve pass and the threshold pass, using
   `delivered`.

Conservation is checked across all three parts together, not within one.
The exact equality is: the change in total stock equals total production
minus total consumption minus total spill. Every term is an `i64`, so the
check is an exact equality, not a tolerance.

---

## 12. Recommended ADR decision block

**This section is ready to apply to the foundational architecture record.
It is written in that record's style and continues its numbering, which
currently ends at D50.**

---

#### D51. Production and upkeep are one kernel with signed rates and two accumulators

A type declares a **recipe**: up to 8 slots, each holding a commodity
identifier, a signed `Fix32` rate per tick, and a priority band. Recipes
intern to a `RecipeId(u32)`, exactly as upgrade sets do. Growth, decay,
research accumulation and pollution use the same recipe form.

The kernel is five steps: map the entity to its configuration, gather the
effective recipe, map each slot to a signed delta, segmented-reduce by
pool, and update the pool.

**The reduce writes two accumulators, not one.** Production sums into
`pool_supply`. Consumption sums into `pool_demand`. They resolve against
each other in D55, because a single signed total loses the information
about which consumers were denied.

Steps 1 to 3 are read-only and belong in the write phase after the events
land. Do not run them in the read phase; they read values that phase 5
writes.

#### D52. Modifier order is a schema property. Flat and percent need no order

The five stages of the modifier pipeline stay as D4 states them. Add four
rules.

1. Flat and percent modifiers sum integers, so their collection order does
   not matter. Never sort them.
2. Multiplier categories fold in the order of one compile-time constant
   array. A modifier names its category. The fold walks the array, not the
   data. Two modifiers in the same category fold in ascending source
   identifier order.
3. **Cap the multiplier categories at 8.** Each category is one multiply
   and one rounding step in every effective-stat computation.
4. Clamp to `field_max` after each multiplier category, not only at the
   end. **This position is part of the contract.** Changing it later
   invalidates every replay.

Every shipped modifier system that this design surveyed fixes the stage
order in the schema and lets no content author reorder it. Follow that.

The bake step must assert that `field_max * (65536 + max_pct_sum)` fits in
62 bits for every field. Put the assertion in the field registry macro.

#### D53. Effective stats are memoised per configuration, not per entity

The key is the triple `(UnitTypeId, FactionId, UpgradeSetId)`. Technology
and policy enter through the faction identifier, because the engine holds
one modifier list per faction.

Intern the triple to a per-faction `ConfigId(u16)` and **store the
`ConfigId` as a column on the entity**. The per-entity work is then one
2-byte sequential read and one gather into a small table.

| Distinct configurations | Table size at 144 B per row | Hit rate at 1M entities |
|---|---|---|
| 190 | 27 KiB | 99.98% |
| 900 | 127 KiB | 99.91% |
| 4,800 | 675 KiB | 99.52% |

The expected hit rate is above 99% in every realistic case, because
upgrade sets correlate strongly with type. The break-even against
per-entity evaluation is at one entity per configuration, so the table is
never the slower choice.

Rebuild the whole table for a faction when that faction's modifiers change.
The rebuild costs about 2.3 core-ms at 4,800 configurations, and it fires a
few times per minute rather than a few times per tick. Rebuild in ascending
faction order at the frame barrier. Do not build partial invalidation in
version 1.

**Monitor the distinct-configuration count and fail a test above 8,192.**
This is the same failure mode D40 already names for the upgrade intern
table: content that gives each unit a distinct upgrade set destroys the
sharing. One monitor covers both.

**The schema rule that protects the sharing:**

> A per-entity input may enter the pipeline only as a post-stage multiplier
> drawn from a small fixed table. It may never select which modifiers apply.

Cap the post-stage multipliers at 4. Health enters as a tier, quantised
into 8 bands. **That quantisation is a design decision, not a rounding
artefact.** A unit at 51% health and a unit at 60% health fight
identically. Record it, so nobody reports it as a bug.

#### D54. Adjacency is cached per producer behind the existing dirty bits

Two adjacency questions exist and they are different.

**Block adjacency is free.** The L1 terrain histogram already answers "how
many forest tiles are in this block" as one 2-byte read. Use it for
regional yields. It costs nothing, because the pyramid already maintains
it.

**Radius-1 adjacency needs a cache.** Reading 6 neighbours costs about
36 ns per producer as a weighted mean, because 76.6% of tiles have all 6
neighbours inside their own 16x16 block and 23.4% cross a block boundary.
At 1M entities that is 36 core-ms per tick, which exceeds the whole economy
budget.

Cache the contribution as one `Fix32` column, which is 4 bytes per
producer. Recompute only when the L1 terrain dirty bit fires, and rescan
**the dirty block and its 8-block halo**, because an edge producer reads
across the boundary. Missing the halo gives a silent one-block-wide error.

At 1,000 terrain writes per tick the cached recompute costs 0.25 core-ms
against 1.8 core-ms uncached at 50,000 producers. Mobile producers reuse
the "changed tile this tick" set that field of view already maintains.

#### D55. Caps route through `transfer` in priority bands. A clamp is forbidden

A clamp after a reduce discards the information about who was denied.
Starvation and bankruptcy are gameplay events, so that information is the
output, not a detail.

The resolve pass runs per pool and per commodity, pruned by a `u64`
presence mask:

- If supply covers demand, pay in full and clamp the **surplus** into
  stock. Record the excess above capacity as a `Spilled` event with the
  pool, the commodity and the amount. **Never discard it silently.**
- If supply does not cover demand, walk the priority bands in ascending
  order. Pay each band in full while budget remains. Split the first band
  that cannot be paid in full by the largest-remainder method of D42, with
  the tiebreak on the ascending entity index. Every band below it receives
  zero.

The largest-remainder split is **exact**: the sum of the payments equals
the available quantity, with no residue. That is only true in integer
arithmetic. A float implementation drifts on every one of the 36,000 ticks
in an hour at 10 Hz.

Add the conservation test. The change in total stock equals total
production minus total consumption minus total spill. It is an exact
equality, so the test needs no tolerance.

#### D56. Threshold crossings use a dense bitset and a sparse ascending scan

Construction completion, upgrade completion, starvation, bankruptcy and
structure destruction share one shape.

**Phase A, dense and parallel.** A map kernel over the entity columns
computes the predicate as a branchless comparison and writes the result
bit with a **plain store**, not an atomic. This is safe because the span
length of 4096 is a multiple of 64, so each span owns whole `u64` words and
no two workers touch one word.

**Barrier.**

**Phase B, sparse.** Scan the plane in ascending word order and extract set
bits with `trailing_zeros`.

> **The ordering rule: handlers run in ascending entity index order, and in
> no other order.** The order does not depend on the thread count.

The bitset type is `Box<[u64]>`, one word per 64 entities, which is 128 KiB
at one million entities. Use **one plane per event class**; five classes
cost 640 KiB and remove every re-test from the handlers.

Reject a per-thread work list. Concatenating per-thread lists is
deterministic only at a fixed thread count, and a per-span list needs a
prefix sum to reproduce the order that the bitset already has.

Phase B is serial by default. It may run in parallel only when the
handler's writes are disjoint per entity. Classify each handler as
region-local, global scalar or cross-region when it is written, using the
three classes D24 already defines.

#### D57. Stockpiles live per settlement. The commodity ceiling is 64

Per tile costs 8.6 GB at 64 commodities and is rejected. Per faction alone
cannot express local scarcity, so the transport network has nothing to
solve, and it is rejected as the only level. **The stockpile lives in a
pool, and a pool is a settlement.** At 5,000 pools and 64 commodities the
stock array is 2.56 MB of `i64`. Keep a per-faction roll-up as a derived
group, which costs 4 KiB and updates by the same exact delta.

**The commodity ceiling is 64.** Four reasons give that number: a presence
mask is one `u64`; 64 `i64` values are exactly 8 cache lines; the L1
summary budget of 256 bytes per cell cannot hold more; and it matches the
capability-mask precedent, where a `u64` makes the ceiling type-checked.
Rare commodities go in a sparse side table and do not tick at full rate.

**The reduce uses no atomics.** Keep producers sorted by pool identifier;
structures are static, so the sort is rebuilt only in the structural phase.
Each task then owns a contiguous pool range, and only the two boundary
pools need a fixup, combined in ascending task index order. Give mobile
entities a `PoolId` column so they sort the same way.

An atomic scatter-add was rejected because a relaxed `fetch_add` emits a
real barrier on the target's weak memory model. Per-worker private
accumulators were rejected at 164 MB for 64 workers.

#### D58. Tile upgrades are a bitplane index with a rank-select payload. Roads are dense

At a plausible 2.7% of tiles carrying an upgrade in a developed late game,
a dense `u16` column costs 33 MiB of memory and 33 MiB of traffic on every
full-grid pass for a field that is 97.3% zero. Reject it.

| Structure | Size | Answers |
|---|---|---|
| `upgrade_present` bitplane | 2 MiB | Bulk counts, pyramid aggregation, pruning |
| Rank-select prefix, per block | 256 KiB | The payload index |
| `UpgradeSetId` payload, dense | 1.8 MB at 2.7% | Which upgrades, for yields |
| `road_tier`, 2 bitplanes | 4 MiB | The pathing cost lookup |
| **Total** | **about 8 MiB** | |

Use the rank-select form directly rather than a hash map, even below the 5%
threshold that report 01 gives. A tile upgrade is read on the pathing hot
path, and a hash lookup does not belong there at any density.

**Roads are the deliberate exception and stay dense**, at 2 bits per tile.
Routing a pathing read through a rank-select indirection would put a
dependent load in the innermost loop.

**Extend the terrain-cost matrix by one dimension. Do not post-process its
result.**

```
cost = terrain_cost[unit_type][terrain][road_tier]
```

At 256 types, 32 terrains and 4 tiers with a `u8` cost, the table is 32 KiB
and fits in L1 cache. A multiplier form was rejected: it costs a multiply
in the innermost pathing loop and it cannot express "this type cannot use a
rail".

A `road_tier` write dirties the pathing chunk, so the flow-tile cache and
the bounded portal repair of D44 handle invalidation with no change.

A tile upgrade modifies a yield through the **same** modifier pipeline,
entering as a post-stage multiplier under D53. There is no second
mechanism.

#### D59. The economy runs in three parts around the transport solve

The economy is not one block of work. It splits around the flow solve.

| Step | Work | Order |
|---|---|---|
| 5a | Effective-stat rebuild, if modifiers changed | ascending faction |
| 5b | Adjacency stencil recompute | ascending producer index |
| 5c | Rate evaluation and segmented reduce | ascending task, boundary fixup |
| — | **The transport solve runs here** | — |
| 5d | Resolve: caps, bands, largest remainder | ascending pool, then commodity |
| 5e | Threshold: dense predicate, then sparse handlers | ascending entity index |

Every step has a stated total order. None reads a value that another step
writes in the same parallel region.

Run the economy with a period of 10 ticks, as D29 proposes, and **stagger
it by faction** so one eighth of the factions run on each tick. That
converts a 10.4 to 20.8 core-ms spike every tenth tick into 1.3 to
2.6 core-ms every tick. Split the period by event class, not by kernel:
the threshold pass for combat-relevant crossings, such as a structure
destroyed, runs every tick.

Add this row to the per-tick cost budget:

| Work | Scale | Core-ms | Wall-ms (12c) |
|---|---|---|---|
| Economy: production, upkeep, caps, thresholds | 1M entities, period 10, staggered | 1.3-2.6 | 0.4-0.6 |

The pass reads about 12 bytes per entity, which is 12 MB at one million
entities, against a full tile pass of 160 MB. **The economy pass is
latency-bound on the configuration gather, not bandwidth-bound.**

---

## 13. Open questions

**OQ17. How many settlements does the target scenario hold?**
Every storage figure in D57 scales with this number. 5,000 pools cost
2.56 MB at 64 commodities. 50,000 pools cost 25.6 MB and make the resolve
pass a 3.2-million-cell map rather than a 320,000-cell map. 5,000 and
50,000 are different designs.
*Blocks:* the resolve-pass cost line and the per-worker fixup size in D57.

**OQ18. Is upkeep per unit or per formation?**
This report assumes per unit, which gives one million entities in the rate
kernel. If upkeep is per formation, the entity count in the kernel drops by
one to two orders of magnitude and the economy becomes free. It also
changes whether a `PoolId` column on the unit is needed at all.
*Blocks:* the whole cost model in section 10.

**OQ19. What fraction of tiles carries an upgrade at the end of a long
game?**
This report estimates 2.7%. That estimate decides rank-select against a
dense column in D58. Above about 10% the dense `u16` column becomes
competitive again.
*Blocks:* D58 and the tile byte budget.

**OQ20. How many multiplier categories does the content need?**
D52 caps them at 8. Each category is a real cost in every effective-stat
computation. A ninth is a content design question and it should be
answered before the bake step is written, because the constant array is
compile-time.
*Blocks:* D52 and the effective-stat rebuild cost.

**OQ21. May a threshold handler be parallel?**
D56 makes phase B serial by default and gives 2 core-ms of serial work at a
1% crossing rate. Construction completion probably qualifies for parallel
execution and bankruptcy does not. The classification must be made per
handler and it decides the largest line in the wall-clock column.
*Blocks:* the wall-ms column in section 10.

---

## References

[^1]: ADR-0001 background report 07, Target Platform and Value Types. `docs/adrs/background/adr-0001/07-target-platform-and-value-types.md`
[^2]: ADR-0001, Foundational Architecture, decisions D4, D5, D9, D10, D13, D17, D18, D19, D24, D27, D28, D29, D40, D41, D42, D44, D50, and the byte and per-tick budget tables. `docs/adrs/draft/adr-0001-foundational-architecture.md`
[^3]: Paradox Interactive forum, discussion of additive against multiplicative modifiers in the Clausewitz engine. https://forum.paradoxplaza.com/forum/threads/additive-bonuses-vs-multiplicative-bonuses.1144836/
[^4]: RimWorld modding documentation, XML definition files and inheritance. https://rimworldwiki.com/wiki/Modding_Tutorials/XML_Defs and https://spdskatr.github.io/RWModdingResources/abstracts.html
[^5]: Factorio data-stage documentation, prototype definition and the one-time bake. https://lua-api.factorio.com/latest/types/Data.html
[^6]: Civilization VI modding community documentation, the Modifiers and RequirementSets database tables. Source not verified at the time of writing; the CivFanatics modding forum is the usual location.
[^7]: ADR-0001 background report 04, Selector Engine, Verb Vocabulary, and Data-Driven Types, sections 4, 6.2, 6.3, 6.4 and 8.3. `docs/adrs/background/adr-0001/04-selector-engine-and-verbs.md`
[^8]: ADR-0001 background report 01, ECS and Memory Layout, section 8 on sparse side tables and the tile byte budget. `docs/adrs/background/adr-0001/01-ecs-and-memory-layout.md`
