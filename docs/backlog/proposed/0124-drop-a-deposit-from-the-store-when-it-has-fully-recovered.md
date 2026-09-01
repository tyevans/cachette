---
id: 0124
title: Drop a deposit from the store when it has fully recovered
status: proposed
created: 2026-08-31
implements: []
changes: []
creates: []
serves: [PRD-0018]
blocked-by: []
---

## Why

Recovery ages a stored take toward zero.[^1] A take of zero says the same
thing as no take at all, and the product record requires that a watcher cannot
tell a fully recovered deposit from an untouched one.[^2] Two ways to say one
thing is the defect shape this project keeps meeting.

The cost claim of the product record depends on this item. The set of depleted
deposits must shrink as well as grow, or a long run stores one entry for every
tile a unit ever touched, and the cost of recovery drifts toward the tile
count.

**What refining this must answer.** Where the drop happens, so that it happens
at one point in the frame and not at a read. A drop that a read performs makes
reading change the world, and the product record forbids that. Whether the
drop is part of item 0123 or follows it. What the state hash does when an
entry goes, and which golden file that moves.

## Impact review

Not done. This item stays in `proposed/` until it is written.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Backlog item 0123. `docs/backlog/refined/0123-recover-a-depleted-deposit-without-a-pass-over-the-world.md`
[^2]: PRD-0018, a depleted deposit comes back. `docs/product/shaped/prd-0018-a-depleted-deposit-comes-back.md`
