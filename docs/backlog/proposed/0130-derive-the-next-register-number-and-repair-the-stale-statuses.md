---
id: 0130
title: Derive the next register number and repair the stale statuses
status: proposed
created: 2026-09-01
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

Three registers hold a stated next number. A writer reads that line, takes the
number, and increments it. The line is a second declaration of a value the rows
already hold, and the check already derives the true value from the rows in
order to compare them.

The line therefore has one purpose: a writer reads it. It has one cost: it goes
stale, and it conflicts on every merge that adds a row.

**A night of parallel work made the cost concrete.** The findings register
stated a next number that its own rows had already passed. The same line then
conflicted in four consecutive merges, once with three different values across
two branches and the trunk. The decisions register and the blockers register hold
the same shape and were stale at the same time.

This is shape 1 of the recurring defect rule, in the register that records the
shapes.[^1]

## Two other defects of the same kind

**A decided row sits under the open heading.** The decisions register holds four
rows that state a decision and sit under `## Open`. A reader who trusts the
heading reads four settled questions as unsettled. The heading and the row state
the same fact, and nothing fails when they disagree.

**A footnote definition has no citation.** The findings register defines a
footnote that no body text cites. It is orphaned in both parents of the merge
that found it, so it predates that merge. The documentation rule numbers
footnotes in the order they occur in the body, and a definition that occurs
nowhere has no place in that order.[^2]

## What the work does

1. The check derives the next number from the rows and prints it. A writer runs
   the check to learn the number, instead of reading a line.
2. The stated line goes from all three registers.
3. The allocation guidance in each register, and in the backlog guide, says how
   to obtain a number without reading a stored one.
4. The decided rows move under the closed heading, or the heading changes to say
   what it holds. Whichever is chosen, the status is stated once.
5. The orphaned footnote definition goes, and the footnotes renumber so that
   they run in the order the body cites them.

## The question this item must answer before it is refined

**Whether removing the stated line makes a number harder to allocate in parallel
work.** The dispatcher allocates ranges to workers in advance, and a worker is
told not to read the line at all. So the line serves a writer working alone. Ask
whether the check printing the number serves that writer as well, and say so.

**Whether the renumbering of the footnotes is worth its risk.** The findings
register holds more than fifty footnote definitions and the file is edited by
several workers at once. A renumbering conflicts with every branch in flight. It
may be right to remove the orphan and leave the gap.

## Impact review

Not done. The item stays in `proposed/` until it is.

## Done when

Filled in when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^2]: Documentation Rules, section 3. `.claude/rules/documentation.md`
