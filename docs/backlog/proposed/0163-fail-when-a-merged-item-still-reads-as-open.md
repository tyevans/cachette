---
id: 0163
title: Fail when a merged item still reads as open
status: proposed
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

An item can be finished and still read as open. Nothing fails.

The work that item 0152 asked for merged into the trunk. The item stayed in
`proposed/`, its Outcome stayed empty, and its row stayed in the priority
index. The index says to take the highest item a worker can start, so the next
worker would have taken work that was already done.[^1]

No check found it. A worker building against the same subsystem found it by
reading the item and noticing that its statement of the problem no longer
matched the code.

Every check the project runs today compares one document against another
document, or a document against the file tree. The gate suite proves the code
passes its tests and proves the registers agree with each other. **No check
compares what merged against the state of the item that asked for it.** That
gap is the whole of this item.

The shape is the one the recurring defect rule names first: one fact in two
places, with nothing failing when the copies disagree.[^2] Here the two places
are the tree and the item, and the fact is whether the work is done.

## What must be answered before this is refined

**Whether a check can know this at all.** An item states its acceptance in
prose under `Done when`. A script cannot read prose and decide whether the tree
satisfies it. So the check cannot be "is this item done"; it must be a weaker
question a script can answer. Decide which.

Two candidates, and neither is obviously right:

- **The item names files, and those files changed.** Cheap, and wrong often
  enough to train a reader to ignore it.
- **A commit body names the item, and the item is not in `complete/`.** This
  ties the signal to the commit message, which is the one document that never
  decays.[^3] It fails when a worker forgets to name the item.

**Whether the answer is a check at all.** A check that reports a question
rather than a failure may be the honest form, in the way the record check
reports an uncited record without failing.[^4] Decide whether this fails the
gate or prints a question.

**What it costs to be wrong in each direction.** A false failure trains
everyone to ignore a red gate, which is worse than the defect. A missed one
leaves the current state, which the project survived. Weigh them.

## Impact review

Not done. The item stays in `proposed/` until it is.

## Done when

Filled in when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Backlog priority index. `docs/backlog/PRIORITY.md`
[^2]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^3]: Commit Message Rules. `.claude/rules/commits.md`
[^4]: Decision Record Scope, section 8. `.claude/rules/adr-scope.md`
