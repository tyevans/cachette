# Report 39: What one tick of the training world costs

This report answers one question. Where does the time of one world tick go?

An earlier report split the wall clock of a training generation and found
that the simulation holds between 96 and 99 percent of it.[^1] It named the
inside of a tick as the largest thing nobody had measured. This report
measures it.

**The short answer is that the training tick spends most of its time deriving
one field, and it derives that field four times.** A late tick of the training
world spends 10.9 of its 15.3 milliseconds on the destination field. One of
those derivations is the barrier rebuild. The other three come from inside a
verb, and each one rebuilds the whole field.

**The report found a second defect while it measured.** The influence solve
starts one thread for each faction and each pass when the caller asks for one
thread, to relax four cells. It is 8.5 percent of an early training tick, and
the trainer asks for one thread. Section 4.4 holds it.

## 0 Provenance, and what this report could not verify

**Every figure in this report is a development-machine figure.** The machine
is x86-64 and the target platform is AWS Graviton.[^2] One blocker governs
every cost figure in this project, and it governs these.[^3] No figure here
belongs in the target platform register.[^4]

The report holds three kinds of claim, and each one is marked.

**Measured.** The author ran the code in this repository and read the clock.
Section 1 states the machine and section 2 states the instrument.

**Derived.** The author computed the figure from a measured one. Every
derived figure states the arithmetic.

**Read.** The author read the source and states what it does.

The author started no instance and stopped none. The author changed no file
outside this report, the stage list, the step, the crate manifest, and one new
benchmark. Section 2 states each change.

## 1 The machine, the shape, and the method

The machine holds one Intel Core i7-1260P. It reports 16 logical processors.
Eight of them sit on four performance cores that share two threads each, and
eight sit on eight efficiency cores. The performance cores reach 4.7 GHz and
the efficiency cores reach 3.4 GHz. **The cores are not alike, and section 8
shows that this changes a published figure by a factor of two.**

The frequency governor is `performance`. Transparent huge pages are set to
`madvise`. The compiler is the dated nightly the project pins. The build
profile is `release`, which sets link-time optimisation across one code
generation unit.

**Every figure below is pinned to one performance core, unless the text says
otherwise.** An unpinned run on this machine measures the scheduler as much as
the engine.

The shape is the shape the trainer runs. The world is 48 tiles wide and 48
tiles high. It holds three factions. The learner takes seat zero, and the
world marks that seat externally controlled, so the built-in controller drives
the other two. The tick limit is 2500 ticks.[^5]

**The benchmark takes no learner action.** The trainer calls one action verb
every ten ticks, and that verb costs 0.0033 milliseconds.[^1] The benchmark
leaves seat zero idle instead. A passive seat changes what the world becomes,
so it changes the figures of section 5. It does not change the stage
structure.

## 2 The instrument, and what it costs

The engine already names every pass of a frame and records what each one
costs. The table sits behind a crate feature that is off by default. It reads
a clock and writes two integers to a static, and no pass reads either, so it
cannot change a result.[^6]

The author made four changes.

1. **A new benchmark, and its entry in the crate manifest.** It builds the
   world the trainer builds, steps it, and prints the stage table. The other
   benchmark builds a world with no settlement and no controller, so it does
   not run the passes that hold most of a training tick.[^7]
2. **Nested stages under the level 1 rebuild.** That stage held 66 percent of
   an early frame and nothing divided it.
3. **Nested stages under the controller, one for each part of it and one for
   each verb it applies.** That stage holds 59 percent of a late frame.
4. **One nested stage inside the send verb.** Section 6 is the reason.

Every stage is nested, so none of them adds to the frame total. Every span the
author added is opened in the way the stages before it are opened, and a span
is an empty type when the feature is off. The author read that property in the
source rather than in the generated code.

**The instrument costs about one percent, and that is inside the noise.** The
author ran the same window five times in each build.

| Build | Frame, us |
|---|---|
| Without the feature | 4152, 4205, 4179, 4168, 4269 |
| With the feature | 4223, 4204, 4244, 4199, 4254 |

The median without the feature is 4179 microseconds and the median with it is
4223. The difference is 1.1 percent. The spread inside one build is 2.8
percent, so the instrument is smaller than the noise it sits in.

**The state hash does not change with the thread count.** The benchmark prints
it. One tick window at 1, 2, 4 and 12 threads gives `ee6a364726835770` every
time, so the instrument keeps the determinism rule.[^6]

### 2.1 The commands

```
cargo build --release --bench training_tick --features stage-cost
taskset -c 0 <binary> window 48x48 3 1 5 20
taskset -c 0 <binary> window 48x48 3 1 1200 20
taskset -c 0 <binary> episode 48x48 3 1 25 100
```

The `window` arguments are the extent, the faction count, the thread count,
the warm-up ticks, and the frames to average over. The `episode` arguments
give the window count and the frames in each window instead. Two further
arguments state the unit reservation and the seed.

The tables below are the median of five runs, unless the text says otherwise.

## 3 The stage breakdown

The table gives one tick of the training world on one performance core, at one
thread. **Early** is the window from tick 5 to tick 25. **Late** is the window
from tick 1200 to tick 1220. A nested row divides the row above it and is not
added again.

| Stage | Early, us | Share | Late, us | Share |
|---|---|---|---|---|
| `controller` | 666.71 | 16.8 % | 9205.17 | 59.4 % |
| `controller_apply` (nested) | 89.81 | 2.3 % | 8197.54 | 52.9 % |
| `send_derive_destinations` (nested) | 86.06 | 2.2 % | 8182.57 | 52.8 % |
| `controller_project` (nested) | 87.70 | 2.2 % | 5458.45 | 35.2 % |
| `rebuild_level_1` | 2646.56 | 66.6 % | 5218.40 | 33.7 % |
| `rebuild_destinations` (nested) | 863.13 | 21.7 % | 2741.34 | 17.7 % |
| `controller_cross` (nested) | 0.00 | 0.0 % | 2734.18 | 17.6 % |
| `rebuild_home_approaches` (nested) | 1213.58 | 30.6 % | 1274.48 | 8.2 % |
| `rebuild_stock` (nested) | 544.07 | 13.7 % | 1156.83 | 7.5 % |
| `controller_solve_plan` (nested) | 575.80 | 14.5 % | 617.82 | 4.0 % |
| `controller_states` (nested) | 0.96 | 0.0 % | 382.25 | 2.5 % |
| `influence_solve` | 337.81 | 8.5 % | 351.56 | 2.3 % |
| `observe` | 31.00 | 0.8 % | 213.19 | 1.4 % |
| `holding_spread` | 92.11 | 2.3 % | 200.82 | 1.3 % |
| `movement_intents` | 15.74 | 0.4 % | 43.28 | 0.3 % |
| `tile_scan` | 41.59 | 1.0 % | 37.99 | 0.2 % |
| `admit` | 2.00 | 0.1 % | 26.49 | 0.2 % |
| `rebuild_pyramid` (nested) | 20.71 | 0.5 % | 23.65 | 0.2 % |
| `settle_positions` | 21.50 | 0.5 % | 23.00 | 0.1 % |
| `gather` | 23.05 | 0.6 % | 17.00 | 0.1 % |
| `choose` | 9.64 | 0.2 % | 11.73 | 0.1 % |
| **all stages** | 3969.15 | | 15502.00 | |
| **frame wall** | 3971.05 | 100 % | 15504.81 | 100 % |

The table leaves out every row that falls below 10 microseconds in both
windows. The sum over the stages and the wall time of the same frames agree
to one part in two thousand, so no measurable part of the tick sits outside a
span.

**Three facts stand out.**

The tile scan is 1.0 percent of an early frame. The world holds 2304 tiles, and
every pass that walks them is nearly free.

The passes that move the units are under one percent together. Those are the
choice, the intents, the admission and the placement. The world holds six
units early and about thirty-eight late, so every pass that walks the units is
nearly free.

**Everything else is field derivation and controller planning.** The level 1
rebuild and the controller together hold 83 percent of an early frame and 93
percent of a late one.

### 3.1 What the level 1 rebuild is made of

The rebuild is not one pass. It rebuilds the pyramid, then derives five
fields. The pyramid rebuild is 0.5 percent of the frame. **The five field
derivations are the rest.**

Three of the five carry the cost. The home approach field steers a unit back
to a city. The stock field steers a gatherer to stock. The destination field
steers a sent unit to where it was sent.

**The three are not the same kind of field, and the difference decides how
each one scales.**

The home approach field and the stock field work at the pitch of one tile,
inside a block. Each derives one entry for each pair of a plane and a block
that a seed lies in. A block is 32 tiles on a side whatever the world size is,
so a block holds 1024 tiles. The relaxation runs a fixed pass count, and it
asks the terrain for the ground of every tile of the block.[^8] The terrain is
generated and never stored, so that question costs a noise evaluation for each
tile.[^9] **The cost of one entry is therefore a constant, and the entry count
is what varies.**

The destination field works at the pitch of a level 1 cell. It relaxes over
the cells of each plane, and the plane count is three for each faction. It
asks no terrain, because a cell already summarises its own open ground.

Section 4 measures how each one moves.

## 4 How each stage scales

### 4.1 The tile count is almost not a cost

The table gives the same early window at five extents, with three factions and
six units in every case. The unit count does not change, so the tile count is
the only quantity that moves.

| Stage, us | 24x24 | 48x48 | 96x96 | 192x192 | 384x384 |
|---|---|---|---|---|---|
| frame wall | 2251 | 3929 | 5629 | 5922 | 7895 |
| `rebuild_level_1` | 1506 | 2600 | 4179 | 4036 | 4546 |
| `rebuild_home_approaches` | 703 | 1197 | 1518 | 1367 | 1437 |
| `rebuild_stock` | 300 | 527 | 1606 | 1593 | 1627 |
| `rebuild_destinations` | 496 | 842 | 1008 | 942 | 1017 |
| `controller` | 449 | 657 | 679 | 732 | 831 |
| `tile_scan` | 23 | 40 | 110 | 381 | 1514 |
| `rebuild_pyramid` | 2 | 21 | 39 | 111 | 404 |
| `influence_solve` | 0.4 | 331 | 335 | 352 | 411 |

The tile count rises by 256 times, from 576 to 147,456. The frame rises by 3.5
times.

**The three field derivations saturate.** They stop growing above 96 tiles on
a side. Six units cannot occupy more than six blocks, and three settlements
cannot seed more than three, so the entry count stops rising while the world
keeps growing.

**Two stages are truly linear in the tiles.** The tile scan and the pyramid
rebuild both walk every tile. Together they are 1.6 percent of the frame at
48 tiles on a side and 24 percent at 384.

### 4.2 The faction count is the cost

The table gives the same early window at five faction counts, at 48 tiles on a
side.

| Stage, us | 2 | 3 | 6 | 12 | 24 |
|---|---|---|---|---|---|
| frame wall | 2686 | 3993 | 8376 | 15567 | 21648 |
| `rebuild_level_1` | 1868 | 2644 | 5088 | 8603 | 11036 |
| `rebuild_home_approaches` | 836 | 1220 | 2305 | 3814 | 5032 |
| `rebuild_destinations` | 482 | 851 | 1929 | 3463 | 4644 |
| `controller` | 311 | 666 | 2037 | 4661 | 6768 |
| `influence_solve` | 220 | 330 | 659 | 1352 | 2749 |
| `rebuild_stock` | 524 | 550 | 835 | 1295 | 1314 |
| `tile_scan` | 41 | 41 | 42 | 42 | 41 |

Twelve times the factions cost 8.1 times the frame. The faction count drives
the plane count of two of the three fields, and it drives the controller and
the influence solve directly.

**The faction count and the unit count move together here.** The seeding gives
each faction a founding group, so a world with more factions holds more units.
The author could not separate the two axes with the trainer's own world, and
section 11 marks that.

### 4.3 More threads make the training tick slower

The table gives the same early window at five thread counts. The process ran
over twelve logical processors: the eight that sit on the four performance
cores, and four efficiency cores.

| Stage, us | 1 | 2 | 4 | 8 | 12 |
|---|---|---|---|---|---|
| frame wall | 4447 | 5159 | 4577 | 5724 | 5542 |
| `rebuild_level_1` | 2616 | 2728 | 2712 | 3185 | 2824 |
| `controller` | 671 | 691 | 698 | 739 | 709 |
| `weather_solve` | 7 | 240 | 417 | 701 | 693 |
| `tile_scan` | 82 | 86 | 94 | 156 | 201 |
| `contest` | 22 | 37 | 52 | 97 | 143 |
| `convert` | 22 | 36 | 53 | 92 | 127 |

**One stage gets faster and several get much slower.** The world holds 2304
tiles, so a stage that starts a thread pays more for the thread than the work
costs. The weather solve rises by a factor of 100. The one stage that falls is
the influence solve, and section 4.4 shows that the fall is not a speedup.

The two largest stages take no thread count at all, so a thread count cannot
help them.

**The run at 8 and at 12 threads reaches the efficiency cores.** The author
pinned the process to twelve logical processors, and four of them are
efficiency cores. Part of the rise in those two columns is the slower core and
not the thread. The columns at 1, 2 and 4 threads have performance cores
available for every thread, and they already show no gain.

**The trainer already runs one thread for each world, and that is the right
setting.**[^5] This section says the setting is not merely convenient. It is
faster than any other on this shape.

### 4.4 The second defect: the influence solve starts threads to avoid threads

The influence solve costs 499 microseconds at one thread and 2.4 microseconds
at four, on the same world, with the same state hash. A four-way split cannot
give a factor of 208, so the author read the source.[^10]

**The guard is the wrong way round for one thread.** The relaxation takes a
single-threaded path when the cell count is at most the thread count, and it
spawns otherwise. A 48 by 48 world holds four level 1 cells, because a block
is 32 tiles on a side. At four threads the guard reads four against four, it
takes the single-threaded path, and the pass costs nothing. **At one thread
the guard reads four against one, so the solve spawns a thread to do the work
of four cells.**

It spawns one for each faction and for each pass. The solve makes eight
passes, so three factions cost 24 thread starts for each frame.

**The measurement fits.** 499 microseconds over 24 starts is 20.8 microseconds
for one start, which is the cost of a thread on this machine. Section 4.2
gives the confirmation: the solve is exactly linear in the faction count, at
about 110 microseconds for each faction, which is 8 passes at about 14
microseconds each.

**The extent sweep proves the guard.** A 24 by 24 world holds one level 1
cell, so at one thread the guard reads one against one and takes the cheap
path. Section 4.1 measures the solve at 0.4 microseconds there and at 331
microseconds on the 48 by 48 world beside it, which holds four cells. **The
world doubled in each direction and the solve grew by 800 times.**

**The thread count sweep confirms it a third way.** At two threads the solve
splits four cells into two chunks, so it starts two threads for each faction
and each pass instead of one. Section 4.3 measures the solve rising from 610
to 843 microseconds between one thread and two. A stage that gets slower when
the thread count doubles is starting threads, not using them.

**The trainer runs one thread for each world, so it pays this on every training
tick.** The solve is 8.5 percent of an early training frame and 2.3 percent of
a late one. A world with 24 factions pays 2.75 milliseconds.

The guard is one condition in one function. The author did not change it. **A
guard that avoids threads for small work must also cover the case of one
thread**, which is the case the trainer runs.

## 5 What the cost does through an episode

The table gives one episode of 2500 ticks, in windows of 100 frames, at seed
zero. The learner seat stays idle.

| First tick | Live units | Frame, us | `controller`, us | `rebuild_level_1`, us |
|---|---|---|---|---|
| 0 | 6 | 7575 | 1299 | 4760 |
| 100 | 15 | 10795 | 3055 | 5735 |
| 200 | 24 | 14038 | 4763 | 7318 |
| 400 | 21 | 23040 | 11614 | 8816 |
| 600 | 30 | 24416 | 13992 | 8971 |
| 1000 | 36 | 23496 | 13707 | 8770 |
| 1500 | 34 | 21269 | 12356 | 8723 |
| 2000 | 37 | 22723 | 13536 | 8491 |
| 2400 | 37 | 20366 | 11313 | 7757 |

**A late tick costs 3.9 times an early one.** The canonical pair of section 3
gives 3971 microseconds early and 15,505 late. The earlier report gave a fall
in throughput for each live world from 5.90 to 4.43 ticks a second, which is
25 percent.[^1] The direct measurement gives a much larger rise. The author
does not know which part of the difference is the passive seat and which part
is the earlier method.

**The controller causes it.** It rises from 1.3 to 14.0 milliseconds. The
level 1 rebuild rises from 4.8 to 9.0, which is 1.9 times, and every other
stage is flat.

The figures in this table were taken while the machine ran another job. Read
the shape of the column and not the value of a row. The canonical values are
in section 3.

## 6 The one defect worth attacking

The controller cost divides completely. The table gives the same episode.
**This table is one run and not a median, and the machine ran another job
during it.** Read the shape of a row and not the value of a cell. The
canonical values are in section 3.

| Stage, us | tick 0 | 100 | 200 | 400 | 600 | 1000 | 2400 |
|---|---|---|---|---|---|---|---|
| `controller` | 1299 | 3055 | 4763 | 11614 | 13992 | 13707 | 11313 |
| `controller_prologue` | 2 | 4 | 4 | 6 | 7 | 7 | 7 |
| `controller_solve_plan` | 882 | 893 | 934 | 934 | 932 | 1037 | 924 |
| `controller_states` | 3 | 148 | 265 | 634 | 702 | 684 | 683 |
| `controller_plan` | 0 | 0 | 1 | 1 | 1 | 2 | 1 |
| `controller_apply` | 291 | 1846 | 3476 | 9889 | 12367 | 12159 | 9854 |
| `controller_project` | 285 | 1039 | 2206 | 4959 | 7655 | 7858 | 5634 |
| `controller_cross` | 0 | 620 | 1153 | 4133 | 4272 | 3883 | 3772 |
| `controller_settle` | 0 | 179 | 108 | 786 | 430 | 407 | 440 |

**The whole rise is in two of the eleven verbs the controller applies.** The
project order sends the units that took a project to the tiles of those
projects. The crossing order sends the idle units that cross water to a place
across the water.

Both call the send verb. **The send verb ends by deriving the whole
destination field.** The author added one span inside the verb to price it.

The late window gives this.

| Row | Entries for each frame | Cost for each frame, us |
|---|---|---|
| `rebuild_destinations` | 1 | 2705 |
| `send_derive_destinations` | 3 | 8152 |
| `controller_apply` | 1 | 8166 |
| frame wall | 1 | 15337 |

**The destination field is derived four times in one tick.** The barrier
derives it once. Each of the three sends derives it again, from the start,
over every plane and every seeded block. The four derivations are 10.9 of the
15.3 milliseconds of the frame, which is 71 percent.

The send verb has a reason for the derivation. A caller may read a direction
between two steps, and a derived value that one path leaves stale is a
confident wrong answer. A finding records the case.[^11]

**The reason justifies one derivation, not one for each send.** Three sends in
one tick produce three whole rebuilds, and only the last one survives to be
read. The barrier rebuild before them is a fourth, and the sends overwrite it.

**Two passes read the field, and both run near the start of a frame.** The
movement pass reads it as one of the things it steers by, and the release of
sent units reads it to see whether a unit arrived. The barrier rebuild and the
controller both run near the end of a frame, and nothing between them reads
the field. The author read the call sites to establish this.

**The author did not fix this.** The task was to measure. Two shapes of fix
are visible from the outside, and both need an owner who reads the record that
governs the field.

- Mark the field stale and derive it when a reader asks. The read path then
  pays once, whatever the number of sends.
- Let the controller collect its sends and derive once after the last one. The
  verb keeps its own derivation for the caller that uses it alone.

**What each fix would buy.** The figures below are derived from the measured
rows above. The late frame is 15.34 milliseconds, the barrier derivation is
2.71, and the three send derivations are 8.15 together, so one costs 2.72.

The second fix collects the sends and derives once after the last one. The
send count falls from three derivations to one, which saves 5.44
milliseconds. A late frame falls to 9.90 milliseconds, which is 1.55 times.

The first fix defers the derivation to the reader. **The first reader is the
movement pass of the next tick**, which takes the field as one of the things it
steers by. A field that derives on the first read therefore derives once for
each tick, whatever the send count is. Four derivations become one, which
saves 8.14 milliseconds. A late frame falls to 7.20 milliseconds, which is
2.13 times.

**The first fix is worth almost twice the second, and it is the harder of the
two.** It needs an owner who reads the finding that put the derivation in the
verb.[^11]

Over a whole episode the gain is smaller, because an early tick makes no send
at all. At seed zero the project and crossing orders are 46.3 percent of the
mean frame over 2500 ticks. **That share is a ratio taken on a loaded
machine.** A ratio survives the load better than a value does, and the two
orders are within one percent of the send derivation in the late window, so
they stand for it. Collecting the sends gives 1.45 times over the
episode, and deferring them gives 1.86 times. **A learner throughput figure
would rise by about that much, because section 7 shows that the tick is nearly
all of the training clock.**

### 6.1 This also explains why worlds cost different amounts

The earlier report measured 512 worlds and found the dearest cost 10.15 times
the cheapest.[^1] The author measured eight seeds in the same late window, on
one pinned performance core.

| Seed | Sends for each tick | Frame, us | `send_derive_destinations`, us | Live units |
|---|---|---|---|---|
| 0 | 3.00 | 14902 | 7906 | 38 |
| 1 | 1.15 | 8472 | 2390 | 17 |
| 2 | 0.00 | 4888 | 0 | 20 |
| 3 | 0.00 | 4993 | 0 | 15 |
| 4 | 0.00 | 5832 | 0 | 24 |
| 5 | 0.35 | 7763 | 742 | 20 |
| 6 | 1.50 | 11364 | 3737 | 27 |
| 7 | 1.25 | 11912 | 2776 | 62 |

**The send count predicts the frame and the unit count does not.** Seed 7
holds 62 units and costs less than seed 0, which holds 38. The spread from
4888 to 14902 microseconds is 3.05 times, and 79 percent of the difference
sits in the send derivation.

The same eight seeds in the early window spread only 1.36 times, from 3533 to
4787 microseconds, because no faction sends yet.

**A fix to the send therefore also flattens the schedule.** The earlier report
showed that a fixed assignment over unequal jobs wastes the difference, and it
recommended a greedy schedule.[^1] A world set whose costs agree needs no
greedy schedule.

## 7 What is not the simulation

The author drove the same world from Python, through the built extension, and
timed the step call.

| Path | Window | Cost for each tick, us |
|---|---|---|
| Rust benchmark, feature off | tick 5 to 25 | 4179 |
| Python, through the binding | tick 5 to 25 | 4032 |

The two agree to 3.5 percent, and the Python figure is the smaller of the two.
**The boundary crossing, the lock and the event batch cost less than the noise
of this pair.** Whatever a tick costs, that cost is simulation work.

The Python run met contention and the Rust run did not, and the Python figure
is still the smaller of the two. The conclusion is therefore safe. A contended
run cannot be cheaper than the same work uncontended.

The episode mean over 2500 ticks at seed zero is 18 to 23 milliseconds for
each frame, on one performance core, taken while the machine ran another job.
Read it as an order of magnitude. Section 6.1 shows that the value depends on
the seed more than it depends on anything else.

### 7.1 The derived unit structure is free in training and not at the target

The step rebuilds the unit-to-tile bridge at its barriers. The bridge is
derived structure and not simulation, so it belongs in this section. The step
opens four separate refresh stages, and the table gives all four.

| Stage | Early, ns | Late, ns |
|---|---|---|
| `bridge_refresh_opening` | 26 | 49 |
| `bridge_refresh_barrier` | 112 | 1834 |
| `bridge_refresh_after_reap` | 138 | 55 |
| `bridge_refresh_closing` | 34 | 76 |
| **All four** | **310** | **2014** |

**The four together are 0.008 percent of an early training frame and 0.013
percent of a late one.** A world of 2304 tiles and about thirty-eight units
rebuilds its bridge in microseconds.

**At the target extent the same barrier refresh is 5.0 percent of the
frame**, which is 41.8 milliseconds.[^4] The rebuild orders on one thread by
decision, so it does not divide.[^12]

**This changes what a stray rebuild costs, and the change is by four orders of
magnitude.** A caller-facing verb that leaves the bridge stale forces a
rebuild that the step would otherwise not make. In the training world that
rebuild is free and no measurement would ever find it. At the target it is 42
milliseconds. **A reader who prices that defect from a training profile would
conclude it does not matter.** It matters at the size the project targets, and
only there.

## 8 What the 9.533 millisecond figure measures

The earlier report gives 9.533 milliseconds for one world tick, as the mean
over 512 worlds of the training shape at 20 ticks each.[^1] **The author tried
to falsify it and found that the figure carries the machine in it.**

The same window, on the same tree, gives this.

| Placement | Frame, us |
|---|---|
| Alone on a performance core | 3636 |
| Alone on an efficiency core | 9247 |
| Two threads of one performance core | 7507 |
| All 16 logical processors loaded, performance cores | 8471 to 8544 |
| All 16 logical processors loaded, efficiency cores | 11249 to 11660 |

The mean over the sixteen loaded processors is 10.0 milliseconds. The
efficiency core alone gives 9.2. **The published 9.533 lies between them, and
it is 2.6 times the cost of the same tick on an idle performance core.**

**The earlier report states that it timed one world at a time on one core.**[^1]
Under that reading the contention rows do not apply, and the efficiency core
row is the match. An unpinned process migrates, and over 512 worlds it visits
both kinds of core. The author cannot say which cores the earlier run used,
because the earlier run did not pin and did not record the placement.

Two readings follow, and the author states both.

**The measurement is sound and the number is real.** A trainer that runs
sixteen workers on this machine does pay about that much for a tick. Nobody
should quote 3.6 milliseconds as the cost of a training tick on this machine.

**The number is not a property of the engine.** It is the cost of the engine on
a loaded machine with two kinds of core. The target is a 64-core Graviton
server with one kind of core and no shared threads.[^2] Every conclusion drawn
from the 9.533 as an engine figure must be taken again there.

**The 10.15 ratio between worlds has two causes and the earlier report names
one.** Section 6.1 measures a 3.05 ratio from world content on one fixed core.
This section measures a 2.54 ratio from the core alone. The two together give
7.7, which is near the published 10.15 and does not reach it. The author did
not close the remaining gap.

## 9 What dominates at the target scale

The target is 16,777,216 tiles and 1,000,000 units.[^13] **The training shape
is not a small version of the target. It is a different shape, and the two
have almost no hot stage in common.**

The target platform register holds a measured stage table at the target
extent, taken on a Graviton instance at 12 threads.[^4] The table below sets
it beside the training frame of section 3. **The two columns come from
different machines and different worlds, so read the shares and never the
difference between the values.**

| Stage | Training frame, early | Target frame, measured |
|---|---|---|
| `holding_spread` | 2.3 % | 61.5 % |
| `change_merge` | not in this tree | 14.4 % |
| `rebuild_level_1` | 66.6 % | 9.3 % |
| `bridge_refresh_barrier` | under 0.1 % | 5.0 % |
| `admit` | 0.1 % | 3.5 % |
| `tile_scan` | 1.0 % | 2.0 % |
| `influence_solve` | 8.5 % | 1.6 % |
| `controller` | 16.8 % | no row |

**The two stage lists are not the same list.** The target table holds a stage
this tree no longer has, and this tree holds stages the target table does not
name. A row with no counterpart is marked rather than compared.

**The measured target world holds no settlement and no seated faction.**[^4]
The register says so. That world therefore founds nothing, and it seeds
neither the home approach field nor the destination field. The controller
drives a seated faction, and that world has none. **The stages that hold most
of a training tick have no row in the only measurement this project has at the
target extent.**

**This is the gap, and it is the finding of this section.** Nobody has measured
a world at the target extent that holds settlements, a controller and sends.
The stages that dominate the training tick are exactly the ones that no target
measurement covers.

### 9.1 What the two measurements together do say

**The level 1 rebuild does grow with the world, and its share falls.** It is
2.6 milliseconds of a 4.0 millisecond training frame and 78.0 milliseconds of
an 836 millisecond target frame. Something else grows faster.

**The holding spread is what grows faster.** It is 0.09 milliseconds of a
training frame and 514 milliseconds of a target frame. The register divides it
further: one inner stage that takes no thread count holds 49.1 percent of the
target frame on its own. **A reader who plans against the training profile
would attack the wrong stage for the target, and a reader who plans against
the target profile would attack the wrong stage for training.**

**The author cannot bound the field derivations at the target.** The naive
extrapolation multiplies the per-entry cost of section 3.1 by the block count
of the target and gives a figure in seconds. The measured target frame is 836
milliseconds, so either the extrapolation is wrong or the measured world seeds
almost nothing. Two explanations fit, and the author tested neither. The
measured world may seed almost no entry, because it holds no city and may hold
no tile stock, and the extrapolation would then still hold for a world that
does seed. The per-entry cost may instead fall sharply when many blocks are
derived in one pass, and the extrapolation would then be wrong. **Do not quote a
derived field cost at the target from this report. There is not one.**

### 9.2 The tile count matters at the target and not in training

**Two passes walk every tile, and they are 1.6 percent of a training frame.**
Section 4.1 measures the tile scan and the pyramid rebuild rising in step with
the tile count while every other stage saturates. At 384 tiles on a side they
are 24 percent of the frame. At the target extent the tile scan alone is 16.5
milliseconds.[^4]

**A stage that is small in training and linear in the tiles is a target
problem. A stage that is large in training and saturates is not.** The three
field derivations saturate in the tiles and grow with the planes and the
occupied blocks, so they follow the faction count and the settlement count
rather than the world size.

### 9.3 What to measure next at the target

Run the stage table at the target extent on a world that was seeded, that
holds settlements, and whose factions the controller drives. That single run
would price the three field derivations, the controller and the send
derivation at the scale the project targets. Every other question in this
section follows from it.

## 10 One defect found on the way

The stage list declares how many times one frame opens each stage, and a test
drives one frame and compares the table against that declaration.[^14] **The
declaration for the inner candidate stage of the holding spread is wrong.**

The list says the step opens `holding_candidates` once for each frame. The
first run the author made, on the tree as it stood and before any change in
this report, opened it 40 times over 20 frames. The test fails with
`holding_candidates: declared 1, opened 2`.

This is one value declared in two places, with nothing that fails when the
copies disagree, which is the defect shape this project meets most often.[^15]
Here the check does fail, so the shape is caught rather than silent. **The
failure is not the author's and the author did not repair it.** The
declaration or the step is wrong, and whoever owns the spread should say
which.

## 11 What the author could not measure

- **The target platform.** Every figure here is a development-machine figure.
  One blocker governs them.[^3]
- **A sampling profile.** The kernel of this machine sets the performance
  event limit to 4, so no sampling profiler could open a counter. The author
  used the project's own stage table instead. That table names a pass rather
  than a function, so a cost inside one pass is not divided further than the
  spans go.
- **The unit count apart from the faction count.** The trainer's world seeds
  one founding group for each faction, so the two rise together. A separate
  fixture would be needed to hold one fixed.
- **The learner in the loop.** The benchmark leaves the learner seat idle. An
  acting seat changes what the world becomes over 2500 ticks.
- **The remaining part of the 10.15 ratio of section 8.**

## 12 What to do next, in order

1. **Derive the destination field once for each tick.** Section 6 gives the
   measurement and two shapes of fix. The cheaper one takes a late training
   tick from 15.3 to 9.9 milliseconds. The better one takes it to 7.2. Both
   also flatten the spread between worlds that the earlier report found.
2. **Correct the thread guard of the influence solve.** Section 4.4 measures
   it and names the condition. It is 8.5 percent of an early training frame,
   it is one condition in one function, and three separate sweeps agree on the
   cause.
3. **Run the stage table at the target extent on a seeded world.** Section 9
   shows that the only measurement this project has at that extent holds no
   settlement and no controller, so it prices none of the stages that hold a
   training tick. One run closes the largest gap in the cost picture.
4. **Take the 9.533 millisecond figure again, pinned.** Section 8 shows that
   it carries the machine. Any scheduling decision made from it should be made
   from a pinned figure instead.

## References

[^1]: Report 38, where the training time goes. `docs/research/reports/38-where-the-training-time-goes.md`
[^2]: ADR-0008, the primary target is aarch64. `docs/adrs/accepted/adr-0008-the-primary-target-is-aarch64.md`
[^3]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^4]: Target platform costs. `docs/reference/graviton-costs.md`
[^5]: The learner environment, the world configuration. `python/cachette/learn/env.py`
[^6]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^7]: The target cost benchmark. `crates/cachette-core/benches/target_cost.rs`
[^8]: ADR-0005, a solver runs a fixed iteration count. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
[^9]: ADR-0068, terrain is generated from the seed and is never stored as a map. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
[^10]: The influence field, the relaxation pass. `crates/cachette-core/src/influence.rs`
[^11]: Findings register, FND-029. `docs/FINDINGS.md`
[^12]: ADR-0071, the bridge rebuild orders on one thread. `docs/adrs/accepted/adr-0071-the-bridge-rebuild-orders-on-one-thread.md`
[^13]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^14]: The stage cost test. `crates/cachette-core/tests/stage_cost.rs`
[^15]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
