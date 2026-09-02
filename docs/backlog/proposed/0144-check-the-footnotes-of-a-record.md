---
id: 0144
title: Check the footnotes of a record
status: proposed
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

## What the work does

Extend the record check with two tests, and give each a broken fixture, in the
way the existing checks have one.[^4]

1. **Order.** The labels of a document, taken in the order of first occurrence
   in the body, must ascend.
2. **Repetition.** Two labels of one document must not hold the same definition
   text.

Repair the records the check then fails, in the same change or in a change
right after it. The commit body holds the search command and the list.

## The questions this item must answer before it is refined

**Which documents the check covers.** The records are the obvious set. The
registers use named labels rather than numbers, and the ordering test does not
apply to them. Decide whether the check reads the registers, the reviews, the
backlog items and the product records, or the records alone.

**Whether the order test admits a gap.** A document that skips a number breaks
nothing today. Decide whether the check rejects the gap or only the descent.

## Done when

Filled in when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Documentation Rules, section 3. `.claude/rules/documentation.md`
[^2]: Findings register, FND-130. `docs/FINDINGS.md`
[^3]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^4]: The record check script. `scripts/check_adrs.py`
