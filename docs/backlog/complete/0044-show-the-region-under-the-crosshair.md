---
id: 0044
title: Show the region under the crosshair
status: complete
created: 2026-08-31
implements: [ADR-0067 D1, ADR-0067 D2, ADR-0022 D4, ADR-0024 D3]
changes: []
creates: []
serves: [PRD-0002, PRD-0003]
blocked-by: []
---

## Why

Level 1 exists and only the tests read it. A developer watching the world sees
tiles and units and no region, so the level that the engine now maintains
every frame is invisible to the person it was built for.

A capability that only its own tests invoke is the shape this project has a
name for. The level is not inert, because the engine maintains it, but nothing
a person can see depends on it.

## What the work does

1. The panel gains a section for the level 1 cell under the middle of the
   window.
2. The section states the extensive fields and the intensive readings, and it
   names which is which by the words it uses.
3. A reading over no tile prints a dash rather than a zero.

## Impact review

**Governed by.**

- ADR-0067 D1: the viewer reads the world through the public interface and
  writes nothing to it. The panel takes a shared reference.
- ADR-0067 D2: the engine holds no value that exists for the viewer. The
  formatting, the labels and the layout are the viewer's.
- ADR-0022 D4: a reader may read any level, and the level it read is part of
  the answer. The section heading says the reading is of a region, so a person
  cannot mistake it for a count of the window or of the world. The panel
  already separates those two, and this is a third.
- ADR-0024 D3: an intensive reading is a division done at read time. The
  viewer calls the reading and does no arithmetic of its own.
- ADR-0002 D4: floating point begins at the viewer boundary. A fixed-point
  reading is turned into text here and never handed back.

**Changes.** No record changes.

**Creates.** No record. A panel section is a layout, and no record holds one.

**Blockers.** None.

**Precedent.** The panel already learned that a count of the world and a count
of the window must be told apart by the label alone, never by the section a
row sits under. A region count is a third kind, and the same rule applies to
it.

A value too wide for the column is cut, and the cut is a guard rather than a
layout. A test reads the same lines and fails when a value reaches it.

## Outcome

The panel states the level 1 cell under the middle of the window, under its own
heading, with the extensive fields and the intensive readings named by the
words they use: tiles, open ground and units here are counts; units a tile,
open share and mean height are readings.

The viewer does no arithmetic on them. It calls the reading the engine
provides and turns the answer into text. A reading the engine could not give
prints a dash rather than a zero.

**A test compares the panel against the engine.** The cell the panel reports
must be the cell that covers the tile the camera reports, and the region must
cover fewer tiles than the window, or the panel would be reporting one thing
twice. A second test moves the camera until it finds a region that holds a
unit, because a section that always said zero would satisfy the first test and
show a person nothing.

**The panel outgrew the test window.** The height follows the content, and the
new section took it past a canvas 560 pixels tall. The fixture now matches the
720 the demonstration binary opens, so the test measures the panel rather than
the gap. The gap is real and an item holds it: nothing fails when the panel
stops fitting, and `bounds` then states a rectangle it did not paint.[^1]

**Checked by eye.** A region of 1 024 tiles reported 692 open, 70 units, 0.10
units a tile, and an open share of 0.68. Seventy over six hundred and
ninety-two is 0.101, and six hundred and ninety-two over one thousand and
twenty-four is 0.676.

## References

[^1]: Backlog item 0045. `docs/backlog/proposed/0045-the-panel-has-no-answer-for-a-short-window.md`
