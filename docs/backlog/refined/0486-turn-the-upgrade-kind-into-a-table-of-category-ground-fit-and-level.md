---
id: 0486
title: Turn the upgrade kind into a table of category, ground fit and level
status: refined
created: 2026-09-05
implements: [ADR-0151 D1, ADR-0151 D2, ADR-0151 D3, ADR-0151 D4, ADR-0151 D6, ADR-0145 D4, ADR-0002 D1, ADR-0001 D4]
changes: []
creates: []
serves: [PRD-0055]
blocked-by: [BLK-007, BLK-050]
---

## Why

**The upgrade kind is an enumeration of four variants, and every effect is a
match arm.** A road and a terrace fit any ground, have one level, and a game
that wants a fifth kind takes a release. The product asks that an upgrade suit
the ground under it, that it carry a level, and that a level raise what the
ground yields or holds.[^1] A decision record now states the shape that
answers it.[^2]

The kind becomes a table that the world is built with, from one constant in
the core crate, in the form the unit type table takes. A row is one category
at one level. It names the ground kinds it fits, the work that finishes it,
and the columns a pass reads today. The table enters the state hash.

The build verb takes a category and no level. It resolves the row from the
ground under each tile and the level that stands there, and it refuses a tile
that no row fits. The entry gains a level and the work done toward the next
level, and a raise happens in place. The capacity composition and the gather
resolve read a column and never a match arm.

The demonstration controller draws a category from the high bits of its one
draw, in the way it draws a kind today. A category the ground refuses is
counted and dropped.

**Pass 4 and pass 8 migrate into this table.** The wall of item 0475 and the
wonder and the store of item 0479 are rows and never variants. Item 0475 waits
for this item.

This item answers item 0348 and closes DEC-143 with its option B. The
outcome of this item says what became of 0348.

**This item touches `fn step` in `world.rs`, for the resolve and the raise.
Only one worker may hold it at a time.**

## Impact review

### The records that govern this work

**ADR-0151 D1, the table is data.** The core crate declares one row struct
through one macro, in the form the unit type row takes.[^4] The macro derives
the column names and the column reader from the declaration, so the boundary
cannot name a column the row does not hold.[^5] The columns are the ground
fit, the work, the yield change, the capacity change, the store capacity
change, the victory claim, and whether the row asks for the builder's own
ground. Every column is an unsigned 32-bit whole number, so the row holds no
padding and enters the hash byte for byte.[^6] The table holds one row for
each pair of a category and a level. A row whose ground fit is empty is not a
row, so the fit states presence and nothing states it twice.[^5]

**One disagreement with D1.** The record says that a column arrives with the
pass that reads it, and it names the cross cost as a column that waits.[^2]
This item therefore writes no cross cost column. The movement pass that would
read one does not exist.

**A second disagreement with D1.** The record says that no column is a flag.
The rule that a build asks for the builder's own ground is a yes or a no, and
D4 asks that the rule become a column read.[^2] The column is therefore a
whole number that no pass reads as a magnitude. The record keeps its text and
this item records the tension.

**ADR-0151 D2, a build order names a category.** The build verb takes a
category and no level. One function resolves the row from the ground kind, the
entry that stands there and the category. The verb calls it at the moment of
the order, and the pass that collects the build intents calls the same one on
every step, as the held-ground rule already does.[^7] The refusal is a typed
error that names the category and the ground kind.[^8] The set form counts a
refusal, and the controller census reads that count.[^9]

**ADR-0151 D3, a level is raised in place.** The entry holds the tile, the
category, the level that stands there and the work done toward the next level.
The level rises by one when the work done reaches the work of the next row,
and the work done returns to zero. The entry count does not grow with the
level. The work done is clamped at the work of the next row, so a builder
banks no surplus.[^10] At the top level the clamp is zero.

**ADR-0090 D2 and D3.** The clamp folds over the next row of the entry and no
longer over the whole catalogue. The overflow property test therefore changes:
it asserted that the work done of every entry sits at or below the largest
work in the catalogue, and it now asserts that the work done sits at or below
the work of the row above the entry, and zero at the top. The invariant check
of the map states the same rule, so a value that reaches the map by another
path fails it. D3 keeps its shape: one function composes the ground capacity
and the upgrade capacity, and it reads a column.

**ADR-0090 D1 and D4.** The storage stays sparse and stays in tile order. A
raise writes no entry. Destroying an upgrade removes the entry at whatever
level it stood.

**ADR-0151 D4, no pass branches on a category.** Every match on a category
name in the tree becomes a column read. The census counts a complete wonder as
a complete entry whose row holds a victory claim, and a store as a complete
entry whose row holds a store capacity change. The win path reads the victory
claim. The one function that composes the capacity reads the capacity change.
The gather resolve reads the yield change. The held-ground test reads the own
ground column.

**ADR-0151 D6, the five categories are rows.** The road, the terrace, the
wonder, the store and the wall are categories of the default table. The wall
is registered with one level and with no effect column set, because the
condition it needs belongs to item 0475 and this item does not write it. One
open category is registered with no row at all, so a caller adds a category
without a release.

**ADR-0151 D5 is not implemented here.** The viewer keeps one drawing for each
category and draws no level. Item 0487 paints the level.

**ADR-0145 D4.** The row is declared once, and the column names derive from
the declaration. A test asserts that the Python stub names equal the Rust
names, as the unit type table has.

**ADR-0002 D1.** Every column and every accumulator is a whole number. No
value in the table is a floating point number.

**ADR-0004 D1 and ADR-0009 D1.** The build pass keeps its key vector sort by
the tile and the category, and each thread writes its own slot. The resolve
reads the terrain, which is a pure function of the seed and the tile address,
so it adds no shared write.

**ADR-0001 D4.** The table enters the whole-world hash. The entry gains a
level, so every stored golden hash moves. The commit that lands the table
regenerates the golden file and gives the reason.

**ADR-0046 D1.** The refusal of a build order is a typed error and not a bare
boolean.

**ADR-0150 D4.** The rule that a build asks for the builder's own ground stays
one function with two callers. It reads a column of the resolved row instead
of naming the road.

### The blockers

**BLK-050 governs every value of the default table.** Which categories exist,
how many levels each holds, which ground each fits and what each level changes
are rules of the downstream game. Each value is a named constant in the core
crate, and each cites a row of the balance register. This item adds the rows
as unset, with a provisional value and a derivation that names this item.

**BLK-007 governs every work value.** A work is a cost in work, so the balance
rows name the blocker.

### How the condition of item 0475 shares the entry

The entry holds the work done toward the next level. Item 0475 adds a
condition as a second field of the same entry, with its own clamp, because the
two answer two questions.[^11] Repair comes before a raise: a build order on a
worn entry raises the condition first, and the work done rises only from full
condition. This item writes the work done and leaves the condition to item
0475. The order of the two fields inside the entry is a layout choice, and the
hash covers both when the second arrives.

### The default table

| Category | Levels | Ground it fits |
|---|---|---|
| Road | 2 | Plain, forest, hill |
| Terrace | 2 | Plain, hill |
| Wonder | 1 | Plain, forest, hill, mountain |
| Store | 1 | Plain, forest, hill, mountain |
| Wall | 1 | Plain, forest, hill, mountain |
| Open | 0 | None. A caller writes the row |

Water fits nothing, because no unit stands on water.

### The boundary

`order_build` takes a category index in place of a kind number. `build_order`
returns a category index. `define_upgrade_row` takes a category, a level and
every column of a row. `upgrade_table` returns every column as a dictionary of
arrays, in the form `unit_type_table` takes. `upgrade_at` gains the level. The
commit searches the tree for every caller of the kind and names the search.

## Done when

- The table is one constant in the core crate, and one macro declares the row,
  the column names and the column reader.
- A build order names a category, and one function resolves the row for both
  the verb and the pass.
- A build the ground does not fit is refused with a typed error that names the
  category and the ground kind.
- A level rises in place, and the entry count does not grow.
- The second level takes its own work and changes its own column.
- No source file outside the viewer matches on a category name. The commit
  body holds the search command.
- A row written at run time through the verb changes what a pass does, with no
  code naming the row.
- The Python stub column names equal the Rust column names, and a test asserts
  it.
- The two determinism tests pass at 1, 2 and 12 threads, and the golden hash is
  regenerated in the same commit with the reason in the body.[^3]
- Each new test was seen red with the defect put back, and the report names the
  defect and the test.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: PRD-0055, a god raises the ground its people hold, and sees what stands there. `docs/product/shaped/prd-0055-a-god-raises-the-ground-its-people-hold-and-sees-what-stands-there.md`
[^2]: ADR-0151, an upgrade is a category with a ground fit and a level, and a build order names the category. `docs/adrs/draft/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
[^3]: Findings register, FND-320. `docs/FINDINGS.md`
[^4]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D4. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
[^5]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
[^6]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
[^7]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D4. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^8]: ADR-0046, every error is typed. `docs/adrs/draft/adr-0046-every-error-is-typed.md`
[^9]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decisions D2 and D3. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^10]: Findings register, FND-011. `docs/FINDINGS.md`
[^11]: Backlog item 0475. `docs/backlog/proposed/0475-give-an-upgrade-a-condition-that-armies-wear-and-workers-repair.md`
