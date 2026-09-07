# Report 31: The state of the learner surface

This report says what the engine gives a reinforcement learning run today. It
compares the code against three accepted decision records that specify the
surface, and against two design documents that describe it.

The report is an audit. A team lead surveyed the engine with a small number of
searches, stated ten conclusions, and asked for each one to be falsified. This
report answers each of the ten. It also answers three questions that the survey
did not cover.

**The code is the authority in this report. A document that says a thing is
built is not evidence that it is built.** Each verdict below names the file and
the line that proves it.

**Every count in this report measures one moment.** The moment is the state of
the `integration` branch on 6 September 2026. A count is correct here because a
research report is fixed to a moment. A count is wrong in a decision record,
because a record must stay true.[^1]

The audit read and searched the tree. It ran no build and no test.

---

## 1 The finding that changes the cost of the work

**Three accepted records specify the learner surface. No source file cites any
of them.**

- ADR-0154 states that the observation and the action of a faction are
  schema-declared bounded tables that the engine owns. Its six decisions cover
  the schema, the flat observation array, the fog rule, the action table, the
  legality answer, and one shared log.[^2]
- ADR-0155 states that a batch of worlds steps in one call, in index order.[^3]
- ADR-0156 states that the option weights of a faction are policy, set through
  one verb.[^4]

All three carry the status `Accepted` in the registry.[^5] This search over the
whole tree returns documents only:

```
grep -rn "adr-015[456]\|ADR-015[456]" --exclude-dir=target --exclude-dir=.git .
```

No file under `crates/` and no file under `python/` names any of the three.

**This changes what the work is.** A reader of the survey alone concludes that
the learner surface is undesigned. That conclusion is wrong. The surface is
designed, the design is binding, and none of it is built. The cost is
implementation, not design.

Two backlog items exist for two parts of it, and both sit in `proposed/`.[^6]
[^7] The other parts have no backlog row at all, so the priority index cannot
show them and no check reports them missing.[^8]

---

## 2 The ten claims

### Claim 1. Fog of war exists and is real

**CONFIRMED, with two corrections.**

| Item | Location |
|---|---|
| `pub struct Observation` | `crates/cachette-core/src/observation.rs:752` |
| `sees_now(faction, tile)` | `crates/cachette-core/src/observation.rs:812` |
| `has_seen(faction, tile)` | `crates/cachette-core/src/observation.rs:821` |
| `visible_layer(faction)` | `crates/cachette-core/src/observation.rs:848` |
| `remembered_layer(faction)` | `crates/cachette-core/src/observation.rs:856` |

**Correction one.** The two predicates sit on the observation type, not on the
world. The world wraps them under different names, and the wrappers take a hex
address rather than a tile index.

| Wrapper | Location |
|---|---|
| `faction_sees_now(faction, address)` | `crates/cachette-core/src/world.rs:6595` |
| `faction_has_seen(faction, address)` | `crates/cachette-core/src/world.rs:6611` |

**Correction two.** Neither layer is a tile bitmap. Each layer is an array over
the block lattice that the summary level aggregates over. Each block holds one
of four forms: no payload when the faction sees nothing in it, a sorted array
of offsets when it sees few, a bitmap when it sees many, and no payload again
when it sees every tile. The form type is `BlockForm` at
`crates/cachette-core/src/observation.rs:256`. This correction matters for
claim 5 and for section 3 of this report.

### Claim 2. The fog is live, not inert

**CONFIRMED. The line number in the survey is right.**

The step calls the rebuild at `crates/cachette-core/src/world.rs:5946`. The
enclosing function is `pub fn step`, which starts at
`crates/cachette-core/src/world.rs:5383`. The next item starts at
`crates/cachette-core/src/world.rs:5998`, so the call sits inside the step.

The call sits in a bare block with a stage span. It has no condition, no
feature gate and no flag. It runs on every tick. The stage is an ordinary
declared stage.

The test file drives the real caller. Every test in
`crates/cachette-core/tests/observation.rs` calls `world.step(1)`. The module
head of that file states the rule. This satisfies the project testing rule that
a test must start at the engine when the engine must invoke the mechanism.[^9]

**Apply the inert-code rule to the readers, and the answer changes.** This
search finds every caller of every fog reader:

```
grep -rn "sees_now\|has_seen\|visible_layer\|remembered_layer\|seen_now\|seen_ever\|block_seen" --include="*.rs" crates/
```

Outside the fog module itself, every caller is either a thin wrapper in
`crates/cachette-core/src/world.rs` or a line in
`crates/cachette-core/tests/observation.rs`.

**No production code reads the fog.** No pass filters on it. No view uses it.
No binding exposes it. The pass computes a correct answer that nothing
consumes. The project rule on inert code names this shape.[^10]

### Claim 3. The fog reaches the state hash

**PARTLY TRUE.**

The world hashes the observation at `crates/cachette-core/src/world.rs:4488`.
That call is real.

The call hashes half the fog. The hash function at
`crates/cachette-core/src/observation.rs:1090` writes the sight rules, then
walks the remembered layers only. **The visible layer never enters the hash.**

This is correct, not a defect. The visible layer is a pure function of the
units and the rules, so its inputs enter instead. A comment at
`crates/cachette-core/src/world.rs:4480` states the reasoning. ADR-0164
decisions D1 to D3 govern it.[^11]

The accurate statement is this. The remembered layer and the sight rules are
deterministic state. What a faction sees now is a derived projection.

### Claim 4. The fog is not exposed to Python at all

**CONFIRMED.**

This search covers the whole binding crate and the whole Python package:

```
grep -rn "observ\|fog\|sees_now\|has_seen\|visible" crates/cachette-py/src/ python/cachette/
```

Every hit is an unrelated word. Two examples are "invisible" at
`python/cachette/demo/app.py:87` and "observed share" at
`python/cachette/balance/__init__.py:15`.

The other two files of the binding crate hold nothing:
`crates/cachette-py/src/columns.rs` and `crates/cachette-py/src/logs.rs`. The
type stub `python/cachette/_core.pyi` holds no fog name in its 1303 lines.

Four files in the whole tree name the observation type. They are the fog
module, the world, the core crate root, and the fog test file.

### Claim 5. The exposed shape gives no bulk faction-scoped array

**PARTLY TRUE. This is the claim the survey got most wrong.**

The survey is right that the per-tile predicate is a per-tile predicate.

The survey is wrong that no bulk call exists. Four calls return a whole
faction-scoped answer in one crossing of no boundary.

| Bulk reader | Location | What it returns |
|---|---|---|
| `visible_layer(faction)` | `observation.rs:848` | the whole visible layer of one faction |
| `remembered_layer(faction)` | `observation.rs:856` | the whole remembered layer of one faction |
| `seen_now(faction)` / `seen_ever(faction)` | `observation.rs:830` / `839` | the two counts for one faction |
| `block_seen_now(block)` / `block_seen_ever(block)` | `observation.rs:871` / `880` | a faction mask for one summary cell |

A caller walks a layer with `populated_blocks()` at
`crates/cachette-core/src/observation.rs:354` and `block(u32)` at
`crates/cachette-core/src/observation.rs:348`. No tile predicate is needed.

The real defects in the shape are different from the one the survey named.

1. A layer is sparse and its length varies. ADR-0154 D2 requires one flat array
   of fixed length.[^2]
2. No schema declares the layout. ADR-0154 D1 forbids a caller that states an
   offset.[^2]
3. Nothing crosses to Python.

### Claim 6. There is no action schema and no legality mask

**CONFIRMED at the boundary. PARTLY TRUE inside the engine.**

No legality reader exists anywhere in the tree.

An enumerated action space does exist inside the engine. The type is
`pub enum Choice` at `crates/cachette-core/src/controller.rs:427`. It holds
eleven variants at this moment: `Gather`, `Build`, `Relation`, `Campaign`,
`Advertise`, `Trade`, `Carry`, `Project`, `Queue`, `Cross` and `Settle`.

The type is public. It carries no number, it is not plain data, and it does not
cross the boundary.

One related type is closer to the record. `ControllerCommand` at
`crates/cachette-core/src/controller.rs:405` is plain data with declared
padding. It holds a tick, a faction, a kind, an argument, a sequence and an
applied flag.

### Claim 7. There is no batch step

**CONFIRMED.**

The only step on the boundary is at `crates/cachette-py/src/lib.rs:554`. It
takes one world and one thread count.

This search returns three comments in the world about "the batch of a
structural change", which is an unrelated use of the word:

```
grep -rn "step_many\|batch\|WorldBatch\|step_batch" crates/ python/
```

ADR-0155 is accepted and specifies this call, including releasing the
interpreter for the whole batch and taking two separate parameters.[^3] Nothing
implements it.

### Claim 8. There is no reward signal defined anywhere

**CONFIRMED. The absence is a decided non-goal, not a gap.**

A case-insensitive search for "reward" over `crates/` and `python/` returns
nothing.

PRD-0056 states under what it does not do that it does not decide the reward,
because what a faction should be rewarded for is a rule of the downstream
game.[^12] One blocker holds the rules of that game.[^13]

The reinforcement learning design keeps the reward in the control plane for a
determinism reason.[^14] A reward inside the engine would enter the state hash,
so every golden file would change when a researcher changed their mind.

**Do not report this as missing work.** Report it as a decision, with an open
blocker.

### Claim 9. There is no configuration schema, and the setter count is past 37

**PARTLY TRUE. The number is wrong.**

No configuration schema exists. A caller configures a world through constructor
keyword arguments and through scattered setters. No call reports the set.

The count of setter definitions in the binding is exactly 37. Four of the 37
are camera setters, not world setters.

| Camera setter | Location |
|---|---|
| `set_tile_width` | `crates/cachette-py/src/lib.rs:7375` |
| `set_tile_height` | `crates/cachette-py/src/lib.rs:7390` |
| `set_origin_x` | `crates/cachette-py/src/lib.rs:7407` |
| `set_origin_y` | `crates/cachette-py/src/lib.rs:7423` |

All four sit inside the camera method block, which starts at
`crates/cachette-py/src/lib.rs:7314`. The world method block starts at
`crates/cachette-py/src/lib.rs:334`.

**The world setter count is 33 at this moment, not 37 and not more.**

One schema call exists. The event schema at `crates/cachette-py/src/lib.rs:7916`
walks the declared event layouts and returns column names with their element
types. It covers events only.

### Claim 10. The demonstration and the view read only what a player would see

**FALSE.**

The drawing pass takes no faction. The entry point is at
`crates/cachette-view/src/paint.rs:1973` and the paced form is at
`crates/cachette-view/src/paint.rs:2005`.

The overlay context holds the world, the address and the ground, and nothing
else. It is at `crates/cachette-view/src/overlay.rs:154`.

The registered overlay list at `crates/cachette-view/src/overlay.rs:662` holds
eleven layers at this moment: moisture, air, wind, temperature, food, wood,
stone, height, holder, upgrade and crowding. Each reads world truth. The holder
layer paints who holds every tile, to any watcher, at any time.

The view crate contains no fog code. It cannot filter what it does not read.

**A second leak, which the survey did not ask about.** The simulation itself
ignores the fog. The scoring function at `crates/cachette-core/src/choose.rs:632`
takes a cell summary, which is level 1 truth. The choice module names the
observation nowhere. A unit therefore scores a cell that its faction has never
observed. PRD-0009 is accepted, and the choice pass does not honour it.[^15]

---

## 3 The reader split

This section is the scope of the observation work. It lists every reader that
hands out world truth, and every reader that answers for one faction.

### 3.1 Readers that hand out whole-world truth

Each reader below takes no faction argument, or takes one and still answers
from the truth of the world.

| Reader | Location | What it gives away |
|---|---|---|
| `tile_values()` | `crates/cachette-py/src/lib.rs:606` | every tile value in the world, as one array |
| `presence_masks()` | `crates/cachette-py/src/lib.rs:919` | which factions stand in every cell |
| `tile_holders()` | `crates/cachette-py/src/lib.rs:1051` | who holds every tile, as one array |
| `founding_survey(...)` | `crates/cachette-py/src/lib.rs:2922` | candidate settlement sites anywhere |
| `region_summary(q, r)` | `crates/cachette-py/src/lib.rs:3101` | any level 1 cell |
| `explain_choice(unit)` | `crates/cachette-py/src/lib.rs:3510` | the scoring of any unit, by identity |
| `tile_report(q, r)` | `crates/cachette-py/src/lib.rs:3618` | any tile |
| `window_census(q, r, radius)` | `crates/cachette-py/src/lib.rs:3713` | unit counts and crowding in any window |
| `subsystem_census()` | `crates/cachette-py/src/lib.rs:4474` | whole-world counters, no faction column |
| `draw(camera, ...)` | `crates/cachette-py/src/lib.rs:4601` | the whole picture |
| `weather_ground()` | `crates/cachette-py/src/lib.rs:5202` | the whole weather lattice |
| `weather_air()` | `crates/cachette-py/src/lib.rs:5217` | the whole weather lattice |

The view crate adds three more, and none takes a faction.

| Reader | Location | What it gives away |
|---|---|---|
| `paint::draw(world, camera, canvas)` | `crates/cachette-view/src/paint.rs:1973` | the whole picture |
| `paint::draw_paced(...)` | `crates/cachette-view/src/paint.rs:2005` | the whole picture |
| `overlay::value_of(...)` with context `At` | `crates/cachette-view/src/overlay.rs:726`, `154` | any overlay value at any address |

The demonstration reads the subsystem census in two places, at
`python/cachette/demo/app.py:641` and `python/cachette/demo/toasts.py:536`. It
draws through the paced drawing pass. It reads no per-faction view.

One further omniscient surface exists, and it is a contributor tool rather than
a harness. The protocol server at `python/cachette/agent/server.py` exposes the
tile report, the window census, the region summary, the founding survey, the
site economy and the unit choice, all without faction scoping. Name it so that
nobody mistakes it for a learner seat.

### 3.2 Readers that answer for one faction, and answer correctly

Each reader below takes a faction and reports that faction's own state.

| Reader | Location | What it reports |
|---|---|---|
| `plan(faction)` | `crates/cachette-py/src/lib.rs:2555` | the faction's own plan |
| `faction_weights(faction)` | `crates/cachette-py/src/lib.rs:3863` | the faction's own weights, read only |
| `is_externally_controlled(faction)` | `crates/cachette-py/src/lib.rs:3910` | the faction's own flag |
| `score(faction)` | `crates/cachette-py/src/lib.rs:4001` | the faction's own score |
| `standing(faction)` | `crates/cachette-py/src/lib.rs:4031` | the faction's own five win-path values |
| `faction_units(faction)` | `crates/cachette-py/src/lib.rs:5305` | the faction's own units |
| `trade_book(faction)` | `crates/cachette-py/src/lib.rs:5735` | the faction's own book |

The standing reader was checked closely, because the survey singled it out. It
returns five keys: the tiles the faction holds, the seats it holds, the sum of
its stores, the highest renown of its characters, and the best wonder progress
on ground it holds. Every value belongs to the named faction. **It is clean.**

One reader crosses factions on purpose. The market reader at
`crates/cachette-py/src/lib.rs:5871` reads the boards of rivals. ADR-0149 is
accepted and states that a trade board is simulated state that any faction may
read.[^16]

### 3.3 The rule that decides what to do with the split

**An omniscient reader is not automatically a defect. Removing the twelve
readers above is not the work.**

ADR-0059 D6 already rules on this. It states that a world-wide reader is not a
per-faction reader, and that nobody may read it as one. It names the subsystem
census, and it says that the census serves a developer who watches the engine
and stays that reader.[^17]

The interfaces design agrees. It states that the full-information readers stay
for the modeller and for the reproducer, and that the environment core calls
none of them.[^18]

So the binding rules are these three.

1. A reader that names a faction must offer no argument that widens its answer.
2. A per-faction count must never include a subject the faction has not
   observed.
3. The environment core must call no full-information reader, and a check must
   catch an adapter that does.

**The work is therefore to add faction-scoped readers beside the world-wide
ones.** It is not to delete the world-wide ones. This is a smaller job than the
reader table suggests, and it is the version to plan against.

---

## 4 The controller seat

The engine holds one hook for an external player.

| Item | Location |
|---|---|
| `Controller::set_externally_controlled` | `crates/cachette-core/src/controller.rs:1065` |
| The binding setter | `crates/cachette-py/src/lib.rs:3895` |
| The binding reader | `crates/cachette-py/src/lib.rs:3910` |
| `run_controller` | `crates/cachette-core/src/world.rs:14521` |

A faction whose flag is set receives no evaluation from the built-in
controller. The controller runs as the last stage of the step, after every
derived structure describes the frame.

The doc comment on the binding setter is honest about the state of it. It says
that the flag is off for every faction of a new world, that nothing in the
engine sets it, and that it exists so that a later player hook has a place to
stand.

**A learner could drive a loop today.** The loop is this. Set the flag for one
faction. Call the step. Call ordinary verbs from Python between steps. The
verbs that act are the gather order, the build order, the queue verb, the
project zoning verb, the send verb, the campaign verb, the relation move, the
advertise verb, the four trade verbs, the group founding verb and the settle
order.

**Three things the loop lacks.** There is no single act verb that takes an
action integer. There is no legality answer. There is no observation. The
weights reader is read only, and no setter exists beside it.

---

## 5 What the design documents say, against what the code does

Three documents describe this surface. Research report 22 surveys the
approaches.[^19] One design document states the design.[^14] One interfaces
document states the contracts.[^18]

Section 2 of the design document lists what the engine already gives a learner.
Each item was checked against the code.

| Claim in the document | Verdict | Evidence |
|---|---|---|
| One binary gives one answer at any thread count | **True** | two determinism tests hold it |
| The interpreter is released for the whole step | **True** | `crates/cachette-py/src/lib.rs:554` uses `python.detach` |
| Set-valued verbs exist and act once | **True** | the verb list in section 4 above |
| The externally-controlled flag exists | **True** | `crates/cachette-py/src/lib.rs:3895` |
| Win paths and a game end record exist | **True** | `game_end` and `standing` on the boundary |
| A multi-seed runner exists | **True** | `python/cachette/balance/` |
| "Aggregate readers are an observation space. **None returns the world**" | **FALSE** | `lib.rs:606`, `919`, `1051`, `5202`, `5217` each return the world as one array |
| "Today the boundary exposes `score(faction)` as one integer, so the reader widens to four before this design lands" | **Stale** | `standing(faction)` exists at `lib.rs:4031` and returns five values |

Everything in the observation contract, the action contract, the mask, the
batch verb and the shared log is unbuilt.

Section 2.3 of the interfaces document is titled "Fog is honoured by the
engine". The engine does not honour it. Section 2 of this report gives the
evidence.

---

## 6 The observation array, and what the block form implies

The team lead asked whether a flat observation array needs a tile-resolution
expansion of the fog, and whether that expansion is the expensive part.

**No expansion is needed. This is the good news in the audit.**

The world builds the fog and the summary level from one lattice value. It
constructs the observation at `crates/cachette-core/src/world.rs:1636` and the
pyramid at `crates/cachette-core/src/world.rs:1643`, and both take the same
layout. The lattice type is at `crates/cachette-core/src/bridge.rs:195`, and it
gives the block count and the two block extents. ADR-0022 D2 is the reason the
two share one lattice.[^20]

ADR-0154 D2 says that the reader reads the aggregates the engine already keeps
and the summary the pyramid already rebuilt. It says a reviewer finds a
violation when the reader walks the world.[^2]

The interfaces document fixes the resolution. It says the map block holds the
cell count across, the cell count down, and six fields for each cell. The sixth
field is the visibility flag that fog adds.[^18]

**So the observation sits at cell resolution.** The visibility flag already
exists, as a faction mask for each cell, at
`crates/cachette-core/src/observation.rs:871` and `880`. Reading it costs one
array index and one bit test. The map block is the two block extents times six
signed integers, and every input exists today.

The four block forms are an internal detail of the tile-resolution layer.
Nothing in the observation contract requires expanding them.

**The expensive part is a different rule.** ADR-0059 D4 binds every reader, not
the fog module alone. It says a reader answers the present value for a place
the faction sees now, the remembered ground for a place it saw once, and
nothing for a place it never saw. It adds that a summary reader combines only
the values that the same rule admits, so that a cell cannot state what its
tiles hide.[^21]

That last sentence carries the cost. A partly seen cell cannot report the
summary total, because the total counts tiles the faction has not seen.
Producing a masked total means combining beneath the fog mask at tile
resolution, and reporting at cell resolution. **The four block forms bite
there, and nowhere else.**

### One question the records leave open

ADR-0059 D4 says a remembered place answers with the ground of that place as
the faction last saw it.[^21] The remembered layer stores membership only. The
block form holds an empty marker, an offset array, a bitmap or a full marker.
It stores no value.

Generated ground is a function of the seed and the address, so most of the
ground is recoverable without a stored snapshot. An upgrade is not. ADR-0090
makes an upgrade the difference between the generated world and the built
world.[^22] A faction that saw a plain, marched away, and whose rival then
built there, has no stored answer for what it last saw.

Either the record means generated ground alone, or a snapshot store is missing.
The audit found no document that settles it. **Ask the project owner before
anyone builds the remembered reader.**

---

## 7 The three accepted records, judged

The three records were accepted before the code that they govern existed. Each
was read against the engine as it now stands.

### ADR-0155, a batch of worlds. Sound. Build it as written

The record holds four decisions.[^3] Nothing in the engine contradicts any of
them. The binding already takes a lock for each call, and the single step
already releases the interpreter for its whole run, which is the mechanism the
second decision needs.

One status risk exists and it is not a design risk. The record depends on
ADR-0047, and that record is still a draft.[^5]

**Verdict: unbuilt, buildable, low risk.**

### ADR-0156, option weights. Sound, and the smallest item

The record holds seven decisions.[^4] The weight type already exists at
`crates/cachette-core/src/controller.rs:137` with four weights. The reader
already exists at `crates/cachette-py/src/lib.rs:3863`. Only the setter is
missing.

The three ground weights that the second decision adds already have reserved
rows in the balance register. Each row is marked unset, each names one blocker,
and each names this record as its writer.[^23] The record and the register
agree.

One clause needs re-reading before anyone writes code. Decision D6 speaks of
work that the plan of a faction zones. ADR-0159 changed one decision of
ADR-0152, the per-unit project assignment, and the registry row of ADR-0156
depends on ADR-0152.[^24] Zoning survived that change and only per-unit routing
moved, so D6 most likely still holds. Check it against the project reader
before starting.

**Verdict: unbuilt, buildable, one clause to re-read.**

### ADR-0154, observation and action. Four decisions hold. One is overtaken. One hides its cost

The record holds six decisions.[^2] They do not all stand equally.

**D1, the schema. Holds.** The pattern is proved in this codebase. The column
builder at `crates/cachette-py/src/columns.rs` derives one array for each
declared field of an event, and it names no field. A field added to an event
reaches Python with no edit to the binding. Build the observation schema the
same way.

**D2, the flat fixed array. Holds.** Section 6 above shows that the pyramid and
the shared lattice make it cheap.

**D3, fog inside the reader. Holds, and its forward reference has resolved.**
D3 says that the storage claim is separate and that the registry reserves a
number for it. That number became ADR-0059, which is now accepted and whose
storage is built.[^17] D3 is satisfied by its dependency. It is not stale.

**D4, the action factorisation. Overtaken by the code.** D4 says the table
factorises into a verb, a target and a magnitude. It also says the verb set is
the set that the choice enumeration of the built-in controller holds, plus one
no-op row.

That enumeration has grown. Three of its eleven variants break the
factorisation.

- The campaign variant carries two targets: an objective kind and a tile.
- The advertise, carry and project variants carry no target and no magnitude.
  Each is a whole-faction command whose content is resolved when it applies.

A clean mixed radix over one verb, one target and one magnitude does not cover
them. The defect is repairable. A verb with no target takes a candidate bound
of one, and the campaign variant takes either a second radix position or a
flattened candidate list. **But the record as written no longer describes the
enumeration that it points at.** Somebody who builds from D4 today finds that
out in the middle of the work.

**Repair D4 before anyone starts. It is the decision people would build first.**

**D5, the legality answer. Sound, but its cost is hidden.** D5 forbids
restating a refusal rule. It says the answer reads the same candidate lists and
the same refusal rules the verbs read, and that it duplicates no rule.

Those refusal rules live inside the verbs, spread through a world module of
15504 lines at this moment. Honouring D5 means giving each verb a path that
reports a refusal without acting. The record does not say that. Anybody who
sizes the work from the record alone will underestimate it.

**D6, one shared log. Needs a layout change.** The controller command type at
`crates/cachette-core/src/controller.rs:405` holds a kind and an argument, each
one byte. D6 requires the row to carry the whole action integer, so that no
field of an action lives anywhere else. Widening a plain data type that already
enters the state path means a declared padding change and a golden file update.
It is feasible. It is not a free addition.

**Verdict: repair D4, size D5 honestly, and plan the D6 layout change. The
record is four-sixths accurate. The stale sixth is the one people reach for
first.**

---

## 8 The work, in order

Each item below names the file it would live in. The order is a dependency
order, not a value order.

**1. Fog-scope the readers.** Backlog item 0495 holds this work.[^6] It already
sits first in the backlog priority index, which calls it the last gate before a
learner can be trained.[^8] It is in `proposed/`, so refining it is part of the
work. Files: `crates/cachette-core/src/world.rs` for the readers, and
`crates/cachette-core/src/observation.rs` for the per-unit mask. It touches the
step, so one worker holds it at a time. The masked summary rule of ADR-0059 D4
is the expensive clause inside it.[^21]

**2. The flat observation and its schema.** ADR-0154 D1 and D2.[^2] Files: a
new module such as `crates/cachette-core/src/learn.rs`, and
`crates/cachette-py/src/lib.rs` for the observation reader and the schema call.
Model both on the column builder in `crates/cachette-py/src/columns.rs`.
Depends on item 1.

**3. The action table and the legality mask.** ADR-0154 D4 and D5.[^2] **Repair
D4 first.** Files: a new module such as `crates/cachette-core/src/action.rs`,
`crates/cachette-core/src/controller.rs` to share the encoding, and
`crates/cachette-py/src/lib.rs` for the act verb and the legality reader.
Depends on item 1, because a mask must not name a target the faction cannot
see. This is the largest item.

**4. The option weight setter.** Backlog item 0496 holds it, and it implements
ADR-0156.[^7] [^4] It is independent of items 2 and 3, and it is the cheapest
real progress. File: `crates/cachette-py/src/lib.rs`, beside the existing
weights reader. One blocker governs its three new weights.[^13]

**5. The reward, in Python.** It reads the standing reader and the game end
record, and both exist today. File: a new module under `python/cachette/`. No
Rust change. Add a reference table row set, each row unset, under the cost
blocker.[^25]

**6. The batch step.** ADR-0155, as written.[^3] File:
`crates/cachette-py/src/lib.rs`, as a new batch type. This is throughput only.
Build it after a single-world loop runs.

**7. The environment wrapper.** Files: a new package under `python/cachette/`,
shaped on the existing balance runner at `python/cachette/balance/`.

### A gap in the queue

Only items 1 and 4 have a backlog file. Items 2, 3, 5, 6 and 7 have accepted
records and no backlog row, so the priority index cannot show them and no check
reports them absent.[^8] The plan lives in the accepted records, and the queue
does not know about it.

### A cheaper first loop, and why it is not the harness

Items 4 and 5 with a crude wrapper close a learning loop on a small world
today, with no fog and no schema. The loop is this. Set the external flag, set
the weights, step, read the standing, compute a reward in Python.

That loop violates the first checkable statement of PRD-0056, which asks that
the harness never show a learner anything a player of that faction could not
see.[^12] **It is a throwaway spike, and it is not the harness.** Anybody who
offers it must say so.

---

## 9 What this audit did not do

The audit changed no file. It ran no build, no test and no gate. Every
statement above comes from reading and searching the tree on the `integration`
branch. **No claim in this report is a measurement of running code.**

The audit did not check that any proposed item compiles. It did not measure any
cost. One blocker says which cost figures in this project are measured and
which are derived, and this report states no cost figure.[^25]

---

## References

[^1]: Decision Record Scope, section 4.3. `.agents/rules/adr-scope.md`
[^2]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decisions D1 to D6. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^3]: ADR-0155, a batch of worlds steps in one call, in index order, decisions D1 to D4. `docs/adrs/accepted/adr-0155-a-batch-of-worlds-steps-in-one-call-in-index-order.md`
[^4]: ADR-0156, a faction's option weights are policy, set through one verb, decisions D1 to D7. `docs/adrs/accepted/adr-0156-a-factions-option-weights-are-policy-set-through-one-verb.md`
[^5]: ADR Registry. `docs/adrs/REGISTRY.md`
[^6]: Backlog item 0495, build the observation plane and let every reader answer for one faction. `docs/backlog/proposed/0495-build-the-observation-plane-and-let-every-reader-answer-for-one-faction.md`
[^7]: Backlog item 0496, let a faction set the weight it gives each option. `docs/backlog/proposed/0496-let-a-faction-set-the-weight-it-gives-each-option.md`
[^8]: Backlog priority index. `docs/backlog/PRIORITY.md`
[^9]: Testing Rules, section 5. `.agents/rules/testing.md`
[^10]: Recurring Defect Shapes, shape 3. `.agents/rules/recurring-defects.md`
[^11]: ADR-0164, every stored value the step reads enters the state hash, decisions D1 to D3. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
[^12]: PRD-0056, a learner plays one faction against the controllers. `docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md`
[^13]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^14]: Reinforcement learning design, sections 2 and 3. `docs/superpowers/specs/2026-09-05-reinforcement-learning-design.md`
[^15]: PRD-0009, a unit acts on the world it can see. `docs/product/accepted/prd-0009-a-unit-acts-on-the-world-it-can-see.md`
[^16]: ADR-0149, a faction's trade board is simulated state that any faction may read. `docs/adrs/accepted/adr-0149-a-factions-trade-board-is-simulated-state-that-any-faction-may-read.md`
[^17]: ADR-0059, fog storage grows with observed area, not with world area, decision D6. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
[^18]: Reinforcement learning interfaces design, sections 2.2 and 2.3. `docs/superpowers/specs/2026-09-05-reinforcement-learning-interfaces-design.md`
[^19]: Research report 22, reinforcement learning approaches. `docs/research/reports/22-reinforcement-learning-approaches.md`
[^20]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^21]: ADR-0059, fog storage grows with observed area, not with world area, decision D4. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
[^22]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D2. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
[^23]: Balance register, the three ground weight rows. `docs/reference/balance.md`
[^24]: Findings register, the supersession of ADR-0152 D5 by ADR-0159. `docs/FINDINGS.md`
[^25]: Blockers register, BLK-007. `docs/BLOCKERS.md`
