---
id: 0006
title: Write the platform and value type records, 0008 to 0011
status: complete
created: 2026-08-30
implements: []
changes: []
creates: [ADR-0008, ADR-0009, ADR-0011]
serves: []
blocked-by: []
---

## Why

The registry allocates rows 0008 to 0011 for platform and value types: the target, the memory model, the cache line, and the newtype rule.

The material for these claims was drafted as one record and removed when the
registry was re-derived from claims rather than topics. The reasoning is in
the history and is not lost.[^1]

## Impact review

**Governed by.** The record scope rule. Each row must pass the
three-condition test before its file is written. A row that fails the test is
dropped and its number is retired, rather than written as a weak record.

**Changes.** None. No record exists for these numbers yet.

**Creates.** The records for rows 0008 to 0011, minus any row that fails the test.

**Blockers.** None for the extraction itself. A claim that depends on an open
blocker states its value parametrically and cites the blocker.

**Precedent.** FND-033 records that record length predicts churn. FND-034
records that a claim title predicts a more stable record than a topic title.
Both are why the original record was split.

## Done when

- Every row in 0008 to 0011 either has a file or has been dropped with its number
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

Three of the four rows are written. One stays reserved.

**ADR-0008, the primary target is aarch64.** It states the triple, that every
other platform is a development target, that a figure taken on one is labelled
as such, and that vector code compiles for the baseline rather than selecting
an implementation at run time. The last of those says plainly that no vector
code exists yet: the decision is written so a reviewer can refuse a dispatch
table when the first of it arrives.

**ADR-0009, parallel stages write disjoint outputs.** The engine already leans
on this everywhere and no file held it. Three source files cited the reserved
registry row, which is the documented way to cite a number with no file, and
they now cite the record. Every parallel stage in the engine takes the shape
the record states: the tile update, the movement intents, the admission
segment table, and the level 1 rebuild.

**ADR-0011, every value type is a newtype.** Six newtypes exist and every one
of them is `repr(transparent)`, which is the decision that makes the wrapper
free. The record states that, and states the conversion rule that keeps two
types sharing one inner integer from collapsing back into each other.

**Row 0010, the cache line size is a compile-time constant, stays reserved.**
No such constant exists in the tree. A record for it would state an intent as
a fact, which the scope rule forbids, and it would be a record with no reader.
The work that needs the constant writes the record.

**Two registry titles were shortened to the claim alone.** Row 0008 read "and
NEON is a baseline rather than a dispatch", which is one of the record's four
decisions rather than its claim. Row 0011 read "with a declared size and
alignment", which is the decision that makes the claim affordable rather than
the claim. Neither number changed.

**The dependency order was checked before writing.** ADR-0009 and ADR-0011
both depend on ADR-0008, so ADR-0008 was written first and all three are
drafts together. None of the three is accepted, so nothing cites one as
binding yet.
