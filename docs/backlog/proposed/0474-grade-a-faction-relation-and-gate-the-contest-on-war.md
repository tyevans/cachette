---
id: 0474
title: Grade a faction relation and gate the contest on war
status: proposed
created: 2026-09-05
implements: [ADR-0146, ADR-0053 D7, ADR-0121 D1, ADR-0003 D1, ADR-0001 D4]
changes: []
creates: []
serves: [PRD-0049]
blocked-by: [BLK-007]
---

## Why

**Two adjacent factions always fight.** The contest pass fires wherever two
factions meet, so no faction can be at peace with a neighbour. This item is
pass 3 of the living world game layer.[^1]

A dense matrix of signed integers covers the ordered faction pairs. The entry
for (A, B) is what A feels toward B. The matrix is simulated state and enters
the state hash. Four bands cover the integer range: alliance, peace, tension
and war. The band edges are rows in the balance register, and no band name
appears in code.[^2]

Four passes read the relation. The contest fires only when at least one side
is in the war band toward the other. A unit converts only when the leading
faction is in a permitted band toward the faction of the unit. The engine
refuses an offer when either side is in the war band. A unit may not enter
ground that another faction holds when the holder is below a stated band.

Five causes move the relation by an integer step, and a drift moves it one
step toward peace on a period and a phase. The verb `move_relation(speaker,
other, step)` moves one entry, and it refuses when the speaker holds no unit
with command reach. A crossing of the war edge writes one plain-data event.

**This pass touches `fn step` in `world.rs`. Only one worker may hold it at a
time.** It waits for pass 1 to merge.

## What is missing before this is refined

- The impact review, decision by decision. ADR-0146 is being written beside
  this item, and the registry holds its status.[^3] ADR-0053 D7 states that a
  relation is one mask row per faction. The review must say whether the graded
  matrix supersedes D7 or sits beside it, because a mask row and a signed
  integer answer different questions.
- Whether the contest gate changes ADR-0121 D1, which states that contact is
  adjacency. The gate adds a condition to the resolution and not to the
  contact, and the review must say so or supersede.
- Which of the five causes this pass wires. The contract causes need pass 6,
  and the storm cause needs pass 5. The review must name which causes are
  stored and unread after this pass.
- The per-field tests for the drift schedule and for the event, and the
  extreme that the fixture reaches: an entry at each band edge, and an entry at
  the integer limit so the step clamps.
- The "Done when" statements, in the shape of item 0472: the two determinism
  tests at 1, 2 and 12 threads, each keyed draw with a per-field test, the
  defect put back and the test red, and the type stub edited by hand in the
  same commit as `move_relation`.[^4]

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Design: the living world game layer, sections 3 and 13. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^2]: Balance register. `docs/reference/balance.md`
[^3]: ADR Registry. `docs/adrs/REGISTRY.md`
[^4]: Findings register, FND-320. `docs/FINDINGS.md`
