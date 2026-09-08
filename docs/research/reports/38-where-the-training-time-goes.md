# Report 38: Where the training time goes

This report answers one question. Would training go faster on a machine with
a graphics processor, and what would that cost in work?

The report starts with a measurement, because every recommendation depends on
it. Nobody had split the wall clock of a training generation before. Section
2 holds that split. Sections 3 to 8 read the published work and the prices
against it.

**The short answer is no.** The simulation holds between 96 and 99 percent of
the wall clock, the policy arithmetic holds less than one tenth of one
percent, and the engine is heavy branching logic that a graphics processor
runs badly. Section 9 states what would have to change for the answer to
flip.

## 0 Provenance, and what this report could not verify

The report holds four kinds of claim, and each one is marked.

**Measured.** The author ran the code in this repository on a development
machine and read the clock. Every measured figure names the script that
produced it. Section 1 states the machine.

**Measured by the project owner.** Two throughput figures come from the
target platform. The author did not take them and did not rent a machine.

**Derived.** The author computed the figure from a measured one. Every
derived figure states the arithmetic.

**Read.** The author read a paper, a repository or a price page, and quotes
the figure from it. Every read figure names its source in a footnote.

**No figure in this report describes the target platform, except the two the
owner supplied.** The development machine is x86-64 and the target is AWS
Graviton. One blocker governs every cost figure in this project, and it
governs these.[^1]

The author started no instance, stopped none, and modified none. The author
ran no test suite and no gate. The author changed no file outside this
report.

**One hypothesis in this report was falsified by the author's own
measurement.** Section 4.3 records it, because the falsification is worth
more than the hypothesis was.

## 1 The machine, the shape, and the moment

The development machine holds one Intel Core i7-1260P. It reports 16 logical
processors over 4 performance cores and 8 efficiency cores. **The cores are
not alike, and section 4.3 shows that this changes a result.** The target is
a 64-core Graviton server with no simultaneous multithreading and cores that
are alike.

The training shape is the one the trainer runs. The world is 48 tiles wide
and 48 tiles high. It holds three factions, and the learner takes one seat.
The tick limit is 2500 ticks. One decision covers 10 ticks, so the horizon is
250 decisions. A generation holds 24 candidates over 6 seeds, which is 144
worlds.[^2]

The observation array holds 184 positions. The action table holds 29 rows.
The linear policy therefore holds 5,365 weights.

Every measurement describes the tip of the main branch on 7 September 2026.
The extension was built in the release profile.

The owner supplied two figures from the target platform. One process with 64
workers over 512 worlds reached 2,935 ticks a second. Five processes of 12
workers each reached 1,231 ticks a second apiece, which is about 6,000
together.

## 2 The measured split

### 2.1 The simulation holds nearly all of the clock

The author replicated the trainer loop with a clock around each phase. The
control run called the trainer's own function on the same worlds. The two
totals agree.

The table gives one generation of 144 worlds over 20 decisions.

| Workers | Wall clock | Ticks a second | Simulation | Python |
|---|---|---|---|---|
| 1 | 309.12 s | 93.2 | 99.52 % | 0.46 % |
| 2 | 208.27 s | 138.3 | 99.22 % | 0.77 % |
| 4 | 121.85 s | 236.3 | 98.82 % | 1.14 % |
| 8 | 67.72 s | 425.3 | 97.70 % | 2.22 % |
| 12 | 53.49 s | 538.5 | 97.35 % | 2.58 % |
| 16 | 44.73 s | 643.8 | 96.45 % | 3.47 % |

The Python share rises with the worker count, because the simulation divides
over the workers and the Python section does not. It reaches 3.47 percent at
16 workers.

### 2.2 What the Python section is made of

The table gives the cost of one world for one decision, at 16 workers.

| Phase | Cost | Share of the Python section |
|---|---|---|
| The simulation | 14.9808 ms | — |
| The reward and the second observation | 0.2622 ms | 47.5 % |
| The legality mask | 0.1679 ms | 30.4 % |
| The first observation | 0.1091 ms | 19.8 % |
| The policy arithmetic | 0.0092 ms | 1.7 % |
| The action | 0.0033 ms | 0.6 % |
| The live scan | 0.0001 ms | 0.0 % |
| The whole Python section | 0.5518 ms | 100 % |

**The policy arithmetic is 1.7 percent of the Python section and 0.06 percent
of a decision.** Section 3 takes this further.

### 2.3 The observation is built three times, and once is enough

The author wrapped a world in a proxy that counts every call, then drove the
trainer loop through it. The count is what the loop does, not what a reader
of the code expects.

Over 10 decisions of one world the loop called `step` 100 times,
`faction_observation` 30 times, `legal_actions` 10 times and `act` 10 times.
**That is three observation builds for each world for each decision.**

Three callers ask for it. The trainer stacks the observations before it
chooses.[^3] The reward reads the array to find its own fields.[^4] The step
result carries the array back to the caller.[^3]

The build is the expensive reader. The table gives one world at tick 100, on
one core.

| Reader | Cost |
|---|---|
| `faction_observation` | 0.4351 ms |
| `Reward.read` | 0.3958 ms |
| `observation_schema` | 0.0163 ms |
| `legal_actions` | 0.0072 ms |
| `act` | 0.0004 ms |
| `game_end` | 0.0001 ms |

`Reward.read` costs what one observation build costs, because that is what it
does. The mask costs 60 times less than the observation.

The build grows through an episode. The phase split above measures 0.109 ms
for each build over the first 20 decisions, when the worlds are still small.
This table measures 0.435 ms at tick 100. Both describe their own moment.

**Two of the three builds are waste.** Removing them cuts the Python section
by about 40 percent early in an episode, and by more later. That figure is
derived from the two tables above.

### 2.4 The commands that produced the split

The scripts are not in the repository. A reader who wants to repeat the work
writes them from the description above. The three that matter are short.

1. Replicate the trainer loop with `time.perf_counter` around each phase.
   Sweep the worker count from 1 to 16 at 144 worlds. Call the trainer's own
   function on the same worlds as a control.
2. Time each reader on one world, after 200 ticks, over 2000 repeats.
3. Wrap a world in a proxy that forwards each call and counts it. Drive the
   trainer loop through the proxy for 10 decisions.

## 3 The policy arithmetic does not matter, and will not

The policy is a 184 by 29 matrix product for each decision. The author timed
it on the processor and on the integrated graphics processor of the same
machine.

The processor figures use NumPy over OpenBLAS. The graphics processor figures
use one OpenCL kernel over integer arithmetic, with one write, one kernel
launch, one read and one wait.

| Batch | Processor, linear | Graphics processor, linear |
|---|---|---|
| 144 | 0.058 ms | 0.588 ms |
| 512 | 0.168 ms | 1.039 ms |
| 4,096 | 3.251 ms | 5.973 ms |
| 65,536 | 16.933 ms | 80.794 ms |

**The graphics processor loses at every batch size the author tried,
including one 128 times larger than a generation.** A blocking kernel round
trip on that device costs 79.0 microseconds on its own, which is more than
the whole product costs on the processor at 512 worlds.

The kernel is naive, and a tuned kernel would do better. It does not matter.
The processor already answers a whole generation in 0.168 ms, which is
0.01 percent of a decision. There is nothing left to win.

### 3.1 The size at which the answer would change

The author timed two larger networks on the processor, at the same batch
sizes.

| Batch | 184 to 24 to 29 | 184 to 512 to 512 to 29 |
|---|---|---|
| 144 | 0.202 ms | 9.639 ms |
| 512 | 0.499 ms | 22.484 ms |
| 4,096 | 6.050 ms | 167.851 ms |
| 65,536 | 79.980 ms | 1712.388 ms |

The hidden-layer policy this project trains holds 24 hidden units. It costs
0.499 ms for a generation of 512 worlds.

A network of 512 units in two hidden layers holds about 370,000 parameters,
which is 69 times this project's policy. It costs 22.484 ms for 512 worlds.
One decision of 512 worlds on the target costs about 1.744 seconds of
simulation, derived from the owner's figure of 2,935 ticks a second. So even
that network is 1.3 percent of a decision.

**The policy would need about a thousand times more arithmetic before it
reached a tenth of a decision.** Move the policy to a graphics processor when
the network is that large, and not before.

## 4 Where the parallel loss is

The Python section is small. The simulation is nearly everything. So the
owner's gap between 2,935 ticks a second and about 6,000 is not a Python
problem, and section 2 rules it out.

### 4.1 The batch assigns worlds by a fixed stride

The batch builds its workers once and keeps them. Worker `w` takes the worlds
at positions `w`, `w + workers`, and so on. `Pool::run` sends one order to
each worker and reads one report from each before it returns.[^5]

Two things follow. The assignment never changes for the life of the batch. A
step costs the time of the slowest worker, and the batch pays that on every
tick.

### 4.2 The worlds do not cost the same

The author timed each world's own step, one world at a time, on one core.
Over 512 worlds of the training shape at 20 ticks each, the mean was 9.533 ms
for one world-tick. The cheapest world cost 2.014 ms and the dearest cost
20.441 ms, a ratio of 10.15.

That spread is the whole difficulty. A fixed assignment over unequal jobs
wastes the difference.

The author then computed what four schedules would cost over the measured
cost matrix. The arithmetic is exact and does not depend on the clock of the
machine. The figures give the fraction of a perfect schedule that each one
reaches, over 144 worlds.

| Workers | Stride, barrier each tick | Stride, one barrier | Greedy, barrier each tick |
|---|---|---|---|
| 8 | 87.5 % | 89.7 % | 99.5 % |
| 12 | 84.7 % | 87.2 % | 99.1 % |
| 16 | 79.5 % | 82.2 % | 98.7 % |
| 32 | 64.3 % | 69.8 % | 94.5 % |
| 64 | 48.6 % | 51.5 % | 87.3 % |

At 512 worlds the same model gives 72.1 percent at 64 workers and 91.0
percent at 12 workers.

Two readings follow. **The barrier frequency is worth almost nothing**, which
is 48.6 against 51.5 at 64 workers. **The assignment is worth a great deal**,
which is 48.6 against 87.3. The greedy figure assumes a scheduler that knows
each cost in advance, so it is an upper bound, not a target.

The ratio between 12 workers and 64 workers at 512 worlds is 91.0 over 72.1,
which is 1.26. The owner measured a ratio of 2.10. So the schedule explains
part of the gap and not all of it.

### 4.3 The author's own end-to-end test falsified this, and why

The model says the assignment decides the throughput. The author tested that
against the real batch, and the first test found nothing.

The test built 32 worlds and 16 workers. One order paired a dear world with a
cheap one on each worker. The other put the dear worlds together. The two
orders gave 565.1 and 583.6 ticks a second, and both reached about 47 percent
of the arithmetic ideal. **Reordering changed nothing.**

The reason is the machine. At 16 workers every logical processor is busy, the
4 performance cores and the 8 efficiency cores run the same job at different
speeds, and the clock falls under load. That loss is uniform, it is about
2 times, and it hides the schedule loss underneath it.

The author repeated the test on 4 workers, where the job fits the performance
cores. Eight worlds ran 40 ticks. The cheap worlds cost 4.44 to 5.66 ms a
tick and the dear worlds cost 16.87 to 18.03 ms.

| Order | Wall clock | Fraction of a perfect schedule |
|---|---|---|
| A dear world paired with a cheap one | 1.09 s | 82.1 % |
| Two dear worlds on one worker | 1.46 s | 61.4 % |

**The balanced order is 1.34 times faster, on the same worlds, over the same
ticks.** So the schedule loss is real and the first test could not see it.

Two things follow for the report. The model in section 4.2 stands, because it
is arithmetic over measured single-world costs. The end-to-end confirmation
of it cannot be taken on this machine at high worker counts, and the target
platform is where to take it. Graviton has no simultaneous multithreading and
its cores are alike, so the masking effect does not exist there.

### 4.4 What the rest of the owner's gap probably is

The report can account for the remainder but cannot measure it.

Five processes overlap their Python sections. While one process stacks
observations, the other four simulate, so the machine never idles. One
process with 64 workers leaves all 64 idle for the length of its Python
section.

The arithmetic is derived, and the figure is a lower bound. The Python
section costs 0.5518 ms for one world for one decision on this machine.

One process with 64 workers holds all 512 worlds, so its Python section costs
282 ms. Its decision costs about 1.744 seconds of simulation, derived from
2,935 ticks a second. The Python section is therefore at least 13.9 percent
of that decision.

One process of five holds about 102 worlds, so its Python section costs
56 ms. Its decision costs about 0.829 seconds of simulation, derived from
1,231 ticks a second. The Python section is therefore at least 6.3 percent of
that decision.

It is a lower bound because Graviton's single-thread speed is below this
machine's, and the Python section runs on one thread.

**The sharding the owner already built collects most of this.** That is the
important consequence, and section 7 ranks the remaining work against it.

### 4.5 Threads inside one world buy nothing at this shape

The author ran one 48 by 48 world at tick 100 at several thread counts.

| Threads | Cost of a tick |
|---|---|
| 1 | 14.109 ms |
| 2 | 15.490 ms |
| 4 | 13.971 ms |
| 8 | 15.384 ms |

There is no gain, and the spread is inside the noise. The trainer gives one
thread to each world, which is correct.

The cost of a tick also rises with the world size, but the seed spread
swamps a size comparison taken at one seed. The author measured 2.01 to
20.44 ms for one world-tick over 512 seeds of the same size. Treat any single
world-size table as indicative only.

## 5 The published batch simulators, and what each demands

Every figure in this section is read, not measured.

**Madrona** runs the whole simulation on the graphics processor as a
megakernel, over an entity component system.[^6] It reports 2 million steps a
second for Hide and Seek, 40 million for Overcooked and 20 million for
Hanabi, on an RTX 4090.[^7] The demands are specific and heavy. The simulator
is written in C++. It needs a Volta or newer NVIDIA device and CUDA 12.4 or
newer, and the graphics processor path is Linux only. Every archetype must be
declared in advance, and a component cannot be added to or removed from an
entity at run time.[^8] The repository calls itself a research code base with
breaking changes ahead.[^8]

**Brax** is written in JAX and simulates rigid bodies.[^9] It reports
millions of physics steps a second on one TPU, and hundreds of millions
across 64 TPU chips.[^9] It reaches that by avoiding branches, and the state
is floating point.

**Isaac Gym** runs PhysX on the device and hands tensors to PyTorch without a
host copy.[^10] It reports training speedups of two to three orders of
magnitude against a processor simulator on continuous control.[^10] It
targets robot physics, and its state is floating point.

**PGX** is written in JAX and holds board games: backgammon, chess, shogi, Go
and smaller boards.[^11] It reports 10 to 100 times the throughput of the
Python implementations of the same games, on a DGX-A100.[^11] The demand is
that the state fits fixed-size tensors and the rules avoid data-dependent
control flow.

**EnvPool** stays on the processor. It is C++ with an asynchronous
executor.[^12] It reports one million Atari frames a second and three million
MuJoCo frames a second on 256 cores, which is 14.9 and 19.6 times a naive
vectorised environment.[^12] On a 12-core machine it reports 3.1 and 2.9
times.[^12] The demand is that the environment is C++ and reentrant.

**PufferLib** also stays on the processor. Its environments are written in C
and it reports millions of steps a second.[^13]

The pattern is plain. **Every graphics-processor entry demands that the
simulation be rewritten in its own language and its own execution model.**
None of them takes an existing engine.

## 6 What the determinism rule does to this

Read this section against the determinism record. It says that one binary
gives one answer, byte for byte, at any thread count, and that this claim
outranks every other constraint.[^14]

### 6.1 The float ban removes the usual problem

The published difficulty with a reduction on a graphics processor is that
float addition is not associative, so a sum whose order depends on the warp
scheduler is not reproducible.[^15] A float atomic add commits in an order
the hardware chooses, and that order is not stable across runs.[^15]

**This project banned floats in simulated and aggregated state, so that
problem does not reach our aggregates.** Integer addition is associative and
exact, and an integer atomic add gives the same total in any order. Integer
overflow wraps deterministically. The pyramid accumulators are `i64`.

This is a real and non-obvious advantage of the float ban. It is the one part
of the picture that a move to a graphics processor would not have to fight.

### 6.2 Four things it would still have to guarantee

The float ban is not enough. An implementation would have to promise all four
of these.

1. **A stable key sorts every parallel result.** An order that comes from
   thread completion or from work stealing is forbidden.[^14] A reduction
   that only sums is safe. A reduction that picks a winner is not, so two
   units that move onto one tile must be resolved by a sort, not by whichever
   thread arrives first.
2. **Every random draw comes from a counter keyed on the system, the frame,
   the entity and the draw.** A per-thread generator destroys determinism.
   This rule transfers to a graphics processor without change, and a
   counter-based generator suits one well.
3. **Every solver runs a fixed iteration count.** No convergence test and no
   time budget.[^14] A device that varies its occupancy varies its timing, so
   a time budget would vary the result.
4. **Padding is declared.** An event type is plain data, and an
   uninitialised byte in a state hash is false nondeterminism.

**No batch simulator the author read states a bit-exact guarantee across
thread counts, block counts or devices.** Brax and Isaac Gym hold float
state, so they cannot give one. Madrona makes no determinism claim in its
repository or on its site, and the author did not read the SIGGRAPH paper, so
mark that as unverified rather than absent.

The consequence is that adopting any of them would mean taking the
determinism claim back, or proving it again from the outside. The record says
the claim outranks the speed.[^14]

## 7 Could this engine move, and should it

**This is a rewrite, not a port, and it is not sensible now.**

The engine is Rust. It holds a hex world with units, cities, markets,
weather, upgrades and a built-in controller, and the logic is heavily
branched. That is the least suited shape for a device that runs one
instruction across many lanes. A branch that diverges across a warp costs
both sides.

Madrona takes C++ in its own entity component system with archetypes declared
in advance.[^8] The JAX entries take fixed-size tensors and near-branchless
rules.[^9] [^11] None of them takes Rust. There is no incremental path: a
partial port would put the world state on the device and the branching logic
on the host, and the transfer would cost more than the logic saves.

The measurement makes the argument anyway. The Python side and the boundary
are 3.5 percent of the clock at 16 workers. **Even a perfect graphics
processor cannot win more than the simulation costs, and the simulation is
the thing that would have to be rewritten.** A rewrite that succeeds
completely replaces work that is already 96 percent of the clock, and gives
up the determinism claim while it does so.

## 8 The cost of renting a graphics processor

Every price here is read from one price aggregator, in its default region, on
7 September 2026. Confirm a price against AWS before spending.

| Instance | vCPU | Device | On demand | Spot |
|---|---|---|---|---|
| `c7g.16xlarge` | 64 | none | $2.320 [^16] | $0.697 [^16] |
| `g6.xlarge` | 4 | 1 NVIDIA L4 | $0.805 [^17] | $0.696 [^17] |
| `g6.12xlarge` | 48 | 4 NVIDIA L4 | $4.602 [^18] | $3.591 [^18] |

The owner's run cost about $4.60 over six hours, which is about $0.77 an
hour. That is the spot price of the Graviton instance.

**A comparison per unit of useful work cannot be made, because the engine
does not run on a graphics processor.** There is no throughput figure to
divide. What the table does say is the shape of the trade.

At almost the same spot price, a `g6.xlarge` gives 4 cores where the
`c7g.16xlarge` gives 64. This engine uses cores and nothing else, so renting
that machine today buys one sixteenth of the compute for the same money.

A `g6.12xlarge` gives 48 cores and four devices for 5.2 times the spot price
of the Graviton machine. Its cores are x86-64, which is a development target
for this project and not the target platform.

So the graphics-processor instances are not cheaper for the work this project
does, and the cheaper one is far more expensive for it.

## 9 Local testing on Intel integrated graphics

The author tested this rather than reasoning about it.

The development machine holds an Intel Iris Xe integrated graphics processor,
Alder Lake-P GT2. The Intel OpenCL driver is installed. The device reports
OpenCL 3.0 with 96 compute units at 800 MHz.

**OpenCL works on this hardware, and the author ran integer kernels on it.**
Section 3 holds those figures. So the mechanism is available with no rented
machine and no new dependency beyond the driver.

**It is a path for correctness and not for performance.** Three reasons, and
each one is specific.

1. **The device is too small to predict the rented one.** 96 execution units
   of an integrated part share the memory bandwidth of the processor. An L4
   does not.
2. **It does not test the thing that would be adopted.** Madrona needs Volta
   or newer NVIDIA hardware and CUDA, and it will not run on this device at
   all.[^8] A local OpenCL result says nothing about a Madrona result.
3. **The measurement it can take has already been taken and settles the
   question the other way.** The one candidate small enough to test locally
   is the policy arithmetic, and section 3 shows the processor wins at every
   size tried.

SYCL and oneAPI would run on this device, and wgpu would run on it through
Vulkan. The portability claim of each is real for a kernel written against
it. It is thin for a framework: none of the batch simulators in section 5
targets any of them, so portability at the kernel level does not buy
portability at the level that matters here.

**The honest use of this hardware is to prove that a kernel gives the same
integer answer as the Rust code**, if the project ever writes one. That is
worth having and it is cheap. It is not a performance test.

## 10 The ranking, and the answer

### 10.1 A plain answer on the graphics processor

**No. Do not pursue a graphics processor now.**

The reasoning is four measured facts. The simulation is 96 to 99 percent of
the clock. The policy arithmetic is 0.06 percent of a decision and loses on
the device at every size tried. The engine is branching logic in Rust, and
every batch simulator demands a rewrite in its own language. The instances
cost more per core than the machine already in use.

The determinism finding is the one part that favours the idea, and it is not
enough on its own. Integer state removes the reduction-order problem that
governs the float engines. Four other guarantees would still have to hold,
and no published simulator states them.

### 10.2 The changes, ranked by gain against work

The ranking is against the owner's current run, which already shards across
five processes.

1. **Nothing large is left on this axis.** The sharding already collects most
   of the parallel loss. Five processes of 12 workers reach 91.0 percent of a
   perfect schedule in the model, against 72.1 percent for one process of 64.
   This is the most useful thing the report found, and it says the next win
   is elsewhere.
2. **Build the observation once for each world for each decision.** The loop
   builds it three times, and the count is measured. This cuts the Python
   section by about 40 percent early in an episode and by more later. It is
   the cheapest change in the report: give the reward the array the trainer
   already holds, and return that same array in the step result. It helps
   every shard. Expect a few percent of the wall clock, not a multiple.
3. **Give the batch a dynamic assignment, keeping the sort by world index.**
   The model says the fixed stride costs 28 percent at 512 worlds over 64
   workers. The determinism argument is unchanged, because the batch already
   sorts its results by the world index and would keep doing so. Measure it
   on the target platform, because section 4.3 shows this machine cannot see
   it. This removes the need to shard rather than adding to it.
4. **Move the legality mask into the boundary call that steps.** The mask is
   0.0072 ms and the observation is 0.4351 ms, so moving the mask alone wins
   little. Do it only when it comes free with change 2.
5. **Make one tick cheaper, or run fewer of them.** One world-tick of the
   training shape costs 9.533 ms on average on this machine. That single
   number sets the throughput of every training run. It is the largest lever
   in the report and the report does not measure inside it. Somebody should.

Changes 2 and 3 are small and worth making. Neither is a multiple. **The
throughput of a training run is the cost of a tick, and everything in this
report that is not the cost of a tick is a few percent.**

### 10.3 What would have to be true for the answer to flip

Revisit the graphics processor when one of these holds. None holds today.

- **The policy grows about a thousand times.** Section 3.1 gives the
  arithmetic. A network of 512 units in two hidden layers is still only 1.3
  percent of a decision, so the threshold is well past that.
- **The simulation stops being the bottleneck.** If the cost of a tick fell
  by a large factor, the boundary and the policy would become visible. They
  are not visible now.
- **A batch simulator appears that takes Rust and states a bit-exact
  guarantee across thread counts and devices.** Nothing the author read does
  either.
- **The engine is rewritten for another reason**, and the rewrite can choose
  its execution model freely. Then the question is open again, and section
  6.2 lists what the implementation would have to promise.
- **Graphics-processor instances become cheaper per core than Graviton.** The
  table in section 8 says they are not, by a wide margin.

## 11 What this report did not do

- It ran no training run, and no claim about learning outcomes appears in it.
- It took no figure on the target platform. Two figures come from the owner.
- It did not read the Madrona SIGGRAPH paper, so its determinism properties
  are unverified rather than absent.
- It read abstracts, repository documentation and price pages for section 5
  and section 8. It did not read the Brax, Isaac Gym, PGX or EnvPool papers
  in full.
- It wrote no kernel that reproduces any part of the engine, so no claim
  about porting difficulty rests on an attempt.
- It ran no test suite and no gate, and it changed no file outside itself.
- It did not measure inside one tick. Section 10.2 names that as the largest
  open lever.

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: The trainer entry point, the world every strategy plays. `python/cachette/learn/__main__.py`
[^3]: The learner environment and the trainer loop. `python/cachette/learn/env.py` and `python/cachette/learn/train.py`
[^4]: The reward module. `python/cachette/learn/reward.py`
[^5]: The batch of worlds and its worker pool. `crates/cachette-py/src/batch.rs`
[^6]: Shacklett and others, An Extensible, Data-Oriented Architecture for High-Performance, Many-World Simulation, SIGGRAPH 2023, abstract read 7 September 2026. https://purl.stanford.edu/js274bf0548
[^7]: Madrona Engine, the reported throughput figures, read 7 September 2026. https://madrona-engine.github.io/
[^8]: The Madrona repository, the requirements and the stated limitations, read 7 September 2026. https://github.com/shacklettbp/madrona/blob/main/README.md
[^9]: Freeman and others, Brax, A Differentiable Physics Engine for Large Scale Rigid Body Simulation, NeurIPS 2021 Datasets and Benchmarks, read 7 September 2026. https://arxiv.org/abs/2106.13281
[^10]: Makoviychuk and others, Isaac Gym, High Performance GPU-Based Physics Simulation For Robot Learning, 2021, read 7 September 2026. https://arxiv.org/abs/2108.10470
[^11]: Koyamada and others, Pgx, Hardware-Accelerated Parallel Game Simulators for Reinforcement Learning, NeurIPS 2023, read 7 September 2026. https://arxiv.org/abs/2303.17503
[^12]: Weng and others, EnvPool, A Highly Parallel Reinforcement Learning Environment Execution Engine, 2022, read 7 September 2026. https://arxiv.org/abs/2206.10558
[^13]: PufferLib documentation, read 7 September 2026. https://puffer.ai/docs.html
[^14]: ADR-0001, one binary gives one answer at any thread count, decisions D1, D3 and D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^15]: Chowdhury and others, Impacts of floating-point non-associativity on reproducibility for HPC and deep learning applications, 2024, read 7 September 2026. https://arxiv.org/abs/2408.05148
[^16]: Vantage instance pricing, `c7g.16xlarge`, default region, read 7 September 2026. https://instances.vantage.sh/aws/ec2/c7g.16xlarge
[^17]: Vantage instance pricing, `g6.xlarge`, default region, read 7 September 2026. https://instances.vantage.sh/aws/ec2/g6.xlarge
[^18]: Vantage instance pricing, `g6.12xlarge`, default region, read 7 September 2026. https://instances.vantage.sh/aws/ec2/g6.12xlarge
