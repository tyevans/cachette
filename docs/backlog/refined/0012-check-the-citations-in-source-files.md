---
id: 0012
title: Check the citations in source files, not only in records
status: refined
created: 2026-08-30
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

A record split renumbered the decisions, and 81 citations in the crate, the
Python package, the gates and the rules kept pointing at the old numbers.
Nothing failed. The record check reads the records only, and a comment is not
compiled.[^1]

A reused number is worse than a deleted path. The reader follows the citation,
reaches a real record, and gets the wrong claim with the authority of a
record.

## Impact review

**Governed by.** No record governs a check script. This item adds a gate, not
a constraint. The record scope rule says where a citation may point, and the
documentation rule says a reference lives in a footnote.[^2] [^3]

**Changes.** None.

**Creates.** None. A gate that enforces an existing rule needs no record: a
future contributor could not reasonably choose otherwise, and the reasoning is
visible in the script.

**Blockers.** None.

**Precedent.** FND-040 records the dangling citations and their cause. FND-039
and the recurring-defect rule record the same shape twice before: one fact
declared in two places, with nothing that fails when the copies disagree.[^1]
[^4]

**Serves.** No product record. This is repository hygiene.

## Done when

- The check reads every source file, script, workflow, and rule, not only the
  records.
- It fails when a citation names an `ADR-NNNN` that no record and no registry
  row has.
- It fails when a citation names `ADR-NNNN Dn` and record `NNNN` exists
  without a decision `Dn`.
- It fails when a footnote path in a source file does not resolve on disk.
- A citation of a registry row that has no file passes, because that is the
  documented way to cite a reserved number.
- The check runs inside the existing record gate, so one command covers both
  corpora.
- A test proves the check can fail: a fixture with a dangling citation makes
  it exit non-zero.
- `just check` runs green.

## Outcome

Filled in on completion.

## References

[^1]: Findings register, FND-040. `docs/FINDINGS.md`
[^2]: Decision Record Scope. `.claude/rules/adr-scope.md`
[^3]: Documentation Rules. `.claude/rules/documentation.md`
[^4]: Recurring Defect Shapes. `.claude/rules/recurring-defects.md`
