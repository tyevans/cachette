---
id: 0494
title: Register an overlay deck, and let the watcher switch the map between info views
status: complete
created: 2026-09-05
implements: []
changes: []
creates: []
serves: [PRD-0004, PRD-0005]
blocked-by: [BLK-007]
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

## Impact review

**The records that govern this work.** Three records govern it, and each is
answered decision by decision below.

- **ADR-0067, the viewer reads the world and never writes to it.** D1 holds:
  every overlay reads the world through a shared reference and writes nothing.
  D2 holds: the overlay deck keeps no per-frame memory in the engine, and the
  chosen overlay travels on the frame command from the caller. D3 holds: the
  colour ramp is floating point arithmetic inside the viewer, and no value
  formed from it returns to the engine.
- **ADR-0093, the window shows what changes.** D5 holds: the caller sends the
  name of an overlay and no text of its own, and the viewer holds the words the
  key writes.
- **ADR-0094, the caller owns the camera and the pixels.** D1 holds: the caller
  owns the buffer, the camera and the choice of overlay, and it draws nothing
  itself. D5 holds: **one renderer feeds every presenter.** A presenter chooses
  among names the one renderer published. It cannot name an overlay the
  renderer does not hold, and it cannot supply a rule of its own for painting
  one, so no presenter becomes a second renderer.

**Records this work changes.** None. The work implements the three records
above and contradicts none of them.

**Records this work creates.** None. The overlay deck is a mechanism inside the
one renderer, and the constraint that governs it is already written: one
renderer feeds every presenter. A record that named the module arrangement
would record an arrangement rather than a constraint.

**Blockers.** BLK-007 governs the cost of a frame. An overlay adds reads to the
drawing pass, and a value that lives on the level 1 lattice adds four cell
reads to each tile it paints. **No figure for that cost is stated here**,
because every cost figure in this project is derived and this one is not
measured on the target platform.

**Settled before.** The findings register holds the shape this work most risks:
one fact declared in two places. The panel deck avoids it, and the overlay deck
copies that avoidance.

## The open questions, answered

**Does an overlay replace the ground colours or tint them?** It tints them, and
it tints them **before** the holder colour is mixed in. The alternative, a
wash over the finished pixel, is the defect the research report measured: an
overlay at a weight near the holder weight erases which faction holds the
ground. Mixing under the holder makes that impossible by construction rather
than by choosing a small weight, and the holder therefore survives at every
overlay strength.

**Does the overlay belong to the frame command or to a second command?** To the
frame command, as one optional name. A second command would be a second entry
to the one renderer, and a caller could then hold a stale overlay against a
fresh frame. The name is refused when no registered overlay carries it, and the
refusal names the overlays that exist, because a frame with no wash on it looks
the same as a frame the caller mistyped.

**Does the reference key become an overlay of its own?** No. The key says what
the colours mean, so it must be able to speak about whichever overlay is on. An
overlay that hid the key would take away the only statement of its own scale.
The key stays a flag, and it gains a section that names the overlay, its low
value, its high value and the unit.

**What does a watcher see when the value is zero everywhere?** The key says so
in words. The drawing pass records the lowest and the highest value it painted,
and the key writes that the overlay found nothing in the window when the two are
zero. An empty overlay and a broken overlay otherwise look the same.

**Which keys does the demonstration bind?** The number keys. The panel deck
holds the function keys from F1 upward, and two actions on one key means a
watcher cannot reach the second. The number keys 1 to 9 name the overlays in
the order the engine registers them, and 0 turns the overlay off. The list comes
from the engine, so an overlay that joins the deck gets a key with no edit to
the demonstration.

## Done when

1. An overlay registers itself in one list in the viewer, with a name, a
   colour, a span and a per-tile value, and nothing else states that it exists.
2. The boundary answers the registered names, derived from that one list.
3. The frame command takes one optional overlay name, and refuses a name that
   the renderer did not publish with an error that names the list.
4. A value that lives on a level 1 cell paints as a field. Two tiles far apart
   inside one cell differ when the neighbouring cells differ.
5. Two runs of one frame give one picture.
6. The colour key names the overlay, its low value, its high value and the
   unit, and says in words when the overlay found nothing.
7. The holder colour and the unit marks survive under every registered
   overlay.
8. The demonstration cycles the overlays on the number keys, derived from the
   engine list, prints the overlay name when it changes, and names the keys in
   its opening lines.
9. A test for each of the statements above, and a picture of the demonstration
   world under each overlay.

## Outcome

**Nine overlays register in one list, and nothing else says that they
exist.** They are moisture, air, food, wood, stone, height, holder, upgrade
and crowding. Each reads only readers the engine already published, so the
work added no reader to the core. An overlay states its own name, its own
colour, its own span and the value of one tile, and the boundary answers the
names from that list.

**The frame command takes one optional overlay name.** The engine refuses a
name it did not publish, and the refusal names the overlays that exist. A
name from a published list is a choice among what the one renderer offers,
so a presenter still draws nothing itself.

**A value on the level 1 cell lattice interpolates between the four nearest
cell centres.** The storm now reads as a field with no straight edge that the
world does not have. The interpolation costs four cell reads for each painted
tile, and the blocker that governs every cost figure still holds, so no
figure for it is stated.

**The overlay mixes into the ground before the holder takes its share.** The
holder tint therefore survives at every overlay strength. A test reads each
overlay at the held tile where that overlay paints most strongly, and it
fails when the overlay is moved over the finished pixel.

**The colour key names the overlay, its low value, its high value, the unit
and what the pass met in the window.** An overlay that found nothing says so
in words, so an empty overlay never reads as a broken one.

**The demonstration binds the number keys.** The keys 1 to 9 name the
overlays in the order the engine registers them, and 0 turns the overlay off.
The mapping is as long as the list the engine answers.

Three defects were put back, one at a time, and the tests were watched. Each
failed the one test that exists to catch it. The commit bodies hold the
detail.

## References

[^1]: Research report 24, demonstration readability, resources and weather, sections 4, 6 and 7. `docs/research/reports/24-demonstration-readability-resources-and-weather.md`
[^2]: The panel standard, the registration list. `crates/cachette-view/src/panel/mod.rs`
[^3]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
[^4]: The frame command and its type stub. `crates/cachette-py/src/lib.rs`, `python/cachette/_core.pyi`
[^5]: ADR-0094, the caller owns the camera and the pixels, decision D5. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
[^6]: ADR-0153, a tile's lease follows the units that stand on it. `docs/adrs/draft/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
[^7]: Backlog item 0278, say what the demonstration world never produced. `docs/backlog/proposed/0278-say-what-the-demonstration-world-never-produced.md`
