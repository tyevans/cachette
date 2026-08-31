---
id: 0008
title: Write the pyramid records, 0022 to 0029
status: complete
created: 2026-08-30
implements: []
changes: []
creates: [ADR-0022, ADR-0023, ADR-0024]
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

## Outcome

Three of the eight rows are written. Five stay reserved, and that is the
answer the scope rule gives rather than an unfinished job.

**Written.** ADR-0022 states that level 0 is the only truth and that every
level above it is a pure function of it. ADR-0023 states that a summary field
combines exactly and commutatively, that only a field with an inverse may be
updated incrementally, and that the equality between two levels is a test.
ADR-0024 states that every summary field declares itself extensive or
intensive, and that an intensive field is stored as the extensive parts it is
divided from.

These three are what an implementation of level 1 must satisfy before it
writes a line. Each states a constraint a reviewer can find a violation of,
each names a choice a contributor could reasonably make otherwise, and none of
the three is visible in code that does not exist yet.

**Reserved.** Rows 0025 to 0029 name the two update paths and their threshold,
the second pyramid, the query index, the descent cost model, and operator
commutation.

Each fails the third condition of the scope rule today, and one fails the
second. The two update paths are a cost decision whose threshold nobody has
measured, and BLK-007 governs every cost figure in this project. The query
index and the descent model describe a subsystem that no product record asks
for yet. A record written now would state an intent as a fact, which the scope
rule names as a category that must not go in a record.[^2]

**Do not read a reserved row as a gap.** The registry holds the number and the
claim, and the work that needs the claim writes the file. That is the
documented way to cite a reserved number, and the citation check passes on
one.

**No count of the written records appears in a record.** This item holds it,
and an item is fixed to one moment in the way a record is not.

## References

[^1]: The commit that removed the drafts. `git log`
[^2]: Decision Record Scope, sections 1 and 4.6. `.claude/rules/adr-scope.md`
