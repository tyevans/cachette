---
id: 0108
title: Let a unit observe the tiles around it
status: proposed
created: 2026-08-31
implements: []
changes: []
creates: []
serves: [PRD-0001]
blocked-by: [BLK-001]
---

## Why

A faction sees a tile when one of its own units observes that tile.[^1] A
unit therefore needs a rule that says which tiles it observes.

The engine has no such rule. A unit holds a position and a faction, and
nothing more. Until a unit observes something, the storage that item 0107
decides has nothing to fill.

This item builds the observation pass: each unit marks the tiles it observes,
the marks combine into what the faction observes this tick, and the result is
identical at every thread count.

## What is missing before this is refined

- **The storage decision comes first.** Item 0107 decides how a faction
  holds what it observed. This item writes into that storage, so it cannot
  be refined before that record exists.[^2]
- **The tile scale is open.** The area one unit observes depends on what one
  tile represents, so the radius stays a parameter.[^3]
- **The ordering rule is not chosen.** Many units mark the same tile in one
  tick. The combination must be exact under any order, and the review must
  say what fixes that order.[^4]

## Done when

Not yet stated. The item is not refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Product record PRD-0001. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
[^2]: Backlog item 0107. `docs/backlog/proposed/0107-decide-how-a-faction-stores-what-it-observes.md`
[^3]: Blockers register, BLK-001. `docs/BLOCKERS.md`
[^4]: Recurring Defect Shapes, shape 4. `.claude/rules/recurring-defects.md`
