# Budgets and Costs

This document is a **register**. It holds the cost and storage figures for the
project. The figures change. A decision record cites this document; a decision
record does not hold a figure.[^1]

The registry names this document as the place for the byte budget table, the
per-tick cost budgets, and every constant that an unanswered question
governs.[^2]

**Every figure in this project is derived, not measured.** No code exists, so no
benchmark has run. Mark each figure with how it was derived. When a measurement
replaces a derivation, say so in the row and give the commit.

## Status

No figures are recorded yet. The project has no code and no benchmark.

## What belongs here

- Per-tick and per-frame cost budgets.
- The byte budget table, for each entity tier and each pyramid level.
- Memory totals at the target scale of 16.7 million tiles and one million
  units.
- Throughput figures and latency figures, once measured on the target platform.
- A constant that a blocker governs, held here until the blocker closes.[^3]

## What does not belong here

- A structural constant of the target platform, such as the cache line size.
  That is a property of the platform the project chose. It belongs in the
  record that chose the platform.
- A decision. A budget is an input to a decision, not a decision.

## Figures still held in a record

Four draft records still hold derived cost figures in their bodies. They must
move here when those records are next revised. The record check carries the
list, and the check fails when an entry in that list matches nothing, so the
list cannot go stale.[^4]

| Record | Kind of figure |
|---|---|
| ADR-0003 | Two cache hit rate percentages |
| ADR-0005 | Allocation and cache miss costs for each frame, and the frame budget |
| ADR-0006 | Boundary call costs, and two percentage splits |

Moving a figure here is not a free edit. An accepted record does not change
except in status.[^2] Move a figure as part of the change that supersedes the
record, or while the record is still a draft.

## Format for a row

Give the name, the value, the unit, how it was derived, and the date. Give the
target platform for any figure that depends on the hardware. Cite the source in
a footnote.

## References

[^1]: Decision Record Scope. `.claude/rules/adr-scope.md`
[^2]: ADR Registry. `docs/adrs/REGISTRY.md`
[^3]: Blockers register. `docs/BLOCKERS.md`
[^4]: The record check baseline. `scripts/adr-volatile-baseline.txt`
