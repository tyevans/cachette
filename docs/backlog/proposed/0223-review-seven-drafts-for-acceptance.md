---
id: 0223
title: Review seven drafts for acceptance
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**Seven records are written and none binds anything.** A draft is not binding,
so work built on one is built on sand, and review is the bottleneck the record
priority index names first.[^1]

The seven are the control plane rule, the tier rule, the selector tree, the
selector range, the influence basis, the group membership rule, and the descent
walk. Each waits for a reader who did not write it.[^2]

Two of the seven govern the work that other items are waiting on. The selector
tree is the destination the control plane verbs are written toward, and the
tier rule is the enforcement the control plane rule has none of. Neither can
govern anything while it is a draft.

## What this item is

One review file for each record, under `docs/reviews/`, named for this item.
Each states what was read, every objection the reviewer attempted, and a
verdict.

A verdict is not a status change. Accepting a record moves its file, and a move
breaks every citation of the old path, which is a separate cost that a separate
item carries.[^3] A review of a well-cited record records the verdict and says
what the move would cost.

## Done when

- Each of the seven has a review file with a verdict and at least one attempted
  objection.
- The registry status of each record is set, or the review says why it was not.
- The record priority index reflects what moved.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Decision record priority index. `docs/adrs/PRIORITY.md`
[^2]: Reviews index, who may review. `docs/reviews/README.md`
[^3]: Backlog item 0205. `docs/backlog/proposed/0205-stop-a-record-path-from-changing-when-it-is-accepted.md`
