# ADR-0004: Level 0 is the only truth and every summary derives from it exactly

**Status:** Draft
**Date:** 2026-08-30
**Depends on:** ADR-0001, determinism as the primary constraint. ADR-0003,
storage.

## Context

Cachette simulates a hex world of about 16.7 million tiles and about one
million units. The engine must answer questions at three scales. A player
inspects one tile. An artificial player reads a city-scale summary. A
diplomacy system reads a region-scale summary. A solver runs on a coarse
grid because a fine grid is too expensive.

One structure serves all four. It is a mipmap-style pyramid over the tile
grid. Level 0 holds the tiles. Level 1 summarises a block of tiles. Level 2
summarises a block of level 1 cells.

This record fixes what the pyramid is, what it may hold, how it updates, and
what it may not be used for.

The record contains no cost table and no byte budget. Both change with every
measurement, so the registers hold them.[^1]

Three properties of the design force most of the decisions below.

**Determinism.** An aggregate combines partial results in an order that the
thread schedule chooses. The combine must therefore give one answer for
every order.[^2]

**Write rate.** The simulation writes tiles and moves units on every tick. A
read-optimal structure that a single write destroys is not usable. The
project rejected the summed-area table for this reason. One tile write
dirties a whole quadrant of a summed-area table. The same write costs two
cell updates in a pyramid.[^3]

**Scale.** Level 0 fits in memory with room to spare. The project therefore
never has to discard detail. This removes a whole class of problem that the
original design treated as its largest risk.[^4]

## Decision

### ADR-0004 D1 — Level 0 is the only source of truth

Level 0 holds the state. Level 1 and level 2 hold projections of level 0.

A projection is derived, incrementally maintained, and disposable. The engine
can delete every level 1 and level 2 cell and rebuild them from level 0. The
rebuild gives byte-identical results, because the combine operation is exact
and order-free.

Nothing writes level 1 or level 2 except the pyramid update. No system reads
a summary and then writes it back. A summary is never an input to its own
next value.

**This is a read-model relationship, not three simulations.** The engine does
not simulate a city at level 1 and a region at level 2. It simulates tiles
and units, and it summarises them.

The consequence is a test. Rebuild the whole pyramid from level 0 and compare
against the incrementally maintained pyramid. Any difference is a defect in
the delta path. Run this test in the golden-state harness.[^2]

### ADR-0004 D2 — The fanout is 16 at each level

A level 1 cell summarises a block of 16 by 16 tiles. A level 2 cell
summarises a block of 16 by 16 level 1 cells. The storage chunk is the level
1 block, so one chunk holds 256 tiles.

Three reasons fix the number at 16.

**A power of two makes the parent index a shift.** The parent of a cell is
its index shifted right by four bits on each axis. A fanout of 7 needs base-7
digit arithmetic, a child index table, and block sizes that never align to a
cache line.[^3]

**A fanout of 32 destroys level 2.** With a fanout of 32 at each level, level
2 holds a few tens of cells. That is a set of quadrants, not a region map. A
fanout of 16 gives a level 2 grid that a diplomacy system and a multigrid
solver can both use.

**A 256-element run is still a good reduction.** One field of one chunk is
256 contiguous elements. That is one memory stream and a short vector
reduction. A larger chunk does not reduce faster.

The pyramid overhead above level 0 is one cell for every 255 tiles at level
1, and one cell for every 65,535 tiles at level 2. The cell count is not the
cost. **The summary width is the cost.** Every field added to the summary
struct multiplies by the level 1 cell count. Declare the summary struct
explicitly and review its width.

The exact cell counts follow from the world extent, which is not yet
decided.[^5] Write the level counts as functions of the extent. They resolve
when the extent resolves.

Hexagons do not tile into larger hexagons. Every hex hierarchy therefore
makes a compromise. This project blocks on a power-of-two block in the
storage index space. The block is a near-rectangle in world space with a
staircase edge. The aggregate is defined over the index set, not over a drawn
polygon, so the staircase costs nothing.

Aperture-7 nesting was rejected. **It was not rejected because it aggregates
inexactly.** It aggregates exactly over its logical child set, and the
project's earlier claim to the contrary was wrong.[^6] It is rejected for
non-power-of-two index arithmetic, for absent cache alignment, and for
non-contiguous children.

### ADR-0004 D3 — The aggregation invariant

**A summary field must combine by an exactly associative operation.**

"Exactly" is the load-bearing word. Float addition is associative in
mathematics and is not associative in a machine. A pyramid built on float
sums drifts away from level 0 as the recombination order varies with which
blocks are dirty. The drift is silent and slow. Float sums are therefore
excluded from the pyramid.[^7] All accumulators are integer or fixed
point.[^2]

**An exactly associative combine with an identity is enough to BUILD the
pyramid. It is not enough to UPDATE it.** Incremental update needs an
inverse. A structure with an inverse is a group, not a monoid.[^8]

The reason is direct. A cell holds a minimum of 3. A child changes from 3 to
9. The cell cannot tell from its own value whether 3 is still the minimum. It
must read all its children again.

The database literature calls this the maintenance condition for a
materialised view. The project derived it by hand.[^9]

Two conversions rescue the cases the project needs. Both turn a
minimum-and-maximum-style aggregate into a group.

**Conversion 1. Store an extremum with a count of children at the
extremum.** The cell holds the minimum and the number of children that equal
it. A child change applies four rules. A new value below the extremum
replaces it and sets the count to one. A new value equal to the extremum
raises the count. A change that touches neither the old nor the new extremum
changes nothing. A child leaving the extremum lowers the count.
**The cell reads its children again only when the count reaches zero.**

The maximum uses the mirror rule. Combining two cells upward takes the
extremum and sums the counts on a tie.

Know the value distribution of the field before you use this. Data with many
equal values keeps the count well above one, and the rescan almost never
fires. Data with a fine continuous distribution keeps the count at one, and
the rescan fires often. A field of the second kind should be a bucketed
histogram instead.

**Conversion 2. Store a count for each bit, not a bare OR mask.** A mask is a
semilattice and has no inverse. A vector of per-bit counts is a histogram,
and a histogram is a group. Increment and decrement update it exactly.

The mask is derived at read. A bit is set in the OR mask when its count is
above zero. A bit is set in the AND mask when its count equals the child
count. **One structure therefore yields the OR mask, the AND mask, and the
exact population, and all three are delta-updatable.** Decision ADR-0004 D8
needs both masks, so this conversion pays for itself twice.

The cost is a vector of counts in place of one integer. Pay it. It converts
the most common categorical aggregate in the engine from monoid to group.

An aggregate that no conversion rescues is not a summary field. Median and
top-K are not associative at all. A dominant value is derived from a
histogram at read, never stored as a dominant value.

### ADR-0004 D4 — Every summary field is extensive or intensive, declared at registration

**An extensive quantity is held in the cell. It sums.** Stored wood,
population, and per-tick production are extensive. The sum over a region is
meaningful.

**An intensive quantity is a density, a potential, a rate per unit, or a
ratio. It does not sum.** Price, temperature, morale, and threat are
intensive. The sum over a region is meaningless. Only the average is
meaningful.

An intensive field aggregates as a pair of extensive accumulators: the sum of
value multiplied by weight, and the sum of weight. **The engine divides at
read, never at write.** Both accumulators are exactly associative sums, so
both are groups, and both take a signed delta.

The weight is the field's own weighting rule. A tile count is the default. A
population-weighted mean price uses population as the weight.

**Make this a typing rule.** Every summary field declares its kind when it
registers. The declaration decides two things that the programmer then cannot
choose: the stored representation, and the combine operation. A field
registry generates both from the declaration.

The rule for a designer is one question. Ask what the sum over a region
means. If the sum is meaningful, the field is extensive. If only the average
is meaningful, the field is intensive. If neither is meaningful, the quantity
is not a field and does not belong in the pyramid.[^10]

Two further rules follow from the kinds. Two fields combine only at the same
level and the same kind. Adding an extensive field to an intensive field is a
category error and the type system rejects it.[^10]

### ADR-0004 D5 — Two update paths, chosen by a measured threshold

**Path A, the delta path.** Sums, counts, histograms, per-bit counts, and
both accumulators of an intensive field take a signed delta. The write site
already holds the old value and the new value. It applies the delta to the
level 1 cell and the level 2 cell directly. The cost is two cell updates for
each changed tile. The path reads no block.

**Path B, the recompute path.** An extremum whose count reached zero, and
anything else that failed the fast path, marks its chunk. At the frame
barrier a worker reads all children of the chunk and folds them. The fold is
one contiguous vector reduction.

**Choose between them by the changed-tile count in the chunk.** Path A wins
when few tiles in a chunk changed. Path B wins when many did, because the
delta work then exceeds the cost of one contiguous scan. The crossover is a
fraction of the chunk size. Count the changed tiles while the writes queue.
Do not compute the count afterwards.

The threshold is a measured number. It belongs in the register, not in this
record.[^1] No measurement exists on the target platform yet.[^5]

### ADR-0004 D6 — Dirty tracking is per chunk, never per tile

A dirty bit for each tile is a large array with a handful of set bits.
Scanning it costs a fraction of the frame budget and yields nothing that a
coarser structure does not yield.

**Track dirtiness per chunk.** One bit for each level 1 cell, and one bit for
each level 2 cell. A flat bitset over the chunks is a few kilobytes at the
target scale. It stays resident in the first-level cache. An exhaustive scan
is a few hundred word loads, and a zero word is skipped by one compare.

This removes the need for a hierarchical bitset, a sparse bitset, or a
compressed bitmap. Those structures solve a scan cost that this project does
not have.

**Mark in parallel by atomic OR.** Marking is idempotent and order-free, so
many workers may mark the same word with no coordination. ADR-0001 permits
this: bitwise OR is exactly commutative and associative, so a scatter-or
gives one answer under any thread order.[^2] Contention is low, because a
chunk is marked once for each dirty chunk, not once for each changed tile.

**Drain in ascending index order.** The drain clears the bitset and produces
the set indices in ascending order. The work set is then identical on every
run, whatever the schedule. Never use completion order.

The update then runs level by level. Drain the level, process the drained
cells in parallel, and mark each parent. Writes are disjoint, one cell for
each worker, so the pass needs no lock. Align the summary struct to a cache
line so that disjoint writes do not share one.

Level 0 is not in this loop. A tile write marks its chunk directly, from the
step that applies the command.

Sub-chunk masks, which would let a recompute skip untouched tiles, are not
built. The branch and mask logic may cost more than the reduction it saves.
Build them only if a measurement shows the recompute path is hot.

### ADR-0004 D7 — Two pyramids, not one

Terrain changes rarely. Unit positions change every frame. One pyramid over
both dirties nearly every chunk on every tick, and every terrain aggregate is
then recomputed for nothing.

**Build a terrain pyramid and a unit pyramid over the same cell grid.** They
share the index arithmetic, the dirty machinery, and the descent code. The
extra code is small. The cadence is what differs.

The terrain pyramid updates when terrain changes. Most ticks touch few
chunks.

**The unit pyramid is delta-only.** Every field in it is a count, a
histogram, or a per-bit count. No field in it is an extremum. It therefore
never enters the recompute path, and a frame of unit motion costs two cell
updates for each unit that crossed a chunk boundary. Conversion 2 of ADR-0004
D3 is what makes this possible, because the faction presence mask is the
field a unit pyramid most needs.

A field that would force a recompute is not admitted to the unit pyramid. If
a consumer needs one, it reads the terrain pyramid or it reads level 0.

### ADR-0004 D8 — The pyramid is the query index

A selector filters tiles by a predicate. The engine evaluates the predicate
against a cell summary and returns one of three verdicts.

- **None.** No tile in this subtree matches. Prune it. No further work.
- **All.** Every tile in this subtree matches. Emit the cell's tile range and
  do not descend.
- **Some.** Descend to the children. At level 0, test each tile.

The test must be conservative. It may return Some when the true answer is
None or All. It must never return None or All when the true answer is Some.

Descent visits children in fixed index order, so the output order does not
depend on the schedule.[^2]

**For every field a selector can filter on, the summary stores a lower bound
AND an upper bound.** Numbers store a minimum and a maximum. Categories store
an OR mask and an AND mask, which conversion 2 of ADR-0004 D3 supplies from
one structure.

One bound alone gives the None verdict and never the All verdict. **Accept is
the larger win, and most designs forget it.** A pruned subtree costs nothing
to reject. An accepted subtree replaces a full scan of its tiles with one
range.

The All verdict is only a win if the selector evaluation can return a tile
range without materialising the tile list. **The selector API must therefore
have a range result.** Without it the AND masks are wasted memory. This
constrains the selector record.[^11]

### ADR-0004 D9 — The pyramid is the statistics catalogue

A query planner needs selectivity: the fraction of rows a predicate keeps. A
database estimates it from a sampled histogram and is often wrong.

**A histogram summary field gives an exact count for an equality
predicate.** The histogram is maintained, not sampled. It covers every child,
not a sample of them. The count of tiles in a cell with terrain value `t` is
read directly.

**Single-predicate selectivity is therefore exact, not estimated.** The
planner does not guess how many tiles match one equality test. It reads the
number.

Selectivity for a conjunction of predicates is not exact. The pyramid holds a
histogram for each field, not a joint distribution over fields. The planner
must assume independence across fields, and that assumption can be wrong.
State it. Do not present a conjunction estimate as exact.

The same catalogue serves the interface directly. A region panel reads its
numbers from the level 1 summary. It does not scan tiles.

### ADR-0004 D10 — Descent has a cost model and a flat fallback

Hierarchical descent helps when the matching tiles are clustered. **It hurts
when they are scattered.**

If a predicate matches a small fraction of tiles spread uniformly, nearly
every cell returns Some. The engine then pays the summary reads and the
descent bookkeeping on top of a full scan of level 0. It is slower than a
flat scan.

**The fallback.** Evaluate the predicate at level 2 first. Count the Some
verdicts. If the fraction of Some verdicts exceeds a threshold, abandon the
descent and run a flat vector scan over the level 0 arrays.

The flat scan is a sequential pass over contiguous arrays. Its cost is
bounded and predictable. It is the worst case the query path must not exceed.

Two rules follow.

**Build the flat path first, and the descent second.** The flat path is the
correctness reference. The descent is an optimisation measured against it. A
test compares the two paths on the same predicate and requires identical
output.

**The threshold is a measured number.** It belongs in the register.[^1] Start
it at half and correct it from measurement on the target platform.[^5]

### ADR-0004 D11 — Operators do not commute with aggregation

**Fields aggregate. Dynamics do not.**

A summary of a field is exact. A summary of the result of a solver is not the
result of the solver on the summary. Restriction followed by diffusion is not
diffusion followed by restriction. The two operators do not commute, and no
choice of coefficients makes them commute.[^12]

This is the most likely conceptual error in the engine. A programmer sees a
correct summary pyramid and concludes that solving at level 1 gives the level
1 projection of solving at level 0. It does not.

**The criterion is the length scale of the feature.** A smooth field
tolerates a coarse solve, because the field varies little inside one cell,
and the aggregation error is second order in the cell width. A sharp feature
does not. A coarse solve smears a barrier, a front, or a chokepoint across
the whole cell that holds it. **If the answer depends on a feature narrower
than a cell, solve at the level where the feature is resolved.**

Two consequences follow for the fields the engine runs.

A potential field that a consumer only compares or ranks may be solved
coarsely. A partially relaxed field preserves the order of two well-separated
cells.

A field whose whole purpose is a barrier must be solved at a level that
resolves the barrier. A defended mountain pass is the point of a chokepoint,
and a coarse solve deletes it.

**The multigrid requirement.** A coarse solve is still useful as an
accelerator. The engine restricts a problem to level 2, solves it there,
prolongs the result back to level 1, and runs a fixed number of correcting
iterations at level 1.[^13] The iteration count is fixed at compile time or
from content, never from a convergence test.[^2]

**Restriction and prolongation must be adjoint.** If they are not, the pair
creates or destroys quantity at every level boundary, on every cycle, and the
error accumulates. Restriction of an extensive field is a sum over the
children. The adjoint of a sum is a broadcast that divides the parent value
by the child count. An intensive field restricts by the weighted mean of
ADR-0004 D4 and prolongs by copying, which is the adjoint pair for that
kind.[^12]

**The integer rounding policy keeps the total exact.** A division by the
child count truncates, and the truncated remainder is quantity that would
otherwise vanish. **Prolongation of an extensive field divides the parent
value by the child count and gives the whole remainder to the
lowest-indexed child.** The children then sum back to the parent exactly. The
choice of the lowest-indexed child is arbitrary and is fixed for that reason:
it is a stated rule, not a schedule outcome, so it is deterministic.[^2]

A test protects this. Restrict a field and prolong it back. The total must be
unchanged, exactly, for every input.

Never apply a clamp to a conserved field at any level. A clamp is not linear
and it deletes or invents quantity silently.[^12]

### ADR-0004 D12 — There is no promotion and no demotion problem

**The engine never discards level 0 detail.** It never materialises invented
level 0 detail from a summary.

The project once believed that generating plausible tile detail from a level
1 summary, when a player zooms in, was the hardest part of the design. That
belief conflated two different things: freezing computation, and discarding
data.[^4]

Level 0 fits in memory at the target scale. Nothing forces the engine to
throw a tile away. **Freeze the processing. Keep the data.**

What the project called coarse background simulation is active-set
simulation. A region outside the active set stops being stepped. Its tiles
remain exactly as they were. When it re-enters the active set, it resumes
from stored state. Nothing is invented, so nothing can be inconsistent.

**Record this so it is not proposed again.** Any future design that
materialises level 0 detail from a level 1 summary must first show that level
0 no longer fits, and that is a different record.

One related case is not the same thing and must not be confused with it. A
field that is a pure function of current sources, such as an influence plane,
is always reconstructible from those sources. Dropping and rebuilding it
loses nothing. A field that is a history, such as explored fog, is not
reconstructible, and dropping it loses information permanently.[^14] The rule
above concerns stored tile state, which is neither.

### ADR-0004 D13 — Hex geometry helps the pyramid's solvers and hurts its distances

Hex geometry is not uniformly better than square geometry. It cuts both
ways, and the two effects land on different users of the pyramid.[^15]

**Diffusion is better on hex.** The seven-point hex stencil has directional
error an order of magnitude below the best nine-point square stencil, with
two fewer taps and no penalty in the step limit. Every field solver that runs
on level 1 or level 2 gains from this.

**The path metric is worse on hex.** A six-connected lattice has a larger
worst-case path error than an eight-connected square grid.

The consequence for this record is a division of labour. **The pyramid's
field and summary users get the better geometry. Its distance users get the
worse one.** Movement and pathing therefore do not take a distance from the
pyramid's block geometry. They use the hex distance rule directly, or a flow
field. The pyramid supplies pruning bounds to a spatial query, not a metric
to a path search.

A bounding radius around a block admits false positives in proportion to the
block's anisotropy. A block that is near-square in world space prunes more
tightly than a block that is a sheared rhombus. This is a reason to block in
the offset index space when the world is a rectangle. If the world is a
rhombus by design, the raw axial block is exact and the conversion
disappears. The world shape is not yet decided.[^5]

## Consequences

### What this buys

One structure serves four users: the summary that an artificial player reads,
the index that a selector query descends, the catalogue that a query planner
reads, and the coarse grid that a multigrid solver uses. The engine builds
one thing and maintains one dirty set.

A rebuild test that catches every defect in the delta path, because the
pyramid is exactly reproducible from level 0.

Exact single-predicate selectivity, which no sampled statistics catalogue
gives.

An update cost proportional to what changed, not to the world size.

A solver accelerator with no new machinery. The level 2 grid, the restriction
path, and the dirty bitsets all exist for the summary pyramid.

### What this costs

Every summary field must be classified and registered. A field that is
neither extensive nor intensive cannot be summarised, and the designer must
be told so rather than served an incorrect number.

Group conversions cost storage. A per-bit count vector is larger than a mask.
An extremum with a count is larger than a bare extremum. Both are paid at
every level 1 cell, so the summary width grows.

An extremum field over finely distributed data keeps the recompute path hot.
Each such field needs its distribution checked before it is admitted.

Two pyramids are two update passes and two dirty sets.

The query path is two implementations, flat and hierarchical, and a test that
holds them equal.

Two thresholds and one summary width are measured numbers with no measurement
yet on the target platform.

### What this forecloses

Summed-area tables and any prefix-sum structure. One write invalidates a
quadrant, and this workload writes constantly.

Any aggregate that is not exactly associative. Median, percentile, and top-K
are excluded as stored summaries. Approximate them from a bucketed histogram
or do not offer them.

Floating-point summaries at every level.

Interoperation with a geospatial index. Power-of-two blocks in an index space
have no geographic reference frame. A future need to publish cell identifiers
that a geospatial tool understands is a rewrite of this record.

A joint distribution across fields, and therefore an exact conjunction
estimate.

## Notes

The dirty pyramid is a materialised view, and the group-with-inverse rule is
the standard maintenance condition from the incremental view maintenance
literature. The project derived a known theorem by hand.[^9] Read that
literature before extending the update path.

A Fenwick tree answers a sum over an arbitrary rectangle. This pyramid
answers a sum over a fixed cell in one read. The queries differ, so the
structures are not alternatives. If a feature later needs frequent arbitrary
rectangle sums, add a Fenwick tree beside the pyramid. Do not replace the
pyramid with one.

## References

[^1]: ADR registry, the section on what does not belong in a record, and the register table. `docs/adrs/REGISTRY.md`
[^2]: ADR-0001, Determinism as the primary constraint, decisions D2, D4, D6, D7, D8 and D11. `docs/adrs/draft/adr-0001-determinism-outranks-every-other-constraint.md`
[^3]: Research report 02, Hex grid representation, LOD aggregation and spatial indexing, sections 3 and 4. `docs/research/reports/02-hex-grid-and-lod-pyramid.md`
[^4]: Findings register, entry FND-007. `docs/FINDINGS.md`
[^5]: Blockers register, entries BLK-001 and BLK-007. `docs/BLOCKERS.md`
[^6]: Findings register, entry FND-008. `docs/FINDINGS.md`
[^7]: Findings register, entry FND-001. `docs/FINDINGS.md`
[^8]: Findings register, entry FND-002. `docs/FINDINGS.md`
[^9]: Research reports merge notes, the section on adjacent fields the project reinvented. `docs/research/reports/MERGE-NOTES.md`
[^10]: Research report 13, the field operator algebra, sections 5.3 and 5.5. `docs/research/reports/13-field-operator-algebra.md`
[^11]: ADR-0010, Selectors and verbs. `docs/adrs/draft/`
[^12]: Research report 13, the field operator algebra, sections 5.2, 5.3 and 5.4. `docs/research/reports/13-field-operator-algebra.md`
[^13]: Research report 09, influence maps, sections 6.1 and 6.3. `docs/research/reports/09-influence-maps.md`
[^14]: Research report 09, influence maps, section 8.5. `docs/research/reports/09-influence-maps.md`
[^15]: Findings register, entry FND-025. `docs/FINDINGS.md`
