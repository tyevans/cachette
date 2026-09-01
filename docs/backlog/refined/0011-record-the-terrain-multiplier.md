---
id: 0011
title: Record the terrain multiplier and reconcile the crossing time
status: refined
created: 2026-08-30
implements: [ADR-0056 D4]
changes: []
creates: []
serves: []
blocked-by: [BLK-007]
---

## Why

A crossing time depends on the terrain multiplier. The multiplier scales the
step cost of a tile. No table states the multiplier, so the engine cannot
compute a crossing time that depends on the ground.

The register answers where the multiplier lives. The terrain step multiplier
is content. It sits in the terrain table, beside the terrain capacity.[^1] The
capacity and the multiplier describe the same tile. A split across content and
code would put one crossing's two levers in two places. Content states a
validated range, and that range gives the bound that engine code would give.

The register also fixes the crossing times. An ordinary crossing takes
12.5 seconds and a mountain crossing takes 50 seconds.[^2] A mountain
multiplier of 2 against ordinary ground follows from those two figures. **That
value is derived, not measured.** Nobody has decided it directly.

A second loose end stays open. The movement timing note gives 12.9 seconds for
a dwell-2 baseline with a capacity-16 crossing. The closed-form throughput law
gives 12.5 seconds for the same parameters.[^3] The difference is 4 ticks and
nobody has explained it. The likely cause is the entry tick and the clearing
tick, which the steady-state law omits. Nobody has checked that.

## Impact review

**Governed by.** ADR-0056 D4 states that capacity is a data-driven property of
the terrain, and that a literal in the movement kernel is the violation the
decision names. The multiplier is a second property of the same kind, so it
takes the same home. ADR-0056 D1 states that a unit occupies one tile, so a
step cost is a property of the tile the unit enters.

**Changes.** No record changes. ADR-0056 D4 already covers a data-driven
terrain property, so the multiplier extends the decision rather than
contradicting it. Do not amend the accepted record to name the multiplier.

**Creates.** No record. A future contributor cannot reasonably put the
multiplier in engine code once the capacity sits in the terrain table, and the
register holds the reasoning for that. The value itself is a figure, and a
record holds no figure.[^4]

**Blockers.** BLK-007 stays open. No measurement exists on the target
platform, so every crossing figure in this project is derived. State the
multiplier as derived wherever the work writes it. BLK-001 and BLK-009 are
closed. The tile edge, the dwell and both capacities are fixed, so the
movement constants no longer need to stay parametric.

**Precedent.** FND-037 records that a crossing time is a function of three
quantities, not two. Capacity, dwell and the terrain multiplier all enter it.
The finding says that no record states the mountain multiplier, so the
50-second figure implies its value rather than citing it. This work closes
that gap.

**Product record.** None.

## What fails if somebody changes it back

A later reader could put a step-cost literal back into the movement kernel, or
could change the mountain multiplier to a different value. Both changes are
silent today.

- A test asserts that the terrain table answers a step multiplier for every
  terrain kind. A kind with no answer fails it.
- A test asserts that a mountain step costs twice an ordinary step, and it
  reads both values through the public interface. A changed multiplier fails
  it.
- A test asserts that the multiplier of every kind sits inside the validated
  range. A value outside the range fails it, and the range is the bound that
  engine code would otherwise give.
- The float ban script and the arithmetic module already refuse a
  floating-point multiplier, so the multiplier is a Q16.16 value.

Put the defect back before you claim any of these tests covers the case. A
test that stays green with the literal restored measures the fixture, not the
behaviour.[^5]

## Done when

- The terrain table answers a step multiplier for each terrain kind, beside
  the capacity.
- The multiplier is an integer or a Q16.16 value. No floating point enters it.
- The scale constants table holds the mountain multiplier, and the row says
  the value is derived from the two accepted crossing times.
- No step-cost literal remains in the movement kernel. A whole-tree search for
  the literal comes back clean, and the search command is in the commit body.
- The tests above exist and each one has been proven able to fail.
- The 12.9-second and 12.5-second figures are reconciled, or the item states
  plainly that the difference is unexplained and names what would explain it.
- FND-037 records the outcome.
- The whole check command runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Decisions register, DEC-017. `docs/DECISIONS.md`
[^2]: Decisions register, DEC-008. `docs/DECISIONS.md`
[^3]: Movement timing note. `docs/research/movement-timing.md`
[^4]: Decision Record Scope, section 4.1. `.claude/rules/adr-scope.md`
[^5]: Testing Rules, section 2a. `.claude/rules/testing.md`
