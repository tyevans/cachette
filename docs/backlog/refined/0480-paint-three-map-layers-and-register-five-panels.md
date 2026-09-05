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

The item has two halves. The first half paints the layers and registers the
weather panel. The second half registers the market, the economy and the score
panels, and it is done. **The relations panel is the one panel that remains**,
and the section below says why.

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

## The second half

The market, the economy and the score panels are registered. Each reads
through a reader the core crate already exposed, and none was added for it.
The impact review below states what each one reads.

- Market. For each faction, the rows of its board: the good, the quantity,
  whether the faction offers or wants it, the asking good and the asking
  quantity. Below the board, the count of live negotiations and contracts
  that name the faction. With a pointer set, the faction that holds the
  pointed tile comes first.
- Economy. For each settlement, its address and faction, its store, its
  production rate, its upkeep rate, and how many of its seats a live unit
  holds. The panel shows a fixed number of settlements and says how many more
  stand. With a pointer set, the settlement on the pointed tile comes first.
- Score. The tick, the tick limit and the ticks that remain, and whether the
  game has ended. When it has, the winner, the path and the tick. For each
  faction, its territory score and its four weights. With a pointer set, the
  faction that holds the pointed tile comes first.

## What is missing

**The relations panel is not done, and this item does not claim it.** It
waits for pass 3, which gives the engine a relation between two factions. It
needs its own impact review, in the shape of the one below. The review must
say which reader the panel reads and that none was added for it, and it must
state the address count the panel reads.

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

**The second half under the same records.** The market panel reads the board
of each faction and the negotiation plane. A board holds a fixed number of
rows. The plane holds one row for each ordered pair of factions, and the
faction ceiling bounds the pairs. The economy panel walks the settlement arena
for a fixed number of sites and stops. For each site it reads the address, the
faction, the store, two rates and the positions, and it reads the arena length
once for the count it did not show. The score panel reads the tick, the tick
limit, the game end record, and one score and one weight vector for each
faction. Every one of these is a stored field or a walk bounded by the faction
ceiling, so no panel starts a pass over a tile or a unit, which ADR-0070 D1
requires. Each pointer costs one holder read or one settlement read. Every
value crosses as an integer, and the one fixed-point conversion happens in the
viewer, which ADR-0067 D3 permits. No reader was added to the core crate.

**Changes.** None.

**Creates.** None. The layers and the panels add no decision. The panel
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
- The market, the economy and the score panels register with the deck, and
  `World.panel_names()` names each with no Python edit.
- The market panel names each advertised good, its quantity and its asking
  price, and the count of live contracts of each faction. The economy panel
  names the store, the rates and the housing of a settlement, and counts the
  settlements it did not show. The score panel names the ticks remaining while
  the game runs, and the winner, the path and the tick when it has ended.
- With a pointer set, each of the three panels puts the pointed faction or the
  pointed settlement first, and a test asserts it for each.
- No line of the three panels is cut at the worst plausible numbers. A test
  for each panel, made to return no lines, fails.
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
