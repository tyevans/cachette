---
id: 0498
title: Give the upgrade table a housing category
status: proposed
created: 2026-09-05
implements: []
changes: []
creates: []
serves: [PRD-0014, PRD-0012]
blocked-by: [0059, BLK-050]
---

## Why

A site holds a housing capacity, and the capacity follows from what has been
built there rather than from the ground.[^1] Nothing builds housing. The upgrade
table holds a category for a road, a terrace, a wonder, a store and a wall, and
it holds none for housing.

A site whose housing is full stops growing.[^2] So a faction that wants more
people must build more housing, and today no build order can. The population of
a faction is then fixed by the housing its sites started with, and a faction
that gathers well and one that gathers badly reach the same ceiling.

The product record asks that the consequence of crowding go away when more
places are built.[^3] That statement is unreachable without this item.

## What the tree already holds, and what it does not

Read on 5 September 2026. **No housing figure exists anywhere.** Three names
look like one and none of them is.

- `crates/cachette-core/src/site.rs`, line 274 and line 343, hold a member named
  `capacity`. It is the ceiling on the slots the settlement arena opens, and it
  is not a number of people.
- `crates/cachette-view/src/paint.rs`, line 2724, raises `units_housed` for
  every unit whose home column names a site. It counts units that have a home.
  It does not compare that count against anything.
- `crates/cachette-view/src/panel/economy.rs`, line 120, prints a row named
  `housed`, and its two numbers come from the seats of the site. A seat is a
  position, not a dwelling.

So this item builds on the housing field that item 0059 adds, and on nothing
that stands today. **The three names above already give one word three
meanings**, and the review must not add a fourth.[^4]

**The store category is the shape to copy.** The upgrade table already holds a
category whose row raises what a site can store, through a column named for the
store capacity change. Housing is the same shape with a different column.

## What the work does

1. Add a housing category to the upgrade table, with a ground fit and levels,
   against the record that says what a category is.[^5]
2. Add one column to the upgrade row, for the housing that a finished level
   gives. Name it for housing, so it does not take the word `capacity`.
3. Make a finished housing upgrade raise the housing of the site on or beside
   its tile.
4. Add the balance rows for the housing each level gives and the work each level
   takes, unset and behind the blocker.
5. Let the faction plan zone a housing project, so the built-in controller can
   order one.

## Impact review

Not done. This item is `proposed/`, and refining it is the work.

**What refining must answer.**

- How a finished upgrade reaches the housing capacity of a site. A tile carries
  the upgrade and a settlement carries the capacity, so the review must name the
  one place that composes them and must not create a second declaration of the
  number.[^4]
- Whether the composition is derived on read or stored on completion. ADR-0157
  D1 says the capacity is a stored field, so a derived composition would
  contradict it and the review must say which changes.[^1]
- Which ground a house fits, against the record that says an upgrade is a
  category with a ground fit and a level.[^5]

**Blockers.** BLK-050 governs the housing each level gives and the work each
level takes. This item states neither in the code.

**Waits on item 0059**, which gives a site the housing field this item raises.

## Done when

- A finished housing upgrade raises the housing capacity of its site, and a test
  asserts the capacity before and after.
- The capacity is declared once. A whole-tree search shows one write path, and
  the commit body carries the search.
- A site whose housing was full grows again after a house is finished, and a
  test drives the engine to it.
- The state hash covers whatever the item adds.
- The thread-count test and the golden state test pass at 1, 2 and 12 threads.
- No figure appears in the code or in a comment.
- The whole check command runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0157, a site's free places are its built housing less the residents the engine already counts, decision D1. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
[^2]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, the consequences. `docs/adrs/draft/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
[^3]: PRD-0014, everyone needs somewhere to live. `docs/product/accepted/prd-0014-everyone-needs-somewhere-to-live.md`
[^4]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
[^5]: ADR-0151, an upgrade is a category with a ground fit and a level. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
