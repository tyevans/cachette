# Open Decisions (Register)

This document is a **register**. It lists choices that are open, the options,
and a recommendation.

A decision needs **judgement**. The options are known and work can continue
under a stated assumption. Compare `BLOCKERS.md`, which lists work that is
stopped for want of information.

Numbers are permanent. Never reuse one. A closed decision keeps its row, with
the outcome recorded.

When a decision closes, and it corrected something the project believed,
record the correction in `FINDINGS.md` as well.


## Allocating a number

**Claim the next number below before you write the row.** Increment it in the
same change that adds the row.

A writer that numbers a row by reading the last row collides with any other
writer working at the same time. That happened, and it is recorded as
precedent.[^ALLOC]

**Next number: DEC-038**

[^ALLOC]: Findings register, FND-038. `docs/FINDINGS.md`

## Open

### DEC-001 — The commodity split

Two reports set different ceilings, and they bound different things.

| Report | Ceiling | Reason |
|---|---|---|
| Entity economy | 64 | A presence mask is one `u64`. 64 `i64` values fill exactly 8 cache lines. |
| Trade and flow | 16, hard limit 32 | Cache residency during the flow solve. |
| Individual agency | 4 to 8 | What one individual can carry. |

**Recommendation:** 64 may exist, 16 take part in the transport solve, the
remainder stay local to a settlement, and an individual carries 8. The three
limits are compatible because they bound existence, participation and carriage
separately.

**Assumption in the meantime:** the recommendation above.

### DEC-002 — Do units make individual decisions?

The needs report concluded that units do not decide, because a decision cost
400 nanoseconds and one million decisions would take four times the tick
budget.

The agency report measured 4.1 nanoseconds. The gathers are sequential, not
random, because units are sorted by tile index and the fields are level-1
planes that stay in cache.

**This is now a design choice, not a budget one.**

**Recommendation:** both tiers. Individuals choose where to go; cohorts choose
what to buy. Cost is 0.18 core-ms. The project owner has asked for individual
experiences, and this delivers them.

### DEC-003 — Do dead characters keep relation edges?

Retaining them costs 531 MB at 100,000 living characters and 1.39 GB at the
ceiling. Dropping them loses the ability to reason about a dead person's
former ties.

The target is now 50,000 living characters, so retention costs roughly half
the first figure at the target.[^TARGET] That scaling is derived, not
measured. The recommendation does not change, because the cost still exceeds
the whole living character layer.

**Recommendation:** drop them. The character report notes this is how
expensive the question is to answer wrongly.

### DEC-004 — One fog layer or two

The fog report specifies explored and visible as separate layers, and asks
whether both are needed.

**Recommendation:** unresolved. It depends on whether the game shows explored
terrain differently from currently visible terrain.

### DEC-005 — Does the military influence plane need terrain conductance?

With conductance the solve costs 150 microseconds. Without it, 12
microseconds. The difference is whether influence flows around mountains or
through them.

**Recommendation:** include it. Twelve times a small number is still a small
number, and influence that ignores terrain will look wrong.

### DEC-006 — Simulated or procedural weather

Procedural weather is a deterministic function of position, tick and seed:
zero storage, no update cost, perfectly reproducible, but no feedback.
Simulated weather supports orographic rain shadow and fire-driven weather at
real cost.

**Recommendation:** procedural base with simulated perturbation, if weather is
built at all. It is not yet in scope.

### DEC-007 — Retained or transient event log

The log is currently transient. Retention costs 3.2 MB per frame, which is
11.5 GB per minute. Retention would buy rollback, time travel and audit.

**Recommendation:** stay transient. Events are already serialisable and the
apply step is pure, so retention remains additive.

### DEC-008 — Is a 50-second mountain crossing acceptable?

The approved calibration puts an ordinary crossing at 12.5 seconds and a
mountain crossing at 50 seconds. The project owner rejected 50 seconds as the
ordinary case. The recalibration relocates it to mountains.

**Recommendation:** accept. A mountain pass should be a serious obstacle.

### DEC-012 — Does a product record cite a decision record?

**Decided: no.** Recorded here because the reasoning is easy to lose.

A product record states a need. A decision record answers to a constraint. A
product direction changes more often than a constraint does, so a citation
from a decision record to a product record would place changing material
inside a historical document, which the scope rule forbids.

The join runs the other way and through one place only: a refined backlog
item names both the record that governs it and the product record it serves.
A check enforces that a product record contains no decision record citation.

**Revisit if.** The backlog stops being the only route from a need to the
work, or a reader cannot answer "which need does this record serve" and needs
to.

### DEC-017 — Is a tile crossing time content-configurable, or fixed by the engine?

A crossing time depends on the terrain multiplier that scales the step cost of
a tile. No record states where that multiplier lives.

**Option A. Content-configurable for each terrain type.** The multiplier sits
in the terrain table beside the terrain capacity. A content author tunes a
crossing without an engine change.

**Option B. Fixed by the engine.** The multiplier sits in engine code. The
engine can then bound the dwell range at compile time.

**Recommendation:** content-configurable. The terrain capacity table is
already content, and the capacity and the multiplier describe the same tile.
Splitting them across content and code would put one crossing's two levers in
two places. Option B buys a compile-time bound that a validated range in
content also buys.

**Assumption in the meantime:** content-configurable.

**Related.** The mountain multiplier has no recorded value. The accepted
50-second mountain crossing implies a multiplier of 2 against ordinary
ground.[^DEC1] Whichever option wins, that value needs recording.

[^DEC1]: See DEC-008 in this document, and the movement timing note, `docs/research/movement-timing.md`.

[^DEC2]: ADR-0005, a solver runs a fixed iteration count, decision D1. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`

[^DEC3]: Blockers register, BLK-007. `docs/BLOCKERS.md`

[^DEC4]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`

[^DEC5]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`

[^DEC6]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`

[^DEC7]: Backlog item 0030. `docs/backlog/complete/0030-enforce-the-barrier-ordering.md`

[^DEC8]: ADR-0067, the viewer reads the world and never writes to it, decision D4 and its consequences. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`

### DEC-018 — Where does movement sit in the frame schedule?

The frame schedule is static and known before the frame runs. The order of
the systems inside it is not recorded anywhere.

The movement design session proposed one order: movement runs after the needs
system and before the combat system.

**Option A. Movement after needs, before combat.** Movement reads what the
needs system produces, so a unit acts in the same frame on the need that
raised it. Combat then sees the positions of this frame.

**Option B. Some other order.** No one has argued for one.

**Recommendation:** option A. The read-after-write dependency between needs
and movement is real, and combat wants the current positions.

**Assumption in the meantime:** option A.

**Why this is a register row and not a record.** Neither the needs system nor
the combat system exists. An order between systems that nobody has written is
an intent, and a record must not state an intent as a fact. Write the order
into the schedule when the schedule exists. Promote it to a record only if a
contributor could reasonably choose otherwise and the reasoning does not show
in the schedule itself.


### DEC-019 — How many admission passes does one frame run?

The admission step runs a fixed number of passes. Each pass admits what it
can against the room the previous pass confirmed. The engine never runs to a
fixpoint, because a fixpoint needs a convergence test and a solver in this
project runs a fixed count.[^DEC2]

The record states that the count is content and declared before the frame
runs. It states no value, and no value follows from the tile scale.

One pass admits no chain. A unit cannot follow another out of a full tile in
the same frame, so a column of units on a road advances one unit for each
frame however long the column is.

Each further pass admits a chain one unit longer, and costs one more scan of
the intents. A chain longer than the pass count waits for the next frame,
which is a delay and never a wrong answer.

**Option A. Two passes.** A unit may follow one departure. This admits the
common case, a unit stepping into the tile a neighbour just left, and costs
one extra scan.

**Option B. One pass.** The cheapest, and it admits no chain at all.

**Option C. More than two.** Each pass buys a longer chain. Nobody has
measured what a chain costs or how long a real column is.

**Recommendation:** option A. It admits the case that a person watching the
world would call obviously correct, and it is the smallest count that admits
any chain. The value is content, so raising it later costs nothing.

**Assumption in the meantime:** two passes.

**What would settle it.** A measurement of how often a chain longer than two
appears in a run, on the target platform. That measurement waits on the
benchmark harness.[^DEC3]

### DEC-020 — Must a spawn respect the tile capacity?

Admission never raises a tile above the capacity of its ground.[^DEC4] A
spawn does not read the capacity at all, so a caller may place a hundred
units on one tile and the engine accepts it.

The record binds admission and says nothing about a spawn. The question is
whether the capacity is a property of the world at rest or a rule that
movement obeys.

**Option A. A spawn refuses a tile at capacity.** The capacity is then a
world invariant, and one check can state it. The spawn must read the
occupancy of the tile, and the record leaves the storage of an occupancy
count undecided, so this option forces that decision.[^DEC5] Counting by a
scan of the arena costs the population for each spawn, which the target scale
does not permit.

**Option B. A spawn may over-fill, and movement never does.** The invariant
is then the one admission gives: no tile gains a unit beyond its capacity. A
tile that starts over capacity can only lose units. No new storage is needed
and the record needs no amendment.

**Option C. A spawn refuses, and the engine holds a dense occupancy array.**
One byte for each tile at the target scale. This is the storage decision the
record defers, and it buys a constant-time spawn as well as a constant-time
admission.

**Recommendation:** option B until something needs otherwise. It is what the
record already states, it needs no storage decision, and the scenario it
allows is a caller mistake rather than a state the simulation reaches.

**Assumption in the meantime:** option B. The movement suite asserts that no
tile gains a unit beyond its capacity, and it does not assert that no tile is
ever above it.

**What would settle it.** A caller that must trust the capacity of a tile
without having placed the units itself. The Python control plane is the
likely one, and it does not spawn units yet.

### DEC-021 — Where does a structural change made outside a frame get its barrier?

The bridge rebuilds at the barrier, after the structural apply.[^DEC6]
Admission reads the occupancy of a target from the bridge, so the bridge must
describe the arena before the intents are admitted.

A spawn or a despawn made between two frames is a structural change that has
passed no barrier. It leaves the bridge stale, and the first step after it
then has nothing to read.

**Option A. The step opens by rebuilding a stale bridge.** The step gives the
caller's changes the barrier they never had. It costs a revision comparison
when nothing changed. The rebuild at the end of the step stays last and stays
the barrier of that frame, so there are two call sites for one operation.

**Option B. The caller rebuilds before it steps.** One call site. Every
caller that spawns must remember, and a caller that forgets gets an error
from the first step rather than from the spawn.

**Option C. A spawn maintains the bridge itself.** The bridge stops being
derived at the barrier, which the record forbids.

**Recommendation:** option A. Option B makes a correct program depend on a
convention that nothing enforces, and the error it raises names the bridge
rather than the spawn that caused it. Option C contradicts the record.

**Assumption in the meantime:** option A.

**Why this is a register row and not a record.** The two call sites are the
part a reviewer might object to, and the objection is about one function
rather than about a constraint on the project. Promote it to a record if a
second structural apply lands inside the frame, because the ordering between
the two is then a real decision.

The ordering of the barrier itself is settled and enforced. A test reads it
from outside: a rebuild that ran before the structural apply leaves the
derived structure stale when the step ends, and four tests fail on that.[^DEC7]
The open part here is the refresh at the top of the step, which serves a
change the caller made outside any frame.

### DEC-022 — May the viewer make the engine wait?

The product record for the first renderable example states that the window
keeps up with the engine, or drops what it cannot draw and reports the drop,
and that it never makes the engine wait. It also states that the engine costs
the same when a viewer is attached.

The viewer record decides the opposite for now. One loop steps and then
draws, so the drawing rate and the tick rate are one number. Its consequences
section says plainly that a slow drawing slows the simulation in the
demonstration binary, and that this is acceptable for a demonstration.[^DEC8]
The binary also caps its own frame rate, so the engine waits on every frame
that finishes early.

Nothing drops a frame and nothing reports a drop. The two block counts the
panel shows count empty spatial blocks, not dropped frames.

**This is a real contradiction, not a defect in either document.** The viewer
record knew it was choosing against the product record, and it named what
would supersede the choice.

**Option A. Amend the product record.** The statement about waiting becomes a
statement about the engine when a viewer is attached through a snapshot, and
the demonstration is excluded by name. The product record then describes what
the project built.

**Option B. Separate the two rates.** The engine runs on its own thread and
publishes a frame the viewer reads. This is what the viewer record names as
its own successor. It needs the snapshot record, which does not exist.

**Option C. Leave both as they are.** The product record then states
something the code does not do, and it cannot reach `Shipped`.

**Recommendation:** option A now, option B when a caller needs the two rates
apart. The product record asked for a property of the engine, and it stated
it over the demonstration. Writing the snapshot record to serve a
demonstration is the wrong order, which the viewer record already argues.

**Assumption in the meantime:** option C. The product record stays in
`shaped/` and this row holds the reason.

**What would settle it.** A person who must watch a world that steps faster
than a screen refreshes.

### DEC-033 — Does the project keep a performance path for the development machine?

**The question.** Every cost figure in this project is derived and belongs to
the target, and one open blocker states that no measurement exists there.[^DEC18]
The rule that follows is that a measurement taken on a development machine
proves nothing about the target, because the two differ in cache line size.

That rule is correct and it is not the whole picture. Development happens on
the development machine. The gate suite runs there many times a day, and its
cost is paid there and nowhere else. Today no rule owns that cost, so it grows
without anything noticing. The golden state hash test is the live instance: it
grew as each subsystem entered the state hash, and it is now the slowest gate
in a debug build.

**The options.**

1. **Keep one path.** Only the target matters. The development loop is slow and
   the project accepts it. This is the position the rules state today, by
   omission rather than by choice.
2. **Keep two paths, with different standing.** The target owns every claim
   about how the engine performs. The development machine owns a separate,
   explicitly local budget: how long the gates take. A figure from one is never
   evidence about the other, and the register says which is which.
3. **Measure both and treat them alike.** Rejected. This is what the cache line
   difference makes unsound, and it is the mistake the platform rule exists to
   prevent.

**The recommendation is option 2.**

The two quantities are not the same kind of thing. How fast the engine runs at
the target scale is a property of the engine, and the target owns it. How long
a contributor waits for the gates is a property of the development loop, and
the machine that runs it owns that. Confusing them is the error the platform
rule guards against; refusing to measure either is not what that rule asks
for.

A development budget must state that it is local and must never be cited as
evidence about the target. The blocker stays open either way, because it is
about the target and this decision does not touch it.

**What follows if the recommendation is taken.** The gate cost gets a stated
budget and a home in the reference tables, and a change that exceeds it is
visible rather than silent. The work is filed.[^DEC19]

**Owner:** the project owner. This is a judgement about what the project
values, not information the project lacks.

### DEC-013 — Which toolchain version does the project pin?

**Open.** The pin is currently the version the development machine had. That
is not a reason.

The record scope rule forbids a version in a record body, so this belongs
here and not in a record. State the property the project needs from the
toolchain, then pin the lowest version that provides it.

**Recommendation.** Decide the property first. The float ban already depends
on toolchain behaviour, because the reassociating methods do not resolve on
the current pin, and a later toolchain may make them resolvable and therefore
bannable by lint rather than by script.

### DEC-014 — Which hash does the golden state test use?

**Open.** The scaffolding chose FNV-1a. Nothing has ratified it.

This choice is load-bearing for determinism. The golden file is written by
the hash, so changing the hash invalidates every stored hash. It is cheap to
change now and expensive later, which is the shape of a decision that earns a
record once it is settled.

**Recommendation.** Confirm FNV-1a or replace it before the first golden file
is committed for real content. State the requirement the hash must meet:
exact, order-sensitive, and stable across the platforms the project builds
on.

### DEC-015 — The Python mutation gate is off

**Decided, and reversible.** The gate was removed rather than left failing,
which the definition of done requires. The Python package only re-exports the
compiled module, so no mutant is covered and the tool exits non-zero.

Turn it on when the Python package holds logic of its own. The testing policy
says how.

### DEC-016 — Type checking uses mypy, not pyright

**Decided.** Chosen to avoid a second language runtime in continuous
integration. Recorded because it was made in passing and no record holds it.

### DEC-023 — What rate does a unit gather at?

A unit told to gather takes an amount from its tile in each step. The engine
holds one rate for every unit and every ground, and the value is content.[^DEC7]

The value interacts with the stock tables. A rate far below the stock of a tile
makes a deposit last many frames, and two units on one deposit then never
contend. A rate at or above the stock empties a deposit in one frame, so the
contested case is ordinary and every test meets it.

**Option A. One rate, high against the stock of a tile.** The contested case is
the normal case, so the resolve is exercised by every scenario. A deposit lasts
one frame, which makes gathering feel instant.

**Option B. One rate, low against the stock of a tile.** A deposit lasts many
frames, which reads better. Two units contend only on a deposit that is nearly
empty, so the contested case is rare and a test must build it deliberately.

**Option C. A rate that the unit type carries.** This is the shape the project
will end at, because a unit type is data.[^DEC8] It needs a unit type table,
and none exists.

**Recommendation:** option A until a content pipeline exists, then option C.
Option B is the better game and it makes the case this subsystem exists for
rare, which is the wrong trade before the subsystem has a second reader.

**Assumption in the meantime:** option A.

### DEC-034 — What does a unit need, and how fast?

A unit carries a need that falls at an interval, and it draws a ration against
the store of the site it belongs to. Four values govern the rule: the decay of
the need, the ration, the threshold below which a unit is in deficit, and the
rate at which the deficit recovers. Every one of them is content.[^DEC7]

The engine holds the four as one rule and refuses a rate below zero. The rule
is a parameter, so a caller replaces it without touching a kernel.

The values interact. The ration equals the decay today, so a unit that receives
its whole ration holds its need level. Any other relation between the two makes
a fully served population drift up or down, which is a design choice and not an
engine constraint.

**Option A. Keep the four values as one default rule in the engine.** The
demonstration runs, and every test states the case it needs by choosing the
production of a site rather than the rule. This is what the engine does today.

**Option B. Give the rule to the caller.** The control plane sets it for each
world. This needs no new machinery, and it prices the decision on whoever
builds a world.

**Option C. Give the rule to the unit type.** This is the shape the project
will end at, because a unit type is data.[^DEC8] It needs a unit type table,
and none exists.

**Recommendation:** option A until a content pipeline exists, then option C.
Option B alone moves the choice without settling it.

**Assumption in the meantime:** option A.

### DEC-032 — What layout does the character arena hold?

The character arena holds its columns as struct-of-arrays, in the same style as
the soldier arena and the settlement arena. A register row says the character
tier wants array-of-structs, and it gives a difference of twelve cache lines
against one for a random graph gather.[^DEC9] The descent and succession pass is
not built yet, so the project must answer this before that work starts.

**The premise is misattributed, and the finding records the correction.**[^DEC10]
The twelve-against-one figure belongs to the vector report and it covers the
personality influence pass over a separate 64-byte trait record.[^DEC11] The
character report covers descent and succession, and it recommends
struct-of-arrays for the character row.[^DEC12] The two reports do not conflict,
because they describe two structures.

**Option A. Keep struct-of-arrays.** Every descent and succession kernel is a
column pass: a map to a mask and a compaction scan for eligibility, a map to a
key tuple and a sort for ranking, a counting sort for the child list, and a map
over a contiguous range for a cadet split.[^DEC12] The two operations that
gather at random, the lowest common ancestor walk and the kinship recursion,
read two or three columns for each node.

**Option B. Move the arena to array-of-structs.** This charges every column pass
a full row read to serve a gather that reads two columns. It also breaks the
zero-copy column view that the Python control plane takes for each shape.

**Option C. A hybrid, with the hot descent fields in one row.** This declares
one value at two sites unless the split is exact, and the split cannot be exact
while nothing has written the pass.[^DEC13]

**Option D. Defer for want of evidence.** The question is not blocked. It turns
on the column count of the pass, which the character report already states.

**Recommendation: option A.** Keep struct-of-arrays for the character arena.
Hold array-of-structs for the trait record, which is a separate structure that
nothing has written. A gather benchmark on a development machine measured the
crossover as a function of the column count, and the crossover sits well above
the two columns that descent reads. The figures are in the commit body, because
the machine is not the target and a measured figure decays.[^DEC14]

**Do not write a decision record yet.** The scope rule needs all three
conditions, and the second fails: the arena holds five columns and no parent
edge, so a later change is cheap.[^DEC15] The registry reserves a row for the
claim that layout follows the access pattern, and the work that adds the descent
columns should write that row.[^DEC16] The backlog holds the item.[^DEC17]

**Assumption in the meantime:** option A.

### DEC-030 — Is the founding the only way to people a world?

**Decided. It is one of two ways.** The founding is a call a caller makes. The
direct spawn stays as it is, and every fixture that spawns a unit keeps
working.

The alternative was to make the founding the only entry, and to remove the
direct spawn or to hide it. That was rejected for three reasons.

The founding is built on the direct spawn. A founding that placed a unit by
some other route would be a second write path into one arena, which is the
first recurring defect shape.[^DEC20]

A test needs to place a unit where the test chooses. A fixture that must ask
the engine where to put its units cannot build the extreme the assertion needs,
and a fixture that supplies no extreme measures itself.[^DEC21]

Every golden file would be re-recorded, and a re-recorded golden file proves
nothing about the change that caused it. A new scenario for a founded world is
the cheaper and the stronger test, because the old files stay as the control.

**What follows.** No existing fixture changes and no existing golden file
moves. The founding adds one scenario and one golden file. The demonstration
binary founds a run rather than spawning a full world, because the
demonstration is what a watcher looks at.

### DEC-031 — What does a founding score read?

**Decided for now. It reads the ground and the stock the ground carries.**

The founding happens before the first frame, so the only properties that exist
are the ones the seed fixes. The score therefore reads the terrain kind of a
place, the food and the wood and the stone within a small radius of it, how
much of that radius admits a unit, and whether open water touches it.

The product record says plainly that it does not decide which properties make
a place good, and it names water, food, high ground and reachable ground as
candidates.[^DEC22] This row records the set that was taken, so that a later
change to it is a change to something written down.

**What is not in the score.** Nothing that a run produces. No faction holding,
no neighbour settlement, no route. Each of those is a property of a world that
has stepped, and the founding runs before any of them exists.

**Revisit when** a second founding exists. A group that splits off from a
settlement chooses against a world that has stepped, and the set above is then
too small.

## Decisions to apply at merge

These are mechanical. They do not need judgement, but they must not be
forgotten.

### DEC-009 — Renumber the colliding decision ranges

Reports 10, 11 and 12 all claim D51. Report 15 overlaps report 14 at D90 to
D95. Every decision number becomes local to its record, so the collision
disappears when the records are written.

### DEC-010 — The needs report must adopt the agency report's decision cost

The needs report's cohort decision line is 16.00 core-ms and is 92 percent of
its subsystem. Corrected, it is under 0.05 core-ms. See DEC-002.

### DEC-011 — Re-run the vector storage argument

The vector report computed against a stale copy of the character report. It
used 8-byte edges at mean degree 8, giving 33.6 MB at the ceiling. The real
figure is 168 MB. The storage argument for vectors is stronger than the report
concluded, and it called that argument its weakest.

### DEC-035 — Does a settlement need a ground rule of its own?

Item 0092 refuses a settlement the ground that cannot carry one, and it reads
the passability of a tile to do it. Passability answers whether a unit may
stand on a tile. It does not answer whether a place may be built there.

The two questions come apart on ground a unit crosses and a settlement cannot
occupy. A mountain is the obvious case. Today the project has one ground
property, so the two answers are the same by accident rather than by decision.

Item 0092 states this as out of scope and settles nothing, so the question
would otherwise live only in an item body.[^DEC23]

**Option A. One ground property.** A settlement stands wherever a unit stands.
Cheapest, and it adds no second declaration site for the ground.[^DEC24]

**Option B. A second suitability property on the tile kind.** A settlement
reads its own rule. It answers the mountain case, and it prices every new
ground kind at two values instead of one.

**Assumption in the meantime:** option A. Item 0092 is written against the
passability reader, so option B is a later widening and not a rewrite.

### DEC-036 — How does a unit find the units of a lost site?

A unit carries the slot of the site it belongs to. When a settlement is
destroyed, every home naming that slot must be cleared, or the settlement
founded next in that slot feeds a population it never took. The engine clears
them by scanning every unit.[^DEC25]

The scan is correct and it is the whole population for one destruction. No
figure is stated here, because no measurement exists on the target
platform.[^DEC18]

**Option A. Keep the scan.** A destruction is rare, and the scan needs no
second structure to maintain. It is one fact in one place.[^DEC24]

**Option B. Carry a reverse index from a site to its units.** The clear
touches only the units that named the site. It adds a structure that the spawn,
the death and the home change must all maintain, and nothing fails when it
disagrees with the home column.

**Assumption in the meantime:** option A. Revisit when a rule destroys sites in
bulk rather than one at a time.

### DEC-037 — How far apart are two foundings, and may a founding widen its sample?

BLK-018 is resolved: every faction founds one group.[^DEC26] That answer needs
two rules the project does not have, and item 0094 cannot be refined without
them.[^DEC27]

**The separation.** Two groups drawn from one bounded sample can land on one
tile, or within one disc of each other. Whether a second founding refuses a
place near the first, and by how much, is a rule no record holds. A world of
sixty-three factions founding into one region makes the question sharper than
a world of four.

**The sample.** The founding record refuses a sample that widens until it
succeeds, because a sample that grows on failure has no bound.[^DEC28] A
second founding that must avoid the first will fail more often than the first
did. Either the sample stays fixed and a founding may fail, or the rule for
widening it is stated once and bounded.

**Option A. A fixed minimum separation, and a fixed sample.** A founding that
finds no admissible place fails, and a failed founding is a correct outcome
that PRD-0012 already allows. Cheapest, and it needs no new mechanism.

**Option B. A separation that scales with the faction count.** The distance
falls as more factions found, so a crowded world still seats everybody. It
introduces a second value derived from the faction count, which is a
declaration site to watch.[^DEC29]

**Option C. Partition the world and give each faction a region.** Every
faction is seated by construction and no founding fails. It decides map
structure, which is a larger claim than a founding rule, and it would need its
own record.

**Assumption in the meantime:** option A. It is the only one that adds no
mechanism, and PRD-0012 already states that a failed founding is correct.

## Closed

None yet.

[^TARGET]: Blockers register, BLK-004, and the scale constants. `docs/reference/budgets.md`
[^DEC7]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D1. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
[^DEC8]: Project orientation, the design principles. `CLAUDE.md`
[^DEC9]: Findings register, FND-022. `docs/FINDINGS.md`
[^DEC10]: Findings register, FND-072. `docs/FINDINGS.md`
[^DEC11]: Vector entity representation, section 9 and decision D155. `docs/research/reports/18-vector-entity-representation.md`
[^DEC12]: The character graph and inheritance, sections 2.1, 3.3 and 15.3. `docs/research/reports/14-character-graph-and-inheritance.md`
[^DEC13]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^DEC14]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^DEC15]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^DEC16]: ADR Registry, reserved row 0021. `docs/adrs/REGISTRY.md`
[^DEC17]: Backlog item 0097. `docs/backlog/proposed/0097-write-the-layout-record-with-the-descent-columns.md`
[^DEC18]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^DEC19]: Backlog item 0098. `docs/backlog/proposed/0098-give-the-gate-suite-a-development-budget.md`
[^DEC20]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^DEC21]: Testing rules, section 2a. `.claude/rules/testing.md`
[^DEC22]: PRD-0012, a world starts small and grows. `docs/product/shaped/prd-0012-a-world-starts-small-and-grows.md`
[^DEC23]: Backlog item 0092. `docs/backlog/refined/0092-refuse-a-settlement-on-the-ground-that-cannot-carry-one.md`
[^DEC24]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^DEC25]: ADR-0014, entity identity is an index plus a generation, decision D7. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^DEC26]: Blockers register, BLK-018. `docs/BLOCKERS.md`
[^DEC27]: Backlog item 0094. `docs/backlog/proposed/0094-decide-how-many-groups-found-a-world.md`
[^DEC28]: ADR-0075, the founding choice reads a bounded sample of the world. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
[^DEC29]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
