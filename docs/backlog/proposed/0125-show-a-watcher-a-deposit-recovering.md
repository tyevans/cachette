---
id: 0125
title: Show a watcher a deposit recovering
status: proposed
created: 2026-08-31
implements: []
changes: []
creates: []
serves: [PRD-0018]
blocked-by: []
---

## Why

The product record requires that a watcher can see a deposit recover, and can
tell a recovering deposit from a full one.[^1] The engine will hold the state
and the window will not show it. That is the gap a review already recorded
against other work, so the project knows the shape.

**What refining this must answer.** What the watcher sees: a shade of the
resource marker, a count, or both. Whether the window reads the recovered
amount through the same call a gatherer uses, so that the two cannot disagree.
Whether the head-up display reports what the drawing pass read, which the
viewer records already require.

## Impact review

Not done. This item stays in `proposed/` until it is written.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: PRD-0018, a depleted deposit comes back. `docs/product/shaped/prd-0018-a-depleted-deposit-comes-back.md`
