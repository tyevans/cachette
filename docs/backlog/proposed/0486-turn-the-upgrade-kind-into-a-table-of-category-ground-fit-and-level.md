---
id: 0486
title: Turn the upgrade kind into a table of category, ground fit and level
status: proposed
created: 2026-09-05
implements: [ADR-0151 D1, ADR-0151 D2, ADR-0151 D3, ADR-0151 D4, ADR-0151 D6, ADR-0145 D4, ADR-0002 D1, ADR-0001 D4]
changes: []
creates: []
serves: [PRD-0055]
blocked-by: [BLK-007, BLK-050]
---

## Why

**The upgrade kind is an enumeration of two variants, and every effect is a
match arm.** A road and a terrace fit any ground, have one level, and a game
that wants a third kind takes a release. The product asks that an upgrade suit
the ground under it, that it carry a level, and that a level raise what the
ground yields or holds.[^1] A decision record now states the shape that
answers it.[^2]

The kind becomes a table that the world is built with, from one constant in
the core crate, in the form the unit type table takes. A row is one category
at one level. It names the ground kinds it fits, the resource the tile must
yield or none, the work that finishes it, and the columns a pass reads today:
the yield bonus and the capacity. The table enters the state hash.

The build verb takes a category and no level. It resolves the row from the
ground under each tile and the level that stands there, and it refuses a tile
that no row fits. The entry gains a level and the work done toward the next
level, and a raise happens in place. The capacity composition and the gather
resolve read a column and never a match arm.

The demonstration controller draws a category from the high bits of its one
draw, in the way it draws a kind today. A category the ground refuses is
counted and dropped.

**Pass 4 and pass 8 migrate into this table.** The wall of item 0475 and the
wonder and the store of item 0479 are written as rows and never as variants.
Item 0475 waits for this item. Item 0479 is being implemented as two flat
kinds now, and its rows move into the table when this item lands.

This item answers item 0348 and closes DEC-143 with its option B. The
outcome of this item says what became of 0348.

**This item touches `fn step` in `world.rs`, for the resolve and the raise.
Only one worker may hold it at a time.**

## What is missing before this is refined

- The impact review, decision by decision, against ADR-0151 and against
  ADR-0090 D2 and D3. The clamp now folds over the next row and not over the
  catalogue, and the review must say how the overflow property test changes.
- How the condition of item 0475 and the work done of this item share one
  entry, so that repair comes before a raise. ADR-0151 D3 states the rule and
  the review must state the fields.
- The default table for the demonstration: which categories, how many levels,
  which ground each fits. The values are rows of the balance register under
  BLK-050, and the review must add the rows with a provisional value and a
  derivation.
- The Python type stub and the build verb signature. The commit that lands the
  table searches the tree for every caller of the kind and names the search.
- The per-field tests and the extreme the fixture reaches: a category at its
  top, so a raise is refused; a tile of another category, so the order is
  refused; a ground that no row fits, so the order is refused; and a raise
  that lands on the tick the work done reaches the row, so the reset is
  proven.
- The "Done when" statements, in the shape of item 0472: the two determinism
  tests at 1, 2 and 12 threads, the defect put back and the test red, and the
  golden hash regenerated in the same commit with the reason in its body.[^3]

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: PRD-0055, a god raises the ground its people hold, and sees what stands there. `docs/product/shaped/prd-0055-a-god-raises-the-ground-its-people-hold-and-sees-what-stands-there.md`
[^2]: ADR-0151, an upgrade is a category with a ground fit and a level, and a build order names the category. `docs/adrs/draft/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
[^3]: Findings register, FND-320. `docs/FINDINGS.md`
