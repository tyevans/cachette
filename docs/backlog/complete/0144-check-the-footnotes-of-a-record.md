---
id: 0144
title: Check the footnotes of a record
status: complete
created: 2026-09-01
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

The documentation rule states two things about footnotes. Number them in the
order they occur in the body, and never repeat one: reuse the marker when one
source supports more than one claim.[^1]

The record check tests neither. A review of four drafts found that three of
them break one rule or both, and the gate passed on all three.[^2] One draft
holds a footnote label with no definition ahead of it in the body order, and two
drafts hold two labels that name one source.

A duplicate footnote is invisible to a reader. It becomes visible later, when
somebody edits one label and not the other, and the two then disagree about what
one claim rests on. That is the shape this project meets most often.[^3]

A parallel run makes the shape common rather than rare. Two branches that each
append an entry to one register take the same next label, and the merge puts
both definitions in one reference section. The dispatcher then resolves the
collision by hand, under time pressure, in a file nobody checks.

## What the work does

Add a check beside the other checks, and give it a broken fixture, in the way
the existing checks have one.[^4] The check reads every Markdown document in
the tree.

## Which documents the check covers

**Every Markdown document, not the records alone.** The item asked whether to
read the registers, the reviews, the backlog items and the product records. The
answer is yes to all of them, because the failure this check exists to catch
happened in a register and not in a record. A check that reads the records alone
would have passed over it.

The ordering test applies only where every label of a document is a number. A
register that labels a footnote by the entry that owns it is following a
different scheme, and a scheme that carries no order cannot be out of order.

## What the check fails on, and what it only reports

The item asked whether the check rejects a gap in the numbering, and left the
mode of each test open. This is the answer, and the reasoning is that a false
failure trains everybody to ignore a red gate, which costs more than the defect
it catches.[^5]

**It fails on a marker with no definition.** A reader sees the raw label. There
is no judgement to make and no document in the tree has a reason to hold one.

**It fails on a label defined twice in one document.** This is the collision
shape after a merge keeps both sides.

**It fails on two labels that hold one definition.** This is the rule against
repeating a footnote, and it is the shape the record review found.

**It fails on a definition that no marker cites.** This is the collision shape
after a merge renumbers one side and misses its marker. It is also the debt the
early records carry, so those enter a baseline.

**It reports the ordering, and does not fail on it.** Ordering is the one test
that cannot be a gate today. Documents across every directory break it, three of
them belong to the project owner and are outside the reach of this work, and the
repair is a renumbering sweep across a whole document, which is the operation
this project gets wrong most often.[^3] A gate nobody can turn green is a gate
everybody learns to skip. The check states the count and lists the documents on
request.

**It does not reject a gap.** A skipped number breaks nothing a reader can see,
and an item that removes a claim would otherwise have to renumber every footnote
after it. The reported ordering test reads the order of first occurrence, not
the completeness of the sequence.

## The baseline

The failing tests carry a baseline of the labels that already break them, in the
way the volatile-figure check carries one.[^4] The baseline is falsifiable: an
entry that matches nothing fails the check, so the list can only shrink and can
never go stale. Do not add to it. Repair the document instead.

The baseline exists because the early accepted records write a reference section
whose footnotes the body never cites. Those records predate the documentation
rule.[^6] Rewriting an accepted record is not this item's work.

## Impact review

**Governed by.** No decision record governs this work. The check reads text and
writes nothing.

**Changes.** No record. The item adds a target to the gate list and a script.

**Creates.** One decision row, not a record. The choice of mode for each test is
a decision the project should be able to read back, and it constrains nothing
that a future contributor could not reasonably reverse against new evidence, so
the scope rule leaves it out of a record.[^7]

**Blockers.** None.

**Precedent.** The finding that opened this item records what the gate missed
and why.[^2] The register numbering check states the same rule for a different
value: a name refers to one thing, and something must fail when two copies
disagree.[^8]

**Serves.** No product record.

## Done when

- The check reads every Markdown document in the tree.
- A marker with no definition, a label defined twice, two labels holding one
  definition, and a definition nothing cites each fail the gate.
- The ordering test reports and does not fail, and the reason is written down.
- The check has a broken fixture for each failing test, and the probe recipe
  requires the check to reject it.
- The baseline is falsifiable: an entry that matches nothing fails.
- The check runs against the real tree, the documents it fails are repaired or
  are baselined, and the commit body holds the counts and the search command.
- The whole check command runs green.

## Outcome

Done as planned. Four tests fail the gate, the ordering test reports, and the
debt is in a falsifiable baseline. Three things changed from the plan.

**The check is its own script, not an extension of the record check.** The item
said to extend the record check. The record check reads the records, and the
failure that opened the sibling item happened in a register. A separate script
reads every Markdown document, which is what the review decided the check must
cover.

**The check found a false failure in itself before it was wired in.** The first
version read a footnote definition after blanking its code spans, which the
citation check does for a different rule. A footnote names its source inside
the code span, so two footnotes to two different files read as one source. The
decision record priority index held the only instance. A finding records
it.[^9]

**A decision row was considered and not written.** The choice of mode for each
test is recorded here and in the finding that states why the ordering rule
cannot be a gate.[^10] A row in the decisions register would be a third copy of
one reasoning, which is the shape the check itself exists to catch.

**What the check found on the real tree.** Every failure it reported was real
after the false one was removed. The commit body holds the counts by test and
the survey command. The repairs were made where they were a line or two, and
the rest went into the baseline with a reason for each group. A follow-up item
takes the baseline down.[^11]

**Registers.** Two findings opened. No blocker opened or closed.

## References

[^1]: Documentation Rules, section 3. `.claude/rules/documentation.md`
[^2]: Findings register, FND-130. `docs/FINDINGS.md`
[^3]: Recurring Defect Shapes, shape 1 and shape 2. `.claude/rules/recurring-defects.md`
[^4]: The record check script. `scripts/check_adrs.py`
[^5]: Definition of Done, pass the gates. `.claude/rules/definition-of-done.md`
[^6]: Project orientation, the documentation rules. `CLAUDE.md`
[^7]: Decision Record Scope, the test for whether a decision needs a record. `.claude/rules/adr-scope.md`
[^8]: The register check. `scripts/check_registers.py`
[^9]: Findings register, FND-153. `docs/FINDINGS.md`
[^10]: Findings register, FND-152. `docs/FINDINGS.md`
[^11]: Backlog item 0166. `docs/backlog/proposed/0166-clear-the-footnote-baseline.md`
