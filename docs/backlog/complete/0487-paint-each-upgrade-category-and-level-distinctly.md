---
id: 0487
title: Paint each upgrade category and level distinctly
status: complete
created: 2026-09-05
implements: [ADR-0151 D5, ADR-0067 D1, ADR-0093 D1, ADR-0094 D2]
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
adds no stage. Item 0486 landed the table, so the level the drawing reads now
exists.

## Impact review

### The records that govern this work

**ADR-0151 D5, the viewer reads the category and the level and paints them
apart.** This is the decision the item implements, and item 0486 left it
undone. The drawing reads `UpgradeSite::category` and `UpgradeSite::level` and
nothing else to choose a mark. Two entries that differ in category draw apart.
Two entries that differ in level draw apart. The record leaves the choice of
mark to the drawing pass, so this item chooses two channels and states why.

**The category stays the glyph.** The adjustment pass gave each category a
shape drawn over a dark rim, because a wash at the weight the progress gave
was the colour of the ground under it.[^3] Every category of the table takes a
shape of its own, and the shapes come from a table indexed by the category, so
a category added to the core fails to compile rather than borrowing the shape
of another.

**The level is a second, independent channel.** The level draws as a count of
pips under the glyph, one pip for each level that stands. A count is not a
hue, so it cannot cancel against the build-progress wash. **A shade of the
category colour is refused**, because the wash already carries the progress in
that channel, and report 25 measured exactly that collision for the site
colour.[^3]

**ADR-0067 D1, the viewer reads and never writes.** Every value the pass reads
comes from `upgrade_at`, `upgrade_table` and `tile_report`. The pass writes
nothing into the world, and no viewer value returns to the engine. The pip
count, the pip colour and the glyph shapes are the viewer's own, and the
engine holds none of them.

**ADR-0067 D2, the engine holds no value that exists for the viewer.** The
level count that scales the overlay is `UPGRADE_LEVEL_COUNT`, which the table
already holds because a row key needs it. Nothing is added to the engine.

**ADR-0093 D1, a quantity earns a place in the window if it changes moment to
moment.** A level rises while a watcher looks, so it earns a place on the map.
The level also goes into the inspector, which is the panel a watcher points at
one tile with, and into the colour key, which names what the map draws.

**ADR-0094 D2, the caller owns the pixels.** The marks are drawn into the
caller's canvas inside the one drawing command. No new pass over the world and
no second renderer is added.

**ADR-0070, the display reports what the drawing pass read.** The inspector
already names the category under the pointer. It gains the level, so a watcher
who counts pips and a watcher who reads the panel get the same answer.

### The blockers

**BLK-007 governs the per-frame cost.** Every cost figure in this project is
derived and not measured, so this item states none. The drawing already reads
`upgrade_at` once for each drawn tile that carries an entry, and the level is a
field of the answer, so the level costs no extra read. The pips and the glyph
are drawn inside the tile rectangle the pass already owns, and only at the tile
width the other per-tile marks draw at, so the added work follows the window
and not the world.

**BLK-050 does not govern this item.** Which categories exist and how many
levels each holds are values of the downstream game, and the drawing reads
them from the table rather than naming them.

### The open questions this item answers

**How the display reports what the drawing read.** The inspector names the
category, the level that stands and the work toward the next level. The colour
key gains a row that names the pip channel, so a reader learns what a count of
pips means.

**Whether the panel standard of item 0316 lands first.** It does not. The
upgrade marks are part of the tile drawing and not a panel, and the level line
goes into the inspector panel that already exists. Nothing here adds a special
case that 0316 would have to unwind.

**What the test asserts.** The tests drive the drawing pass and read pixels,
never a palette function. A test that read a colour back through the function
that produced it would stay green under its own defect.[^4] A build now needs
a zoned project before the engine accepts the order, so every fixture zones one
as the other viewer fixtures do.[^5]

## Done when

- Each category of the table draws a glyph shape of its own, and the shapes
  come from a table indexed by the category, so a new category cannot borrow
  one.
- The level draws as a channel that is not the category hue and not the
  progress wash, and a level 2 road, a level 1 road and a level 1 terrace are
  three different pictures.
- A site under work marks the middle of its tile, and a site that stands
  washes it. The distinction holds at every level.
- The upgrade overlay reads `UpgradeSite::level` and scales it against the
  level count of the table. It no longer derives a level from the category
  ordinal.
- The colour key holds a row that names the level channel.
- The inspector names the category and the level under the pointer.
- Each new test was seen red with the defect put back, and the report names the
  defect and the test.
- The pictures are written and their paths are reported.

## Outcome

**The category is the shape and the level is a count of pips.** Each category
takes a glyph from a table indexed by the category number, so a category added
to the core fails to compile rather than borrowing the shape of another. The
wall and the open category gained shapes of their own, which one match arm gave
them both alike. Under the glyph stands one pip for each level. A pip is white
against the dark rim, and its count is not a hue, so the level cannot cancel
against the wash that carries the build progress.

**A site that stands now draws the glyph as well as the wash.** The category was
readable only while a site was under work, because a site that stood washed its
tile and drew no mark. The distinction between the two states holds: a site
under work marks the middle and leaves the corner, and a site that stands washes
the whole tile. Below the width at which a glyph reads, the wash is still the
only mark.

**The upgrade overlay read the category ordinal and called it a level.** It now
reads the level of the entry and scales against the level count of the table.
The value is the level and one, because a site under work stands at level zero
and zero is also a tile that carries nothing.

**The colour key names the level channel, and the inspector names the level.**
The inspector also reads the work against the row above the entry, so a level 1
road that somebody is raising no longer reads as finished.

**Every new test was seen red with its defect put back.** The commit that landed
the tests holds the table of defects and the test each one failed. One of them
matters for the future: the older glyph test compares the pixels of two tiles,
and every category already draws in a colour of its own, so it stayed green
while the wall took the shape of the store. The new test compares the set of
pixels each mark covers, which is the shape alone.

**A fixture that steps two worlds by different tick counts proves nothing.** The
two worlds would differ in the weather, in the holding and in where every unit
stands. Both worlds now step the same number of ticks, and each test asserts
that the two draw the tile alike once the site is destroyed.

**A builder of a viewer fixture dies while it builds.** The first form of the
fixture stopped with a refusal that named no builder. A build that takes more
than a few ticks must put a live unit back on the tile and order it again each
tick. The same holds in the picture example.

**One disagreement with ADR-0151 D5 stands.** The record says that the drawing
pass reads nothing but the category and the level to choose a drawing. The wash
also reads the progress toward the next level, which the adjustment pass added
before this item and which report 25 asked for. The progress is a third channel
beside the two the record names. The record keeps its text, and this item
records the tension.

**The pictures are written and not committed.** They land under a build
directory, in the way the adjustment pictures do.

## References

[^1]: PRD-0055, a god raises the ground its people hold, and sees what stands there. `docs/product/shaped/prd-0055-a-god-raises-the-ground-its-people-hold-and-sees-what-stands-there.md`
[^2]: ADR-0151, an upgrade is a category with a ground fit and a level, and a build order names the category, decision D5. `docs/adrs/draft/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
[^3]: Research report 25, defect 1. `docs/research/reports/25-demonstration-readability-upgrades-and-units.md`
[^4]: Testing Rules, section 2a. `.agents/rules/testing.md`
[^5]: Findings register, FND-487. `docs/FINDINGS.md`
