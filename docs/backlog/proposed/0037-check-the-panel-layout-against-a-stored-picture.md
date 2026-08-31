---
id: 0037
title: Check the panel layout against a stored picture
status: proposed
created: 2026-08-30
implements: []
changes: []
creates: []
serves: [PRD-0005]
blocked-by: []
---

## Why

The head-up display draws text into a framebuffer. Its tests read the numbers
the panel holds, and they check that everything the panel paints stays inside
the rectangle the panel states.

They do not check that the panel is readable. A value that overruns its
column is cut, so it stays inside the rectangle and says the wrong thing. A
row that overlaps the row under it stays inside the rectangle too. Both are
layout defects, both are silent, and one of them already happened during the
first build: a row of two counts ran past the panel edge, and only a rendered
picture showed it.

The window is not the answer. A window needs a display, and continuous
integration has none. The painting is separable from the showing, which is why
every viewer test paints into a canvas and none opens a window.

## What is missing before this is refined

- The form of the stored picture. A whole frame changes whenever the world
  changes, so it would fail on every unrelated commit. A picture of the panel
  rectangle alone, drawn from a readout the test builds by hand, would not.
- Whether a readout can be built by hand. It is read from a world, a camera, a
  canvas and a set of measurements today. A test that wants a fixed picture
  needs a fixed readout, and the cost figures inside it come from a clock.
- What the test does when it fails. A pixel comparison that prints "the
  pictures differ" is not usable. It needs to write the picture it got, so a
  person can look at it.

## Done when

Not yet stated. The item is not refined.

## Outcome

Filled in when the item moves to `complete/`.
