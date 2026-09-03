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

The figures below come from one run, on one instance, on one date. They cover
four operations of the public crate interface. They do not cover the passes
that a world with settlements runs, because the measured world holds none.

## How to take a figure

Run the script. It launches an instance, copies the tracked files to it,
builds the benchmark, runs the sweep, brings the rows back, and destroys
everything it made.

```
just graviton-bench full
just graviton-orphans
```

The script needs the AWS command line tool, authenticated. The run below cost
about one cent and took about seven minutes, and the build was most of that
time. The second command lists what a run left behind, and it must list
nothing.

The benchmark also runs on a development machine, and a figure taken there
belongs in neither this register nor any other.[^7] Use it to check the
apparatus.

```
just bench quick
```

## The machine

Every figure below was taken on this machine, and on no other.

| Fact | Value |
|---|---|
| Instance type | `c7g.large` |
| Region | `us-west-2` |
| Processor | Graviton3. Implementer `0x41`, part `0xd40` |
| Hardware threads | 2 |
| Cache line | 64 bytes |
| Memory | 3,897,492 kB |
| Kernel | `Linux 6.18.44-99.149.amzn2023.aarch64` |
| Compiler | `rustc 1.91.1 (ed61e7d7e 2025-11-07)` |
| Build profile | `bench`, which inherits `release` |
| Commit | `9aaf6443f80ebfdb35348d679d7c23604bb3807b` |
| Date | 3 September 2026 |

**The instance holds two hardware threads.** The project targets a server, and
a server holds many more. Read the thread columns below as the scaling this
machine showed, and not as the scaling a large instance would show.

**The instance is not burstable.** A burstable instance earns processor
credits and falls back to a fraction of one core when they run out. A sweep of
this length would exhaust them, and every row after that point would measure
the throttle.

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
project rests on it.[^8]

## One frame at the target scale

This is the row the project needs most.

| Threads | Samples | Minimum, ns | Median, ns | Maximum, ns |
|---|---|---|---|---|
| 1 | 6 | 1,840,105,650 | 1,860,922,702 | 1,940,664,749 |
| 2 | 9 | 1,119,920,790 | 1,135,293,186 | 1,190,617,672 |
| 4 | 9 | 1,125,529,839 | 1,139,342,771 | 1,170,654,514 |

The world held 16,777,216 tiles and 1,000,000 units, and no settlement.

**The frame budget is 100 milliseconds.** The engine runs at ten ticks for
each second, and the scale constants table derives that rate.[^2] The median
frame above costs 1,135 milliseconds at two threads. That is 11.4 times the
budget on this machine.

**The frame holds 1.86 core-seconds of work.** The one-thread row is that
figure. A frame of 100 milliseconds therefore needs a speedup of at least 18.6
against one core, so it needs at least 19 cores even when every core is used
perfectly. This machine reached a speedup of 1.64 on two cores, so it used
0.82 of the two cores it had.

**A fourth thread bought nothing.** The instance holds two hardware threads,
and the four-thread row matches the two-thread row.

## The cost of a frame, as two straight lines

The measured cost is the tile count times a constant, plus the unit count
times a second constant.

| Quantity | Cost at 1 thread | Cost at 2 threads |
|---|---|---|
| One tile, one frame | 78 ns | 43 ns |
| One unit, one frame | 557 ns | 411 ns |

The two constants predict the target scale row. At two threads they give
1,129 milliseconds against the 1,135 milliseconds measured, which is a
difference of one part in two hundred.

The unit constant comes from the difference between a world of 1,000,000 units
and a world of none, at 4,194,304 tiles. The same difference at 100,000 units
gives 417 ns and at 10,000 units gives 350 ns, so the line is straight over
two orders of magnitude.

**The units scale worse than the tiles.** The tile pass ran 1.81 times faster
on two threads than on one. The unit passes ran 1.35 times faster.

## One frame against the tile count

The world holds no unit in every row of this table.

| Tiles | Threads | Samples | Minimum, ns | Median, ns | Maximum, ns | Median, ns for each tile |
|---|---|---|---|---|---|---|
| 4,096 | 1 | 9 | 1,472,983 | 1,511,341 | 1,642,037 | 369 |
| 4,096 | 2 | 9 | 2,448,516 | 2,535,514 | 3,670,996 | 619 |
| 4,096 | 4 | 9 | 235,419 | 242,654 | 276,706 | 59 |
| 65,536 | 1 | 9 | 4,683,312 | 4,997,363 | 5,378,755 | 76 |
| 65,536 | 2 | 9 | 4,509,280 | 4,845,675 | 5,827,293 | 74 |
| 65,536 | 4 | 9 | 6,952,238 | 7,678,044 | 8,506,648 | 117 |
| 1,048,576 | 1 | 9 | 72,321,895 | 74,683,858 | 84,260,887 | 71 |
| 1,048,576 | 2 | 9 | 38,095,061 | 38,983,900 | 46,100,844 | 37 |
| 1,048,576 | 4 | 9 | 40,833,011 | 45,872,330 | 51,309,765 | 44 |
| 4,194,304 | 1 | 9 | 286,187,036 | 291,980,590 | 335,700,630 | 70 |
| 4,194,304 | 2 | 9 | 164,716,910 | 168,199,802 | 194,785,952 | 40 |
| 4,194,304 | 4 | 9 | 158,612,382 | 166,862,993 | 193,992,172 | 40 |
| 16,777,216 | 1 | 8 | 1,270,605,624 | 1,303,922,093 | 1,457,765,572 | 78 |
| 16,777,216 | 2 | 9 | 707,988,028 | 718,533,494 | 811,507,732 | 43 |
| 16,777,216 | 4 | 9 | 730,095,296 | 742,332,248 | 842,049,703 | 44 |

**The rows at 4,096 tiles disagree with the rest of the table, and nobody
has explained why.** One thread costs 369 ns for each tile
and four threads cost 59 ns, on a machine with two hardware threads. Every
other extent costs between 37 and 78 ns for each tile. The result repeated
across nine samples with a spread under one fifth, so it is not noise. A
backlog item holds the question.[^9] Do not cite the 4,096-tile rows.

## One frame against the unit count

Every row below holds 4,194,304 tiles.

| Units | Threads | Samples | Minimum, ns | Median, ns | Maximum, ns |
|---|---|---|---|---|---|
| 0 | 1 | 9 | 303,919,358 | 309,426,222 | 352,424,250 |
| 0 | 2 | 9 | 164,958,771 | 168,031,578 | 193,613,809 |
| 0 | 4 | 9 | 159,873,272 | 165,813,439 | 190,615,890 |
| 10,000 | 1 | 9 | 310,294,762 | 315,203,913 | 358,105,768 |
| 10,000 | 2 | 9 | 168,839,527 | 171,504,795 | 196,420,606 |
| 10,000 | 4 | 9 | 163,614,638 | 168,373,126 | 194,237,013 |
| 100,000 | 1 | 9 | 362,946,177 | 365,723,806 | 391,955,811 |
| 100,000 | 2 | 9 | 206,043,380 | 209,690,746 | 230,021,514 |
| 100,000 | 4 | 9 | 200,561,755 | 207,498,707 | 231,548,700 |
| 1,000,000 | 1 | 9 | 840,637,132 | 866,042,558 | 927,499,751 |
| 1,000,000 | 2 | 9 | 550,258,972 | 578,770,080 | 624,622,056 |
| 1,000,000 | 4 | 9 | 559,232,237 | 582,273,957 | 626,056,798 |

## Building a world

| Tiles | Samples | Minimum, ns | Median, ns | Maximum, ns | Median, ns for each tile |
|---|---|---|---|---|---|
| 4,096 | 9 | 1,153,188 | 1,156,689 | 1,274,970 | 282 |
| 65,536 | 9 | 18,065,869 | 18,071,591 | 18,254,339 | 276 |
| 1,048,576 | 9 | 287,286,297 | 287,321,050 | 289,491,597 | 274 |
| 4,194,304 | 9 | 1,155,862,111 | 1,156,171,565 | 1,156,818,869 | 276 |
| 16,777,216 | 3 | 4,630,439,413 | 4,630,699,305 | 4,633,008,922 | 276 |

**Building a world costs 276 ns for each tile, at every extent measured.**
Building a world at the target extent therefore takes 4.63 seconds, and it
takes them on one core. The tile value field generates a tile and stores
nothing, so it visits no tile.[^10] The first level of the pyramid does visit
every tile, and the constructor rebuilds it. A proposed backlog item holds
that pass and names the two shapes that would remove it.[^11]

**Reserving a million unit slots costs nothing that this run could see.** A
build at the target extent with the default reservation took a median of
4,631,106,264 ns, against 4,630,699,305 ns with a reservation of 1024. The
difference is one part in ten thousand, and it is inside the spread of both
rows. The reservation record states that the cost is paid once, at
construction, and that the cost of a tick does not grow with it.[^12] This run
gives the first measured support for the first half of that claim.

## The hash of the whole world

| Tiles | Samples | Minimum, ns | Median, ns | Maximum, ns | Median, ns for each tile |
|---|---|---|---|---|---|
| 4,096 | 9 | 1,273,541 | 1,278,276 | 1,292,892 | 312 |
| 65,536 | 9 | 21,283,563 | 21,343,512 | 21,505,996 | 326 |
| 1,048,576 | 9 | 351,362,737 | 351,741,590 | 351,997,609 | 335 |
| 4,194,304 | 7 | 1,426,173,459 | 1,427,107,778 | 1,445,243,855 | 340 |
| 16,777,216 | 3 | 6,018,511,641 | 6,023,536,841 | 6,023,813,980 | 359 |

The hash costs about 340 ns for each tile, and it takes one core. A golden
state test at the target extent therefore costs 6.02 seconds for each frame it
checks. The hash regenerates the ground, the stock and the tile value, because
each of those is generated from the seed rather than stored.

## What this run did not measure

Read this section before you cite a figure above.

- **The measured world holds no settlement.** The rate pass, the consumption
  pass and the position pass therefore did no work. Every frame figure above
  is a lower bound on a frame at the target scale, and not the whole of it.
- **The machine holds two hardware threads.** No figure above says what a
  large instance does. The thread columns give the scaling of two cores.
- **One run, one instance, one date.** A second run on a second instance of
  the same type would give a second set of figures, and nobody has taken one.
- **No figure above measures a cache hit rate, an allocation count, or the
  cost of a call across the language boundary.** Three draft records state
  derived figures of those kinds, and this run leaves every one of them
  derived.[^2]
- **The benchmark measures four operations.** The step, the build and the hash
  reach the public interface. No figure separates the stages inside a step.

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
[^8]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^9]: Backlog item 0229. `docs/backlog/proposed/0229-explain-the-frame-cost-at-the-smallest-extent.md`
[^10]: ADR-0088, a tile field is a generated base and a stored change. `docs/adrs/draft/adr-0088-a-tile-field-is-a-generated-base-and-a-stored-change.md`
[^11]: Backlog item 0171. `docs/backlog/proposed/0171-build-the-first-level-without-a-pass-over-every-tile.md`
[^12]: ADR-0084, the world reserves the unit columns at construction, decision D3. `docs/adrs/draft/adr-0084-the-world-reserves-the-unit-columns-at-construction.md`
