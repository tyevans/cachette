---
id: 0333
title: Fail when a doc comment states a number the code owns
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0021]
blocked-by: []
---

## Why

**A reference that answers a reader states numbers, and every number it states
is a second declaration site.** The published reference now gives four
constructor defaults, a faction ceiling, a camera tile size and its range, a
census radius default and a census radius ceiling, and a commodity count. The
code owns all of them. Nothing fails when a doc comment and the code disagree,
which is the defect shape this project names first in its own rule.[^1]

**A test now pins each of those numbers, and the test is a hand-written list.**
It reads each value through the public interface and fails when the code moves
one. It does not fail when somebody adds a number to a doc comment and adds no
row to the list, so the list itself decays.

The work is to derive the list rather than write it: read the numeric literals
of the published prose, and require each one to appear in a pinning test or in
an allow list with a reason. The reference check already derives its expectation
from the imported module rather than holding a copy, and it is the model.[^2]

## References

[^1]: Recurring Defect Shapes, shape 1, redundant declaration sites. `.claude/rules/recurring-defects.md`
[^2]: The reference check script. `scripts/check_reference.py`
