---
id: 0236
title: Repair every record that calls BLK-007 open when the benchmark lands
status: complete
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**A benchmark merged, and about ninety documents said that no measurement
existed.** BLK-007 narrowed on 3 September 2026 and did not close.[^1] Every
document that stated the blocker in its own words became false in the general
case on that day. Nothing failed, because a document is prose.

**The rules name this shape and record that it has happened twice before.** A
document written parametrically under a blocker is correct when it is written
and false the moment the blocker moves.[^2] [^3]

**The scale was the problem.** Sentences of the form "no measurement exists on
the target platform, and every cost figure in this project is derived" appear in
accepted records, in draft records, in product records, in the decisions
register, in the reference registers, in the research notes, in open and
completed backlog items, in reviews, and in doc comments in the engine. Each was
correct when it was written. Each is a separate site.

This item was written before the merge rather than after it, so that the sweep
was ready when the blocker moved.

## What the work does

Find every site, decide what each should say, and change the ones a sweep is
free to change.

**The two kinds of site take different repairs.** A document that cited the
blocker for the *state* of measurement is now wrong, and the repair deletes the
restatement and keeps the citation. A document that expressed a *value*
parametrically under the blocker is still right, because the blocker narrowed
and did not close, and that site needs no repair at all.

**The repair form is the one the earlier finding names.** A document states a
register by citation and never in its own words.[^4] Replacing a stale sentence
with a sentence that is true today would only set the next sweep.

## Impact review

**Governed by.** No decision record governs a sweep over prose. Three rules do.
The definition of done states the rule this item executes: when a blocker
changes, search the tree for its number and repair every document that
misstates it.[^2] The documentation rule states the form a repaired sentence
takes, which is a citation and not a restatement.[^5] The registry states which
records a sweep may not touch.[^6]

**Changes.** No record changes its claim. Every edit removes a statement about
the state of a register and leaves the citation that already pointed at it.

**Creates.** No decision record. The question this work ran into is already open
in the decisions register, and a second row for it would be a second declaration
site.[^7]

**Blockers.** BLK-007 narrowed and did not close, so the sweep repairs a general
claim and leaves a specific one.[^1] Three classes of figure are still derived:
those in the research reports, those for a world that holds settlements or
characters, and those for a stage inside a step. The measurement register states
what two runs covered.[^8]

**Precedent.** FND-223 records the scale of the spread and names the defence.[^4]
FND-042 records the two earlier instances.[^3]

## What the sweep did not touch, and why

**The accepted decision records.** The freeze governs them. Rewording the
sentence that carries a footnote marker is an amendment, and each of these
records has dependents, so the retcon window is shut.[^6] DEC-096 is the open
row that decides the repair form, and a reviewer owns it.[^7] A separate item
holds the work.[^9]

**The completed backlog items and the reviews.** Each is a record of one moment
and each was correct when it was written. Repairing them would record a history
that did not happen.

**The doc comments in the engine.** They sat outside the scope this item was
given. The item that follows this one covers them.[^9]

**The project orientation and the two other reference registers.** They were
repaired before this item ran.

## Done when

- Every document that guides work today states the blocker by citation and not
  in its own words.
- A whole-tree search for the phrase family returns only the registers that own
  the statement, the frozen records, and the records of a moment.
- The search command is in the commit body, with the count of each class.
- The absence of a mechanical guard is tested rather than assumed, and the
  result is in the findings register.[^10]

## Outcome

The sweep repaired every document it was free to touch: the product records, the
draft decision records, the open backlog items, the decisions register, the
research notes and the target reference register. It left the accepted decision
records, the completed backlog items, the reviews and the engine comments, and
each exclusion is stated above with its reason.

**Two things came out of it that this item did not predict.**

The target reference register held a merge defect. Its status section carried
two spliced versions of one paragraph, and the surviving half cited the wrong
footnote. Both are repaired.

**No check sees the defect, and that was tested rather than assumed.** One
repaired sentence was put back in its stale form, and all eight document checks
passed. A finding holds the case and a refined item holds the check that would
catch it.[^10] [^11]

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Definition of Done, section 4. `.claude/rules/definition-of-done.md`
[^3]: Findings register, FND-042. `docs/FINDINGS.md`
[^4]: Findings register, FND-223. `docs/FINDINGS.md`
[^5]: Documentation Rules, section 3. `.claude/rules/documentation.md`
[^6]: ADR Registry, the retcon window and the citation rule. `docs/adrs/REGISTRY.md`
[^7]: Decisions register, DEC-096. `docs/DECISIONS.md`
[^8]: Target platform costs. `docs/reference/graviton-costs.md`
[^9]: Backlog item 0243. `docs/backlog/proposed/0243-repair-the-accepted-records-that-state-the-missing-measurement.md`
[^10]: Findings register, FND-258. `docs/FINDINGS.md`
[^11]: Backlog item 0242. `docs/backlog/refined/0242-fail-a-check-when-a-document-states-a-register-in-its-own-words.md`
