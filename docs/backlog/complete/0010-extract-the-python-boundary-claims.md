---
id: 0010
title: Write the the Python boundary records, 0040 to 0047
status: complete
created: 2026-08-30
implements: []
changes: []
creates: [ADR-0041, ADR-0042, ADR-0044, ADR-0046, ADR-0047]
serves: []
blocked-by: []
---

## Why

The registry allocates eight rows for the Python boundary: the control plane, the crate split, the interpreter, the tier rule, and view safety.

The material for these claims was drafted as one record and removed when the
registry was re-derived from claims rather than topics. The reasoning is in
the history and is not lost.[^1]

## Impact review

**Governed by.** The record scope rule. Each row must pass the
three-condition test before its file is written. A row that fails the test is
dropped and its number is retired, rather than written as a weak record.

**Changes.** None. No record exists for these numbers yet.

**Creates.** The records for rows 0040 to 0047, minus any row that fails the test.

**Blockers.** None for the extraction itself. A claim that depends on an open
blocker states its value parametrically and cites the blocker.

**Precedent.** FND-033 records that record length predicts churn. FND-034
records that a claim title predicts a more stable record than a topic title.
Both are why the original record was split.

## Done when

- Every row in 0040 to 0047 either has a file or has been dropped with its number
  retired and the reason recorded.
- Each record holds one claim a reviewer could reject on its own.
- Each record has the context, decision, consequences and references
  sections, and cites its evidence in footnotes.
- No record holds a figure, a version pin, a count, or a module arrangement.
- The record check passes.

## Outcome

**Five records were written, two rows already had files, and one row was
retired.**

Each row was read against the three-condition test of the record scope
rule.[^2] The material behind the rows is the draft that the registry
re-derivation removed, and it was read in full before the test was applied.[^1]

**Already written.** The control plane rule and the tier rule were drafted
before this item was taken. Neither was changed. Both moved to the review queue
of the record index, where a written record belongs.

**Written.** The crate split, because a gate proves it, both build manifests
and both crate roots cite the number, and it converts the rule against a
mid-step Python callback into a compile error. The interpreter release, because
the bindings implement it and because a frame being a function of what was
fixed before it began is a determinism property that no other record states.
What copies, because seven files cite the number and every read across the
boundary copies today. Typed errors, because the bindings implement the
hierarchy and the Python tests drive it. Many worlds, because the bindings
implement it and a test proves that two worlds diverge.

**Retired.** View safety, because every read across the boundary copies. The
three layers guarded a borrow that the engine does not hand out, so the record
would have described a capability nothing invokes.

The registry holds the reason, and the finding holds what the project believed
before the review and what it found.[^3]

**Two gaps this found and did not close.** The source files that cite these
numbers all point their footnote at the registry, because no file existed when
they were written. Those footnotes now name a record they could name directly,
and repointing them is a separate item.[^4] The error hierarchy declares four
exception types that nothing raises, which the record states in its own
consequences, and closing that is a separate item.[^5]

## References

[^1]: The removed drafts, at commit `4937cd2`. Read with `git show 4937cd2:docs/adrs/draft/`.
[^2]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^3]: Findings register, FND-214. `docs/FINDINGS.md`
[^4]: Backlog item 0221. `docs/backlog/proposed/0221-point-a-source-footnote-at-the-record-it-names.md`
[^5]: Backlog item 0222. `docs/backlog/proposed/0222-raise-the-error-types-the-hierarchy-declares.md`
