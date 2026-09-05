---
id: 0487
title: Paint each upgrade category and level distinctly
status: proposed
created: 2026-09-05
implements: [ADR-0151 D5, ADR-0067 D1]
changes: []
creates: []
serves: [PRD-0055]
blocked-by: []
---

## Why

**A watcher sees that a tile carries an upgrade and nothing more.** The map
does not show which upgrade, and it cannot show a level, because no level
exists. The product asks that a watcher read the category and the level from
the map without a query, that two levels of one category look different, and
that two categories of one level look different.[^1]

The drawing pass reads the category and the level of every entry it draws and
paints them apart. Which colour, glyph or shape stands for which row is a
choice of the view and a game may change it. What the view may not do is draw
two rows the same.[^2]

**This item touches the viewer only.** It writes nothing into the world and it
adds no stage. It waits for item 0486, because the level it draws does not
exist before the table lands.

## What is missing before this is refined

- The impact review against ADR-0151 D5 and ADR-0067 D1: the pass reads the
  category and the level and nothing else to choose a drawing, and it writes
  nothing.
- How the head-up display reports what the drawing pass read, so that a
  watcher who hovers a tile reads the same category and level the map
  shows.[^3]
- Whether the panel standard of item 0316 lands first, so that the upgrade
  layer registers as a panel rather than as one more special case.
- The test: a world with two entries that differ only in level, and two that
  differ only in category, and an assertion that the four drawings are
  pairwise distinct. The test drives the drawing pass and not a palette
  function.
- The "Done when" statements, in the shape of item 0472.[^4]

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: PRD-0055, a god raises the ground its people hold, and sees what stands there. `docs/product/shaped/prd-0055-a-god-raises-the-ground-its-people-hold-and-sees-what-stands-there.md`
[^2]: ADR-0151, an upgrade is a category with a ground fit and a level, and a build order names the category, decision D5. `docs/adrs/draft/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
[^3]: ADR-0070, the head-up display reports what the drawing pass read. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
[^4]: Findings register, FND-320. `docs/FINDINGS.md`
