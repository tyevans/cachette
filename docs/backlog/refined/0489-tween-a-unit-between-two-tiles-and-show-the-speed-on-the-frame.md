---
id: 0489
title: Tween a unit between two tiles and show the speed on the frame
status: refined
created: 2026-09-05
implements: [ADR-0094 D1, ADR-0094 D2, ADR-0094 D3, ADR-0094 D4, ADR-0094 D5, ADR-0067 D1, ADR-0067 D2, ADR-0067 D3, ADR-0093 D1, ADR-0093 D5]
changes: []
creates: []
serves: [PRD-0048]
blocked-by: [BLK-007]
---

## Why

**A unit jumps from tile to tile, and a watcher cannot read the speed.** The
engine moves a unit one whole tile in one tick, and the window paints the unit
at the centre of its tile. At a speed below one tick for each frame the unit
stands still for several frames and then jumps. The window names the tick and
does not name the speed, so a watcher who slows the world cannot see that the
world slowed.

This item does two things.

- The viewer draws a unit that moved between two neighbouring tiles at a point
  between the two tile centres. The caller passes the share of the current tick
  that has elapsed on the wall clock, and the viewer places the unit at that
  share of the way from the old centre to the new one. A unit that is new to
  the view, or one that jumped further than a neighbouring tile, draws at its
  tile with no tween.
- The frame command takes the speed as a number, and the window renders it as
  a word beside the tick: `x1`, `x1/2`, `paused`.

## The memory the viewer keeps

The viewer keeps one table between frames: the tile at which each unit was
last painted, keyed by the unit identity. The table holds an entry for a unit
painted last frame and for no other unit. It is rebuilt from the units the
frame paints, so a unit that left the view, died, or fell outside the frame is
gone from the table at the end of that frame. That is the eviction rule.

**The table is bounded by the frame and never by the population.** The caller
gives the bound, and the frame command that fills a caller's pixels uses the
pixel count of the frame. A frame cannot show more units than it has pixels,
so the bound loses nothing a watcher could see. When more units are painted
than the bound admits, the units past the bound are not recorded and draw at
their tile on the next frame.

The table is the caller's memory. The record that bounds the viewer says that
interpolation is the viewer's memory and not the engine's, and this table is
that memory.[^1] The binding holds it beside the timing and the founding
report it already keeps for the panel.

## Impact review

**Governed by.** ADR-0094 D1 holds that a frame is one command that carries no
tile and no unit. The phase and the speed are two numbers on that command, and
the caller names no entity. ADR-0094 D2 holds that the caller owns the pixels
and the engine keeps no frame. The tween table is not a frame and holds no
pixel. ADR-0094 D3 holds that the camera is the caller's. The tween adds no
camera value; the interpolated point is computed from the camera the caller
passed. ADR-0094 D4 holds that the render core carries no window library. The
tween adds no dependency. ADR-0094 D5 holds that one renderer feeds every
presenter and that the engine takes no text from the caller. The speed crosses
the boundary as an integer, and the viewer renders the word. ADR-0067 D1 holds
that the drawing reads the world and never writes to it. The tween reads the
address of each painted unit and writes to the viewer's own table. ADR-0067 D2
holds that the engine holds no value that exists for the viewer. The table
lives in the viewer, and the core crate gains no field. ADR-0067 D3 holds that
floating point begins at the viewer and never returns. The phase is a floating
point number that stops at the viewer. ADR-0093 D1 holds that the window shows
what changes moment to moment. The speed changes when the watcher changes it
and shows beside the tick, which also changes; the row is one word on the card
that already moves. ADR-0093 D5 holds that one reading feeds every layout. The
speed enters the readout once and both layouts render it from there.

**Determinism.** ADR-0002 D1 forbids a floating point number in simulated or
aggregated state, and ADR-0002 D4 allows one in rendering.[^2] The phase, the
interpolated point and the tween table are rendering. No pixel enters a state
hash, no thread-count test compares a frame, and the table is read by key and
never iterated. A fractional position in the viewer therefore violates no
determinism rule. The order in which units are painted does not change.

**Changes.** None.

**Creates.** None.

**Blockers.** BLK-007 governs the cost of a frame.[^3] The tween adds one table
lookup and one table insertion for each painted unit, on the loop that already
paints. The cost is derived and not measured, and the figure stays open until
the target platform measures it.

**Precedent.** FND-051 records that a fixture chosen for realism hides the
defect it should show, so the tween test moves one unit one tile by a verb and
reads the pixels at the midpoint, and a test that puts the tween back to a
no-op fails.[^4]

**Serves.** PRD-0048, a developer watches factions play a game to an end.[^5]

## Done when

- A unit that moved one tile paints at the midpoint of the two tile centres at
  phase one half, and at its new tile at phase zero of the next tick. A test
  reads the pixels, and the test fails when the tween is a no-op.
- A unit that jumped further than a neighbouring tile paints at its tile.
- The table never holds more entries than its bound, and a unit not painted on
  a frame is gone from the table after that frame.
- The frame command takes the phase and the speed as numbers and no text.
- The window and the whole panel render the speed word beside the tick, and a
  test asserts each word at its number.
- The demonstration clock yields the phase and the speed each frame, and its
  speeds include one half and one quarter of a tick for each frame.
- The two pictures at phase zero and phase one half differ, at a city zoom.
- The targeted gates run green: the format check, the lint of the viewer and
  the binding, the viewer tests, and the demonstration tests.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0067, the viewer reads the world and never writes to it, consequences. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^2]: ADR-0002, state holds no floating point number, decisions D1 and D4. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^3]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^4]: Findings register, FND-051. `docs/FINDINGS.md`
[^5]: PRD-0048, a developer watches factions play a game to an end. `docs/product/accepted/prd-0048-a-developer-watches-factions-play-a-game-to-an-end.md`
