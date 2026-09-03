---
id: 0227
title: Ask the merge defect questions before the commit
status: complete
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

A merge conflict in a register is resolved by choosing between two sides.
Each side is a correct file. The merged result is not, because it answers a
question that neither side was asked.

Four defects come out of that shape, and one dispatcher produced all four by
hand in a single day: a moved file leaving a stale path, a footnote label
defined twice, two register entries carrying one number, and a next-number
line behind its own entries.

Each was caught. Each was caught by the gate, minutes later, after the commit
existed. Each is a one-second question.

## What the work did

1. A check that asks those questions of the staged change.
2. A pre-commit hook that runs it, versioned in the repository.
3. A gate recipe that runs the same check over the branch.

## Impact review

**Governed by.** No decision record. The four rules are checks, not
constraints, and the constraint each one serves is already recorded elsewhere.

**Changes.** No record.

**Creates.** No record. A future contributor could not reasonably choose to
let a merge defect through, so there is no decision here.[^1] The one open
choice is how a clone installs the hook, and a register row holds it.[^2]

**Blockers.** None.

**Serves.** No product record. This is a gate.

**Precedent.** The check imports the two existing checks rather than
restating their rules, because one value declared twice is the shape this
project keeps recording.[^3]

## What fails if somebody changes it back

- A fixture stages each defect separately and the check fails on each.
- A fixture builds two branches that are each clean, merges them so that a
  register conflicts, resolves the conflict by keeping both sides, and the
  hook refuses the merge commit.
- Removing the `merge-defects` recipe from the gate leaves the hook as the
  only guard, and the hook is bypassable.

Put each defect back separately and watch the check fail before claiming it
covers the case.[^4]

## Done when

- The check finds all four in a staged change.
- The check works during a merge, where the other branch's rename appears in
  the staged change against the first parent.
- The check runs in about a second, and runs no search when nothing moved.
- The hook is versioned and one recipe installs it.
- The gate runs the same check over the branch.
- The whole check command runs green.

## Outcome

**Done on 2 September 2026.**

**Every rule but one was already checked, and the work did not restate any of
them.** The register check already failed on a duplicate number and on a stale
next-number line. The footnote check already failed on a duplicated label. The
priority check already failed on an index row listed twice. The new check
imports all three and calls them. Only the moved-path rule is new, because no
existing check ties a move to the citations of the path it moved from. FND-239
records that the premise of the task was wrong in an instructive way: the gap
was latency, not coverage.[^5]

**A fifth defect of the same shape appeared during the work, and it was not
arranged.** Merging the trunk into this branch left four rows of the records
priority index listed twice, because the auto-merge kept both sides of a table
that had been rewritten on one side. The priority check caught it, which is
the thesis of FND-239 confirmed inside an hour. The check now asks that
question too, by calling the priority check rather than restating it.

**The merge case is proven, not asserted.** A fixture builds two branches that
are each clean on their own, moves a cited file on one, writes a new citation
of the old path on the other, and conflicts them on a register. Resolving the
conflict by keeping both sides produces two defects, and the hook refuses the
merge commit while naming both. Correcting both lets the same merge commit.

**The check has had one live run on real work.** Merging thirty commits of
trunk into this branch carried four moved files from other branches. The check
read the staged merge and found no stale path, so those branches had updated
their citations.

**Cost on an Intel Core i7-1260P:** 1.28 s with nothing moved, 1.46 s with a
heavily cited file moved. About 0.9 s of that is the Python interpreter
starting, which every check in the scripts directory pays and which running
without site imports does not reduce. The work itself is about 0.4 s. **These
figures are measured on a development machine and say nothing about the target
platform.**[^6]

**The hook is not installed by this work.** Worktrees share the repository
config, so arming it would arm it in every live worktree of the clone. DEC-094
holds the question and recommends leaving it manual until a defect reaches the
gate that the hook would have caught.

**What is not covered.** Git does not run the pre-commit hook for a merge that
applies cleanly and commits itself. The gate recipe covers that case, and the
hook covers the hand-resolved merge, which is the one that produced all four
defects.

**There is no allow-list**, so a document that names a moved path on purpose
will fail the check. That has not happened yet, and an escape written before a
real instance would be a capability nobody invokes.[^3]

## References

[^1]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^2]: Decisions register, DEC-094. `docs/DECISIONS.md`
[^3]: Recurring defect shapes, shapes 1 and 3. `.claude/rules/recurring-defects.md`
[^4]: Testing Rules, section 2a. `.claude/rules/testing.md`
[^5]: Findings register, FND-239. `docs/FINDINGS.md`
[^6]: Blockers register, BLK-007. `docs/BLOCKERS.md`
