---
id: 0133
title: Let a watcher reach a panel longer than the window
status: proposed
created: 2026-09-01
implements: []
changes: []
creates: []
serves: [PRD-0005]
blocked-by: []
---

## Why

The panel is longer than the window holds. It cuts at the foot, and it says
so with a notice on the last line.[^1] A watcher who reads the notice cannot
then reach the rows below it. There is no scroll, no fold and no other page.

The panel grew again when the viewer gained a shortage section and a section
that names each faction that founded.[^2] Both sit above the view rows,
because a section near the foot is the first thing a cut removes. That
placement is a choice made against a fixed list, and every later section
forces the same choice again.

What a cut costs today, in the demonstration window: the ground legend and
the cost rows fall off. The product record asks that the window name every
colour it draws, and a cut panel does not.[^3]

The notice is honest, and honesty is not the same as reachable. A number that
says it is missing is better than a number missing in silence, and it is
still a number a watcher cannot read.

## What the work might do

The shape is open. Three candidates, none chosen:

1. A key that folds a section, so a watcher opens the one they want.
2. A key that scrolls the panel, so every row is reachable at some position.
3. A second column when the window is wide enough, so the panel uses the
   width it has rather than only the height.

## The questions this item must answer before it is refined

**Whether the panel gains a state.** A fold and a scroll both hold a position
between frames. The viewer holds the camera already, so a panel position is
the same kind of value, and it never reaches the engine.[^4] The item must
say where it lives and who owns it.

**What a picture test then pins.** The stored picture of the panel is taken
of one layout. A panel with a position has more than one, and the item must
say which the picture holds and how a test reaches the others.

**Whether the order of the sections is the cheaper answer.** A panel ordered
so that the rows a watcher needs most come first may close the gap with no
new mechanism. That is a smaller change and it should be measured against
the others.

## What this item does not do

It adds no rule to the engine, and it changes no simulated value. It does not
change what the panel says. It changes what a watcher can reach.

## Done when

Filled in when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: The head-up display module. `crates/cachette-view/src/hud.rs`
[^2]: Backlog item 0120. `docs/backlog/complete/0120-draw-what-a-unit-suffers-and-where-each-faction-founded.md`
[^3]: PRD-0005, a watcher can tell what is happening and why. `docs/product/shipped/prd-0005-a-watcher-can-tell-what-is-happening-and-why.md`
[^4]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
