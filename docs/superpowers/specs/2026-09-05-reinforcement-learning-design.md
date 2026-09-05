# Design: A learner plays one faction against the controllers

Date: 2026-09-05
Status: Draft for review

## 1 The need

This design serves one audience: a researcher or a game developer who wants
to train a policy for one faction while the built-in controllers play the
other factions.

That person cannot do this today. The engine has a controller stage, and the
controller plays every faction that has a seat.[^1] A per-faction flag says
that an external caller controls a faction, and a caller can set it.[^2] The
flag stops the controller. Nothing then plays the faction. The caller has the
verbs, but no bounded action set, no fixed observation vector, no reward, no
episode boundary, and no way to run many worlds in one process. The caller
would build each of these by hand, and would build them against the readers
that exist today, which return dictionaries of named fields and not arrays.

The design builds on the living world game layer.[^3] It names structure, and
it holds no tuned figure. Every figure it needs is a row in a reference
table, and each section names the table.

## 2 What the engine already gives a learner

The engine gives a learner six things that a training loop usually has to
build. Each one is a binding record.

**A world is a pure function of a seed, at any thread count.** One binary
gives one answer at one thread, at two threads, and at twelve threads, and
two tests hold it to that.[^4] A training run that repeats a seed repeats the
world. A result reproduces on another machine.

**A step has no wall clock, and Python does not run while it runs.** The
interpreter lock is released for the whole step.[^5] Every solver and the
controller run a fixed evaluation count.[^1] A step therefore costs the same
work under any load, and a learner that steps many worlds from many threads
in one process meets no interpreter contention inside the step.

**Set-valued verbs are an action space.** A verb takes a set and acts once.
Python sends one command and never loops over the units.[^6] [^7] A discrete
action that names a verb and a target is one crossing, whatever the size of
the set it resolves to.

**Aggregate readers are an observation space.** The region summary, the
window census, the faction population, the relation entry, the market board,
the score and the census each return one bounded answer for one
crossing.[^6] None returns the world.

**The externally-controlled flag exists.** A faction whose flag is set
receives no evaluation from the controller.[^2] This is the hook this design
stands on.

**Four win paths and a game end record exist.** The step writes the record
once, at the first tick a reader fires, and the boundary exposes the running
value of a faction on each path that has a reader.[^8] That is an episode
boundary and a terminal signal.

**A multi-seed runner exists.** The balance harness plays a fixed seed set to
game end and reports on the set.[^9] It loops over seeds and factions only.
The environment wrapper of this design has the same shape.

## 3 Reward

A reward is a scalar that Python computes after each decision. It reads two
things the engine exposes: the running value of the faction on each win
path, and the game end record.[^8] The design names the per-path reader
`standing(faction)` and it returns one integer per path. Today the boundary
exposes `score(faction)` as one integer, so the reader widens to four before
this design lands.

Two reward shapes exist.

- **Terminal.** The reward is zero on every decision until the game end. At
  the game end it is positive when the learner's faction won, negative when
  another faction won, and zero at the tick limit with no winner.
- **Shaped.** The reward on each decision is the change in a weighted sum of
  the standing values since the last decision, plus the terminal reward.

**The reward lives in the control plane.** It is a training choice and not a
rule of the world. Two researchers train two policies on one engine with two
rewards, and the engine does not know. A reward inside the engine would be a
value that enters the hash and changes every golden file when a researcher
changes their mind. A reward in Python changes nothing in the engine.

The reward reads aggregates only. It reads no unit and no tile, so it stays
inside the control plane rule.[^6]

**The default reward and its parameters are reference table rows.** A new
table holds them.[^10] The rows are the terminal win value, the terminal loss
value, the weight of each win path in the shaped sum, and the discount. Each
row is unset until a training run measures it, and each row holds its
derivation when a value is written. This document states no value.

## 4 Observation

One reader `observe(faction)` returns one flat integer array of fixed
length. The length is a function of the world parameters and never of the
population. The array holds six blocks, in this order.

| Block | Content | Length follows |
|---|---|---|
| Census | The subsystem census counts | The subsystem table |
| Standing | The running value of the faction on each win path | The path count |
| Relation | The relation entry of the faction toward each other faction | The faction ceiling |
| Board | The market rows of every faction | The faction ceiling times the board size |
| Weights | The faction's own weight vector | Four |
| Map | A window of level 1 cells around the faction's cities | The window size |

The map block holds five fields per cell: the share of tiles the faction
holds, the unit count, the store total, the upgrade level total, and the
wetness. Each is an exact integer total over the tiles of the cell, in the
shape the level 1 summary holds today.[^11] The wetness is the ground water
of the cell, which the weather field holds on the same lattice.[^12] The
window is a square of cells centred on the faction's seat and clipped to the
world. The window radius, in cells, is a row in the new reference table.[^10]

A city is a settlement site. The window follows the seat because the
controller plans around the seat and the seat is a row the engine already
holds.[^1] A later pass may widen the window to the bounding box of every
site the faction holds.

**Fog.** The product record says a faction sees only what its own units
observe.[^13] Nothing in the engine implements that record today. Every
reader returns the truth for every faction. `observe` therefore returns the
truth in this design. The map block reads a level 1 cell, and the product
record says a level 1 answer must not leak what its tiles would hide, so a
fog-honouring `observe` is one masking step on the map block when the
observation plane lands. That is a later record. This document does not
answer it, and it states that a policy trained on `observe` today trains
with full information.

**Type and bounds.** The array is a signed 64-bit integer array. Every field
is a whole number or a Q16.16 value as its raw integer.[^14] No field is a
floating point number. A field that is a total over a cell fits a 64-bit
accumulator by construction.[^15] The reader also exposes
`observation_bounds()`, which returns the lower and upper bound of each
position as two arrays of the same length. A learner scales the array in
Python.

**Cost shape.** The cost follows the cell count of the window, the faction
ceiling and the board size. No term follows the unit count or the tile
count. The reader starts no pass over the world. It reads the level 1 cells
the pyramid already rebuilt at the barrier.[^11]

## 5 Action

An action is one row of a bounded discrete table. The table is shared by
the controller and the learner. One row is a triple.

| Field | Content |
|---|---|
| Verb | One of a closed set of verb kinds |
| Target | One index into a bounded candidate list for that verb |
| Magnitude | One bucket from a small fixed set |

The candidate lists are bounded by world parameters and not by the
population.

| Verb | Candidate list | Bound |
|---|---|---|
| Gather | The resource kinds | The kind count |
| Build | The upgrade categories | The upgrade table row count |
| Campaign | Enemy cities in reach, then deposits in reach | Rows in the new reference table |
| Relation | The other factions | The faction ceiling |
| Advertise | The board rows | The board size |
| Send | Own cities | The site ceiling per faction |
| No-op | None | One |

The upgrade categories come from the upgrade table when that record lands.
The registry allocates the record number and the file does not exist
today.[^16] Until it lands the build list is the two kinds the enumeration
holds. The city lists and the deposit list follow the territory work in the
same way, and section 13 gives the order.

**The magnitude bucket** selects how much of the set a verb acts on, or how
large a step a relation move takes. The bucket edges are rows in the new
reference table.[^10]

**`act(faction, action_id)`** decodes the row and applies it through the same
verbs a Python caller and the controller use.[^17] It resolves the candidate
list at the tick it is called, so an index names a different city when the
list changes. The list order is fixed by a stable key, the tile index, so the
same world state gives the same list. A row the verb refuses is dropped, and
the refusal counts in the census, as a controller command is.[^18]

**The no-op action** applies nothing. A learner that cannot act on a decision
takes it. Without it the learner would be forced to act on every decision,
and a forced action is a bias in the table.

**`action_mask(faction)`** returns one byte per row, one when the row would
not be refused now and zero otherwise. It reads the same candidate lists and
refusal rules the verbs read. It duplicates no rule. A row that the mask
allows and the verb then refuses is a defect, and a test compares the two
over a seed set.

**Why the controller shares the table.** The controller's choice today is an
enumeration with three variants: gather a kind, build a kind, move a
relation.[^19] Each variant is one row of the table above. When the controller
emits a row and not a variant, every controller command is a labelled
example. The controller log already records each command with its faction,
kind, argument and sequence. A learner reads that log as an imitation
target: the state, the row the controller chose, and whether the verb took
it. Nothing is added to get that. A second action table for the learner
would be a second declaration of one fact, and nothing would fail when the
two disagreed.[^20]

The table is a constant in the core crate. The Python action count and the
mask length derive from it, in the way the panel names derive from the
panel deck. A check fails when a hand-written list disagrees.

## 6 Decision cadence

The learner acts every K ticks. K is a row in the new reference table.[^10]
Between two decisions the world steps K times. Two modes govern what happens
to the learner's faction between decisions.

- **Suppressed.** The flag is set for the whole episode. The controller never
  evaluates the faction. The last command stands until the learner changes
  it, because a gather order and a build order persist on the units.
- **Filled.** The flag is cleared after the learner acts and set again before
  the next decision. The controller plays the faction between decisions.

**This design recommends the suppressed mode.** Three reasons hold. The
credit for a result belongs to the learner alone, and a controller that acts
between decisions takes part of it. The filled mode toggles a hashed field
twice per decision, so the state hash of a training world differs from the
hash of the same world with no learner, and a replay must carry the toggles.
The suppressed mode is one flag write at reset. The filled mode is a
curriculum choice, and a curriculum is a training choice that belongs in
Python.

The filled mode stays available because the flag verb exists. No engine
work adds it.

## 7 The environment wrapper

One Python class holds N worlds. Its interface has the shape a common
training library uses, and it imports no training library.[^21] The methods
are:

- `reset(seeds)`: builds N worlds from N seeds, runs the seeding layer, sets
  the flag for the learner's faction in each, and returns N observations.
- `step(actions)`: applies one action per world through `act`, steps each
  world K ticks, and returns N observations, N rewards, N done flags and N
  info dictionaries.
- `observe()`, `reward()`, `done()`, `info()`: read the same values without
  stepping.

The done flag is true when the game end record is set, or when the tick
reaches the tick limit. The info dictionary holds the game end record, the
standing values and the census.

**Vectorisation.** The wrapper holds a list of worlds. It loops over that
list and over the factions. It loops over nothing else. That is the shape the
balance harness already has, and it stays inside the control plane rule
because the loop count follows the world count and never the
population.[^6] [^9] Each world steps with a thread count the wrapper is
given.

**Threads.** The step releases the interpreter lock, so the wrapper may step
N worlds from N Python threads.[^5] The observations are then gathered in
world index order. No result depends on which world finished first.[^22]

**The observation batch** is one two-dimensional integer array of shape N by
the observation length. The action batch is one array of N integers. Both
are fixed shape.

## 8 Self-play and opponents

**The controllers are the fixed opponents.** Every faction except the
learner's plays by the controller, biased by its seeded weight vector.[^1]
A training run needs no second policy to have an opponent.

**The weight vectors give opponent diversity for free.** Each faction draws
four weights from the seed, and the range is a balance value.[^23] A run over
many seeds meets many opponents: a faction that builds, a faction that
fights, a faction that trades. No archetype is coded, and no opponent pool is
maintained. A researcher who wants a fixed opponent sets the seed. A
researcher who wants a range sets many seeds.

**One learner per faction.** The wrapper accepts a list of learner factions.
Each has its own flag, its own observation and its own action. The wrapper
returns one observation and one reward per learner faction per world. Two
learners in one world is self-play. The wrapper does not choose the
algorithm.

**League play** is a later step. It needs a pool of saved policies and a
matching rule, and both live in Python. Nothing in the engine changes for it.

## 9 Throughput

A training run wants many short games, not one large one. The extents to use
are small. The seed set is large. Two axes trade against each other:
threads per world and worlds per process. A small world gains little from
more threads because each stage has few tiles to split. Many small worlds
gain from more processes.

**What to measure.** The cost of one step of a small world at one thread.
The cost of `observe` and `act` per call. The cost of the seeding layer per
reset. The number of decisions per second at N worlds and T threads per
world, for a grid of N and T. The resident memory of one small world.

**Where the figures go.** A new reference table holds them.[^10] Every figure
is derived until the target platform measures it, and the blocker that
governs cost figures stays open until it does.[^24] The table names the
machine, the commit and the command for each row, as the target platform
table does today.[^25] This document states no figure.

## 10 Determinism under training

**The learner is outside the hash.** The policy, its weights and its
sampling live in Python and change nothing in the engine. The world holds one
flag per faction and the commands the learner sent through the verbs. Two
worlds that received the same commands at the same ticks have the same hash,
whatever produced the commands.

**A replay is a seed and an action log.** The log holds one row per decision:
the tick, the faction and the action row. A replay builds the world from the
seed, sets the flag, and applies each logged action at its tick. It
reproduces the world exactly, at any thread count.[^4]

**The action log is a Pod event.** Each row is plain data with `repr(C)`,
declared padding and no `bool`, in the shape every event holds.[^26] The
engine writes the row when `act` applies, so the log is the engine's record
and not the wrapper's. A stored replay with its final hash is then a golden
test: build, replay, compare. That test can fail, because a change to any
verb the learner used moves the hash.[^27]

The controller log today holds the same fields for a controller command. When
the controller emits a row of the shared table, one event type holds both
logs, and one replay path reads both.

## 11 Out of scope

This design does not choose a learning algorithm, a network shape or a
training library. It does not choose the reward a researcher uses, only the
default and where its rows live. It does not implement fog. It does not add a
verb. It does not change the controller's choice rule, only the shape the
choice is written in.

## 12 The records this work needs

The scope rule gives a three-condition test. A decision needs a record when a
contributor could reasonably choose otherwise, when choosing otherwise costs
more than changing it later, and when the reasoning is not visible in the
code.[^28]

**One product record.** A learner plays one faction against the controllers.
It states the need of section 1 and no structure. It cites the record that
says a faction sees only what it observes, for the fog question, and the
record that says a game is balanced across seeds, for the runner shape.[^13]
[^29]

**One decision record.** The observation and the action of a faction are
bounded tables that the controller and a learner share. It passes the test. A
contributor could give the learner its own action set, or return the
observation as a dictionary, or let `act` reach a store directly. Each is
shorter. A separate action set breaks the imitation target and doubles a
declaration. A dictionary observation breaks the fixed shape a learner needs.
A direct write breaks the one rule set that the controller record
protects.[^17] None of those reasons is visible in a table constant.

The decision record cites the controller record for the shared verbs and the
flag, the control plane record for the crossing shape, the game end record
for the standing readers, the pyramid record for the map block, the relation
and board records for two observation blocks, the capability record for the
build list, and the determinism records for the log.[^1] [^6] [^8] [^11]
[^30] [^31] [^32] [^4]

**Why the reward needs no record.** A reward is a training choice. A
contributor may choose otherwise at no cost to the project, because the
reward lives in Python and enters no hash. The second condition fails. Its
default parameters are reference table rows, because they change when a run
measures them.[^33]

**Why the cadence needs no record.** K is a balance value. The mode is a flag
the wrapper sets. Both change cheaply. The second condition fails.

## 13 Sequencing

Three pieces of the game layer must land before the tables are written.

- **Territory.** The candidate lists name own cities, enemy cities in reach
  and deposits in reach. Each is a set of tiles a faction holds or can reach,
  and the territory work defines those sets. A list written before it would
  be rewritten when it lands.
- **The upgrade table.** The build list indexes the upgrade table. The table
  today is an enumeration of two variants. A build list over the enumeration
  would be rewritten when the table record lands.[^16]
- **Campaigns and standing.** The campaign verb is one row kind, and the
  campaign register does not exist. The standing reader returns one integer
  today and the design needs one per path. An action table without the
  campaign row and an observation without the standing block are both
  rewritten when those land.

Writing the tables before those three would mean writing them twice, once
against the enumerations that exist and once against the tables that
replace them. The passes are:

| Pass | Content |
|---|---|
| 1 | The product record and the decision record; the registry rows |
| 2 | The `standing` reader with one value per path; `observe` with every block except the map |
| 3 | The map block over the level 1 window; `observation_bounds` |
| 4 | The action table constant; the controller emits a row; `act`, `action_mask`, the no-op; the shared event |
| 5 | The Python wrapper; the replay test as a golden test |
| 6 | The candidate lists over territory, the upgrade table and campaigns, as each lands |
| 7 | The throughput measurement on the target platform; the reference table rows |

Passes 1 to 3 need nothing from the three prerequisites and may start now.
Pass 4 may start with the three rows the controller holds today and widen in
pass 6.

## 14 Open questions for the owner

1. **Does `observe` honour fog from the first pass, or later?** Recommendation:
   later. Nothing implements fog today, and a fog-honouring reader before the
   observation plane exists would be a capability nothing invokes.
2. **Which mode is the default cadence?** Recommendation: suppressed, for the
   reasons in section 6.
3. **Does the learner's faction get a seat?** The controller skips a faction
   with no seat, and the seeding layer founds one for every faction.
   Recommendation: yes, the learner starts from the same seeding as every
   other faction, so the world is fair across factions.
4. **Is the window centred on the seat or on the bounding box of every
   site?** Recommendation: the seat, in the first pass. The bounding box
   changes size as the faction grows, and a fixed observation length needs
   a fixed window.
5. **Does the controller's log become the shared event in pass 4, or stay
   separate?** Recommendation: one event. Two logs of one shape are the
   declaration defect.[^20]
6. **Does the wrapper live in the package or in a separate package?**
   Recommendation: in the package, beside the balance harness, because it
   shares the runner shape and has no dependency the package lacks.

## Corrections to the brief

The code disagreed with the brief at these points, and this document follows
the code.

1. `score(faction)` returns one integer. The brief names `standing(faction)`
   with one value per path. Section 3 names the widening as pass 2.
2. No `campaigns` reader exists. The stubs hold no campaign register.
   Section 13 lists it as a prerequisite.
3. No record numbered 0151, and no backlog item numbered 0484 or 0486, is in
   the tree. The brief names them as the upgrade table and the territory
   work. Section 13 names them by subject and cites the registry for the
   number.[^16]
4. The controller emits an enumeration with three variants today, not a table
   row. Section 5 makes the row the shared shape.

## References

[^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decisions D1, D4 and D7. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^2]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D6. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^3]: Design, the living world game layer. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^4]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^5]: ADR-0042, the interpreter is released for the whole step. `docs/adrs/draft/adr-0042-the-interpreter-is-released-for-the-whole-step.md`
[^6]: ADR-0040, Python is a control plane, not a data plane, decisions D1 and D2. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^7]: ADR-0043, a declared tier enforces the no-loop rule, and the API refuses the loop. `docs/adrs/draft/adr-0043-a-declared-tier-enforces-the-no-loop-rule.md`
[^8]: ADR-0148, a game end is recorded once and stops the controllers, decisions D1 and D2. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^9]: The balance harness. `python/cachette/balance/__init__.py`
[^10]: Reinforcement learning costs and parameters, to be created. `docs/reference/rl-costs.md`
[^11]: ADR-0022, level 0 is the only truth, and every level above it is derived. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^12]: ADR-0140, weather is a field over the level 1 cell lattice. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
[^13]: PRD-0001, a faction sees only what its own units observe. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
[^14]: ADR-0002, state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^15]: ADR-0023, an aggregate combines exactly, in any order, decision D3. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^16]: ADR Registry. `docs/adrs/REGISTRY.md`
[^17]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^18]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D3. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^19]: The controller, the choice enumeration. `crates/cachette-core/src/controller.rs`
[^20]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
[^21]: Gymnasium, the environment interface. https://gymnasium.farama.org/api/env/
[^22]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^23]: Balance register, the weight vector range. `docs/reference/balance.md`
[^24]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^25]: Target platform costs. `docs/reference/graviton-costs.md`
[^26]: ADR-0001, one binary gives one answer at any thread count, the event rule. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^27]: Testing Rules, section 1. `.agents/rules/testing.md`
[^28]: Decision Record Scope, section 1. `.agents/rules/adr-scope.md`
[^29]: PRD-0053, a game is balanced across seeds. `docs/product/accepted/prd-0053-a-game-is-balanced-across-seeds.md`
[^30]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold. `docs/adrs/draft/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
[^31]: ADR-0149, a faction's trade board is simulated state that any faction may read. `docs/adrs/draft/adr-0149-a-factions-trade-board-is-simulated-state-that-any-faction-may-read.md`
[^32]: ADR-0145, a unit type is a row of capability columns, and zero means cannot. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
[^33]: Decision Record Scope, section 4.1. `.agents/rules/adr-scope.md`
