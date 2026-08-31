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

**Next number: DEC-022**

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

[^DEC7]: Backlog item 0030. `docs/backlog/proposed/0030-enforce-the-barrier-ordering.md`

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
the two is then a real decision.[^DEC7]

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

None yet.

[^TARGET]: Blockers register, BLK-004, and the scale constants. `docs/reference/budgets.md`
