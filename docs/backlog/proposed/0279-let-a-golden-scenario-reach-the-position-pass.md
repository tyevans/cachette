---
id: 0279
title: Let a golden scenario reach the position pass
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**The position table folds into the state hash and no golden scenario reaches
the pass that writes it.** A change to who works where therefore moves no
golden file, and the guard that exists to notice a changed simulation notices
nothing.

This was measured rather than assumed. The seating pass was added, it seats
sixteen units in the world the demonstration builds, and every golden file
matched without being recorded again. Two reasons combine, and each is enough
on its own.

- The settling schedule has a default period of ten frames. The founding
  scenario is a wide scenario and runs eight, so it never reaches a settling
  frame.
- The scenarios that run long enough spawn their units directly rather than
  founding them, so no unit names a home site and no site has an applicant.

The golden state test is one of the two tests the project cannot lose.[^1] A
scenario set that misses a whole pass is the same weakness that item 0179
records for the build pass, seen in a second subsystem.[^2]

## What is missing before this is refined

- The impact review.
- **Which of the two reasons to fix.** Founding a scenario gives units homes
  and covers the pass at the default schedule if the scenario runs long
  enough. Shortening the period covers it sooner and states a schedule that no
  other scenario states. The item must choose one and say why, rather than
  doing both.
- **What the added frames cost.** The gate suite has a development budget and
  the golden test collects most of its cost from the wide scenarios.[^3] A
  scenario that runs longer is paid for by every worker on every run.
- Whether this is one scenario or a change to an existing one. A new row costs
  a new golden file; a changed row moves an existing one, and a moved golden
  file must be read before it is committed.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^2]: Backlog item 0179, give a golden scenario a build. `docs/backlog/proposed/0179-give-a-golden-scenario-a-build.md`
[^3]: Development budgets, the gate suite budget. `docs/reference/development-budgets.md`
