---
id: 0154
title: Fail a gate when a merge conflict marker reaches the tree
status: proposed
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

The decisions register carried an unresolved merge conflict marker in its
reference section, on the main branch, across several commits. A reader found
it while rebasing. No check found it, and the gate suite ran green over it
each time.

That is the point of the item. A gate that passes is evidence that the rules
the gate encodes hold, and nothing more. The register checks read the entries
and the next number. The record checks read the records. Markdown does not
care. The findings register holds this class of mistake.[^3]

A parallel run merges the registers often, so the case will recur.

The search costs nothing. One grep over the tree finds every marker in under a
second, which is why the absence of the check is worth closing rather than
living with.

## What the work does

Add a check that searches the tree for a conflict marker at the start of a line
and fails when it finds one. Run it with the other checks.[^1]

The search must cover the whole tree, not a list of directories somebody
thought of. A marker in a source file, a test fixture or a register is the same
defect.

## What it must not do

It must not skip a directory to make a fixture pass. A fixture that must hold a
marker belongs behind an exception the check names, or the fixture holds the
marker in a form the check does not match.

## Done when

- A conflict marker anywhere in the tree fails a gate.
- The check has a broken fixture that it rejects, in the way the record checks
  do.[^2]

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: The check targets. `justfile`
[^2]: Backlog guide. `docs/backlog/README.md`
[^3]: Findings register, FND-136. `docs/FINDINGS.md`
