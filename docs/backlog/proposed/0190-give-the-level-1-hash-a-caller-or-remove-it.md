---
id: 0190
title: Give the level 1 hash a caller or remove it
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

The pyramid holds a verb that folds every cell of level 1 into a state hash,
and each cell summary holds a verb that folds its own fields. **No file calls
either one.** The world state hash does not read them, and no test does.

Two answers are defensible, and neither is written down.

**Level 1 is derived, so the world hash should not read it.** The tiles it
summarises already enter the hash, so a fold over the level says the same thing
twice, and a derived value in a hash makes a rebuild that is merely slower look
like a change of state.[^1]

**A derived level that disagrees with level 0 is a defect the hash could
catch.** The record requires the equality between a level and the level below
to be a test, and one test recomputes it. A hash would carry it into every
golden run instead.[^2]

Whichever answer is right, an unread fold is the shape the project keeps
meeting: a capability that its own test would pass and that nothing
invokes.[^3] [^4] The work that added the food total to the summary found
this.[^5]

## What the work does

Decide, then act. Either the world hash reads level 1, and the golden files are
recorded again, or the two folds are removed and the removal is stated.

## What is missing before this is refined

- The impact review.
- Whether a rebuild that gives the same answer more slowly should move a golden
  file. That is the argument against putting a derived level in the hash.
- What a hash over level 1 would catch that the recomputation test does not.
- Whether the removal takes the state hash argument out of the public interface
  of the pyramid, and what a control plane reader would then lose.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^2]: ADR-0023, an aggregate combines exactly, in any order, decision D5. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^3]: Recurring defect shapes, shape 3. `.claude/rules/recurring-defects.md`
[^4]: Findings register, FND-181. `docs/FINDINGS.md`
[^5]: Backlog item 0183, carry the food of a cell into the level 1 summary. `docs/backlog/complete/0183-carry-the-food-of-a-cell-into-the-level-1-summary.md`
