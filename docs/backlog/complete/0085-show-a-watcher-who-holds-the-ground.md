---
id: 0085
title: Show a watcher who holds the ground
status: complete
created: 2026-08-31
implements: [ADR-0067 D1, ADR-0067 D2, ADR-0053 D2, ADR-0070 D1, ADR-0017 D2, ADR-0022 D1]
changes: []
creates: []
serves: [PRD-0006, PRD-0005]
blocked-by: []
---

## Why

The engine now says who holds each tile, and the holding changes while the
world runs. The viewer does not draw it. A watcher therefore cannot see a
boundary, and cannot see one holding meet another.

The product record asks for exactly that, and it is the one statement in its
list of checkable statements that the engine alone cannot answer.[^1] A fact
that nobody can see is a fact nobody can check.

## What the work does

1. The viewer draws a holder layer over the ground it already paints.
2. A tile that nobody holds draws as it does today.
3. A tile that a faction holds takes that faction's colour from the table the
   viewer already owns.
4. The edge between two holdings is drawn, and the edge between a holding and
   unheld ground is drawn, because the edge is what the record asks a watcher
   to see.

## Where the colour comes from

The proposed item asked this, and the answer is that the colour already
exists. This item adds none.

**The viewer holds one table of faction colours, and it is the only such
table.** The reader that gives the colour of a faction is in the painting
module, the head-up display already uses it to name the factions it draws, and
the engine holds no colour at all.[^2] The holder layer calls that same
reader. It does not build a second table, and it does not pick a colour from a
faction identifier itself. A second table would be one fact in two places, and
a watcher who compared a unit against the ground under it would see two
colours for one faction with nothing to say which was right.[^3]

The engine cannot supply the colour, and the boundary record is why: the
engine holds no value that exists because something draws it.[^4]

## Which faction column the layer reads

**The layer reads the holder, and never the tile faction column of the stub
system.** A tile carries two values that name a faction today, and item 0084
records that only one of them is a holder.[^5] The other is written when the
world is built, it never changes, and it covers water as well as open ground.

A viewer that drew the stub column would paint a full, still map of holdings
that no rule ever made. The picture would be plausible, and it would be wrong,
which is the failure the head-up display record was written against.[^6] This
item therefore reads the holder reader of the world and nothing else. Item
0084 removes the second column. This item does not wait for it, because the
reader it calls is unambiguous today.

## Impact review

**Governed by.**

- ADR-0067 D1. The layer holds a shared reference to the world and calls no
  method that takes a mutable one.[^7]
- ADR-0067 D2. The engine gains no colour, no layer and no field. Every value
  the layer needs, it derives from what it reads.[^4]
- ADR-0053 D2. A tile carries one holder field, and exclusivity is a property
  of the storage. The layer therefore paints one colour for one tile and
  cannot express two, so the picture shows the exclusivity rather than
  asserting it.[^8]
- ADR-0070 D1. The panel adds no pass over the world, and this layer adds none
  either. The holder of a tile is read where that tile is painted, on a loop
  that already runs, so the cost follows the window.[^9]
- ADR-0017 D2. The neighbours are six fixed offsets and the edge does not
  wrap. The edge test reads those offsets, and a neighbour outside the world
  is unheld ground rather than a wrap to the far side.[^10]
- ADR-0022 D1. Level 0 is the only truth. The layer reads the holder column of
  level 0 and never a summary level, so it cannot draw a holding that a
  summary has not caught up with.[^11]

**Changes.** No record changes.

**Creates.** No record. **This is a deliberate judgement against the scope
rule, and here is the reasoning.**[^12] The claim a record would hold is that
the presentation of a faction belongs to the viewer, and an accepted record
already holds it in a stronger form.[^4] The edge rule fails condition two: it
is a comparison between neighbouring tiles, and changing it costs one
function. The colour table fails condition three, because a reader of the
painting module sees why the table is there.

**Blockers.**

- BLK-007 governs every cost figure, so this item states none.[^13] The cost
  statement here is a shape: the layer grows with the tiles the window covers.
- BLK-013 is resolved, and the faction ceiling is above the number of colours
  the viewer can tell apart.[^14] The item therefore invents no colour count.
  A faction beyond the table shares a colour, the viewer already says so
  rather than showing a count it cannot back, and the holder layer inherits
  that.
- No blocker governs the rule that spreads a holding. The engine applies it
  already.

**Precedent.**

- FND-071 records that the whole-world pass the pyramid gave up was still
  alive in the viewer, and that nothing failed, because a whole-world loop is
  ordinary code.[^15] This item adds a per-tile read, and the read must sit
  inside the loop over the visible tiles.
- FND-054 records that a world narrower than the coarsest lattice spacing
  holds one kind of ground.[^16] A fixture here must hold two holdings that
  meet, and the extent belongs in the fixture.
- FND-061 records that a fixture assertion belongs over the outcome and not
  over the inputs.[^17] The fixture asserts that the world it built holds a
  boundary, by reading the holders back.
- FND-051 records that a fixture chosen for realism hides the defect it should
  show.[^18]

**Serves.** PRD-0006, and PRD-0005 for the legend.

PRD-0006 lists eight checkable statements. **This item answers the one that no
engine work can answer**: a watcher can see who holds a tile, and can see
where one holding meets another.[^1] It also makes a second statement visible,
because the record asks that a watcher see that holding is exclusive, and one
colour for one tile is how a watcher sees it. The other statements are the
engine's, and the engine meets them already.

PRD-0005 asks that the window name every colour it draws.[^19] The layer draws
no new colour, so the legend the panel already states names the holder colours
too. This item confirms that. It does not restate it.

**Conflict surface.** `crates/cachette-view/src/paint.rs` at the loop over the
visible tiles, and at the colour readers.
`crates/cachette-view/tests/draws_the_ground.rs` and
`crates/cachette-view/tests/paints_the_world.rs` gain the cases.
`crates/cachette-core/src/holding.rs` and `crates/cachette-core/src/world.rs`
are read and not changed.

**It cannot run beside item 0069**, which paints the same tiles to show a tile
over its capacity, and the two layers compete for the same pixels. **It cannot
run beside item 0084**, which removes one of the two faction columns this item
had to choose between. **It cannot run beside item 0093**, which changes the
frame call in the same crate. No engine item conflicts, because this item
changes no engine file.

## Done when

- A tile that a faction holds draws in that faction's colour, and a tile that
  nobody holds draws as it does today. A picture test asserts both.
- The edge between two holdings is drawn, and a picture test builds two
  holdings that meet and asserts the edge.
- The layer reads the holder of the world. A reviewer finds no read of the
  tile faction column of the stub system in the drawing path.
- The colour comes from the one table the viewer already owns. A whole-tree
  search finds no second table and no second reader that maps a faction to a
  colour, and the search command is in the commit body.
- The engine gains no field, no colour, and no method that exists to be drawn.
- The layer starts no loop over the world and no loop over the units. The
  holder read sits inside the loop over the visible tiles, and the count of
  tiles the layer reads equals the count the ground pass painted, plus the
  neighbours the edge test needs.
- A test asserts that the layer touches the same number of tiles for the same
  window, in a small world and in a large one.
- The fixture holds two holdings that meet, and it asserts that by reading the
  holders back after the world has stepped.[^17] [^18]
- The layer is put back to reading the wrong faction column, and the tests are
  watched failing, before the item is claimed done.
- The panel names every colour the frame drew, and no new legend row is
  needed. A test asserts that the holder colours are named.
- The two determinism tests are unaffected, and the commit body says so. The
  viewer is outside them.
- `just check` runs green.

## Outcome

The drawing pass reads the holder of every tile it paints, on the loop that
already runs. A tile that a faction holds takes that faction's colour, mixed
over the ground, so a watcher reads the kind of ground and the holder of it at
once. A tile that nobody holds draws as it did before.

A held tile whose neighbour has another holder takes a border in the same
colour. The six neighbours are the fixed offsets, and the edge of the world
does not wrap, so a neighbour outside the world reads as unheld ground rather
than as a wrap to the far side. The boundary between two holdings and the
boundary between a holding and unheld ground are both drawn.

The colour comes from the one table the viewer already owns. The layer builds
no second table and picks no colour from a faction identifier itself. A
whole-tree search found one table and one reader.

The layer reads the holder of the world, and never the tile faction column of
the stub system. That column has no public reader, so the viewer cannot reach
it at all today. The break-it experiment therefore reproduced the column's
rule inside the layer, which is the tile index modulo the faction count.

The drawing pass counts the holders it reads and the held tiles it paints.
Two tests read those counts. The first states the rule exactly: one read for
each painted tile, and six more for each held one. The second draws the same
window in a world and in a world six times as wide, and the two counts are
equal.

Four defects went into the drawing code, one at a time, and each was watched
failing. The wrong faction column failed four tests. A layer that drew no
border failed the edge test. A layer that swept the world failed both cost
tests.

The panel needed no new legend row, and a test asserts that the panel names
every holder colour the frame drew.

The two determinism tests are unaffected. The viewer is outside them, and no
engine file changed.

## References

[^1]: PRD-0006, a place belongs to somebody. `docs/product/shaped/prd-0006-a-place-belongs-to-somebody.md`
[^2]: ADR-0067, the viewer reads the world and never writes to it, decision D5. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^3]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^4]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^5]: Backlog item 0084. `docs/backlog/proposed/0084-give-a-tile-one-faction-column.md`
[^6]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
[^7]: ADR-0067, the viewer reads the world and never writes to it, decision D1. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^8]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^9]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
[^10]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D2. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
[^11]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^12]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^13]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^14]: Blockers register, BLK-013. `docs/BLOCKERS.md`
[^15]: Findings register, FND-071. `docs/FINDINGS.md`
[^16]: Findings register, FND-054. `docs/FINDINGS.md`
[^17]: Findings register, FND-061. `docs/FINDINGS.md`
[^18]: Findings register, FND-051. `docs/FINDINGS.md`
[^19]: PRD-0005, a watcher can tell what is happening and why. `docs/product/shaped/prd-0005-a-watcher-can-tell-what-is-happening-and-why.md`
