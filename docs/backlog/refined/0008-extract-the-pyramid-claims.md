---
id: 0008
title: Write the the pyramid records, 0022 to 0029
status: refined
created: 2026-08-30
implements: []
changes: []
creates: [ADR-0022 to ADR-0029]
serves: []
blocked-by: []
---

## Why

The registry allocates rows 0022 to 0029 for the pyramid: level 0 as truth, the aggregation invariant, the field typing, the two pyramids, and the query index.

The material for these claims was drafted as one record and removed when the
registry was re-derived from claims rather than topics. The reasoning is in
the history and is not lost.[^1]

## Impact review

**Governed by.** The record scope rule. Each row must pass the
three-condition test before its file is written. A row that fails the test is
dropped and its number is retired, rather than written as a weak record.

**Changes.** None. No record exists for these numbers yet.

**Creates.** The records for rows 0022 to 0029, minus any row that fails the test.

**Blockers.** None for the extraction itself. A claim that depends on an open
blocker states its value parametrically and cites the blocker.

**Precedent.** FND-033 records that record length predicts churn. FND-034
records that a claim title predicts a more stable record than a topic title.
Both are why the original record was split.

## Done when

- Every row in 0022 to 0029 either has a file or has been dropped with its number
  retired and the reason recorded.
- Each record holds one claim a reviewer could reject on its own.
- Each record has the context, decision, consequences and references
  sections, and cites its evidence in footnotes.
- No record holds a figure, a version pin, a count, or a module arrangement.
- The record check passes.

## Outcome

Filled in on completion.

## References

[^1]: The removed drafts, at commit `4937cd2`. Read with `git show 4937cd2:docs/adrs/draft/`.
