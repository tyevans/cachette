---
id: 0009
title: Write the the log records, 0030 to 0039
status: complete
created: 2026-08-30
implements: []
changes: []
creates: [ADR-0031, ADR-0032]
serves: []
blocked-by: []
---

## Why

The registry allocates ten rows for the log: the rejection of event sourcing, the arenas, what the log holds, the barrier, and the save format.

The material for these claims was drafted as one record and removed when the
registry was re-derived from claims rather than topics. The reasoning is in
the history and is not lost.[^1]

## Impact review

**Governed by.** The record scope rule. Each row must pass the
three-condition test before its file is written. A row that fails the test is
dropped and its number is retired, rather than written as a weak record.

**Changes.** None. No record exists for these numbers yet.

**Creates.** The records for rows 0030 to 0039, minus any row that fails the test.

**Blockers.** None for the extraction itself. A claim that depends on an open
blocker states its value parametrically and cites the blocker.

**Precedent.** FND-033 records that record length predicts churn. FND-034
records that a claim title predicts a more stable record than a topic title.
Both are why the original record was split.

## Done when

- Every row in 0030 to 0039 either has a file or has been dropped with its number
  retired and the reason recorded.
- Each record holds one claim a reviewer could reject on its own.
- Each record has the context, decision, consequences and references
  sections, and cites its evidence in footnotes.
- No record holds a figure, a version pin, a count, or a module arrangement.
- The record check passes.

## Outcome

**Two records were written and eight rows were retired.**

Each row was read against the three-condition test of the record scope
rule.[^2] The material behind the rows is the draft that the registry
re-derivation removed, and it was read in full before the test was applied.[^1]

**Written.** The arenas, because the code holds one array for each event type,
a source file already cites the number, and one enumeration holding every kind
is the implementation a contributor reaches for with nothing in the code to say
why it was refused. What the log holds, because no solver writes an event
today, and a system author has to draw the line between a fact and a derived
value correctly the first time.

**Retired.** The rejection of event sourcing, because it stated the alternative
that the arena record refuses, and the alternative belongs in the record that
refuses it. The barrier concatenation, because two decisions of an accepted
record already forbid a shared output and already fix the combine order. The
command seal, because no command queue exists and the durable half is now a
decision of the record for releasing the interpreter. The rejection
enumeration, because it described a set-valued command layer that does not
exist and disagreed with how the bindings refuse today. The snapshot, because
nothing snapshots the world and no product record asks for one. The transient
log, because its own argument was that adding retention later is cheap, which
fails the second condition. The region aggregate, because it named a structure
this project does not have. The save format, because nothing had chosen
anything.

The registry holds the reason for each retirement, and the finding holds what
the project believed before the review and what it found.[^3]

**A cost this created.** Retiring nine numbers across both items raised the
cost of the open item that carries the record check reading any mention of a
number as a citation.[^4] The row moved up the index for that reason. Three
documents were repaired: an accepted record lost a footnote that named a
retired number, and two historical documents now write the number in a code
span, which is what the registry prescribes.

## References

[^1]: The removed drafts, at commit `4937cd2`. Read with `git show 4937cd2:docs/adrs/draft/`.
[^2]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^3]: Findings register, FND-214. `docs/FINDINGS.md`
[^4]: Backlog item 0198. `docs/backlog/proposed/0198-tell-a-mention-of-a-record-number-from-a-citation.md`
