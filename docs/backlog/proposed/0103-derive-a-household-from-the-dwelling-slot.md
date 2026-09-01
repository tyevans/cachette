---
id: 0103
title: Derive a household from the dwelling slot
status: proposed
created: 2026-08-31
implements: []
changes: []
creates: []
serves: [PRD-0014, PRD-0015]
blocked-by: [0059]
---

## Why

A watcher cannot ask who lives together. Item 0059 gives a unit the dwelling it
lives in, and item 0067 gives a unit its parents. Neither one answers the
question a watcher asks about a family: who is under this roof.

PRD-0014 and PRD-0015 both need the answer, and neither one owns it today.
PRD-0015 states that a watcher can ask who is in a household. Nothing in the
backlog derives one.

## What the work does

1. A watcher names a dwelling and reads every unit that lives in it.
2. The reader derives the household. It stores nothing and it declares
   nothing.
3. A unit that takes a dwelling of its own leaves the household it was in, by
   moving and not by a rule that splits a household.
4. A transfer of a dwelling slot moves a unit between households.

## The answer this item takes, stated plainly

**A dwelling is stored and a household is derived.** A unit carries the slot of
the dwelling it lives in, and a household is every unit that carries one
slot.[^1]

**A household reads no descent.** The recommendation that a household is the
residents of one site who share a line is rejected. Two strangers under one
roof are one household, and a parent and a child who live apart are two. A
household is a fact about a place, not a fact about a family.

**Nothing stores a household roster.** A stored roster would be a second
declaration of where a person lives, and nothing would fail when the roster and
the slot disagreed. That is the shape this project keeps meeting.[^2]

## What is missing before this is refined

- The impact review. The records that govern the derived read have not been
  read against this work, and the item cannot yet name them by decision.
- The reader shape. Whether a household read is a set-valued query the control
  plane issues, or a level 0 gather the viewer runs, is not worked out. Both
  answer the need, and they cost differently.
- The index. Reading every unit of one dwelling needs an order over units by
  dwelling slot. Whether that order already exists after item 0059, or whether
  this item builds it, is not settled.

## Done when

- A watcher names a dwelling and reads every unit that lives in it, through the
  public interface.
- No structure stores a household, and no check reconciles two rosters, because
  there is only one.
- A unit that moves to another dwelling leaves one household and joins another,
  with no rule that names a household. A test asserts both sides of the move.
- A unit that lives nowhere is in no household, and that is a representable
  answer rather than an error.
- The read order over the members of one household is fixed by a stable key,
  and a property test asserts that the members come back in the same order at
  1, 2 and 12 threads.
- The fixture holds a dwelling with one resident, a dwelling with several, an
  empty dwelling, and two residents who share no ancestor. The commit body says
  how that was checked.[^3]
- No cost figure appears in the code or in a comment.
- `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Decisions register, DEC-039. `docs/DECISIONS.md`
[^2]: Recurring Defect Shapes, section 1. `.claude/rules/recurring-defects.md`
[^3]: Testing Rules, section 2a. `.claude/rules/testing.md`
