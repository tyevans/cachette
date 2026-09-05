---
id: 0480
title: Paint three map layers and register five panels
status: refined
created: 2026-09-05
implements: [ADR-0067 D1, ADR-0067 D2, ADR-0067 D3, ADR-0070 D1, ADR-0070 D2, ADR-0093 D1, ADR-0093 D5, ADR-0094 D1, ADR-0094 D5, ADR-0140 D1]
changes: []
creates: []
serves: [PRD-0048]
blocked-by: [BLK-007]
---

## Why

**A watcher cannot see a relation, a board, a storm or a score.** The engine
holds them after passes 3 to 8, and the window shows none. This item is pass 9
of the living world game layer.[^1]

The item has two halves. **This refinement covers the first half only.** The
second half waits for passes 3, 6 and 8, and the section below says so.

The first half paints three layers on the map and registers one panel.

- Upgrades. A tile that carries an upgrade takes a tint per kind. The build
  progress sets the depth of the tint, so a finished road and a road just
  begun read as two different things.
- Weather. The water in the air over a cell is an overlay on every tile of
  that cell. Wet ground draws darker than dry ground.
- Luxuries. A tile that holds a deposit takes one mark.

One panel, weather, joins the deck. It shows the totals the field keeps: the
water in the air, the water on the ground, what evaporated, what a god raised,
and how many cells are wet. When the caller sets a pointer, it also shows the
air and the ground at the pointed cell.

The viewer holds the deck in one registration. The Python `panel_names` reader
derives from it, so the new panel appears with no Python edit.

**This pass does not touch `fn step` in `world.rs`.** It adds no field to the
engine and no reader to the core crate. Every value it paints comes from a
reader the core crate already exposes.

## What is missing before the second half is refined

**The second half is not done, and this item does not claim it.** It holds
four panels: relations, market, economy and score. Each waits for a pass.

- The relations panel waits for pass 3, which gives the engine a relation
  between two factions.
- The market panel waits for pass 6, which gives the engine a board.
- The economy and score panels wait for pass 8, which gives the engine a
  score.

Each of those panels needs its own impact review, in the shape of the one
below. The review must say which reader the panel reads and that none was
added for it, and it must state the address count each panel reads. When the
passes land far apart, split the second half into one item per panel.

## Impact review

**Governed by.** ADR-0067 D1 holds that the drawing reads the world through a
shared reference and never writes to it. Every layer and the panel take
`&World`. ADR-0067 D2 holds that the engine holds no value that exists for the
viewer. The colours, the tint depths and the mark size live in the viewer.
ADR-0067 D3 holds that floating point begins at the viewer boundary and never
returns. The layers mix colours in integers and convert nothing back.
ADR-0070 D1 holds that the panel adds no pass over the world. The weather panel
reads five totals and two values at one cell. Two of the totals, what a god
raised and what evaporated, are running fields. Three of them, the air, the
ground and the wet cell count, are sums over the level 1 lattice that the field
computes when asked. The lattice is smaller than the world by the square of
the block edge, and no reader keeps those three as running fields. The panel
reads them through the same reader the Python `weather_totals` verb reads.
ADR-0070 D2 holds that a number the panel cannot afford is absent. The panel
states a dry field as dry, and it names no cell when no pointer is set.
ADR-0093 D1 holds that the window shows what changes moment to moment. Air,
ground and wet cells change every tick after a storm. ADR-0093 D5 holds that
one reading feeds every layout. The panel reads at the same barrier the frame
draws at. ADR-0094 D1 holds that a frame is one command that carries no tile.
The layers run inside the tile loop the frame already runs. ADR-0094 D5 holds
that one renderer feeds every presenter. The layers live in the one drawing
pass. ADR-0140 D1 holds that a tile is answered from the cell that covers it,
so two tiles of one cell take the same weather tint, and the picture shows the
lattice.

**Changes.** None.

**Creates.** None. The layers and the panel add no decision. The panel
standard already states how a panel registers.[^2]

**Blockers.** BLK-007 governs the draw cost.[^3] The upgrade layer and the
luxury layer each do one binary search over a sorted table for each painted
tile. The holder layer does one array read. The weather layer does two array
reads through the cell of the tile. The cost is derived and not measured, and
the figure stays open until the target platform measures it.

**Precedent.** FND-198 records what the numbers cost when two readings of one
world disagreed, so the panel reads at the frame barrier and nowhere
else.[^4] FND-051 records that a fixture chosen for realism hides the defect it
should show, so each layer test compares a world with the feature to the same
world without it, and a test that puts the layer back to a no-op fails.[^4]

**Serves.** PRD-0048, a developer watches factions play a game to an end.[^5]

**Item 0316.** The panel standard that item asks for exists, and the weather
panel registers with it.[^2] Nothing in this half waits for that item.

## Done when

The first half is done when every statement below is true.

- A tile that carries an upgrade draws a tint per kind, and a tile at half
  progress draws a different tint from a finished one. A test paints a world
  with a building site and the same world without one, and the pixels differ.
- A tile under water in the air draws an overlay, and a tile on wet ground
  draws darker than the same tile on dry ground. A test paints a world after
  a storm and the same world without one, and the pixels differ.
- A tile that holds a luxury draws one mark. A test paints a world with a
  deposit and the same world without one, and the pixels differ.
- The weather panel registers with the deck. `World.panel_names()` names it
  with no Python edit, and a test asserts the count from the registration.
- The panel names the five totals. No line is cut at the worst plausible
  numbers. With a pointer set, the panel names the air and the ground at the
  pointed cell.
- This pass adds no keyed draw, so the per-field statement of item 0472 does
  not apply. The item says so rather than omitting it.
- The whole check command runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Design: the living world game layer, sections 9 and 13. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^2]: The panel standard. `crates/cachette-view/src/panel/mod.rs`
[^3]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^4]: Findings register, FND-198 and FND-051. `docs/FINDINGS.md`
[^5]: PRD-0048, a developer watches factions play a game to an end. `docs/product/accepted/prd-0048-a-developer-watches-factions-play-a-game-to-an-end.md`
