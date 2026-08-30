# Cachette Research Agenda — Index

- **Status:** Draft
- **Date:** 2026-08-30
- **Type:** Index. This document points at fields to study. It does not
  hold findings.

## Purpose

Cachette is a world simulation engine. Rust holds the data and does the
work. Python drives it. The engine simulates a hex world at three levels
of detail. Level 0 holds 16.7 million tiles and up to one million units.
Level 1 and level 2 hold derived summaries. One decision record holds 50
numbered decisions and 16 open questions. Eight background documents
support that record.[^1]

This document is a map of the research the project should do over the
next months. Each entry names a field, says why Cachette needs it, names
the seminal and the current work, names the software that exists, and
gives a priority. The entries are not findings. Depth comes later, one
topic at a time.

Three project properties shape every entry.

1. **Determinism is the highest-priority non-functional property.** The
   engine must produce the same bytes for one binary at any thread
   count. No floating point may enter simulated or aggregated state.[^2]
2. **Cost must scale with map area or with changed state.** Cost must
   not scale with entity count where the design can avoid it.
3. **Python is a control plane.** Python never loops over entities. It
   builds a lazy selector and sends one set-valued command.

Three audiences use the engine, in this order: the author who builds a
strategy game on it, other simulation developers, and researchers who
run reinforcement learning and agent-based models.

## How to read a priority label

| Label | Meaning |
|---|---|
| **Load-bearing** | A recorded decision is blocked or at risk without this study. |
| **High-value** | The study improves a decision that is already made, or removes a known risk. |
| **Speculative** | The study may pay off. No current decision depends on it. |

## Work already in flight

Four further background reports are in preparation. They cover influence
maps, crowd and movement, resource and trade flow, and entity economy
and modifiers. None of the four existed when this agenda was written.
Each entry below that overlaps one of the four says so. The owner of
this agenda must not duplicate that work.

---

## Area 1 — Data-oriented design, entity systems, and memory layout

### 1.1 Data-oriented design as a discipline

**What it is.** Data-oriented design treats the memory layout as the
primary design artefact. It selects structure-of-arrays storage, narrow
types, and linear access. It rejects object graphs and virtual dispatch
in hot loops.

**Why it matters to Cachette.** The record decides a generational
struct-of-arrays arena instead of a third-party entity component system.
It also decides globally contiguous columns with a logical span as the
query and parallelism unit. Both decisions rest on data-oriented
arguments that the project has not yet tested against a benchmark. The
record itself flags the span length as an open question that needs a
measurement.[^1]

**Key work.** Mike Acton's 2014 talk sets the position.[^3] Richard
Fabian's book is the long form.[^4] Unity DOTS, Flecs, and Bevy are the
three production designs that the reports compare.[^5][^6][^7] A 2025
academic comparison of archetype storage against sparse-set storage
gives measured numbers.[^8]

**Software.** `hecs` and `legion` are small archetype stores worth
reading.[^9] `bytemuck` and `zerocopy` give the plain-data bound.[^10]
`crossbeam-utils` gives cache-line padding.[^11] No crate gives what the
record wants, which is per-span user metadata plus a fixed column
layout. The project must write that.

**Priority.** Load-bearing.

**What it unblocks.** The span length and the arena benchmark. The
archetype question itself is now answered: entity storage holds four fixed
shapes, so the benchmark measures a chosen design rather than deciding
one.[^ARCH]

### 1.2 Generational handles and slot recycling

**What it is.** A generational handle packs an index and a generation
counter. A stale handle fails a generation check instead of reading a
recycled slot.

**Why it matters to Cachette.** The record packs a 32-bit index and a
32-bit generation into a non-zero 64-bit value, and it recycles slots
first-in-first-out. The reasoning is sound but the failure modes are
subtle. The command buffer and the tile-to-unit index both hold handles
across the frame barrier.

**Key work.** Catherine West's 2018 talk is the standard reference for
generational indices in Rust.[^12] Bevy's move of the niche into the
index is the current state of the art.[^13]

**Software.** `slotmap` and `generational-arena` own their storage, which
fights the column layout.[^14] `nonmax` gives the niche type.[^15]

**Priority.** High-value.

### 1.3 Allocators, arenas, and huge pages

**What it is.** An arena allocates a large region once and hands out
slices from it. Huge pages reduce translation lookaside buffer misses on
large working sets.

**Why it matters to Cachette.** The tile side alone is 160 MiB. At
4 KiB pages that is about 42,500 page entries against a level 2 buffer
of 1,500 to 3,000 entries. The record calls for two-megabyte aligned
arenas and one `madvise` call at start-up.

**Key work.** Drepper's memory paper is still the best single
introduction.[^16] The Linux transparent huge page documentation states
the failure mode.[^17] Shipilev's measurement shows the effect.[^18]

**Software.** `bumpalo` and `typed-arena` cover the simple cases.[^19]
The Rust allocator interface is still unstable, so a custom aligned
allocation needs `std::alloc` directly.

**Priority.** High-value.

---

## Area 2 — Database internals

This is the single richest source of prior art for Cachette. The
selector engine is a query engine. The pyramid is an index and a
statistics catalogue. The verb apply step is a vectorised update.

### 2.1 Data skipping, zone maps, and small materialised aggregates

**What it is.** A zone map stores the minimum, the maximum, and other
summaries for a block of rows. A scan reads the zone map first and skips
blocks that cannot match.

**Why it matters to Cachette.** The level 1 and level 2 summaries are
exactly zone maps. The three-valued descent, with the verdicts `None`,
`All` and `Some`, is exactly block skipping with an added acceptance
case. The record treats this design as its own invention. It is not. A
literature exists, it names the failure modes, and it gives measured
clustering thresholds.

**Key work.** Moerkotte introduced small materialised aggregates in
1998.[^20] PostgreSQL block range indexes are the open-source
implementation.[^21] Snowflake documents micro-partition pruning at
scale.[^22] Recent work studies how much clustering a data set needs
before skipping pays.

**Software.** DuckDB and ClickHouse both implement zone maps and are
readable.[^23][^24] No Rust crate offers this as a library.

**Priority.** Load-bearing.

**What it unblocks.** The cost model that chooses between the pyramid
descent and the flat scan. The record sets that threshold at 50 percent
by guess and marks it as reasoning rather than measurement.

### 2.2 Selectivity estimation and query planning

**What it is.** A planner estimates how many rows a predicate matches,
then chooses a plan. Estimates come from histograms, sketches, and
sampling.

**Why it matters to Cachette.** A histogram summary gives an exact count
for an equality predicate over one cell. Conjunctions do not. The record
proposes the independence assumption with a clamp to the smaller
selectivity. The independence assumption is the single largest source of
planner error in real database systems, and a world map is strongly
correlated in exactly the fields selectors filter on.

**Key work.** Selinger and colleagues set the framework in 1979.[^25]
Leis and colleagues measured how badly modern estimators fail in
2015.[^26] Graefe's Volcano and Cascades papers define the plan search
that a future optimiser would follow.[^27]

**Software.** Apache Calcite is the reference planner.[^28] Nothing in
Rust is close.

**Priority.** High-value.

**What it unblocks.** The `.explain()` output, the conjunct ordering
rule, and the honesty of the estimated-against-actual report.

### 2.3 Columnar storage and vectorised execution

**What it is.** A column store holds one array per field. A vectorised
engine runs each operator over a batch of values rather than one row at
a time.

**Why it matters to Cachette.** The tile grid and the unit columns are a
column store. The verb apply step is a vectorised operator. The design
question the record has not answered is the batch size and whether the
engine should compile a plan or interpret it. That question returns when
the deferred expression language arrives.

**Key work.** Boncz, Zukowski and Nes defined vectorised execution with
MonetDB/X100 in 2005.[^29] Kersten and colleagues compared compiled
against vectorised execution in 2018 and found the answer is workload
dependent.[^30] Neumann's Umbra is the current state of the art for
compiled execution.[^31]

**Software.** Apache Arrow defines the memory format and a
language-neutral interface.[^32] DataFusion and Polars are the mature
Rust and Rust-backed engines.[^33][^34] Velox is the current C++
reference.[^35]

**Priority.** High-value.

**What it unblocks.** The staged expression language, and the choice of
batch size for the selector predicate scan.

### 2.4 Morsel-driven parallelism

**What it is.** A morsel is a small fixed-size unit of input. Workers
pull morsels from a queue. The scheduler adapts to skew without a static
partition.

**Why it matters to Cachette.** The record's logical span is a morsel.
The record sets the span length to 4096 entities by reasoning and asks
for a benchmark. Morsel-driven parallelism gives the measured guidance
that the record lacks, and it names the skew failure that a static split
causes.

**Key work.** Leis and colleagues, 2014.[^36]

**Priority.** High-value.

**Caution.** Morsel-driven parallelism uses dynamic work assignment,
which conflicts with the project's determinism rule. The study must
extract the sizing guidance and reject the dynamic assignment, or must
prove that the assignment cannot leak into a result.

### 2.5 Bitmap indexes and bitmap compression

**What it is.** A bitmap index stores one bit vector per distinct value.
Set algebra over the vectors answers a conjunctive query.

**Why it matters to Cachette.** The record stores one bitplane per
boolean tile attribute, and it stores results as two-level masks. Report
08 goes further and proposes a shared block-adaptive tile set for both
fog of war and selector tile results.[^1] The record marks the choice of
a purpose-built mask over a general Roaring library as a design argument
that needs a benchmark.

**Key work.** Roaring is the modern baseline, with three papers by
Chambi, Lemire and colleagues.[^37] O'Neil and Quass introduced the
bit-sliced index in 1997, which encodes a numeric column as a set of bit
planes and answers range queries with a small number of vector
operations.[^38] Bit slicing is not in the record and may be a better
form for elevation and health than a min-max pair.

**Software.** The `roaring` crate is mature.[^39] `CRoaring` holds the
vectorised kernels.[^40]

**Priority.** Load-bearing.

**What it unblocks.** The mask benchmark that the record names as open
question 16, and the unification of the fog container with the selector
tile result.

### 2.6 Incremental view maintenance

**What it is.** A materialised view is a stored query result. Incremental
view maintenance updates that result from a change set instead of
recomputing it.

**Why it matters to Cachette.** This is the most under-recognised field
in the whole design. The record's dirty pyramid is a materialised view
over the level 0 write model, and the record's rule that a summary must
be a group with an inverse is the standard incremental maintenance
condition. The field names the general case, gives the change-propagation
algebra, and states which operators can and cannot maintain
incrementally. The record derives a small part of this by hand and calls
it a rule.

The field also answers a question the record does not ask. The unit
pyramid updates every tick because units move every tick. That is a
high-change-rate view, and the field knows when such a view costs more
than it saves.

**Key work.** Gupta and Mumick surveyed the classical results in
1995.[^41] McSherry and colleagues built differential dataflow in
2013.[^42] Budiu and colleagues gave a full algebraic account in DBSP in
2022, which includes a proof that any query in a defined language has an
incremental form.[^43] Gjengset and colleagues built Noria for a
read-heavy workload with partial materialisation, which is the same
shape as the pyramid.[^44]

**Software.** `timely-dataflow` and `differential-dataflow` are mature
Rust crates.[^45] Neither is usable inside the frame, because both
allocate and both order by arrival. Read them for the algebra.

**Priority.** Load-bearing.

**What it unblocks.** The correctness of decision D16, the honesty of
the extremum-count fast path, and the question of whether the second
unit pyramid earns its cost.

### 2.7 Self-adjusting computation and incremental frameworks outside databases

**What it is.** A self-adjusting program records a dependency graph and
recomputes only the parts that a change affects.

**Why it matters to Cachette.** The selector cache is a demand-driven
incremental computation. The record deliberately restricts the cache to
one Python phase so that no invalidation logic is needed. A cross-frame
cache would need this field. The record defers it, and this study says
what the deferral costs.

**Key work.** Acar's thesis defined self-adjusting computation in
2005.[^46]

**Software.** `salsa` powers incremental analysis in `rust-analyzer` and
is the readable Rust example.[^47]

**Priority.** Speculative.

---

## Area 3 — Sparse, succinct, and compressed data structures

### 3.1 Rank, select, and succinct bit vectors

**What it is.** A succinct bit vector answers "how many bits are set
before position `i`" and "where is the `k`th set bit" in constant time,
in space close to the information-theoretic minimum.

**Why it matters to Cachette.** The record's sparse tile side tables use
a hash map for the payload and a bitplane for the index. It notes that a
rank-select structure would remove the hash and give a dense payload
array. Report 08's block-adaptive leaves need a population count per
block, which is a rank query. Rank and select also give an ordered
iteration that a hash map cannot, which the determinism rule requires.

**Key work.** Jacobson introduced rank and select in 1989.[^48] Navarro's
book is the current standard text.[^49] Clark's select structure is the
classical constant-time answer.

**Software.** `sdsl-lite` is the mature C++ library.[^50] In Rust,
`sucds`, `vers` and `bitm` exist and are small.[^51] None is as mature
as `sdsl-lite`. The project would likely write the narrow case it needs.

**Priority.** High-value.

### 3.2 Population count on the target

**What it is.** A population count returns the number of set bits.

**Why it matters to Cachette.** The design is bitplane-heavy. Graviton
and Neoverse cores have no scalar population count instruction, so a
count routes through the vector unit. The record therefore requires
every aggregation kernel to count a whole block at once.

**Key work.** Muła, Kurz and Lemire gave the Harley-Seal method for
vectorised population counting in 2017.[^52] It applies directly to the
16-word and 1024-word block counts that the design needs.

**Priority.** High-value.

**What it unblocks.** The pyramid aggregation kernel and the fog block
rebuild, which are both on the critical path.

### 3.3 Compressed integer sequences

**What it is.** Delta encoding, variable-byte encoding, bit packing, and
Elias-Fano encoding store a sorted integer sequence in far less space
than a fixed-width array.

**Why it matters to Cachette.** The event log is already columnar and
nearly sorted, so it compresses well. The record estimates three to ten
times. A retained event log becomes affordable at a good ratio and stays
unaffordable at a poor one. The fog `Array` leaf is a sorted `u16`
sequence, which is the exact input shape these methods want.

**Key work.** Vigna's quasi-succinct indices apply Elias-Fano to
inverted lists.[^53] Lemire and colleagues give Stream VByte and
SIMD-BP128, which decode at billions of integers per second.[^54]

**Software.** `stream-vbyte` and `bitpacking` exist in Rust and are
small.[^55]

**Priority.** High-value, and it becomes load-bearing if the retained
event log stops being deferred.

### 3.4 Wavelet trees and other compressed indexes

**What it is.** A wavelet tree stores a sequence over an alphabet and
answers rank, select and range queries over the alphabet in logarithmic
time.

**Why it matters to Cachette.** A wavelet tree over the terrain column
would answer "how many tiles of terrain `t` in this range" without
storing a histogram per cell. The record spends 256 bytes per level 1
cell on summaries and enforces a hard budget. A wavelet tree is the
alternative shape for that budget.

**Key work.** Grossi, Gupta and Vitter, 2003.[^56] Navarro's book covers
the modern variants.[^49]

**Priority.** Speculative. Read it before the summary budget is
finalised, then decide.

---

## Area 4 — Persistent and immutable data structures

**What it is.** A persistent structure preserves earlier versions when it
is updated. Structural sharing keeps the cost of a version low.

**Why it matters to Cachette.** Report 08 holds each fog leaf behind an
atomic reference count and copies on write, so a fog snapshot costs 256
pointer writes. The record's dirty-cell snapshot is copy-on-write at
block granularity. Rollback, replay, and time travel all depend on
cheap versioning, and all three are deferred rather than rejected.

**Key work.** Bagwell's hash array mapped trie is the base of most
practical persistent maps.[^57] Bagwell and Rompf's relaxed radix
balanced tree gives a persistent vector with fast concatenation.[^58]
Rodeh's copy-on-write B-tree is the storage-side answer and is what
several file systems use.[^59]

**Software.** `im` and `rpds` give persistent collections in Rust.[^60]
Neither fits a hot column. Read them for the sharing pattern only. The
reference-counted leaf in report 08 is the correct shape for this
project.

**Priority.** High-value.

**What it unblocks.** The rollback window question, and whether the
snapshot ring is built at all.

---

## Area 5 — Spatial data structures and computational geometry

### 5.1 Hex grids and discrete global grid systems

**What it is.** A discrete global grid system partitions a surface into
cells at several resolutions. Hexagonal systems must choose an aperture,
because hexagons do not tile into larger hexagons.

**Why it matters to Cachette.** The record rejects aperture 7 and takes
power-of-two blocks in an offset index space. It records that a
geospatial requirement would reverse that decision. It also records that
the original reason for the rejection was wrong. The field is settled
enough that further study has a narrow purpose: to confirm that no
published planar hex hierarchy gives both exact nesting and a
power-of-two parent shift.

**Key work.** Amit Patel's hexagon reference is the practical
source.[^61] Sahr, White and Kimerling give the geodesic discrete grid
framework.[^62] The Eisenstein integer paper covers aperture `2^2k`
indexing.[^63]

**Software.** H3 and S2 are the mature systems.[^64][^65] The `hexx`
crate covers axial arithmetic in Rust.[^66]

**Priority.** Speculative, unless geospatial interoperation becomes a
requirement. Then it is load-bearing.

### 5.2 Hierarchical sparse grids from computer graphics

**What it is.** A sparse volumetric structure stores a large grid as a
shallow tree of fixed-branching nodes with dense leaf tiles. Empty
regions cost nothing.

**Why it matters to Cachette.** This is the closest existing engineering
match to the record's design, and no report mentions it. OpenVDB is a
three-level tree with a fixed branching factor, dense leaf tiles, per-node
value masks, and constant-time random access. Report 08's block-adaptive
tile set independently rediscovers a large part of it. OpenVDB also
solves the problems Cachette has next: sparse iteration in a defined
order, level-set operations, and out-of-core streaming.

**Key work.** Museth's OpenVDB paper, 2013.[^67] NanoVDB is the
read-only, pointer-free form, which matters because Cachette needs
plain-data types.[^68] Laine and Karras give the sparse voxel octree for
comparison.[^69]

**Software.** OpenVDB is mature C++.[^70] Nothing comparable exists in
Rust.

**Priority.** High-value.

**What it unblocks.** The unified tile set that report 08 proposes, the
leaf threshold question, and the iteration order guarantee.

### 5.3 Neighbour search, cell lists, and spatial sorting

**What it is.** A cell list bins particles into a uniform grid. A
neighbour query reads the own cell and its neighbours. A Verlet list
caches the neighbour set and rebuilds it on a displacement threshold.

**Why it matters to Cachette.** The record's tile-to-unit bridge is a
cell list. The record's steering term reads the occupancy index of the
six neighbours and says that no neighbour search is needed. The record
also asks whether the arena should be re-sorted by spatial key each
tick, and marks the interaction with determinism as open. Molecular
dynamics has answered exactly this question for thirty years, at
comparable particle counts, and with a hard reproducibility requirement.

**Key work.** Verlet, 1967.[^71] Allen and Tildesley cover cell lists
and neighbour lists in full.[^72] The molecular dynamics codes publish
their reordering cadence and their determinism guarantees.

**Software.** LAMMPS, GROMACS and HOOMD-blue are the production
codes.[^73] Their spatial sorting and their treatment of reproducibility
under domain decomposition are the material to read.

**Priority.** Load-bearing.

**What it unblocks.** Open question 14, which is whether the arena gets
re-sorted by spatial key each tick. It also informs the crowd and
movement report that is in flight.

### 5.4 Robust geometric predicates

**What it is.** An exact predicate answers a geometric test without a
rounding error changing the answer.

**Why it matters to Cachette.** The hex line-drawing routine rounds in
cube space and corrects the component with the largest error. Report 02
names a tie on an exact edge as a classic determinism bug. The design
bans floating point from state, which removes most of the risk, but the
tie-breaking rule still needs a proof.

**Key work.** Shewchuk's adaptive precision predicates, 1997.[^74]

**Priority.** Speculative. The integer-only rule already covers most of
this.

### 5.5 Space-filling curves

**What it is.** A Morton or Hilbert curve maps a multi-dimensional index
to one dimension while preserving locality.

**Why it matters to Cachette.** Report 02 rejects per-tile Morton order
and suggests Morton order for the chunk array as an optional refinement.
The record does not decide it. The choice affects level 2 aggregation
locality and viewport scans.

**Key work.** Moon and colleagues analysed Hilbert curve clustering in
2001.[^75]

**Priority.** Speculative.

---

## Area 6 — Parallel algorithm design and deterministic parallelism

### 6.1 Deterministic parallelism as a discipline

**What it is.** A parallel program is internally deterministic when every
intermediate value, not only the final result, is independent of the
schedule.

**Why it matters to Cachette.** This is the project's highest-priority
property, and the record enforces it with hand-written rules: disjoint
outputs, indexed slot reductions, ordering by a stable key, and
concatenation in span-index order. Those rules are correct. They are
also folklore in the record. A field exists that states them as theorems,
gives the class of algorithms that admit them, and names the primitives
that preserve them under composition.

**Key work.** Blelloch, Fineman, Gibbons and Shun defined internally
deterministic parallel algorithms in 2012 and gave a benchmark suite of
them.[^76] Frigo and colleagues defined reducer hyperobjects in Cilk in
2009, which give an associative reduction whose result does not depend on
the steal pattern.[^77] Blumofe and Leiserson gave the work-stealing
bounds in 1999.[^78]

**Software.** `rayon` is the Rust scheduler and its `fold` and `reduce`
are exactly the unsafe case the record bans.[^79]

**Priority.** Load-bearing.

**What it unblocks.** A stated determinism argument in the record, in
place of a list of rules. It also gives the vocabulary for the
determinism test.

### 6.2 Work-efficient primitives: scan, segmented scan, and sort

**What it is.** A small set of primitives composes into most
data-parallel algorithms. The set is map, scan, reduce, filter,
segmented scan, and sort.

**Why it matters to Cachette.** The record names a small kernel
vocabulary as a project ideal and lists map, gather, scatter, reduce,
scan, sort, stencil and local join. It also decides to build one very
good parallel stable radix sort, and names sorting as the second most
used primitive after the flow tile. Nothing in the record cites the
literature that defines these primitives or their work and depth bounds.

**Key work.** Blelloch's prefix sums paper is the origin.[^80] Merrill
and Grimshaw gave the modern high-throughput radix sort in 2011.[^81]
Obeya and colleagues gave a work-efficient parallel in-place radix sort
in 2019.[^82]

**Software.** `rdst` and `voracious_radix_sort` are the Rust
candidates.[^83] Neither promises stability and a deterministic parallel
split at once. The project will likely write its own, as the record
already assumes.

**Priority.** Load-bearing.

**What it unblocks.** Decision D50. The sort is on the critical path for
the occupancy rebuild, sort-merge joins, event apply, and batched
nearest neighbour.

### 6.3 Scheduling and conflict graphs

**What it is.** A static schedule colours a conflict graph into stages
and fixes the order. A dynamic schedule dispatches on availability.

**Why it matters to Cachette.** The record compiles a static stage list
and rejects a dynamic executor. It then adds a dynamic element that Bevy
does not have: the region scope of a command comes from the resolved
selector, so the conflict graph rebuilds every tick. The record estimates
that cost as a few million bitmask comparisons and does not cite a
source.

**Key work.** Graph colouring and list scheduling are classical. The
interesting current material is in task-parallel runtimes and in
register allocation, both of which colour conflict graphs under a time
budget.

**Priority.** High-value.

### 6.4 Graph partitioning and domain decomposition

**What it is.** A partitioner splits a graph or a domain into balanced
parts with few edges between them.

**Why it matters to Cachette.** The record makes the level 1 cell the
aggregate and the parallelism boundary. A uniform grid partition is the
simplest possible decomposition, and it will produce stragglers when one
cell holds a battle and its neighbours hold empty ground. The record
names the straggler as a failure mode and offers no method.

**Key work.** METIS is the standard partitioner.[^84] The high-performance
computing literature on load balancing for particle codes is closer to
this workload, because it faces the same clustering.

**Priority.** High-value.

---

## Area 7 — Numerical methods, in fixed point

### 7.1 Eikonal solvers and distance fields

**What it is.** An eikonal solver computes the arrival time of a front
across a domain with a variable speed. A flow field is the gradient of
that arrival time.

**Why it matters to Cachette.** The record builds a flow tile inside one
32-by-32 pathing chunk with a bucket-queue Dijkstra. That is the discrete
form. The continuous form gives smoother motion and handles a density
term naturally. The record takes one idea from continuum crowds, which is
density in the cost, and rejects the rest on cost. This is the right
call, but the project has not checked whether a cheap fixed-point
eikonal solve is now affordable at a 32-by-32 tile.

**Key work.** Tsitsiklis in 1995 and Sethian in 1996 gave the fast
marching method.[^85] Zhao's fast sweeping method from 2005 is often
faster on a regular grid and is far simpler to make deterministic,
because it is a fixed number of directional sweeps rather than a priority
queue.[^86] Dial's bucket queue from 1969 is the integer-cost shortest
path that the record already uses without naming it.[^87]

**Priority.** Load-bearing.

**Note.** The crowd and movement report now in preparation covers part of
this. Confirm its scope before studying the steering side.

### 7.2 Diffusion, relaxation, and multigrid

**What it is.** A diffusion step blends each cell with its neighbours.
Relaxation methods solve a linear system by repeated local updates.
Multigrid accelerates that by solving on a hierarchy of resolutions.

**Why it matters to Cachette.** The record maintains four to eight
influence maps as a seven-point hex stencil over the level 1 grid, and
the resource flow design uses a few Gauss-Seidel iterations per tick
without solving to convergence. Cachette already owns a resolution
hierarchy. Multigrid is the method that uses such a hierarchy to make
relaxation converge in a number of steps that does not grow with the
grid size. Nobody has checked whether the pyramid can carry a multigrid
cycle.

**Key work.** Brandt's 1977 paper defines multigrid.[^88] The Briggs
tutorial is the readable introduction.[^89]

**Priority.** High-value.

**Note.** The influence map report and the resource and trade flow report
are both in preparation. Both touch this. Coordinate before studying it.

### 7.3 Fixed-point arithmetic and numerical error

**What it is.** Fixed-point arithmetic represents a fractional value as
an integer with an implied scale. It is exactly reproducible.

**Why it matters to Cachette.** The record bans floating point from
simulated and aggregated state and sets one scale, Q16.16, everywhere.
That is a strong and correct decision. It also creates work the record
has not scoped: division needs a shift and overflow care, and square
root, sine and arc tangent need tables or polynomials. The record
estimates 300 lines. That estimate is not sourced. The record also lists
"which transcendental functions does the simulation actually need" as an
open question.

**Key work.** Higham's book covers error analysis in general.[^90] The
digital signal processing literature holds the practical fixed-point
material. Deterministic physics engines are the closest applied source.

**Software.** The `fixed` crate gives the types.[^91] The `libm` crate
gives a pure-Rust libm if the design ever needs a pinned floating-point
path.[^92] Nothing gives a fixed-point transcendental library that the
project can adopt directly.

**Priority.** Load-bearing.

**What it unblocks.** The `sim_math` module contents, and the cost of the
no-float rule stated as a real number.

### 7.4 Stencil computing and temporal blocking

**What it is.** A stencil updates each cell from a fixed neighbourhood.
Temporal blocking fuses several time steps over a tile to raise the
arithmetic intensity.

**Why it matters to Cachette.** Every full-map pass costs at least 3.3 ms
of pure bandwidth, and the record allows two or three per tick. The
influence map update, the fog rebuild, and the pyramid recompute are all
stencil-shaped or reduction-shaped. Stencil optimisation is the field
that tells the project how to fuse them into fewer passes.

**Key work.** Datta and colleagues gave the standard auto-tuning study in
2008.[^93] The roofline model gives the framing for whether a kernel is
bandwidth bound or compute bound.[^94]

**Priority.** High-value.

---

## Area 8 — Operations research

### 8.1 Network flow

**What it is.** A flow problem routes a quantity through a capacitated
graph at minimum cost.

**Why it matters to Cachette.** The resource and trade flow report is in
preparation, so this area is partly covered. The record's `transfer` verb
already contains a small flow problem: sum demand, scale, distribute the
remainder. Supply range, trade routes, and reinforcement routing are all
flow problems on a level 1 graph of tens of thousands of nodes.

**Key work.** Ahuja, Magnanti and Orlin is the standard text.[^95]
Goldberg and Tarjan's push-relabel algorithm is the practical
maximum-flow method.[^96] Network simplex is the practical minimum-cost
method and has an exact integer form, which matters here.

**Software.** `pathfinding` and `petgraph` cover the basics in Rust.[^97]
An exact integer minimum-cost flow at this scale would need work.

**Priority.** High-value. Confirm the in-flight report's scope first.

### 8.2 Assignment and matching

**What it is.** An assignment problem pairs agents with tasks at minimum
total cost.

**Why it matters to Cachette.** Target selection, worker allocation, and
transport loading are assignment problems. The record's `attack` verb
does target acquisition by a local ring scan inside contested cells,
which is a greedy heuristic. A greedy heuristic is order-dependent, and
order-dependence is the thing the design most wants to avoid.

**Key work.** Kuhn's Hungarian method from 1955 is the exact
answer.[^98] Bertsekas's auction algorithm from 1988 parallelises and
has an integer form with a defined epsilon.[^99]

**Priority.** High-value.

**What it unblocks.** A determinism proof for target selection, which
the record does not have.

### 8.3 Apportionment and exact integer division

**What it is.** An apportionment method divides an integer quantity into
integer parts in proportion to weights.

**Why it matters to Cachette.** The record uses the largest-remainder
method in three places: `transfer`, `build`, and `spawn`. It states that
this conserves exactly and is order-independent. The apportionment
literature knows that largest remainder has paradoxes, including the
case where increasing the total reduces one share.

**Key work.** Balinski and Young give the full treatment, including the
impossibility result.[^100]

**Priority.** High-value. The study is short and it closes a real
correctness question in three shipped verbs.

### 8.4 Constraint satisfaction and scheduling

**What it is.** A constraint solver assigns values to variables subject
to constraints. A scheduler orders tasks under precedence and resource
limits.

**Why it matters to Cachette.** Production queues, build orders, and
placement under capacity are all constraint problems. The record does not
have a solver and does not need one in version 1. The question is whether
the control plane should be able to reach one.

**Software.** OR-Tools CP-SAT is the mature solver and has a Python
interface, so it belongs on the control plane rather than in the
core.[^101] `good_lp` covers linear programming in Rust.[^102]

**Priority.** Speculative.

### 8.5 Multi-agent path finding

**What it is.** Multi-agent path finding routes many agents without
collisions, optimally or near-optimally.

**Why it matters to Cachette.** The record rejects per-agent avoidance at
one million units and takes a flow field plus a three-term steering
blend. That is correct at that scale. The field still matters for two
reasons. It gives the vocabulary for the failure modes the record accepts,
such as deadlock in a corridor. It also gives the method for the small
unit classes where the record allows an exact solver.

**Key work.** Stern and colleagues give the standard problem definitions
in 2019.[^103] Sharon and colleagues gave conflict-based search in
2015.[^104]

**Priority.** Speculative.

**Note.** The crowd and movement report in preparation may cover this.

---

## Area 9 — Agent-based modelling and complexity science

**What it is.** Agent-based modelling simulates a population of
interacting agents and studies the aggregate behaviour that emerges.

**Why it matters to Cachette.** Audience three is the clearest
differentiator. That audience has conventions, reporting standards, and
existing frameworks. Cachette must either match them or state clearly why
it does not. The field also holds the scaling techniques for large
populations, which is the same problem the record solves.

**Key work.** Epstein and Axtell's 1996 book set the pattern.[^105]
Bonabeau's 2002 survey is the standard citation.[^106] The ODD protocol
is the reporting standard that reviewers expect for a published
model.[^107] Population-scale epidemic models are the current proof that
this scale is reachable and useful.[^108]

**Software.** Repast HPC is the distributed C++ framework.[^109] FLAME
GPU shows the data-parallel form.[^110] Mesa, NetLogo and MASON are the
accessible frameworks that set user expectations.[^111] Agents.jl is the
current well-engineered example.[^112] `krABMaga` is the Rust
framework and is small.[^113]

**Priority.** High-value.

**What it unblocks.** The shape of the public interface for audience
three, and the decision on whether Cachette should emit an ODD-style
model description.

---

## Area 10 — Reinforcement learning environments

**What it is.** A reinforcement learning environment exposes a `step`
function, an observation space, and an action space. Throughput is the
binding constraint on research.

**Why it matters to Cachette.** The record makes `WorldBatch` an open
question and says it is the highest-value feature for audience three. The
record also defers the observation-space design entirely. Observation
space design is not a small matter. It decides what the pyramid must
expose, whether the summaries are the observation, and whether a zero-copy
tensor handoff is possible.

**Key work.** Weng and colleagues built EnvPool in 2022 and measured the
cost of the Python boundary in vectorised environments.[^114] Shacklett
and colleagues built Madrona in 2023, which is a batch simulator engine
with a design that overlaps Cachette closely.[^115] Suarez and colleagues
built Neural MMO, which is the closest published environment by
domain.[^116]

**Software.** Gymnasium and PettingZoo define the interfaces that
researchers expect.[^117] PufferLib addresses the multi-agent throughput
problem directly.[^118] The array interchange standards, DLPack and the
Python array API, define how to hand a buffer to PyTorch or JAX without a
copy.[^119]

**Priority.** Load-bearing.

**What it unblocks.** Open question 10 on `WorldBatch`, the observation
interface, and the decision on whether to support a tensor handoff.

---

## Area 11 — Determinism, reproducibility, and verification

### 11.1 Floating-point reproducibility

**What it is.** A reproducible computation returns identical bits for
identical inputs on a stated set of platforms.

**Why it matters to Cachette.** The record bans floating point from
state, which removes most of this problem. It keeps a `sim_math`
boundary so the stronger cross-platform guarantee stays reachable. The
remaining risk is the algebraic float operations that stabilised in Rust
1.98, and the floating-point control register that another library in
the same process can change.

**Key work.** Rust RFC 3514 states the language guarantee.[^120] IEEE
754-2019 is the standard.[^121] Goldberg's 1991 paper is the standard
introduction.[^122]

**Priority.** Load-bearing, and largely already resolved by the record.

### 11.2 Counter-based random number generation

**What it is.** A counter-based generator computes a value as a keyed
hash of a counter. It has no state to advance.

**Why it matters to Cachette.** The record requires every draw to be a
pure function of system, frame, entity and draw index. It proposes a
`splitmix64` of a packed key and pins it with known-answer tests. The
statistical quality of that construction is not established.

**Key work.** Salmon and colleagues defined Philox and Threefry in
2011.[^123] The test suites are TestU01 and PractRand.

**Priority.** High-value. The study is bounded: run the proposed mixer
through a standard test battery and record the result.

### 11.3 Property-based and metamorphic testing

**What it is.** A property test generates inputs and checks a property.
A metamorphic test checks a relation between the outputs of two related
inputs, when no oracle exists for a single output.

**Why it matters to Cachette.** There is no oracle for "the simulation
gave the right answer". There are many metamorphic relations. A selector
and its negation partition the world. Running the same tick at a
different thread count must give identical bytes. Recomputing a pyramid
cell in full must equal the incremental value. Applying a command to an
empty set must change nothing. The record lists several of these and does
not name the technique.

**Key work.** Claessen and Hughes introduced property-based testing in
2000.[^124] Chen and colleagues introduced metamorphic testing in 1998,
and Segura and colleagues surveyed it in 2016.[^125]

**Software.** `hypothesis` on the Python side, with its stateful
rule-based machine, is the right tool for boundary properties.[^126]
`proptest` covers the Rust side.[^127]

**Priority.** Load-bearing.

### 11.4 Deterministic simulation testing

**What it is.** A whole system runs inside a deterministic scheduler with
injected faults. A failing run replays exactly.

**Why it matters to Cachette.** Cachette is already deterministic by
construction, which is the expensive prerequisite that other projects pay
for. That means the technique is nearly free here, and the project should
know what it buys. The technique finds bugs that no unit test finds, and
it turns a rare failure into a reproducible one.

**Key work.** FoundationDB's simulation testing is the origin and is well
documented.[^128] TigerBeetle and Antithesis are the current
practitioners.[^129]

**Priority.** High-value.

### 11.5 Formal methods and verification tools

**What it is.** A model checker explores the state space of a
specification. A verifier proves a property of real code.

**Why it matters to Cachette.** The five-barrier frame loop, the phase
split between reading and writing, and the invariant classification are
all specifications that a model checker can check. The unsafe code in the
arena is what a verifier can check. The record already requires Miri in
continuous integration, which is the cheapest step on this ladder.

**Key work.** Lamport's TLA+ is the standard specification tool.[^130]
Jung and colleagues gave the aliasing model that Miri implements.[^131]

**Software.** Miri, `loom`, and `kani` cover three different classes of
error in Rust.[^132] `loom` is worth a look for the atomic dirty-bitset
pattern under the weak ARM memory model.

**Priority.** High-value.

---

## Area 12 — Compression, streaming, and out-of-core state

**What it is.** An out-of-core system works on a data set larger than
memory. A buffer manager decides what is resident.

**Why it matters to Cachette.** The current target fits in memory. The
record's rejection of generative promotion rests on that fact and says
to revisit if the map grows to 65536 by 65536. A larger map, many worlds
in one process, or a retained event log all move the project out of
core. The record defers the retained log because it costs 11.5 GB per
minute of simulated time.

**Key work.** Leis and colleagues built LeanStore in 2018, which shows a
modern low-overhead buffer manager.[^133] Neumann and Freitag's Umbra
extends it with variable-size pages.[^134] Pelkonen and colleagues gave
Gorilla in 2015, which is the standard time-series compression.[^135]

**Software.** `zstd` with a trained dictionary is the practical default
for the log.[^136] `io_uring` is the current Linux path for high-rate
asynchronous input and output.[^137]

**Priority.** Speculative today. It becomes load-bearing if the map size
grows or the retained log is undeferred.

---

## Area 13 — Rust specifics

### 13.1 Unsafe abstractions and their verification

**Why it matters to Cachette.** The arena needs raw column pointers,
casts to component slices, and manual layout. The record puts the core in
a crate with no Python dependency so that Miri can run on it. Miri finds
aliasing and provenance errors that no test finds.

**Key work.** The Rustonomicon and the unsafe code guidelines are the
practical sources. Stacked Borrows is the aliasing model Miri
implements.[^131]

**Priority.** Load-bearing.

### 13.2 Single instruction multiple data on aarch64

**Why it matters to Cachette.** NEON is mandatory in the base
instruction set, so the record writes one code path and no runtime
dispatch. The remaining risk is that automatic vectorisation fails
silently after a refactor. The record requires pinning the hot kernels
with an assembly check in continuous integration.

**Software.** `wide` gives explicit lanes on stable Rust.[^138] `pulp`
adds multiversioning but exposes only the native width.[^139]
`std::simd` is still nightly-only and a nightly toolchain would break
wheel packaging.[^140] `cargo-show-asm` is the assembly check.[^141]

**Priority.** High-value.

### 13.3 Const generics, newtypes, and zero-cost interfaces

**Why it matters to Cachette.** The record newtypes every index and
declares a compile-time budget of 256 bytes per level 1 cell. The field
registry macro must generate four artefacts from one declaration and must
fail the build on a budget breach. That is a macro and const-generic
design problem, and it is the mechanism that makes the aggregation rule
mechanical rather than advisory.

**Priority.** High-value.

### 13.4 Toolchain and supply chain

**Why it matters to Cachette.** The record puts packaging in week one:
five platforms, a stable application binary interface, a free-threaded
job, a source distribution test, a pinned minimum Rust version, and a
licence and advisory check.

**Software.** `cargo-deny`, `sccache`, and the maturin continuous
integration generator cover this.[^142]

**Priority.** High-value.

---

## Area 14 — Python interoperability and the scientific ecosystem

**What it is.** The boundary between a compiled extension and the Python
interpreter, and the conventions that the scientific Python ecosystem
expects.

**Why it matters to Cachette.** The record decides that the core crate
holds no Python dependency, that the interpreter detaches for the whole
step, that a view lives inside a scope, and that `to_numpy` copies. It
also records that a stale array view pointing at Rust memory is the
highest-severity risk in the whole boundary area. That risk is not
solved by any library. It is solved by the three-layer scheme the record
proposes, which nobody has yet tested.

**Key work.** The relevant Python enhancement proposals are 703 on
free-threading, 779 on its supported status, 803 on the stable
application binary interface for free-threaded builds, and 734 on
multiple interpreters.[^143]

**Software.** PyO3 and rust-numpy are the binding crates and must be
version-matched.[^144] maturin and cibuildwheel build the wheels.[^145]
The array interchange standards define the handoff to other array
libraries.[^119] `nanobind` is the C++ comparison and its performance
notes are worth reading.[^146]

**Priority.** Load-bearing.

**What it unblocks.** The view-safety scheme, the minimum Python version,
and the free-threading position.

---

## Area 15 — Distributed and parallel simulation

**What it is.** Parallel discrete event simulation splits a simulation
across processes and keeps the event order correct. Conservative methods
never execute out of order. Optimistic methods execute ahead and roll
back.

**Why it matters to Cachette.** Cachette is single-server today and
netcode is deferred. Three ideas transfer even so. Interest management
decides which observer needs which update, and the record's fog tiers
are an interest management scheme without the name. Time Warp is the
canonical rollback design and it states exactly what state saving must
cost. Lookahead analysis is the formal version of the record's
five-barrier argument.

**Key work.** Fujimoto's book is the standard text.[^147] Jefferson
defined Time Warp in 1985.[^148] Chandy, Misra and Bryant gave the
conservative method.[^149] The distributed interactive simulation and
high level architecture standards hold the interest management
work.[^150]

**Software.** ROSS is the current open optimistic simulator.[^151]

**Priority.** High-value for interest management. Speculative for the
rest until netcode is undeferred.

---

## Area 16 — Game artificial intelligence at scale

**What it is.** Decision-making for game agents. The standard methods are
utility systems, goal-oriented planning, hierarchical task networks, and
behaviour trees.

**Why it matters to Cachette.** The record has a per-unit budget of about
400 nanoseconds of core time for everything, shared between movement,
combat, and planning. None of the four standard methods fits that budget
per agent. The honest question is which of them survive at all, and at
what level of the hierarchy they run. The plausible answer is that agent
decisions move to the level 1 cell and to the faction, and that units
become a field rather than a set of deciders. The record's influence maps
and flow fields already point that way. Nobody has written that argument
down.

**Key work.** Orkin's goal-oriented action planning is from 2006.[^152]
Dave Mark's utility system work is the standard reference for scoring
architectures.[^153] Nau and colleagues gave SHOP2 for hierarchical task
networks in 2003.[^154] Colledanchise and Ögren's 2018 book covers
behaviour trees.[^155] Tozour's influence map chapter is the classic
statement of the field-based alternative.[^156]

**Priority.** High-value.

**What it unblocks.** A stated position on where decisions happen, which
is currently implicit. The influence map report in preparation covers
part of the field-based side.

---

## Area 17 — Procedural generation

**What it is.** Procedural generation produces content from a seed and a
set of rules.

**Why it matters to Cachette.** The engine needs a world. A 16.7 million
tile world cannot be hand-authored. Generation must be deterministic from
a seed, and it must use the same counter-based generator as the
simulation, or the project has two randomness systems and one of them
will drift. Generation is also the first serious test of the tile schema
and of the write path at full scale.

**Key work.** Perlin's 1985 noise paper is the origin.[^157] Gumin's wave
function collapse and Karth and Smith's 2017 analysis of it cover
constraint-based generation.[^158] Amit Patel's polygon map generation is
the readable worked example for a game world.[^159] The procedural
content generation book covers the field.[^160] Musgrave and colleagues
gave the erosion model in 1989.[^161]

**Software.** `noise-rs` and `fastnoise-lite` cover the noise
functions.[^162] Neither is deterministic across versions by contract, so
the project must pin or vendor.

**Priority.** High-value.

**What it unblocks.** The dogfooding milestone. It also stress-tests the
tile schema before the schema is frozen.

---

## Area 18 — Event stores, log-structured storage, and time series

**What it is.** A log-structured store appends and never updates in
place. An event store keeps the sequence of facts as the source of
truth.

**Why it matters to Cachette.** The record uses event sourcing as a shape
and defers the retained log. The shape it keeps is the important part:
validation reads, apply writes, events are plain data, and the apply step
is pure. When the log is undeferred, the field gives the storage format,
the compaction policy, and the snapshot cadence.

**Key work.** O'Neil and colleagues defined the log-structured merge tree
in 1996.[^163] Kreps's account of the log as an abstraction is the
practical framing.[^164] Fowler and Young give the event sourcing and
command-query separation vocabulary that the record already borrows.[^165]

**Software.** `redb` and `sled` are the embedded Rust key-value
stores.[^166] Neither is needed for the raw block snapshot the record
specifies. The columnar event arenas map onto Parquet-style encodings for
export, which is the useful connection to the research audience.[^167]

**Priority.** Speculative today. It becomes load-bearing when the log is
retained.

---

## Area 19 — Hardware

**What it is.** The cache hierarchy, the prefetchers, the translation
lookaside buffer, the memory model, and the performance counters of the
target processor.

**Why it matters to Cachette.** Every byte-budget claim in the record is
a hardware claim. The record already records that development machines
mislead, because Apple Silicon uses 128-byte cache lines and Neoverse
uses 64. It records that the weak memory model promotes the
disjoint-output rule from preferred to required. It has not measured any
of this on the target.

**Key work.** Drepper's memory paper is the general introduction.[^16]
The Arm Neoverse software optimisation guides give the latency and
throughput tables.[^168] The AWS Graviton technical guide gives the
platform-specific advice.[^169] The roofline model gives the framing for
which kernels are bandwidth bound.[^94] Yasin's top-down method is the
standard way to attribute stalls, and Arm publishes a Neoverse
version.[^170]

**Software.** `perf` covers the counters. The Arm Statistical Profiling
Extension gives instruction-level sampling on Neoverse and is the tool
that will answer the cache questions.[^171]

**Priority.** Load-bearing.

**What it unblocks.** Every budget table in the record. The record itself
says to treat the core-millisecond column as reliable and to re-derive
the wall-time column on the target.

---

## Area 20 — Visualisation and rendering of very large state

**What it is.** Rendering and interactive inspection of a data set too
large to draw one element at a time.

**Why it matters to Cachette.** The engine is headless and multi-tenant.
It has no renderer and should not grow one. It still has to hand a view
of 16.7 million tiles and one million units to something that draws it.
That means server-side aggregation to a viewport, tiling, and a defined
level-of-detail policy. The pyramid is already the level-of-detail
structure. The open question is the transport.

**Key work.** Ulrich's chunked level of detail from 2002 and Losasso and
Hoppe's geometry clipmaps from 2004 are the terrain methods.[^172] The
in-situ visualisation field addresses the general case, which is a
simulation too large to move to a visualiser.[^173]

**Software.** Datashader aggregates large point sets server-side and its
model matches the pyramid closely.[^174] deck.gl and vector tiles cover
the web transport.[^175] The cloud-optimised raster formats are the
established answer to "serve a pyramid over a network".[^176]

**Priority.** High-value for the dogfooding audience. Speculative
otherwise.

---

## Area 21 — Observability and benchmarking methodology

**What it is.** The measurement of a running system, and the statistics
that make a measurement trustworthy.

**Why it matters to Cachette.** The record contains many numbers that it
marks as reasoning rather than measurement: the 32-by-32 pathing chunk,
the span length of 4096, the 50 percent descent threshold, the mask
design, and the false-sharing analysis. Each needs a benchmark that is
believable. A benchmark on a shared cloud instance, on a different
architecture, is not believable.

**Key work.** Ousterhout's argument for measuring one level deeper is the
right discipline.[^177] Curtsinger and Berger's causal profiler answers
"which change would help", which a flame graph does not.[^178] The
statistics of micro-benchmarking are the part most projects get
wrong.[^179]

**Software.** `criterion` and `divan` are the Rust harnesses.[^180]
`tracing` gives structured spans and `tracy` gives frame timing.[^181]
`py-spy` with native frames is the best first tool across the language
boundary.[^182] A continuous benchmarking service on a pinned Graviton
runner is the piece the project does not have.

**Priority.** Load-bearing.

**What it unblocks.** Every open question in the record that is marked
"needs a measurement". There are five of them.

---

## The highest-leverage unknowns

These are the places where the project is most likely to be wrong in a
way that matters. Each names the field that would tell us.

### U1. The pyramid may not prune the queries the game actually asks

The record's central claim is that the pyramid is the display structure,
the query index, and the statistics catalogue at once. Hierarchical
skipping works when matches cluster. A world map clusters strongly in
terrain and weakly in health, order state, and anything that changes
every tick. If most real predicates return `Some` at every level, the
descent costs the summary reads on top of a full scan.

The record accepts this risk and mitigates it with a flat scan fallback
and a threshold set by guess. It has no measurement of the real query
mix, because no game code exists.

**The field that would tell us:** database data skipping and selectivity
estimation, and specifically the measured clustering thresholds at which
zone maps stop paying.[^20][^22][^26]

**What would settle it:** write the flat path first, as the record
already says, and instrument the descent to report its verdict
distribution per level. Then dogfood, and read the distribution.

### U2. Cost may scale with entity count after all, because units move

The ideal is that cost scales with map area or with changed state. Units
break it. One million units moving every tick dirty most level 1 cells
every tick. The record's answer is a second, delta-only unit pyramid.
Delta-only removes the recompute pass. It does not remove the write.

The record has not stated the arithmetic for the case where every unit
crosses a cell boundary. Neither has any report.

**The field that would tell us:** incremental view maintenance, which
gives the condition under which an incremental update costs more than a
recomputation.[^41][^43] Also kinetic and dynamic data structures, which
study exactly the case of a structure over moving points.

**What would settle it:** a benchmark of the unit pyramid at full unit
count with a worst-case movement pattern.

### U3. Determinism may not survive the optimisations the design wants

Three recorded decisions each interact with determinism, and each is
open. Re-sorting the arena by spatial key changes iteration order. The
dynamic conflict graph rebuild depends on resolved selectors. The
extremum-count rescan path fires at a data-dependent time. The record
notes each interaction and resolves none.

**The field that would tell us:** internally deterministic parallel
algorithms, which gives the definition strong enough to check these
cases.[^76] Also the reproducibility work in molecular dynamics, which
faced the same reordering question at the same scale.[^73]

**What would settle it:** state the determinism contract as a property
over intermediate values, not only over the final hash, and then check
each of the three cases against it.

### U4. Fixed point may not be cheap for the numerical kernels

The no-float rule is correct and the record defends it well. Its cost is
scoped as "about 300 lines" for square root, sine, and arc tangent. That
estimate is unsourced. The kernels that need care are the flow-tile
gradient, the steering normalisation, and any diffusion with a decay
factor. Each involves a division or a normalisation, and each can
accumulate a systematic bias in fixed point that a float would not have.

A biased flow field is not a correctness bug that a test catches. It is a
slow drift in unit behaviour.

**The field that would tell us:** fixed-point numerical analysis and
error analysis, and integer shortest-path and eikonal
methods.[^90][^85][^86][^87]

**What would settle it:** implement one flow tile in fixed point and
compare its field against a high-precision reference over many builds.

### U5. The verb vocabulary may not be expressively complete

The record ships about 12 verbs and applies three tests before adding
another. It also records the real failure mode: if a common need has no
selector form, users write the loop, and the whole control-plane design
fails at that point. The operations it flags to watch are set difference,
top-k, nearest, sort-by, and random sample.

This is a language design question, and language design questions are
settled by use, not by argument. The risk is highest for audience three,
because a reinforcement learning action space must cover everything a
policy may choose, and a policy will choose things a human player never
would.

**The field that would tell us:** action-space design in reinforcement
learning, and relational completeness in query languages.[^114][^117]

**What would settle it:** write the action space for one real learning
task against the proposed verb set, before the verb set is frozen.

---

## Adjacent fields that are not obviously relevant and probably are

### A1. Sparse volumetric data structures in computer graphics

Visual effects has solved "a very large grid that is mostly empty, read
and written at high rate, queried hierarchically, and streamed to disk".
OpenVDB is a three-level tree with a fixed branching factor, dense leaf
tiles, and per-node value masks. Report 08 independently reinvents a
large part of it. NanoVDB is the pointer-free read-only form, which is
what a plain-data discipline needs. The field also holds the answers to
the questions Cachette asks next: a defined sparse iteration order, level
set operations, and out-of-core streaming.[^67][^68][^70]

This is the single most underrated adjacent field for this project.

### A2. Molecular dynamics neighbour handling

Molecular dynamics runs millions of particles in lockstep, queries them
by proximity, aggregates them hierarchically, and demands
reproducibility. It has done so for decades on the same class of
hardware. Cell lists, Verlet lists, and periodic spatial reordering are
the standard machinery, and they are exactly the occupancy index and the
spatial re-sort that the record proposes. The field has published
guidance on how often to reorder, on how reordering interacts with
reproducibility, and on how to keep a reduction deterministic under
domain decomposition.[^71][^72][^73]

The record asks whether to re-sort the arena each tick and marks it open.
This field answered that question a long time ago.

### A3. Incremental view maintenance in databases

Named in area 2.6 and repeated here because it is the largest single gap.
The dirty pyramid is a materialised view. The rule that a summary must be
a group with an inverse is the standard maintenance condition. The record
derives a fragment of the theory by hand and states it as a project rule.
The field states it as an algebra, proves which queries have an
incremental form, and names the cases where incremental maintenance loses
to recomputation.[^41][^42][^43][^44]

### A4. Electronic design automation

Chip design tools maintain very large graphs under continuous local edit,
and they must report an exact answer after every edit. Incremental static
timing analysis is the closest analogue to the dirty pyramid: a local
change propagates through a hierarchy, and the tool recomputes only the
affected cone. Placement and legalisation are large discrete optimisation
problems under geometric constraints, which is the shape of the placement
problem in `spawn` and `build`. The field also has a long history of
deterministic tool behaviour as a hard requirement.

### A5. Geographic information systems

Raster pyramids, tiled storage, overviews, and cloud-optimised formats
are the established answer to "serve a hierarchical summary of a very
large grid". Spatial join algorithms are the established answer to the
tile-to-unit bridge. The record rejects H3 for good reasons, but it
should take the pyramid and tiling conventions, which are independent of
the cell shape.[^176]

### A6. In-situ scientific visualisation

The field exists because a simulation produces more state than can be
moved to a visualiser. That is Cachette's problem for both rendering and
research telemetry. The standard answer is to compute the reduction
inside the simulation and to move only the reduction. Cachette already
computes exactly the right reduction, which is the pyramid. What it
lacks is the transport and the policy.[^173]

### A7. Financial market simulation

Limit order book simulators run many agents in a strictly ordered
discrete event loop with an auditable log, and they must replay exactly
for regulatory reasons. The engineering pattern of a sealed input, a
deterministic matching step, and an append-only event log is the same
pattern the record adopts. The field also has strong practice on
sequencing, on stable total orders, and on replay from a log.

### A8. Compilers and incremental computation

The selector expression tree is an intermediate representation. A future
expression language needs rewriting, common subexpression elimination,
and a cost model. Equality saturation with an e-graph is the current
method for rewriting under a cost model without a phase ordering
problem, and it has a mature Rust implementation.[^183] Incremental
compilation frameworks are the practical form of demand-driven
recomputation.[^47]

### A9. Stream processing

Windowed aggregation over an unbounded stream is the same algebra as the
pyramid, in the time dimension rather than the space dimension. The field
formalised the combiner interface, the requirement for associativity, and
the treatment of aggregates that lack an inverse. It also formalised
watermarks and out-of-order arrival, which Cachette does not need. Take
the combiner algebra and leave the rest.

### A10. High-performance computing communication patterns

Halo exchange, ghost cells, and domain decomposition are how stencil
codes handle a boundary between two workers. Cachette's parallel splits
have exactly this boundary problem at every block edge, and the record
handles it by constraining the split rather than by exchanging halos.
That is the right choice for a single node. The field's vocabulary is
still the right vocabulary for describing it.

---

## A suggested order

Three principles set the order. Study first what a recorded decision
depends on. Study second what a benchmark cannot proceed without. Study
last what only matters after version 1 exists.

### Phase 1 — before the first commit

These three studies each block or endanger a decision that the record
calls unretrofittable.

1. **Incremental view maintenance, plus data skipping and zone maps.**
   One study, because the two halves answer the same subsystem. It tests
   decision D16 on aggregation, decision D17 on the two pyramids and the
   fallback, and decision D18 on the field registry. It also gives the
   verdict distribution instrumentation that unknown U1 needs. Nothing
   else in the record has as many decisions hanging on it.

2. **Deterministic parallel primitives, and a parallel stable radix
   sort.** It tests decisions D19, D22, D27 and D50, and it produces the
   sort that is on the critical path of five subsystems. It also gives
   the vocabulary to close unknown U3. Do it second, because the
   determinism test in continuous integration is a week-one item and it
   needs this vocabulary to be written correctly.

3. **Fixed-point numerical methods, and integer eikonal and diffusion
   solvers.** It tests decision D4 on the value types, decision D44 on the
   flow tiles, and decision D45 on steering. It scopes the real cost of
   the no-float rule, which is currently an unsourced estimate. It closes
   unknown U4. Coordinate with the crowd and movement report and the
   influence map report, which are both in preparation.

### Phase 2 — before the vertical slice ships

4. **Benchmarking methodology on Graviton.** Every later study needs a
   believable measurement, and no measurement taken on a laptop is
   believable for this design. Set up the pinned runner before the
   benchmarks, not after.

5. **Data-oriented design and the arena benchmark.** The archetype
   question is answered, so the benchmark no longer decides it. The
   benchmark now sizes the chunk and the span for the four fixed
   shapes.[^ARCH]

6. **Reinforcement learning environments and batched stepping.** It
   decides `WorldBatch`, which the record says constrains the whole
   world interface and must be decided early. It also closes unknown U5.

7. **Property-based and metamorphic testing.** It converts the record's
   list of invariants into an executable test suite.

### Phase 3 — while the vertical slice widens

8. **Sparse volumetric data structures.** It informs the unified tile set
   that report 08 proposes, and the leaf threshold that report 08 leaves
   open.

9. **Bitmap indexes and bit-sliced indexes.** It settles open question 16
   and may replace the min-max summary pair with a better form.

10. **Molecular dynamics neighbour handling and spatial sorting.** It
    settles open question 14.

11. **Hardware, cache behaviour, and the Neoverse top-down method.** It
    re-derives the wall-time budget on the target.

12. **Procedural generation.** The dogfooding milestone needs a world.

### Phase 4 — after version 1

13. Game artificial intelligence at scale, and where decisions happen.
14. Operations research: flow, assignment, and apportionment.
15. Agent-based modelling conventions and reporting standards.
16. Visualisation transport and level-of-detail policy.
17. Event stores and log-structured storage, when the log is undeferred.
18. Distributed simulation and interest management, when netcode is
    undeferred.
19. Out-of-core state, if the map size grows.

### Dependencies worth naming

- Study 4, the benchmarking setup, blocks any conclusion in studies 1, 2,
  5, 9, 10 and 11. Do it early even though it appears in phase 2.
- Study 2, deterministic parallelism, blocks study 10, because the
  spatial re-sort question is a determinism question first and a
  performance question second.
- Study 1, incremental view maintenance, blocks study 9, because the
  choice of summary representation follows from the maintenance
  condition, not the other way round.
- Study 6, reinforcement learning environments, blocks nothing
  technically, but it constrains the public interface. A late answer
  means a breaking change.

---

## References

[^1]: ADR-0001, Foundational Architecture, and its eight background documents. `docs/adrs/REGISTRY.md`, `docs/research/reports/`
[^2]: Cachette project instructions, "Hard invariants" and "Design principles". `CLAUDE.md`
[^3]: Mike Acton, "Data-Oriented Design and C++", CppCon 2014. https://www.youtube.com/watch?v=rX0ItVEVjHc
[^4]: Richard Fabian, *Data-Oriented Design*, 2018. https://www.dataorienteddesign.com/dodbook/
[^5]: Unity Entities, archetype concepts. https://docs.unity3d.com/Packages/com.unity.entities@1.0/manual/concepts-archetypes.html
[^6]: Flecs, frequently asked questions and query documentation. https://www.flecs.dev/flecs/md_docs_2FAQ.html
[^7]: bevy_ecs storage documentation. https://docs.rs/bevy/latest/bevy/ecs/storage/struct.Table.html
[^8]: Staffordshire University, archetype against sparse set comparison, CGVC 2025. https://eprints.staffs.ac.uk/9315/1/cgvc20251224.pdf
[^9]: hecs and legion. https://crates.io/crates/hecs and https://docs.rs/crate/legion/latest
[^10]: bytemuck and zerocopy. https://docs.rs/bytemuck and https://docs.rs/zerocopy
[^11]: crossbeam-utils `CachePadded`. https://docs.rs/crossbeam-utils/latest/crossbeam_utils/struct.CachePadded.html
[^12]: Catherine West, "Using Rust For Game Development", RustConf 2018 closing keynote. https://www.youtube.com/watch?v=aKLntZcp27M
[^13]: Bevy pull request 18704, non-max entity index. https://github.com/bevyengine/bevy/pull/18704
[^14]: slotmap and generational-arena. https://docs.rs/slotmap and https://docs.rs/generational-arena
[^15]: nonmax. https://docs.rs/nonmax
[^16]: Ulrich Drepper, "What Every Programmer Should Know About Memory", 2007. https://people.freebsd.org/~lstewart/articles/cpumemory.pdf
[^17]: Linux kernel, transparent hugepage support. https://docs.kernel.org/admin-guide/mm/transhuge.html
[^18]: Aleksey Shipilev, "JVM Anatomy Quark #2: Transparent Huge Pages". https://shipilev.net/jvm/anatomy-quarks/2-transparent-huge-pages/
[^19]: bumpalo and typed-arena. https://docs.rs/bumpalo and https://docs.rs/typed-arena
[^20]: Guido Moerkotte, "Small Materialized Aggregates: A Light Weight Index Structure for Data Warehousing", VLDB 1998. https://www.vldb.org/conf/1998/p476.pdf
[^21]: PostgreSQL, block range index documentation. https://www.postgresql.org/docs/current/brin.html
[^22]: Benoit Dageville et al., "The Snowflake Elastic Data Warehouse", SIGMOD 2016. https://dl.acm.org/doi/10.1145/2882903.2903741
[^23]: Mark Raasveldt and Hannes Mühleisen, "DuckDB: an Embeddable Analytical Database", SIGMOD 2019. https://duckdb.org/pdf/SIGMOD2019-demo-duckdb.pdf
[^24]: ClickHouse MergeTree engine documentation. https://clickhouse.com/docs/en/engines/table-engines/mergetree-family/mergetree
[^25]: Patricia Selinger et al., "Access Path Selection in a Relational Database Management System", SIGMOD 1979. https://dl.acm.org/doi/10.1145/582095.582099
[^26]: Viktor Leis et al., "How Good Are Query Optimizers, Really?", VLDB 2015. https://www.vldb.org/pvldb/vol9/p204-leis.pdf
[^27]: Goetz Graefe, "The Cascades Framework for Query Optimization", 1995, and "Volcano: An Extensible and Parallel Query Evaluation System", 1994. https://ieeexplore.ieee.org/document/273032
[^28]: Apache Calcite. https://calcite.apache.org/
[^29]: Peter Boncz, Marcin Zukowski and Niels Nes, "MonetDB/X100: Hyper-Pipelining Query Execution", CIDR 2005. https://www.cidrdb.org/cidr2005/papers/P19.pdf
[^30]: Timo Kersten et al., "Everything You Always Wanted to Know About Compiled and Vectorized Queries But Were Afraid to Ask", VLDB 2018. https://www.vldb.org/pvldb/vol11/p2209-kersten.pdf
[^31]: Thomas Neumann and Michael Freitag, "Umbra: A Disk-Based System with In-Memory Performance", CIDR 2020. https://www.cidrdb.org/cidr2020/papers/p29-neumann-cidr20.pdf
[^32]: Apache Arrow columnar format and C data interface. https://arrow.apache.org/docs/format/Columnar.html
[^33]: Apache DataFusion. https://datafusion.apache.org/
[^34]: Polars, lazy application programming interface. https://docs.pola.rs/user-guide/concepts/lazy-api/
[^35]: Pedro Pedreira et al., "Velox: Meta's Unified Execution Engine", VLDB 2022. https://www.vldb.org/pvldb/vol15/p3372-pedreira.pdf
[^36]: Viktor Leis et al., "Morsel-Driven Parallelism", SIGMOD 2014. https://db.in.tum.de/~leis/papers/morsels.pdf
[^37]: Samy Chambi, Daniel Lemire et al., "Better Bitmap Performance with Roaring Bitmaps", 2014, and follow-on papers. https://arxiv.org/pdf/1402.6407
[^38]: Patrick O'Neil and Dallan Quass, "Improved Query Performance with Variant Indexes", SIGMOD 1997. https://dl.acm.org/doi/10.1145/253260.253268
[^39]: roaring crate. https://docs.rs/roaring
[^40]: CRoaring. https://github.com/RoaringBitmap/CRoaring
[^41]: Ashish Gupta and Inderpal Singh Mumick, "Maintenance of Materialized Views: Problems, Techniques, and Applications", 1995. https://sites.cs.ucsb.edu/~tyang/class/240a17/slides/mv-gupta.pdf
[^42]: Frank McSherry, Derek Murray, Rebecca Isaacs and Michael Isard, "Differential Dataflow", CIDR 2013. https://www.cidrdb.org/cidr2013/Papers/CIDR13_Paper111.pdf
[^43]: Mihai Budiu et al., "DBSP: Automatic Incremental View Maintenance for Rich Query Languages", VLDB 2023. https://arxiv.org/abs/2203.16684
[^44]: Jon Gjengset et al., "Noria: dynamic, partially-stateful data-flow for high-performance web applications", OSDI 2018. https://www.usenix.org/system/files/osdi18-gjengset.pdf
[^45]: timely-dataflow and differential-dataflow. https://github.com/TimelyDataflow/differential-dataflow
[^46]: Umut Acar, "Self-Adjusting Computation", PhD thesis, Carnegie Mellon University, 2005. https://www.cs.cmu.edu/~rwh/students/acar.pdf
[^47]: salsa. https://github.com/salsa-rs/salsa
[^48]: Guy Jacobson, "Space-efficient Static Trees and Graphs", FOCS 1989. https://ieeexplore.ieee.org/document/63533
[^49]: Gonzalo Navarro, *Compact Data Structures: A Practical Approach*, Cambridge University Press, 2016. https://users.dcc.uchile.cl/~gnavarro/CDSbook/
[^50]: sdsl-lite. https://github.com/simongog/sdsl-lite
[^51]: sucds, vers and bitm. https://docs.rs/sucds and https://docs.rs/vers-vecs and https://docs.rs/bitm
[^52]: Wojciech Muła, Nathan Kurz and Daniel Lemire, "Faster Population Counts Using AVX2 Instructions", 2017. https://arxiv.org/abs/1611.07612
[^53]: Sebastiano Vigna, "Quasi-Succinct Indices", WSDM 2013. https://arxiv.org/abs/1206.4300
[^54]: Daniel Lemire et al., "Stream VByte: Faster Byte-Oriented Integer Compression", 2017, and "SIMD Compression and the Intersection of Sorted Integers", 2014. https://arxiv.org/abs/1709.08990
[^55]: stream-vbyte and bitpacking crates. https://docs.rs/stream-vbyte and https://docs.rs/bitpacking
[^56]: Roberto Grossi, Ankur Gupta and Jeffrey Scott Vitter, "High-Order Entropy-Compressed Text Indexes", SODA 2003. https://dl.acm.org/doi/10.5555/644108.644250
[^57]: Phil Bagwell, "Ideal Hash Trees", 2001. https://lampwww.epfl.ch/papers/idealhashtrees.pdf
[^58]: Phil Bagwell and Tiark Rompf, "RRB-Trees: Efficient Immutable Vectors", 2011. https://infoscience.epfl.ch/record/169879/files/RMTrees.pdf
[^59]: Ohad Rodeh, "B-trees, Shadowing, and Clones", ACM Transactions on Storage, 2008. https://dl.acm.org/doi/10.1145/1326542.1326544
[^60]: im and rpds crates. https://docs.rs/im and https://docs.rs/rpds
[^61]: Amit Patel, "Hexagonal Grids", Red Blob Games. https://www.redblobgames.com/grids/hexagons/
[^62]: Kevin Sahr, Denis White and A. Jon Kimerling, "Geodesic Discrete Global Grid Systems", 2003. https://www.tandfonline.com/doi/abs/10.1559/152304003100011090
[^63]: "Designing aperture 2^2k hexagonal grids and their indexing as factor rings of Eisenstein integers", Theoretical Computer Science, 2023. https://www.sciencedirect.com/science/article/pii/S0304397523005704
[^64]: Uber H3. https://h3geo.org/docs/highlights/indexing
[^65]: Google S2 geometry. https://s2geometry.io/
[^66]: hexx crate. https://docs.rs/hexx
[^67]: Ken Museth, "VDB: High-Resolution Sparse Volumes with Dynamic Topology", ACM Transactions on Graphics, 2013. https://www.museth.org/Ken/Publications_files/Museth_TOG13.pdf
[^68]: Ken Museth, "NanoVDB: A GPU-Friendly and Portable VDB Data Structure", 2021. https://dl.acm.org/doi/10.1145/3450623.3464653
[^69]: Samuli Laine and Tero Karras, "Efficient Sparse Voxel Octrees", I3D 2010. https://research.nvidia.com/publication/2010-02_efficient-sparse-voxel-octrees
[^70]: OpenVDB. https://www.openvdb.org/
[^71]: Loup Verlet, "Computer Experiments on Classical Fluids", Physical Review, 1967. https://journals.aps.org/pr/abstract/10.1103/PhysRev.159.98
[^72]: Michael Allen and Dominic Tildesley, *Computer Simulation of Liquids*, second edition, Oxford University Press, 2017. https://global.oup.com/academic/product/computer-simulation-of-liquids-9780198803201
[^73]: LAMMPS, GROMACS and HOOMD-blue. https://www.lammps.org/ and https://www.gromacs.org/ and https://hoomd-blue.readthedocs.io/
[^74]: Jonathan Shewchuk, "Adaptive Precision Floating-Point Arithmetic and Fast Robust Geometric Predicates", 1997. https://www.cs.cmu.edu/~quake/robust.html
[^75]: Bongki Moon et al., "Analysis of the Clustering Properties of the Hilbert Space-Filling Curve", IEEE Transactions on Knowledge and Data Engineering, 2001. https://ieeexplore.ieee.org/document/908985
[^76]: Guy Blelloch, Jeremy Fineman, Phillip Gibbons and Julian Shun, "Internally Deterministic Parallel Algorithms Can Be Fast", PPoPP 2012. https://www.cs.cmu.edu/~guyb/papers/BFGS12.pdf
[^77]: Matteo Frigo et al., "Reducers and Other Cilk++ Hyperobjects", SPAA 2009. https://dl.acm.org/doi/10.1145/1583991.1584017
[^78]: Robert Blumofe and Charles Leiserson, "Scheduling Multithreaded Computations by Work Stealing", Journal of the ACM, 1999. https://dl.acm.org/doi/10.1145/324133.324234
[^79]: rayon. https://docs.rs/rayon
[^80]: Guy Blelloch, "Prefix Sums and Their Applications", 1990. https://www.cs.cmu.edu/~guyb/papers/Ble93.pdf
[^81]: Duane Merrill and Andrew Grimshaw, "High Performance and Scalable Radix Sorting", 2011. https://research.nvidia.com/publication/high-performance-and-scalable-radix-sorting-gpu
[^82]: Omar Obeya et al., "Theoretically-Efficient and Practical Parallel In-Place Radix Sorting", SPAA 2019. https://dl.acm.org/doi/10.1145/3323165.3323198
[^83]: rdst and voracious_radix_sort. https://docs.rs/rdst and https://docs.rs/voracious_radix_sort
[^84]: METIS graph partitioning. https://github.com/KarypisLab/METIS
[^85]: James Sethian, "A Fast Marching Level Set Method for Monotonically Advancing Fronts", PNAS 1996, and John Tsitsiklis, "Efficient Algorithms for Globally Optimal Trajectories", 1995. https://www.pnas.org/doi/10.1073/pnas.93.4.1591
[^86]: Hongkai Zhao, "A Fast Sweeping Method for Eikonal Equations", Mathematics of Computation, 2005. https://www.ams.org/journals/mcom/2005-74-250/S0025-5718-04-01678-3/
[^87]: Robert Dial, "Algorithm 360: Shortest-Path Forest with Topological Ordering", Communications of the ACM, 1969. https://dl.acm.org/doi/10.1145/363269.363610
[^88]: Achi Brandt, "Multi-Level Adaptive Solutions to Boundary-Value Problems", Mathematics of Computation, 1977. https://www.ams.org/journals/mcom/1977-31-138/S0025-5718-1977-0431719-X/
[^89]: William Briggs, Van Emden Henson and Steve McCormick, *A Multigrid Tutorial*, second edition, SIAM, 2000. https://epubs.siam.org/doi/book/10.1137/1.9780898719505
[^90]: Nicholas Higham, *Accuracy and Stability of Numerical Algorithms*, second edition, SIAM, 2002. https://epubs.siam.org/doi/book/10.1137/1.9780898718027
[^91]: fixed crate. https://docs.rs/fixed
[^92]: libm crate. https://docs.rs/libm
[^93]: Kaushik Datta et al., "Stencil Computation Optimization and Auto-tuning on State-of-the-Art Multicore Architectures", SC 2008. https://dl.acm.org/doi/10.5555/1413370.1413375
[^94]: Samuel Williams, Andrew Waterman and David Patterson, "Roofline: An Insightful Visual Performance Model", Communications of the ACM, 2009. https://dl.acm.org/doi/10.1145/1498765.1498785
[^95]: Ravindra Ahuja, Thomas Magnanti and James Orlin, *Network Flows: Theory, Algorithms, and Applications*, Prentice Hall, 1993. https://dspace.mit.edu/handle/1721.1/49424
[^96]: Andrew Goldberg and Robert Tarjan, "A New Approach to the Maximum-Flow Problem", Journal of the ACM, 1988. https://dl.acm.org/doi/10.1145/48014.61051
[^97]: pathfinding and petgraph crates. https://docs.rs/pathfinding and https://docs.rs/petgraph
[^98]: Harold Kuhn, "The Hungarian Method for the Assignment Problem", Naval Research Logistics Quarterly, 1955. https://onlinelibrary.wiley.com/doi/10.1002/nav.3800020109
[^99]: Dimitri Bertsekas, "The Auction Algorithm: A Distributed Relaxation Method for the Assignment Problem", Annals of Operations Research, 1988. https://link.springer.com/article/10.1007/BF02186476
[^100]: Michel Balinski and Peyton Young, *Fair Representation: Meeting the Ideal of One Man, One Vote*, Yale University Press, 1982. https://yalebooks.yale.edu/book/9780300027242/fair-representation/
[^101]: Google OR-Tools CP-SAT solver. https://developers.google.com/optimization/cp/cp_solver
[^102]: good_lp crate. https://docs.rs/good_lp
[^103]: Roni Stern et al., "Multi-Agent Pathfinding: Definitions, Variants, and Benchmarks", SoCS 2019. https://arxiv.org/abs/1906.08291
[^104]: Guni Sharon et al., "Conflict-Based Search for Optimal Multi-Agent Pathfinding", Artificial Intelligence, 2015. https://www.sciencedirect.com/science/article/pii/S0004370214001386
[^105]: Joshua Epstein and Robert Axtell, *Growing Artificial Societies*, MIT Press, 1996. https://mitpress.mit.edu/9780262550253/growing-artificial-societies/
[^106]: Eric Bonabeau, "Agent-based modeling: Methods and techniques for simulating human systems", PNAS 2002. https://www.pnas.org/doi/10.1073/pnas.082080899
[^107]: Volker Grimm et al., "The ODD protocol for describing agent-based and other simulation models: A second update", JASSS 2020. https://www.jasss.org/23/2/7.html
[^108]: OpenABM-Covid19, an agent-based model at population scale. https://github.com/BDI-pathogens/OpenABM-Covid19
[^109]: Nicholson Collier and Michael North, "Parallel agent-based simulation with Repast for High Performance Computing", Simulation, 2013. https://repast.github.io/repast_hpc.html
[^110]: FLAME GPU. https://flamegpu.com/
[^111]: Mesa, NetLogo and MASON. https://mesa.readthedocs.io/ and https://ccl.northwestern.edu/netlogo/ and https://cs.gmu.edu/~eclab/projects/mason/
[^112]: George Datseris et al., "Agents.jl: a performant and feature-full agent-based modeling software", Simulation, 2022. https://juliadynamics.github.io/Agents.jl/stable/
[^113]: krABMaga. https://krabmaga.github.io/
[^114]: Jiayi Weng et al., "EnvPool: A Highly Parallel Reinforcement Learning Environment Execution Engine", NeurIPS 2022. https://arxiv.org/abs/2206.10558
[^115]: Brennan Shacklett et al., "An Extensible, Data-Oriented Architecture for High-Performance, Many-World Simulation", SIGGRAPH 2023. https://madrona-engine.github.io/shacklett_siggraph23.pdf
[^116]: Joseph Suarez et al., "Neural MMO: A Massively Multiagent Game Environment". https://arxiv.org/abs/1903.00784
[^117]: Gymnasium and PettingZoo, Farama Foundation. https://gymnasium.farama.org/ and https://pettingzoo.farama.org/
[^118]: PufferLib. https://github.com/PufferAI/PufferLib
[^119]: DLPack and the Python array application programming interface standard. https://dmlc.github.io/dlpack/latest/ and https://data-apis.org/array-api/latest/
[^120]: Rust RFC 3514, Float Semantics. https://rust-lang.github.io/rfcs/3514-float-semantics.html
[^121]: IEEE 754-2019, standard for floating-point arithmetic. https://ieeexplore.ieee.org/document/8766229
[^122]: David Goldberg, "What Every Computer Scientist Should Know About Floating-Point Arithmetic", ACM Computing Surveys, 1991. https://dl.acm.org/doi/10.1145/103162.103163
[^123]: John Salmon et al., "Parallel Random Numbers: As Easy as 1, 2, 3", SC 2011. https://www.thesalmons.org/john/random123/papers/random123sc11.pdf
[^124]: Koen Claessen and John Hughes, "QuickCheck: A Lightweight Tool for Random Testing of Haskell Programs", ICFP 2000. https://dl.acm.org/doi/10.1145/351240.351266
[^125]: Sergio Segura et al., "A Survey on Metamorphic Testing", IEEE Transactions on Software Engineering, 2016. https://ieeexplore.ieee.org/document/7422146
[^126]: Hypothesis, stateful testing. https://hypothesis.readthedocs.io/en/latest/stateful.html
[^127]: proptest crate. https://docs.rs/proptest
[^128]: FoundationDB simulation testing. https://apple.github.io/foundationdb/testing.html
[^129]: TigerBeetle deterministic simulation testing, and Antithesis. https://tigerbeetle.com/blog/2023-03-28-random-fuzzy-thoughts/ and https://antithesis.com/
[^130]: Leslie Lamport, TLA+ specification language. https://lamport.azurewebsites.net/tla/tla.html
[^131]: Ralf Jung et al., "Stacked Borrows: An Aliasing Model for Rust", POPL 2020. https://plv.mpi-sws.org/rustbelt/stacked-borrows/paper.pdf
[^132]: Miri, loom and kani. https://github.com/rust-lang/miri and https://docs.rs/loom and https://model-checking.github.io/kani/
[^133]: Viktor Leis et al., "LeanStore: In-Memory Data Management Beyond Main Memory", ICDE 2018. https://db.in.tum.de/~leis/papers/leanstore.pdf
[^134]: Umbra buffer manager. https://umbra-db.com/
[^135]: Tuomas Pelkonen et al., "Gorilla: A Fast, Scalable, In-Memory Time Series Database", VLDB 2015. https://www.vldb.org/pvldb/vol8/p1816-teller.pdf
[^136]: Zstandard. https://facebook.github.io/zstd/
[^137]: io_uring. https://kernel.dk/io_uring.pdf
[^138]: wide crate. https://docs.rs/wide
[^139]: pulp crate. https://docs.rs/pulp
[^140]: Rust portable single instruction multiple data, tracking issue 86656. https://github.com/rust-lang/portable-simd/issues/364
[^141]: cargo-show-asm. https://github.com/pacak/cargo-show-asm
[^142]: cargo-deny, sccache and maturin. https://embarkstudios.github.io/cargo-deny/ and https://github.com/mozilla/sccache and https://www.maturin.rs/
[^143]: Python enhancement proposals 703, 779, 803 and 734. https://peps.python.org/pep-0703/ and https://peps.python.org/pep-0779/ and https://peps.python.org/pep-0803/ and https://peps.python.org/pep-0734/
[^144]: PyO3 and rust-numpy. https://pyo3.rs/ and https://docs.rs/numpy
[^145]: cibuildwheel. https://cibuildwheel.pypa.io/
[^146]: nanobind. https://nanobind.readthedocs.io/
[^147]: Richard Fujimoto, *Parallel and Distributed Simulation Systems*, Wiley, 2000. https://www.wiley.com/en-us/Parallel+and+Distributed+Simulation+Systems-p-9780471183839
[^148]: David Jefferson, "Virtual Time", ACM Transactions on Programming Languages and Systems, 1985. https://dl.acm.org/doi/10.1145/3916.3988
[^149]: K. Mani Chandy and Jayadev Misra, "Distributed Simulation: A Case Study in Design and Verification of Distributed Programs", IEEE Transactions on Software Engineering, 1979. https://ieeexplore.ieee.org/document/1702653
[^150]: IEEE 1516, High Level Architecture, and its data distribution management services. https://standards.ieee.org/ieee/1516/3744/
[^151]: ROSS, Rensselaer's Optimistic Simulation System. https://github.com/ROSS-org/ROSS
[^152]: Jeff Orkin, "Three States and a Plan: The AI of F.E.A.R.", GDC 2006. https://alumni.media.mit.edu/~jorkin/gdc2006_orkin_jeff_fear.pdf
[^153]: Dave Mark, *Behavioral Mathematics for Game AI*, 2009, and the Infinite Axis Utility System talks. https://intrinsicalgorithm.com/IAonAI/
[^154]: Dana Nau et al., "SHOP2: An HTN Planning System", Journal of Artificial Intelligence Research, 2003. https://www.jair.org/index.php/jair/article/view/10362
[^155]: Michele Colledanchise and Petter Ögren, *Behavior Trees in Robotics and AI*, CRC Press, 2018. https://arxiv.org/abs/1709.00084
[^156]: Paul Tozour, "Influence Mapping", in *Game Programming Gems 2*, 2001. https://www.gameaipro.com/
[^157]: Ken Perlin, "An Image Synthesizer", SIGGRAPH 1985. https://dl.acm.org/doi/10.1145/325165.325247
[^158]: Isaac Karth and Adam Smith, "WaveFunctionCollapse is Constraint Solving in the Wild", FDG 2017, and Maxim Gumin's implementation. https://github.com/mxgmn/WaveFunctionCollapse
[^159]: Amit Patel, "Polygonal Map Generation for Games", Red Blob Games. https://www-cs-students.stanford.edu/~amitp/game-programming/polygon-map-generation/
[^160]: Noor Shaker, Julian Togelius and Mark Nelson, *Procedural Content Generation in Games*, Springer, 2016. https://www.pcgbook.com/
[^161]: F. Kenton Musgrave, Craig Kolb and Robert Mace, "The Synthesis and Rendering of Eroded Fractal Terrains", SIGGRAPH 1989. https://dl.acm.org/doi/10.1145/74334.74337
[^162]: noise-rs and fastnoise-lite. https://docs.rs/noise and https://github.com/Auburn/FastNoiseLite
[^163]: Patrick O'Neil et al., "The Log-Structured Merge-Tree", Acta Informatica, 1996. https://www.cs.umb.edu/~poneil/lsmtree.pdf
[^164]: Jay Kreps, "The Log: What every software engineer should know about real-time data's unifying abstraction", 2013. https://engineering.linkedin.com/distributed-systems/log-what-every-software-engineer-should-know-about-real-time-datas-unifying
[^165]: Martin Fowler, "Event Sourcing" and "CQRS". https://martinfowler.com/eaaDev/EventSourcing.html
[^166]: redb and sled. https://docs.rs/redb and https://docs.rs/sled
[^167]: Apache Parquet encodings. https://parquet.apache.org/docs/file-format/data-pages/encodings/
[^168]: Arm Neoverse N1 and V2 software optimization guides. https://developer.arm.com/documentation/swog309707/latest
[^169]: AWS Graviton getting-started technical guide. https://github.com/aws/aws-graviton-getting-started
[^170]: Ahmad Yasin, "A Top-Down Method for Performance Analysis and Counters Architecture", ISPASS 2014, and the Arm Neoverse topdown methodology. https://ieeexplore.ieee.org/document/6844459
[^171]: Arm Statistical Profiling Extension. https://developer.arm.com/documentation/ddi0487/latest/
[^172]: Thatcher Ulrich, "Rendering Massive Terrains using Chunked Level of Detail Control", SIGGRAPH 2002, and Frank Losasso and Hugues Hoppe, "Geometry Clipmaps", SIGGRAPH 2004. https://hhoppe.com/geomclipmap.pdf
[^173]: Utkarsh Ayachit et al., "ParaView Catalyst: Enabling In Situ Data Analysis and Visualization", 2015, and the Ascent in-situ library. https://ascent.readthedocs.io/
[^174]: Datashader. https://datashader.org/
[^175]: deck.gl and the Mapbox vector tile specification. https://deck.gl/ and https://github.com/mapbox/vector-tile-spec
[^176]: Cloud Optimized GeoTIFF. https://www.cogeo.org/
[^177]: John Ousterhout, "Always Measure One Level Deeper", Communications of the ACM, 2018. https://dl.acm.org/doi/10.1145/3213770
[^178]: Charlie Curtsinger and Emery Berger, "Coz: Finding Code that Counts with Causal Profiling", SOSP 2015. https://arxiv.org/abs/1608.03676
[^179]: Tomas Kalibera and Richard Jones, "Rigorous Benchmarking in Reasonable Time", ISMM 2013. https://kar.kent.ac.uk/33611/
[^180]: criterion and divan. https://docs.rs/criterion and https://docs.rs/divan
[^181]: tracing and Tracy. https://docs.rs/tracing and https://github.com/wolfpld/tracy
[^182]: py-spy. https://github.com/benfred/py-spy
[^183]: Max Willsey et al., "egg: Fast and Extensible Equality Saturation", POPL 2021. https://arxiv.org/abs/2004.03082
[^ARCH]: ADR-0066, entity storage holds four fixed shapes. `docs/adrs/draft/adr-0066-entity-storage-holds-four-fixed-shapes.md`
