# ADR-0064: A unit chooses by scoring a small fixed option set

## Context

A unit draws a direction and steps. The draw is keyed, so it repeats.[^1] It is
also meaningless: the unit reads nothing, so nothing in the world can change
what it does. The engine has a world and a population, and no rule that joins
them.

This record states how a unit decides. It does not state what a unit weighs.

**The first force is the target scale.** The project simulates a million units,
and every one of them decides.[^2] A behaviour tree walks a graph for each
unit. A search expands nodes for each unit. Both are affordable for a named
leader and neither is affordable for a population, so the method must cost a
fixed small number of integer operations for each unit.[^3]

**The second force is that the cheap method only works because of where the
values come from.** A unit reads the summary of the level 1 cell it stands in.
The engine computes that summary once for each cell, at the barrier, and every
unit in the cell then reads it.[^4] The reasoning that makes the loop
affordable is therefore invisible in the loop. It lives in the fact that the
engine precomputes the option values for each cell rather than for each unit.

**The third force is exactness.** A score must repeat at any thread count and
on any machine. Integer multiplication and integer addition are exactly
commutative and exactly associative. A floating point score is not, and the
project bans it from simulated state anyway.[^5] The method survives that ban
with no loss, because a weighted product of integers is exact.

**The fourth force is that a near-flat world is the failure mode.** When every
option scores near zero, the comparison chain still selects something. Every
unit then selects the same option, and the whole population walks one way. The
project has recorded that shape already: a flat field makes everyone a
mover.[^6] The movement stage is sized for a part of the population, so that
failure multiplies what movement costs.

**The fifth force is locality.** The choice is periodic, so a schedule decides
who re-reads the world on which frame. The engine holds the unit array in tile
order. A schedule keyed on the identity of the unit therefore selects a
scattered set, and a scattered set reads a whole cache line to use a few bytes
of it. The project has recorded that shape as well.[^7]

**A contributor could reasonably choose otherwise on every point above.** A
behaviour tree and a utility score are both standard. A random tie-break is the
obvious answer to a tie. An identity stagger is the obvious schedule. Each
alternative is cheap to write and expensive to undo, and none of the reasoning
is visible in the code that results.

## Decision

**A unit chooses by scoring a small fixed option set against the level 1 cell
it stands in, and it takes the highest score.**

### D1. A unit scores a fixed option set and takes the highest score

The option set is fixed at compile time. Every unit scores every option in it.

Each score is one product: what the unit wants, multiplied by how much of that
thing is near. What the unit wants comes from the unit and from a weight in a
table. How much is near comes from the summary of the level 1 cell.

The unit reads that one cell. It never searches the world, and it never walks a
neighbourhood of its own choosing. The cost of the pass is therefore the option
count times the population, and nothing else.

The engine calls no content code inside the choice. A content author supplies a
weight, which is a value in a table and never a function.[^8]

Every arithmetic operation goes through the arithmetic module, and every
operation saturates rather than wraps.[^9]

### D2. The score is transient, and only the choice reaches state

Nothing stores a score. The pass compares the scores and discards them, so no
score reaches simulated state and no score reaches the state hash.[^5]

The unit stores the option it selected. That value decides a later frame, so it
is state and the state hash covers it.[^10]

The stored choice is sticky. A unit keeps it between two choices. A unit that
re-decides on every tick swaps between two options of nearly equal score and
arrives nowhere, so stickiness is what makes the behaviour legible.[^3]

The engine answers a question about a choice by computing the scores again from
the world as it stands. It does not answer from a stored score, because no
stored score exists.

### D3. An option below a floor holds the choice, and the floor is a frame-budget parameter

A unit whose highest score is below a floor holds what it was doing and does
not move.

**The floor is a frame-budget parameter. It is not a design knob.** Without it,
a world in which every option scores near zero lets the tie-break decide. Every
unit then selects the lowest option index, the whole population walks one way,
and every unit becomes a mover.[^6] A change to the floor changes the mover
count, and the mover count is what the project sizes the movement stage
against.

The value is a parameter of the engine, and the reference table holds it with
its derivation.[^11] The project derives every cost figure rather than
measuring it, and one blocker governs that.[^12]

### D4. The choice runs at an interval, and the stagger key is the level 1 cell

The choice does not run on every tick. It runs at an interval, and a stagger
spreads the population across the ticks of that interval.

**The stagger key is the level 1 cell, and it is never the identity of the
unit.** The engine holds the unit array in tile order, and the cell index is a
fixed function of the tile index, so a cell key selects a few long runs that
lie together. An identity key selects the same number of units and scatters
them.[^7] This choice is neutral for determinism and large for cost, which is
why this record holds it rather than the code.

The key mixes the cell index. A bare mask of the cell index selects a regular
stripe of the map on each tick, which ties the phase of the decision to the
geography.

The schedule is a pure function of the cell and the frame. It reads no counter
and no accumulator, so it gives one answer at any thread count.[^13]

A unit that crosses a cell boundary can choose twice inside one interval, or
skip one interval. **This is accepted behaviour and not a defect.** A unit that
arrives in a new region must read it again, and a skipped interval delays a
choice rather than losing it.

The interval is a parameter of the world, and the reference table holds the
recommended value.[^11]

### D5. The tie breaks by the lowest option index, and never by a draw

The pass reads the options in ascending option index and compares with a strict
greater-than. The lowest option index therefore wins a tie.

The option indices are distinct, so the order is total and the pass needs no
second key.[^14]

**A tie does not draw.** A keyed draw would repeat, and it would add a
generator call to the hot loop for a case that carries no design value.[^1]
[^3]

### D6. The choice reads level 1 and writes nothing above level 0

The pass reads the summary that the last barrier built. It writes the choice
column, which is level 0. It writes nothing to any level above level 0.[^4]
[^15]

The level a unit read is part of the answer it gave.[^16] A unit therefore acts
on the world as the last barrier left it, and not on a world that a later stage
of the same frame has changed.

## Consequences

A unit now acts on the world. Every other subsystem that changes a level 1
summary changes behaviour, whether or not it meant to.

A unit that has never chosen holds no choice, and it does not move. A world
that has just spawned a population is therefore still until the interval of
each cell comes round. A test about movement states the interval it needs.

A unit cannot hold a goal that outlives its interval, and it cannot follow a
long path. The product record asks for this, so it is a limit and not a stage
of the work.[^17]

This record does not bind the character tier. A named entity is few enough to
afford a real planner, and this method exists because a population is not.[^3]

The engine cannot rank two options by a rule that reads two cells. The unit
reads one cell, and a wider read would remove the property that makes the pass
affordable.

A content author cannot supply a function. An author supplies a weight.[^8] An
author who needs a new behaviour needs a new option in the set, and the engine
owns the set.

The order of the option set is now behaviour. A change to the order changes
which option wins a tie, so the order is not a listing that anyone may sort.

## Alternatives rejected

**A behaviour tree for each unit.** A tree walks a graph for each unit and each
tick, and the walk is a chain of dependent branches. It is the standard answer
and it is affordable for a named entity. It is not affordable for a
population.[^3]

**A search or a planner for each unit.** A planner gives a unit a goal that
outlives one tick and a path towards it. The product record excludes both by
shape, and the cost is far above a tree.[^17] [^3]

**A floating point utility score.** The literature uses this, and the project
bans it from simulated state. The ban costs nothing here: a weighted product of
integers repeats exactly, and a floating point sum does not.[^5]

**A random tie-break.** A keyed draw would repeat, so it would not break
determinism. It would add a generator call to the hot loop and buy nothing. A
tie between two options of equal value is not a case that needs variety.[^3]

**A stagger keyed on the identity of the unit.** This is the obvious schedule
and it repeats. It selects a scattered set out of an array held in tile order,
and the project has recorded what that costs.[^7]

**No floor at all.** The comparison chain always selects something, so a rule
with no floor is shorter. It also turns a near-flat world into a walk of the
whole population, which is the failure this record exists to prevent.[^6]

## References

[^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^2]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^3]: Research report 16, individual agency and occupations, section 3. `docs/research/reports/16-individual-agency-and-occupations.md`
[^4]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^5]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^6]: Findings register, FND-014. `docs/FINDINGS.md`
[^7]: Findings register, FND-023. `docs/FINDINGS.md`
[^8]: ADR-0007, content supplies a key vector, never a comparator, decision D3. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^9]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^10]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^11]: Budgets and costs, the choice pass. `docs/reference/budgets.md`
[^12]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^13]: ADR-0001, one binary gives one answer at any thread count, decision D5. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^14]: ADR-0004, iteration order is explicit, decision D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^15]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D3. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^16]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D4. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^17]: PRD-0009, a unit acts on the world it can see. `docs/product/shaped/prd-0009-a-unit-acts-on-the-world-it-can-see.md`
