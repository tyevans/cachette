---
id: 0210
title: Generate the ground of a drawn tile once
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: [PRD-0005]
blocked-by: []
---

## Why

**The drawing generates the ground of every visible tile twice on every
frame.** It asks the terrain for the tile, to get the kind and the height that
give the colour. It then asks the world for the food the tile still holds, and
that reader generates the ground again to find what the tile started with.
Neither call stores anything, because the ground is a pure function of the seed
and the address and the engine keeps no copy of it.[^1]

The second generation produces a value the drawing already has. Nothing fails,
because both answers are the same answer.

**This is what a watcher feels as a sluggish camera.** At the far zoom the
window covers the whole world, so the count of visible tiles is the count of
the world. The cost of one drawing is then the count of visible tiles times the
cost of generating a tile, twice.

**The measurement.** The drawing was timed outside itself on a development
machine, over five frames at each of ten tile widths, on the world the
demonstration binary builds. At the smallest tile the camera allows, one
drawing cost about a third of a second, and about nineteen parts in twenty of
that was the two generations in equal shares. Reading the holder of a tile,
which the drawing also does for every tile, cost about four parts in a thousand
of the drawing. The figures are on a development machine and not on the target
platform, and no figure here is evidence about the target.[^2]

**Removing one generation halves the cost of a drawing at every zoom.** That is
arithmetic from the split above, not a second measurement.

## What is missing before this is refined

- The impact review.
- **The engine reader does not exist.** The generator that turns a ground kind
  into a starting stock is private to the resource module. The public readers
  all start from an address, so a caller that already holds the ground cannot
  ask what that ground produces. The work needs a reader that takes the ground
  the caller has. That reader lives in the core crate, and this item does not
  say what it should be called.[^3]
- Whether the answer is that reader, or a cache the viewer keeps. A cache is a
  second copy of a fact the engine can produce, and this project treats a second
  copy as a defect shape unless a check compares them.[^4] A reader adds no copy.
- Whether the drawing should read the ground of a tile it draws one pixel wide
  at all. That is a separate question about what the far zoom shows, and it must
  not be folded in here, because it changes what a watcher sees and this item
  changes only what the drawing costs.
- What a test asserts. A test that counts generations needs a counter the
  engine does not have. A test that asserts on elapsed time is forbidden.[^5]
  The honest check may be a benchmark, which gates nothing.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D1. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^2]: Findings register, FND-208. `docs/FINDINGS.md`
[^3]: The resource field and its generator. `crates/cachette-core/src/resource.rs`
[^4]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^5]: Testing Rules, section 3. `.claude/rules/testing.md`
