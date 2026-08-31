---
id: 0038
title: Fail when a citation calls an accepted record a draft
status: complete
created: 2026-08-31
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

Five citations in the tree describe ADR-0056 as a draft record. The registry
says it is accepted, and the registry is the only place that holds the status
of a record.

The claim is false and nothing fails. A reader who follows the citation
reaches an accepted record and is told by the prose beside it that nothing may
cite it as binding. That inverts the meaning of the record.

The citation check already reads every citation outside the records. It fails
when a citation names a record that does not exist, names a decision that a
record does not define, or names a path that does not resolve. It does not
read what the citation says about the status.

## What the work does

1. The citation check fails when a footnote names a path under the accepted
   directory and calls the record a draft.
2. The five stale citations are repaired.
3. A fixture proves the check can fail.

## Impact review

**Governed by.** The registry states that it is the only place that holds the
status of a record.[^1] The documentation rule states that every reference to
external material goes in a footnote, so a footnote is where the claim lives
and where a check can read it.[^2]

**Why this direction only.** The check reads the false claim, not the missing
one. A citation of a draft that does not say so is a different question: it is
a convention that the project has never held, and about fifty citations would
have to change to adopt it. That is a change to the rule, not a defect, and it
is not this item.

The opposite drift needs no new check. A record that moves from the draft
directory to the accepted one breaks every citation of its old path, and the
existing path check already fails on that.

**Changes.** No record changes. One script gains a rule and one fixture gains
a case.

**Creates.** No record. The three-condition test fails on condition one: there
is no second workable option. A false status claim is a defect, not a choice.

**Blockers.** None.

**Precedent.** The recurring-defect rule names the shape: one fact in two
places with nothing that fails when the copies disagree, and a document that
rots when the tree moves under it.[^3] The rule for that shape says to write a
check that derives one listing from the tree and compares, rather than sweep
by hand.

## Outcome

The check reads a footnote definition, and it fails when that footnote names a
path under the accepted directory and calls the record a draft. The path is
the derivation, so there is no second listing to drift.

The check reads a footnote definition and nothing else. The review that
accepted ADR-0056 states in a table that the record's status was `Draft` on
the day it was reviewed. That is true, it must stay, and the shape of the line
is what separates it from a live claim.

Five citations were repaired, all of ADR-0056. The review that accepted the
record is in the tree, so the sweep that should have followed it never ran,
and the claim was copied into new work twice more before the check found it.
FND-055 records the shape.

## References

[^1]: ADR Registry. `docs/adrs/REGISTRY.md`
[^2]: Documentation Rules, section 3. `.claude/rules/documentation.md`
[^3]: Recurring defect shapes, shapes 1 and 2. `.claude/rules/recurring-defects.md`
