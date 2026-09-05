# Design: One environment core serves every learning stack

Date: 2026-09-05
Status: Draft for review

## The problem

The reinforcement learning design gives a learner one faction, a bounded
action table, a flat observation, a reward in Python, and a wrapper over N
worlds.[^1] The research report that supports it names many backing systems
a project like this one might use: policy gradient trainers, a set of
primitives, a cluster framework, high-throughput samplers, a batched
environment pool, JAX stacks, behaviour cloning, evolution strategies and
league self-play.[^2] Each one asks for the environment in a different shape.

The project cannot write one environment for each. It must write one for all
of them, and survey the backing systems against it. This document says what
the interfaces must look like so that the survey is possible. It names
structure and holds no tuned figure. Every figure it needs is a row in one
reference table under the blocker that governs cost figures.[^3] [^4]

The owner has ruled that the harness exposes to a learner only what a player
of that faction could see.[^5] Every interface here is written under that
ruling, and this document does not revisit it.

## 1 The boundary

The environment is three layers. Each layer has one job.

### 1.1 The engine boundary

The engine boundary is Rust behind the Python binding crate. It exposes batch
readers and batch verbs over N worlds in one call. One call steps every
world, one observes every learner faction, one returns every mask, and one
applies every action. Each call releases the interpreter lock for its whole
duration.[^6]

The boundary orders worlds by index. World zero is the first row of every
returned array. A thread pool inside the call may run the worlds in any
order, and the call writes each result into the row its index names.[^7]

The boundary carries an instruction and an answer, never the population.[^8]
A batch reader returns one array shaped by the world count and the schema. It
returns no dictionary and no handle. The schemas of sections 2 and 3 are
answers of this kind: the engine states its own layout, and Python reads it.

### 1.2 The environment core

The environment core is pure Python. It imports the binding crate and a
numeric array library, and nothing else. Its methods are:

- `reset(seeds)`: builds N worlds, runs the seeding layer, sets the
  externally controlled flag for each learner faction, and returns the first
  observation batch.[^9]
- `step(actions)`: applies one action per learner faction per world, steps
  every world K ticks, and returns the observation, reward, done and info
  batches.
- `observe()`, `action_mask()`, `reward()`, `done()`, `info()`: read the same
  values without stepping.
- `replay(seed, action_log)`: builds one world and applies each logged action
  at its tick, then returns the final state hash.

Every return is an array with the world count as its first axis. The core
loops over nothing but worlds and learner factions.[^8] The core has one
shape, and every adapter wraps that shape.

### 1.3 The adapters

An adapter is one thin module per backing system. The adapters this design
plans are:

- A single environment and a vector environment for the standard single
  agent interface.[^10]
- A parallel environment for the standard multi-agent interface.[^11]
- A binding for the sampler that writes into shared buffers.[^12]
- An environment base class for the tensor primitives library.[^13]
- A host callback adapter for the JAX stacks, if section 7 finds it honest.
- A plain callable for evolution strategies, from a parameter vector and a
  seed set to a return per seed.

**What an adapter may do.** It reshapes. It renames a method to the name the
backing system expects. It builds a space object from the schema. It moves an
array into a tensor or a shared buffer. It splits one batch into the
per-agent dictionaries a multi-agent interface wants.


**What an adapter may not do.** It never reads the world. It never calls the
binding crate. It never computes a reward, a mask or a done flag. It never
holds a value the core does not hold. An adapter that does any of these has
become a second environment, and two environments over one engine is the
declaration defect this project names first.[^14]

A check enforces the boundary. The adapter package imports the core and the
backing system, and a test fails when it imports the binding crate.

## 2 The observation contract

The observation of a faction is one flat signed 64-bit integer array of fixed
length.[^1] This section says how a backing system learns the layout without a
Python file that repeats it.

### 2.1 The schema object

The engine returns `observation_schema()`. Each row is one field of the
observation, and a row holds:

| Column | Content |
|---|---|
| Name | The field name, unique in the table |
| Offset | The first position of the field in the flat array |
| Length | The number of positions the field holds |
| Dtype | The integer width the field needs |
| Lower | The lower bound of each position in the field |
| Upper | The upper bound of each position in the field |
| Shape | The declared shape of the field, or empty for a vector |

The engine builds the table from the same constants that build the array.
The offset of a field is the sum of the lengths before it, so a change to one
constant moves every offset after it. A backing system builds its space from
the table: the lower and upper columns become the bounds of a box space, and
the shape column becomes the shape of the map block.

**No Python file hard-codes an offset.** The recurring defect register names
a constant in Rust and the same constant in Python as the first place this
project will meet the declaration shape.[^14] The schema is the one
declaration site. A test builds the observation, reads the schema, and
asserts that each field sits where its row says.

### 2.2 The map block

The map block is the one field with a declared shape: the cell count across,
the cell count down, and the field count per cell. The field count is six:
the five totals the reinforcement learning design names, and the visibility
flag that fog adds.[^1] A backing system builds a convolution from the shape
row. It reads no radius from a Python constant.

### 2.3 Fog is honoured by the engine

The engine applies the mask inside `observe`. A caller passes a faction and
receives what that faction sees. No argument asks for the truth.[^5] A
learner cannot see through fog, because the interface offers no way to ask.

The full-information readers that exist today stay for the modeller and the
reproducer, the two audiences the product record does not serve.[^5] The
environment core does not call them, and the check of section 1.3 catches an
adapter that does.

### 2.4 The schema is versioned

The schema carries one version integer. The engine increments it whenever a
row changes: a field added, removed, relengthened or rebounded. A stored
policy records the version it trained under, and a learner that loads a
policy under a different version stops with an error that names both. A
trained policy is a function of the layout, and a layout change without a
version change would make it silently wrong.

The version is one Rust constant. A test stores a digest of the schema beside
the version, and it fails when the digest moves and the version does not.

## 3 The action contract

An action is one row of a bounded table shared by the controller and the
learner.[^1] This section says how a backing system reads the table, masks
it, and maps a factored policy onto it.

### 3.1 The action schema

The engine returns `action_schema()`. It holds one row per verb:

| Column | Content |
|---|---|
| Verb | The verb kind |
| Candidate kind | What the target list names: a resource kind, an upgrade category, a city, a deposit, a faction, a board row, or nothing |
| Candidate bound | The ceiling on the list length, from the world parameters |
| Magnitude buckets | The bucket count for this verb, or one when the verb takes none |

The flat action count is the sum over rows of the candidate bound times the
bucket count. The schema exposes that sum, and a backing system builds its
discrete space from it. A list shorter than its bound pads to the bound, and
the mask marks the padding invalid.

### 3.2 The factorisation

A flat categorical head over the full table learns slowly, because the table
is a product.[^2] A backing system with three autoregressive heads, one per
field of the row, must map the three choices onto the flat id and back. The
schema declares that mapping.

The flat id is a mixed radix number. For a row with verb v, target t and
magnitude m, the id is the offset of verb v, plus t times the bucket count of
v, plus m. The schema exposes the verb offsets, the candidate bounds and the
bucket counts. A backing system decodes an id by subtraction and division,
and encodes three parts by the inverse. Both directions are arithmetic over
the schema, and neither is a Python table.

The engine returns the mask in the flat shape. A backing system with three
heads derives the three masks from it by the same factorisation. A verb is
valid when any row of it is valid, and a target is valid when any bucket of
it is valid. That derivation is a reshape and a reduction in the adapter.

### 3.3 The mask

`action_mask(faction)` returns one byte per flat id, one when the verb would
not refuse the row now and zero otherwise. The batch form is shaped by the
world count, the learner count and the flat action count. The mask reads the
same candidate lists and refusal rules the verbs read, and duplicates no
rule.[^1] The no-op row is always valid, so no mask is ever all zero.

### 3.4 The batch verb

`act(batch)` takes one integer array. Each row is a world index, a faction
and a flat action id. The engine decodes each id, resolves the candidate list
at the current tick of that world, and applies the row through the same verbs
a Python caller and the controller use.[^15] Rows apply in array order. A row
the verb refuses is dropped and counted, as a controller command is.[^16] A
learner that plays one faction in a thousand worlds makes one call per
decision.[^8]

### 3.5 One log for the controller and the learner

The controller emits a row of the same table, and the engine writes each
applied row to one event log with the tick, the world, the faction and the
flat id.[^1] A learner's row and a controller's row are one event type.
Behaviour cloning reads that log as a labelled dataset: the observation at
the tick, the row chosen, and whether the verb took it. It reads one log and
one encoding. A second log for the learner would be a second declaration of
one fact.[^14]

## 4 The reward contract

The engine exposes `standing(faction)`, which returns one integer per win
path, and `game_end()`, which returns the winner, the path and the tick, or
nothing while the game runs.[^17] It exposes nothing else toward a reward.
Neither reader is new here, and the reinforcement learning design names the
widening of the standing reader.[^1]

**The reward is a Python function object.** The environment core takes it at
construction. It receives the standing before the decision, the standing
after it, the game end record and the learner faction, and returns one number
per world. The core calls it once per step over the batch. It never enters
the engine and never enters the hash.

**A registry of named rewards.** The core holds one dictionary from a name to
a reward function. The terminal and shaped forms the reinforcement learning
design names are its first two entries.[^1] A survey run names its reward, so
two backing systems that name the same reward train on the same signal.

**Per-path components.** The reward function may return one component per win
path beside the scalar. The core exposes the components in the info batch. A
backing system with one value head per path reads them.[^2] A backing system
that wants one scalar ignores them. The weights that combine the components
are rows in the reference table, and this document states none.[^4]

## 5 The batching contract

### 5.1 The world batch

One Rust type, `WorldBatch`, holds N worlds. It exposes four batch calls.

- `step_all(threads)`: steps every world one tick.
- `observe_all()`: returns the observation of every learner faction of every
  world, shaped by the world count, the learner count and the observation
  length.
- `mask_all()`: returns the action mask in the same shape, with the flat
  action count as the last axis.
- `act_all(batch)`: applies the action array of section 3.4.

Each call releases the interpreter lock for its whole duration, and writes
each result into the row its world index names, so the world order is
deterministic whatever the thread pool did.[^6] [^7] The environment core
calls these four and nothing per world.

### 5.2 Worlds per process and threads per world

Two axes trade against each other. A world steps on T threads, and a batch
holds N worlds. The batch call may give each world T threads in turn, or give
N worlds one thread each, or any split between them. Both N and T are
parameters. This document fixes neither.

**The cost shape.** The cost of `step_all` follows N times the frame cost of
one world at T threads, and the blocker names that frame cost as not yet
measured for a world with settlements and a controller.[^3] The cost of
`observe_all` follows N times the learner count times the observation length,
and the cost of `mask_all` follows the same product with the flat action
count. The cost of `act_all` follows the row count of the batch. No cost
follows the tile count or the unit count, because every call reads aggregates
and applies set-valued verbs.

**Where every figure goes.** The reference table holds one row per measured
quantity, with the machine, the commit and the command, as the target
platform table does today.[^4] [^18] Every figure stays derived until the
target platform measures it.[^3]

## 6 The replay and evaluation contract

### 6.1 One seed and one action log give one game

A replay is a seed and an action log.[^1] The core's `replay` builds one
world from the seed, sets the flag for each learner faction, and applies each
logged row at its tick through `act`. It steps to the last logged tick and
returns the state hash. Two replays of one log give one hash, at any thread
count.[^7]

### 6.2 The action log is a golden test

Each logged row is one event: plain data, `repr(C)`, declared padding, no
`bool`, as every event in this engine is.[^19] The engine writes the row when
`act` applies, so the log is the engine's record and not the wrapper's. A
stored log with its final hash is a golden test: build, replay, compare. The
test can fail, because a change to any verb the learner used moves the
hash.[^20] The fixture holds a log from a short game, and the test replays it
at more than one thread count.

### 6.3 The balance harness is the evaluation runner

The balance harness plays a fixed seed set to game end and reports on the
set.[^21] An evaluation run is the same loop with one seat given to a policy.
The harness gains one option, the policy for a seat, and one report column,
the controller of each seat. The report keeps its shape, with nothing about
the machine or the clock in it.

The policy plugs in through the environment core. The harness constructs the
core over the seed set, and on each decision it asks the policy for one
action batch and passes it to `step`. The policy is a callable from an
observation batch and a mask batch to an action batch, and every adapter can
produce one. The harness never calls a backing system.

### 6.4 Ratings are computed outside the engine

Win rate against the controllers saturates, and a rating over a pool of
checkpoints does not.[^2] Python computes Elo or TrueSkill from the game end
records the harness writes. The engine holds no rating, no checkpoint and no
pool. A rating is a training artefact, and a training artefact enters no
hash.

## 7 The survey plan

### 7.1 The matrix

One survey run is one backing system on one adapter.

| Backing system | Adapter |
|---|---|
| Masked policy gradient trainer | Vector environment, single agent |
| One-file policy gradient trainer | Vector environment, single agent |
| Tensor primitives library | Environment base class |
| Cluster framework | Vector environment, single agent |
| Shared-buffer sampler | Shared-buffer binding |
| Asynchronous sampler | Vector environment, single agent |
| Batched environment pool | None; the world batch already has its shape |
| JAX stacks | Host callback adapter, or none |
| Behaviour cloning | The action log, no adapter |
| Evolution strategies | Plain callable |
| League self-play | Parallel environment, multi-agent |

**The batched environment pool** requires an environment written against its
template in its language.[^2] The world batch of section 5 gives the same
shape without the dependency, so the survey measures the world batch and
ports nothing.

**The JAX stacks** gain their speed because the environment step is a
compiled function on the device.[^2] The step here is Rust on the host, so a
host callback breaks the compiled loop at every step. The survey builds the
adapter only if a first measurement shows the crossing is cheap against the
step. If it is not, the row records that the adapter was not built, and
why.

### 7.2 What each run measures

Every run uses the same reward name, the same schema version, the same seed
set and the same decision cadence. A run that changes one of those is not a
survey run. Each run records:

- Wall time per thousand ticks per world, at a stated N and T.
- Sample throughput, as decisions per second across the batch.
- Resident memory of the process at a stated N.
- Ease of masking: where the backing system reads the mask, and how many
  lines the adapter needed for it.
- Whether the backing system installs on the target platform from a wheel,
  from a source build, or not at all.[^22]

The first three are measured on the target platform, and each row names the
machine, the commit and the command.[^18] The last two are observations and
carry a date.

### 7.3 Where the results go

The results go to one reference table and not to a record.[^4] A record holds
a constraint, and a survey result is a measurement that the next release of a
backing system changes.[^23] The table is the evidence a later record cites
when the project chooses a stack.

## 8 Determinism

**Inside the hash.** The world, and the action log. The world holds one flag
per faction and every effect of every command the learner sent through the
verbs. The log holds each applied row with its tick. Two worlds that received
the same rows at the same ticks have the same hash, whatever produced the
rows.[^7]

**Outside the hash.** The policy, its weights, its sampling and its random
state. The reward function and its parameters. The rating pool. The adapter
and the backing system. The engine reads none of these.

**Why the split holds the determinism record.** The record says one binary
gives one answer at any thread count, and two tests hold it to that.[^7] The
learner is a source of commands, as a Python caller is. The engine's
guarantee is that one seed and one command sequence give one world. A learner
whose tensor sums reassociate produces a different command sequence on a
different run, and the engine replays each sequence exactly. The two
determinism tests remain the engine's tests. The learner's variance is a
measurement the evaluation reports over several seeds, and the boundary keeps
every floating point number on the learner's side.[^24]

**What a reviewer checks.** Every batch call writes results by world index
and not by completion order. Nothing in the engine reads a learner value. The
replay test runs at more than one thread count.

## 9 The records this work needs

The scope rule gives a three-condition test. A decision needs a record when a
contributor could reasonably choose otherwise, when choosing otherwise costs
more than changing it later, and when the reasoning is not visible in the
code.[^25] Each candidate below is tested.

**One decision record: the observation and the action of a faction are
schema-declared bounded tables the engine owns.** A contributor could give the
learner a dictionary observation, a Python file of offsets, or an action set
separate from the controller's. Each is shorter. A dictionary breaks the
fixed shape every backing system needs. A Python offset table is a second
declaration site, and nothing fails when it disagrees with the Rust layout. A
separate action set breaks the imitation target and doubles the log. Changing
back after a policy has trained invalidates the policy. None of that
reasoning is visible in a table constant. The record passes. The
reinforcement learning design names this record, and this document widens its
claim from a shared table to a schema-declared one.[^1]

**One decision record: a batch of worlds steps in one call in index order.**
A contributor could step worlds from Python threads, return results in
completion order, or step one world per call. The first works and costs one
call per world per tick. The second is faster and breaks the determinism
record. The third is the slowest. The reasoning for index order is the
determinism record, and a reader of a batch loop does not see it. The record
passes, and the counter-test also applies: a decision that governs
determinism always needs a record.[^25]

**The product record the reinforcement learning design names.** A learner
plays one faction against the controllers. It states the need and no
structure. This document adds one checkable statement to it: a second backing
system uses the environment without a change to the core.[^1]

**Dropped: the three-layer boundary as a record.** A contributor could merge
the adapters into the core. That is a module arrangement, and the scope rule
says not to record one.[^26] The constraint it serves, that an adapter never
reads the world, follows from the control plane record and needs no new
record.[^8] The import check of section 1.3 enforces it.

**Dropped: the reward registry.** A reward is a training choice, it enters no
hash, and it changes at no cost. The second condition fails.[^1]

**Dropped: the schema version.** It is the one workable way to make a layout
change detectable, so the first condition fails. The code and its test are
the record.

**Dropped: the survey matrix.** It is a plan, and a plan is a backlog item.
Each survey run is one item, and its result is a reference table row.

## 10 Sequencing

The reinforcement learning design gives three prerequisites from the game
layer: the territory work, the upgrade table, and the campaign register with
the per-path standing.[^1] The owner's ruling adds a fourth, the observation
plane. This section places the interfaces of this document against all
four.

**Territory.** The candidate lists for the campaign verb and the send verb
name own cities, enemy cities in reach and deposits in reach. A draft record
defines held ground as the ground within reach of an owned city, and the
backlog holds the item the owner asked for now.[^27] [^28] The action schema
of section 3 can be written before it, because the schema declares a
candidate kind and a bound and not a list.

**The upgrade table.** The build row indexes the upgrade table. A draft
record now holds the upgrade categories, and the registry allocates its
number.[^29] The candidate kind of the build row becomes an upgrade category
when that record lands, and its bound becomes the category count. Until then
the bound is the enumeration count the code holds. A second draft record
gives a faction one solver for its roads. It adds no action row, because a
road is a build.[^29]

**Campaigns and standing.** The standing reader with one value per path needs
no prerequisite. The campaign row of the action schema waits for the campaign
register.

**The observation plane.** The reinforcement learning design states what
exists toward it and what is missing: a sight rule, an observed set per
faction, and a remembered reading per cell.[^1] The map block and the
per-faction census wait for it. The schema object, the action schema, the
batch type, the core, the adapters and the replay test do not, because none
of them reads the map. The survey may start over the other blocks and widen
when the plane lands. A survey result taken without the map block is marked
as such in the reference table.

**The order this document adds.**

| Pass | Content |
|---|---|
| A | The two decision records and the product record; the registry rows |
| B | `observation_schema()`, `action_schema()` and the version, over the blocks and rows that exist |
| C | `WorldBatch` with its four calls; the world order test at more than one thread count |
| D | The environment core; the replay test as a golden test; the import check on the adapter package |
| E | The single agent vector adapter and the plain callable; the first two survey runs |
| F | The remaining adapters, one survey run each |
| G | The map block and the per-faction census, after the observation plane |
| H | The candidate lists, as territory, the upgrade table and campaigns land |

Passes A to F need nothing from the four prerequisites. Pass G waits for the
observation plane, and pass H for the three game layer prerequisites.

## Corrections to the brief

The brief and the tree disagreed at two points, and this document follows the
tree.

1. The brief names a backlog item numbered 0486 as the upgrade table. No item
   with that number is in the tree. Section 10 names the upgrade categories by
   subject and cites the registry for the record number.[^29]
2. The brief offers a JAX host callback adapter if it is honest. The research
   report finds that the JAX stacks give no benefit to a host
   environment.[^2] Section 7 makes the adapter conditional on a measurement.

## References

[^1]: Design, a learner plays one faction against the controllers. `docs/superpowers/specs/2026-09-05-reinforcement-learning-design.md`
[^2]: Research report 22, reinforcement learning approaches for one faction. `docs/research/reports/22-reinforcement-learning-approaches.md`
[^3]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^4]: Reinforcement learning costs and parameters, to be created. `docs/reference/rl-costs.md`
[^5]: PRD-0001, a faction sees only what its own units observe. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
[^6]: ADR-0042, the interpreter is released for the whole step. `docs/adrs/draft/adr-0042-the-interpreter-is-released-for-the-whole-step.md`
[^7]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^8]: ADR-0040, Python is a control plane, not a data plane, decisions D1 and D2. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^9]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D6. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^10]: Gymnasium documentation, the vector API, read 5 September 2026. https://gymnasium.farama.org/api/vector/
[^11]: PettingZoo documentation, the parallel API, read 5 September 2026. https://pettingzoo.farama.org/api/parallel/
[^12]: PufferLib documentation, read 5 September 2026. https://puffer.ai/docs.html
[^13]: TorchRL documentation, EnvBase, read 5 September 2026. https://docs.pytorch.org/rl/stable/reference/generated/torchrl.envs.EnvBase.html
[^14]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
[^15]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^16]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D3. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^17]: ADR-0148, a game end is recorded once and stops the controllers, decisions D1 and D2. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^18]: Target platform costs. `docs/reference/graviton-costs.md`
[^19]: Project orientation, the hard invariants. `CLAUDE.md`
[^20]: Testing Rules, section 1. `.agents/rules/testing.md`
[^21]: The balance harness. `python/cachette/balance/__init__.py`
[^22]: Toolchain register. `docs/reference/toolchain.md`
[^23]: Decision Record Scope, section 4.1. `.agents/rules/adr-scope.md`
[^24]: ADR-0002, state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^25]: Decision Record Scope, section 1. `.agents/rules/adr-scope.md`
[^26]: Decision Record Scope, section 4.4. `.agents/rules/adr-scope.md`
[^27]: ADR-0150, held ground is the ground within reach of a city its faction owns. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^28]: Backlog item 0484, hold ground only within reach of an owned city. `docs/backlog/proposed/0484-hold-ground-only-within-reach-of-an-owned-city-and-refuse-a-build-outside-it.md`
[^29]: ADR Registry. `docs/adrs/REGISTRY.md`
