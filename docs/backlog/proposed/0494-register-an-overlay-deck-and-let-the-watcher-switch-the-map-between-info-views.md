---
id: 0494
title: Register an overlay deck, and let the watcher switch the map between info views
status: proposed
created: 2026-09-05
implements: []
changes: []
creates: []
serves: [PRD-0004, PRD-0005]
blocked-by: []
---

## Why

**The project owner asked for three things.** He asked that the weather layer
be drawn properly. He asked for a moisture overlay that the watcher can switch
on. He asked for other switchable information maps beside it.

The present drawing does not serve any of the three, and a research report
measured why.[^1] Three measurements state the case.

- **The air overlay is near absent at rest.** Before a storm, the air over a
  cell runs from 54 to 156 drops, which is an overlay weight of 1 to 5 of 255.
  A watcher therefore sees nothing when no storm is falling.
- **A storm paints one level 1 cell's value flat across every tile of that
  cell.** The boundary between two cells is a straight line, and the world is a
  sheared rhombus, so the footprint reads as rectangles that follow nothing in
  the world.
- **The wet shade cancels the food shade exactly.** The wet layer takes 34 from
  every channel and the full food ramp adds 34 to every channel. The two are
  the same size and opposite in sign, so a wet tile with the most food draws as
  the same colour as a dry tile with none.

This item is the general form of the three requests. **The map shows one thing
at a time, and the watcher chooses which thing.** A single picture that carries
every quantity at once cannot carry any of them, which is what the three
measurements above each show in a different way.

## What it asks for

**An overlay deck that registers itself.** The viewer already holds one list
that is the registration of every panel, and the Python reader of panel names
derives from that list.[^2] The overlay deck copies that pattern. The engine
owns the list and answers it, so a new overlay gets a name and a key with no
edit to the Python demonstration. This is the defect shape the panel deck
already avoids: one declaration site for one fact.[^3]

**The caller names one overlay when it asks for a frame.** The frame command
already takes a list of panel names and a reference flag, and the caller sets
both.[^4] An overlay name is the same kind of argument. **The engine takes a
name from a list it published, not free text**, so a presenter chooses among
what the one renderer offers and draws nothing of its own, which ADR-0094 D5
requires.[^5]

**The overlays to register first** are these eight.

- Moisture. The water on the ground of a cell.
- Air. The water above the ground of a cell.
- Food.
- Wood and stone.
- Height.
- The faction that holds each tile.
- The level of the upgrade on each tile.
- How crowded each tile is.

The lease of ADR-0153 and the influence field are later additions.[^6] The deck
takes each of them as one more registration, with no new machinery.

**A value that lives on a level 1 cell must not paint as a rectangle.** Weather
lives on the cell lattice, and a cell is 32 tiles a side. The requirement is
that a cell value is spread across the tiles of the cell, so that the map reads
as a field and not as a grid of blocks. **How it is spread is the implementer's
choice**, under one constraint: two runs of one frame give one picture.

**Each overlay states its own scale**, so that a watcher can tell a small value
from a large one. The reference key names what the colours mean for the overlay
that is on, and not for a fixed set of layers.

**The base map stays legible under an overlay.** An overlay must not erase
which faction holds the ground. The report measured that the air overlay does
erase it today: the overlay covers a tile at a weight of 92 of 255 against a
holder weight of 96 of 255, in a colour no faction uses, so a stormed holding
loses its colour.[^1]

## What is missing before this is refined

- The impact review. ADR-0067, ADR-0093 and ADR-0094 govern this work, and the
  review must go through each of them decision by decision.
- Whether an overlay replaces the ground colours or tints them, and what each
  answer costs at the target scale.
- Whether the overlay belongs to the frame command or to a second command, and
  what ADR-0094 requires of that choice.
- Whether the reference key becomes an overlay of its own rather than a flag.
- What a watcher sees when the value of an overlay is zero everywhere, so that
  an empty overlay is not read as a broken one. A subsystem that produces no
  instance looks the same as a subsystem that nothing draws, and backlog item
  0278 holds that shape and what it has already cost.[^7]
- Which function keys the demonstration binds. The panel deck already takes F1
  upward, and the refinement must say how a watcher cycles overlays.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Research report 24, demonstration readability, resources and weather, sections 4, 6 and 7. `docs/research/reports/24-demonstration-readability-resources-and-weather.md`
[^2]: The panel standard, the registration list. `crates/cachette-view/src/panel/mod.rs`
[^3]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
[^4]: The frame command and its type stub. `crates/cachette-py/src/lib.rs`, `python/cachette/_core.pyi`
[^5]: ADR-0094, the caller owns the camera and the pixels, decision D5. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
[^6]: ADR-0153, a tile's lease follows the units that stand on it. `docs/adrs/draft/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
[^7]: Backlog item 0278, say what the demonstration world never produced. `docs/backlog/proposed/0278-say-what-the-demonstration-world-never-produced.md`
