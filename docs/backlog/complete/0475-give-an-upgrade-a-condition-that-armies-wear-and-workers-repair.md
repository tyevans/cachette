---
id: 0475
title: Give an upgrade a condition that armies wear and workers repair
status: complete
created: 2026-09-05
implements: [ADR-0145, ADR-0146, ADR-0151 D3, ADR-0151 D6, ADR-0002 D1, ADR-0001 D4]
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
the full value of each row is a row of the balance register.[^2] A hostile unit
standing on the tile wears the upgrade by one integer step per tick. A hostile
unit is one whose faction is in the war band toward the holder. Condition zero
means the upgrade is gone, and the engine removes it through the existing
destroy path.

Repair is `order_build` on a tile that already carries an upgrade. The build
pass adds the build rate of the unit type to the condition, clamped at full.
No new verb exists.

The wall joins the upgrade table as a row and never as a variant.[^3] It
raises the movement cost for a unit whose faction does not hold the tile, and
it absorbs contest harm on its tile before any unit falls. The work of the wall
and the absorption are rows in the balance register.

Repair comes before a raise. The build order on a worn entry raises the
condition first, and the work reaches the next level only from full
condition.[^3] The condition and the work done are two fields of one entry.

**This pass touches `fn step` in `world.rs`. Only one worker may hold it at a
time.** It waits for pass 3 to merge, because a hostile unit is defined by the
war band. Item 0486 is complete, so the table is in place: the wall is a row of
it with a work and no effect column, and the condition is a second field of the
entry beside the work done.

## What is missing before this is refined

- The impact review, decision by decision. The pass reads the build rate
  column of ADR-0145 and the war band of ADR-0146, and both records are being
  written beside this item.[^4]
- How the wall row states its ground fit, and which columns the movement
  pass and the contest pass read for the cost raise and the absorption. A
  column exists when a pass reads it, and the review names both.[^3]
- Whether BLK-036 touches wear. An army that wears an upgrade stands on ground
  that its faction may now hold, and the blocker asks whether the upgrade
  changed hands with the ground.[^5] The review must state the wear rule
  parametrically if it does.
- The per-field tests and the extreme that the fixture reaches: a condition
  at one, so one step destroys, and a condition at full, so a repair clamps.
- The "Done when" statements, in the shape of item 0472: the two determinism
  tests at 1, 2 and 12 threads, the defect put back and the test red, and the
  type stub edited by hand in the same commit as any new reader.[^6]

## Done when

Stated when the item is refined.

## Outcome

Complete. An upgrade entry now holds a condition, two causes take it, a
worker mends it and a site at nothing is removed.

**What was built.** `UpgradeSite` gained a condition on one scale that every
category shares. `Stage::UpgradeWear` runs after the build advance and walks the
sites in ascending tile order, summing a weather cause and a hostile-unit cause.
`UpgradeMap::wear_ascending` returns the tiles that collapsed, and a collapsed
site is dropped rather than stored at nothing. `resolve_build_row` gained the
arm that resolves a damaged site to the row standing there, so a worker can mend
a top-level upgrade. Two public readers answer the condition and the collapse
count of a tick.

**A defect found and repaired inside the item.** The repair first spent a whole
builder tick whatever the gap cost, so a level under any wear at all could never
rise again and the site collapsed at about tick 2088. The repair is now priced
at the gap it closes, and a gap worth less than one unit of work is free.[^C1]

**The record this item creates.** No record covered the condition. ADR-0169 now
does, and it changes ADR-0151 D3, which said that the work reaches the next
level only from full condition.[^C2]

**Left open.** Neither wear rate reads the category, because no column resists
wear. A wall and a road therefore wear alike. The balance register holds both
rates, unset under their blockers.[^C3]

## References

[^1]: Design: the living world game layer, sections 6 and 13. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^2]: Balance register. `docs/reference/balance.md`
[^3]: ADR-0151, an upgrade is a category with a ground fit and a level, and a build order names the category, decisions D3 and D6. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
[^4]: ADR Registry. `docs/adrs/REGISTRY.md`
[^5]: Blockers register, BLK-036. `docs/BLOCKERS.md`
[^6]: Findings register, FND-320. `docs/FINDINGS.md`
[^C1]: Findings register, FND-556. `docs/FINDINGS.md`
[^C2]: ADR-0169, an upgrade holds a condition that wear takes and work mends. `docs/adrs/draft/adr-0169-an-upgrade-holds-a-condition-that-wear-takes-and-work-mends.md`
[^C3]: Balance register, the upgrade rows. `docs/reference/balance.md`
