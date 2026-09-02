---
id: 0186
title: Let the engine order a gather
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: [PRD-0007, PRD-0009]
blocked-by: []
---

## Why

**No unit in the demonstration ever gathers.** The gather order is a
control-plane verb, and the engine issues none. The resource module, the
gather ledger, the depletion set and the recovery pass are all correct and all
idle. The option named `forage` is named for an act that never happens.[^1]

The testing rule already names this shape. Ask who is obligated to invoke the
thing: the user of the library, or the engine. If the engine, the test starts
at the engine.[^2] Nothing today starts at the engine.

## What the work does

1. A unit whose option is `forage`, and whose tile holds the resource, gathers
   there. The engine issues the order.
2. The gather resolve, the depletion and the recovery then run in the
   demonstration, on the path the step already holds.

**This closes a feedback loop.** Food falls where the crowd stands. The cell
summary falls with it. The exit direction of item 0185 turns the crowd away.
Without this item, the flow field of 0185 produces one rush in one
direction.[^3]

## What is missing before this is refined

- The impact review.
- Whether the engine sets the order or replaces it, and what happens to an
  order a caller set from the control plane. Two writers of one column is the
  shape the project records as a recurring defect.[^4]
- Which resource kind a `forage` option names, when the option set holds one
  forage row and the world holds three kinds.
- Whether a unit stops gathering when its option changes, and which stage
  clears the order.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: What a unit does in a tick, section 3.6. `docs/research/what-a-unit-does-in-a-tick.md`
[^2]: Testing Rules, section 5. `.claude/rules/testing.md`
[^3]: Backlog item 0185, steer a step by the option the unit chose. `docs/backlog/proposed/0185-steer-a-step-by-the-option-the-unit-chose.md`
[^4]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
