---
id: 0273
title: Invert the drawing so the control plane owns the camera and the pixels
status: complete
created: 2026-09-02
implements: [ADR-0094]
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

**The demonstration was a Rust program, and there was nowhere else to stand.**
The program that owned the loop owned everything about the picture: it built
the window, it held the camera, it allocated the canvas, and it called the
drawing. A second program that wanted a picture had to be a Rust program, had
to link the same window library, and had to repeat the loop.

**The project owner asked for the demonstration in the control plane.** The
decision record already settled the shape: the caller owns the camera and the
pixels, and one command fills them.[^1] This item is that record's
implementation.

**The rule this project already has covers the boundary.** The control plane
builds a selector and sends one command, and it never loops over entities. A
camera is a selector over tiles, and a frame is the set-valued command over
it.[^2]

## Done when

- One command takes a world, a camera and a caller's memory, and writes one
  frame into it. The command names no tile, no unit and no entity.
- The camera lives in the control plane. The engine holds none, so a frame is
  a pure function of a world and a camera.
- The crate that fills a frame carries no window library, so a wheel built
  from the binding runs on a machine with no display.
- The command refuses a buffer of the wrong size and a camera below one pixel
  for each tile, and each refusal names what it refused against.
- The Rust binary still runs, and it reaches the pixels through the same
  command the control plane uses.

## How it came out

**Both front ends now stand on one command.** The binary owns a buffer of its
own and calls the same function the binding calls, so the two presenters
cannot disagree about the world. The drawing itself did not move.

**The canvas learned to borrow.** It held its own pixels, and a caller's
memory could not reach the drawing without a copy through a frame the engine
allocated, which the record forbids. The pixels are now either owned or
borrowed, and the borrow checker holds the lifetime rather than a comment.
The drawing touches pixels through a handful of accessors and nowhere else, so the change was contained. The commit body holds the count.

**The camera verbs took a size instead of a surface.** Every one of them read
the width and the height of the canvas and no pixel. A caller that has not
drawn yet can now steer, which the control plane needs and a canvas could not
give it.

**The window library is optional, and the binding takes the drawing without
it.** The demonstration binary needs a window and nothing else does, so the
dependency sits behind a feature and the binary declares it. The register holds
which library and why.[^3]

**The refusal below one pixel for each tile was unreachable, so the camera
gained a setter.** Every camera verb held the scale to a floor, so nothing a
caller could build reached the bound the record names, and the refusal would
have shipped inert.[^4] The caller now owns the scale outright and the verb is
what refuses, which is what the record asks for. The scroll and zoom verbs
still hold the scale, because a person should not be able to press a key into
a refusal.

**A frame asked for before the first step is refused.** Founding puts units in
the world, and the structure that says which units stand on a tile rebuilds at
the step barrier.[^5] Between the two the drawing refuses rather than showing
a world without its people. This is the existing lifecycle and not a new
constraint, and the tests and the demonstration both step before they draw.

## References

[^1]: ADR-0094, the caller owns the camera and the pixels, and one command fills them. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
[^2]: ADR-0040, Python is a control plane and not a data plane. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^3]: Decisions register, DEC-107. `docs/DECISIONS.md`
[^4]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
[^5]: ADR-0018, the unit-to-tile bridge is derived and rebuilds at the barrier. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
