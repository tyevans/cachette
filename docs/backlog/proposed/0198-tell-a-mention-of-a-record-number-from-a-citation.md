---
id: 0198
title: Tell a mention of a record number from a citation of it
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**The record check reads every mention of a record number as a citation.** It
runs its citation pattern over the whole text of a record. A code span, a
fenced code block and a line of ordinary prose all give the same result. It
then fails when the number names no record and no registry row.

The registry states the opposite rule for a retired number. A retired number
holds no claim, so a document mentions it and never cites it, and the registry
says to write it in a code span for exactly that reason.[^1] The rule and the
check disagree, and the check is the one that runs.

This is not theory. One draft record needed to say which retired number held a
claim of its shape, so that a reader learns why the project stopped holding it.
It wrote `ADR-0057` in a code span, as the registry directs. The check failed
it, and the record was rewritten to name the number nowhere.[^2] A reader of
that record now cannot tell which number went.

The cost is small today, because the project has retired one number. It grows
with every number retired after it, and it falls on the record that most needs
to explain itself.

A citation of a superseded record does not have this problem. A superseded
record keeps its row and its file, so the check resolves it.

## What is missing before this is refined

- The impact review.
- Whether the fix belongs in the check, in the registry rule, or in both. A
  check that skips a code span would also skip a real citation that somebody
  wrote inside one.
- Whether the same shape reaches the footnote definitions. The check reads
  them as citations on purpose, because that is where a record cites another
  record, so a repair that strips exempt material must not strip that section.
- Whether the citation check that runs over the files outside the records
  holds the same defect or the opposite one. It enforces the code span rule,
  so the two checks may already disagree with each other.
- What a reader should see instead. A retired number that no document may name
  is a number a reader cannot look up.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR Registry, the retired numbers. `docs/adrs/REGISTRY.md`
[^2]: Findings register, FND-192. `docs/FINDINGS.md`
