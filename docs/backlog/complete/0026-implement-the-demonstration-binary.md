---
id: 0026
title: Implement the demonstration binary
status: complete
created: 2026-08-30
---

One command builds a world, steps the engine, and shows the window. This is
the item that meets PRD-0002.

It must be the real engine at a real thread count, not a rendering loop with
a simulation shaped like one. The recurring-defect rule calls the alternative
inert: a capability that passes its own test because the test drives the
mechanism instead of the engine.

Refine this at sprint 4 planning.

## Impact review

**Governed by.** PRD-0002 states the need. ADR-0067 D4 puts the loop on the
stepping thread and states the consequence.

**Changes.** None.

**Creates.** None.

**Blockers.** None.

## Done when

- One command opens a window and the world appears in it.
- Entities move while the developer watches, with no input.
- The thing that moves them is the engine the tests exercise.
- The same seed shows the same behaviour on every run.

## Outcome

`just watch` runs it. The binary builds a world of 40 by 28 tiles with 220
soldiers, steps the real engine at the machine's thread count, and draws
each frame.

The loop is nine lines: step, draw, show. There is no simulation in the
binary, which is what the product record's "the window shows the simulation,
not a copy" asks for.
