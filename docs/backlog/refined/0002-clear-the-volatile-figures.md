---
id: 0002
title: Remove the volatile figures from the six drafts
status: refined
created: 2026-08-30
implements: []
changes: [ADR-0001, ADR-0002, ADR-0003, ADR-0004, ADR-0005, ADR-0006]
creates: []
serves: []
blocked-by: [BLK-007]
---

## Why

The six drafts hold fifteen figures that the scope rule forbids a record to
hold. They were baselined rather than removed, because the drafts were in
review elsewhere at the time. The baseline is a holding action, not a fix.

## Impact review

**Governed by.** The record scope rule. A record must not hold a value that
a measurement can change.

**Changes.** The six drafts. Each figure either moves to the reference
budgets and is cited, or is removed as an unmeasured guess.

**Creates.** None.

**Blockers.** BLK-007 governs this. No measurement exists on the target
platform, so a figure that moves to the budgets is marked derived and not
measured.

**Precedent.** FND-033 records that record length predicts churn, and a
figure-bearing record churns 1.67 times the mean.

## Done when

- No entry remains in the volatile figure baseline.
- Every figure that survives sits in the reference budgets, marked derived.
- The record check passes with an empty baseline.

## Outcome

Filled in on completion.
