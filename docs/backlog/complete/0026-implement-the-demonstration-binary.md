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

`just watch` runs it. The binary builds a small world, spreads soldiers over
its open ground, steps the real engine at the machine's thread count, and
draws each frame. The extent and the soldier count are constants in the
binary, and later work changed both.

The loop steps, draws, then shows. There is no simulation in the binary,
which is what the product record's "the window shows the simulation, not a
copy" asks for.

**A correction.** This outcome first named the extent and the soldier count.
Later work changed both, and the record then stated a world the binary does
not build. The findings register holds the shape.[^1]

## References

[^1]: Findings register, FND-059. `docs/FINDINGS.md`
