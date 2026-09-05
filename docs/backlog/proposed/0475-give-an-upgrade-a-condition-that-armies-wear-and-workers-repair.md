---
id: 0475
title: Give an upgrade a condition that armies wear and workers repair
status: proposed
created: 2026-09-05
implements: [ADR-0145, ADR-0146, ADR-0002 D1, ADR-0001 D4]
changes: []
creates: []
serves: [PRD-0052]
blocked-by: [BLK-007, BLK-036]
---

## Why

**An upgrade lasts forever once it is built.** Nothing wears it and nothing
repairs it, so a road is a fact and not a possession. This item is pass 4 of
the living world game layer.[^1]

Each upgrade entry gains an integer condition. Completion sets it to full, and
the full value per kind is a row in the balance register.[^2] A hostile unit
standing on the tile wears the upgrade by one integer step per tick. A hostile
unit is one whose faction is in the war band toward the holder. Condition zero
means the upgrade is gone, and the engine removes it through the existing
destroy path.

Repair is `order_build` on a tile that already carries an upgrade. The build
pass adds the build rate of the unit type to the condition, clamped at full.
No new verb exists.

The wall joins the upgrade kinds. It raises the movement cost for a unit whose
faction does not hold the tile, and it absorbs contest harm on its tile before
any unit falls. The work of the wall and the absorption are rows in the
balance register.

**This pass touches `fn step` in `world.rs`. Only one worker may hold it at a
time.** It waits for pass 3 to merge, because a hostile unit is defined by the
war band.

## What is missing before this is refined

- The impact review, decision by decision. The pass reads the build rate
  column of ADR-0145 and the war band of ADR-0146, and both records are being
  written beside this item.[^3]
- Whether a record on the upgrade catalogue governs the new kind. Item 0348
  asks to make the catalogue a table that the world is built with, and the wall
  is a variant of an enumeration today. The review must say which lands first.
- Whether BLK-036 touches wear. An army that wears an upgrade stands on ground
  that its faction may now hold, and the blocker asks whether the upgrade
  changed hands with the ground.[^4] The review must state the wear rule
  parametrically if it does.
- The per-field tests and the extreme that the fixture reaches: a condition
  at one, so one step destroys, and a condition at full, so a repair clamps.
- The "Done when" statements, in the shape of item 0472: the two determinism
  tests at 1, 2 and 12 threads, the defect put back and the test red, and the
  type stub edited by hand in the same commit as any new reader.[^5]

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Design: the living world game layer, sections 6 and 13. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^2]: Balance register. `docs/reference/balance.md`
[^3]: ADR Registry. `docs/adrs/REGISTRY.md`
[^4]: Blockers register, BLK-036. `docs/BLOCKERS.md`
[^5]: Findings register, FND-320. `docs/FINDINGS.md`
