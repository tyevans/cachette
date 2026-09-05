---
id: 0476
title: Let weather harm upgrades, stores, production, units and movement
status: proposed
created: 2026-09-05
implements: [ADR-0145, ADR-0141 D1, ADR-0142 D1, ADR-0142 D3, ADR-0003 D1, ADR-0001 D4]
changes: []
creates: []
serves: [PRD-0048]
blocked-by: [BLK-007, BLK-130]
---

## Why

**Weather moves water and changes nothing else.** A flooded cell looks like a
dry one to every unit, store and upgrade on it. This item is pass 5 of the
living world game layer.[^1]

Five harms join the weather. An upgrade on a cell above the wet mark wears one
step per tick. A site on a flooded cell loses an integer share of its store per
tick, and the rate pass skips it. One bounded keyed draw per flooded cell names
units at full strength that fall. The movement cost on wet ground rises by a
step. A flooded cell is a cell whose ground water is above a second mark,
higher than the wet mark. Every value is a row in the balance register.[^2]

The verb `inflict_weather` gains one refusal. It refuses when the faction holds
no unit whose type has weather reach above zero. The signature does not
change, and the controller calls the same verb.

**This pass touches `fn step` in `world.rs`. Only one worker may hold it at a
time.** It waits for pass 4 to merge, because the wear step it adds writes the
condition that pass 4 creates.

## What is missing before this is refined

- The impact review, decision by decision. ADR-0141 D1 states that a spread
  pass is a gather of exact integer transfers, and a spoilage is a removal and
  not a transfer. The review must say whether that is inside D1 or beside it.
- BLK-130 governs what weather is worth, and it is open.[^3] Every harm rate
  is a value behind it. The review must express each rate as a row and cite the
  blocker in every place a rate is read.
- The key of the unit loss draw is (weather harm system, tick, cell, draw),
  and the rotation that names the units follows the contest. The review must
  name the per-field test for each of the four fields.
- The extreme that the fixture reaches: a cell exactly at the flood mark, a
  cell above it with no units, and a site whose store is one so the share
  rounds to zero.
- The "Done when" statements, in the shape of item 0472: the two determinism
  tests at 1, 2 and 12 threads, each keyed draw with a per-field test, the
  defect put back and the test red, and the type stub edited by hand in the
  same commit as any new reader.[^4]

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Design: the living world game layer, sections 7 and 13. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^2]: Balance register. `docs/reference/balance.md`
[^3]: Blockers register, BLK-130. `docs/BLOCKERS.md`
[^4]: Findings register, FND-320. `docs/FINDINGS.md`
