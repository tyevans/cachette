# Fog of War Representation

Research report 08 for the foundational architecture decision record.

## 0. Context

Cachette is a world simulation engine. The core is Rust. The control plane
is Python. The engine simulates a hex world at three levels of detail.
Level 0 holds individual tiles. Level 1 summarises blocks of 256 tiles.
Level 2 summarises blocks of 65,536 tiles. The target scale is 16,777,216
tiles and one million units.[^1]

Fog of war records what each faction sees. It holds two facts for each
tile. The first fact is "explored": the faction saw this tile at some
earlier time. The second fact is "visible": the faction sees this tile
now.

The draft decision record states decision D48. D48 stores three dense
arrays for each faction. The arrays are an explored bitset at 2.10 MB, a
visible bitset at 2.10 MB, and a `u8` visibility counter at 16.8 MB. The
total is 21.0 MB for each faction. This is 168 MB at eight factions and
4.2 GB at two hundred factions. Open question OQ5 asks for the faction
ceiling because of this cost.[^2]

This report compares three designs and recommends one.

- **Option A.** A block-adaptive sparse tile set for each faction.
- **Option B.** A tree of shared masks, with a root, alliance nodes, and
  faction leaves.
- **Option C.** One transposed grid. Each tile holds a `u64` faction
  bitmask. Bit `N` set means faction `N` sees this tile.

### 0.1 The four findings

**Finding 1. The `u8` counter is an artifact and must go.** The counter
exists to avoid clearing and re-scattering a dense bitset. A
block-granular rebuild removes the need for it. The counter also costs
about 868,000 cache-missing writes for each large faction each tick, which
breaks its own budget line.

**Finding 2. Option B is retired, not rejected on cost.** A per-faction
relation row named `vision_shared` makes shared vision a derived quantity.
There is nothing left for a shared node to store. Section 5 gives the
arithmetic that also rejects it on cost.

**Finding 3. The faction bitmask is the right primitive. The level 0
transposed grid is the wrong materialisation of it.** The transpose is not
a memory saving. It costs a fixed 256 MiB and it moves 64 bits of traffic
to update 1 bit of information. Every advantage the transpose offers
survives at level 1 granularity, where the same structure costs 1 MiB
instead of 256 MiB.

**Finding 4. The recommendation is a hybrid.** Store option A as the
authority. Derive a `u64` faction mask at level 1, and derive a `u64`
"who sees me" mask for each unit. This buys every stated advantage of
option C for under 1 percent of its size.

---

## 1. Terms

**Tile index.** A `u32` that names one tile. The index is block-tiled.
The low 16 bits give the offset inside one level 2 cell. The high 8 bits
give the level 2 cell number.

**Level 1 cell.** A block of 256 tiles. The map holds 65,536 of them.

**Level 2 block.** A block of 65,536 tiles. The map holds 256 of them. A
dense bitset for one block is 8,192 bytes.

**Disc.** The set of tiles that one observer sees. An observer at sight
radius `r` sees `1 + 3r(r+1)` hexes on an open hex map. Radius 4 gives 61
hexes. Radius 8 gives 217 hexes. Radius 12 gives 469 hexes. Radius 16
gives 817 hexes.

**Leaf.** One of four storage forms for one layer inside one level 2
block. The forms are `Empty`, `Array`, `Bits` and `Full`.

**Stamp.** The write of one disc into one layer.

**Observer tile.** A tile that holds at least one unit with sight. The
engine shares a disc across all units on one observer tile at one
quantised radius.[^3]

**Faction mask.** A `u64` in which bit `N` refers to faction `N`.

---

## 2. Test of the sparse analysis

### 2.1 The arithmetic that holds

The session lead gave four claims about a sparse per-faction form. All
four are correct.

| Claim | Stated | Computed | Verdict |
|---|---|---|---|
| Sorted `u32` at 30,000 tiles | 120 KB | 120,000 B | Correct |
| Sorted `u32` at 250,000 tiles | 1.0 MB | 1,000,000 B | Correct |
| Crossover against a dense bitset | 525,000 tiles | 524,288 tiles | Correct |
| Crossover as a fraction of the map | about 3 percent | 3.125 percent | Correct |

A dense bitset for the map is 2,097,152 bytes. A sorted `u32` list of `n`
tiles is `4n` bytes. The two costs are equal at 524,288 tiles.

### 2.2 The density claim needs a correction

The lead cites 0.18 percent density and about 30,000 visible tiles. The
figure of 30,000 comes from the per-tick cost budget. That budget line
counts field-of-view recomputations, not visible tiles.[^4] The two
numbers are different. A recomputation happens only for an observer that
changed tile. A visible tile stays visible while the observer stays put.

The visible-set size is therefore larger than 30,000 for a large faction.
Section 2.3 models it.

### 2.3 Four scenarios

Let `T` be the count of distinct observer tiles for one faction. Let `r`
be the quantised sight radius. Let `S` be the stamp count, which is `T`
times the disc area. Let `V` be the count of distinct visible tiles after
the union removes overlap.

| Scenario | `T` | `r` | Disc | `S` | `V` | `V` / map |
|---|---|---|---|---|---|---|
| A. Early game | 30 | 4 | 61 | 1,830 | 1,464 | 0.009% |
| B. Mid game | 600 | 8 | 217 | 130,200 | 71,610 | 0.427% |
| C. Late empire | 20,000 | 8 | 217 | 4,340,000 | 303,800 | 1.811% |
| D. Pathological | 2,000 | 16 | 817 | 1,634,000 | 1,470,600 | 8.765% |
| E. Whole map | — | — | — | — | 16,777,216 | 100% |

Scenario C models an empire with 125,000 units stacked onto 20,000
observer tiles in dense clusters. The overlap factor is 0.07 because the
clusters are tight. Scenario D models 2,000 wide-radius scouts spread
apart. The overlap factor is 0.9 because the discs barely touch.

### 2.4 Where a sorted list loses

| Scenario | Sorted `u32` | Dense bitset | Winner |
|---|---|---|---|
| A | 5.9 KB | 2.10 MB | Sparse, by 358x |
| B | 286 KB | 2.10 MB | Sparse, by 7.3x |
| C | 1.22 MB | 2.10 MB | Sparse, by 1.7x |
| D | 5.88 MB | 2.10 MB | Dense, by 2.8x |
| E | 67.1 MB | 2.10 MB | Dense, by 32x |

A plain sorted list loses above 3.125 percent density. Scenario D is a
plausible late-game state for a faction with wide sensors. A plain sorted
list is therefore not safe on its own. This argues for a density-adaptive
container, which section 3 gives.

### 2.5 The counter claim needs a correction

The lead states that a per-tick rebuild is cheap enough to delete the
counter. This is true for scenarios A and B. It is false for scenario C.

A full rebuild writes `S` stamps. At about 1 nanosecond for each stamp
into a cache-resident scratch bitmap, the costs are 0.002 ms at scenario
A, 0.13 ms at scenario B, 4.3 ms at scenario C, and 1.6 ms at scenario D.
The fog budget line allows 3 to 10 core-ms for all factions together.[^4]
Scenario C alone spends 4.3 core-ms. A full rebuild for every faction
every tick breaks the budget.

**The correct form is a block-granular rebuild.** Section 3.4 gives it.
The lead's conclusion holds. The counter goes away. The reason is the
block granularity, not the rebuild alone.

### 2.6 The hidden cost of the current counter

The dense `u8` counter is 16.8 MB. An update writes single bytes at
scattered tile indices. Each write misses the last-level cache. A cache
miss on a Graviton core costs about 80 to 120 nanoseconds.[^5]

Scenario C with 10 percent churn moves 2,000 observers. Each move removes
one disc and adds one disc. That is 868,000 counter writes. At 100
nanoseconds for each write this costs 87 core-ms for one faction.

**The current D48 does not meet its own budget line at scenario C.** The
budget line assumes 30,000 deltas. The real delta count is 868,000. This
is a second, independent reason to remove the counter.

---

## 3. Option A — the block-adaptive tile set

### 3.1 One container, four leaf forms

Store each layer for each faction as 256 level 2 blocks. Each block holds
one leaf.

```rust
#[repr(u8)]
enum Leaf {
    Empty,
    Array(Arc<[u16]>),      // sorted, ascending, no duplicates
    Bits(Arc<[u64; 1024]>), // 8,192 bytes, 64-byte aligned
    Full,
}

struct TileLayer {
    blocks: [Leaf; 256],    // index is the level 2 cell number
}
```

The choice rule is fixed and deterministic. Use `Empty` at population 0.
Use `Full` at population 65,536. Use `Array` at population 1 to 4,096.
Use `Bits` above 4,096. The threshold of 4,096 is where a `u16` array and
an 8,192-byte bitmap cost the same. The header cost is 16 bytes for each
block, so 4,096 bytes for each layer.

### 3.2 Why the 2^16 split is not arbitrary here

Research report 04 rejected a general Roaring library for selector
results. The stated reason is that the Roaring split at 2^16 cuts an
arbitrary key space, while the storage layout already defines a natural
split.[^6]

That objection does not apply to fog. The map holds 2^24 tiles. A level 2
cell holds 2^16 tiles. The map holds exactly 256 level 2 cells. A split at
2^16 is therefore the level 2 cell boundary, not an arbitrary cut.

Three consequences follow. A block maps to one pyramid cell, so a pyramid
descent writes its result in place. The level 2 dirty bitset is 256 bits,
so a dirty-block scan is four `u64` words. A block bitmap is 8,192 bytes
and fits in the level 1 data cache during a rebuild.

### 3.3 Alignment and popcount on the target

Align each `Bits` leaf to 64 bytes. Graviton uses a 64-byte cache line.[^7]
An 8,192-byte leaf is exactly 128 lines, so two leaves never share a line.
Parallel writes to different blocks therefore never false-share.

aarch64 has no scalar popcount instruction. A `u64` popcount routes
through the vector unit as a move, a `CNT` per byte, and an `ADDV`
reduction.[^7] Count a whole leaf in one pass with a NEON accumulator, and
reduce once at the end. Never popcount one word inside a loop.

### 3.4 The update, without a counter

The frame loop splits phases into a read half and a write half. Phases 1
to 4 read the world and write only events. Phases 5 to 8 write the world.
Fog runs in phase 8.[^3] The update has five steps.

1. **Collect.** Walk the units that changed tile this tick. Produce the
   set of observer tiles that gained sight and the set that lost it.
2. **Mark.** For each changed observer, mark every level 2 block that its
   disc touches. A disc at radius 16 touches at most 4 blocks.
3. **Gather.** For each dirty block, collect every current observer tile
   whose disc touches that block. Sort the list by tile index.
4. **Rebuild.** Clear an 8,192-byte scratch bitmap. Apply each disc in
   list order with masked `OR` writes. Popcount the scratch bitmap in one
   batch. Choose the leaf form and store the result.
5. **Accumulate.** `OR` the new visible leaf into the explored leaf for
   the same block. Explored is monotonic, so this never removes a tile.

No structure holds a reference count. Removal is correct because step 4
rebuilds the block from the current observer set.

### 3.5 Cost of the block rebuild

The rebuild cost is the sum, over the dirty blocks, of the disc count for
that block times the disc area.

| Scenario | Dirty blocks | Discs in them | Stamps | Core-ms | Traffic |
|---|---|---|---|---|---|
| A | 2 | 30 | 1,830 | 0.002 | 32 KB |
| B | 8 | 220 | 47,740 | 0.05 | 131 KB |
| C | 40 | 6,000 | 1,302,000 | 1.3 | 655 KB |
| D | 30 | 500 | 408,500 | 0.4 | 491 KB |

The writes land in an 8,192-byte scratch bitmap that stays in the level 1
data cache. The traffic column counts the read and the write of the
scratch bitmaps only. This meets the 3 to 10 core-ms budget line. The
current dense counter does not, as section 2.6 shows.

### 3.6 Storage, by scenario

| Scenario | Visible leaves | Visible | Explored | Total |
|---|---|---|---|---|
| A. Early | 3 `Array` | 2.9 KB | 3.0 KB | 14 KB |
| B. Mid | 24 `Array` | 143 KB | 200 KB | 351 KB |
| C. Late empire | 32 `Bits`, 32 `Array` | 470 KB | 480 KB | 958 KB |
| D. Pathological | 90 `Bits`, 30 `Array` | 1.28 MB | 1.20 MB | 2.49 MB |
| E. Whole map | 256 `Bits` | 2.10 MB | 1 KB, all `Full` | 2.11 MB |

Totals include 8 KB of block headers. Scenario E shows the graceful end.
A faction that sees the whole map costs exactly the old dense visible
bitset, and its explored layer collapses to 256 `Full` leaves. The
previous design costs 21.0 MB in every one of these cases.

---

## 4. Option B — the tiered shared mask

This option is **retired**. Section 6.3 gives the reason: a
`vision_shared` relation row makes shared vision a derived quantity, so
there is nothing for a shared node to store. The cost analysis below is
kept short, because it is no longer the deciding argument.

### 4.1 The break-even formula

Let `k` be the count of factions under one shared node. Let `C` be the
count of shared tiles and `P` the count of private tiles for each faction.
Define the overlap fraction as `phi = C / (C + P)`. The saving fraction is:

```
saving = 1 - (C + kP) / (k(C + P)) = phi * (k - 1) / k
```

The saving can never exceed the overlap fraction. The factor `(k - 1) / k`
is 0.500 at 2 members, 0.875 at 8, 0.938 at 16, and 0.995 at 200. **Eight
members already reach 87.5 percent of the limit.** The gain from 8 members
to 200 is a factor of 1.14.

This refutes the framing that the tree is overhead at 8 factions and
decisive at 200. The driver is the overlap fraction, not the faction
count. If the tree does not pay at 8 factions, it does not pay at 200.

### 4.2 The absolute saving

Take the favourable case of 0.4 overlap for visible, 0.6 for explored,
and 4 members. That gives a 30.0 percent saving on visible and 45.0
percent on explored.

| Factions | Flat, scenario C | Tree | Saved |
|---|---|---|---|
| 8 | 7.66 MB | 4.76 MB | 2.90 MB |
| 64 | 61.3 MB | 38.0 MB | 23.3 MB |
| 200 | 192 MB | 119 MB | 73 MB |

Option A saves 4.2 GB at 200 factions. The tree then saves a further 73
MB. **Option A captures 96 percent of the available reduction.**

### 4.3 Three further objections

**The shared node needs a reference count.** Visibility churns every tick,
so the shared node changes every tick. Incremental maintenance needs to
know how many members still cover each shared tile. That count is a
reference count at group granularity. The tree therefore reintroduces the
exact structure that section 3 deletes.

**Reparenting is a full-set pass.** A faction that leaves an alliance must
union the old path into a new leaf, and the alliance node must shrink to
the intersection over the remaining members. Both are full passes over 256
blocks. At scenario C this is 2 to 4 core-ms on a player-triggered event.

**A point query walks a path.** Depth 3 means three leaf lookups instead
of one, for a 30 percent storage saving.

### 4.4 Tree against directed acyclic graph

Alliances give a tree, because each faction belongs to at most one
alliance. Arbitrary shared subsets give a directed acyclic graph. Finding
the sharing that minimises storage is a set-cover problem, which is
NP-hard. Running an approximation each tick is not affordable.

Recommend a fixed schema over a computed one, if a tree is built at all.
This report recommends building neither.

### 4.5 Sharing at container granularity

The alternative to the tree places an atomic reference count on each leaf
and copies on write. Option A already specifies this in section 3.1.

Test whether it dedups anything. Two factions share a `Bits` leaf only
when every observer disc in that block produces an identical bit pattern.
Allied units stand on different tiles, so their unions differ. Independent
factions almost never produce a bit-identical block.

**Container sharing does not dedup independent factions either.** It gets
the same near-zero benefit as the tree, for far less machinery. It does
pay in three specific cases, and all three are worth having: a fog
snapshot shares every unmodified leaf with the live state; a vassal
faction that inherits sight starts as 256 pointer writes; a new faction
clones a template.

Clone an `Arc` once for each block, never once for each tile. An ARM
atomic emits a real barrier under the weak memory model.[^7]

---

## 5. Option C — the transposed faction grid

Store one grid over all tiles. Each tile holds a `u64` faction mask. Bit
`N` set means faction `N` sees the tile. Store a second grid for explored.

### 5.1 The size arithmetic, verified

| Quantity | Value |
|---|---|
| Tiles | 16,777,216 |
| Bytes for each tile | 8 |
| Visible grid | 134,217,728 B = **128.0 MiB exactly** |
| Explored grid | 134,217,728 B = **128.0 MiB exactly** |
| Both grids | **256.0 MiB, fixed** |
| 64 separate dense bitplanes | 134,217,728 B = **128.0 MiB exactly** |

**The lead is correct, and the duality is exact.** Sixty-four factions at
one bit for each tile hold the same bit count as 16.7 million tiles at 64
bits each. The transpose is the same bits in a different order.

**This is not a memory optimisation. State that plainly.** The grid costs
64 bits for each tile whether the world holds 4 factions or 64. At 8
factions it is 8 times larger than 8 dense bitplanes. Against option A at
8 factions in the mid-game case it is 268 MB against 2.81 MB, which is 95
times larger. Its value is entirely in the access pattern.

The cost is also permanent. The decision record budgets about 260 MB for
everything else at 1M units.[^2] Option C adds 268 MB and makes fog the
largest line item in the engine in every game, including a two-player
game.

### 5.2 The five stated advantages, checked

**Advantage 1. "Which factions see tile T" is one 8-byte load.** True.
Option A answers the same question with one leaf lookup for each faction,
so 64 lookups and up to 256 cache misses. Option C wins this query by
about 250 times.

The question is how often the engine asks it. The common query is "can
faction F see tile T", which is one leaf lookup under option A and one
load plus one bit test under option C. Both are cheap. The all-faction
form matters for detection: "which hostile factions can see this unit".
Section 5.5 shows that this is better served by a per-unit projection than
by a per-tile grid.

**Advantage 2. It fits the pyramid machinery.** True, and important.
Bitwise `OR` is associative, commutative, idempotent, and integer-only. It
is a monoid, so it satisfies the exact-combination invariant with no
special case.[^1] A level 1 cell summary is the `OR` of its 256 tile
masks.

**This advantage does not require the level 0 grid.** The level 1 `OR`
mask is derivable from option A at the same cost: when a faction rebuilds
a block, set its bit in every level 1 cell that its new leaf touches. The
level 1 mask array is 65,536 cells times 8 bytes, which is **512 KiB for
all factions together**.

**Advantage 3. Selector pruning falls out.** True, and again it needs only
the level 1 mask. A descent tests `l1_mask[cell] & (1 << 3)` and skips the
cell when the bit is clear. This costs 512 KiB, not 128 MiB.

**Advantage 4. It dissolves the tiered mask.** True. Section 6.3 shows
that the `vision_shared` relation row dissolves it under option A as well,
so this advantage is not unique to option C.

**Advantage 5. NEON and cache lines.** True. A 64-byte line holds exactly
8 tile masks. Bulk `AND` and `OR` vectorise trivially at 2 masks for each
128-bit NEON register.

One correction. Counting how many tiles each faction sees is a *vertical*
popcount across 64 bit positions, not a `CNT` over bytes. A per-faction
cardinality therefore needs either one masked full-grid pass for each
faction, or a positional-popcount network. Option A gives every faction's
cardinality for free, because each leaf stores its population.

**An advantage the lead did not list.** Under option C a state hash of all
fog is one linear pass over two contiguous grids. Under option A the hash
must walk 256 leaves for each faction in a defined order. Option C is
simpler to hash and harder to get wrong. The advantage is real but small,
because option A's block order is already fixed.

**A second advantage the lead did not list.** Option C makes "is this tile
seen by anybody" a single test against zero. Option A needs the level 1 or
level 0 `OR` mask, which the hybrid provides at level 1 only.

### 5.3 Cost 1 — the dual query

"List every tile that faction F sees" becomes a scan of 128 MiB with a bit
test. At about 40 GB/s this is **3.36 ms of wall time**. That figure is
exactly the decision record's stated floor for a full-map pass, and the
record allows two or three such passes for each tick.[^4] Eight rendered
factions would spend 27 ms of wall time, which exceeds the whole tick
budget.

Now add pruning. Keep a level 1 `OR` mask of 512 KiB and a level 2 `OR`
mask of 2 KiB. Scan the masks, then scan only the touched level 1 cells.
A level 1 cell is 2,048 bytes in the grid.

| Scenario | Touched L1 cells | Bytes scanned | Wall time |
|---|---|---|---|
| B. Mid | 1,000 | 2.56 MB | 0.06 ms |
| C. Late empire | 4,000 | 8.70 MB | 0.22 ms |
| D. Pathological | 40,000 | 82.4 MB | 2.06 ms |

Pruning is enough for the clustered cases and not enough for scattered
scouts. Option A answers the same query by iterating its own container:
470 KB at scenario C, which is about 12 microseconds. **Option A is 18
times faster at scenario C and 170 times faster at scenario D.**

For rendering, the honest answer is that neither cost matters. A viewport
covers one or two level 2 blocks. Both options answer a viewport query in
kilobytes. The full-map dual query is an analytics operation, not a
rendering operation.

### 5.4 Cost 2 — write contention

Two factions that update the same tile touch the same `u64`. Two
approaches exist.

**Atomic `fetch_or`.** This is deterministic in its result, because `OR`
is commutative and idempotent, so the order of the writes does not change
the outcome. That is a genuine exception to the general ordering
rule.[^1] It is still the wrong choice. ARM has a weak memory model, so a
relaxed atomic emits a real barrier where x86 emits none.[^7] The engine
would issue about 10 million atomic operations for each tick at scenario C
across 8 factions. False sharing also becomes severe, because 8 adjacent
tiles share one line and adjacent tiles are exactly what a disc writes.

**Disjoint ranges.** Partition the grid by level 1 cell. Give each worker
a disjoint set of cells. Bucket every stamp by target cell first, then
apply. No atomic is needed and the outputs never overlap.

**The lead's preference for disjoint ranges is correct.** It satisfies the
required disjoint-output rule and it avoids the ARM barrier cost.

One further cost follows from the choice. Under option A the rebuild
scratch is an 8,192-byte bitmap that stays in the level 1 data cache.
Under option C the equivalent scratch for a level 2 block is 512 KiB,
which does not fit. Option C must therefore bucket at level 1 granularity,
where the working set is 2,048 bytes. That means 65,536 buckets instead of
256, so the bucketing pass is larger.

### 5.5 Cost 3 — clearing and the delta volume

A full 128 MiB clear for each tick costs 3.36 ms of wall time and is pure
waste. Report 06's scanline-delta stamping does solve it, because only
units that moved change their contribution.[^3] Quantify the residue.

A disc at radius 8 spans 17 tiles, so it touches about 4 level 1 cells.
Scenario C moves 2,000 observers for each faction at 10 percent churn, so
each faction dirties about 8,000 level 1 cells. Across 8 factions with
partial overlap the distinct dirty count is about 40,000 cells.

Each dirty cell is read and written once at 2,048 bytes.

| Design | Traffic for each tick | Wall time |
|---|---|---|
| Option C, 40,000 dirty L1 cells | 164 MB | 4.10 ms |
| Option A, 8 factions, 40 blocks each | 5.24 MB | 0.13 ms |

**Option C moves 31 times more memory to carry the same information.**

The root cause is structural and worth stating directly. Option C moves 64
bits of traffic to update 1 bit of information. Option A moves 1 bit in a
`Bits` leaf or 16 bits in an `Array` leaf. The ratio is set by the design,
not by the scenario, and it does not improve with tuning.

### 5.6 Cost 4 — the hard 64-faction cap

The cap forecloses three things. A grand-strategy world with many minor
powers cannot give each one a bit. A scenario editor cannot create a 65th
faction at runtime. A long campaign cannot fragment an empire past the
cap.

Four options exist.

| Option | Grids | Factions | Traffic against `u64` |
|---|---|---|---|
| Full 64 | 256 MiB | 64 | 1.0x |
| 63 plus one reserved bit | 256 MiB | 63 addressable, more derived | 1.0x |
| `u128` | 512 MiB | 128 | 2.0x |
| Hybrid: bits for the hot set | 256 MiB | 63 addressable, rest derived | 1.0x |

Section 6.4 recommends the reserved bit and explains what it buys.

### 5.7 Verdict on option C

**Adopt the faction mask. Reject the level 0 grid.**

The mask domain is right. Advantages 2, 3, 4 and 5 are real, and all four
survive at level 1 granularity, where the structure costs 512 KiB for each
layer instead of 128 MiB. Advantage 1 is real and is better served by a
per-unit projection.

The level 0 grid costs a fixed 256 MiB in every game, moves 31 times more
memory on the update, makes per-faction cardinality a full-map pass, and
makes the dual query 18 to 170 times slower.

---

## 6. The faction mask as a project-wide primitive

### 6.1 The domain is shared

The project owner's framing is that a faction gets one maskable
identifier, and that identifier serves several subsystems at once.

| Use | Form | Size |
|---|---|---|
| Tile visibility and explored state | `u64` for each tile or L1 cell | 512 KiB at L1 |
| Unit and structure ownership filter | `1 << unit.faction & hostile_mask` | none |
| Diplomacy relations | one `u64` row for each faction | 512 B for each relation |
| L1 and L2 summary faction masks | `u64` for each cell | 512 KiB and 2 KiB |
| Contested-block detection | `AND` on L1 faction masks | none |

The full relation plane is small. Sixty-four rows of 8 bytes is 512 bytes
for one relation. Three relations, for allies, war, and shared vision,
total 1,536 bytes. That stays resident in the level 1 data cache for the
whole tick.

This report does not design diplomacy or ownership. Those belong to a
separate revision. This report states only the three consequences below.

### 6.2 The cap is architectural, and belongs on the day-one list

Once the mask domain is shared, the width is no longer a fog decision.
Widening it later touches six places.

1. The value-type table gains a `FactionMask` type, and every use of it
   changes width.
2. Every selector predicate that tests a mask changes.
3. The level 1 and level 2 summary schema changes. The record caps a
   summary at 256 bytes for each cell, so a mask field that grows from 8
   to 16 bytes consumes 3 percent of that cap for each mask field.[^2]
4. The diplomacy plane grows from 512 bytes to 2,048 bytes for each
   relation. This is still cache-resident and is not a problem.
5. Every event type that carries a mask changes its `repr(C)` layout. All
   event types must be `bytemuck::Pod` with declared padding.[^1]
6. Every golden state-hash file regenerates, and every recorded replay
   becomes unreadable.

**Item 6 is the reason this is unretrofittable.** The engine's two
determinism tests compare an event log byte for byte and hash the world
state against a golden file.[^1] A width change invalidates both. Any
replay recorded before the change cannot be replayed after it.

**Recommend adding one entry to the decision record's day-one list:** the
faction mask width and the reserved-bit policy. Rank it near the crate
split, because both are shapes rather than features and both are
impossible to add later.

**The widening cost, stated as a number for the owner.** Moving from
`u64` to `u128` doubles the level 1 mask arrays from 1 MiB to 2 MiB, which
is negligible. It doubles the traffic of every mask operation on the hot
path. It invalidates every golden file and every replay. If the engine has
shipped, the cost is a format migration. If it has not, the cost is one
afternoon. **Decide the width before the first golden file exists.**

### 6.3 The vision-sharing relation plane retires option B

With a `u64` row named `vision_shared`, the effective visibility of
faction `me` is:

```
effective_visible(t) = tile_mask(t) & (own_bit | vision_shared[me]) != 0
```

**This holds, and it holds under option A as well.** Under option A the
effective visible set is the union of the visible sets of the factions
named in `own_bit | vision_shared[me]`. A union of `k` sorted sparse sets
is a `k`-way merge, and `k` is small. Under the hybrid of section 7 the
level 1 test is exactly the expression above, evaluated on the derived
level 1 mask.

Alliance-level and coalition-level visibility therefore become derived
quantities. No shared node stores them. **Option B is retired for a second
and independent reason: the semantics of sharing are now free, so there is
nothing left to share structurally.** This is a retirement, not a
rejection on cost. The cost argument of section 4 stands, but it is no
longer needed.

One property is worth naming. The relation plane expresses non-transitive
and asymmetric sharing. Faction A may grant vision to B without B granting
it to A, and B may not pass A's vision on to C. A tree cannot express
either. The relation plane is therefore strictly more expressive than the
structure it replaces.

### 6.4 Sixty-three factions plus a reserved bit

Evaluate the three candidates for the reserved bit.

**Neutral or gaia.** Reject. Gaia has no fog and no diplomacy row. It also
needs no bit for ownership, because a mask of zero already means "no
faction".

**Unowned or wilderness sentinel.** Reject. A mask of zero already encodes
this. Spending a bit on a value that the empty mask already carries is
waste.

**An overflow bit for factions outside the domain.** Accept. Set bit 63
when at least one faction outside the addressable set sees or owns the
subject. A side table names which ones.

**Recommend 63 addressable factions plus bit 63 as the overflow bit.**

The overflow bit converts a hard cap into a soft one. Every disjunctive
query keeps working without a side-table lookup. Examples are "is this
tile seen by anybody", "is this block contested", and "does any faction
outside my alliance see me". Those are the queries that drive pruning, and
pruning is where the mask earns its place.

**State the limit precisely.** The overflow bit is sound only for
disjunctive queries. Any query that needs faction identity must fall
through to the side table when bit 63 is set. Two examples that must fall
through are "which faction owns this unit" and "is this specific minor
faction hostile to me". Make that fall-through a single function, and make
the side table a sorted array keyed on faction identifier, so its
iteration order is deterministic.

The cost of reserving the bit is one addressable faction slot. The
alternative is a cap of 64 with no escape hatch, which forecloses minor
factions permanently. One slot is a small price.

---

## 7. The three-way comparison and the query mix

### 7.1 The query mix

The representation follows from the query mix, so enumerate it first.

| Query | Frequency | Best form |
|---|---|---|
| Q1. Can faction F see tile T | very high, scattered | either; both are one lookup |
| Q2. Which factions see tile T | high, only for occupied tiles | per-unit mask |
| Q3. List every tile F sees, viewport | every rendered frame | either; both are kilobytes |
| Q4. List every tile F sees, whole map | rare, analytics and export | option A |
| Q5. Does F see anything in this L1 cell | very high, selector pruning | L1 mask |
| Q6. Is this L1 cell contested | high, AI and selectors | L1 mask |
| Q7. How many tiles does F see | low, scoring and UI | option A, free |
| Q8. Effective visibility under shared vision | high | L1 mask with the relation row |
| Q9. Update after observer movement | every tick | option A |
| Q10. Hash all fog state | every tick, determinism test | option C, marginally |

**Q2 is the query that appears to demand option C, and it does not.** The
question is asked about units, not about tiles. There are 1 million units
and 16.7 million tiles. Materialise the answer as one `u64` for each unit,
which is 8 MB, rather than as one `u64` for each tile, which is 128 MiB.

Producing the per-unit mask from option A is a merge. For each faction,
merge its sorted visible tile list against the sorted unit-tile index, and
`OR` the faction bit into the accumulator for each matching unit. The unit
index is already sorted for the spatial radix sort.[^2] A full build for
64 factions at scenario B costs about 68 million merge steps, or roughly
68 core-ms. An incremental build, restricted to units that moved and tiles
whose visibility changed, costs about a tenth of that, so about 7
core-ms. The full build runs once at load; the incremental build runs each
tick.

**Q5, Q6 and Q8 all want a level 1 faction mask, and none of them wants a
level 0 one.** A selector prunes at a cell, not at a tile. Contested-block
detection is defined on a block.[^6] Shared vision is tested once for each
candidate cell before the descent reaches level 0.

### 7.2 Side by side

| Property | A. Sparse per faction | B. Shared tree | C. L0 transposed grid |
|---|---|---|---|
| Size, 8 factions, mid game | 2.81 MB | 1.75 MB | 268 MB |
| Size, 64 factions, mid game | 22.5 MB | 13.9 MB | 268 MB |
| Size, 64 factions, worst case | 269 MB | 167 MB | 268 MB |
| Size floor, 2 factions | 0.70 MB | 0.70 MB | 268 MB |
| Update traffic, scenario C | 5.24 MB | 5.24 MB plus node repair | 164 MB |
| Q1 point query | 1 leaf lookup | 3 leaf lookups | 1 load |
| Q2 all-faction, per tile | 64 lookups | 192 lookups | 1 load |
| Q4 whole-map list | 12 us | 20 us | 220 us to 2.1 ms |
| Q5 pruning | needs derived L1 mask | needs derived L1 mask | needs derived L1 mask |
| Q7 cardinality | free | free | full-map pass |
| Faction cap | 65,535 | 65,535 | 63 or 64 |
| Alliance change | free | full-set repair | free |
| Machinery | one container type | container plus tree plus repair | one flat grid |

Option B loses to option A on size only when the overlap fraction is high,
and it loses on every other row. It is retired by section 6.3 in any case.

### 7.3 Recommendation — the hybrid

**Store option A. Derive the mask at level 1 and at the unit.**

| Layer | Form | Size | Serves |
|---|---|---|---|
| L0 visible, for each faction | option A container | 0.47 MB typical | Q1, Q3, Q4, Q7, Q9 |
| L0 explored, for each faction | option A container | 0.48 MB typical | Q1, Q3, Q4 |
| L1 visible faction mask | one `u64` for each L1 cell | 512 KiB, shared | Q5, Q6, Q8 |
| L1 explored faction mask | one `u64` for each L1 cell | 512 KiB, shared | Q5, Q8 |
| L2 visible faction mask | one `u64` for each L2 block | 2 KiB, shared | Q5, Q6 |
| Per-unit visibility mask | one `u64` for each unit | 8 MB, shared | Q2 |
| Diplomacy relation plane | three `u64` rows for each faction | 1,536 B | Q8 |

The three shared structures total **9.03 MB, independent of the faction
count**. Option C's level 0 grids total 268 MB. The hybrid delivers every
one of option C's five stated advantages for 3.4 percent of the size.

Build the level 1 mask in step 4 of section 3.4, at no extra traffic. When
a faction rebuilds a level 2 block, it already holds the new leaf in a
scratch bitmap. Walk the 256 level 1 cells inside that block, popcount
each 32-byte span in one NEON pass, and set or clear the faction bit
accordingly. This adds 256 popcounts for each dirty block.

The level 1 mask is a monoid fold over `OR`, so it satisfies the exact
aggregation invariant with no special case.[^1] The level 2 mask is the
same fold applied again.

### 7.4 When to revisit and adopt option C

Adopt the level 0 grid if two conditions both become true.

1. A measured profile shows that query Q2 is needed for tiles rather than
   for units, at a rate above about 10 million lookups for each tick.
2. The faction count is fixed at or near 63, so the fixed 256 MiB is not
   paid for factions that do not exist.

Neither condition holds on the current design. Record the trigger so the
question is not reopened without a measurement.

---

## 8. Tiering by faction kind

Define three tiers. A faction belongs to exactly one tier. The control
plane sets the tier at creation and may change it later.

### 8.1 Tier R — rendered

A human player or a recorded observer watches this faction.

- Visible: the level 0 container of section 3.
- Explored: the level 0 container of section 3.
- Mask: an addressable bit. A level 1 mask bit and a level 2 mask bit.
- Update: the block rebuild of section 3.4, every tick.

Typical cost 0.95 MB. Worst case 4.21 MB.

### 8.2 Tier A — active

An artificial-intelligence controller drives this faction. No client draws
its fog, but combat and targeting need exact line of sight.

- Visible: the level 0 container, with a hard cap of 1 MB. On overflow,
  downgrade the densest blocks to `Full`. This over-reports visibility
  inside a cell that the faction already saturates. The error is bounded
  and the direction is safe for a controller.
- Explored: a level 1 bitset, 8,192 bytes fixed. A level 1 cell is 256
  tiles, which is the right granularity for choosing an exploration target.
- Mask: an addressable bit.
- Update: the block rebuild, every third tick. Report 06 already allows a
  lower field-of-view rate.[^3]

Typical cost 0.48 MB. Worst case 1.01 MB.

### 8.3 Tier P — passive

A minor faction, a neutral power, or a frozen faction.

- Visible: not stored. Derive it on demand from the observer disc cache.
- Explored: a level 1 bitset, 8,192 bytes fixed.
- Mask: the overflow bit, bit 63, plus an entry in the side table. A tier
  P faction consumes no addressable slot.
- Update: on demand only.

Cost 8,192 bytes, fixed.

### 8.4 Tier table

| Tier | Visible | Explored | Mask bit | Typical | Worst |
|---|---|---|---|---|---|
| R, rendered | L0 container | L0 container | addressable | 0.95 MB | 4.21 MB |
| A, active | L0 container, 1 MB cap | L1 bitset | addressable | 0.48 MB | 1.01 MB |
| P, passive | derived | L1 bitset | overflow | 8 KB | 8 KB |

### 8.5 Tier promotion

Promotion from tier P to tier A or R must build a level 0 visible
container. Step 4 of section 3.4 does this for one block. Run it for every
block that the faction's observers touch. At 600 observers this is 130,200
stamps, or 0.13 core-ms. Promotion is a single-tick operation.

Promotion also needs a free addressable bit. If none is free, the
promotion fails. Make that a checked error at the control-plane boundary,
not a panic.

Promotion from tier P to tier R must also build a level 0 explored
container. The level 1 explored bitset holds no level 0 detail, so the
detail is lost. Fill each explored level 1 cell fully. The result
over-reports explored by at most 255 tiles for each frontier cell. State
this limit in the interface. A faction that may become rendered must start
in tier R.

---

## 9. The binding constraints

### 9.1 Determinism

Four rules make the update deterministic.

- Process blocks in ascending block number. The block number is the sort
  key.
- Sort the disc list for each block by observer tile index, then by
  quantised radius.
- Give each worker a disjoint set of blocks. The outputs never overlap, so
  the update needs no atomic operation. This satisfies the disjoint-output
  rule that the weak ARM memory model requires.[^7]
- Never iterate a hash container. The dirty set is a bit word set and the
  overflow side table is a sorted array, so both iterate in a fixed order.

The engine keys every random draw on the tuple of system, frame, entity
and draw.[^1] Fog draws no random number, so this invariant is not at
risk.

### 9.2 No floating point

The containers store `u16` offsets, `u64` bit words, and `u32`
populations. The masks are `u64`. The disc cache stores runs as pairs of
`u32` tile indices. The sight radius is a quantised integer. No fog value
is a float.

### 9.3 The frame loop split

Phases 1 to 4 hold a shared reference to the world. Phases 5 to 8 hold a
mutable reference.[^3] Fog writes in phase 8 only.

A selector that filters on visibility runs in phases 1 to 4. It reads the
containers and the level 1 mask that phase 8 produced on the previous
tick. This is a one-tick lag, and the record already accepts a one-tick
lag for derived data.[^2]

Give the fog module two types. `FogRead` exposes the point query, the
batch query, the level 1 mask, and the export. `FogWrite` exposes the
rebuild. Phase 8 is the only phase that receives `FogWrite`.

The per-unit visibility mask is a derived projection. Build it at the end
of phase 8, after the containers settle.

### 9.4 Graviton

- **Batch popcounts.** Popcount a whole 8,192-byte leaf in one NEON pass.
  Never popcount one word in a loop.[^7]
- **64-byte lines.** Align every `Bits` leaf to 64 bytes. Make the line
  size a compile-time constant, because Apple Silicon uses 128 bytes and
  development happens there.[^7]
- **NEON baseline.** NEON is mandatory in the base aarch64 instruction
  set, so the masked `OR` in the stamp loop needs no runtime feature check.
- **Weak memory model.** Give each worker a disjoint set of blocks. The
  update then needs no atomic operation at all.
- **Reference counts.** An `Arc` clone emits a real barrier on ARM. Clone
  once for each block, never once for each tile.
- **Level 1 mask locality.** The level 1 mask array is 512 KiB. It fits in
  a Neoverse level 2 cache, so a selector descent that scans it does not
  reach main memory.

### 9.5 Delivery to Python

The Python control plane must never loop over tiles.[^1] The interface
therefore hands over arrays only.

| Call | Returns | Kind |
|---|---|---|
| `visible_indices(faction)` | `uint32[n]`, sorted ascending | Copy into a scratch buffer |
| `visible_indices_in(faction, block)` | `uint32[m]`, sorted ascending | Copy into a scratch buffer |
| `explored_bits(faction)` | `uint8[2097152]` | Copy into a scratch buffer |
| `explored_l1_bits(faction)` | `uint8[8192]` | Zero-copy view, tiers A and P |
| `l1_visible_masks()` | `uint64[65536]` | **Zero-copy view** |
| `l1_explored_masks()` | `uint64[65536]` | **Zero-copy view** |
| `unit_visibility_masks()` | `uint64[unit_count]` | **Zero-copy view** |
| `relation_plane(name)` | `uint64[64]` | **Zero-copy view** |

Four notes follow.

`visible_indices` copies. Decision D35 requires the documentation to say
"copies" wherever the engine gathers.[^2] The internal form is 256 leaves,
so a flat array does not exist in memory. At scenario C the expansion
writes 1.22 MB, which is about 0.1 ms.

`visible_indices_in` is the call a renderer should use. A viewport covers
one or two level 2 blocks, so the call copies a few kilobytes.

**The four mask calls are genuinely zero-copy, and they are the strongest
part of this interface.** Each is one flat, contiguous array that already
exists in the engine. The control plane can compute alliance visibility,
contested cells, and detection entirely in NumPy with bitwise operators,
with no loop and no per-tile call. The hybrid therefore recovers a
zero-copy fog demonstration that a per-faction sparse form alone would not
provide.

The engine loses one promise. D48 offered the level 0 visible bitset as a
zero-copy view.[^2] That does not survive, because the visible layer is no
longer a flat bitset. The tile grid remains the flagship zero-copy
demonstration, and D35 already rests on the tile grid rather than on fog.

---

## 10. The tension with the selector result type

Research report 04 rejected a general Roaring library for selector results
and recommended a purpose-built two-level chunk mask. The stated reason is
that the Roaring split at 2^16 cuts an arbitrary key space, while the
storage layout already defines a natural split.[^6]

**Recommendation: use one structure for both, not two.**

Section 3.2 shows that the objection does not apply. The map holds 2^24
tiles and a level 2 cell holds 2^16 tiles, so a split at 2^16 is the level
2 cell boundary. The purpose-built tile mask and the fog container are the
same shape, split at the same place.

Report 04's tile mask uses three levels and hash maps for the sparse
levels.[^6] Two changes unify the two structures.

1. Replace the hash maps with the fixed array of 256 leaves. A hash map
   iterates in an unspecified order, which the determinism invariant
   forbids.[^1] A fixed array iterates in block order for free.
2. Add the `Array` leaf form. Report 04's mask has `Full` and `Bits` only.
   A selector result at 1 percent density gains from `Array` for the same
   reason that fog does.

The unified type is:

```rust
enum Leaf { Empty, Array(Arc<[u16]>), Bits(Arc<[u64; 1024]>), Full }
struct TileSet { blocks: [Leaf; 256] }
```

Selector results use it with reference counts that never exceed 1. Fog
uses it with sharing. Set algebra, iteration, export, and the batch
popcount are written once.

**Two structures in one codebase are not acceptable here.** Both hold a
set of tile indices over the same key space with the same split. Two
implementations mean two set-algebra kernels, two export paths, and two
places to break determinism.

The unified type does not replace the unit chunk mask. A unit set splits
on the archetype chunk, which is a different key space.[^6] Keep that as a
separate type.

Report 04 asks for a benchmark before its claim is committed.[^6] That
request stands. Add three cases: a point-query batch of 10,000 lookups
against a 2 percent dense set, a block rebuild of 6,000 discs, and a
level 1 mask scan with an alliance `AND`.

---

## 11. Proposed replacement for D48

The text below replaces decision D48 in the draft record. Do not apply it
to the record. The record is a draft under review.

---

> #### D48. Fog is a block-adaptive tile set, with a derived faction mask
>
> Store fog for each faction as two layers, explored and visible. Store
> each layer as 256 level 2 blocks. Each block holds one of four leaf
> forms.
>
> | Leaf | Payload | Bytes | Used when |
> |---|---|---|---|
> | `Empty` | none | 0 | population 0 |
> | `Array` | sorted `u16` | `2n` | population 1 to 4,096 |
> | `Bits` | `[u64; 1024]` | 8,192 | population above 4,096 |
> | `Full` | none | 0 | population 65,536 |
>
> A level 2 cell holds 65,536 tiles and the map holds 256 of them. The
> split at 2^16 is therefore the level 2 cell boundary, not an arbitrary
> cut. This is why fog uses the same structure as a selector tile result.
> D34 is amended to add the `Array` leaf and to replace its hash maps with
> the fixed array of 256 blocks, because a hash map has no defined
> iteration order.
>
> **Delete the `u8` visibility counter.** The counter existed only to
> avoid clearing and re-scattering a dense bitset. A block-granular
> rebuild removes that need. The counter also costs 16.8 MB for each
> faction and about 868,000 cache-missing writes for each large faction
> each tick, which is about 87 core-ms. That breaks its own budget line.
>
> **Rebuild dirty blocks, not the map.** Mark a block dirty when an
> observer whose disc touches it gains or loses sight. For each dirty
> block, clear an 8,192-byte scratch bitmap, apply every current disc that
> touches the block with masked `OR` writes, popcount the result in one
> NEON pass, choose the leaf form, and store it. Then `OR` the new visible
> leaf into the explored leaf. Explored is monotonic, so it never needs a
> removal.
>
> **Adopt a 64-bit faction mask, and derive it at level 1, not level 0.**
> A transposed level 0 grid of one `u64` for each tile is 128.0 MiB for
> each layer, so 256 MiB fixed. That is exactly the same bit count as 64
> dense bitplanes; the transpose is the same bits in a different order and
> it is not a memory saving. It also moves 64 bits of traffic to update 1
> bit of information, which costs 164 MB of traffic for each tick at a
> late-empire scale against 5.24 MB for the block rebuild.
>
> Derive instead:
>
> | Derived structure | Size | Purpose |
> |---|---|---|
> | L1 visible faction mask | 512 KiB total | selector pruning, contested cells |
> | L1 explored faction mask | 512 KiB total | selector pruning |
> | L2 visible faction mask | 2 KiB total | coarse pruning |
> | Per-unit visibility mask | 8 MB at 1M units | "which factions see this unit" |
> | Diplomacy relation plane | 1,536 B | allies, war, shared vision |
>
> These total 9.03 MB and do not grow with the faction count. They deliver
> every advantage of the level 0 grid for 3.4 percent of its size. Build
> the level 1 mask inside the block rebuild: after the scratch bitmap
> settles, popcount its 256 level 1 spans and set or clear the faction bit.
> Bitwise `OR` is an exact, integer, associative monoid, so this fold
> satisfies the aggregation invariant with no special case.
>
> **Reject a tree of shared masks.** With a `u64` `vision_shared` row, the
> effective visibility of a faction is
> `mask & (own_bit | vision_shared[me]) != 0`. Shared vision is therefore a
> derived quantity and no shared node needs to store it. The relation plane
> also expresses asymmetric and non-transitive sharing, which a tree
> cannot. The cost argument is secondary: the saving fraction of a shared
> node over `k` members with overlap `phi` is `phi(k-1)/k`, which reaches
> 87.5 percent of its limit at only 8 members, and the shared node needs a
> group-level reference count to stay incremental.
>
> **Reserve bit 63 as the overflow bit. Use 63 addressable factions.** Set
> bit 63 when a faction outside the addressable set sees or owns the
> subject, and name it in a sorted side table. This keeps every disjunctive
> query correct without a lookup. Any query that needs faction identity
> must fall through to the side table when bit 63 is set. Do not reserve a
> bit for neutral or for unowned; a mask of zero already encodes both.
>
> **The faction mask width is architectural, not a fog decision.** The same
> domain serves visibility, ownership filters, diplomacy rows, summary
> masks, and contested-block detection. Widening it later changes the value
> type table, every mask predicate, the summary schema, and the `repr(C)`
> layout of every event that carries a mask. That invalidates every golden
> state hash and every recorded replay. Add the mask width and the
> reserved-bit policy to the day-one list.
>
> **Determinism.** Process blocks in ascending block number. Sort the disc
> list for each block by observer tile index, then by quantised radius.
> Give each worker a disjoint set of blocks, so the update needs no atomic
> operation. Never iterate a hash container; the overflow side table is a
> sorted array.
>
> **Sharing.** Hold each non-empty leaf behind an `Arc` and copy on write.
> This makes a fog snapshot and a derived faction cost 256 pointer writes.
> It does not dedup independent factions, and it is not expected to. Clone
> an `Arc` once for each block, never once for each tile.
>
> **Tier the factions.**
>
> | Tier | Visible | Explored | Mask bit | Update rate |
> |---|---|---|---|---|
> | R, rendered | L0 container | L0 container | addressable | every tick |
> | A, active | L0 container, 1 MB cap | L1 bitset, 8 KB | addressable | every third tick |
> | P, passive | derived on demand | L1 bitset, 8 KB | overflow | on demand |
>
> A tier A faction at its cap downgrades its densest blocks to `Full`. A
> tier P faction stores no visible set and consumes no addressable bit. A
> faction that may become rendered must start in tier R, because the level
> 1 explored bitset cannot be refined back to level 0.
>
> **Cost.**
>
> | Case | Visible | Explored | Total |
> |---|---|---|---|
> | Early game, 1,464 visible tiles | 2.9 KB | 3.0 KB | 14 KB |
> | Mid game, 71,610 visible tiles | 143 KB | 200 KB | 351 KB |
> | Late empire, 303,800 visible tiles | 470 KB | 480 KB | 958 KB |
> | Wide sensors, 1,470,600 visible tiles | 1.28 MB | 1.20 MB | 2.49 MB |
> | Whole map visible and explored | 2.10 MB | 1 KB | 2.11 MB |
> | Tier A, typical | 470 KB | 8 KB | 0.48 MB |
> | Tier P | 0 | 8 KB | 8 KB |
> | Derived structures, all factions | | | 9.03 MB |
>
> Totals include 8 KB of block headers. The worst case for a tier R faction
> is 4.21 MB, which is two dense bitsets plus headers. The previous cost
> was 21.0 MB for every faction in every case.
>
> **Budget.** Replace the "fog counter update" line in the per-tick budget
> with two lines: "fog block rebuild" at 1 to 4 core-ms and 0.1 to 0.4
> wall-ms, and "per-unit visibility mask" at 5 to 10 core-ms and 0.4 to 0.9
> wall-ms. The second line is new work that the old design did not do.
>
> **Delivery to Python.** Return `visible_indices(faction)` and
> `visible_indices_in(faction, block)` as sorted `uint32` arrays, and
> `explored_bits(faction)` as a `uint8` array. These copy into a reusable
> Rust-owned scratch buffer, and the documentation must say "copies", as
> D35 requires. Return `l1_visible_masks()`, `l1_explored_masks()`,
> `unit_visibility_masks()` and `relation_plane(name)` as zero-copy
> `uint64` views. Those four views let the control plane compute alliance
> visibility and detection with NumPy bitwise operators and no loop. They
> replace the zero-copy demonstration that the old flat visible bitset
> provided.
>
> **Field of view is unchanged.** Recursive shadowcasting over 6 sextants
> produces runs of tile indices. Each run stays inside one 16-wide block
> row, so each run is at most 16 tiles long. The rebuild writes a masked
> word for each run instead of a bit for each tile. The disc cache, keyed
> on observer tile, quantised radius and terrain version, is unchanged.
>
> **Revisit trigger.** Adopt the level 0 transposed grid only if a measured
> profile shows more than about 10 million per-tile all-faction lookups for
> each tick, and the faction count is fixed near 63. Neither holds today.

---

## 12. The faction ceiling, and OQ5

**OQ5 is closed. Fog memory no longer sets the ceiling. The faction mask
width does.**

### 12.1 The two-class model, chosen against two alternatives

A `u64` mask caps at 64 factions. Section 8 tiers factions to a practical
total of 1,024. Those two numbers conflict unless the design separates
"has a bit" from "exists". Three resolutions are available.

**A hard cap of 64.** Reject. It forecloses minor powers, runtime faction
creation, and empire fragmentation. It also gives the engine no graceful
behaviour at the boundary: the 65th faction cannot exist at all.

**A wider mask, such as `u128`.** Reject for now. It doubles the traffic
of every mask operation on the hot path to raise a limit that the
two-class model already removes. Section 6.2 also shows the change is
unretrofittable once a golden state hash exists.

**A two-class model. Accept this one.** Sixty-three factions are
mask-addressable. They hold a bit, a level 0 fog container, a diplomacy
row, and an ownership bit. Any further faction exists as tier P: it has a
`FactionId`, units, and a level 1 explored bitset, but no bit. Bit 63
reports that some non-addressable faction sees or owns the subject, and a
sorted side table names which.

**Justification against the owner's stated intent.** The owner holds that
64 is enough for essentially any simulation. The two-class model agrees
with that claim and does not depend on it. Sixty-three addressable
factions cover every faction that a player sees, fights, or negotiates
with. The extra class costs 8 KB for each faction and exists so that a
scenario with many dormant minor powers does not force a mask widening
later. **The model makes the owner's judgement cheap to hold and cheap to
be wrong about.** That is the property to buy, because the mask width is
the one part of this design that cannot be changed after release.

The limit of the model is stated precisely in section 6.4. Bit 63 is sound
for disjunctive queries only. Any query that needs faction identity must
fall through to the side table when bit 63 is set.

### 12.2 The recommended numbers

The recommended ceiling is **63 mask-addressable factions**, plus an
unbounded count of tier P factions that share the overflow bit. Recommend
a practical total of **1,024 simulated factions**.

Recommend the addressable split as **8 tier R and 55 tier A**. That split
is the number the owner should argue about, because widening the mask
later is unretrofittable.

| Tier | Count | Typical each | Worst each | Typical | Worst |
|---|---|---|---|---|---|
| R | 8 | 0.95 MB | 4.21 MB | 7.6 MB | 33.7 MB |
| A | 55 | 0.48 MB | 1.01 MB | 26.4 MB | 55.6 MB |
| P | 961 | 8 KB | 8 KB | 7.7 MB | 7.7 MB |
| Derived | shared | | | 9.0 MB | 9.0 MB |
| **Total** | **1,024** | | | **50.7 MB** | **106 MB** |

The old design costs 21.0 MB for each faction, so 1,024 factions cost 21.5
GB. The new worst case is 106 MB, which is a reduction of 203 times. The
transposed level 0 grid would cost 268 MB regardless of the faction count,
so it is 2.5 times the new worst case and 5.3 times the new typical case.

**What sets the ceiling now.** Four limits bind before fog memory does.

1. **The faction mask width.** Sixty-three addressable factions. This is
   the binding limit and it is architectural, not local to fog.
2. **The tier A stamp cost.** Fifty-five tier A factions at a mid-game
   scale cost about 2.8 core-ms every third tick, so about 0.9 core-ms
   for each tick.
3. **The per-faction influence maps.** The record budgets 8 influence maps
   of 65,536 cells for each faction, at about 2 MB.[^2] At 1,024 factions
   that is 2 GB, which exceeds all fog by 19 times. **Influence maps are
   now the binding per-faction cost and need their own tiering.** This
   report does not solve that.
4. **The `FactionId` type.** `FactionId` is a `u16`, so the absolute limit
   is 65,535.[^7] Tier P alone reaches 65,535 factions at 8 KB each, which
   is 522 MB. That is affordable, so raise the total above 1,024 only after
   limit 3 is solved.

---

## 13. Open questions from this report

1. **Confirm 63 addressable factions and the reserved overflow bit.** This
   is the decision the owner must make, and section 6.2 shows it is
   unretrofittable. Confirm the tier split of 8 rendered and 55 active at
   the same time.
2. **How do influence maps tier?** Section 12 shows they cost 2 GB at
   1,024 factions, which is 19 times all fog. This is now the binding
   per-faction cost and needs its own report.
3. **Does a tier A faction need exact level 0 visibility?** This report
   assumes yes, because combat targeting reads it. If a controller accepts
   level 1 visibility, tier A drops from 0.48 MB to 16 KB.
4. **Is the 4,096 leaf threshold correct on the target?** The threshold is
   where a `u16` array and an 8,192-byte bitmap cost the same bytes. It is
   not where they cost the same time. Measure and lower it if query time
   dominates.
5. **What is the real per-tick rate of the all-faction per-tile query?**
   Section 7.4 sets the revisit trigger at 10 million lookups for each
   tick. The trigger cannot be evaluated before a controller exists.
6. **Does the unified tile set hurt selector performance?** Section 10
   recommends one type for fog and for selector tile results. Extend report
   04's benchmark rather than writing a second one.

---

## References

[^1]: Cachette project instructions, sections "Hard invariants" and "Design principles". `CLAUDE.md`
[^2]: ADR-0001, Foundational Architecture, decisions D34, D35, D48, the memory budget table, and open question OQ5. `docs/adrs/REGISTRY.md`
[^3]: Research report 06, Algorithms and Scheduling, sections 4.3, 4.4 and 6.1. `docs/research/reports/06-algorithms-and-scheduling.md`
[^4]: Research report 06, Algorithms and Scheduling, section 10, per-tick cost budget. `docs/research/reports/06-algorithms-and-scheduling.md`
[^5]: Arm Neoverse N1 Software Optimization Guide, memory system latency tables. https://developer.arm.com/documentation/swog309707/latest
[^6]: Research report 04, Selector Engine and Verbs, section 3, result representation. `docs/research/reports/04-selector-engine-and-verbs.md`
[^7]: Research report 07, Target Platform and Value Types. `docs/research/reports/07-target-platform-and-value-types.md`
