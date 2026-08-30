# ADR-0003: Tiles are dense columns and units are a generational arena

**Status:** Draft
**Date:** 2026-08-30
**Depends on:** ADR-0001, ADR-0002

## Context

Cachette holds two populations. One is the tile grid, at about 16.7 million
tiles. The other is the mobile and built population, at about one million
units and a smaller number of structures.

These two populations have almost nothing in common. Every tile exists.
Every tile has the same fields. A tile never moves, never dies, and never
changes its identity. A unit does all three. A tile is addressed by its
position. A unit is addressed by a handle that must stay valid after other
units die.

One storage design cannot serve both populations well. A design that gives
each tile an identity pays for 16.7 million handles that nothing uses. A
design that addresses units by position cannot answer a question about a
unit that moved.

The project must also obey the determinism rule.[^1] That rule bans thread
completion order, work-stealing order, and hash-map iteration order from
simulation code. Storage is where those orders would otherwise enter,
because storage decides the order in which a system sees its data. Storage
is therefore a determinism decision before it is a performance decision.

Three research reports examined this area.[^2][^3][^4] They agree on the
split. They disagree on one structure, and this record settles that
disagreement.

The engine also has a hard memory shape. The target platform uses a 64-byte
cache line and a limited translation lookaside buffer.[^5] At 16.7 million
tiles, one extra byte for each tile costs 16 MiB of memory, and it costs
16 MiB of memory traffic in every full-grid pass. Field width is therefore a
budget item, not a matter of taste.

## Decision

Decision numbers in this record are local to this record. Cite them as
`ADR-0003 D1`, and so on.

### ADR-0003 D1 — Two storage regimes, not one

Tiles use dense struct-of-arrays storage indexed by position. **A tile is
not an entity.** A tile has no handle, no generation, and no row in any
entity table. A system reads a tile field by computing an index from a
coordinate.

Units and structures live in a generational arena. Each one has a handle.
The handle stays meaningful after any other unit dies.

Nothing bridges the two regimes by identity. The bridge is spatial, and
ADR-0003 D8 defines it.

This split follows from the access pattern. A tile pass reads every tile in
a known order, so an index is enough and a handle is waste. A unit pass
follows references between units that die at different times, so a handle is
needed.

### ADR-0003 D2 — The project writes its own entity storage

The engine takes no dependency on a general entity component system crate.
The project writes the storage layer, at an estimated 2,000 lines.

Three reports reached this conclusion independently.[^2][^3][^4] Five
requirements drive it, and no available crate meets them.

**Apply order.** Determinism requires a declared order for every structural
change.[^1] A general command buffer does not promise one.

**Schedule control.** Determinism requires a declared schedule. A general
scheduler chooses its own order.

**Per-chunk metadata.** Query pruning needs a faction mask, a bounding box,
and a type histogram at chunk granularity. No crate exposes a hook for
user metadata at that granularity.

**Per-chunk change ticks.** ADR-0003 D10 requires them. The most mature
crate hard-codes per-entity ticks, and its own maintainers record the cost
as a defect.[^2]

**Stable raw column pointers.** The Python boundary needs a column address
with a stable layout.[^6] The crates that hold columns contiguously keep
that type private and give no layout guarantee.

The counter-argument is fair and the project accepts it. Writing entity
storage is a known way to spend months and ship nothing. The scope stays
small because the requirements are narrow. Write the iteration benchmark
first, so the project knows when to stop tuning.

### ADR-0003 D3 — Whether archetype machinery is needed is blocked

An archetype is the exact set of component types that an entity carries. It
is not a category of unit.

This record cannot decide whether the engine needs archetype machinery. The
decision needs one piece of information the project does not have: three
archetypes that the owner expects to exist. That question is open.[^7]

Record the conditional, and do not guess the answer.

**If one shape exists.** The storage is a generational struct-of-arrays
arena. It is a set of parallel columns plus a generational free list. There
is no archetype graph, no table move, and no chunk. The archetype machinery
is dead weight, and the project must not build it. Name the structure an
arena, not an entity component system, because the honest name saves the
code.

**If two to four fixed shapes exist.** Each shape gets its own set of
columns. Structural change becomes a move between column sets, so ADR-0003
D11 applies and the archetype graph earns its place. Chunking then earns its
place as well, because a chunk is where the per-chunk metadata of ADR-0003
D2 hangs. The chunk size becomes a compile-time constant that the project
measures; the reports recommend a starting value larger than the common
16 KiB figure, for prefetch and translation-lookaside-buffer reasons.[^2]

**If the shapes vary at run time.** This case is out of scope. Unit types
are data and upgrades are a bitmask, so composition does not vary with
content.

Everything else in this record holds under either answer. The blocked part
is bounded, and it is about 2,000 lines of code and the shape of the Python
column view.[^7]

### ADR-0003 D4 — Entity identity is an index plus a generation in eight bytes

A handle holds a `NonMaxU32` index and a `u32` generation, packed in a
`NonZeroU64`. The non-maximum encoding puts the niche in the index, so the
generation may hold any value, and `Option<Entity>` is 8 bytes. Assert that
size in a test.

The generation increments when the slot becomes free, not when the engine
allocates the slot. A stale handle therefore becomes invalid at the moment
the entity dies.

Free slots recycle first-in-first-out. A last-in-first-out free list returns
the same slot inside the same frame. A handle that a command buffer captured
before the removal then matches a different entity, because the generation
advanced only once. First-in-first-out recycling removes that case.

A slot retires on generation overflow. The engine never returns it to the
free list. Retirement leaks four bytes for each retired slot, and only in a
pathological run. This removes the reuse hazard from the design at a price
the project can state.

The exposure is internal. Python never sees a handle.[^6] The holders that
cross the frame barrier are the command buffer and the spatial bridge, and
this rule covers both.

### ADR-0003 D5 — Tile columns are narrow, with bitplanes and sparse side tables

Each tile field is one aligned array. Three rules govern what a field may
be.

**Narrow types.** A tile field uses the narrowest integer that holds its
range. Most are one byte. Elevation is two bytes. Every extra byte for each
tile costs memory traffic in every full-grid pass, so each field must
justify its width. The register holds the totals.[^8]

**Bitplanes for booleans.** A boolean attribute gets its own plane of `u64`
words. The engine does not pack several booleans into one byte for each
tile. A separate plane gives three properties. A combined query is a bitwise
operation followed by a population count over the words of one block, not a
loop over the tiles. A set operation is one word operation for each 64
tiles. The population count is a sum, so it is an exact monoid and it
satisfies the aggregation rule.[^1] The dirty bitset is already a plane, so
this is one code path and not two.

**Sparse side tables for rare data.** Data that few tiles carry does not get
a column. It gets a bitplane index plus a keyed payload. The bitplane is the
index, and the payload map serves only a point lookup. Every bulk query
reads the plane and never touches the map. If the present fraction grows
large enough to hurt, replace the map with a rank-select structure over the
plane. Build that only on a measurement. The upgrade fraction is not yet
known.[^9]

### ADR-0003 D6 — Tile storage uses block-tiled order at the aggregation block size

Tile columns and bitplanes are stored in blocks, at the same block size that
the summary pyramid aggregates over. Inside a block the order is row-major.
The block edge is a power of two.

One aggregation step then reads exactly one contiguous span of one field.
Under plain row-major order the same step touches one span for each row of
the block, spread across the width of the world, on separate pages. Block
tiling turns many prefetch streams and many translation entries into one.

Index arithmetic stays a shift, a mask, and an add, because the block edge
and the world width are powers of two. There is no division.

Block tiling also makes the parallel split safe. A block of tiles is a whole
number of `u64` words in every bitplane. Two workers therefore never
read-modify-write the same word. That is a correctness property, not a
speed property: a shared word loses one update and produces a wrong
answer.[^2] Splitting parallel work at block granularity removes the hazard
by construction, so **the block is the unit of parallelism everywhere.**

What this costs is a long scan across the whole world, which becomes
strided. The engine has no such query. A future one walks block rows.

### ADR-0003 D7 — Tiles are indexed by odd-r offset, not by raw axial

The logical coordinate stays axial, and geometry derives from it.[^3] The
**array index** is an odd-r offset index. The conversion is one shift and
one add in each direction.

Raw axial indexing is rejected for two reasons.

**Waste.** A rectangular world stored at a raw axial index needs a bounding
parallelogram. The shear moves each row start by half a row, so over the
height of the world the array wastes about half of its cells. At the target
scale that waste is larger than several whole tile fields.

**Loose pruning.** A power-of-two block in raw axial space is a 60-degree
rhombus in world space, with an aspect ratio of about 1.73 to 1. A
conservative bounding radius around such a block admits many false
positives, so a radius query descends into subtrees it does not need. The
same block in offset space is a near-rectangle at about 1.15 to 1, and a
rectangular viewport maps to a rectangular block range.

The offset block has a staircase edge in world space. This costs nothing.
An aggregate is defined over the index set, not over a polygon, so no tile
is missed and none is counted twice.

If the owner chooses a rhombus world, raw axial storage becomes correct and
the conversion disappears. That choice is open, and it belongs to the world
extent question.[^10]

### ADR-0003 D8 — The unit-to-tile bridge is a sorted array, a per-block range, and a dense per-tile count

Two reports disagreed here, and this decision reconciles them.

The first report rejected a full-grid compressed-sparse-row occupancy index.
An offset array over every tile costs four bytes for each tile, which is
more than the whole minimum tile schema. Over 94 percent of those offsets
repeat their neighbour. Its replacement keeps the unit array sorted by tile
index and stores one start and one length for each block.[^2]

A later report found a defect in that replacement. A separation term reads
the occupancy of each of the six neighbours of a tile. Under a block-level
bridge, each of those reads becomes a search inside a range that spans a
whole block, which is up to 256 entries at the relevant block size. Seven
searches for each unit does not fit any budget, so the separation term is
not achievable. Its fix is a dense count of one byte for each tile.[^4]

**Both are right, because they solve different problems.** The rejected
structure is a four-byte offset array whose purpose is to find *which* units
sit on a tile. The proposed structure is a one-byte count whose purpose is
to answer *how many*. The second is a quarter of the size of the first, and
it is the one the hot kernels need.

The bridge is therefore three structures.

1. **A unit array sorted by packed tile index.** It answers "iterate the
   units of a region in a deterministic order". ADR-0003 D9 maintains it.
2. **A start and a length for each block.** It answers "where does this
   block begin in that array". It costs a small fraction of the unit array.
3. **A dense occupancy count, one byte for each tile.** It answers "how many
   units are on this tile" in one load, with no search. The register holds
   its total.[^8]

The count array is not new storage. The owner already approved one byte for
each tile, and that array serves three uses at once: the occupancy count,
the capacity check before admitting a unit, and the density field that the
crowd kernels read.[^11] Tile capacity is eight units, and crossing terrain
raises it to sixteen, so a `u8` leaves large headroom for transient overflow
during a sort-then-admit pass.

Add a bitplane that marks which blocks hold any unit. A selector descent
then skips empty blocks with a population count.

Where a system needs the full per-tile list often enough that the search
inside a block hurts, build a per-block offset array on demand. It is small
enough to stay in the first-level cache, and it is discarded after use. Do
not build the full-grid form.

### ADR-0003 D9 — Units stay sorted by tile index. This is an invariant

**The unit arrays are ordered by packed tile index at every frame barrier.**
This is an invariant of the engine, at the same rank as the no-float rule.
It is not an optimisation, and no system may break it.

Much of the performance story depends on it. A spatial kernel becomes a
sequential scan rather than a random gather. A field read becomes a
sequential read of a summary plane that stays in cache with high reuse. One
cost estimate fell by two orders of magnitude when this property was applied
to it, and that single line was 92 percent of its subsystem.[^12] The
invariant is also what makes the bridge of ADR-0003 D8 affordable, because a
block range is contiguous only if the array is sorted.

The engine maintains the order incrementally. Only a unit that changed tile
moves. The engine collects those units at the barrier, sorts them with a
radix sort on the tile key, and merges them back. A radix sort has no
data-dependent comparison, so it is deterministic, and it satisfies the
total-order rule.[^1] Append the entity index as the final key so no tie
remains.

A full re-sort is the fallback for a bulk operation that moves a large
fraction of the population at once.

This structure is not novel. Molecular dynamics solves the same problem with
cell lists and periodic spatial reordering, under the same reproducibility
requirement.[^13] Read that literature before extending this design.

### ADR-0003 D10 — Change detection is per chunk, not per entity

The engine records one change tick for each column of each chunk. It does
not record a tick for each component of each entity.

Per-entity ticks cost tens of megabytes of write traffic in each frame at
the target scale, and the engine spends that bandwidth even when nothing
reads the result. Per-chunk ticks cost about four orders of magnitude
less.[^2]

A per-entity dirty bit is available as an opt-in for a single field, and the
component declares it. A worker owns a whole chunk, so it owns whole words
of the bit plane and needs no atomic operation. The default is no tracking
at all.

The dirty pyramid is the real change-detection system of this engine.[^1]
The tick system answers only the narrow question of which unit columns
changed. Do not build a second general mechanism that duplicates the
pyramid.

### ADR-0003 D11 — Structural change is batched at the barrier and applies by tombstone and compact

No system creates, removes, or reshapes an entity inside a step. Each
worker records the request in its own buffer. The engine applies every
request at the frame barrier.

The apply order is fixed.

1. Concatenate the per-worker buffers in a fixed worker order. Never use
   completion order.
2. Radix-sort the requests by source and destination. The sort is stable and
   deterministic.
3. Resolve the shared columns and the dropped columns once for each
   source-and-destination run, not once for each entity.
4. Move each run with one block copy for each column and each destination.
5. Compact the source.

Steps 3 and 4 are the reason to sort. Without the sort the engine repeats
the column intersection for every entity.

**Removal uses tombstone and compact, not swap-remove.** A swap-remove moves
the last row of a chunk into the gap. If a later request in the same batch
refers to that row, its recorded row number is now stale. Marking the
removed rows in a bitset and compacting each dirty chunk once at the end
avoids the case entirely. Compaction is a filtered copy, so it also
vectorises.

Rewrite the recorded row of every entity that physically moved.

Give bulk creation its own path. Creating many units of one shape appends
whole chunks and returns a contiguous handle range, so a verb fills the
columns with one copy for each column.

### ADR-0003 D12 — Layout follows the access pattern. This is a rule

**A pass that reads its data sequentially uses struct-of-arrays. A pass that
gathers its data at random uses array-of-structs.**

This is a rule, not a preference. It is not a matter of habit or of style.
The project once held that struct-of-arrays vectorises well and therefore
belongs everywhere. That is wrong. A random graph gather in struct-of-arrays
form touches one cache line for each field of each candidate. The same
gather in array-of-structs form touches one cache line for the whole
candidate. The measured difference is twelve times.[^14]

Both forms therefore exist in this engine, and each pass declares which one
it uses. Tile columns are struct-of-arrays, because every tile pass is
sequential. Unit columns are struct-of-arrays, because ADR-0003 D9 makes
every spatial kernel sequential. A record that a graph traversal visits at
random is array-of-structs.

Apply the rule the same way to periodic work. Stagger a periodic pass by a
mix of the summary cell index, never by the entity index. Staggering by
entity index scatters the active fraction through the whole array and costs
three to four times more.[^15]

### ADR-0003 D13 — State plainly what the Python view copies

**A whole tile field is a zero-copy view. A whole unit column is a zero-copy
view. A subset is always a copy.**

The project once believed every component array could be a zero-copy view.
That is false under a chunked layout, because a chunked layout has no flat
array for a component. One million units is thousands of chunks, and a view
needs one address and one stride.[^16]

Two consequences follow, and the project accepts both.

The zero-copy story for unit columns depends on ADR-0003 D3. One shape means
flat columns and whole-column zero copy. Several shapes mean one view for
each shape, or a copy.

The tile side is flat under either answer. A tile field is one contiguous
array, so it is the honest demonstration of the zero-copy path.

The method that returns a subset copies. Document that in the method, in the
reference material, and in the first tutorial. Do not describe the boundary
as zero-copy without the qualifier. A user who plans around a promise the
engine does not keep will find out at the worst time.

## Consequences

### What this buys

A tile pass that reads one contiguous span for each aggregation step, with
one prefetch stream and one translation entry where a row-major layout needs
many.

A spatial kernel that is a sequential scan rather than a random gather,
which is the single largest cost reduction in the research.

A handle that is eight bytes with or without an option wrapper, and that
becomes invalid at the moment its entity dies.

A parallel split that cannot lose a bitset update, because the block is a
whole number of words.

A change-detection scheme that costs kilobytes for each frame rather than
tens of megabytes.

A boolean query answered by a population count over words rather than a loop
over tiles, which is also an exact monoid and therefore legal in the
pyramid.

A storage layer the project controls, so the apply order and the schedule
are declared rather than inherited.

### What this costs

The project writes and maintains about 2,000 lines that a dependency would
otherwise supply. That code has no upstream, no other users, and no external
test suite.

Every system must respect the sorted-by-tile invariant. A system that writes
a position without recording the tile change breaks every kernel downstream,
and it breaks them silently until a state hash disagrees.

A long scan across the whole world is strided under block tiling.

The offset index adds one conversion between the logical coordinate and the
array index. It is two instructions, and it appears in every tile access
path.

Retiring a slot on generation overflow leaks a small amount of memory in a
pathological run.

Bulk movement forces a full re-sort, which is the worst case of ADR-0003 D9
and is much more expensive than the incremental path.

The narrow-field rule is friction on every schema change. Adding one byte to
the tile schema is a memory decision that needs a number, not a preference.

### What this forecloses

Any storage design that gives a tile an identity. Adding one later means
16.7 million handles and a rewrite of every tile pass.

Any system that reorders the unit arrays for its own purposes.

Any component whose composition varies for each entity at run time. Unit
types are data and upgrades are a bitmask, so this is already true, and this
record makes it structural.

Per-entity change detection as a general facility. It remains available for
one declared field at a time.

A promise of zero-copy access to an arbitrary subset from Python. That
promise cannot be kept, and the boundary must not imply it.

## Notes

Every cost figure that supports this record is derived, not measured. No
benchmark exists on the target platform, and the development machines differ
from it in cache line size.[^5] Treat each figure as a ranking of options,
not as a prediction. Build the iteration benchmark before tuning any
constant in this record.[^17]

## References

[^1]: ADR-0001, Determinism as the primary constraint, decisions D2, D6, D7 and D10. `docs/adrs/draft/adr-0001-determinism-outranks-every-other-constraint.md`
[^2]: Research report 01, ECS core, memory layout and cache-aligned data structures. `docs/research/reports/01-ecs-and-memory-layout.md`
[^3]: Research report 02, hex grid and level-of-detail pyramid. `docs/research/reports/02-hex-grid-and-lod-pyramid.md`
[^4]: Merge notes for the research reports, section 6, defects found in the superseded draft. `docs/research/reports/MERGE-NOTES.md`
[^5]: ADR-0002, Target platform and value types. `docs/adrs/draft/adr-0002-value-types-are-exact-and-sized-for-one-target.md`
[^6]: Findings register, entry FND-003. `docs/FINDINGS.md`
[^7]: Blockers register, entry BLK-002. `docs/BLOCKERS.md`
[^8]: Budgets register, cost and storage tables. `docs/reference/budgets.md`
[^9]: Blockers register, entry BLK-006. `docs/BLOCKERS.md`
[^10]: Blockers register, entry BLK-001. `docs/BLOCKERS.md`
[^11]: Blockers register, entry BLK-009, and merge notes section 2, owner decisions. `docs/BLOCKERS.md`
[^12]: Findings register, entry FND-017. `docs/FINDINGS.md`
[^13]: Merge notes, section 14, adjacent fields the project reinvented. `docs/research/reports/MERGE-NOTES.md`
[^14]: Findings register, entry FND-022. `docs/FINDINGS.md`
[^15]: Findings register, entry FND-023. `docs/FINDINGS.md`
[^16]: Findings register, entry FND-003, and research report 05, the Rust and Python boundary. `docs/FINDINGS.md`
[^17]: Blockers register, entry BLK-007. `docs/BLOCKERS.md`
