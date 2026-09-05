---
id: 0481
title: Run a fixed seed set to game end and check four balance statements
status: proposed
created: 2026-09-05
implements: [ADR-0148, ADR-0001 D4]
changes: []
creates: []
serves: [PRD-0053]
blocked-by: [BLK-007]
---

## Why

**Nothing says whether the game is fair.** One seed may hand every game to one
seat or to one path, and no run would say so. This item is pass 10 of the
living world game layer.[^1]

A recipe `just balance` runs a fixed seed set to game end. It checks four
statements against thresholds in the balance register.[^2]

1. No win path wins more than its share of the seeds.
2. No seat wins more than its share of the seeds.
3. Every game ends before the tick limit in more than a stated share of the
   seeds.
4. Every subsystem count is nonzero in every seed.

The harness is long, so it is not a merge gate. It runs in the slow test
recipe, on the schedule that recipe runs, and before any commit that changes
a balance value. Its output names the seed set and every failing seed.

**This pass sets the values.** Every row of the balance register is unset until
this pass measures it. The pass writes each value and its derivation into the
register, in the same commit as the measurement.

**This pass does not touch `fn step` in `world.rs`.** It waits for pass 8,
because it needs every win path.

## What is missing before this is refined

- The impact review. The harness reads the game end record of ADR-0148 and
  proves that the two determinism tests hold across a whole game, which FND-174
  records that neither test defends today.[^3]
- How the seed set is chosen and where it lives. A seed set is a fixture, and a
  fixture that models the typical case hides the defect.[^4] The review must
  say which extreme each seed reaches.
- What the harness does when a path cannot fire. Renown is behind BLK-150, so
  statement 1 must state a share for a path that never wins.
- Whether the balance shares are costs under BLK-007.[^5] They are not run
  times, but the tick count to game end is one, and the review must say which
  rows the blocker governs.
- The proof that the harness can fail: a seed set chosen so that one seat wins
  every game, and the recipe red.
- The "Done when" statements, in the shape of item 0472: the two determinism
  tests at 1, 2 and 12 threads on one full game, the defect put back and the
  recipe red, and the type stub edited by hand in the same commit as any new
  reader.[^6]

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Design: the living world game layer, sections 10.2 and 13. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^2]: Balance register. `docs/reference/balance.md`
[^3]: Findings register, FND-174. `docs/FINDINGS.md`
[^4]: Testing Rules, section 2a. `.agents/rules/testing.md`
[^5]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^6]: Findings register, FND-320. `docs/FINDINGS.md`
