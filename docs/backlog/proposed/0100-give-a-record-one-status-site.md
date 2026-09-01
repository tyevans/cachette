---
id: 0100
title: Give a record one status site
status: proposed
created: 2026-08-31
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

The registry allocates a record's number and holds its status, and it says so.
It is meant to be the only document that holds it.

Most record files carry a `Status:` line as well. Five do not, and those five
are the most recently written. So the project has two shapes of record, one
that states its status twice and one that states it once, and nothing fails
when the two copies disagree.

This is the first shape in the recurring defect list: one value declared in
more than one place, with nothing that fails on disagreement.[^1] The register
already holds a related instance, where a citation carried a status and the
status went stale.[^2] The acceptance of fourteen records showed the cost: the
registry rows changed in one edit, and the status line inside each file had to
be swept separately.

## What the work does

Decide which site wins, then remove the other.

The registry is the likely answer, because the rule already names it and
because a status is a fact about a record's place in a process rather than a
fact about its content. If the file keeps its line instead, the registry must
derive from it.

Whichever wins, the check must fail when a second site appears.[^3]

## What to be careful of

- **A record's history is in git.** Removing a line from twenty-four accepted
  records is a large diff that changes no decision. Say that in the commit body.
- An accepted record does not change except in status, so this edit is inside
  what the retcon window permits. Say so in the commit.[^4]
- Do not add a status line to the five that lack one in order to make them
  uniform. That is the wrong direction, and it doubles the problem.

## Done when

- A record's status has one site.
- The record check fails when a second site appears.
- A whole-tree search finds no record stating a status the registry does not
  hold, and the search command is in the commit body.
- `just check` exits 0.

## References

[^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^2]: Findings register, FND-055. `docs/FINDINGS.md`
[^3]: The record check script. `scripts/check_adrs.py`
[^4]: ADR Registry, the retcon window. `docs/adrs/REGISTRY.md`
