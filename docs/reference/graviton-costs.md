# Target Platform Costs

This document is a **register**. It holds every figure that this project has
measured on the target platform.

The target platform is AWS Graviton, and the primary target triple is
`aarch64-unknown-linux-gnu`.[^1] A second register holds the scale constants
and the figures the project derived.[^2] A third register holds the one figure
the project keeps about a development machine.[^3] A figure never moves
between the three. Each register states the machine that produced its rows.

**A figure here is a measurement. Say so when you cite one.** Most cost
figures in this project are still derived, and one blocker states which.[^4]

## Status

Measured figures exist. A benchmark lives in the repository and a script runs
it on the target platform.[^5] [^6]

A third run named every stage of a frame and measured what huge pages are
worth. Two sections hold it, and it comes from one instance on one commit.

The figures below come from two runs on one date, on two instances of
different size. They cover five quantities of the public crate interface: the
cost of a frame, the cost of building a world, the cost of the whole-world
hash, the resident memory of a world, and whether a frame at the target scale
trips an integer overflow check. They do not cover the passes that a world
with settlements runs, because the measured world holds none.

**The two runs agree.** Every row the two instances share falls within 1.4
percent of the other. The build at the target extent differs by one part in
eight hundred and the hash by one part in two thousand. Two machines that
agree to that degree are measuring the engine and not the machine.

## How to take a figure

Run the script. It launches an instance, copies the tracked files to it,
builds the benchmark, runs the sweep, brings the rows back, and destroys
everything it made.

```
just graviton-bench full
just graviton-orphans
```

Two profiles take the figures the later sections hold. The first names every
stage of a frame. The second runs the same point under each huge page setting,
in a process for each.

```
CACHETTE_BENCH_INSTANCE=c7g.4xlarge CACHETTE_BENCH_FEATURES=stage-cost \
  CACHETTE_BENCH_POINT="stage-cost 4096x4096 1000000 12 scattered" \
  ./scripts/graviton-benchmark.sh stage-cost

CACHETTE_BENCH_INSTANCE=c7g.4xlarge CACHETTE_BENCH_FEATURES=stage-cost \
  CACHETTE_BENCH_THP="never madvise always" \
  CACHETTE_BENCH_POINT="stage-cost 4096x4096 1000000 12 scattered" \
  ./scripts/graviton-benchmark.sh hugepages
```

The stage table is behind a crate feature and it is off by default. A run
without the feature reports zeros and says so in its own preamble, so a reader
cannot mistake a build that does not measure for a frame that cost nothing.

The script needs the AWS command line tool, authenticated. Every axis is a
parameter: the instance type, the extents, the thread counts and the unit
counts each read an environment variable, and the script header lists them. A
run on a larger machine is a setting and not a change to a file.

The two runs below took about seven and about ten minutes, and the build was
most of both. Together they cost about twelve cents. The second command lists
what a run left behind, and it must list nothing.

The benchmark also runs on a development machine, and a figure taken there
belongs in neither this register nor any other.[^7] Use it to check the
apparatus.

```
just bench quick
```

## The machines

Two machines produced the figures below. Every table names which one.

| Fact | Machine A | Machine B |
|---|---|---|
| Instance type | `c7g.4xlarge` | `c7g.large` |
| Region | `us-west-2` | `us-west-2` |
| Processor | Graviton3. Implementer `0x41`, part `0xd40` | The same |
| Hardware threads | 16 | 2 |
| Cache line | 64 bytes | 64 bytes |
| Memory | 32,246,808 kB | 3,897,492 kB |
| Kernel | `Linux 6.18.44-99.149.amzn2023.aarch64` | The same |
| Compiler | `rustc 1.91.1 (ed61e7d7e 2025-11-07)` | The same |
| Build profile | `bench`, which inherits `release` | The same |
| Commit | `6b845be102134c21b31f762fda3cee705cbeeb2b` | `9aaf6443f80ebfdb35348d679d7c23604bb3807b` |
| Run time | About 10 minutes | About 7 minutes |
| Date | 3 September 2026 | 3 September 2026 |

**Machine A is the one to cite.** It holds 16 hardware threads, so it runs
every thread count the determinism tests use, and those are 1, 2 and 12.[^8]
Machine B holds two, so it cannot run the top count at all, and that is a fact
about the machine rather than about the engine.

**Neither instance is burstable.** A burstable instance earns processor
credits and falls back to a fraction of one core when they run out. A sweep of
this length would exhaust them, and every row after that point would measure
the throttle rather than the engine.

## What each row measures

**`step_by_tiles`** runs one frame on a world that holds no unit. The step
scans every tile on every frame, so this row is the tile pass alone.

**`step_by_units`** runs one frame on a world of a fixed extent, against a
rising unit count. The choice pass, the movement pass and admission all walk
the live units.

**`step_at_target_scale`** runs one frame at 16,777,216 tiles and 1,000,000
units. Those are the two figures the scale constants table names.[^2]

**`build`** builds a world and drops it. The reservation is 1024 unit slots,
so the tile count is the only quantity that moves across those rows.

**`build_at_target_reservation`** builds a world that reserves 1,000,000 unit
slots, which is the reservation a world takes by default.

**`state_hash`** takes the hash of the whole world. The golden state test
compares this value against a stored file, and the determinism rule of this
project rests on it.[^9]

## One frame at the target scale

This is the row the project needs most. Machine A, 16 hardware threads. The
world held 16,777,216 tiles and 1,000,000 units, and no settlement.

**The units are packed.** A section below shows that a unit costs about twice
as much at the density the project describes, so every row here is a lower
bound.

| Threads | Samples | Minimum, ns | Median, ns | Maximum, ns | Speedup |
|---|---|---|---|---|---|
| 1 | 6 | 1,827,238,759 | 1,848,231,346 | 1,945,931,865 | 1.00 |
| 2 | 9 | 1,107,133,264 | 1,120,190,816 | 1,159,728,596 | 1.65 |
| 4 | 9 | 751,737,651 | 767,062,531 | 790,873,646 | 2.41 |
| 12 | 9 | 506,451,963 | 517,347,864 | 536,613,400 | 3.57 |
| 16 | 9 | 479,584,531 | 500,368,433 | 526,893,922 | 3.69 |

**The frame budget is 100 milliseconds, and it is a target the project chose
rather than a figure anybody derived or measured.** No record states a frame
budget. One sentence states that the engine runs at ten ticks for each second,
and the scale constants table takes that rate as an input to derive how long a
simulated day costs in real time.[^10] [^2] The 100 milliseconds is the
reciprocal of the rate, and this document is the first place that writes it
down.

**Read every ratio below as a measurement against a chosen target.** The
engine is not slow in any absolute sense. It does not reach the rate the
project picked, and the project may pick another rate.

**The engine does not meet the budget, and 16 cores do not close the gap.**
The frame costs 500 milliseconds on all 16, which is 5.0 times the budget.
Sixteen cores bought a speedup of 3.69, so the run used 0.23 of the machine.

## The budget is out of reach on any core count

This is the most important consequence of the run, and it does not depend on
the size of the machine.

**Separate the frame into two halves.** The tile pass scales. The unit passes
stop scaling. The table below gives each half at the target scale, on machine
A. The unit half is the difference between a world of 1,000,000 units and a
world of none, at 4,194,304 tiles.

| Threads | Tile pass, ns | Speedup | Unit passes, ns | Speedup |
|---|---|---|---|---|
| 1 | 1,304,351,811 | 1.00 | 561,211,275 | 1.00 |
| 2 | 716,310,130 | 1.82 | 416,075,247 | 1.35 |
| 4 | 444,188,060 | 2.94 | 348,697,567 | 1.61 |
| 12 | 247,703,940 | 5.27 | 297,877,514 | 1.88 |
| 16 | 212,768,859 | 6.13 | 303,443,126 | 1.85 |

**The unit passes reach a floor near 300 milliseconds and stay there.** They
gain nothing between 12 threads and 16. A frame at the target scale therefore
cannot fall below about 300 milliseconds on this engine, whatever the core
count, and the budget is 100 milliseconds. Adding cores does not reach it.
Something must do less work for each unit, or fewer units must do work in a
frame.

**The same result, stated as the thing a reader can decide about.** At 500
milliseconds the engine runs 2 ticks for each second, so a simulated day of
600 ticks costs 5 minutes of real time. The project chose 10 ticks for each
second, which is one minute for a simulated day. At the floor of 300
milliseconds, which no machine beats, a simulated day costs 3 minutes. The
choice in front of the project is to make a unit cost less, or to accept a
simulated day that runs three to five times slower than the rate it picked.

The tile pass has no such floor in the range measured. It reached 6.13 on 16
threads and it was still improving at 12.

**This is a statement about one million units in one frame.** It is not a
statement about a smaller population, and the measured world held no
settlement, so the unit half is a lower bound rather than the whole of it.

## How the units are placed, and why it changes every unit figure

**Every unit figure above and below was taken with the units packed.** The
pattern walks the world from the first tile and puts one unit on each tile
that admits one. At 1,000,000 units on a 4096-wide grid that fills a band
across the top of the map at one unit for each tile, and leaves the rest of
the world empty.

**That is about seventeen times denser than the target scale describes.** One
million units over 16,777,216 tiles is one unit for each seventeen tiles.

A second pattern spreads the units over the whole world at a stride, at the
density the scale constants imply. The table below runs the same frame under
both, in one process, on one machine, from one build, so the difference
between two rows is the placement and nothing else.

16,777,216 tiles, 1,000,000 units, machine A.

| Threads | Packed, ns | Scattered, ns | Scattered over packed |
|---|---|---|---|
| 1 | 1,882,832,969 | 2,667,755,464 | 1.42 |
| 2 | 1,138,817,995 | 1,678,469,907 | 1.47 |
| 4 | 773,662,530 | 1,190,655,363 | 1.54 |
| 12 | 526,225,993 | 835,171,248 | 1.59 |

Take the tile pass out and compare the unit cost alone.

| Threads | Unit cost packed, ms | Unit cost scattered, ms | Ratio |
|---|---|---|---|
| 1 | 578.5 | 1,363.4 | 2.36 |
| 2 | 422.5 | 962.2 | 2.28 |
| 4 | 329.5 | 746.5 | 2.27 |
| 12 | 278.5 | 587.5 | 2.11 |

**A unit costs about twice as much when the population sits at the density the
project describes.** At 12 threads it is 279 ns packed and 587 ns scattered.

**Read every packed figure in this document as a lower bound on the unit
cost.** A frame at the target scale costs 835 milliseconds at 12 threads under
the scattered pattern, which is 8.4 times the budget, against 526 and 5.3
times under the packed one.

**The shape of the thread scaling survives the change.** The unit passes reach
2.08 times on 12 threads under the packed pattern and 2.32 under the
scattered one. Both scale badly, so the conclusion that the budget is out of
reach does not depend on the placement. The level does.

**The memory does not move.** At 12 threads a packed world holds 875,782,144
bytes and a scattered one holds 875,999,232, which is one part in four
thousand.

**The order of the unit arena does not move between these two rows, and a
reader has taken it as though it did.** Both patterns spawn in ascending tile
order, so the arena carries the units in cell order in both. The ratio above
prices the density of the population. It does not price the arena order, and
no row of this document does. A backlog item read it as an arena order figure
and a finding records the correction.[^ARENAORDER]

## The thread count moves the memory

Every memory row in this document was taken at one thread, and the thread
count is not free.

| Threads | Resident, bytes | Peak, bytes |
|---|---|---|
| 1 | 545,161,216 | 871,923,712 |
| 2 | 571,658,240 | 898,244,608 |
| 12 | 875,782,144 | 956,690,432 |

**A world at the target scale holds 545 MB at one thread and 876 MB at 12.**
The step gives each thread its own output slot, so the resident size grows
with the thread count. A memory figure that does not name a thread count is
not usable.

**The peak moves much less**, from 872 MB to 957 MB. The peak is set by the
build, which runs at one thread whatever the caller asks for. A machine needs
about 960 MB free to build and step a world at the target scale.

## The block edge

**Every figure in this document was taken at a block edge of 32 tiles**, which
gives 16,384 level 1 cells at the target extent. The benchmark passes no block
edge, so a world takes the default that the bridge states.[^11]

The value is a default rather than a decision, and the constant says so in its
own documentation: the record that fixes the tile storage order is not
written, so the layout takes the exponent as a parameter, and a research
report recommends 32 tiles.[^11]

The level 1 cell count moves with this value, and the rebuild, the summary and
the choice stagger all scale with the cell count. **A figure here that named no
block edge would not be reproducible**, which is why this section exists. The
sensitivity is not measured.

## A second prediction, written before the run that tests it

**This section was committed before the measurement it names existed**, in the
same way the 500,000-unit prediction was. The commit that adds it holds no
result.

The claim under test belongs to the record being written: cost should follow
the lattice rather than the population. The exit field is that record's first
instance, so the question is whether its derivation scales like the tile pass,
which reached 6.13 on 12 threads, or floors like the unit passes, which stop
near 1.85.

**The prediction is that it does neither, and that the reason is structural
rather than empirical.** The derivation takes no thread count. Its own
documentation states that the pass runs on the calling thread and that the
result depends on no thread count. So:

1. **The derivation costs the same at 1 thread and at 12.** A speedup outside
   0.9 to 1.1 refutes this.
2. **The derivation costs under 10 milliseconds** at the target extent, which
   holds 16,384 level 1 cells. The pass visits each cell once for each of six
   options and looks at six neighbours, which is about 590,000 inner steps.
3. **The level 1 rebuild beside it does scale**, because it takes a thread
   count. It improves by more than 2 times from 1 thread to 12.

**What each outcome means for the record.** If the derivation is small and
flat, the lattice claim is supported in the way that matters: the work is
proportional to the cells and there are few of them, so it does not need
threads. If the derivation is large, a serial pass sits in every frame and the
record has a problem its first instance created.

**Result: all three predictions hold.**

Machine A, 16,777,216 tiles, 1,000,000 units scattered, 16,384 level 1 cells.

| Threads | Exit field derive, ns | Speedup | Level 1 rebuild, ns | Speedup |
|---|---|---|---|---|
| 1 | 2,154,344 | 1.00 | 1,109,538,229 | 1.00 |
| 2 | 2,155,192 | 1.00 | 557,850,461 | 1.99 |
| 12 | 2,156,196 | 1.00 | 95,704,964 | 11.59 |

**The derivation is flat to three figures**, at 2.15 milliseconds whatever the
thread count. It was predicted flat within a tenth and it is flat within a
thousandth. It costs 132 ns for each cell, and it is one quarter of one
percent of a frame at the target scale.

**The level 1 rebuild scales at 11.59 times on 12 threads**, which is 0.97 of
the machine. That is the best scaling measured anywhere in this project. It is
11.5 percent of a frame at the target scale.

**The lattice claim is supported by its first instance, and the support is
sharper than the claim.** Three kinds of work now have measured scaling on the
same machine at the same extent:

| Work follows | Example | Speedup on 12 threads |
|---|---|---|
| The cells | The exit field derivation | 1.00, and it does not need threads |
| The tiles | The level 1 rebuild | 11.59 |
| The population | The unit passes | 2.08 packed, 2.32 scattered |

Work that follows the lattice is small enough not to need threads. Work that
follows the tiles takes them almost perfectly. **Work that follows the
population barely takes them at all**, and that is the half of the frame the
project cannot currently reduce.

## Would the choice pass collapse if it decided for each cell?

One weight profile serves every unit alive, so two units in the same level 1
cell with the same need score the same options and choose alike. A pass that
decided once for each distinct pair of cell and need would do that much less
work. The collapse factor is the live unit count divided by the number of
distinct pairs.

**Read the caution before the numbers.** The measurement below does not test
the premise it was built to test.

16,777,216 tiles, 1,000,000 units, block edge 32, so 16,384 level 1 cells
exist. Machine A. Every figure comes through the public crate interface.

| Placement | Cells occupied | Units for each cell | Median cell | Biggest cell | Distinct pairs | Collapse |
|---|---|---|---|---|---|---|
| Packed | 1,351 | 740.2 | 916 | 1,113 | 1,351 | 740.2 |
| Scattered | 14,970 | 66.8 | 64 | 430 | 14,970 | 66.8 |

**The distinct pair count equals the cell count exactly, at every bucket width
including the exact need.** The reason is that the need column holds **one
value**, 65536, which is one in the fixed point scale, for all 1,000,000
units, under both placements.

**So this measures the cell count and says nothing about the need.** The
measured world holds no settlement, so no unit has a home to draw from and
consumption never moves a need away from the value a unit spawns with. The
figures above are the collapse a world would show **if every unit held the
same need**, which is the best case and not a prediction.

**What the numbers do establish.** The geometry gives 14,970 occupied cells
for 1,000,000 units at the density the project states, so **66.8 is the
ceiling on the collapse factor** and no need distribution can beat it. The
packed figure of 740.2 is a property of a fixture that puts the whole
population into 8 percent of the cells, and it is not a figure about the
engine.

**What decides where the real answer falls.** A cell holds 64 units at the
median under the scattered pattern. The distinct pairs in a cell are the
smaller of the units in it and the number of need values those units take, so
the collapse in the median cell is about 64 divided by the number of need
buckets in play.

| Need buckets in a cell | Collapse at the median cell |
|---|---|
| 1 | 64 |
| 4 | 16 |
| 16 | 4 |
| 64 | 1 |

**The need is a Q16.16 quantity and takes about four thousand million
values.** So the bucket width is not a detail of the rule. It is the
mechanism. Unbucketed, the collapse is 1 and the rule buys nothing.

**What nobody has measured.** How many need values coexist in one cell in a
world that consumes. No fixture in this project produces one: it needs
settlements, home sites and a running economy, and the benchmark world has
none of the three. That measurement is the one the rule rests on, and this run
did not take it.

## Every stage of a frame, by name

**This section supersedes the stage split below it.** That split had three
switches and left 62 percent of the cost in one residual. This one names every
pass, and the residual is 0.0025 percent.

**A later section supersedes this one.** Backlog item 0291 changed the largest
pass named here, and item 0297 changed the two passes beside it. The section
that measures the tree after both is above the huge page section.[^ITEM291]
[^ITEM297] Read this table as the cost before those changes.

Machine C. 16,777,216 tiles, 1,000,000 units scattered, 12 threads, block edge
32. The kernel gave transparent huge pages on the `madvise` setting, which is
the default of the image and which this engine never asks for, so read this
table as the cost without huge pages. The next section gives the same table
with them.

| Machine C | Value |
|---|---|
| Instance type | `c7g.4xlarge` |
| Region | `us-west-2` |
| Processor | Graviton3. Implementer `0x41`, part `0xd40` |
| Hardware threads | 16 |
| Cache line | 64 bytes |
| Memory | 32,246,808 kB |
| Kernel | `Linux 6.18.44-99.149.amzn2023.aarch64` |
| Compiler | `rustc 1.91.1 (ed61e7d7e 2025-11-07)` |
| Base commit | `79201addd452178480684d5c4c61777eb39d5b0b` |
| Working tree | Modified. The script copies the tracked files with the content the tree holds |
| Crate features | `stage-cost` |
| Date | 3 September 2026 |

Every row is the mean of nine frames after two warm-up frames.

**An indented row comes from a second run**, at the same extent, thread count,
placement, setting and base commit, from a tree that added three spans inside
the largest stage. Its share is against the frame that second run measured, which is 816.1
milliseconds. A share here is a proportion and not an amount, and the
paragraph below the table gives the spread between the two runs.

| Stage | ns for each frame | Share of the frame | Takes a thread count |
|---|---|---|---|
| `holding_spread` | 514,291,130 | 61.5 percent | yes |
| — of which `holding_candidates` | 400,791,924 | 49.1 percent | **no** |
| — of which `holding_decide` | 71,110,442 | 8.7 percent | yes |
| — of which `holding_apply` | 32,707,035 | 4.0 percent | **no** |
| `change_merge` | 120,529,050 | 14.4 percent | **no** |
| `rebuild_level_1` | 77,987,770 | 9.3 percent | yes |
| `bridge_refresh_barrier` | 41,829,834 | 5.0 percent | yes |
| `admit` | 29,298,598 | 3.5 percent | yes |
| `tile_scan` | 16,548,947 | 2.0 percent | yes |
| `influence_solve` | 12,979,788 | 1.6 percent | yes |
| `stamp_holders` | 6,850,640 | 0.8 percent | **no** |
| `log_join` | 5,983,495 | 0.7 percent | **no** |
| `movement_intents` | 4,698,552 | 0.6 percent | yes |
| `gather` | 2,062,766 | 0.2 percent | yes |
| `build` | 1,602,971 | 0.2 percent | yes |
| `choose` | 570,560 | 0.07 percent | yes |
| `place_granted` | 517,411 | 0.06 percent | **no** |
| `consume` | 165,279 | 0.02 percent | yes |
| `reap` | 38,817 | under 0.01 percent | yes |
| `apply_rates` | 545 | under 0.01 percent | yes |
| `settle_positions` | 400 | under 0.01 percent | yes |
| `bridge_refresh_after_reap` | 260 | under 0.01 percent | yes |
| `bridge_refresh_opening` | 174 | under 0.01 percent | yes |
| `depletion_recover` | 87 | under 0.01 percent | no |
| **Every stage** | **835,957,085** | | |
| **The frame, timed from outside** | **835,978,143** | | |

**The residual is 21,058 nanoseconds, which is 0.0025 percent of the frame.**
That is the part of the step no stage covers, and it includes what the clock
costs to read forty-two times. The frame is now attributed.

**The apparatus agrees with the earlier run.** A scattered frame at this
extent, this unit count and this thread count measured 835,171,248 ns on
machine A at an earlier commit. This run measures 835,978,143 ns. The two
differ by one part in a thousand, and they were taken by different code on
different machines, so the instrument costs nothing this apparatus can see.

**Three stages are 85.3 percent of the frame.** They are the holding spread,
the change merge and the level 1 rebuild.

**The `change_merge` stage no longer exists.** The tile value field now stores a
dense delta and the tile scan's workers write it directly, so there is no run to
sort, no join to make and no merge to run.[^DENSE1] [^DENSE2] The rows above are
kept because they name their own commit and they are what the project measured,
and they are the reason the stage was removed. A later run measures the frame
without it.

**The holding spread alone is 61.5 percent, and no backlog item named it
before this run.** The earlier split could not see it, because the spread has
no switch and the split priced only what a switch could remove.

**Inside it, one serial function is 49.1 percent of the whole frame.** The
three indented rows come from a second run on the same machine type and the
same base commit, and they divide the spread. The candidate list walks every held tile
and every live unit on the calling thread, pushes an index for each, then
sorts several million indices and removes the duplicates. The half that
decides takes a thread count and costs 71.1 milliseconds. The half that
chooses what to decide about takes none and costs five and a half times as
much.

Those three rows sum to 504.6 milliseconds against 507.9 for the spread, so
3.3 milliseconds of the spread is the checking and the setting up around them.
An indented row is inside the row above it, so it is not added to the frame
twice.

**The second run measured a frame at 816.1 milliseconds against 835.98 for the
first, under the same setting.** The two runs differ by 2.4 percent, on one
machine type, from one base commit, with three spans added between them. Treat
2.4 percent as the spread between two runs of this apparatus, and read every
share in this section as a proportion rather than as an amount.

**The choice is no longer expensive.** The earlier split measured it at 71.4
milliseconds, which was 26 percent of the cost of a unit. It is now 0.571
milliseconds. The pass decides once for each pair of cell and need instead of
once for each unit, and the two figures are the same pass before and after
that change.

**Five stages of the frame take no thread count**, and together they are 16.0
percent of it. The change merge is nearly all of it. It sorts the tile changes
of the frame by tile index on the calling thread, and then merges one ascending
run. Its cost follows the number of changed tiles, which follows the tile
count.

**Two of the three rows inside the holding spread take no thread count as
well**, and they are a further 53.1 percent of the frame. The spread itself
takes one, so a reader who counts only the outer rows misses them.

**The column that says whether a stage takes a thread count is a declaration
in the source, not a measurement.** A stage declared `no` that improves with
the thread count means the declaration is wrong, and this table is where the
two can be compared.

## Every stage of a frame, after the candidate pass became a bit plane

**A later section supersedes this one**, and it is above the huge page
section.[^ITEM297]

**This section supersedes the section above it.** That one measured the tree
before backlog item 0291, in which the candidate pass of the holding spread
built a list of tile indices and ordered it with a comparison sort. The pass
now sets one bit for each candidate in a plane over the tiles and reads the
plane back in ascending order, and it takes a thread count.[^ITEM291]

Machine C, the same instance type, the same region and the same base commit as
the section above. The tree is modified against that commit, which is how the
script carries a change that is not yet merged.

| Machine C | Value |
|---|---|
| Instance type | `c7g.4xlarge` |
| Region | `us-west-2` |
| Processor | Graviton3. Implementer `0x41`, part `0xd40` |
| Hardware threads | 16 |
| Cache line | 64 bytes |
| Kernel | `Linux 6.18.44-99.149.amzn2023.aarch64` |
| Compiler | `rustc 1.91.1 (ed61e7d7e 2025-11-07)` |
| Base commit | `14e311a1088825f32ee4c421827e87a286d4094d` |
| Working tree | Modified. The script copies the tracked files with the content the tree holds |
| Crate features | `stage-cost` |
| Date | 3 September 2026 |

16,777,216 tiles, 1,000,000 units scattered, 12 threads, block edge 32. The
kernel gave transparent huge pages on the `madvise` setting, which the engine
never asks for, so read this table as the cost without huge pages. Every row
is the mean of nine frames after two warm-up frames.

**Every row here comes from one run**, unlike the section above, so the three
indented rows and the outer rows are the same nine frames and their shares are
against the same frame.

| Stage | ns for each frame | Share of the frame | Takes a thread count |
|---|---|---|---|
| `holding_spread` | 124,953,422 | 26.96 percent | yes |
| — of which `holding_decide` | 71,397,398 | 15.41 percent | yes |
| — of which `holding_apply` | 35,640,121 | 7.69 percent | **no** |
| — of which `holding_candidates` | 16,732,018 | 3.61 percent | yes |
| `change_merge` | 119,454,829 | 25.78 percent | **no** |
| `rebuild_level_1` | 78,112,992 | 16.86 percent | yes |
| `bridge_refresh_barrier` | 40,511,452 | 8.74 percent | yes |
| `admit` | 30,605,355 | 6.60 percent | yes |
| `tile_scan` | 22,871,237 | 4.94 percent | yes |
| `influence_solve` | 12,846,131 | 2.77 percent | yes |
| `stamp_holders` | 6,660,936 | 1.44 percent | **no** |
| `log_join` | 5,924,590 | 1.28 percent | **no** |
| `movement_intents` | 4,722,220 | 1.02 percent | yes |
| `gather` | 2,155,229 | 0.47 percent | yes |
| `build` | 1,591,385 | 0.34 percent | yes |
| `choose` | 565,668 | 0.12 percent | yes |
| `place_granted` | 517,194 | 0.11 percent | **no** |
| `consume` | 166,714 | 0.04 percent | yes |
| `reap` | 31,657 | under 0.01 percent | yes |
| `apply_rates` | 633 | under 0.01 percent | yes |
| `bridge_refresh_after_reap` | 346 | under 0.01 percent | yes |
| `settle_positions` | 297 | under 0.01 percent | yes |
| `bridge_refresh_opening` | 284 | under 0.01 percent | yes |
| `depletion_recover` | 137 | under 0.01 percent | **no** |
| **Every stage** | **451,692,718** | | |
| **The frame, timed from outside** | **463,431,747** | | |

**The frame costs 463.4 milliseconds against 825.4 for the same apparatus
before the change.** That is 1.78 times less, and it is 4.6 times the 100
millisecond budget instead of 8.3 times.[^BUDG291]

**The candidate pass costs 16.7 milliseconds against 400.9.** That is 24.0
times less. It was 49.1 percent of a frame and it is now 3.6 percent. The two
figures come from two runs of this script on this instance type at this base
commit, one before the change and one after it.

**The measurement that gives the 825.4 milliseconds is a run of its own.** The
section above quotes 836.0 from an earlier run of the same point, and the two
differ by 1.3 percent. Read every comparison in this section against 825.4,
because that run and this one are a pair.

**The residual is 11,739,029 nanoseconds, which is 2.53 percent of the
frame.** It was 22,202 nanoseconds before the change, and the finding holds
what is known about the difference and what was ruled out.[^RESID291]

**Three stages are 69.6 percent of the frame.** They are the holding spread,
the change merge and the level 1 rebuild, which is the same three as before
the change, in the same order. The spread fell from 61.5 percent of a larger
frame to 27.0 percent of a smaller one.

**The change merge is now the largest serial pass in the engine.** It is 119.5
milliseconds and it takes no thread count, so it bounds every thread count
above it. Five stages take no thread count and together they are 34.5 percent
of the frame, and the merge is three quarters of that.

**Two of the three rows inside the holding spread now take a thread count.**
The one that does not is the apply, at 35.6 milliseconds. It sorts the changed
tiles and merges one ascending run into the held list, which is the same shape
as the change merge.

## Every stage of a frame, after the tile value field became a dense delta

**This section supersedes the two above it.** They measured a tree in which the
tile value field stored its changes as a sorted list, and a stage merged a run
into that list on every frame. The field now stores one delta for each tile and
the tile scan's workers write it directly, so the merge stage no longer
exists.[^DENSE1]

Machine D. 16,777,216 tiles, 1,000,000 units scattered, 12 threads, block edge
32. Every row is the mean of nine frames after two warm-up frames, and every
row comes from one run. The kernel gave transparent huge pages on the `madvise`
setting, which the engine never asks for, so read this table as the cost
without huge pages.

| Machine D | Value |
## Every stage of a frame after the ground read moved last

**This section supersedes every stage table above it.** Two of those tables
measured trees that no longer exist, and the shares in them are against frames
that no longer exist either. Read them as history.

Backlog item 0297 changed three things inside the holding spread: the rule that
decides a tile reads the ground last rather than first, the walk through the
derived unit structure replaced a search for each tile, and the two repairs
after a write took a thread count.[^ITEM297] Other work between the tables
changed the tile value field and the change merge.

| Machine C | Value |
|---|---|
| Instance type | `c7g.4xlarge` |
| Region | `us-west-2` |
| Processor | Graviton3. Implementer `0x41`, part `0xd40` |
| Hardware threads | 16 |
| Cache line | 64 bytes |
| Memory | 32,246,808 kB |
| Kernel | `Linux 6.18.44-99.149.amzn2023.aarch64` |
| Compiler | `rustc 1.100.0-nightly (0dfb098f3 2026-08-31)` |
| Base commit | `4e768e9e34e998264b5c9750d545c15f35dd0772` |
| Working tree | Clean |
| Crate features | `stage-cost` |
| Date | 3 September 2026 |

| Stage | ns for each frame | Share of the frame | Takes a thread count |
|---|---|---|---|
| `holding_spread` | 122,719,178 | 50.84 percent | yes |
| — of which `holding_decide` | 69,845,980 | 28.94 percent | yes |
| — of which `holding_apply` | 34,593,757 | 14.33 percent | **no** |
| — of which `holding_candidates` | 17,201,761 | 7.13 percent | yes |
| `admit` | 32,569,259 | 13.49 percent | yes |
| `bridge_refresh_barrier` | 31,332,853 | 12.98 percent | yes |
| `tile_scan` | 14,319,802 | 5.93 percent | yes |
| `influence_solve` | 12,681,675 | 5.25 percent | yes |
| `rebuild_level_1` | 6,755,064 | 2.80 percent | yes |
| `stamp_holders` | 6,365,000 | 2.64 percent | **no** |
| `log_join` | 5,700,377 | 2.36 percent | **no** |
| `movement_intents` | 3,296,169 | 1.37 percent | yes |
| `gather` | 2,021,124 | 0.84 percent | yes |
| `build` | 1,562,305 | 0.65 percent | yes |
| `choose` | 525,289 | 0.22 percent | yes |
| `place_granted` | 465,914 | 0.19 percent | **no** |
| `consume` | 169,537 | 0.07 percent | yes |
| `reap` | 36,166 | 0.01 percent | yes |
| `apply_rates` | 610 | under 0.01 percent | yes |
| `settle_positions` | 421 | under 0.01 percent | yes |
| `bridge_refresh_after_reap` | 376 | under 0.01 percent | yes |
| `depletion_recover` | 294 | under 0.01 percent | **no** |
| `bridge_refresh_opening` | 221 | under 0.01 percent | yes |
| **Every stage** | **240,521,640** | | |
| **The frame, timed from outside** | **241,374,071** | | |

**The frame costs 241.4 milliseconds against 463.4 for the same apparatus
before the change.** That is 1.92 times less. It is 2.41 times the 100
millisecond budget instead of 4.63 times.[^BUDG291]

**Resident memory is 645,193,728 bytes.**

### Where the 222 milliseconds went

The whole saving is 222,057,676 nanoseconds for each frame. Four stages hold
208,542,791 of it and the rest is spread across every other row.

| Stage | Before | After | Change |
|---|---|---|---|
| `change_merge` | 119,454,829 | the stage no longer exists | −119,454,829 |
| `rebuild_level_1` | 78,112,992 | 6,755,064 | −71,357,928 |
| `bridge_refresh_barrier` | 40,511,452 | 31,332,853 | −9,178,599 |
| `tile_scan` | 22,871,237 | 14,319,802 | −8,551,435 |
| `admit` | 30,605,355 | 32,569,259 | +1,963,904 |

**Only the first row was predicted.** The change was justified by the merge
alone, and the merge is a little over half of what it returned.

**The second row is the larger surprise and it has a plain cause.** The level 1
rebuild sums the value of every tile. Reading one tile used to be a binary
search into a sorted list that held an entry for almost every tile, so a pass
over the world paid a logarithmic search 16,777,216 times. It is now one index.
The rebuild costs 11.6 times less and nothing in it changed.

The tile scan gains for the same reason, and it gains despite taking over the
writes the merge used to do.

**The engine reads the tile value field far more often than it writes it, and
the cost of the sparse form was concentrated in the reads.** The finding that
opened this work measured the write side, because the write side is what the
stage table named. A structure is not priced by the pass that carries its
name.[^DENSE2]

| Kernel | `Linux 6.18.44-99.149.amzn2023.aarch64` |
| Compiler | `rustc 1.91.1 (ed61e7d7e 2025-11-07)` |
| Base commit before | `21a82f5f14e15b3fd129d881687d32a8ed41df67` |
| Base commit after | `0f8d54b56ecee5b8bcb3bff62ecfe011583f5b29` |
| Crate features | `stage-cost` |
| Date | 3 September 2026 |

16,777,216 tiles, 1,000,000 units scattered, 12 threads, block edge 32. The
kernel gave transparent huge pages on the `madvise` setting, which the engine
never asks for, so read this table as the cost without huge pages. Every row is
the mean of nine frames after two warm-up frames, from one run.

### What the three changes bought

| Stage | Before, ns | After, ns | Change |
|---|---|---|---|
| `holding_spread` | 121,594,656 | 70,520,746 | 42.0 percent less |
| — of which `holding_decide` | 69,666,121 | 33,468,314 | 52.0 percent less |
| — of which `holding_apply` | 34,010,330 | 18,638,605 | 45.2 percent less |
| — of which `holding_candidates` | 16,783,043 | 17,347,039 | 3.4 percent more |
| **The frame, timed from outside** | **300,016,572** | **249,035,855** | **17.0 percent less** |

**The frame is 2.49 times its budget of 100 milliseconds.**[^BUDG291] It was 8.3
times that budget before the candidate pass changed, so the two items together
took it from 8.3 to 2.5.

**A third run divides the deciding change.** Reading the ground last, on its own,
took the deciding pass from 69,666,121 to 41,580,885 nanoseconds. Walking the
derived unit structure rather than searching it took the rest, to 33,468,314. The
run between the two is not in this table because the tree it measured was a
measurement point and not a commit anyone should return to.

**The candidate pass reads 3.4 percent higher and nothing in it changed.** Treat
that as the spread between two runs of this apparatus at this size, which an
earlier section put at 2.4 percent.

### The whole frame

| Stage | ns for each frame | Share of the frame | Takes a thread count |
|---|---|---|---|
| `holding_spread` | 70,520,746 | 28.32 percent | yes |
| — of which `holding_decide` | 33,468,314 | 13.44 percent | yes |
| — of which `holding_apply` | 18,638,605 | 7.48 percent | yes |
| — of which `holding_candidates` | 17,347,039 | 6.97 percent | yes |
| `change_merge` | 43,591,292 | 17.50 percent | **no** |
| `admit` | 32,461,246 | 13.03 percent | yes |
| `bridge_refresh_barrier` | 31,474,424 | 12.64 percent | yes |
| `tile_scan` | 20,317,530 | 8.16 percent | yes |
| `influence_solve` | 13,343,828 | 5.36 percent | yes |
| `rebuild_level_1` | 6,571,927 | 2.64 percent | yes |
| `stamp_holders` | 6,249,835 | 2.51 percent | **no** |
| `log_join` | 5,515,419 | 2.21 percent | **no** |
| `movement_intents` | 3,150,753 | 1.27 percent | yes |
| `gather` | 1,986,444 | 0.80 percent | yes |
| `build` | 1,525,866 | 0.61 percent | yes |
| `place_granted` | 698,822 | 0.28 percent | **no** |
| `choose` | 546,063 | 0.22 percent | yes |
| `consume` | 163,657 | 0.07 percent | yes |
| `reap` | 30,716 | 0.01 percent | yes |
| `apply_rates` | 616 | under 0.01 percent | yes |
| `bridge_refresh_after_reap` | 377 | under 0.01 percent | yes |
| `settle_positions` | 367 | under 0.01 percent | yes |
| `depletion_recover` | 297 | under 0.01 percent | **no** |
| `bridge_refresh_opening` | 208 | under 0.01 percent | yes |
| **Every stage** | **238,150,441** | | |
| **The frame, timed from outside** | **249,035,855** | | |

**The frame no longer has one dominant stage.** The largest is 28.3 percent and
the next four are between 8 and 18 percent. The holding spread was 61.5 percent
when the stage table was first taken.

**The change merge is the largest serial pass, at 17.5 percent.** Four stages
take no thread count and together they are 22.5 percent of the frame. The merge
is three quarters of that.

**The level 1 rebuild is 2.64 percent.** A plan written from an earlier table
called it 16.9 percent and put it second. Nothing in the rebuild changed between
the two readings, and a finding holds what that cost.[^DENOM297]

**The residual is 10,885,414 nanoseconds, which is 4.37 percent of the frame.**
It has grown as a share because the frame shrank, and it is 12 milliseconds in
absolute terms in every run since the candidate pass began allocating on each
frame. The cause is still not identified.[^RESID291]

## Huge pages

**The engine asks for no huge page.** This measurement changes the kernel
setting for the whole machine instead, which is the cheapest form of the
experiment: the same binary, the same base commit, the same machine, three
processes, and the kernel backing the large anonymous mappings either at 4 kB
or at 2 MB.

Machine C. 16,777,216 tiles, 1,000,000 units scattered, 12 threads. Each row
is a process of its own, because a process keeps the mapping it was given.

| Setting | Frame, ns | On huge pages, bytes | Resident, bytes |
|---|---|---|---|
| `never` | 841,364,267 | 0 | 1,051,910,144 |
| `madvise` | 835,978,143 | 0 | 1,051,910,144 |
| `always` | 803,042,781 | 719,323,136 | 1,079,799,808 |

**A frame costs 3.9 percent less on huge pages.** That is 32.9 milliseconds of
a 836 millisecond frame.

**A second run repeated it and agreed.** The run that divided the holding
spread measured the same two settings again, on the same machine type, from
the same base commit, from a tree that added three spans.

| Setting | Frame, ns | On huge pages, bytes | Resident, bytes |
|---|---|---|---|
| `madvise` | 816,126,687 | 0 | 1,051,869,184 |
| `always` | 780,600,154 | 616,562,688 | 1,079,472,128 |

That is 4.35 percent against 3.94 for the first run. **The two runs agree on
the direction and roughly on the size, and they disagree with each other by
more than the effect is quoted to.** Two figures near four percent is what
this apparatus supports. One figure to two decimal places is not.

**Which stages gained is the same answer in both runs.** In the second run the
change merge gives 16.1 milliseconds, the candidate list 11.7 and the holding
apply 0.9. The decide half is flat to one part in three thousand, at
71,110,442 nanoseconds and 71,084,844. **The stage that gains nothing is the
one that reads a compact list**, and the stages that gain are the ones that
walk a large array.

**The two settings the engine cannot tell apart differ by 0.64 percent.** The
engine calls no advice, so `never` and `madvise` give it the same 4 kB pages,
and the huge page column proves it: both landed nothing.

That 0.64 percent is the spread between two rows taken minutes apart in one
sweep. The spread between two sweeps is larger, at 2.4 percent. **Read the
huge page effect as a comparison inside one sweep**, which is the comparison
that was made: two rows from one machine, one binary, minutes apart, differing
in one kernel setting. Both sweeps gave near four percent that way.

**The setting reached the process.** 719,323,136 bytes of the resident set sat
on huge pages under `always`, which is 66.6 percent of it. A row that reported
a time and not this column would be a claim that the setting worked.

**The saving is concentrated in three stages.** Every other stage moved by
less than half a millisecond.

| Stage | `madvise`, ns | `always`, ns | Difference |
|---|---|---|---|
| `change_merge` | 120,529,050 | 105,128,942 | −15,400,108 |
| `holding_spread` | 514,291,130 | 499,920,726 | −14,370,404 |
| `bridge_refresh_barrier` | 41,829,834 | 38,466,307 | −3,363,527 |
| Everything else | 159,307,071 | 159,505,748 | +198,677 |

**Those are the three passes that write scattered over a large array**, which
is where item 0269 predicted a translation cost would appear. The prediction
named the shape and the stage table found it in that shape.

**The memory cost is five times what the item estimated.** The item put the
waste at the end of an allocation under one part in two hundred. The resident
set grew by 27,889,664 bytes, which is 2.65 percent.

**This is one run for each condition, taken in two sweeps.** Nine frames for
each row, one process for each row, one machine type, one base commit, two
trees. It is enough to say
the effect exists and that it is near four percent. It is not enough to quote
it to two figures: the two runs differ by 2.4 percent under the same setting,
and two conditions that should be identical differ by 0.64 percent.

**Nothing here is a code change.** A machine setting is not something the
engine controls where it runs. Capturing this saving portably means asking for
the pages in the allocation, and that is a separate item.[^19]

## Where the unit cost goes

**A later run supersedes this section, and the section above holds it.** The
engine now records what every stage costs, so the residual this section could
not divide is 0.0025 percent rather than 62 percent. Two figures here are also
stale against the code: the choice cost 71.4 milliseconds when this was taken
and costs 0.571 now, because the pass decides for each cell and need rather
than for each unit. This section is kept because it is what the project
believed and it names its own machine and commit.

The unit passes are 274 milliseconds of a 521 millisecond frame at the target
scale, at 12 threads. This section splits that as far as the public interface
allows, and no further.

**The engine holds no instrumentation and this benchmark adds none.** A stage
inside a step is not callable on its own, so a stage is priced by running a
whole frame with it switched off and taking the difference. Three switches
exist. The rest of the frame stays in one residual.

Machine A, 16,777,216 tiles, 1,000,000 units, 12 threads, units packed. The
shares below are of a packed unit cost, so read them as proportions rather
than as amounts.

| Row | Samples | Median, ns |
|---|---|---|
| Everything on | 9 | 521,302,567 |
| The economy off | 9 | 515,585,208 |
| The choice off | 9 | 449,856,083 |
| Both off | 9 | 447,879,653 |
| One bridge rebuild, alone | 9 | 26,286,017 |

| Part | Milliseconds | Share of the unit cost |
|---|---|---|
| The choice, scoring only | 71.4 | 26 percent |
| One bridge rebuild | 26.3 | 10 percent |
| The economy | 5.7 | 2 percent |
| **The residual, which this cannot divide** | **170.1** | **62 percent** |

**It is not one stage.** The largest thing here is the part that could not be
divided.

**The residual holds** the movement intents, admission, the holder spread, the
death scan, the part of the level 1 rebuild that reads the units, and the walk
over every live unit inside the choice pass that the interval does not remove.
Nothing on the public interface separates them.

**The bridge is one rebuild in a frame, not three.** The step calls the
refresh three times, and the refresh compares a revision counter and returns
when the bridge is still accurate. That check is a constant cost. In this
world one call finds the bridge stale, because movement moved the units, so a
frame pays one rebuild. A world in which units also die each frame would pay
two.

**The choice scores about one unit in 32 and costs 71 milliseconds doing it.**
The interval is 32 ticks, keyed on the level 1 cell, so about 31,000 of the
1,000,000 units score in a frame. That is about 2.3 microseconds for each unit
scored. The figure is a division and not a measurement, and it holds only if
the schedule spreads the cells evenly.

**The economy is small here and this run understates it.** The period is 10
ticks, so it applies on one frame in ten and the median of nine samples mostly
misses it. The maximum is where it shows. The measured world holds no
settlement, so the rate pass had nothing to apply.

**Two switches together cost less than the two apart**, 73.4 milliseconds
against 77.2. The difference is inside the spread of the rows.

## The cost of a frame, as two straight lines

The measured cost is the tile count times a constant, plus the unit count
times a second constant. Machine A.

| Quantity | 1 thread | 2 threads | 16 threads |
|---|---|---|---|
| One tile, one frame | 78 ns | 43 ns | 13 ns |
| One unit, one frame | 561 ns | 416 ns | 303 ns |

The two constants reproduce the target scale row at two threads: they give
1,132 milliseconds against the 1,120 measured. **Do not read that agreement as
accuracy.** A prediction written before its run, at 500,000 units, missed by
6.2 percent, and the section above holds it. The cost of one unit rises with
the population and also depends on the extent, so the two lines are an
approximation good to about ten percent.

The unit constant comes from the difference between a world of 1,000,000 units
and a world of none, at 4,194,304 tiles. The same difference at 100,000 units
gives 573 ns for each unit at one thread, and at 10,000 units it gives 768 ns,
so the line is straight over two orders of magnitude and bends upward slightly
at the smallest count.

## A prediction, written before the run that tests it

**This section was committed before the measurement it names existed.** The
commit that adds it holds no result, and the commit that follows holds the
result. The order of the two commits is the evidence, and a reader who doubts
it can read the history.

The two constants above were computed after every row was in hand. That makes
the agreement between them and the target scale row a consistency check and
not a prediction, and a consistency check is the weaker of the two. This
section removes the doubt for one point.

**The prediction.** A world of 16,777,216 tiles and 500,000 units, stepped at
12 threads, on a Graviton3 instance, costs a median of **396.6 milliseconds**
for one frame.

**How it is computed.** The tile pass at 16,777,216 tiles and 12 threads is a
measured row, at 247,703,940 ns. The cost of one unit at 12 threads is a
measured difference, at 297.878 ns. The prediction is the first plus 500,000
times the second.

**What counts as a hit.** A median within five percent, which is 376.8 to
416.5 milliseconds. A median outside that band refutes the additive model, and
this section stays in the register saying so.

Nothing in the configuration was measured before. No row above holds 500,000
units, and no row above holds a world of 16,777,216 tiles with any unit count
between zero and one million.

**Result: the prediction missed.** The measured median is **371.9
milliseconds** against a prediction of 396.6, which is 6.2 percent low and
outside the band the prediction set.

| | Milliseconds |
|---|---|
| Predicted | 396.6 |
| The band that counted as a hit | 376.8 to 416.5 |
| Measured, 9 samples | 371.9 |
| Minimum, maximum | 369.6, 400.9 |

**What the miss says.** The additive model overstates the cost of a unit at
this population. The cost of one unit is not one constant. At 16,777,216
tiles it is 248 ns at 500,000 units and 270 ns at 1,000,000, so it rises with
the population rather than staying flat. The constant the prediction used came
from a world of 4,194,304 tiles, where it is 298 ns, so it also depends on the
extent that holds the units.

**The register said the constants predict the target scale row to one part in
two hundred. That agreement was one point, and this run shows it was not a
property.** Treat the two constants as an approximation good to about ten
percent, and not better.

**The headline result does not rest on the model.** The frame at the target
scale, the tile pass and the unit passes are each measured rows at the thread
count they name. The floor in the unit passes is a difference between measured
rows. None of them is computed from a constant, so none of them moves.

## Resident memory

**Size a machine at about 960 MB, not 545.** The 545 MB below is the figure at
one thread. The same world holds 876 MB at 12 threads and peaks at 957 MB
while it builds. This paragraph exists because the one-thread figure is the
one somebody sizing a machine would otherwise quote.[^12]

**Units packed, one thread.** Each row below comes from a process that
measured one point and exited. A
process that has already built a large world does not return the memory to the
operating system, so one process measuring every point would report the high
mark of the run rather than the cost of the world it holds. Machine A, one
thread, two frames run before the reading.

| Tiles | Units | Empty process, bytes | Resident, bytes | Peak, bytes |
|---|---|---|---|---|
| 4,096 | 0 | 2,056,192 | 2,564,096 | 2,564,096 |
| 65,536 | 0 | 2,027,520 | 4,476,928 | 5,443,584 |
| 65,536 | 10,000 | 2,064,384 | 5,570,560 | 6,639,616 |
| 1,048,576 | 0 | 2,015,232 | 33,947,648 | 55,984,128 |
| 1,048,576 | 10,000 | 2,121,728 | 35,262,464 | 57,225,216 |
| 1,048,576 | 100,000 | 2,027,520 | 45,568,000 | 67,608,576 |
| 4,194,304 | 0 | 2,068,480 | 155,004,928 | 192,585,728 |
| 4,194,304 | 10,000 | 2,048,000 | 156,016,640 | 193,576,960 |
| 4,194,304 | 100,000 | 2,076,672 | 166,506,496 | 204,083,200 |
| 4,194,304 | 1,000,000 | 2,097,152 | 217,436,160 | 294,014,976 |
| 16,777,216 | 0 | 2,076,672 | 456,155,136 | 707,706,880 |
| 16,777,216 | 10,000 | 2,031,616 | 457,134,080 | 708,923,392 |
| 16,777,216 | 100,000 | 2,015,232 | 463,851,520 | 721,616,896 |
| 16,777,216 | 1,000,000 | 2,121,728 | 545,161,216 | 871,923,712 |

**A world at the target scale holds 545 MB at one thread.** That is
16,777,216 tiles and 1,000,000 units, and no settlement and no character. The
same world holds 876 MB at 12 threads, and a section below holds the rows.

**The tiles are the cost, and the units are not.** The same world with no unit
holds 456 MB, so the whole population of one million adds 89 MB. A tile costs
27 bytes and a unit costs 89 bytes.

**A tile costs 27 bytes even though the ground is generated.** Two records
state that a tile field is a generated base with only the change stored, and
that a tile stock is generated with only what was taken stored.[^13] [^14]
Both hold: nothing here stores a tile value or a stock. The 27 bytes are the
columns the world does allocate for each tile, and one proposed item already
names the holder column as one of them.[^15]

**Building the world needs 872 MB, not 545 MB.** The peak is 60 percent above
the resident size at every large row. A machine sized to hold the world will
fail to build it. The gap is the build and not the frame, because the
constructor sums every tile into the first pyramid level.

## Integer overflow at the target scale

Hard invariant 9 states that a `u8` tile field summed over 16,777,216 tiles
reaches 4,258,500,000, that this is inside a `u32` by 0.85 percent, and that
an accumulator must not depend on that margin.

**No accumulator overflowed.** One frame at the target scale ran a second
time, built with the overflow check on, and it passed.

The check is not on in the rows above. The bench profile inherits the release
profile, which carries no overflow check, so a wrap in any row above would
have wrapped in silence.[^16] The check costs time, so the checked run is a
separate build and gives no timing row. A timing row taken under it would
measure the check.

## One frame against the tile count

The world holds no unit in every row of this table, so the placement pattern
does not reach it. Machine A.

| Tiles | Threads | Samples | Minimum, ns | Median, ns | Maximum, ns | Median, ns for each tile |
|---|---|---|---|---|---|---|
| 4,096 | 1 | 9 | 1,717,026 | 1,744,274 | 1,896,541 | 426 |
| 4,096 | 2 | 9 | 2,306,275 | 2,338,395 | 2,465,025 | 571 |
| 4,096 | 4 | 9 | 210,463 | 216,027 | 248,083 | 53 |
| 4,096 | 12 | 9 | 291,742 | 332,634 | 381,595 | 81 |
| 4,096 | 16 | 9 | 342,502 | 396,705 | 454,637 | 97 |
| 65,536 | 1 | 9 | 4,982,822 | 5,099,313 | 5,656,232 | 78 |
| 65,536 | 2 | 9 | 4,163,700 | 4,257,284 | 4,748,725 | 65 |
| 65,536 | 4 | 9 | 4,925,825 | 5,103,368 | 5,516,727 | 78 |
| 65,536 | 12 | 9 | 9,637,008 | 9,952,308 | 10,708,199 | 152 |
| 65,536 | 16 | 9 | 13,911,391 | 14,729,123 | 15,271,263 | 225 |
| 1,048,576 | 1 | 9 | 73,085,231 | 73,669,292 | 85,340,039 | 70 |
| 1,048,576 | 2 | 9 | 37,502,534 | 38,355,233 | 45,016,455 | 37 |
| 1,048,576 | 4 | 9 | 25,545,133 | 26,213,968 | 30,578,016 | 25 |
| 1,048,576 | 12 | 9 | 22,504,222 | 23,490,612 | 24,267,862 | 22 |
| 1,048,576 | 16 | 9 | 24,881,684 | 25,446,686 | 27,096,053 | 24 |
| 4,194,304 | 1 | 9 | 286,930,898 | 292,797,311 | 336,879,542 | 70 |
| 4,194,304 | 2 | 9 | 165,799,063 | 168,525,184 | 195,158,892 | 40 |
| 4,194,304 | 4 | 9 | 94,216,038 | 96,860,321 | 116,157,579 | 23 |
| 4,194,304 | 12 | 9 | 60,061,382 | 61,796,720 | 71,233,285 | 15 |
| 4,194,304 | 16 | 9 | 58,284,041 | 59,992,025 | 73,305,872 | 14 |
| 16,777,216 | 1 | 8 | 1,265,389,463 | 1,304,351,811 | 1,461,720,279 | 78 |
| 16,777,216 | 2 | 9 | 704,467,143 | 716,310,130 | 813,892,821 | 43 |
| 16,777,216 | 4 | 9 | 438,273,221 | 444,188,060 | 501,454,124 | 26 |
| 16,777,216 | 12 | 9 | 241,174,924 | 247,703,940 | 273,698,751 | 15 |
| 16,777,216 | 16 | 9 | 210,788,974 | 212,768,859 | 248,676,104 | 13 |

**The rows at 4,096 tiles disagree with the rest of the table, and nobody
has explained why.** One thread costs 426 ns for each tile, two threads cost
571 ns, and four threads cost 53 ns. Every larger extent costs between 13 and
78 ns for each tile at every thread count. Nine samples produced a spread
under one fifth in each row, and the pattern repeated on both machines, so it
is not noise and it is not one instance. A backlog item holds the
question.[^17] Do not cite the 4,096-tile rows.

**The small extents lose from a high thread count, and that is ordinary.**
At 65,536 tiles a frame costs more at 16 threads than at 2. The step starts
threads for each parallel stage, and at a small tile count that cost is larger
than the work it divides. It is not the same effect as the 4,096-tile rows,
because it rises with the thread count instead of falling.

## One frame against the unit count

Every row below holds 4,194,304 tiles, and the units are packed. Machine A.

| Units | Threads | Samples | Minimum, ns | Median, ns | Maximum, ns |
|---|---|---|---|---|---|
| 0 | 1 | 9 | 304,007,597 | 309,451,825 | 353,963,846 |
| 0 | 2 | 9 | 165,490,045 | 168,193,306 | 193,524,047 |
| 0 | 4 | 9 | 93,961,116 | 96,668,092 | 116,226,479 |
| 0 | 12 | 9 | 60,404,398 | 62,262,243 | 72,036,426 |
| 0 | 16 | 9 | 58,249,543 | 59,994,847 | 72,758,851 |
| 10,000 | 1 | 9 | 311,377,770 | 317,134,685 | 357,567,720 |
| 10,000 | 2 | 9 | 169,602,316 | 173,992,098 | 198,989,514 |
| 10,000 | 4 | 9 | 98,892,057 | 100,642,724 | 116,506,830 |
| 10,000 | 12 | 9 | 64,804,280 | 66,011,497 | 75,277,327 |
| 10,000 | 16 | 9 | 63,020,801 | 64,511,186 | 77,297,883 |
| 100,000 | 1 | 9 | 363,083,821 | 366,151,243 | 401,824,560 |
| 100,000 | 2 | 9 | 208,075,968 | 209,997,919 | 230,769,076 |
| 100,000 | 4 | 9 | 127,815,907 | 129,865,250 | 145,846,669 |
| 100,000 | 12 | 9 | 88,990,069 | 90,361,628 | 97,612,221 |
| 100,000 | 16 | 9 | 86,424,837 | 89,265,763 | 97,551,034 |
| 1,000,000 | 1 | 9 | 837,376,783 | 870,663,100 | 931,796,421 |
| 1,000,000 | 2 | 9 | 550,162,380 | 584,268,553 | 622,467,209 |
| 1,000,000 | 4 | 9 | 418,979,726 | 445,365,659 | 476,276,656 |
| 1,000,000 | 12 | 9 | 333,401,457 | 360,139,757 | 384,476,187 |
| 1,000,000 | 16 | 9 | 327,615,102 | 363,437,973 | 376,565,148 |
## Building a world

Machine A. The world holds no unit, so the placement does not reach these
rows. The build takes no thread count from the caller.

| Tiles | Samples | Minimum, ns | Median, ns | Maximum, ns | Median, ns for each tile |
|---|---|---|---|---|---|
| 4,096 | 9 | 1,178,508 | 1,189,665 | 1,370,799 | 290 |
| 65,536 | 9 | 18,096,642 | 18,099,749 | 18,282,420 | 276 |
| 1,048,576 | 9 | 287,456,064 | 287,509,793 | 289,825,287 | 274 |
| 4,194,304 | 9 | 1,156,823,643 | 1,157,183,481 | 1,157,780,822 | 276 |
| 16,777,216 | 3 | 4,636,191,433 | 4,636,309,563 | 4,638,105,825 | 276 |

**Building a world costs 276 ns for each tile, at every extent measured.**
Building a world at the target extent therefore takes 4.64 seconds.

**Sixteen cores did not make it faster.** The two-core machine took 4.63
seconds for the same build, which is within one part in eight hundred of
machine A. The constructor rebuilds the first pyramid level at one thread, so
the build is serial and the size of the machine does not reach it.

The tile value field generates a tile and stores nothing, so it visits no
tile.[^13] The first level of the pyramid does visit every tile, and the
constructor rebuilds it. A proposed backlog item holds that pass and names the
two shapes that would remove it.[^15]

**Reserving a million unit slots costs nothing that this run could see.** A
build at the target extent with the default reservation took a median of
4,637,235,543 ns, against 4,636,309,563 ns with a reservation of 1024. The
difference is one part in five thousand, and it is inside the spread of both
rows. The reservation record states that the cost is paid once, at
construction, and that the cost of a tick does not grow with it.[^18] This run
gives the first measured support for the first half of that claim.

## The hash of the whole world

Machine A. The world holds no unit, so the placement does not reach these
rows. The hash takes no thread count.

| Tiles | Samples | Minimum, ns | Median, ns | Maximum, ns | Median, ns for each tile |
|---|---|---|---|---|---|
| 4,096 | 9 | 1,276,666 | 1,282,615 | 1,298,580 | 313 |
| 65,536 | 9 | 21,262,532 | 21,272,684 | 21,377,755 | 325 |
| 1,048,576 | 9 | 351,351,703 | 351,620,466 | 351,964,335 | 335 |
| 4,194,304 | 8 | 1,424,822,378 | 1,426,095,428 | 1,427,671,952 | 340 |
| 16,777,216 | 3 | 6,018,879,287 | 6,020,360,980 | 6,022,534,409 | 359 |

The hash costs about 340 ns for each tile, and it takes one core. A golden
state test at the target extent therefore costs 6.02 seconds for each frame it
checks. The hash regenerates the ground, the stock and the tile value, because
each of those is generated from the seed rather than stored.

## What these runs did not measure

Read this section before you cite a figure above.

- **The measured world holds no settlement, and no character.** The rate pass,
  the consumption pass and the position pass therefore did no work, and the
  character arena was empty. Every frame figure above is a lower bound on a
  frame at the target scale, and the 545 MB is a lower bound on the memory.
  The scale constants table names 5,000 settlements and 50,000 living
  characters, and this run held none of either.[^2]
- **Two runs, on one date.** A run on another date would give another set of
  figures, and nobody has taken one.
- **No figure above measures a cache hit rate, an allocation count, or the
  cost of a call across the language boundary.** Three draft records state
  derived figures of those kinds, and these runs leave every one of them
  derived.[^2]
- **No figure separates the stages inside a step.** The step, the build and
  the hash reach the public interface, and the passes inside a step do not.
  The two halves in the table above are a difference between two worlds, not
  a measurement of a stage.
- **The thread counts above are not the thread counts of a running engine.**
  Each row asks the step for a thread count, and the step starts threads for
  each parallel stage. No figure says what a pool would cost.

## What belongs here

- A figure measured on the target platform, with the machine that produced it.
- The command that produced the figure, and the commit it was taken at.
- A statement of what a run did not cover.

## What does not belong here

- A derived figure. The other target register holds those.[^2]
- A figure taken on a development machine. The local register holds those.[^3]
- A decision. A measurement is an input to a decision, not a decision.

## Format for a row

Give the operation, the extent, the unit count, the thread count, the sample
count, and the minimum, the median and the maximum in nanoseconds. Give the
machine, the commit and the date beside the table.

**Name the fixture as well as the machine.** State how the units are placed
and what block edge the world took. Two figures in this document turned out to
describe the fixture rather than the engine: a packed population costs about
half what the stated density costs, and a memory figure without a thread count
is out by 60 percent. A number whose fixture is not named is not
reproducible.

Record a new table when a run changes a figure on purpose, and say in the
commit what changed. Do not edit a row to make a later run agree with it.

## References

[^1]: ADR-0008, the primary target is `aarch64-unknown-linux-gnu`, decision D2. `docs/adrs/accepted/adr-0008-the-primary-target-is-aarch64.md`
[^2]: Budgets and costs, the target register. `docs/reference/budgets.md`
[^3]: Development budgets, the local register. `docs/reference/development-budgets.md`
[^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^5]: The benchmark. `crates/cachette-core/benches/target_cost.rs`
[^6]: The provisioning script. `scripts/graviton-benchmark.sh`
[^7]: Testing rules, section 3. `.claude/rules/testing.md`
[^8]: ADR-0001, one binary gives one answer at any thread count, decision D5. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^9]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^10]: Blockers register, BLK-012, the resolution. `docs/BLOCKERS.md`
[^11]: The block edge default. `crates/cachette-core/src/bridge.rs`
[^12]: Findings register, FND-246. `docs/FINDINGS.md`
[^13]: ADR-0088, a tile field is a generated base and a stored change. `docs/adrs/draft/adr-0088-a-tile-field-is-a-generated-base-and-a-stored-change.md`
[^14]: ADR-0072, a tile stock is generated, and only what was taken is stored. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^15]: Backlog item 0171. `docs/backlog/proposed/0171-build-the-first-level-without-a-pass-over-every-tile.md`
[^16]: ADR-0083, the gate build checks every integer overflow. `docs/adrs/draft/adr-0083-the-gate-build-checks-every-integer-overflow.md`
[^17]: Backlog item 0229. `docs/backlog/proposed/0229-explain-the-frame-cost-at-the-smallest-extent.md`
[^18]: ADR-0084, the world reserves the unit columns at construction, decision D3. `docs/adrs/draft/adr-0084-the-world-reserves-the-unit-columns-at-construction.md`
[^19]: Backlog item 0290, ask for a huge page in the allocation. `docs/backlog/proposed/0290-ask-for-a-huge-page-in-the-allocation.md`
[^ARENAORDER]: Findings register, FND-273. `docs/FINDINGS.md`
[^ITEM291]: Backlog item 0291, stop the holding spread walking the population. `docs/backlog/complete/0291-stop-the-holding-spread-walking-the-population.md`
[^BUDG291]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^RESID291]: Findings register, FND-286. `docs/FINDINGS.md`

[^DENSE1]: ADR-0103, the tile value field stores a dense delta, never a sparse change list. `docs/adrs/draft/adr-0103-the-tile-value-field-stores-a-dense-delta.md`
[^DENSE2]: Findings register, FND-292. `docs/FINDINGS.md`
[^ITEM297]: Backlog item 0297, take the rest of the holding spread. `docs/backlog/complete/0297-take-the-rest-of-the-holding-spread.md`
[^DENOM297]: Findings register, FND-296. `docs/FINDINGS.md`
