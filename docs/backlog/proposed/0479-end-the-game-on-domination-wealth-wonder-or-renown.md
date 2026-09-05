---
id: 0479
title: End the game on domination, wealth, wonder or renown
status: proposed
created: 2026-09-05
implements: [ADR-0148, ADR-0002 D3, ADR-0001 D4]
changes: []
creates: []
serves: [PRD-0048]
blocked-by: [BLK-007, BLK-150]
---

## Why

**Only one win path exists after pass 1, and it fires only at the tick
limit.** A faction that removes every rival keeps playing to the limit. This
item is pass 8 of the living world game layer.[^1]

Three readers join the territory reader. Domination fires when one faction
holds every seat, or when every other faction has no units. Wealth or wonder
fires when a faction stock total reaches a target, or when a wonder upgrade
completes. Renown fires when a character of the faction reaches a renown
target. The controller stage checks the readers in table order before it
evaluates. The stock total sums the stores of the own sites in a 64-bit
accumulator.

Two upgrade kinds join the wall. The wonder has large work, and its completion
fires the wealth-or-wonder path. The store raises the store capacity of the
site on its tile, and a flood spoils it. The work of each kind is a row in the
balance register.[^2]

**This pass does not touch `fn step` in `world.rs`.** It may run beside passes
6 and 7 once pass 5 merges.

## What is missing before this is refined

- The impact review, decision by decision. ADR-0148 is being written beside
  this item.[^3] The review must say how the domination reader counts units per
  faction without a pass over the population.
- BLK-150 governs renown, and it is open.[^4] No pass writes the renown
  column, so the renown reader reads a value that is always zero. The review
  must say whether the reader ships inert behind the blocker, and how the
  balance harness then treats a path that cannot fire.
- Whether the store capacity raise contradicts a record on site capacity, and
  whether item 0348 on the upgrade catalogue lands first.
- The extreme that the fixture reaches: a stock total that overflows a 32-bit
  accumulator, so the 64-bit width is proven, and two readers that fire on one
  tick, so the table order is proven.
- The "Done when" statements, in the shape of item 0472: the two determinism
  tests at 1, 2 and 12 threads, the defect put back and the test red, and the
  type stub edited by hand in the same commit as any new reader.[^5]

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Design: the living world game layer, sections 5, 6.4 and 13. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^2]: Balance register. `docs/reference/balance.md`
[^3]: ADR Registry. `docs/adrs/REGISTRY.md`
[^4]: Blockers register, BLK-150. `docs/BLOCKERS.md`
[^5]: Findings register, FND-320. `docs/FINDINGS.md`
