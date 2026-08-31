---
id: 0032
title: Draw the terrain in the viewer
status: complete
created: 2026-08-30
implements: [ADR-0067 D2, ADR-0067 D3, ADR-0068 D1, ADR-0068 D4]
changes: []
creates: []
serves: [PRD-0003]
blocked-by: []
---

## Why

The core generates a terrain kind and a height for every tile. The engine now
reads the passability of a tile, but nothing shows a person what the ground
is. The viewer still paints every tile the same, so a developer cannot see the
ground that the tests prove exists.

The product record states the need plainly: a developer cannot tell one part
of the world from another, and therefore cannot judge whether the world is a
place or see a defect that has a position.[^1]

## What the work does

1. The viewer colours a tile by its kind and shades it by its height.
2. The viewer derives every colour itself. The engine gains no field.
3. The viewer reads the ground only for the tiles the window covers.
4. Tests assert that the five kinds paint five distinct colours, that a taller
   tile of one kind is brighter than a shorter one, and that the picture of
   one seed differs from the picture of another.

## Impact review

**Governed by.**

- ADR-0067 D2 states that the engine holds no value that exists for the
  viewer. A colour table therefore lives in the viewer crate. A field named
  for a display, in the engine, is the violation the record names.
- ADR-0067 D3 states that floating point begins at the viewer boundary and
  never returns. The height is a Q16.16 value in the engine. The viewer may
  read it as a number and turn it into a brightness, and it must never hand
  the result back.
- ADR-0067 D1 states that the viewer reads the world through the public
  interface. `World::tile_terrain` is that interface and it already exists.
- ADR-0068 D1 states that the ground is a pure function of the seed and the
  address, and that the engine stores no map. A read costs arithmetic. The
  record calls a whole-world sweep every frame a design mistake, so the
  drawing must stay inside the window.
- ADR-0068 D4 states that the engine says what a tile is and never what a tile
  costs. A colour is neither. It is a property of the picture, and the viewer
  owns it.

**Changes.** No record changes.

**Creates.** No record. The three-condition test fails on condition two: a
later contributor may reasonably choose another palette, and changing one
costs nothing. A palette is not a constraint.

**Blockers.** BLK-007 governs every cost figure, so this item states none. The
work states the shape the product record asks for: the cost of the ground
follows the window and not the world.[^2] [^1]

**Precedent.** The recurring-defect rule names the inert-capability shape: the
project declares a capability, tests it directly, and nothing acts on it.[^3]
The terrain was the instance, and backlog 0033 was the first consumer of it.
This is the second and the last one the product record needs.

A fixture that supplies no extreme hides the defect.[^4] A world narrower than
the coarsest lattice spacing of the generator holds one kind of ground only,
so a fixture that must show five kinds is wider than that spacing.[^5]

## Outcome

The viewer draws the ground. A tile takes its colour from its kind and its
brightness from its height, so a person reads both the kind and the relief.
The simulated tile value ripples on top, so a still camera over a stepping
world still moves.

The panel gained a section that names each kind against a count of the tiles
of that kind in the window. The product record asks that the kinds be few and
that a person be able to name them, and a palette alone does not meet that: it
leaves the reader guessing which green is forest.

No record changed and none was written. The palette, the names and the counts
are the viewer's own, which is what the record requires of every display
value.

**The height shades a tile and never tints it.** Two kinds are therefore told
apart by hue alone, whatever height either tile has. The first palette failed
this: a bright forest tile matched the plain colour more closely than the
forest one, and the test caught it. The two greens now differ in the blue
channel by more than the shading can move them.

**A test that read one corner passed against the defect.** The first version
of the colour test used a large zoom on the corner of the world, and that
corner was all water. Every tile it read was one kind, so a palette that gave
every kind one colour passed. The test now fits the whole world in the window
and asserts that it saw all five kinds. This is the fixture shape the register
already holds, in a third subsystem.[^6]

An example was added that writes one frame to an image file. The demonstration
binary needs a display; this needs none, so a person without one can look at
the ground and a reviewer can attach the picture to a change.

## References

[^1]: PRD-0003, a developer sees a world worth looking at. `docs/product/shaped/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
[^2]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^3]: Recurring defect shapes, shape 3. `.claude/rules/recurring-defects.md`
[^4]: Testing rules, section 2a. `.claude/rules/testing.md`
[^5]: Findings register, FND-054. `docs/FINDINGS.md`
[^6]: Findings register, FND-051. `docs/FINDINGS.md`
