---
id: 0210
title: Generate the ground of a drawn tile once
status: complete
created: 2026-09-02
implements: [ADR-0072 D1, ADR-0072 D4, ADR-0067 D1, ADR-0068 D1, ADR-0070 D1]
changes: []
creates: []
serves: [PRD-0005]
blocked-by: []
---

## Why

**The drawing generated the ground of every visible tile twice on every
frame.** It asked the terrain for the tile, to get the kind and the height
that give the colour. It then asked the world for the food the tile still
holds, and that reader generated the ground again to find what the tile
started with. Neither call stored anything, because the ground is a pure
function of the seed and the address and the engine keeps no copy of it.[^1]

The second generation produced a value the drawing already had. Nothing
failed, because both answers were the same answer.

**This is what a watcher feels as a sluggish camera.** At the far zoom the
window covers the whole world, so the count of visible tiles is the count of
the world. The cost of one drawing was then the count of visible tiles times
the cost of generating a tile, twice.

**The measurement.** The drawing was timed outside itself on a development
machine, over five frames at each of ten tile widths, on the world the
demonstration binary builds. At the smallest tile the camera allows, one
drawing cost about a third of a second, and about nineteen parts in twenty of
that was the two generations in equal shares. Reading the holder of a tile,
which the drawing also does for every tile, cost about four parts in a
thousand of the drawing. The figures are on a development machine and not on
the target platform, and no figure here is evidence about the target.[^2]

## Impact review

**Governed by.** ADR-0072 D1 states that the stock a tile started with is a
pure function of the seed, the address and the ground. That decision is what
makes the third term a value a caller can supply. ADR-0072 D4 states that the
engine stores what was taken and nothing else, and that no second copy of an
amount may disagree with the first. ADR-0067 D1 states that the viewer holds a
shared reference to the world and writes nothing to it. ADR-0068 D1 states
that the ground is generated and never stored as a map, which is why a
generation has a cost at all. ADR-0070 D1 states that the drawing pass reports
what it read, which is the mechanism the tests use.

**Read and not contradicted.** ADR-0093 governs what the window shows, and
this work changes nothing a watcher sees. ADR-0094 D6 refuses a frame below
one pixel for each tile. This work does not move that bound and does not read
a summary level.

**Changes.** No record changes. No record states how many times the drawing
generates a ground.

**Creates.** No record. The scope rule gives three conditions, and a decision
needs a record when all three hold.[^3] The second fails. A contributor could
choose a cache in the viewer instead of a reader, so the first holds. Changing
a cache back into a reader is cheap and contained in one crate, so the second
does not. ADR-0072 D4 already forbids a second copy of an amount, and the
defect rule already forbids a second declaration site with no check that fails
when the copies disagree.[^4] A new record would restate both.

**Blockers.** None. No value here comes from an unanswered question.

**Precedent.** FND-208 measured the split that this item rests on, and it also
records that a drawing cannot measure itself. FND-206 records that this layer
must not be judged from a render, and that a count settles what a picture
cannot. Both apply directly, and the tests read counts.

**Product record.** PRD-0005 asks that a watcher can tell what is happening
and why, and it names the developer who cannot tell a slow engine from a slow
drawing.

## What was chosen, and what was refused

**A reader, not a cache.** The item asked which of the two answers the defect.
A cache in the viewer is a second copy of a fact the engine can produce, and
nothing would fail when the two disagreed.[^4] A reader adds no copy. The
reader takes the ground that the caller already holds, and it generates
nothing.

**The far zoom question stays out.** Whether the drawing should read the
ground of a tile it draws one pixel wide is a separate question about what the
far zoom shows. It changes what a watcher sees. This item changed only what
the drawing costs.

## Done when

- The drawing asks the core for a ground once for each tile it paints.
- A reader exists that takes the ground the caller holds, and it does not
  generate a ground of its own. A test proves that by giving the reader a
  ground the address does not carry, and asserting that the answer follows the
  argument.
- The reader and the reader that starts from the address agree at every tile
  and every kind, over a world in which somebody gathered.
- The drawing carries a count of the grounds it asked for, and a test reads
  that count against the count of painted tiles.
- Each test has been proven able to fail. The commit body names each defect
  and which test caught it.
- No picture is used as evidence.[^5]
- The whole check command runs green.

## Outcome

**Done.** The drawing now reads the ground of a tile once, from the terrain,
and gives that ground to the stock reader. Two readers were added to the core
crate, and both take a ground rather than generating one.

**The claim that the cost halves is arithmetic and not a measurement.** It
follows from the split in the register, which says the two generations were
about equal shares of nineteen parts in twenty of a drawing.[^2] Nobody timed
the drawing again, on any machine. A figure taken on a development machine
would not be evidence about the target platform in any case.

**The count in the canvas counts the calls the drawing makes, not the
generations the engine runs.** A reader below the drawing that generated a
ground of its own would not appear in it. The two halves of the claim are
therefore held by two different tests, in two different crates, and no single
test states the whole of it. A finding records that limit and an item holds
the mechanism that would close it.[^6] [^7]

## References

[^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D1. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^2]: Findings register, FND-208. `docs/FINDINGS.md`
[^3]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^4]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^5]: Findings register, FND-206. `docs/FINDINGS.md`
[^6]: Findings register, FND-261. `docs/FINDINGS.md`
[^7]: Backlog item 0271, count the ground generations that one frame runs. `docs/backlog/proposed/0271-count-the-ground-generations-that-one-frame-runs.md`
