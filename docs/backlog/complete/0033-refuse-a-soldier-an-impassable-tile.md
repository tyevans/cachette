---
id: 0033
title: Refuse a soldier an impassable tile
status: complete
created: 2026-08-30
implements: [ADR-0068 D4, ADR-0056 D1, ADR-0056 D2]
changes: []
creates: []
serves: [PRD-0003]
blocked-by: []
---

## Why

The terrain says whether a unit may stand on a tile. Nothing reads that
answer. A soldier walks into water, and no test fails.

State the wider truth plainly: **no system reads the terrain at all.** Nothing
in the engine, nothing in the viewer, and nothing in the Python control plane
reads a kind, a height, a moisture or a passability. The state hash folds
every generated tile in, so a change to the generator fails a test, but no
behaviour depends on what the ground is. This is the inert-capability shape:
the project declares a capability, tests it directly, and nothing acts on it.
The test must start at the engine.

This item is the first consumer. Backlog 0032 is the second.

## What the work does

1. The world refuses a spawn onto an impassable tile, with a named error.
2. The world refuses a placement onto an impassable tile, with the same error.
3. The movement system drops an intent that names an impassable tile. The
   soldier then stays put.
4. A test drives a stepping world and asserts that no soldier ever stands on
   water.

## Impact review

**Governed by.**

- ADR-0068 D4 states that the ground says whether a unit may stand on a tile,
  and never what a tile costs. This work is the first reader of that answer.
  The record is a draft, so the code cites it as one.
- ADR-0056 D1 states that a unit occupies exactly one tile. A refused move
  leaves the soldier on the tile it already holds.
- ADR-0056 D2 states that a move is an intent and that a separate step admits
  it. The passability test belongs to the intent half, not to admission.
- ADR-0017 D2 and D3 state that the world is a rhombus and does not wrap. An
  address outside the extent already names no tile, and that refusal stays
  separate from this one.
- ADR-0001 governs the result. The test is a pure function of the address, so
  it adds no order and no thread-count dependency.

**How admission and passability compose.** They are two different refusals,
and this item builds only the first.

Passability is a property of the ground. It refuses every unit, on every
frame, whatever else stands there. It reads one address and nothing else.

Admission is a contest between units for one tile with a capacity. It refuses
a particular unit on a particular frame, and ADR-0056 D5 says that a unit
refused this way is not stuck: it takes a lateral step or replans.

Passability therefore filters an intent before admission sees it, so admission
never weighs a tile that admits nobody. A soldier the ground refuses is not
"rejected" in the sense of D5 and takes no lateral step, because it entered no
contest. When the admission step lands, it inherits an intent list that is
already free of impassable tiles.

**Changes.** No record changes. ADR-0068 D4 already states the constraint.

**Creates.** No record. The three-condition test fails on condition three: the
constraint is already written, and the code that reads it is visible.

**Blockers.** None. DEC-017 holds the crossing cost multiplier, and this item
states no cost. It reads only the answer that ADR-0068 D4 already gives.

**Precedent.** The recurring-defect rule names the inert-capability shape and
says the test must start at the engine.[^1] The testing rule says a fixture
that supplies no extreme hides the defect, so the test must prove that its
world puts water next to a soldier.[^2]

## Outcome

The engine now reads the ground. Three paths put a soldier on a tile, and each
one refuses water: the spawn and the placement return a named error, and the
movement system drops the intent so the soldier holds its tile. A world
invariant states that no live soldier stands on ground that admits no unit,
and it fails when a later path forgets the rule.

No record changed and none was written. ADR-0068 D4 already held the
constraint, and this work is its first reader.

**The cost fell on the fixtures, not on the engine.** Eleven test fixtures and
the demonstration binary named tiles by arithmetic over the extent, and the
ground refuses a share of them. Each now takes its tiles from the open ground
of its own world.

**The golden state hash could not see the rule.** Every populated scenario was
narrower than the coarsest lattice spacing of the generator, so it held no
water and no soldier in it ever met one. The suite passed unchanged with the
rule added and with the rule removed again. A wider scenario was added, and it
fails when the rule is removed. The finding records the shape.[^3]

Admission is still not built. When it lands it inherits an intent list that
holds no impassable tile.

## References

[^1]: Recurring defect shapes, shape 3. `.claude/rules/recurring-defects.md`
[^2]: Testing rules, section 2a. `.claude/rules/testing.md`
[^3]: Findings register, FND-054. `docs/FINDINGS.md`
