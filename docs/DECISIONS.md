# Decisions (Register)

This document is a **register**. It lists the choices this project has made, the
options that were considered, and the outcome.

A decision needs **judgement**. The options are known and work can continue
under a stated assumption. Compare the blockers register, which lists work that
is stopped for want of information.

Numbers are permanent. Never reuse one. A closed decision keeps its row, with
the outcome recorded.

When a decision closes, and it corrected something the project believed, record
the correction in the findings register as well.


## Allocating a number

**Claim the next number below before you write the row.** Increment it in the
same change that adds the row.

A writer that numbers a row by reading the last row collides with any other
writer working at the same time. That happened, and it is recorded as
precedent.[^ALLOC]

**Next number: DEC-058**

## Open

### DEC-057 — Does a site store its resident count, or read the one the engine keeps?

**Open. The recommendation is to read the count the engine already keeps.**

Housing needs the number of units that live at a site. A review of the housing
draft found that the engine answers the question today.[^FND128] The cohort
table holds one row for each faction at each site, and each row holds a
headcount derived from the home column of the soldier arena. The residents of a
site are the sum of its rows.

**The options.**

1. Read the resident count from the cohort table. Nothing new is stored. The
   table is rebuilt inside the consumption pass, so a reader between frames sees
   the count the frame settled on.
2. Store a per-site occupancy count and maintain it by the change, with a check
   that fails when it disagrees with the home column. This is what the housing
   draft states.
3. Store the count and retire the cohort headcount, so that one site holds the
   fact.

**The recommendation is option 1.** It adds no declaration site. Option 2 makes
three sites hold one fact, and a check between two copies does not guard
three.[^SHAPE1] Option 3 is coherent, but the cohort headcount is split by
faction and the pooled draw needs that split, so retiring it moves the cost
rather than removing it.

**What holds it back.** Very little. The table is already public, and the check
that compares it against the home column is already public. What option 1 needs
is a reader that sums the rows of one site, because the table splits the count
by faction. It needs no new store and no new check.

**What follows either way.** The housing draft states decision D3 as option 2,
and it must be rewritten against whatever this row decides.[^ADR81]

### DEC-056 — May a decision record cite a product requirement record?

**Open. The recommendation is to drop the rule.**

The product guide states that a decision record cites no product requirement
record, and the project orientation repeats it. Nothing checks the rule, and
four accepted records break it.[^FND129]

**The options.**

1. Drop the rule. A decision record may cite a product record for the need that
   made a choice hard. The backlog item stays the place that names what work
   serves what need.
2. Keep the rule and enforce it. Repair the four accepted records, and add a
   check that fails a decision record which names a product file.
3. Keep the rule for the reasoning and admit the citation in a consequence. A
   record may say that a decision falsifies a product statement, and it may not
   rest a decision on one.

**The recommendation is option 1.** The reason the rule gives is that a product
direction changes more often than a constraint does. That reason is sound, and
the records that break the rule do not show the harm it predicts: each cites a
product record for a need that is stable, and none quotes a figure from one. A
rule that nobody follows and nothing checks is worse than no rule, because it
gives a reviewer an objection that the project will not support.

Option 3 is the honest reading of what the records actually do, and it is the
second choice. It costs a rule that a check cannot express.

**What holds it back.** Nothing. Work continues under any option, and only the
prose of the records moves.

### DEC-044 — Should the default ration be above the decay?

**Open. The recommendation is to raise the default ration above the decay.**

The default need rule sets the ration equal to the decay, so a unit that
receives its whole ration holds the need it has.[^DEC34REF] A unit whose need
reached zero therefore holds at zero, even when its site feeds it again. Its
need never climbs back over the threshold, so its deficit never falls.[^FND089]

The consequence changed when a shortage gained an end. A deficit that only
rises reaches the bound, so every shortage that empties a need is fatal, and
the recovery rate of the rule reaches nothing.

Three options. Raise the ration above the decay, and a fed population climbs
back to a full need; this makes a fully served population drift up, which the
clamp at the top of a need absorbs. Feed the recovery from the ration a unit
received rather than from the need it holds, which changes a kernel. Leave the
rule and accept that a shortage which empties a need is fatal.

The values are content, and no content pipeline exists.[^DEC34REF] The row
therefore asks for a default, not for a rule.

**A refined item now waits on this row.** Growth adds mouths to the same
store, so under the default rule a site that grows into a shortage loses the
population it grew and does not recover. The item states its behavioural
tests against a rule it chooses rather than against the default, so the work
can proceed, and it records the fatal default as a test of its own.[^DEC44ITEM]


### DEC-038 — Which slot does the faction take in the founding draw key?

**Decided. The faction identity fills the entity slot, and the candidate
ordinal moves to the draw index.** The decision belongs in ADR-0076, which
replaces the slot assignment in ADR-0075 D2.[^ADR75D2] ADR-0075 keeps its
number and its status.

ADR-0075 D2 puts the candidate ordinal in the entity slot. That record was
written for one founding, and one founding has no actor, so the ordinal
occupied the slot that names the actor. With one founding for each faction,
the faction is the actor. The key then means what the names of its fields say.

**Why this is the cheap answer as well as the correct one.** Every faction
that keys alike draws alike. A separation rule against an identical sample
narrows the pool that every founding after the first draws from, for a reason
that belongs to the key and not to the world. Keying on the faction gives each
faction its own sample, so a founding after the first chooses from the whole
pool rather than from what the foundings before it left. The project buys that
with a field assignment rather than with a mechanism, and DEC-037 keeps the
fixed sample it chose.

**Corrected on 1 September 2026.** This paragraph said that an identical sample
seats the first faction and refuses the rest. It does not. The sample holds many
places, they stand far apart at this extent, and a founding after the first takes
a lower-ranked place that still keeps the distance. The work that implemented
this decision removed the faction from the key and counted the factions seated
at four, six, eight and twelve factions; every faction was seated every
time.[^FND106] The decision is unchanged and the reasoning above is narrowed to
what is true. A consequence test built on the stronger claim was written,
found to catch nothing, and deleted.

**What this opens, and does not decide.** A per-faction sample can later carry
a per-faction bias, so that factions value different ground. Nothing decides
that here.

**The test that holds it.** Change the faction and assert that the sample
changes. A draw keyed on the wrong field draws the same wrong value on every
thread and every run, so both determinism tests pass while the defect
stands.[^TESTKEY]

### DEC-039 — Is a household the same thing as a place to live?

**Decided. No. A dwelling is stored and a household is derived.**

A dwelling is a structure that stands on a tile and holds a capacity. A unit
carries the slot of the dwelling it lives in. A household is every unit that
carries one slot. Nothing stores a household, and nothing declares one.

This follows the rule that level 0 is the only truth and every level above it
is derived.[^LEVEL0] A stored roster of a household would be a second
declaration of where a person lives, and nothing would fail when the roster
and the slot disagreed.[^SHAPE1]

**What follows, and it is the reason to prefer this.** A household forms when
people share a roof and dissolves when they stop. A child who takes a dwelling
of their own splits a household by moving, not by a rule that splits
households. An inheritance is a transfer of a slot. None of that needs a
kinship rule, because a household is a fact about a place rather than a fact
about a family.

### DEC-040 — How does the decision of a ruler reach a unit?

**Decided. Through the world. The ruler writes a field, and a unit reads the
level 1 cell it already reads.** No unit asks who rules it, and nothing walks
from a unit to its faction.

A unit already gathers from level 1 planes, and those gathers are the reason a
decision is cheap.[^AGENCY] A ruler contributes a source term to the influence
field of its faction. The field carries the writ. How that field is stored is
a separate question, and a proposed record holds it.[^ADR60]

**Why this is the good answer.** The influence solve carries terrain
conductance, so influence flows around a mountain rather than through
it.[^DEC5REF] The writ of a ruler therefore runs strongly near the seat and
weakly far from it, and a mountain range obstructs it. A distant province is
less governed than a near one, and the engine spends nothing to achieve that,
because the field and its conductance already exist for another reason.

**The bound.** A ruler sets a field. A ruler does not command a unit. Nothing
here gives a ruler a per-unit order, and a per-unit order would be a data
plane in Python by another route.[^ORIENT]

### DEC-041 — What does a faction with no ruler do?

**Decided. Nothing special. An absent ruler is an absent source term.**

The engine holds no branch for a faction without a ruler, and no rule asks
whether a ruler exists. The influence field of that faction simply has nothing
writing into it. The solver runs its fixed iteration count either
way.[^FIXEDITER]

**What a reader sees.** The writ relaxes from the edge inward, because the
periphery is the part the field held least strongly. The far provinces stop
being governed first, and the seat is the last place to lose its hold. An
interregnum is drift rather than a state, and whoever takes the seat inherits
the drift.

**Why it is worth choosing.** The felt behaviour a crisis needs comes free
from a solver the project already runs. A branch on the absence of a ruler
would cost a check on every pass and would produce a worse result, because it
would make the loss of a ruler instant everywhere rather than gradual from the
edge.


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

## Closed

### DEC-055 — Does a period return one unit or the whole deposit?

**Open. The recommendation is one unit, and the engine holds that today.**

A recovery period must say what it is the period of. Two readings are
available, and the record for recovery states the shape and not the
reading.[^ADR80]

**The options.**

1. The period returns one unit of stock. A deposit that lost more takes longer
   to return, in proportion to what it lost.
2. The period returns the whole deposit. Every depleted deposit is whole again
   after one period, whatever it lost.

**The recommendation is option 1.** It makes heavy extraction cost more than
light extraction, which is the statement the product record asks the world to
be able to make.[^PRD18] It also keeps the arithmetic a whole-number division
with no reference to the generated stock of the tile.

Option 2 stays available and costs the same to compute. The row exists because
a reader of the code cannot tell which reading was chosen on purpose.

**What holds it back.** Nothing. Work continues under either reading, and only
the meaning of one parameter changes.

### DEC-054 — What period does each recovering kind take?

**Open. The recommendation is one simulated day for food and several for wood.**

The decision that food and wood recover and that stone does not is closed, and
it states the period as a parameter of the kind.[^DEC49REF] It states no value.
The engine now needs one, because a default rule set must hold something.

The engine carries a default in one place, and a caller replaces the whole rule
set. The value is therefore cheap to change and no second site holds it.

**The options.** Any pair of periods. The shape does not change with the value,
so this row asks for a judgement about how a run should feel and not for
information.

**The recommendation.** Food returns fast enough that a worked patch is worth
returning to within a run, and wood returns several times slower, because a
felled wood is a longer loss than a grazed field. The engine holds that pair
until the owner or a content pipeline replaces it.

**What holds it back.** Nothing. No measurement governs a content value, and no
blocker names one.

### DEC-049 — Which resource kinds recover, and how fast?

**Decided under delegated authority, 1 September 2026. Food and wood recover.
Stone does not.** This is option 1, as recommended below.

The project owner delegated the decision for this session and left the run
unattended. This row states that work continues under any of the options and
that only the value of a parameter changes, so deciding it is cheap to reverse
and blocking the work overnight was not. **The owner may reverse this without
supersession, because it is a parameter and not a constraint.**

The reasoning is the one the recommendation gives. It matches what a player
expects. It needs no value for stone. It makes the absent case a real case that
the engine carries from the first day, rather than one somebody adds later and
then discovers the shape does not hold.

The row as it stood follows.

The product record for a deposit that comes back states the recovery period as
a parameter of the resource kind and states no value.[^PRD18]

The world holds three kinds: food, wood and stone. Two of them are alive in
the ordinary meaning and one is not, so the shape of the answer is probably
one period for each kind, with one of the periods absent.

**The options.**

1. Food and wood recover. Stone does not. A period for each of the two.
2. Every kind recovers, with stone far slower than the others.
3. One period for the whole world, and no difference between the kinds.

**The recommendation is option 1.** It matches what a player expects, it needs
no value for stone, and it makes the absent case a real case that the engine
must carry from the first day rather than a case somebody adds later.

**How to state a period.** State it in simulated time and derive the tick
count. One tick is a fixed span of simulated time, and the register holds
that constant.[^SCALE] A period given in ticks alone would go stale if the
tick span ever moved.

**What holds it back.** Nothing. Work can start under option 1 with the two
periods as parameters, and the parameters carry the same name in the engine
and in this row.

### DEC-050 — Does a deposit that reached nothing recover?

**Decided under delegated authority, 1 September 2026. A deposit that reached
nothing recovers, in the same way as any other depleted deposit.** This is
option 1, as recommended below. **The owner may reverse this without
supersession.**

The same reasoning applies as for the row above: this row states that work
continues under either option and that only the value of a parameter changes.

Option 2 stays interesting and stays unchosen. Permanent ruin is a separate
need, and the product record names it as one on purpose, so it should arrive as
a need rather than as a side effect of this parameter. Option 3 reads a
neighbourhood, so its cost stops following the depleted set alone, and no
measurement exists on the target platform to justify that.[^BLK7]

The row as it stood follows.

The product record states this as a parameter and states no value.[^PRD18]

A deposit that units emptied is a different case from one they reduced. The
question is whether the world treats it as a wound that heals or as a thing
that is gone.

**The options.**

1. It recovers, in the same way as any other depleted deposit. Nothing in the
   world is ever permanently spent.
2. It never recovers. A deposit that reached nothing stays at nothing.
3. It recovers only when a neighbouring tile of the same kind still holds
   something.

**The recommendation is option 1 for now, and option 3 is a later need.**
Option 1 is the cheap one and the one that keeps the rule uniform. Option 2
gives a player a way to ruin ground, which is interesting, and the product
record names permanent ruin as a separate need on purpose. Option 3 reads a
neighbourhood, so its cost does not follow the depleted set alone, and this
row should not choose it without a measurement the project cannot take
today.[^BLK7]

**Why this is a decision and not a blocker.** Both options are known and work
continues under either. Only the value of one parameter changes.

### DEC-043 — What deficit ends a unit?

**Outcome. One default bound in the engine, carried by the need rule, until a
content pipeline exists.**

A unit that fails its draw builds a deficit. The deficit is a rate against a
bound, and the unit ends when it reaches the bound. The bound is content, in
the same way that the decay, the ration, the threshold and the recovery are
content.[^DEC34REF]

The engine holds the bound as a fifth value of the need rule and refuses a
value below zero. A caller replaces the whole rule, so the bound is a
parameter and no kernel holds one. The condition of a unit is read against the
bound in one place, so a watcher never compares a deficit against a rule of
its own.[^SHAPE1]

A bound written into the death pass was rejected, because it is the value a
world tunes most and a kernel is the hardest place to reach. A bound derived
from the threshold was rejected, because a value derived from another value
rots silently.[^FND015]

**Revisit when** a unit type table exists. The bound then belongs to the unit
type, with the rest of the rule.

### DEC-001 — The commodity split

**Outcome. 64 commodities exist. 16 take part in the transport solve. An
individual carries 8.** The remainder stay local to a settlement.

Two reports set different ceilings, and they bound different things.

| Report | Ceiling | Reason |
|---|---|---|
| Entity economy | 64 | A presence mask is one `u64`. 64 `i64` values fill exactly 8 cache lines. |
| Trade and flow | 16, hard limit 32 | Cache residency during the flow solve. |
| Individual agency | 4 to 8 | What one individual can carry. |

The three limits are compatible, because they bound existence, participation
and carriage separately. The project therefore takes all three rather than one
of them.

### DEC-002 — Do units make individual decisions?

**Outcome. Both tiers decide. An individual chooses where to go. A cohort
chooses what to buy.**

The needs report concluded that units do not decide, because a decision cost
400 nanoseconds and one million decisions would take four times the tick
budget. The agency report measured 4.1 nanoseconds. The gathers are sequential,
not random, because units are sorted by tile index and the fields are level-1
planes that stay in cache.

The corrected cost made this a design choice and not a budget one. The project
owner asked for individual experiences, and two tiers deliver them.

### DEC-003 — Do dead characters keep relation edges?

**Outcome. A dead character drops its relation edges.**

Retention costs 531 MB at 100,000 living characters and 1.39 GB at the ceiling.
The target is now 50,000 living characters, so retention costs roughly half the
first figure at the target.[^TARGET] That scaling is derived, not measured.

The cost still exceeds the whole living character layer. Dropping the edges
loses the ability to reason about a dead person's former ties, and the project
accepts that loss.

### DEC-004 — One fog layer or two

**Outcome. Two fog layers. Explored and visible stay separate.**

The fog report specifies the two as separate layers, and asked whether both are
needed. The answer depends on whether the game shows explored terrain
differently from currently visible terrain. It does, so the project keeps both.

### DEC-005 — Does the military influence plane need terrain conductance?

**Outcome. The plane includes terrain conductance.**

With conductance the solve costs 150 microseconds. Without it, 12 microseconds.
The difference is whether influence flows around mountains or through them.
Twelve times a small number is still a small number, and influence that ignores
terrain looks wrong.

### DEC-006 — Simulated or procedural weather

**Outcome. A procedural base with a simulated perturbation, if weather is
built.** Weather is not yet in scope.

Procedural weather is a deterministic function of position, tick and seed. It
needs no storage and no update cost, and it reproduces exactly, but it gives no
feedback. Simulated weather supports an orographic rain shadow and fire-driven
weather at real cost. The base carries the cheap part, and the perturbation
buys the feedback where the project needs it.

### DEC-007 — Retained or transient event log

**Outcome. The event log stays transient.**

Retention costs 3.2 MB for each frame, which is 11.5 GB for each minute. It
would buy rollback, time travel and audit. Events are already serialisable and
the apply step is pure, so retention stays additive. The project can take it
later at the same price.

### DEC-008 — Is a 50-second mountain crossing acceptable?

**Outcome. The project accepts the 50-second mountain crossing.**

The approved calibration puts an ordinary crossing at 12.5 seconds and a
mountain crossing at 50 seconds. The project owner rejected 50 seconds as the
ordinary case, and the recalibration relocated it to mountains. A mountain pass
must be a serious obstacle.

### DEC-012 — Does a product record cite a decision record?

**Outcome. No.** Recorded here because the reasoning is easy to lose.

A product record states a need. A decision record answers to a constraint. A
product direction changes more often than a constraint does, so a citation from
a decision record to a product record would place changing material inside a
historical document, which the scope rule forbids.

The join runs the other way and through one place only: a refined backlog item
names both the record that governs it and the product record it serves. A check
enforces that a product record contains no decision record citation.

**Revisit if.** The backlog stops being the only route from a need to the work,
or a reader cannot answer "which need does this record serve" and needs to.

### DEC-013 — Which toolchain version does the project pin?

**Outcome. The project pins the current toolchain version and records the
reason. The reason is that the project tracks a recent stable release.**

**The owner chose against the recommendation.** The recommendation asked the
project to state the property it needs from the toolchain first, and then to
pin the lowest version that provides it. The owner chose the simpler rule.

What follows is that the pin carries no property statement, so a later reader
cannot tell which toolchain behaviour the project depends on. The float ban
depends on toolchain behaviour today, because the reassociating methods do not
resolve on the current pin. A later toolchain may make them resolvable, and
therefore bannable by a lint rather than by a script. Whoever raises the pin
must check that case.

The record scope rule forbids a version in a record body, so the pin belongs
here and not in a record.

### DEC-014 — Which hash does the golden state test use?

**Outcome. The project confirms FNV-1a.**

The scaffolding chose it and nothing had ratified it. The choice is
load-bearing for determinism. The golden file is written by the hash, so a
change to the hash invalidates every stored hash.

The hash must be exact, order-sensitive, and stable across the platforms the
project builds on. FNV-1a meets all three. This decision earns a record when
someone writes one, because it is cheap to change now and expensive later.

### DEC-015 — The Python mutation gate is off

**Outcome. The gate is off, and the choice is reversible.** The gate was
removed rather than left failing, which the definition of done requires. The
Python package only re-exports the compiled module, so no mutant is covered and
the tool exits non-zero.

Turn the gate on when the Python package holds logic of its own. The testing
policy says how.

### DEC-016 — Type checking uses mypy, not pyright

**Outcome. Type checking uses mypy.** The project chose it to avoid a second
language runtime in continuous integration. Recorded because the choice was
made in passing and no record holds it.

### DEC-017 — Is a tile crossing time content-configurable, or fixed by the engine?

**Outcome. The terrain step multiplier is content. It sits in the terrain table
beside the terrain capacity.**

A crossing time depends on the terrain multiplier that scales the step cost of
a tile. The alternative was to fix the multiplier in engine code, which bounds
the dwell range at compile time.

The terrain capacity table is already content, and the capacity and the
multiplier describe the same tile. A split across content and code would put
one crossing's two levers in two places. A validated range in content buys the
same compile-time bound.

**Related.** The mountain multiplier has no recorded value. The accepted
50-second mountain crossing implies a multiplier of 2 against ordinary
ground.[^MOVETIME] That value needs recording in the terrain table.

### DEC-018 — Where does movement sit in the frame schedule?

**Outcome. Movement runs after the needs system and before the combat system.**

The frame schedule is static and known before the frame runs. The order of the
systems inside it was recorded nowhere. The movement design session proposed
this order, and nobody argued for another.

Movement reads what the needs system produces, so a unit acts in the same frame
on the need that raised it. Combat then sees the positions of this frame. The
read-after-write dependency between needs and movement is real.

**Why this is a register row and not a record.** Neither the needs system nor
the combat system exists. An order between systems that nobody has written is
an intent, and a record must not state an intent as a fact. Write the order
into the schedule when the schedule exists. Promote it to a record only if a
contributor could reasonably choose otherwise and the reasoning does not show
in the schedule itself.

### DEC-019 — How many admission passes does one frame run?

**Outcome. One frame runs two admission passes.** A unit may follow one
departure.

The admission step runs a fixed number of passes. Each pass admits what it can
against the room the previous pass confirmed. The engine never runs to a
fixpoint, because a fixpoint needs a convergence test and a solver in this
project runs a fixed count.[^FIXEDITER] The record states that the count is
content and declared before the frame runs. It states no value, and no value
follows from the tile scale.

One pass admits no chain. A unit cannot follow another out of a full tile in
the same frame, so a column of units on a road advances one unit for each
frame however long the column is. Each further pass admits a chain one unit
longer, and costs one more scan of the intents. A chain longer than the pass
count waits for the next frame, which is a delay and never a wrong answer.

Two passes admit the case that a person watching the world calls obviously
correct: a unit stepping into the tile a neighbour just left. Two is the
smallest count that admits any chain. More than two buys a longer chain, and
nobody has measured what a chain costs or how long a real column is. The value
is content, so raising it later costs nothing.

**Revisit when** a measurement exists of how often a chain longer than two
appears in a run, on the target platform. That measurement waits on the
benchmark harness.[^BLK7]

### DEC-020 — Must a spawn respect the tile capacity?

**Outcome. No. A spawn may over-fill a tile, and admission is the only rule
that enforces the capacity. The engine holds no dense per-tile count.**

**This row was decided twice, and the second answer stands.** The row first
recorded that a spawn refuses a tile at capacity and that the engine holds a
dense occupancy array. The owner reversed that before any code was written.
The first answer is kept here because a reader who finds the reversed record
needs to know the project held both positions and why it moved.

Admission never raises a tile above the capacity of its ground.[^ADR56D3] A
spawn reads the faction ceiling and the passability of the ground, and it does
not read the capacity. A caller may therefore place a hundred units on one
tile and the engine accepts it. The capacity is a rule that movement obeys,
not a property of the world at rest.

**The guarantee this gives is monotone, and that is the point.** Admission
computes the room of a target as the capacity less the occupancy, and it
saturates at zero. A tile that stands above its capacity therefore admits
nobody, while its units may still leave. An over-full tile drains and never
fills further. Crowding is a state the world can reach and then relieve,
rather than a state the engine refuses to represent.

**What the project gives up.** The capacity is not a world invariant, so no
single check can state it. A test may assert that no tile gains a unit beyond
its capacity. It may not assert that no tile is ever above one.

**What this costs nowhere.** The engine already behaves this way, so the
reversal changed no code. The dense array is not written, so the occupancy
storage question that the movement record defers stays deferred, and the
project pays no second declaration of where units stand.[^SHAPE1]

### DEC-021 — Where does a structural change made outside a frame get its barrier?

**Outcome. The step opens by rebuilding a stale bridge.**

The bridge rebuilds at the barrier, after the structural apply.[^ADR18D3]
Admission reads the occupancy of a target from the bridge, so the bridge must
describe the arena before the intents are admitted. A spawn or a despawn made
between two frames is a structural change that has passed no barrier. It leaves
the bridge stale, and the first step after it then has nothing to read.

The step gives the caller's changes the barrier they never had. It costs a
revision comparison when nothing changed. The rebuild at the end of the step
stays last and stays the barrier of that frame, so one operation has two call
sites.

Two options were rejected. A rebuild by the caller before it steps makes a
correct program depend on a convention that nothing enforces, and the error it
raises names the bridge rather than the spawn that caused it. A spawn that
maintains the bridge itself stops the bridge being derived at the barrier,
which the record forbids.

**Why this is a register row and not a record.** The two call sites are the
part a reviewer might object to, and the objection is about one function rather
than about a constraint on the project. Promote it to a record if a second
structural apply lands inside the frame, because the ordering between the two
is then a real decision.

The ordering of the barrier itself is settled and enforced. A test reads it
from outside: a rebuild that ran before the structural apply leaves the derived
structure stale when the step ends, and four tests fail on that.[^ITEM0030]

### DEC-022 — May the viewer make the engine wait?

**Outcome. The project amends the product record now. It separates the two
rates when a caller needs them apart.**

The product record for the first renderable example states that the window
keeps up with the engine, or drops what it cannot draw and reports the drop,
and that it never makes the engine wait. It also states that the engine costs
the same when a viewer is attached.

The viewer record decides the opposite for now. One loop steps and then draws,
so the drawing rate and the tick rate are one number. Its consequences section
says plainly that a slow drawing slows the simulation in the demonstration
binary, and that this is acceptable for a demonstration.[^ADR67D4] The binary
also caps its own frame rate, so the engine waits on every frame that finishes
early. Nothing drops a frame and nothing reports a drop. The two block counts
the panel shows count empty spatial blocks, not dropped frames.

**This was a real contradiction, not a defect in either document.** The viewer
record knew it was choosing against the product record, and it named what would
supersede the choice.

The amendment makes the statement about waiting a statement about the engine
when a viewer is attached through a snapshot, and it excludes the demonstration
by name. The product record then describes what the project built, and it can
reach `Shipped`.

The rejected option was to separate the two rates now. The engine would run on
its own thread and publish a frame the viewer reads. That needs the snapshot
record, which does not exist. Writing the snapshot record to serve a
demonstration is the wrong order, which the viewer record already argues. Take
it when a person must watch a world that steps faster than a screen refreshes.

### DEC-023 — What rate does a unit gather at?

**Outcome. One rate, high against the stock of a tile, until a content pipeline
exists. Then a rate that the unit type carries.**

A unit told to gather takes an amount from its tile in each step. The engine
holds one rate for every unit and every ground, and the value is
content.[^ADR73D1]

The value interacts with the stock tables. A rate far below the stock of a tile
makes a deposit last many frames, and two units on one deposit then never
contend. A rate at or above the stock empties a deposit in one frame, so the
contested case is ordinary and every test meets it.

A high rate makes the contested case the normal case, so every scenario
exercises the resolve. A deposit lasts one frame, which makes gathering feel
instant. A low rate is the better game and it reads better, but it makes the
case this subsystem exists for rare, which is the wrong trade before the
subsystem has a second reader.

A rate on the unit type is the shape the project ends at, because a unit type
is data.[^ORIENT] It needs a unit type table, and none exists.

### DEC-030 — Is the founding the only way to people a world?

**Outcome. It is one of two ways.** The founding is a call a caller makes. The
direct spawn stays as it is, and every fixture that spawns a unit keeps
working.

The alternative was to make the founding the only entry, and to remove the
direct spawn or to hide it. That was rejected for three reasons.

The founding is built on the direct spawn. A founding that placed a unit by
some other route would be a second write path into one arena, which is the
first recurring defect shape.[^SHAPE1]

A test needs to place a unit where the test chooses. A fixture that must ask
the engine where to put its units cannot build the extreme the assertion needs,
and a fixture that supplies no extreme measures itself.[^TEST2A]

Every golden file would be re-recorded, and a re-recorded golden file proves
nothing about the change that caused it. A new scenario for a founded world is
the cheaper and the stronger test, because the old files stay as the control.

**What follows.** No existing fixture changes and no existing golden file
moves. The founding adds one scenario and one golden file. The demonstration
binary founds a run rather than spawning a full world, because the
demonstration is what a watcher looks at.

### DEC-031 — What does a founding score read?

**Outcome. It reads the ground and the stock the ground carries.**

The founding happens before the first frame, so the only properties that exist
are the ones the seed fixes. The score therefore reads the terrain kind of a
place, the food and the wood and the stone within a small radius of it, how
much of that radius admits a unit, and whether open water touches it.

The product record says plainly that it does not decide which properties make a
place good, and it names water, food, high ground and reachable ground as
candidates.[^PRD12] This row records the set that was taken, so that a later
change to it is a change to something written down.

**What is not in the score.** Nothing that a run produces. No faction holding,
no neighbour settlement, no route. Each of those is a property of a world that
has stepped, and the founding runs before any of them exists.

**Revisit when** a second founding exists. A group that splits off from a
settlement chooses against a world that has stepped, and the set above is then
too small.

### DEC-032 — What layout does the character arena hold?

**Outcome. The character arena keeps struct-of-arrays.** The trait record holds
array-of-structs, and it is a separate structure that nothing has written.

The character arena holds its columns as struct-of-arrays, in the same style as
the soldier arena and the settlement arena. A register row said the character
tier wants array-of-structs, and it gave a difference of twelve cache lines
against one for a random graph gather.[^FND022]

**That premise is misattributed, and the finding records the
correction.**[^FND072] The twelve-against-one figure belongs to the vector
report and it covers the personality influence pass over a separate 64-byte
trait record.[^REP18] The character report covers descent and succession, and
it recommends struct-of-arrays for the character row.[^REP14] The two reports
do not conflict, because they describe two structures.

Every descent and succession kernel is a column pass: a map to a mask and a
compaction scan for eligibility, a map to a key tuple and a sort for ranking, a
counting sort for the child list, and a map over a contiguous range for a cadet
split.[^REP14] The two operations that gather at random, the lowest common
ancestor walk and the kinship recursion, read two or three columns for each
node.

Array-of-structs would charge every column pass a full row read to serve a
gather that reads two columns. It would also break the zero-copy column view
that the Python control plane takes for each shape. A hybrid, with the hot
descent fields in one row, declares one value at two sites unless the split is
exact, and the split cannot be exact while nothing has written the
pass.[^SHAPE1]

A gather benchmark on a development machine measured the crossover as a
function of the column count, and the crossover sits well above the two columns
that descent reads. The figures are in the commit body, because the machine is
not the target and a measured figure decays.[^BLK7]

**Do not write a decision record yet.** The scope rule needs all three
conditions, and the second fails: the arena holds five columns and no parent
edge, so a later change is cheap.[^SCOPE1] The registry reserves a row for the
claim that layout follows the access pattern, and the work that adds the
descent columns should write that row.[^REG21] The backlog holds the
item.[^ITEM0097]

### DEC-033 — Does the project keep a performance path for the development machine?

**Outcome. The project keeps two performance paths, and they have different
standing.** The target owns every claim about how the engine performs. The
development machine owns a local gate-time budget, and that budget is never
evidence about the target.

Every cost figure in this project is derived and belongs to the target, and one
open blocker states that no measurement exists there.[^BLK7] The rule that
follows is that a measurement taken on a development machine proves nothing
about the target, because the two differ in cache line size.

That rule is correct and it is not the whole picture. Development happens on
the development machine. The gate suite runs there many times a day, and its
cost is paid there and nowhere else. No rule owned that cost, so it grew
without anything noticing. The golden state hash test is the live instance: it
grew as each subsystem entered the state hash, and it is now the slowest gate
in a debug build.

The two quantities are not the same kind of thing. How fast the engine runs at
the target scale is a property of the engine, and the target owns it. How long
a contributor waits for the gates is a property of the development loop, and
the machine that runs it owns that. To measure both and treat them alike was
rejected, because the cache line difference makes it unsound, and that is the
mistake the platform rule exists to prevent.

**What follows.** A development budget must state that it is local and must
never be cited as evidence about the target. The gate cost gets a stated budget
and a home in the reference tables, and a change that exceeds it is visible
rather than silent. The work is filed.[^ITEM0098] The blocker stays open,
because it is about the target and this decision does not touch it.

### DEC-034 — What does a unit need, and how fast?

**Outcome. One default rule in the engine, until a content pipeline exists.
Then a rule that the unit type carries.**

A unit carries a need that falls at an interval, and it draws a ration against
the store of the site it belongs to. Four values govern the rule: the decay of
the need, the ration, the threshold below which a unit is in deficit, and the
rate at which the deficit recovers. Every one of them is content.[^ADR73D1]

The engine holds the four as one rule and refuses a rate below zero. The rule
is a parameter, so a caller replaces it without touching a kernel.

The values interact. The ration equals the decay today, so a unit that receives
its whole ration holds its need level. Any other relation between the two makes
a fully served population drift up or down, which is a design choice and not an
engine constraint.

The engine default is what the engine does today. The demonstration runs, and
every test states the case it needs by choosing the production of a site rather
than the rule. To give the rule to the control plane for each world moves the
choice without settling it. A rule on the unit type is the shape the project
ends at, because a unit type is data.[^ORIENT] It needs a unit type table, and
none exists.

### DEC-035 — Does a settlement need a ground rule of its own?

**Outcome. The tile kind carries a second suitability property. A settlement
reads its own rule.**

**The owner chose against the assumption in force.** Work proceeded on the
assumption of one ground property, and item 0092 is written against the
passability reader. The second property is a widening of that item rather than
a rewrite, but the item and the tile kind both need the new value.

Item 0092 refuses a settlement the ground that cannot carry one, and it reads
the passability of a tile to do it. Passability answers whether a unit may
stand on a tile. It does not answer whether a place may be built there. The two
questions come apart on ground a unit crosses and a settlement cannot occupy. A
mountain is the obvious case. The project had one ground property, so the two
answers were the same by accident rather than by decision.

What follows is that every new ground kind is priced at two values instead of
one. The project accepts that price, because the mountain case is real. Item
0092 states the question as out of scope and settles nothing, so the question
would otherwise live only in an item body.[^ITEM0092]

### DEC-036 — How does a unit find the units of a lost site?

**Outcome. The engine keeps the scan.**

A unit carries the slot of the site it belongs to. When a settlement is
destroyed, every home naming that slot must be cleared, or the settlement
founded next in that slot feeds a population it never took. The engine clears
them by scanning every unit.[^ADR14D7]

The scan is correct and it is the whole population for one destruction. No
figure is stated here, because no measurement exists on the target
platform.[^BLK7] A destruction is rare, and the scan needs no second structure
to maintain. It is one fact in one place.[^SHAPE1]

A reverse index from a site to its units would touch only the units that named
the site. It adds a structure that the spawn, the death and the home change
must all maintain, and nothing fails when it disagrees with the home column.

**Revisit when** a rule destroys sites in bulk rather than one at a time.

### DEC-051 — Which slot of the draw key holds the faction?

**Outcome. The frame slot holds the faction.** The candidate ordinal keeps the
entity slot, and the axis keeps the draw slot.[^ADR75D2] A record holds the
decision.[^ADR76]

A founding happens before the first frame, so the frame slot carried a
constant. It now carries the faction, and two factions read two samples. The
key keeps the shape the determinism record fixes, and no slot carries two
meanings.[^KEYED]

Two options were rejected. **An amendment to the founding record** would state
the key for several foundings inside a record written for one. That record is
accepted and it is still true, and an accepted record changes only by
supersession. **A fold of the faction and the ordinal into the entity slot**
puts two values in one slot, so a later change to either one can collide with
the other, and nothing would fail when it did.[^SHAPE1]


### DEC-037 — How far apart are two foundings, and may a founding widen its sample?

**Outcome. A fixed minimum separation, and a fixed sample.** A founding that
finds no admissible place fails, and a failed founding is a correct outcome.

Every faction founds one group.[^BLK18] That answer needs two rules the project
did not have, and item 0094 could not be refined without them.[^ITEM0094]

**The separation.** Two groups drawn from one bounded sample can land on one
tile, or within one disc of each other. Whether a second founding refuses a
place near the first, and by how much, was a rule no record held. A world of
sixty-three factions founding into one region makes the question sharper than a
world of four.

**The sample.** The founding record refuses a sample that widens until it
succeeds, because a sample that grows on failure has no bound.[^ADR75] A second
founding that must avoid the first fails more often than the first did. The
fixed sample keeps the bound and accepts the failure.

Two options were rejected. A separation that scales with the faction count
seats everybody in a crowded world, but it introduces a second value derived
from the faction count, which is a declaration site to watch.[^SHAPE1] A
partition of the world into one region for each faction seats every faction by
construction, but it decides map structure, which is a larger claim than a
founding rule and would need its own record.

The chosen option adds no mechanism, and the product record already states that
a failed founding is correct.[^PRD12]

## References

[^ADR75D2]: ADR-0075, the founding choice reads a bounded sample of the world, decision D2. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
[^ADR76]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D3. `docs/adrs/draft/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
[^KEYED]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^TESTKEY]: Testing rules, section 2. `.claude/rules/testing.md`
[^LEVEL0]: ADR-0022, level 0 is the only truth and every level above it is derived. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^AGENCY]: Individual agency and occupations, the decision cost, and DEC-002 above. `docs/research/reports/16-individual-agency-and-occupations.md`
[^ADR60]: ADR Registry, proposed row 0060, an influence map is stored as a shared basis. `docs/adrs/REGISTRY.md`
[^DEC5REF]: See DEC-005 in this document.
[^PRD18]: Product record PRD-0018, a depleted deposit comes back. `docs/product/shaped/prd-0018-a-depleted-deposit-comes-back.md`
[^SCALE]: Budgets and costs, the scale constants. `docs/reference/budgets.md`

[^ALLOC]: Findings register, FND-038. `docs/FINDINGS.md`
[^TARGET]: Blockers register, BLK-004, and the scale constants. `docs/reference/budgets.md`
[^MOVETIME]: The movement timing note, and DEC-008 above. `docs/research/movement-timing.md`
[^FIXEDITER]: ADR-0005, a solver runs a fixed iteration count, decision D1. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
[^BLK7]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^ADR56D3]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^ADR56D4]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^ADR18D3]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^ITEM0030]: Backlog item 0030. `docs/backlog/complete/0030-enforce-the-barrier-ordering.md`
[^ADR67D4]: ADR-0067, the viewer reads the world and never writes to it, decision D4 and its consequences. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^ADR73D1]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D1. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
[^ORIENT]: Project orientation, the design principles. `CLAUDE.md`
[^SHAPE1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^TEST2A]: Testing rules, section 2a. `.claude/rules/testing.md`
[^PRD12]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
[^FND022]: Findings register, FND-022. `docs/FINDINGS.md`
[^FND072]: Findings register, FND-072. `docs/FINDINGS.md`
[^REP18]: Vector entity representation, section 9 and decision D155. `docs/research/reports/18-vector-entity-representation.md`
[^REP14]: The character graph and inheritance, sections 2.1, 3.3 and 15.3. `docs/research/reports/14-character-graph-and-inheritance.md`
[^SCOPE1]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^REG21]: ADR Registry, reserved row 0021. `docs/adrs/REGISTRY.md`
[^ITEM0097]: Backlog item 0097. `docs/backlog/refined/0097-write-the-layout-record-with-the-descent-columns.md`
[^ITEM0098]: Backlog item 0098. `docs/backlog/complete/0098-give-the-gate-suite-a-development-budget.md`
[^ITEM0092]: Backlog item 0092. `docs/backlog/complete/0092-refuse-a-settlement-on-the-ground-that-cannot-carry-one.md`
[^ADR14D7]: ADR-0014, entity identity is an index plus a generation, decision D7. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^BLK18]: Blockers register, BLK-018. `docs/BLOCKERS.md`
[^ITEM0094]: Backlog item 0094. `docs/backlog/complete/0094-decide-how-many-groups-found-a-world.md`
[^FND128]: Findings register, FND-128. `docs/FINDINGS.md`
[^FND129]: Findings register, FND-129. `docs/FINDINGS.md`
[^ADR81]: ADR-0081, a residence is a stored column and occupancy is a maintained count, decision D3. `docs/adrs/draft/adr-0081-a-residence-is-a-stored-column-and-occupancy-is-a-maintained-count.md`
[^ADR75]: ADR-0075, the founding choice reads a bounded sample of the world. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
<<<<<<< HEAD
[^FND106]: Findings register, FND-106. `docs/FINDINGS.md`
[^DEC44ITEM]: Backlog item 0060. `docs/backlog/proposed/0060-grow-the-population-from-the-store-and-the-housing.md`
=======
>>>>>>> worktree-agent-a4ea51fa97c6b231a
