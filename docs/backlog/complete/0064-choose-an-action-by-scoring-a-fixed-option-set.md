---
id: 0064
title: Choose an action by scoring a fixed option set
status: complete
created: 2026-08-31
implements: [ADR-0003 D1, ADR-0002 D1, ADR-0004 D1, ADR-0004 D4, ADR-0007 D3, ADR-0022 D1]
changes: []
creates: [ADR-0064]
serves: [PRD-0009]
blocked-by: [0053, 0055]
---

## Why

A unit draws a direction and steps. The draw is correct, repeatable and
meaningless. The unit does not read the world, so nothing in the world can
change what it does.

PRD-0009 calls itself the record the whole project points at, and this item is
the first half of answering it. Every other item in this plan builds the world
this one reads.

## What the work does

1. A unit scores a small fixed set of options and takes the highest.
2. Each option's score is one multiplication: how much the unit wants a thing,
   multiplied by how much of it is near.
3. How much of it is near comes from the level 1 cell the unit stands in, so
   the unit reads a bounded neighbourhood and never searches the world.
4. A unit whose best score is below a floor holds what it was doing and does
   not move.
5. The choice does not run on every tick. It runs at an interval, staggered so
   that the whole population does not choose at once.

## Impact review

**Governed by.**

- ADR-0022 D1. Level 0 is the only truth and every level above it is derived.
  The unit reads level 1 and writes nothing to it.[^1]
- ADR-0002 D1. Every score is an integer or a Q16.16 value, and the score is
  transient: it is compared and discarded, so it never enters simulated state.
- ADR-0004 D1 and D4. The options are scanned in index order and the tie
  breaks by the lowest option index. The option indices are distinct, so the
  order is total.
- ADR-0007 D3. The engine never calls content code inside the choice. A
  content-supplied weight is a value in a table, not a function.
- ADR-0003 D1. If any part of the choice draws, the draw is keyed. **The
  research recommends that the tie does not draw at all**, and that is the
  cheaper and the stricter answer.[^2] [^3] [^4] [^5]

**Changes.** No record changes.

**Creates.** ADR-0064. The registry reserves the row and states the claim: a
unit chooses by scoring a small fixed option set.[^6] This item writes it. The
claim passes the three-condition test in the strongest form: a contributor
would reasonably write a behaviour tree or a search, PRD-0009 rejects both by
shape, and the reason the cheap method works at all is that the option values
are precomputed once for each cell rather than once for each unit. That
reasoning is invisible in the loop.

**Two things the record must carry that are not obvious.**

The **floor** is a frame-budget parameter and not a design knob. Without it,
a world where every option scores near zero makes the tie-break decide, and the
whole population then walks in one direction. That turns every unit into a
mover and multiplies what movement costs.[^2] Record it as a budget parameter,
and state the failure it prevents.

The **stagger key is the level 1 cell, not the unit identity.** Staggering by
identity scatters the active units through the arena and destroys the locality
that makes the pass affordable at all.[^2] This is a determinism-neutral choice
with a large cost consequence, so it belongs in the record with its reason.

**Blockers.** BLK-007 governs every cost figure, so this item states none. The
option count, the interval and the floor are all parameters, and none of them
is invented here.

**Precedent.** FND-014 records that a flat field makes everyone a mover, which
is exactly the failure the floor prevents.[^7] FND-023 records that a stagger
keys on the cell index rather than the entity identity.[^8] **Both findings
already exist, and this item is the first code to read them.**

**Serves.** PRD-0009. It gives a unit no goal that outlives its interval and no
long path; PRD-0009 excludes both.

**Conflict surface.** `crates/cachette-core/src/choose.rs` is new.
`crates/cachette-core/src/soldier.rs` gains an intent column;
`crates/cachette-core/src/world.rs` at the step, before movement.
`crates/cachette-core/src/pyramid.rs` is read and not written. **It cannot run
beside item 0065**, which changes what the option set is, and it changes the
step stage that movement reads, so it does not run beside any movement work.

## Done when

- A unit chooses by reading the world. A test changes one value in the world
  and asserts that the choice changes.
- A unit responds to the ground and a unit responds to another unit, and two
  separate tests assert each.
- A unit with nothing to respond to holds its intent and does not move. A test
  asserts that an empty world produces no movement and no stuck unit.
- The tie breaks by the lowest option index, and a test constructs the tie
  rather than hoping for it.
- The score never enters the state hash, and a test asserts that changing a
  weight the world does not hold changes no hash.
- A watcher asks why a unit chose what it chose, and the engine answers.
- The choice runs as one operation over all units, with no loop in the control
  plane.
- A property test asserts that the choices are identical at 1, 2 and 12
  threads.
- A test perturbs the option order behind a test-only switch and asserts that
  the determinism test then fails.[^9]
- ADR-0064 is written, the registry row moves to `Draft`, and the record holds
  no option count, no floor value and no cost figure. Those go to the reference
  tables.[^10]
- `just check` runs green.

## Outcome

The choice pass exists and the step runs it before movement. A unit scores a
fixed set of four options against the level 1 cell it stands in, and it takes
the highest. Each score is one product of what the unit wants by how much of
that thing is near. The soldier arena gained a column that holds the option a
unit chose, and movement reads it: a unit that holds no option does not move.

ADR-0064 is written and sits at `Draft`. The registry row moved, and the record
holds no option count, no floor value and no interval value. Those went to the
reference table.[^11]

The floor and the stagger landed as this item describes. The floor is a
constant of the engine, and a test asserts that a world with nothing to respond
to produces no movement. Removing the floor fails that test. The stagger keys
on a mix of the level 1 cell index, and a test asserts that every unit of one
cell chooses on the same frame. Keying the stagger on the unit identity fails
that test.

Six existing test fixtures now set the choice interval to every tick, because
movement is not their subject and a unit does not move before it has chosen.
The golden files were regenerated.

The perturbed build reads the option set from the top, so a tie goes to the
highest option index. The probe recipe runs the choice test binary and requires
it to fail.

Two things this item did not do. It states no cost figure, because BLK-007
governs every one of them. It opened no finding and no blocker, because it
corrected nothing the project believed and found no new gap.

## References

[^1]: ADR-0022, level 0 is the only truth and every level above it is derived. `docs/adrs/REGISTRY.md`
[^2]: Individual agency and occupations. `docs/research/reports/16-individual-agency-and-occupations.md`
[^3]: ADR-0002, simulated and aggregated state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^4]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^5]: ADR-0007, content supplies a key vector, never a comparator. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^6]: ADR Registry, row 0064. `docs/adrs/REGISTRY.md`
[^7]: Findings register, FND-014. `docs/FINDINGS.md`
[^8]: Findings register, FND-023. `docs/FINDINGS.md`
[^9]: Testing Rules, section 1. `.claude/rules/testing.md`
[^10]: Budgets and costs. `docs/reference/budgets.md`
