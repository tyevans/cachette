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

## The cost of a frame, as two straight lines

The measured cost is the tile count times a constant, plus the unit count
times a second constant. Machine A.

| Quantity | 1 thread | 2 threads | 16 threads |
|---|---|---|---|
| One tile, one frame | 78 ns | 43 ns | 13 ns |
| One unit, one frame | 561 ns | 416 ns | 303 ns |

The two constants predict the target scale row. At two threads they give
1,132 milliseconds against the 1,120 milliseconds measured, which is a
difference of one part in a hundred.

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

**Result: not yet run.**

## Resident memory

Each row below comes from a process that measured one point and exited. A
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

**A world at the target scale holds 545 MB.** That is 16,777,216 tiles and
1,000,000 units, and no settlement and no character.

**The tiles are the cost, and the units are not.** The same world with no unit
holds 456 MB, so the whole population of one million adds 89 MB. A tile costs
27 bytes and a unit costs 89 bytes.

**A tile costs 27 bytes even though the ground is generated.** Two records
state that a tile field is a generated base with only the change stored, and
that a tile stock is generated with only what was taken stored.[^11] [^12]
Both hold: nothing here stores a tile value or a stock. The 27 bytes are the
columns the world does allocate for each tile, and one proposed item already
names the holder column as one of them.[^13]

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
have wrapped in silence.[^14] The check costs time, so the checked run is a
separate build and gives no timing row. A timing row taken under it would
measure the check.

## One frame against the tile count

The world holds no unit in every row of this table. Machine A.

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
question.[^15] Do not cite the 4,096-tile rows.

**The small extents lose from a high thread count, and that is ordinary.**
At 65,536 tiles a frame costs more at 16 threads than at 2. The step starts
threads for each parallel stage, and at a small tile count that cost is larger
than the work it divides. It is not the same effect as the 4,096-tile rows,
because it rises with the thread count instead of falling.

## One frame against the unit count

Every row below holds 4,194,304 tiles. Machine A.

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

Machine A. The build takes no thread count from the caller.

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
tile.[^11] The first level of the pyramid does visit every tile, and the
constructor rebuilds it. A proposed backlog item holds that pass and names the
two shapes that would remove it.[^13]

**Reserving a million unit slots costs nothing that this run could see.** A
build at the target extent with the default reservation took a median of
4,637,235,543 ns, against 4,636,309,563 ns with a reservation of 1024. The
difference is one part in five thousand, and it is inside the spread of both
rows. The reservation record states that the cost is paid once, at
construction, and that the cost of a tick does not grow with it.[^16] This run
gives the first measured support for the first half of that claim.

## The hash of the whole world

Machine A. The hash takes no thread count.

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
machine, the commit and the date beside the table. A figure without those
three facts is not usable.

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
[^11]: ADR-0088, a tile field is a generated base and a stored change. `docs/adrs/draft/adr-0088-a-tile-field-is-a-generated-base-and-a-stored-change.md`
[^12]: ADR-0072, a tile stock is generated, and only what was taken is stored. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^13]: Backlog item 0171. `docs/backlog/proposed/0171-build-the-first-level-without-a-pass-over-every-tile.md`
[^14]: ADR-0083, the gate build checks every integer overflow. `docs/adrs/draft/adr-0083-the-gate-build-checks-every-integer-overflow.md`
[^15]: Backlog item 0229. `docs/backlog/proposed/0229-explain-the-frame-cost-at-the-smallest-extent.md`
[^16]: ADR-0084, the world reserves the unit columns at construction, decision D3. `docs/adrs/draft/adr-0084-the-world-reserves-the-unit-columns-at-construction.md`
