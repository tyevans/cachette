# ADR-0001 — Foundational Architecture for Cachette

- **Status:** Draft
- **Date:** 2026-08-30
- **Supersedes:** nothing
- **Background:** `docs/adrs/background/adr-0001/00-context-brief.md` through
  `07-target-platform-and-value-types.md`

---

## Context

Cachette is a simulation engine. Rust holds the data and does the work.
Python drives it. The target is a headless, multi-tenant simulation server.

The world has three levels of detail. L0 is a hex grid of about 16.7 million
tiles. L1 and L2 are summaries of L0. Units number in the hundreds of
thousands to the millions.

Three audiences use it, in this order of priority:

1. The author, who builds a strategy game on it. This is the dogfooding path.
2. Other developers who build simulations.
3. Researchers who run reinforcement learning and agent-based models. This
   audience is the clearest differentiator, because it wants a NumPy-native,
   deterministic, `step()`-shaped API.

Eight background documents inform this record. `00-context-brief.md` states
the position at the start of the work. Six research reports test that
position. `07-target-platform-and-value-types.md` is a later addendum from
the session lead. It sets the target platform. The six reports did not see
it. Where report 07 conflicts with an earlier report on a platform fact,
report 07 wins, and this record says so at each place.

Several claims in the background documents were checked against current
documentation on 2026-08-30. Section "Verification of external claims"
records the results. Two claims in report 07 were refuted by that check.

---

## Decision

### Part A — Platform and value types

#### D1. Target `aarch64-unknown-linux-gnu` as the primary platform

The deployment target is AWS Graviton. Development happens on x86-64 and on
Apple Silicon. Those are development targets only.

Set this in `.cargo/config.toml` on day one:

```toml
[target.aarch64-unknown-linux-gnu]
rustflags = ["-C", "target-cpu=neoverse-n1"]   # see the table below
```

| Graviton | Core | ISA | `-C target-cpu` |
|---|---|---|---|
| 2 | Neoverse-N1 | Armv8.2-A | `neoverse-n1` |
| 3 / 3E | Neoverse-V1 | Armv8.4-A | `neoverse-v1` |
| 4 | Neoverse-V2 | Armv9.0-A | `neoverse-v2` |
| 5 | Neoverse-V3 | Armv9.2-A | `neoverse-v3` |

Pick the lowest generation the deployment must support. A binary built for
`neoverse-v2` will not run on Graviton 2.

Source: `07-target-platform-and-value-types.md`. The core mapping is
verified. See "Verification of external claims".

#### D2. Write one code path. Do not build runtime SIMD dispatch

NEON is mandatory in the AArch64 base ISA. It is always present. So the
`wide` plus `multiversion` machinery that reports 01 and 06 recommend for
x86-64 has no purpose on the target.

Write plain struct-of-arrays loops. Let LLVM vectorise them. Use the `wide`
crate only where a measured kernel needs explicit lanes. Do not use
`std::simd`; it is still nightly-only, and a nightly toolchain would break
wheel packaging.

This supersedes report 01 section 6 and report 06 section 9.3 item 2 on the
target. Keep the x86 dispatch advice only if a supported x86 build appears
later.

Report 01 recommendation 7 and report 06 recommendation 10 both survive in
their important form: pin the hot kernels with `cargo-show-asm` in CI.
Autovectorisation fails silently when someone refactors a loop.

#### D3. Cache-line size is a compile-time constant

Neoverse N1, V1 and V2 use 64-byte cache lines. Apple M-series uses
128-byte lines. Development machines will therefore mislead on false
sharing and on alignment.

```rust
#[cfg(target_arch = "aarch64")] pub const CACHE_LINE: usize = 128; // safe upper bound
```

Set the constant per target. Benchmark on Graviton, never on a laptop.
Report every false-sharing measurement with the machine it came from.

#### D4. Ban floats from simulated and aggregated state. Use one fixed-point scale

All positions, stats, modifiers and pyramid accumulators are fixed-point
integers. `Fix32` is `i32` in Q16.16. `Fix64` is `i64` in Q32.32. `Accum` is
always `i64`.

Four reports reached this independently: report 01 (parallel float
reductions are not reproducible), report 02 (a float fold is not
associative), report 03 (a monoid needs exact associativity), report 04
(modifier stacking is not associative in float).

**Conflict resolved: one scale, not two.** Report 04 proposes 1/1024 for
modifiers. Report 07 proposes Q16.16 for positions. Report 04's only reason
for the coarser scale is to keep the modifier multiply inside `i32`. Report
07 establishes that 64-bit integer arithmetic runs at full rate on the
target, so a widening `i64` intermediate is free. The reason for the second
scale is gone. **Q16.16 everywhere.** Report 04's modifier pipeline stays
unchanged in shape; only its constant changes:

```
stage 0  base       = stat_table[unit_type][field]
stage 1  flat       = base + sum(flat modifiers)                 // i64, order-free
stage 2  percent    = (flat * (65536 + sum(pct modifiers))) >> 16
stage 3  multiplier = fold(mult modifiers, in declared category order)
stage 4  clamp      = clamp(result, field_min, field_max)
```

The 1/65536 step is finer than any designer needs. That is harmless.

#### D5. The value-type vocabulary

Newtype every index. These are zero-cost and they stop `TileIdx` and
`ChunkIdx` from being confused.

| Type | Repr | Notes |
|---|---|---|
| `TileIdx` | `u32` | Block-tiled odd-r offset index. 2^24 tiles. |
| `ChunkIdx` | `u32` | Derived from `TileIdx` by shift. |
| `Entity` | `NonZeroU64` | `NonMaxU32` index plus `u32` generation. `Option<Entity>` stays 8 bytes. |
| `UnitTypeId` | `u16` | Index into the stat table. |
| `TerrainId` | `u8` | Keep under 32 variants for the terrain-cost matrix. |
| `FactionId` | `u16` | |
| `UpgradeSetId` | `u32` | Interned. Unbounded. Authoritative. |
| `CapabilityMask` | `u64` | Derived from `UpgradeSetId`. Hot loops and predicates only. |
| `Fix32` | `i32`, Q16.16 | Positions and per-entity scalars. Range +/-32768. |
| `Fix64` | `i64`, Q32.32 | Where range demands it. |
| `Accum` | `i64` | Pyramid accumulators. Always. |
| `Tick` | `u64` | |

Two invariants apply to every type that enters an event or a summary:

1. It must be `bytemuck::Pod`. That means `repr(C)`, explicit padding, and
   `u8` in place of `bool`. Report 05 section 10 shows that struct padding
   makes per-frame state hashing non-reproducible. This rule prevents that.
2. Size and alignment should be 8 or 16 bytes where practical. ARM loads and
   stores register pairs with `LDP` and `STP`.

**The accumulator-width rule.** A pyramid accumulator must be wider than the
tile field it summarises, and the widening must happen at L1.

Report 07 states that a `u8` field over 2^24 tiles "reaches 2^32 exactly".
That is not exact. The true maximum is 255 * 16777216 = 4,278,190,080, which
fits in `u32` with 0.4 percent to spare. **That is not headroom. It is a
trap**, and one extra tile field or one wider tile type breaks it. The rule
stands and is strengthened: **`Accum` is `i64` with no exceptions.** It is
free on a 64-bit target.

---

### Part B — Storage

#### D6. Two storage regimes, and the tile grid is not in the entity store

Tiles are dense struct-of-arrays, indexed by grid position. Units and
structures live in a separate generational arena. A bridge index joins them.
This holds the brief's decision 2 unchanged.

#### D7. Axial coordinates for logic, odd-r offset for the array index

Every public type, selector and geometry routine uses axial `(q, r)` as
`i32`. Convert to cube in-register for distance, rotation and reflection.
The conversion is one subtraction.

The array index is odd-r offset. The conversion is one shift and one add.

A raw axial store of a rectangular world wastes 50 percent of the array,
because a 4096-row parallelogram needs 6144 columns. It also gives blocks an
aspect ratio of 1.73:1 in world space, which weakens query pruning. Offset
blocks are 1.15:1.

Source: `02-hex-grid-and-lod-pyramid.md` section 1.2 and 3.4. This refines
the brief's decision 3.

If the world is a rhombus by design, store raw axial and delete the
conversion. See open question OQ2.

#### D8. Exact power-of-two nesting. Reject H3 aperture 7

Blocks are power-of-two in the offset index space. The parent index is a
right shift. The partition is exact, disjoint and complete.

The original reason for rejecting aperture 7 was wrong. See "What research
refuted", item R3. The conclusion still stands, on different grounds:
aperture 7 costs power-of-two index arithmetic, cache alignment and
contiguous child storage, and buys a parent shape no player will inspect.

Render L1 and L2 cells as a merged outline of their member tiles if a hex
look matters. The aggregate is defined over the index set, not over a
polygon, so the drawn shape is free.

Recorded failure mode: if Cachette ever has to interoperate with a real
geospatial dataset, parallelogram blocks are a dead end and H3 is the
answer. This is a game world with no geographic reference frame, so that
case does not apply.

#### D9. Fanout is 16. Three levels

| Level | Cells | Children per cell |
|---|---|---|
| L0 | 4096 x 4096 = 16,777,216 | — |
| L1 | 256 x 256 = 65,536 | 256 |
| L2 | 16 x 16 = 256 | 256 |

**Conflict resolved: 16x16, not 32x32.** Report 01 wants 32x32. Reports 02
and 06 want 16x16, because 32x32 leaves L2 with 4x4 = 16 cells, and 16 cells
prune nothing. Reports 02 and 06 reached that independently. Report 07
resolves the conflict by separating two granularities that the lead had
treated as one.

**Report 07's resolution holds, but its mitigation is wrong.** The reasoning
in full:

- Report 01's argument for 32x32 is that a 32x32 bitplane block is 16 `u64`
  words, which is exactly two 64-byte cache lines, so parallel bitset writes
  are race-free by construction (report 01 section 10, hazard 1).
- That argument conflates two things. **Correctness needs whole-*word*
  alignment. False sharing needs whole-*line* alignment.** A 16x16 bitplane
  block is 256 bits, which is 4 whole `u64` words. No read-modify-write can
  lose an update. Report 01's correctness claim is satisfied at 16x16.
- What 16x16 loses is only performance: a 32-byte block is half a cache
  line, so two adjacent blocks share a line. Under ARM's weak memory model
  that costs more than it would on x86.
- Report 07 proposes to pad each block to 64 bytes and states the cost as
  "about 2 MB across the map". **That figure is per bitplane.** 65,536
  blocks times 32 wasted bytes is 2 MiB *for one plane*. A 16-plane schema
  wastes 32 MiB against a 160 MiB tile side. That is 20 percent, not 1.5
  percent. Report 07 undercounted by the plane count.
- **Better fix, at zero cost: do not pad. Constrain the parallel split
  instead.** A 64-byte line covers exactly two consecutive 16x16 bitplane
  blocks in block-tiled order. Require every parallel split index to be a
  multiple of two blocks. Then no two workers ever touch the same line. In
  practice the constraint is already met, because a rayon task should cover
  16 blocks or more (report 01 section 3, report 06 section 5.4).

So: **aggregate at 16x16, keep bitplanes unpadded, align each plane's base
allocation to 2 MiB, and make the parallel split granularity an even number
of blocks.** This is a correction to report 07, not an override of it. The
platform fact report 07 introduced — 64-byte Neoverse lines — is confirmed
and is what makes the constraint necessary.

#### D10. Block-tiled storage order

Store every tile field in 16x16 blocks. Inside a block use row-major order.
The block is the L1 aggregation cell.

```
index(col, row) = (((row >> 4) * (W >> 4)) + (col >> 4)) * 256
                + ((row & 15) << 4)
                + (col & 15)
```

Every operation is a shift, a mask or an add. There is no division, because
4096 and 16 are powers of two.

One L1 aggregation step then reads one contiguous 256-byte span of a `u8`
field, or one contiguous 32-byte span of a bitplane. In row-major order the
same work touches 16 separate spans on 16 different pages.

A long horizontal scan across the whole map becomes strided. Accept this.
No such query exists. If one appears, it walks block rows.

Source: `01-ecs-and-memory-layout.md` section 8,
`02-hex-grid-and-lod-pyramid.md` section 2.

#### D11. One bitplane per boolean attribute

Do not pack several booleans into one byte per tile. Separate planes give:

- Popcount queries. "How many tiles in this block are forested and unowned"
  is `(a[i] & b[i]).count_ones()` over 4 `u64` words, not 256 tiles.
- Set algebra as one `u64` operation per 64 tiles.
- An exact monoid: `count_ones` is a sum, and bitwise OR carries an
  identity.
- The dirty pyramid is already a bitplane, so it shares the code path.

**Batch every popcount.** On Graviton, `u64::count_ones()` compiles to a
NEON sequence: move to a vector register, `CNT` per byte, `ADDV` to reduce.
That is 3 to 4 instructions where x86 has one. Counting one word at a time
pays the setup each time. Structure every aggregation kernel to count a
whole block at once and accumulate across it.

Source: `07-target-platform-and-value-types.md`. The claim is verified for
Graviton but needed a scope correction; see "Verification", item V11.

#### D12. Build a generational SoA arena. Do not depend on an external ECS

**Conflict resolved: no third-party ECS, and no archetype machinery either.**

Report 01 recommendation 8 argues for writing about 2000 lines rather than
depending on `bevy_ecs`, `hecs` or `legion`. Its five reasons hold:
per-chunk user metadata for selector pruning, per-chunk rather than
per-entity change ticks, raw column pointers with a stable layout, a
specific deterministic structural-change order, and the fact that one
archetype uses almost none of what those crates do.

Reports 03 and 06 arrive at the same place from determinism. Report 03
section 5.2 says thread completion order is fatal to reproducibility.
Report 06 section 5.2 says to compile a static stage list and to reject
Bevy's dynamic executor, because dispatch order can leak into results.
**Three reports independently conclude that Cachette cannot depend on a
scheduler it does not control.** That is decisive.

Report 01 goes one step further and asks whether an ECS is needed at all.
If every unit lives in one archetype, what remains is parallel columns plus
a generational free list. That is a generational SoA arena, not an ECS.
**Take that position.** Name it honestly and build the arena.

**Report 01's challenge back to the owner is recorded and unanswered: name
three archetypes you expect to exist.** See OQ1. If the owner can name
three, D12 and D13 must be revisited before any code is written. If the
owner cannot, this decision is confirmed.

Counter-argument, stated fairly: writing an entity store is a well-known way
to spend six months not shipping a game. Timebox it. Write the iteration
benchmark first, so there is a signal for when to stop tuning.

#### D13. Columns are globally contiguous. A "chunk" is a logical span

**Conflict resolved, and it resolves two other conflicts at once.**

The brief says 16 KiB chunks. Report 01 recommendation 5 says 64 KiB with
huge pages. Report 05 section 3.3 shows that either choice destroys the
promise of a flat zero-copy NumPy view, because chunked storage gives one
array per component *per chunk*, not one array per component.

D12 removes the reason for separate chunk allocations. Once there is one
archetype, there is no archetype migration, so there is nothing for a chunk
allocation to serve.

Therefore: **allocate one contiguous column per field over the whole arena.
A "chunk" becomes a fixed-size logical span of index positions across every
column.** It is a query granularity and a parallelism granularity. It is not
an allocation.

This keeps everything report 01 wanted from chunks:

- Per-span metadata for selector pruning: bounding box, faction mask,
  unit-type mask, change tick (report 01 recommendation 1).
- Per-span change ticks rather than per-entity ticks. Bevy's per-entity
  scheme costs 80 MB of writes per frame at 1M entities and 10 components.
  Per-span costs about 100 KB (report 01 recommendation 2).
- 64-byte alignment for every column start, so a vector load never straddles
  a line and two workers never share one.

And it adds one thing no report could offer, because report 05 assumed
separate chunk allocations: **a whole-column NumPy view over units is now
genuinely zero-copy.** See D22.

Span length is a compile-time constant. Default it to 4096 entities. That is
a multiple of 64, so a per-entity dirty bitplane splits on whole words. It
puts 4,000 to 25,000 units in a rayon task, which is report 06's guidance
for 50 to 500 microseconds of work per task. Measure it. Do not treat 4096
as more than a starting point.

Allocate every column from a 2 MiB-aligned arena and call
`madvise(MADV_HUGEPAGE)` once at start-up. Never grow the arena inside a
frame; `madvise` at the wrong moment stalls the calling thread. About 166
MiB of tile data needs 42,500 four-kilobyte page entries and only 83
two-megabyte entries, and an L2 TLB holds 1500 to 3000 entries.

#### D14. Entity identity and slot recycling

`Entity` is a `NonZeroU64` holding a `NonMaxU32` index and a `u32`
generation, so `Option<Entity>` is 8 bytes. Assert the size in a test.

Recycle slots **FIFO**, not LIFO, and increment the generation on **free**,
not on allocate. A LIFO list hands the same slot back inside one frame, so a
stale identifier captured by a command buffer before the despawn still
matches. Retire a slot when its generation overflows; that leaks 4 bytes in
a case that takes years to reach.

Python never sees an entity identifier (D19, D21), so the exposure is
internal: the command buffer and the tile-to-unit index, both of which hold
identifiers across the frame barrier. FIFO plus free-time increment covers
both.

Source: `01-ecs-and-memory-layout.md` section 4.

#### D15. The tile-to-unit bridge is block-level, not per-tile

Reject the literal CSR form in the brief's decision 2. A CSR offset array
over 16,777,217 tiles costs 64 MiB at `u32`. That is more than the whole
minimum tile schema, and over 94 percent of those offsets duplicate their
neighbour.

Instead:

1. Keep an array of unit indices sorted by packed tile index. Rebuild it at
   the frame barrier with a stable parallel radix sort on the `u32` key.
2. Store one `(start, len)` pair per 16x16 block.
3. To find the units on one tile, take the block range and search inside it.
   A block holds 256 tiles, so the range is short.
4. Add a "block holds any unit" bitplane. A selector descent then skips
   empty blocks with a popcount.

If per-tile lookup turns out to be hot, build the full CSR lazily per block.
A per-block CSR is a 257-entry `u16` array, which is about 512 bytes and
sits in L1 cache.

Source: `01-ecs-and-memory-layout.md` section 9.

---

### Part C — Aggregation and the pyramid

#### D16. The corrected aggregation rule

The brief's decision 4 is defective in three ways. Report 03 shows it must
demand *exact* associativity, so float sums fail it. Report 02 shows a
monoid builds the pyramid but does not update it incrementally. Report 04
shows modifier stacking is not associative at all and does not belong under
this rule.

**The corrected rule, as one invariant:**

> A quantity may appear in an L1 or L2 summary only if its combine is
> **exactly** associative over its stored representation and has an
> identity, and only if it is either
> **(a)** a group — it has an inverse, so a child delta updates every
> ancestor in O(levels) with no block read — or
> **(b)** declared with an explicit recompute path plus a stored witness
> that bounds how often recompute fires.
>
> Exact associativity means integer or fixed-point. Float addition is not
> associative, so a float sum is banned from every summary.
>
> Modifier stacking is **not** aggregation. It is governed by the fixed
> pipeline of D4, not by this rule.

The declaration is mechanical. A field declares its combine in the field
registry (D18), and the macro refuses to emit a summary slot for a field
with no declared combine.

Case (b) exists so that `min`, `max` and bitwise OR remain usable. Report 02
gives the two conversions that turn each into case (a):

- **Extremum count.** Store the extremum and a count of children that reach
  it. A child moving away from the extremum decrements the count. Only when
  the count reaches zero does a rescan fire. For data with many equal values
  — terrain, elevation on a plain — the count is usually above 1 and the
  rescan almost never fires. For uniformly random continuous data the count
  is usually 1 and it fires often. Know which case each field is in.
- **Popcount per bit.** Store a count per bit instead of a bare mask. The
  counts form a histogram, which is a group. The OR mask is `count > 0` and
  the AND mask is `count == total`, both derived at read. This costs 32
  bytes per cell for 16 factions against 2 bytes for a bare mask. Pay it: it
  converts the most common categorical aggregate from case (b) to case (a),
  and it gives both bounds that pruning needs (D17).

| Statistic | Exactly associative | Class | Delta update |
|---|---|---|---|
| integer or fixed-point sum | yes | group | yes |
| count | yes | group | yes |
| histogram `[u32; K]` | yes | group | yes |
| popcount per bit | yes | group | yes |
| min / max with extremum count | yes | (b) -> (a) | usually |
| bitwise OR / AND as a bare mask | yes | (b) | no, rescan |
| mean | — | store `(sum, count)`, divide at read | yes |
| argmax with a stable tiebreak key | yes | monoid | no |
| dominant value | **no** | store a histogram, argmax at read | — |
| **float sum** | **no** | **banned** | — |
| median, percentile | no | approximate from a bucketed histogram | — |
| distinct count via HLL | yes | monoid, no inverse | no |

The failure mode this rule prevents is specific and hard to debug. The dirty
pyramid recomputes only some cells, so the combination order changes between
frames and between thread counts. A float sum drifts. Over hours L1 silently
disagrees with a full recomputation of L0.

Add the test that detects it anyway: periodically recompute a cell in full
and compare it against the incremental value.

#### D17. Two pyramids, two bounds, and a guaranteed flat fallback

**Two pyramids over the same cell grid.** Tile terrain changes rarely. Unit
positions change every frame. If both feed one pyramid, unit motion dirties
every cell every frame and the terrain aggregates are recomputed for
nothing. Keep a tile pyramid on a slow dirty cadence and a unit pyramid
updated every frame. They share the index arithmetic and the dirty
machinery, so the extra code is small. Make the unit pyramid delta-only —
counts and histograms — so it never needs a recompute pass. The popcount
trick in D16 is what makes that possible.

**Store a lower and an upper bound for every field a selector filters on.**
One bound alone gives `None` pruning but never `All` acceptance, and `All`
acceptance is the larger win.

Selector descent is three-valued:

- `None`: prune the subtree. No further work.
- `All`: accept the subtree whole. Emit the cell's range without descending.
- `Some`: descend.

| Predicate | Fields needed | `None` test | `All` test |
|---|---|---|---|
| `field > k` | min, max | `max <= k` | `min > k` |
| `field in [a,b]` | min, max | `max < a or min > b` | `min >= a and max <= b` |
| `terrain == t` | `hist[t]`, `tile_count` | `hist[t] == 0` | `hist[t] == tile_count` |
| `faction in M` | popcount vector | `or_mask & M == 0` | `and_mask & M == M` |
| `has any unit` | `unit_count` | `unit_count == 0` | — |

**The pyramid is an optimisation with a guaranteed fallback, not the only
query path.** If a predicate matches 5 percent of tiles scattered uniformly,
almost every cell returns `Some`, and the descent costs the summary reads on
top of a full scan. Evaluate at L2 first. If the fraction of `Some` verdicts
passes a threshold — start at 50 percent — abandon the descent and run a
flat vector scan of the L0 arrays. **Build the flat path first and the
descent second.** That order is safer, and the flat path is the worst case
that must not be exceeded.

Reject summed-area tables. One tile write dirties a whole quadrant, a `u8`
field needs a `u64` accumulator (134 MB for one field), and SAT cannot do
min, max or argmax. Record it as considered and rejected: it is a
read-optimal, write-hostile structure and this workload writes constantly.

Do not build a Fenwick tree now. Note it as the answer if a future feature
needs "sum of field X over an arbitrary rectangle" at high frequency. It
composes with the pyramid rather than replacing it. The mip pyramid answers
the query Cachette actually asks — "aggregate over a fixed pyramid cell" —
in one array read.

#### D18. One field registry generates the accessor, the combine, the summary slot and the predicate

The brief's decision 6 defines the summary schema circularly, as "the fields
selectors filter on". That definition will drift, because the schema and the
predicate list live in different files.

One declaration generates four things:

1. The L0 accessor for the column.
2. The combine function the dirty-pyramid update calls.
3. The summary struct layout and its slot offsets.
4. The Python-visible predicate constructor and its `eval_summary`.

This makes D16 mechanical: a field appears at L1 only if it declares a legal
combine. It makes pruning mechanical too: a predicate can prune only if its
field declares a combine at that level. A field with no combine still gets a
predicate; that predicate returns `Some` at every summary, and `.explain()`
must say so.

**Hard budget: 256 bytes per L1 cell, checked at compile time.** The macro
fails the build when a declaration exceeds it. At 65,536 L1 cells that is 16
MiB.

A note that falls out of the arithmetic: a `Histogram<64>` at `u16` per
bucket is 128 bytes, which is half the whole budget for one field. Summarise
unit type as a presence mask (`u64`, 8 bytes) plus a count unless the full
histogram is deliberately bought.

#### D19. Dirty tracking is per cell, not per tile

A per-tile dirty bitset over 16.7M tiles is 2 MiB, and scanning it to find a
handful of set bits costs about 70 microseconds. That is a large part of a
tick for no information gain.

L1 has 65,536 cells, so its dirty bitset is 8 KiB. L2 has 256 cells, so its
bitset is 32 bytes. Both fit in L1 cache. An exhaustive scan is about a
microsecond, and zero words are skipped with one compare.

Mark with a relaxed atomic `fetch_or`. That is deterministic in result,
because OR is commutative and associative, so the brief's decision 9
survives. Drain in ascending index order, so the work set is identical on
every run regardless of scheduling.

Under ARM's weak memory model a `fetch_or` emits a real barrier where x86
would emit almost nothing. Keep the pattern, but do not scatter atomics
casually. Report 07 promotes report 06's "disjoint outputs, indexed slot
reductions" rule from preferred to **required** for the same reason.

Do not build sub-chunk dirty masks first. A full vector reduction over 256
contiguous `u8` values is cheap, and the branch and mask logic to skip tiles
may cost more than it saves. Build them only if profiling shows chunk
recompute is hot. Recorded as OQ12.

---

### Part D — Determinism, events and the frame loop

#### D20. Determinism contract: bit-exact for one binary on one architecture

Ship this contract, and write it in the documentation in exactly these
words:

> Identical results for the same binary, the same input, and any thread
> count. Results may differ between platforms and between versions.

Report 03 recommends this and recommends architecting so the stronger
cross-platform guarantee stays reachable. Report 07 makes the case stronger:
on a fleet of identical instance types under our own control, this is not a
compromise. It is the actual deployment. The earlier framing assumed
arbitrary player hardware, and that assumption is gone.

The brief lists the determinism target as an open question. **Close it.** The
*target* could stay open; the *architecture* cannot, because it decides
whether floats are usable in state at all, and retrofitting is a full audit.

Route every simulation float operation through one `sim_math` module. Add a
lint that denies direct use of `f32::sin`, `f32::cos`, `f32::exp`,
`f32::ln`, `f32::powf` and `f32::atan2` outside it. The module is thin today
and calls `std`. Swapping it to the `libm` crate later is a one-file change.
Retrofitting the boundary after 200 call sites exist is not.

Ban all of this in the simulation, by lint where possible and by review
otherwise:

- `Instant` and `SystemTime`.
- `HashMap` and `HashSet` iteration order. Use `BTreeMap`, a fixed hasher,
  or an index-sorted `Vec`.
- Pointer or address values as sort keys or hash inputs.
- Any thread identifier or worker count in a computed result.
- Uninitialised memory read as data.
- NaN bit patterns. RFC 3514 makes NaN payloads and the NaN sign bit
  explicitly non-deterministic.
- **The algebraic float operations `f32::algebraic_add` and its family.**
  These stabilised in Rust 1.98.0 on 2026-08-20, after every research report
  was written. They permit reassociation on a per-operation basis. They are
  exactly the hazard that "Rust has no stable fast-math" used to rule out.
  Add them to the lint on day one.

Rust helps in three ways that C and C++ do not. There is no fast-math flag.
FMA contraction is off by default and RFC 3514 forbids it. `f32::mul_add` is
explicit and IEEE-defined. Transcendental functions are the remaining gap:
they call the platform libm, so glibc, musl and macOS give different last
bits, and a glibc upgrade changes results on the same machine. That is why
`sim_math` exists.

Consider reading the floating-point control register at the start of each
step. Another library loaded into the same Python process can set
flush-to-zero for the whole thread, and the setting persists. A warning is
cheap; silence is a very confusing bug.

#### D21. Counter-based RNG, keyed on (system, frame, entity, draw)

The brief does not mention randomness anywhere. That is a gap, and it must
be closed before any system uses randomness, because a retrofit changes
every call site and invalidates every replay.

> Every random draw in the simulation must be a pure function of the frame
> number, a compile-time system identifier, an entity or tile identifier,
> and a draw index. No other source of randomness is allowed.

A counter-based generator has no state to advance. It is a keyed hash of a
counter. Entity 500 gets the same value whether it is processed first or
last, and whatever the thread count. Nothing enters the snapshot except the
frame number. There is no sharing, no atomic and no contention. A draw for
one entity at one frame can be computed without replaying anything, which is
exactly what rollback needs.

Every other approach fails. A thread-local generator is seeded from the OS
and advances in scheduling order. A shared seeded generator is deterministic
only if draws happen in a fixed order, and in a parallel pass they do not.
One generator per thread seeded from the frame breaks when the thread count
changes — a trap, because it passes a same-machine test and fails on a
different machine. One generator per entity costs 8 to 32 MB and enters
every snapshot.

Write it. A `splitmix64` of a packed `(system, frame, entity, draw)` key is
about 40 lines and is good enough for game use. Pin it with known-answer
tests so no refactor can silently change it. Do not depend on `rand_philox`;
it has one release and a few hundred downloads. Read it as a reference.
Implement the uniform-integer and uniform-float mapping directly, so a
dependency bump cannot change the simulation.

#### D22. Events are type-segregated POD arenas

Classic event sourcing fails here by two orders of magnitude, not by a
margin. At 1M entities and 10 percent emitting one event per frame, a
`Box<dyn Event>` design costs 2.0 to 5.0 ms in allocation and 8 to 10 ms in
cache misses, against a 16.6 ms budget. If every entity emits, it is about
100x over budget.

Give each event type its own `Vec<T>` where `T: Pod`. A push is a bounds
check, a store and a length increment: 1 to 2 ns. The apply step becomes one
tight loop per type with no dispatch, and it vectorises.

Preallocate every arena at start-up from a measured high-water mark, so a
frame does zero allocation. Set a hard capacity and report an overflow as a
rejected command, never as a panic. `Vec::clear` on a `Pod` type is one
store to the length field, so a transient log costs nothing to reset.

The cost is open extensibility: every event type must be named at compile
time, so a plugin cannot add one. For about 30 verbs that is acceptable.

Retention memory, not CPU, is the strongest argument for the brief's
decision 12. 100,000 events per frame at 32 bytes is 3.2 MB per frame, 192
MB per second, 11.5 GB per minute. The log starts transient. Keep that.

Write to one buffer per **span index**, not per thread, and concatenate in
span-index order. A per-thread buffer concatenated in thread order is
deterministic only for a fixed thread count.

#### D23. Commands carry a total stable sort key

```rust
pub struct QueuedCommand {
    priority: u16,        // caller-set ordering class
    issuer:   u16,        // which source queued it
    sequence: u64,        // monotonic, assigned on queue
    verb:     VerbId,
    selector: SelectorHandle,
    params:   ParamBlob,  // POD bytes, or a second SelectorHandle
}
```

The key is `(priority, issuer, sequence)`. It is total and stable. It never
depends on a thread, a clock or an address.

The brief says commands sort by "issue order". Issue order is well defined
only when one source issues them. The `issuer` field removes the ambiguity
now for two bytes. Adding an ordering field later invalidates every replay
and touches every call site.

Use two queues: one for external commands from Python, one for internal
commands that systems generate. Apply external first, then internal. Do not
interleave. **A verb may not issue a command in version 1** (OQ7). If that
changes, cap the cascade depth and report the cap as a rejection; an
uncapped cascade is a source of tick-time spikes.

#### D24. Validation reads. Apply writes. This is a type-level split

```
fn validate(world: &World, sel: &ResolvedSet, params: &P)
    -> (AcceptedMask, RejectionCounts)
```

A handler reads and never writes. That is what makes parallel validation
safe without locks and what makes the apply step replayable.

Enforce it in the type system, not by convention. Phase 1 to 4 systems
receive `&World` and `&mut EventSink`. Phase 5 to 6 systems receive
`&mut World` and `&EventStream`.

A command is all-or-nothing per entity, never per field. Validate fully,
then apply fully. A partial application makes the state unreplayable.

**Classify every invariant when it is written.** The brief's decision 9
says "aggregate boundary = parallelism boundary". That is right for one
class only. An aggregate here is a region, not an entity: an entity is a row
in an array, and a row cannot own an invariant.

| Class | Where it is checked |
|---|---|
| Region-local | Inside the parallel region pass. Cheap. |
| Global scalar (for example "a faction may hold at most 500 units") | A serial reduce after the parallel pass, or a reservation from an atomic budget taken in sorted command order. |
| Cross-region (a unit moves from region A to region B) | A separate two-phase pass after the parallel pass. Never inside it. |

This classification is missing from the brief. It is cheap now and expensive
later, because it changes how every handler is written.

#### D25. Partial failure returns summaries

```rust
pub struct CommandResult {
    affected:      u32,
    rejected:      u32,
    reason_counts: [u32; MAX_REASONS],  // closed u16 reason enum
    rejected_set:  SelectorHandle,      // lazy, for chaining
}
```

Reason codes are a closed `u16` enum. A string reason means an allocation
per rejection. Map the code to a message in Python.

The rejected set is a bitset over the selected set, not a list of entity
identifiers. Stamp the handle with the frame and reject a stale handle with
a clear error; entities die, and a stale handle must not read a freed row.

#### D26. Keep only three ideas from DDD

Keep the ubiquitous language, the command/event split, and the explicit
invariant boundary. Each costs nothing and helps a lot. The command/event
split is what makes the apply step pure, and a pure apply step is what makes
replay possible.

Reject the rest, and say why in the code review that first proposes it:

- **Aggregate root objects** mean a pointer to a graph. That destroys the
  struct-of-arrays layout.
- **Repositories** hide the storage. The storage layout *is* the design.
- **Value objects that check an invariant on every write** block
  vectorisation. A newtype that compiles away is fine.
- **Domain services as trait objects** have the same vtable problem as
  polymorphic events.
- **One aggregate per entity** is the biggest trap. It turns a 1M-row array
  pass into 1M transactions.

#### D27. A static compiled schedule, and parallelism from data

Derive the conflict graph from declared access at registration time. Colour
it greedily into stages. Freeze the stage list and assert it does not change
at runtime.

Reject a dynamic work-stealing executor over systems. In a dynamic executor
the completion order of two non-conflicting systems depends on thread
timing, and eventually two such systems will share hidden state. A static
list removes the variable. A static list can also be printed and diffed
across builds.

**Get parallelism from data, not from systems.** At 1M units, one system
split across the cores with `par_chunks` already saturates them. Running two
systems side by side adds nothing and costs determinism risk. The schedule
should be narrow and deep — a few stages, each holding one or two very wide
systems — not wide and shallow. Report 07 strengthens this: Graviton has no
SMT, so 64 vCPU is 64 real cores, and single-thread performance is lower
than high-clock x86. Favour wide and simple over clever and serial.

Rayon rules:

- Use `par_chunks_mut` with an explicit size. Aim for 50 to 500 microseconds
  of work per task. A task costs 1 to 5 microseconds to spawn and join, so
  guard small inputs with a serial path.
- Own a named rayon pool with an explicit thread count, set at start-up and
  recorded in the replay header. Do not use the global pool: the process may
  also hold a Python thread pool, and oversubscription follows.
- Disjoint outputs. Each task writes only its own slice. **Required, not
  preferred**, per D19.
- Indexed slot reductions combined in index order. Never a free-form
  `reduce` over a work-stealing tree.
- One level of parallelism per stage. Nested `par_iter` gives deep task
  trees and poor locality.

Keep an ambiguity report as a development tool. Warn when two systems in one
stage touch the same data with no declared ordering.

#### D28. The frame loop: five barriers

```
  [ Python phase — Python is attached ]
 0. Python builds selectors and queues commands. Commands are inert data.

--- BARRIER 1: SEAL ---------------------------------------------
    The queue closes. Each command gets a sequence number. Rust
    detaches from the interpreter. It fixes the order once and for all.

  [ Rust phase — detached ]
 1. RESOLVE.    Evaluate every selector against the pyramid. Read-only.
                Parallel across commands.
 2. VALIDATE.   Check preconditions. Compute each command's L1 write set.
                Read-only. Parallel.
 3. PLAN.       Batch commands into conflict-free groups. Serial and cheap.

--- BARRIER 2: PLAN COMPLETE ------------------------------------

 4. EXECUTE.    Run each batch in order, its commands in parallel. The
                wide data-parallel systems run here as schedule stages.
                Commands emit EVENTS. They do not mutate the world.

--- BARRIER 3: EVENTS SEALED ------------------------------------
    Concatenate event buffers in span-index order.

 5. APPLY.      Walk the event stream. Mutate values. Mark cells dirty.
 6. STRUCTURAL. Spawn, despawn, occupancy rebuild. Kept separate because
                it invalidates every index. No system may hold a
                reference across this phase.

--- BARRIER 4: STRUCTURE STABLE ---------------------------------

 7. PYRAMID.    Walk the L1 dirty bitset. Recompute. Mark L2 parents.
                Repeat upward, with a barrier between levels.
 8. FOG / FOV.  Recompute for units that changed tile. Optionally every
                Nth tick.

--- BARRIER 5: FRAME COMPLETE -----------------------------------
    Reattach to the interpreter.

  [ Python phase ]
 9. DELIVER.    Hand the event batch to Python as arrays.
```

Five barriers. Each one exists because the next phase needs a property the
previous phase establishes. Do not add more.

**Phases 1 to 4 read the world and write only events. Phases 5 to 8 write
the world and read only events.** That split is what makes phase 4 safely
parallel. It also answers report 06's open question about read sets: no
snapshot copy is needed, because nobody writes during the read phase.

#### D29. Fixed timestep, 10 Hz

The tick rate is fixed and is recorded in the replay header. Default it to
10 Hz. Report 06's budget totals 12 to 46 ms of wall time on 12 cores; at 30
Hz that is tight and at 10 Hz it is comfortable. Raise it only after
measurement on the target.

Determinism requires a fixed step. Replay requires it. The research audience
requires it, because `step()` must be a well-defined unit.

If the simulation falls behind, run one tick and report the overrun. Never
run a catch-up loop, because that spirals.

Give each system a period and a phase offset, and stagger them: movement and
combat every tick, field of view every 2 to 3 ticks, economy every 10,
long-range planning every 30 staggered by faction. Keep every period a
constant. A data-driven period is another determinism hazard.

#### D30. Snapshot dirty cells, never the whole world

A full copy of 16.7M tiles at 16 bytes is 268 MB, which is about 27 ms of
memcpy on one core. That is more than one frame at 60 Hz and a quarter of a
tick at 10 Hz. Even at 8 bytes per tile it is about 13 ms.

Reuse the dirty bitset. Copy only the cells that changed since the last
snapshot. In a typical tick far fewer than 1 percent of tiles change, so the
copy is well under 1 ms.

The chunks hold no pointers, so a snapshot is a byte copy and a restore is a
byte copy back. This is the single largest benefit of the chosen memory
layout, and it should be stated as a *reason* for the layout, not only as a
consequence. It is also the same mechanism a future rollback needs.

Write the save format by hand over `bytemuck`, with a version field, an
endianness marker and a checksum. Do not depend on any serializer's byte
format for the authoritative save file; then a dependency bump can never
break a user's save. That is about 200 lines and it removes a class of
future pain. Use `postcard` for the header and metadata.

---

### Part E — The public API

#### D31. Python is a control plane. Set-valued commands only

`Command = (Selector, Verb, Params)`. This holds from the brief.

The FFI cost is not the risk. One PyO3 call with scalar arguments costs
about 100 ns. 2000 commands per tick cost 0.2 ms. The cliff is at about
100,000 calls per tick, where the boundary alone costs 10 ms. One million
calls costs 100 ms, which is 6 ticks per second before any simulation work.

**So do not optimise the single call. Make the vectorised path the only
convenient path.** There is about 50x headroom on the stated 2000 commands.

The strongest tactic is to not provide what should not be used. If there is
no `world.get_entity(id)` that returns an object with attributes, nobody
writes the loop.

```python
class Selector:
    def __bool__(self):
        raise TypeError("A Selector has no truth value. Use `&`, `|`, `~` "
                        "instead of `and`, `or`, `not`. For emptiness use "
                        "`.is_empty()`.")
    def __len__(self):
        raise TypeError("A Selector has no length. Use `.count()`.")
    def __iter__(self):
        raise TypeError("A Selector is not iterable. Use "
                        "`.to_numpy(fields=[...])`, or apply a verb to the "
                        "whole set.")
    def __getitem__(self, key):
        raise TypeError("A Selector does not support indexing.")
```

Do not provide `.for_each()`, `.map()` or `.iter_chunks()` in the public
API. Each one is an invitation. Return selectors, not identifiers, from
everything; if no API hands out a list of identifiers, no user can loop over
one.

Use an explicit field namespace, `units.f.health`, not a magic `_`. The Ibis
project records the usability problems of the `_` API in its own tracker:
the name is not searchable and it collides with other conventions.
`units.f.health` is longer, searchable, and autocompletes.

Type the selector by domain. `UnitSelector & TileSelector` raises a
`TypeError` when the tree is built, not an empty set at run time. The
cross-domain bridges are explicit in both directions: `.on_tiles(tile_sel)`
and `.holding(unit_sel)`.

Give it `.explain()`, modelled on `EXPLAIN ANALYZE`. It must show the plan,
the pruning verdict per level, and the estimated against the actual row
count. It must label a node that cannot prune. A user who cannot see that a
query does a full scan will report a performance bug that is really a schema
bug.

#### D32. Selector parameters use snapshot semantics. This is the only semantics

A verb may take a selector as a parameter:
`units.faction(me).attack(units.faction(enemy) & (f.health < 20))`. The
brief does not say when the parameter resolves. **Report 04 identifies this
as a determinism hole, and it must be closed before any verb takes a
selector parameter.**

Every selector resolves against the sealed pre-frame state. The result never
depends on which commands already ran this tick.

Live semantics would reintroduce the order dependence that the sealed batch
exists to remove, and it would block parallel command application, because a
command's read set would depend on another command's writes.

Document this in bold. If live semantics is ever needed, add it as an
explicit `.live()` marker that forces the command into a serial phase.

#### D33. Selector caching is valid for the Python phase and cleared at the barrier

The write model does not change during the Python phase. So a selector
resolved during the Python phase stays valid for the whole Python phase.
Clear the cache at BARRIER 1.

This needs no invalidation logic, no epoch counters and no dependency
tracking. It is correct by construction, and it covers the case that
matters, which is resolving the same selector several times in one tick.

Key the cache on a 128-bit structural hash of the normalised tree.
Normalise by sorting the children of commutative nodes on their own hashes.

Do not build a cross-frame cache in version 1. It adds a whole class of
correctness bug for an unmeasured gain.

#### D34. Results are two-level masks. Sorted arrays exist only at the FFI edge

For units: a vector of `(span_id, mask)` entries sorted by span, with a
`Full` variant for "every unit in this span matches". The span identifier
plays the role of Roaring's high bits and is already the unit of storage, so
iteration follows memory order and a verb walks the columns with no gather.
`Full` is common after a faction filter, so it makes the common case free.

For tiles: an L2 mask, a sparse L1 mask for partial L2 cells, sparse L0
bitplanes for partial L1 cells, and a "whole subtree matches" mask. This
mirrors the pyramid exactly, so a descent writes its result in the shape it
produced. There is no conversion step.

Export a sorted `u32` array only at the FFI boundary, because NumPy needs a
flat array. Never use it as the internal working form.

Use a general Roaring library only for cold sparse side tables — tag
membership, upgrade-set membership, named-entity lookup — where the key
space really is arbitrary.

**This is a design argument, not a measurement.** Write the benchmark before
committing: an intersection of two 1-percent-dense sets, an intersection of
two 40-percent-dense sets, a union of ten sets, and a full iteration with a
column read.

#### D35. `to_numpy()` copies. Tile views are zero-copy. Say so plainly

**This changes a user-facing promise in the brief.**

The brief's decision 7 says "bulk data access is zero-copy NumPy views onto
component arrays". Report 05 section 3.3 shows that chunked archetype
storage cannot deliver it: chunked storage gives one array per component
*per chunk*, so a single flat view over all units does not exist in memory.

D13 changes the storage, so the position is now more favourable than report
05 could know, but the promise still does not survive in full:

| Access | Zero-copy? |
|---|---|
| A whole tile field | **Yes.** A tile column is one flat array. |
| A whole unit column, all units | **Yes**, because of D13. |
| A unit column for a *selected subset* | **No.** A subset is a gather by definition. |
| A unit column under archetype chunking | **No.** This is why D13 exists. |

So `.to_numpy()` on a selector **copies**. Rust gathers the requested
columns for the selected entities into a reusable, Rust-owned scratch
buffer and returns one flat view of it. One million `f32` values is a 4 MB
copy, about 0.3 ms. That is worth it for the ergonomics.

**Do not claim zero-copy where the engine gathers.** Say "copies" in the
docstring and in the documentation. **The tile grid is the flagship
zero-copy demonstration**, because a tile field genuinely is one flat array.

#### D36. Views live inside a scope. Three layers of protection

A NumPy array pointing at Rust memory is a raw pointer with a length. NumPy
knows nothing about Rust. rust-numpy has a borrow checker, but its own
documentation says it does not defend against unsafe Rust, other threads, or
callbacks that mutate the array. Our risk is exactly that. **This is the
highest-severity risk in the whole boundary area: a use-after-free with no
check by default.**

Use three layers together. Any one alone is not enough.

1. **Explicit scope, the primary defence.** Views exist only inside a
   context manager. On exit, reset each array's shape to zero and clear its
   writeable flag. A later read then returns an empty array rather than
   freed memory.
   ```python
   with world.tiles.view() as v:
       elevation = v.elevation        # zero-copy
       mask = elevation > 100
   # every array from v is now zero-length and read-only
   ```
2. **Generation stamps, the backstop.** The world holds a structural
   generation counter. Every structural change increments it. Every view
   records the generation at issue. Every entry point that receives a view
   checks the stamp and raises `StaleViewError`. This covers a stale view
   passed *back* into the engine.
3. **A structural-change lock.** While any scope is open, `step()`,
   `spawn()` and `despawn()` raise `ViewsOpenError`. It is a counter, not a
   real lock. It turns a use-after-free into a clear exception.

At the barrier, bump the generation and close any scope the user forgot.
Log a warning.

**A view scope may not span a `step()`.** The clean answer is far easier to
make correct. Relax it later only if a real use case appears.

Do not rely on an `Arc` keeping the allocation alive. It keeps the
*allocation*, but a swap-remove still moves another entity's data into the
slot the view points at. The reader then sees wrong data with no crash, and
silent wrong data is worse than a crash.

#### D37. Split the crates. `cachette-core` has no PyO3 dependency

`Python::detach` requires that the closure implement `Ungil`, which is not
implemented for `Python<'py>`. So a `Python` token cannot cross into the
step. **The compiler enforces "no Python callback fires mid-step."** No
discipline and no review is needed.

Make it structural as well as type-level. Put the simulation in
`cachette-core` with **no PyO3 dependency at all**. Put the bindings in
`cachette-py`. Then calling Python from the step is not merely hard; it is
a compile error, because the core crate does not know Python exists.

This pays off a second time: `miri` cannot run PyO3 code, and the entity
store has unsafe code by necessity — raw column pointers, casts to component
slices, manual layout. Run `miri` on `cachette-core` in CI. It finds
aliasing and provenance errors that no test will.

This is expensive to retrofit. Do it in the first commit.

#### D38. No Python-registered verbs. No sub-interpreters. Several worlds instead

**Python may not register a verb.** A Python verb would have to run
in-frame, and everything in D37 breaks. Custom verbs need a Rust crate. Say
this explicitly in the documentation, because users will ask, and a "maybe
later" answer will leak into the design.

PyO3 does not support sub-interpreters and raises `ImportError` on a second
one. PEP 734 added `concurrent.interpreters` in Python 3.14, so users will
try it and will hit that error. Document it.

Run several simulations as several `World` objects in one interpreter. Each
holds its own arena, its own rayon pool handle and its own generation
counter. On a GIL build, two `step()` calls in two Python threads run in
parallel because both detach.

**This requires no global mutable state anywhere in the Rust crates.** No
`static mut`, no global registry. The verb registry and the stat table are
immutable after construction and may be shared behind an `Arc`. Write this
into the rules; it is easy to violate accidentally and hard to unwind.

Make `WorldBatch(n=64).step()` a first-class API, not an afterthought. It
steps 64 worlds with rayon across worlds, so it is one FFI call for 64
steps and perfect parallelism. It is the highest-value feature for the
research audience and it constrains the `World` API, so decide it early.
Recorded as OQ10.

#### D39. Verb dispatch is a function table

Dispatch happens once per command, not once per entity. A tick carries a few
thousand commands, so even a virtual call is far below the noise floor.
**The usual enum-against-trait-object comparison does not matter here, and
the ADR says so.** Choose on ergonomics.

```rust
pub struct VerbDescriptor {
    pub name:   &'static str,
    pub id:     VerbId,       // u16
    pub domain: Domain,       // Units | Tiles
    pub params: ParamSchema,  // validated in Python at issue time
    pub access: AccessSet,
    pub apply:  fn(&mut World, &ResolvedSet, &Params) -> VerbReport,
}
```

A table gives four things an enum does not: runtime registration for test
stubs, introspection so the Python bindings and the documentation generate
from one source, a parameter schema that validates in Python and produces a
good error instead of a Rust panic, and a path to plugin verbs with no
change at the call site.

Declare each verb's component access **and its region scope**. Two commands
may run in parallel if their component accesses do not conflict in the Bevy
sense, or if both are region-local and their resolved regions are disjoint.

**The interesting difference from Bevy: the region is dynamic**, because it
comes from the resolved selector. So the scheduler runs after resolution and
rebuilds the conflict graph each tick. With a few thousand commands that is
a few million bitmask comparisons, well under a millisecond, and bucketing
on the L2 mask first reduces it further — two commands whose L2 masks do not
intersect cannot conflict on a local region, and that test is one AND.

Under-declaration is unsound and causes a data race. In debug builds, wrap
component access in a tracking guard and assert every access was declared.
Run it in CI on every verb. Over-declaration is slow and silent; report the
achieved parallelism per tick and name the top serialising verbs.

A global command such as "all units of faction X retreat" has a map-wide
write set and serialises everything. Detect this case and convert it into a
data-parallel pass over the selector result rather than a command with a
write set.

#### D40. Upgrades and capabilities are two things

**Conflict resolved in favour of report 04. Report 07 adopts the same
split.**

The brief's decision 8 makes upgrades a `u64` bitmask and treats that mask
as both the authored content and the hot predicate. Only one of those fits
in 64 bits.

- An **upgrade** is authored content. An author defines hundreds or
  thousands. Unbounded.
- A **capability** is a predicate that hot code tests: `CAN_SWIM`,
  `IS_RANGED`, `IGNORES_ZOC`. There are few, because each one is a branch
  someone wrote in Rust.

```rust
struct UnitRow {
    unit_type:    UnitTypeId,     // u16, index into the stat table
    upgrades:     UpgradeSetId,   // u32, interned, authoritative, unbounded
    capabilities: CapabilityMask, // u64, derived, hot
}
```

`UpgradeSetId` interns into a deduplicated table of upgrade *sets*. Real
populations hold few distinct sets, because units are upgraded in batches. A
grant is "look up or insert the union". `capabilities` is recomputed
whenever `upgrades` or `unit_type` changes:
`type_base_caps | OR(caps of each upgrade in the set)`.

The 64-bit ceiling now applies only to capabilities, where 64 is genuinely
enough, and where exceeding it means someone wrote 65 special branches. That
is a design smell worth catching. Allocate the bits from a central registry
with a compile-time check (OQ8).

Rejected alternatives: `[u64; 4]` moves the ceiling from 64 to 256 and costs
four times the bandwidth in every hot loop that tests one bit. A Roaring
bitmap per unit puts a heap allocation and a pointer chase in the entity
row.

**Failure mode:** if content lets each unit accumulate a different set, the
intern table grows toward one entry per unit and the saving disappears.
Monitor the table size and hit rate, and report it as a content problem when
the distinct-set count passes a threshold.

#### D41. Types are data. Bake them once

`UnitTypeId(u16)` indexes an immutable stat table of parallel columns:
move speed, attack, defence, max health, base capability mask, and a
flattened terrain-cost matrix.

Copy Factorio's model: a mutable authoring phase, a one-time bake, and an
immutable run-time table. The bake assigns the dense `u16` indices,
validates references, and lays out the columns. Give the authoring format
inheritance, as RimWorld does; without it an author copies fifty fields to
change one.

The stat lookup is a gather, and that is this design's one real cost. It is
small: at most 65,536 types, in practice a few hundred, so the table fits in
cache. Do not sort for this join. Add a per-span "dominant type" field and a
"homogeneous" flag, so the lookup hoists out of the inner loop when a span
is homogeneous. Measure the homogeneity rate before relying on it.

#### D42. About 12 load-bearing verbs in version 1

Report 04 identifies 34 named verbs, of which 17 need genuinely new Rust.
Several of those are optional, so a version 1 core is about 12.

Apply three tests before adding any verb:

1. **Symmetric pair.** If it mirrors an existing verb, it is a sign, not a
   verb. `heal` and `damage` are one `adjust_vital` with a signed delta.
2. **Parameter.** If it differs only in a constant, it is a parameter.
   `attack_move` is `move_to(dest, stance=AGGRESSIVE)`. `guard` is
   `follow(sel, stance=DEFENSIVE)`. Say this in the documentation, because
   reviewers will ask for both by name.
3. **Composition.** If it is two verbs in sequence, it belongs at the
   composite layer, not in new Rust.

Version 1 core: `move_to`, `teleport`, `adjust_vital`, `attack`, `spawn`,
`despawn`, `transform`, `set_terrain`, `build`, `claim`, `harvest`,
`reveal`.

The set-valued form is not just a batched loop. In several cases it is a
better algorithm, and in one case the per-entity form is simply wrong:

- **`attack`.** The L1 faction popcount gives "does this cell hold two
  hostile factions" as one AND. Well under 1 percent of cells are contested
  on a typical front. The search collapses from every attacker against every
  target to a local ring scan inside contested cells. This is an algorithm
  change, and it is a stronger argument for the pyramid than selector
  pruning is.
- **`reveal`.** A scanline delta buffer costs O(N*r) writes and one prefix
  sum, against O(N*r^2) with heavy overdraw. Overlapping discs simply add
  their deltas. It is exact and it is order-independent, because integer
  addition commutes. It is not expressible one unit at a time.
- **`transfer`.** Sum total demand, compute one scale, write the floor, then
  distribute the remainder by the largest-remainder method in canonical
  order. This conserves exactly. **The per-entity loop is subtly wrong**: its
  result depends on iteration order, late receivers get nothing, and a
  parallel version can drive the pool negative.
- **`spawn`.** Descend to the free-capacity summary, split N across eligible
  cells by one deterministic multinomial draw seeded from
  `(frame, command_seq)`, then fill each cell with one linear scan. There is
  no retry loop and the cells fill in parallel.
- **`adjust_vital`.** Algorithmically nothing. Include it in the
  documentation *because* it is trivial: a signed delta over a `u16` column
  under a span mask autovectorises, so even the simplest verb benefits.

#### D43. The extensibility ladder, staged and mostly deferred

| Stage | Cost | Verdict |
|---|---|---|
| 1. Fixed verbs, parameterised by data | zero, it is the plan | Version 1 |
| 2. Composite verbs: a named sequence of `(verb, binding)` | 1 to 2 weeks | First extension, and only after usage shows which compositions repeat |
| 3. A vectorised expression DSL over columns | 4 to 8 weeks | **Highest value-to-cost step in the ladder.** Defer, but design for it |
| 4. A bytecode VM | 3 to 6 months, plus a debugger that is not optional | Defer indefinitely |
| — Native Rust plugin ABI | 2 to 4 weeks | Acceptable for the author. Unacceptable as the documented path, because the Rust ABI is unstable and a plugin can break determinism and corrupt memory |

Stage 3 is cheap relative to its value because the selector engine already
has an expression tree, a Python builder, operator overloading, a type
checker and a vectorised evaluator. It only extends the node set from
predicates to arithmetic. **So build the selector evaluator in a way that
generalises: keep the node types separate from the boolean combination
logic.** That costs nothing now and is what makes stage 3 cheap later.

The one design note that matters for stage 4, if it ever happens: **each
opcode must operate on a whole column.** A scalar per-entity VM recreates
exactly the cost this architecture exists to remove. This decision cannot be
made later.

---

### Part F — Algorithms

#### D44. Portal graph plus cached flow tiles. Never a map-wide field

A map-wide flow field is not an option. 16.7M nodes and about 50M directed
hex edges give 150 to 250 ms for one bucket-queue Dijkstra, and 50 MB of
arrays per destination.

Two levels:

- **The plan.** A portal graph over pathing chunks. On each chunk border,
  the maximal runs of mutually passable cells become portal nodes.
  Intra-chunk edges carry the precomputed walk cost between portal pairs.
  One A* over it costs 50 to 500 microseconds. This is HPA*, which reports
  paths usually within 1 percent of optimal. Use L2 to prune it further: if
  a region has no passable connection toward the goal, skip its portals.
- **The steering.** Build a flow field *inside one chunk only*, with the
  chunk's exit portal as the goal. A 32x32 chunk is 1024 cells and about
  3000 edges, so 5 to 20 microseconds. Build lazily, only when a unit is
  about to enter. Most chunks on a long path never need one, because the
  group is redirected before it arrives.

**The pathing chunk is 32x32. The aggregation cell is 16x16.** This is a
derived consequence of D9 that no report covers, and it is the same lesson
as D9 applied a second time: these are different granularities. At 16x16
there are 65,536 chunks, so a portal graph built on the aggregation cell
would hold roughly 400,000 nodes and 1.6M edges — about 4x report 06's
figures. Keep the pathing chunk at 32x32, which is a 2x2 group of L1 cells,
so it stays at about 16,384 chunks and 100,000 portal nodes under 10 MB.
**This needs a benchmark; it is reasoning, not measurement.**

**Key the flow-tile cache on `(chunk_id, exit_portal_id)`, not on the
command.** Two commands routing 5000 units each through the same chunk
toward the same portal share one tile. About 4096 entries LRU, about 12 MB.
Drop every entry whose chunk is dirty in the terrain bitset.

Invalidation has three tiers, cheapest first: drop the flow tiles for a
dirty chunk (nearly free); recompute that chunk's intra-chunk portal costs
with one Dijkstra per portal (50 to 200 microseconds); mark affected command
plans stale and re-plan each when its units next reach a chunk boundary,
which spreads the cost and hides the latency.

Bound the repair work at K chunks per tick. Units in a not-yet-repaired
chunk follow the stale field for one more tick. A visible one-tick lag beats
a frame spike. A player who spams wall construction will otherwise dirty
many chunks per tick.

Detect a low cache hit rate. Many small groups with many distinct
destinations defeat the cache; fall back to snapping the goal to the nearest
portal, which forces sharing.

#### D45. Per-unit steering is a table lookup plus a three-term blend

```
desired = w_flow * flow_dir
        + w_sep  * separation_from_occupancy_index
        + w_coh  * formation_offset
```

ORCA and RVO are not affordable. Each solves a small linear program per
agent per frame: 1 to 5 microseconds, so 1 to 5 seconds at 1M units.

The separation term reads the occupancy index of the 6 hex neighbours and
the own tile, and pushes away from the crowded ones. **No neighbour search
is needed, because the occupancy index is the search result.** About 7
offset reads and a few multiplies: 20 to 40 ns per unit.

Add "unit density raises local cost" when building a flow tile. That is the
one idea worth taking from Continuum Crowds. It handles macro congestion, so
the separation term only has to stop visual overlap.

This gives good mass movement and poor individual movement. Units clip and
jostle. For 1M units that is the right trade. If one unit class needs exact
avoidance, run ORCA for that class only and keep its count in the hundreds.

#### D46. The pyramid is the spatial index. Do not add a second structure

A separate uniform hash grid would hold the same information twice and add a
second invalidation path that will fall out of sync in some rare case. The
pyramid is already maintained by the dirty walk, so the marginal cost of
using it for queries is zero. If a query wants a different cell size, choose
a different *level*, not a different structure.

#### D47. Field of view is shared, quantised and cached

Use recursive shadowcasting over **6 sextants**, not 8 octants. Within a
sextant, index by `(ring, position_in_ring)`, scan by increasing ring, and
maintain open angular intervals in units of `position / ring`. This is
O(3 R^2). At radius 12 it is 469 hexes, about 2 to 6 microseconds.

Do not draw a Euclidean line to every hex; lines overlap heavily and examine
most hexes several times.

**This is the real scaling problem.** 1M units at 3 microseconds each is 3
seconds. Four mitigations, all needed:

1. Recompute only for units that changed tile. Typical churn is 5 to 20
   percent.
2. Share across stacked units. Units on the same tile with the same sight
   radius have identical field of view. Group by `(tile, radius)` first. In
   dense armies this collapses thousands of computations into dozens.
3. Quantise sight radius to a few values, for example 4, 8, 12, 16. This
   makes step 2 far more effective. See OQ13.
4. Cache by `(tile, radius)` with terrain-version validation.

With all four, expect 10,000 to 50,000 real computations per tick, which is
still 3 to 13 ms of wall time. Run field of view every second or third tick.

#### D48. Fog of war is two bitsets and a saturating counter per faction

| Structure | Type | Size at 16.7M tiles |
|---|---|---|
| Explored (ever seen) | 1 bit/tile | 2.10 MB |
| Visible (seen now) | 1 bit/tile | 2.10 MB |
| Visibility count | u8/tile | 16.8 MB |
| **Per faction** | | **21.0 MB** |

The counter is what makes the update incremental. Increment when a field of
view gains a tile, decrement when it loses one; a tile is visible when the
count is above zero. **Do not rebuild the visible bitset from scratch each
tick**; that costs a 2.1 MB clear plus a full re-scatter per faction.

The `u8` counter saturates at 255, which is a real risk with deep stacking.
Use saturating `u8` plus a periodic full rebuild, about every 600 ticks, to
correct the drift. Amortise the rebuild by doing one L1 row per tick.

Deliver fog to Python as a NumPy view over the bitsets, never as per-tile
calls. This is one of the zero-copy demonstrations that D35 preserves.

**Memory scales linearly with the faction count. See OQ5; the report 04
figure of 2.1 MB per faction is the bitplane only, not the whole cost.**

#### D49. Freeze the computation. Keep the data

**Stage 1, version 1: freeze, with a deterministic resume.** Record
`frozen_at_tick`. On thaw, apply a catch-up function to the L0 state: grow
stockpiles by rate times elapsed, advance production queues, age units. This
is pure L0 arithmetic.

**Stage 2: an active set, Factorio style.** A cell is active when it holds
an observer, holds a unit with a pending command, borders an active cell, or
holds a queued event. Simulate the active set only. Everything else stays
frozen and gets the stage 1 catch-up on activation.

**Stage 3, optional: a coarse layer for genuinely global processes only.**
Diffusion-like quantities: influence, supply, pollution, migration pressure.
These are already averages, so a coarse model gets them right. Run over all
65,536 L1 cells at a low rate, for example every 64 ticks. The cost is
microseconds. Note Factorio's lesson: the coarse layer is a **different
model**, not a downsampled copy of the fine model, and it runs at a lower
rate.

**Stage 4, probably never: generative promotion from a summary.** See "What
research refuted", item R1.

#### D50. Build one very good parallel radix sort

"Sort by spatial key" appears in batched nearest neighbour, sort-merge
joins, event apply, the occupancy rebuild and chunk locality. It will be the
second most used primitive after the flow tile.

It must be **stable**, because stability preserves issue order within equal
keys, which D23 requires. 1M `u32` keys cost about 4 to 8 ms
single-threaded and it parallelises well.

Batched algorithms that depend on it:

- **Multi-source Dijkstra.** "For every cell, which of my 200 cities is
  nearest?" Seed one queue with all 200 sources at distance 0, each tagged
  with its source. One pass gives the full Voronoi partition and the
  distance field. Territory control, supply range, nearest depot, zone of
  control and threat range are all this same algorithm. Run it at L1
  (65,536 cells), not at L0, and refine to L0 only inside cells a boundary
  crosses.
- **Sort-merge joins in place of per-item hash lookups.** Any "for each
  unit, look up something by key" is a join. The per-item form is a likely
  cache miss at about 100 ns, so 100 ms at 1M units. Sorted and merged, both
  sides stream: 5 to 15 ms, and it parallelises. **Exception: the stat
  table.** It is a few hundred entries and fits in cache. Index it directly.
- **Influence maps.** A 7-point hex stencil over the L1 grid. Maintain 4 to
  8 maps, each 65,536 cells. Update them in one fused pass. Expose them to
  Python as NumPy views; researchers will want exactly this.

**Keep the units roughly sorted by spatial key between ticks.** Then the
per-tick sort is an almost-sorted pass and is nearly free, and every other
pass gets better cache locality. This interacts with determinism, because it
changes iteration order. See OQ14.

---

## What research refuted

This section exists so that these ideas are not proposed again. Each was
believed at the start of the work and was then disproved.

**R1. The promotion and demotion problem does not exist at this scale.**
Source: `06-algorithms-and-scheduling.md` section 7.4.

The brief treats "freeze or simulate the background" as one hard question,
and the promotion problem is what makes it hard: an L1 summary is a monoid
reduction, so it threw information away by construction and cannot be
inverted. Materialising 15,000 plausible units from a summary means
inventing, and the invention must not contradict what the player remembers.

**The whole problem only arises if L0 data is discarded. At 4096x4096 it
does not have to be.** The full L0 tile state is about 134 MB of hot fields.
Freezing the *computation* and discarding the *data* are separate decisions,
and the brief conflates them slightly. Separate them and stage 4 disappears.
Revisit only if the target grows to something like 65536x65536.

**R2. "One flow field for 5000 units" understates the machinery.**
Source: `06-algorithms-and-scheduling.md` sections 3.3 and 3.4.

The brief's decision 11 says `move_to` for 5000 units computes one
hierarchical flow field. One field is correct only when the units share a
destination and a region. In the general case the engine computes one
*plan* over the portal graph and several *flow tiles*, and it reuses the
tiles across commands. **The saving is real and larger than the brief
claims, but it comes from the shared cache keyed on
`(chunk, exit_portal)`, not from a single field.** Do not write the brief's
wording into the documentation; it will set the wrong expectation about
where the cost lives.

**R3. H3 aperture 7 *can* aggregate exactly. The original reason for
rejecting it was wrong.** Source: `02-hex-grid-and-lod-pyramid.md`
section 3.1.

The belief was that non-exact nesting makes aggregation inexact. That is
false. H3's own documentation states that **logical containment in the index
is exact**; only *geographic* containment is approximate. Define a parent's
aggregate as the fold over its seven logical children and the fold is exact,
complete and disjoint. Nothing is double counted. The only thing that is
inexact is the match between the aggregate and the *drawn shape*.

The pentagon objection also does not apply. H3 has 12 pentagons per
resolution only because it projects an icosahedron onto a sphere. A planar
aperture-7 hierarchy has none. Do not use pentagons as an argument.

**The conclusion in D8 still stands, on the real costs:** 7 is not a power
of two, so parent lookup is a base-7 digit shift over Eisenstein integers
rather than a bit shift; level sizes of 7, 49 and 343 never align to a cache
line or a vector width; and a parent's seven children are not contiguous in
any simple linear order, so an explicit child index table is needed.

**R4. A full snapshot is not viable.** Source:
`03-event-sourcing-cqrs-determinism.md` section 8.2.

268 MB of tile state is about 27 ms of memcpy on one core, which is more
than a 60 Hz frame. Even at 8 bytes per tile it is about 13 ms. The brief
does not state this cost. It forces chunk-level copy-on-write, and that in
turn is exactly the mechanism a future rollback needs. Recorded in D30.

**R5. 32x32 blocks are not required for race-free bitset writes.** Source:
this record, D9, reading `01-ecs-and-memory-layout.md` section 10 against
`07-target-platform-and-value-types.md`.

Report 01 argues that a 32x32 bitplane block is exactly two cache lines and
that this makes parallel writes race-free by construction. **Correctness
needs whole-*word* alignment, not whole-*line* alignment.** A 16x16 block is
4 whole `u64` words, so no update can be lost. Only false sharing, a
performance effect, needs the line. The argument does not support the block
size it was used to support.

**R6. Report 07's bitplane padding cost is understated by the plane count.**
Source: this record, D9.

Report 07 states the cost of padding a 16x16 bitplane block to a cache line
as "about 2 MB across the map". That is 2 MiB *per plane*. A 16-plane schema
pays 32 MiB against a 160 MiB tile side. The padding is not needed anyway;
constraining the parallel split to an even number of blocks costs nothing.

**R7. LSE does not have to be enabled explicitly.** Source: verification,
item V9, against `07-target-platform-and-value-types.md`.

Report 07 says that Rust's `aarch64-unknown-linux-gnu` baselines at ARMv8.0,
so LSE atomics compile to LL/SC retry loops unless a flag is set. The
baseline claim is right. **The conclusion is wrong**, because
`outline-atomics` has been on by default for that target since about Rust
1.57. Atomics go through `__aarch64_cas*` and `__aarch64_ldadd*` with
runtime dispatch on HWCAP, so they already take the LSE path on Graviton 2
and later with no flag at all. Pinning `-C target-cpu` still helps by
inlining LSE and dropping the dispatch, but it is an optimisation, not a
correctness or a contention fix.

---

## Verification of external claims

Checked on 2026-08-30 against current documentation. Report 07 explicitly
asked for this. Nothing below is asserted on trust.

| # | Claim | Result | Current fact |
|---|---|---|---|
| V1 | PyO3 `attach`/`detach` rename | **Verified, with a correction** | PyO3 0.29.2. The rename landed in 0.26.0. **The old names were removed in 0.28.0, not deprecated.** Report 05 says "renamed"; treat them as gone. Also `prepare_freethreaded_python` -> `Python::initialize`, `GILOnceCell` -> `PyOnceLock`. |
| V2 | rust-numpy tracks PyO3 | Verified | `numpy` 0.29.0, depends on pyo3 0.29. Minor-for-minor. Pin and upgrade together. |
| V3 | abi3 minimum | **Verified, with a correction** | The minimum is **`abi3-py39`**, not py311. Features go to `abi3-py315`. Report 05's *recommendation* of `abi3-py311` still stands on its own merits (the buffer protocol under abi3 needs 3.11), but the floor is lower than stated. |
| V4 | PEP 703 | Verified | Final. Python 3.13. |
| V5 | PEP 779 | Verified | Final. Python 3.14. Free-threaded Python is **officially supported, not experimental**, from 3.14. Default-on is a later phase and is not scheduled. |
| V6 | PEP 803 / `abi3t` | Verified | Final, adopted 2026-03-30, **Python 3.15**. Classic abi3 does **not** work on free-threaded builds; PyO3 warns and ignores it. So 3.14t still needs version-specific wheels. PyO3 0.29.0 dropped 3.13t and added the `abi3t` features. |
| V7 | `std::simd` still nightly | Verified | Tracking issue #86656 open. No stabilisation PR. D2 stands. |
| V8 | RFC 3514 | **Verified, with a new hazard** | The RFC is accepted; tracking issue #128288 is open for documentation. **`f32::algebraic_add` and its family stabilised in Rust 1.98.0 on 2026-08-20.** They permit per-operation reassociation. No research report could know this. D20 bans them. There is still no global fast-math flag. |
| V9 | LSE in ARMv8.1, Graviton support | **Verified fact, refuted conclusion** | FEAT_LSE is Armv8.1-A. Graviton 2 and later support it. Rust's target does baseline at Armv8.0, **but `outline-atomics` is on by default on Linux aarch64, so the LSE path is already taken at run time.** See R7. |
| V10 | Graviton to Neoverse `target-cpu` | Verified | The mapping in D1. Graviton 5 exists (Neoverse-V3, 192 cores). Valid rustc names: `neoverse-e1/-n1/-n2/-n3/-v1/-v2/-v3/-v3ae`, `neoverse-512tvb`. |
| V11 | No scalar aarch64 popcount | **Verified for the target; the flat claim is out of date** | Default AArch64 codegen is `fmov / cnt.8b / addv.8b / fmov`. **FEAT_CSSC adds a scalar `CNT`** (optional Armv8.7, mandatory Armv8.9). No Neoverse `target-cpu` enables it; it appears only for AppleM5, Ampere1B and Ampere1C, and rustc's `cssc` feature is unstable. **Graviton 1 to 5: no.** So D11 is right on the target, but write it as "no scalar popcount on Graviton and Neoverse", not "on AArch64". |
| V12 | 64-byte Neoverse lines, 128-byte Apple | Verified | N1, N2, V1, V2 use 64 bytes at every level. Apple M-series uses 128. D3 stands. |
| V13 | NEON is baseline | Verified | Advanced SIMD and FP are mandatory in Armv8-A AArch64. `target_feature="neon"` is always on. The exception is bare-metal softfloat targets, which do not apply. D2 stands. |
| V14 | No SMT on Graviton | Verified | 1 vCPU is 1 physical core on every generation. |
| V15 | SVE per generation | Verified, **and worse than report 07 implies** | G3 has SVE1 at **256-bit**. G4 and G5 have SVE2 at **128-bit**. That is a width regression from G3 to G4. A width-sensitive kernel can be *slower* on Graviton 4 than on Graviton 3. This strengthens D2's "skip SVE, write one NEON path". |

**Nothing in this record is marked UNVERIFIED.** Every load-bearing external
claim was checked. Items that are *reasoning rather than measurement* are
marked as such at the point they appear: the block-size false-sharing
analysis (D9), the 32x32 pathing chunk (D44), the purpose-built mask against
general Roaring (D34), and the span length of 4096 (D13).

---

## Byte budgets

These tables are load-bearing. Read them as budgets, not as plans. At 16.7M
tiles every extra byte per tile costs 16 MiB of memory and 16 MiB of
traffic on any full-grid pass, and a full-grid pass costs at least 3.3 ms of
pure bandwidth.

### Tile side, L0

| Field | Type | Bytes/tile | Total |
|---|---|---|---|
| terrain | u8 | 1 | 16 MiB |
| elevation | i16 | 2 | 32 MiB |
| owner | u8 | 1 | 16 MiB |
| moisture | u8 | 1 | 16 MiB |
| temperature | u8 | 1 | 16 MiB |
| resource type | u8 | 1 | 16 MiB |
| resource amount | u8 | 1 | 16 MiB |
| 16 boolean flags | 16 bitplanes | 2 | 32 MiB |
| **Subtotal, rich schema** | | **10** | **160 MiB** |

| Schema | Bytes/tile | Total |
|---|---|---|
| Minimum: terrain, elevation, owner, 8 flags | 5 | 80 MiB |
| Rich, as above | 10 | 160 MiB |
| Very rich: 4 more u8 and 16 more flags | 16 | 256 MiB |

### Pyramid and indexes

| Structure | Size |
|---|---|
| L1 dirty bitset (65,536 cells) | 8 KiB |
| L2 dirty bitset (256 cells) | 32 bytes |
| L1 summaries, at the 256 B/cell cap | 16 MiB |
| L2 summaries | 64 KiB |
| Unit pyramid at L1, delta-only | within the same cap |
| Sorted unit index (1M x u32) | 4 MiB |
| Sorted tile key (1M x u32) | 4 MiB |
| Per-block occupancy ranges (65,536 x u64) | 512 KiB |
| "Block holds any unit" bitplane | 8 KiB |
| `EntityMeta` table (1M x 16 B) | 16 MiB |
| Hot unit columns (1M x 32 B) | 32 MiB |
| Portal graph (about 100k nodes plus edges) | under 10 MiB |
| Flow-tile cache (4096 entries LRU) | about 12 MiB |
| Influence maps (8 maps x 65,536 cells) | about 2 MiB |
| **Fog of war, per faction** | **21.0 MB** |

Structures that were rejected on size, and are recorded so they are not
proposed again:

| Rejected structure | Size | Replaced by |
|---|---|---|
| Full-grid CSR occupancy offsets | 64 MiB | Block-level ranges plus a sorted array, about 8.5 MiB (D15) |
| Per-tile dirty bitset | 2 MiB, ~70 us to scan | Per-cell bitsets, 8 KiB (D19) |
| Per-entity change ticks, Bevy style | 80 MB of writes per frame | Per-span ticks, about 100 KB (D13) |
| Summed-area table, one u8 field | 134 MB | The mip pyramid (D17) |
| One map-wide flow field | 50 MB per destination | Portal graph plus flow tiles (D44) |
| Full snapshot each frame | 268 MB, about 27 ms | Dirty-cell copy-on-write (D30) |
| `[u64; 4]` capability mask | 32 MB at 1M units, 4x the hot bandwidth | `UpgradeSetId` plus a derived `u64` (D40) |

**Rough total for the rich schema at 1M units and 8 factions:** 160 MiB of
tiles, plus about 16 MiB of pyramid, plus about 25 MiB of indexes and
bridge, plus about 32 MiB of unit columns, plus about 22 MiB of pathing and
influence, plus 168 MB of fog. **Fog is the largest single line item after
the tile grid.** See OQ5.

---

## Per-tick cost budget

From `06-algorithms-and-scheduling.md` section 10, for one tick at 1M units
and 16.7M tiles.

**These figures assume 12 cores at about 3.5 GHz and about 40 GB/s of
bandwidth. The target is Graviton: more cores, lower clock per core, no
SMT. Treat the core-ms column as the reliable one and re-derive the wall-ms
column on the target.** This is the direct consequence of D1 and it is why
D27 asks for wide-and-simple stages.

| Work | Scale | Core-ms | Wall-ms (12c) |
|---|---|---|---|
| Selector resolution, pyramid descent | 100 commands | 5-20 | 0.5-2 |
| Portal-graph A* | 100 plans | 5-50 | 0.5-4 |
| Flow tile builds | 100 tiles | 0.5-4 | 0.1-0.5 |
| Movement and steering | 1M units | 20-40 | 2-4 |
| Combat resolution | 100k engaged | 5-20 | 0.5-2 |
| Event concatenate and apply | 500k events | 10-30 | 2-6 |
| Structural changes | 10k spawns | 2-10 | 2-10 (mostly serial) |
| Pyramid dirty update | 5% dirty | 5-15 | 0.5-2 |
| Field of view (shared, quantised) | 30k real | 30-150 | 3-13 |
| Fog counter update | 30k deltas | 3-10 | 0.3-1 |
| Influence maps at L1 | 8 maps | 1-3 | 0.1-0.3 |
| Spatial radix sort | 1M units | 4-8 | 0.5-1 |
| **Total** | | **90-360** | **12-46** |

**The two largest items are field of view and event apply.** Attack those
first if the budget breaks. Field of view responds to the sharing and
quantisation in D47. Event apply responds to pre-partitioning events by
target region.

Two rules follow from the bandwidth figures and govern every algorithm
above:

- **A full-map pass costs at least 3.3 ms.** Afford two or three per tick,
  not twenty. Add a debug counter that reports full-map passes per tick and
  fail a test above a threshold. Someone will add "just one loop over all
  tiles".
- **Per-unit work has a hard ceiling of about 400 ns of core time** at 1M
  units and 400 core-ms per tick, shared between movement, combat and
  planning. A per-unit A* costs 10 to 100 microseconds. That is the argument
  for flow fields, stated as arithmetic.

---

## Day one, and unretrofittable

Everything in this list is cheap now and impossible or very expensive later.
Each item is a *shape* — a sort key, a purity rule, a module boundary, a
type bound, a CI job — not a feature. Build the shapes now and the features
later.

Ordered by the cost of getting it wrong.

1. **The `cachette-core` / `cachette-py` crate split, with no PyO3
   dependency in the core.** This makes a mid-step Python callback a compile
   error rather than a rule. It also makes `miri` usable on the entity
   store. (D37; report 05 section 4.1)
2. **The command-handler shape: validation reads, apply writes,** enforced
   by the types the phase receives. If handlers mutate during validation,
   the apply step is not pure, and neither replay nor rollback is ever
   possible. This shape cannot change later without rewriting every verb.
   (D24; report 03 section 12 item 8)
3. **A stable total sort key on every command and every event:**
   `(priority, issuer, sequence)`. Adding an ordering field later
   invalidates every replay and touches every call site. (D23; report 03
   section 12 item 1)
4. **The counter-based RNG interface,** keyed on
   `(system, frame, entity, draw)`, used from the first commit at every
   randomness call site. A retrofit invalidates every saved replay. (D21;
   report 03 section 7)
5. **The `sim_math` boundary plus the lint.** One file now; a full audit of
   200 call sites later. The lint must include the algebraic float
   operations that stabilised in Rust 1.98. (D20; report 03 section 6.5)
6. **The ban on ambient non-determinism,** in a lint where possible and in
   review otherwise: no clock, no unordered hash-map iteration, no
   addresses, no thread count in a result. (D20)
7. **`Pod` on every event and every component,** checked at compile time,
   with `repr(C)` and explicit padding. This is what keeps the memcpy
   snapshot possible and what stops padding bytes from making the state hash
   non-reproducible. (D5, D22, D30)
8. **A frame sequence counter in the world state,** with every event stamped
   with it. It is the key for the RNG and for the log. (D21)
9. **The determinism test in CI, from the first week.** Run the same tick
   with 1, 2, 12 and 64 threads and compare the event log byte for byte.
   Hash the world state at the end of every tick and compare against a
   committed golden file. **This is the single highest-value test in the
   project.** A determinism bug found at month six is very expensive; found
   at week one it is trivial. (D20; report 03 section 6.5, report 06
   section 5.4)
10. **The selector API shape:** `__bool__`, `__len__`, `__iter__` and
    `__getitem__` all raise with a message that names the fix; no
    `for_each`, no `map`, no public `iter_chunks`; every API returns a
    selector rather than a list of identifiers. Adding a per-entity escape
    hatch later is easy; removing one after users depend on it is not. (D31)
11. **Snapshot semantics for selector parameters.** This must be decided
    before any verb takes a selector parameter, because it changes the
    scheduler. (D32; report 04 section 2.6)
12. **abi3 packaging CI, all platforms, in week one,** even against a stub.
    Include the free-threaded job and the sdist test. The sdist is the item
    that always gets skipped and always hurts, because a broken sdist is
    invisible until a user on an unusual platform tries to install. (report
    05 section 7.4)
13. **`target-cpu` flags and the cache-line constant,** set per target, with
    the Apple Silicon discrepancy recorded in the benchmark harness. (D1,
    D3)
14. **No global mutable state in any Rust crate.** No `static mut`, no
    global registry. Easy to violate accidentally and hard to unwind, and it
    is what makes several `World` objects possible. (D38)
15. **Exact-monoid accumulators in the pyramid,** enforced by the field
    registry macro rather than by review. (D16, D18)
16. **The verb registry as a table rather than a hard-coded match.** This
    keeps the door open for a second in-frame extension plane later at zero
    cost today. (D39)
17. **The node types separated from the boolean combination logic in the
    selector evaluator.** This is what makes the stage 3 expression DSL
    cheap. (D43)

---

## Staged implementation plan

The owner's stated priority is dogfooding: build a game and let the library
fall out. So the first milestone is a vertical slice that touches every
layer, not a well-built layer.

### Milestone 0 — Skeleton, week one

Nothing here simulates anything. All of it is unretrofittable.

- The `cachette-core` and `cachette-py` crate split, with no PyO3 in core.
- `.cargo/config.toml` with `target-cpu`. The `CACHE_LINE` constant.
- The value types of D5, with size assertions.
- `maturin generate-ci github`, all five platforms, `abi3-py311`, a
  free-threaded job, an sdist test, `sccache`, `cargo-deny`, a pinned MSRV.
- The `sim_math` module and its lint, including the algebraic float ban.
- The counter-based RNG with known-answer tests.
- `cargo clippy -D warnings`, `cargo miri` on core.

### Milestone 1 — The vertical slice

**Scope it small on every axis except the number of layers it touches.**

- A 512 x 512 world, so L0 is 262,144 tiles, L1 is 32 x 32, L2 is 2 x 2.
  The arithmetic is identical and the whole thing fits in cache, so bugs
  are visible.
- Three tile fields: terrain (`u8`), elevation (`i16`), one bitplane.
- One unit type. About 10,000 units in the generational SoA arena.
- Two summary fields: `unit_count` (a group) and a faction popcount vector
  (a group). Both take the delta path, so the recompute path stays untested
  for now — that is deliberate, and milestone 2 adds it.
- Three verbs: `spawn`, `move_to`, `adjust_vital`. Between them they
  exercise placement allocation, the flow machinery and a vectorised column
  write.
- Two predicates: `f.faction == x` and `f.health < k`. Enough for a
  three-valued descent with both `None` and `All` verdicts.
- The full five-barrier loop of D28, with all five barriers real.
- The event arena and the span-ordered concatenation.
- `.count()`, `.to_numpy()` (copying, honestly documented) and one
  zero-copy tile view under a context manager.
- The determinism test at 1, 2, 12 and 64 threads against a golden hash.
- One `criterion` benchmark of the full-column iteration loop, so there is a
  signal for when to stop tuning the arena (D12).

**Exit criterion:** a Python script spawns units, filters them with a
selector, moves them, reads a tile field as a zero-copy NumPy array, and
produces the same state hash at every thread count. Ship a wheel of it.

This slice is what makes the rest low-risk. Every later milestone widens one
axis of something that already works end to end.

### Milestone 2 — Widen to the real scale

- Grow the grid to the confirmed extent (OQ2). Add the rich tile schema.
- Add the recompute path: min and max with extremum counts.
- Add the second pyramid for units (D17).
- Add the flat vector scan fallback and the cost model that chooses between
  it and the descent (D17). **Build the flat path before tuning the
  descent.**
- Add huge-page arenas and `madvise` at start-up.
- Add the block-level occupancy bridge and the parallel stable radix sort
  (D15, D50).
- Add the field registry macro with its 256-byte budget check (D18).

### Milestone 3 — Movement at scale

- The portal graph on 32x32 pathing chunks, the flow-tile cache keyed on
  `(chunk, exit_portal)`, and the three-tier invalidation (D44).
- The three-term steering blend against the occupancy index (D45).
- Density in the flow-tile cost function.
- Benchmark the 32x32 pathing chunk against 16x16, which D44 flags as
  reasoning rather than measurement.

### Milestone 4 — The rest of the version 1 verb set

The remaining nine of the twelve in D42, in this order, because each one
adds a distinct piece of machinery: `attack` (contested-cell detection),
`claim` (ownership and border recomputation), `build` and `harvest` (shared
budgets and the largest-remainder split), `set_terrain` (path-cache
invalidation), `reveal` (the scanline delta buffer), `transform`,
`teleport`, `despawn`.

### Milestone 5 — Visibility and the control plane

- Sextant shadowcasting, with sharing, quantisation and caching (D47).
- Fog as two bitsets and a saturating counter, delivered as NumPy views
  (D48). **Decide OQ5 before building this**, because the faction ceiling
  changes the representation.
- `.explain()` with per-level pruning verdicts and estimated against actual
  counts.
- The custom exception hierarchy via `create_exception!`, with the Rust
  error chain attached as data. Never a bare `PyRuntimeError`.
- The panic hook that captures a backtrace into a thread-local and attaches
  it to the exception. Without it, a panic in production gives one line and
  no location.
- Hand-written `.pyi` stubs for the Python-side code, generated stubs for
  the rest, and the CI check that fails a pull request which changes the API
  without regenerating.

### Milestone 6 — Dogfood, then generalise

Build the game. Let the missing verbs and predicates announce themselves.
Audit the selector and verb list against the author's own game code, because
the author is audience number one: **if a common need has no selector form,
users will write the loop.** The operations to watch for are set difference,
top-k, nearest, sort-by and random sample.

Only then: `WorldBatch` (OQ10), composite verbs (D43 stage 2), and the
active-set simulation LOD (D49 stage 2).

Deferred, and each deferral is a decision rather than an omission: the
retained event log, rollback and time travel; cross-platform bit-exactness;
delta and compressed snapshots; netcode; the expression DSL; a plugin event
type registry; a bytecode VM.

---

## Consequences

### What this buys

- **Reproducibility as a product feature.** Same binary, same input, any
  thread count, same bytes. That is what the research audience needs for
  reproducible experiments, and D20 makes it honest rather than aspirational.
  Because the deployment is a controlled fleet, the weaker cross-platform
  guarantee costs nothing real.
- **A snapshot is a memcpy.** No serializer, no traversal, no per-entity
  allocation. This falls out of the POD discipline and is a *reason* for the
  layout, not only a consequence. It is also the exact mechanism a future
  rollback needs, so rollback becomes nearly free later.
- **Better algorithms, not just batched loops.** `attack` becomes a bitmask
  test over cells. `reveal` drops a factor of r. `transfer` becomes correct,
  where the loop form is subtly wrong. `spawn` loses its retry loop. The
  set-valued API is what makes each of these expressible.
- **Query pruning is exact where it matters.** A histogram summary gives an
  *exact* count for an equality predicate over a cell, so single-predicate
  selectivity is not an estimate. **The pyramid is the index and the
  statistics catalogue at once, and the dirty walk keeps it current for
  free.**
- **The compiler enforces the most important rules.** `Ungil` makes a
  mid-step Python callback impossible. The crate split makes it
  unrepresentable. The field registry macro makes an illegal summary field a
  build error. The 256-byte budget is checked, not advised.
- **One code path on the target.** NEON is baseline, so there is no runtime
  dispatch, no function multiversioning and no feature detection. That is
  less code and it is easier to keep deterministic.
- **A guaranteed worst case.** The flat vector scan over L0 bounds every
  query. The pyramid is an optimisation above a floor, not a structure the
  system depends on.

### What it costs

- **About 2000 lines of entity store that nobody else maintains.** Writing
  one is a well-known way to spend six months not shipping. Timebox it and
  write the benchmark first.
- **A user-facing promise is smaller than the brief claimed.**
  `.to_numpy()` copies. Only whole columns and tile fields are zero-copy.
  This must be said in the docstring, not buried.
- **No floats in state.** Every division needs a shift and overflow care.
  `sqrt`, `sin` and `atan2` need tables or polynomials — a fixed one-time
  cost of about 300 lines. Anything a user sees as a float converts at the
  boundary.
- **Event types are named at compile time.** A plugin cannot add one. For
  about 30 verbs that is acceptable, but it is a real limit and it is why
  D43 stage 4 stays deferred.
- **The scheduler rebuilds its conflict graph every tick,** because the
  region scope is dynamic. Bevy's analysis is static and cheaper. The cost is
  bounded — a few million bitmask comparisons — but it is not zero and it is
  new machinery.
- **Development machines mislead.** Apple Silicon uses 128-byte lines; x86
  has TSO and a cheap `POPCNT`. Every false-sharing and alignment
  measurement taken locally is suspect. Benchmarking must happen on the
  target.
- **A view scope may not span a step.** That is inconvenient and it is the
  price of turning a use-after-free into an exception.
- **The pyramid maintenance cost is real.** Every summary field costs memory
  at two levels and update time on every dirty cell. Fields that change every
  tick for every entity cost more than they save.

### What it forecloses

- **Per-entity Python.** By design and by construction. Not merely
  discouraged: the objects that would make it possible do not exist. A user
  who needs it must reach for `.to_numpy()` and loop over arrays, where the
  profiler will show the cost.
- **Third-party ECS crates.** D12 rules out `bevy_ecs`, `hecs` and `legion`,
  and the schedule rules out any executor Cachette does not own.
- **Sub-interpreters.** PyO3 does not support them and will not soon. PEP
  734 makes users try, so document the `ImportError`.
- **Geospatial interoperation.** Parallelogram blocks cannot publish cell
  identifiers a geospatial tool understands. If that ever becomes a
  requirement, H3 is the answer and D8 must be revisited.
- **Cross-version stability of the canonical iteration order.** Promising it
  would freeze the arena allocator. The order is stable within a released
  version and not across versions.
- **Cross-platform bit-exactness, for now.** It stays reachable through the
  `sim_math` swap plus a fixed-point pass, and the day-one items keep it
  reachable, but it is not what version 1 promises.
- **Float aggregates, permanently.** Not deferred. Banned. Allowing one
  later would reintroduce silent L0/L1 drift, which is the hardest class of
  bug in this design.

---

## Open questions

Deduplicated across all eight documents. Each is attributed to its source
and says what it blocks. The first five are for the owner and should be
answered before milestone 1.

### For the owner

**OQ1. Name three archetypes you expect to exist.**
Source: `01-ecs-and-memory-layout.md` open question 1.
*Blocks:* D12 and D13, and therefore D35. If three real archetypes exist,
the generational arena is the wrong shape, archetype machinery comes back,
and the zero-copy unit column that D13 buys is lost again. If none can be
named, D12 is confirmed and about 2000 lines of archetype code are never
written. **This is the single highest-leverage question in this record.**

**OQ2. Confirm the grid extent and the world shape.**
Source: brief open question 1; `01` open question 6; `02` open question 2;
`06` open question 1.
*Blocks:* every byte budget above, the fanout choice in D9, and whether D7's
offset conversion is needed at all. A rhombus world permits raw axial
storage and deletes the conversion. 4096x4096 is assumed throughout; state
whether that is the real target and whether the shape is a rectangle.

**OQ3. What is the real upper bound on unit count?**
Source: `01-ecs-and-memory-layout.md` open question 6.
*Blocks:* the arena sizing, the span length in D13, and the per-tick sort
cost in D50. 200k and 2M are different designs. The brief says "hundreds of
thousands to millions", which spans both.

**OQ4. May a verb issue a command?**
Source: `04-selector-engine-and-verbs.md` open question 5;
`03-event-sourcing-cqrs-determinism.md` section 4.1.
*Blocks:* D23. D23 answers "no" for version 1. If the answer is yes, the
sealed-batch model needs a defined fixed point or a depth limit, and an
uncapped cascade becomes a source of tick-time spikes.

**OQ5. What is the faction ceiling, and how is per-faction visibility
budgeted?**
Source: `04-selector-engine-and-verbs.md` open question 6;
`06-algorithms-and-scheduling.md` open question 5.
*Blocks:* D48 and milestone 5. **Note that report 04's figure of 2.1 MB per
faction is the bitplane only.** The full cost is 21.0 MB per faction: two
bitplanes at 2.1 MB and a `u8` counter at 16.8 MB. That is 168 MB at eight
factions, which is fine, and 4.2 GB at two hundred, which is not. Decide the
ceiling now, because it changes the representation — fog only for factions
with an observer, or fog shared within an alliance.

### Design questions with a proposed answer

**OQ6. When is a selector passed as a verb parameter evaluated?**
Source: `04-selector-engine-and-verbs.md` section 2.6 and open question 1.
*Blocks:* any verb that takes a selector parameter, and the scheduler.
**Report 04 flags this as a determinism hole.** D32 closes it with snapshot
semantics. Confirm the decision explicitly; it is listed here because it
must be an active choice, not an inherited default.

**OQ7. Is the sim tick 10 Hz or 30 Hz?**
Source: `06-algorithms-and-scheduling.md` open question 7.
*Blocks:* the whole per-tick budget. D29 proposes 10 Hz. It should be an
explicit decision, not an emergent one.

**OQ8. What is the capability bit budget, and who allocates the bits?**
Source: `04-selector-engine-and-verbs.md` open question 4.
*Blocks:* D40. 64 bits is enough only if allocation is disciplined.
Recommend a central registry with a compile-time check.

**OQ9. Does a unit ever occupy more than one tile?**
Source: `01-ecs-and-memory-layout.md` open question 7.
*Blocks:* D15. The occupancy design assumes one tile per unit. Multi-tile
structures break it. Related: does the design have unit stacks?
`split`/`merge` is in report 04's verb list, and retrofitting stacks is
expensive.

**OQ10. Is `WorldBatch` in scope for version 1?**
Source: `05-rust-python-boundary.md` open question 6.
*Blocks:* the `World` API shape. It is the highest-value feature for
audience 3 and it constrains `World`, so it must be decided early rather
than retrofitted.

### Questions that need a measurement

**OQ11. What is the span length in D13?**
Source: brief decision 2 (16 KiB); `01-ecs-and-memory-layout.md`
recommendation 5 (64 KiB) and open question 2.
*Blocks:* nothing structural, because D13 makes it a compile-time constant
over a contiguous column rather than an allocation size. Default 4096
entities. **Neither the brief nor report 01 has the benchmark.** Write the
iteration benchmark in milestone 1 and sweep it.

**OQ12. Are sub-chunk dirty masks needed?**
Source: `02-hex-grid-and-lod-pyramid.md` open question 4;
`01-ecs-and-memory-layout.md` open question 3.
*Blocks:* nothing. D19 says do not build them first. Build them only if
profiling shows cell recompute is hot. The branch and mask logic may cost
more than the reduction it saves. Related: does anything need per-entity
change detection at all, or does the dirty pyramid already cover every
projection?

**OQ13. How many distinct sight radii?**
Source: `06-algorithms-and-scheduling.md` open question 4.
*Blocks:* the field-of-view budget in D47, which is one of the two largest
line items in the per-tick table. Fewer values means much cheaper sharing.
Is four acceptable to the game design?

**OQ14. Does the arena get re-sorted by spatial key each tick?**
Source: `06-algorithms-and-scheduling.md` open question 6.
*Blocks:* nothing immediately, but it interacts with determinism because it
changes iteration order. It improves nearly every pass and costs one sort
per tick. Decide it with a measurement and a determinism test, together.

**OQ15. Which fields are min/max?**
Source: `02-hex-grid-and-lod-pyramid.md` open question 5.
*Blocks:* the extremum-count fast path in D16. Every min/max field carries a
possible rescan. Check whether each field's value distribution makes the
count effective, or whether a bucketed histogram serves the real query
better.

**OQ16. Does the purpose-built mask beat general Roaring?**
Source: `04-selector-engine-and-verbs.md` section 3.2, which states its own
caveat.
*Blocks:* D34. It is a design argument, not a measurement. Benchmark an
intersection of two 1-percent-dense sets, an intersection of two
40-percent-dense sets, a union of ten sets, and a full iteration with a
column read.

### Smaller questions, recorded so they are not lost

- **Are all boolean tile attributes known at compile time?** If mods add
  planes at run time, `BitPlanes<N>` with a const generic is the wrong API.
  (`01` open question 5)
- **What is the event budget per tick?** It sets the arena preallocation and
  the overflow policy. (`03` open question 5)
- **Which transcendental functions does the simulation actually need?**
  `sqrt` is IEEE-exact and therefore free. `atan2` and `sin` are not.
  Knowing the list decides how much fixed-point maths must be written.
  (`03` open question 6)
- **Should the engine detect and reject a changed floating-point control
  register set by another library in the process?** A warning is cheap.
  Silence is a very confusing bug. (`03` open question 7)
- **Is `.explain()` output part of the tested interface?** It will be in
  practice, because people will assert on it. Decide whether to give it a
  stable machine-readable form beside the human-readable one. (`04` open
  question 7)
- **Is the canonical iteration order part of the public contract?**
  Recommend "stable within a released version, not across versions".
  Anything stronger freezes the arena allocator. (`04` open question 2)
- **What is the minimum Python version?** D-level packaging assumes 3.11.
  Confirm against the research audience's real floor, which is sometimes
  older than expected in academic environments. (`05` open question 5)
- **What is the target rollback window, if any?** It sets the snapshot ring
  size and its memory budget. Nothing in version 1 depends on it, but the
  answer decides whether the ring is built in milestone 6 or never. (`03`
  open question 2)
