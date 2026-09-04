---
id: 0432
title: Decide the lifetime of every log the bindings expose
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

The bindings expose four logs, and each one holds the last step alone. The next
step empties it. Every doc comment says so, and nothing else holds the caller
to it: a caller that steps twice before it reads loses the first step, and no
call fails.

The fallen log made the fourth call site with the same sentence in it. A rule
stated four times in prose, and nowhere in code, is one fact in four places.[^1]

A decision holds the options, with a recommendation to leave the engine as it
is and to put a recorder in the control plane.[^2] This item is the work that
follows whichever option the decision takes.

## What is missing before this can be refined

- Whether a caller needs more than one step. No downstream need is recorded.
- Whether a recorder in the Python package is one object for every log or one
  for each, and what it costs to keep a step of each log at the target scale.
- Which decision record holds the outcome, if the engine changes at all.

## References

[^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^2]: Decisions register, DEC-224. `docs/DECISIONS.md`
