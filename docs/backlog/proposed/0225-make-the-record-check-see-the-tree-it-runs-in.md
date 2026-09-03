---
id: 0225
title: Make the record check see the tree it runs in
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

The record check reports a record that no other record and no source file
cites. It reads no source file when it runs inside a worktree, so the note is
a false signal there.

The check walks the tree for Rust and Python files. It skips a path that holds
a part named `.git`, `target` or `worktrees`. A worktree of this project lives
under a directory of that name. Every path below the worktree root therefore
holds the part, and the walk returns nothing. Twelve records carry the note in
a worktree. One of them is cited from two source modules.[^1]

The skip set means "do not descend into a worktree from the main checkout".
Inside a worktree it means "skip everything".

**A check that reads nothing reports the same shape as a check that found
nothing.** That is the failure worth closing. The wrong skip is one line. The
silence is the defect.

## What the work does

1. Make the walk skip a worktree directory only when the walk did not start
   inside one.
2. Make the check report how many source files it read. A run that reads zero
   says so, and a reader can tell an empty tree from a broken filter.
3. Add a test that drives the check from a path holding a skipped part and
   asserts that it still reads the sources.

## Impact review

**Governed by.** No record. The check enforces the mechanical part of the
record scope rule, and that rule states which failures a regular expression
can see.[^2] Nothing there fixes how the walk finds a file.

**Changes.** No record.

**Creates.** No record. A future contributor could not reasonably choose to
leave a check reading nothing, so there is no decision here.[^3]

**Blockers.** None.

**Serves.** No product record. This is a gate defect.

**Precedent.** FND-237 records the finding and its evidence.[^1] The shape is
the one the project already names: a mechanism that passes its own test and
reaches nothing.[^4]

## What fails if somebody changes it back

- A test drives the check from a path that holds a skipped part. It asserts
  that the source scan is not empty. Restoring the skip fails it.
- The check reports its file count, so a reader sees zero rather than
  inferring it.

Put the wrong skip back and watch the test stay green before claiming it
covers the case.[^5]

## Done when

- The check reads the source tree from a worktree and from the main checkout.
- The check states how many source files it read.
- The test above exists and has been proven able to fail.
- The whole check command runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-237. `docs/FINDINGS.md`
[^2]: Decision Record Scope, section 8. `.claude/rules/adr-scope.md`
[^3]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^4]: Recurring defect shapes, shape 3. `.claude/rules/recurring-defects.md`
[^5]: Testing Rules, section 2a. `.claude/rules/testing.md`
