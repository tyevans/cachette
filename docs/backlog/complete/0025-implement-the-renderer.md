---
id: 0025
title: Implement the renderer
status: complete
created: 2026-08-30
---

A Rust crate draws the hex world and the entities on it. It reads a published
frame and never writes to the world.

The rhombus world is a parallelogram on the screen, so the viewer applies the
skew. ADR-0017 says the engine does not.

Refine this at sprint 4 planning, after item 0024 writes row 0067.

## Impact review

**Governed by.** ADR-0067 D1, D2, D3 and D5. ADR-0017 D4 puts the skew here.
ADR-0002 D4 allows floating point outside simulated state.

**Changes.** The float ban is a workspace lint, and the viewer needs the
types the lint bans. The allowance is stated once at the crate root with its
citation. The script that closes the gap the lint leaves already reads the
core crate only, and that scope is correct.

**Creates.** None. ADR-0067 holds the claims.

**Blockers.** BLK-007 governs every cost figure. None appears.

## Done when

- A world becomes pixels, and a step changes them.
- The skew is applied by the viewer and not by the engine.
- The viewer cannot write to the world, and a test shows nothing moved.
- No test needs a display.
- Every drawing rule has a mutation that kills a test.

## Outcome

`crates/cachette-view` holds the painter. `Camera::centre_of` applies the
skew, so a row shifts the column and the rhombus becomes a parallelogram.

The mutation check found two gaps that the first eleven tests missed. Drawing
tiles and no soldiers killed nothing, so the product record's "entities appear
on the world" had no test at all. Painting every tile one colour also killed
nothing, because the soldiers supplied the colours that the colour-count test
was counting. Three tests now cover both, and every mutation dies.

The target check no longer builds the viewer for the primary target. The
viewer opens a window, so it links a C library that needs a cross-compiler,
and a window on a headless server means nothing. The engine is what ships.
