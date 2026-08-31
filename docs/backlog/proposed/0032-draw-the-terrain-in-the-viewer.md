---
id: 0032
title: Draw the terrain in the viewer
status: proposed
created: 2026-08-30
implements: []
changes: []
creates: []
serves: [PRD-0003]
blocked-by: []
---

The core generates a terrain kind and a height for every tile. No system reads
either one. The viewer still paints every tile the same, so a developer cannot
see the ground that the tests prove exists.

The work colours a tile by its kind and shades it by its height. The viewer
reads only the tiles the screen shows, and it derives the colour itself. The
engine holds no colour.

This item was left out of the terrain milestone on purpose. The viewer crate
was held by another agent while the terrain work ran, and two agents never
hold one source file.

The impact review must name the viewer record and the record that keeps a
display value out of the engine.
