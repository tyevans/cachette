---
id: 0025
title: Implement the renderer
status: proposed
created: 2026-08-30
---

A Rust crate draws the hex world and the entities on it. It reads a published
frame and never writes to the world.

The rhombus world is a parallelogram on the screen, so the viewer applies the
skew. ADR-0017 says the engine does not.

Refine this at sprint 4 planning, after item 0024 writes row 0067.
