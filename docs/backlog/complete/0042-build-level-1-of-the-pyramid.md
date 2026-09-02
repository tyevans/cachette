---
id: 0042
title: Build level 1 of the pyramid
status: complete
created: 2026-08-31
implements: [ADR-0022 D1, ADR-0022 D2, ADR-0022 D3, ADR-0023 D1, ADR-0023 D2, ADR-0023 D3, ADR-0023 D4, ADR-0023 D5, ADR-0024 D1, ADR-0024 D2, ADR-0024 D3, ADR-0024 D5]
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

The engine simulates a hex world at three levels of detail. Only level 0
exists. Nothing summarises a block of tiles, so nothing can answer a question
about a region without reading every tile in it.

The three foundational records are written and nothing implements them. A
record the code does not reach is weaker evidence than one it does, and three
records with no reader are three claims nobody has tested.

## What the work does

1. A level 1 cell holds the exact combination of the tiles it covers.
2. A cell carries extensive fields only. Every intensive reading is a division
   of two extensive fields, done when a caller asks.
3. The pyramid rebuilds at the barrier, after the derived structure it reads.
4. Tests assert the equality with level 0, the independence from grouping and
   order, the independence from the thread count, and the weighting of an
   intensive read.

## Impact review

**Governed by.**

- ADR-0022 D1 and D2: level 0 is the only truth, and a cell equals the exact
  combination of what it covers. The pyramid holds no fact of its own.
- ADR-0022 D3: no system writes to a level above level 0. One mechanism
  maintains the pyramid.
- ADR-0023 D1 and D2: every field combines exactly associatively and
  commutatively. Field-wise integer addition is both.
- ADR-0023 D3: the arithmetic is exact and the accumulator is wide. A `u8`
  tile field summed over the target tile count overflows a `u32`, so a level 1
  accumulator is an `i64`.
- ADR-0023 D4: a field with an inverse may be updated incrementally. Every
  field in this work is a sum, so every field has one. This work builds the
  rebuild path only, and the incremental path is a later cost decision.
- ADR-0023 D5: the equality is a test, not a comment, and it runs at more than
  one thread count.
- ADR-0024 D1, D2, D3 and D5: every field declares its kind, an intensive
  field is stored as the extensive parts it divides, and a division by a zero
  denominator returns no value rather than a zero.
- ADR-0018 D2: the derived structure partitions the world by the same block
  the pyramid aggregates over. The pyramid reuses that layout and declares no
  geometry of its own.
- ADR-0004 D1: the fold visits the tiles of a cell in index order, and each
  thread writes its own cells.

**Changes.** No record changes.

**Creates.** No record. Every claim is written. The one choice this work makes
that no record holds is which fields level 1 carries, and a field set is
content rather than a constraint.

**Blockers.** BLK-007 governs every cost figure, so this work states none. The
scale constants table holds the tile count that the accumulator width follows
from.

**Precedent.** The block edge exponent is already declared once, in the
derived structure, and its own documentation says the pyramid aggregates over
the same block so neither subsystem may choose it alone. A second geometry
here would be the shape the rule names first.[^1]

A capability nothing invokes ships inert, so the engine must drive the
rebuild and a test must start at the engine.[^2]

## Outcome

Level 1 exists, the engine maintains it at the barrier, and twelve tests hold
it.

**A cell carries extensive fields only.** The tile count, the open ground, the
units, the value total and the height total. Every reading that does not scale
with the ground is a division of two of them, done when a caller asks: the
mean value, the mean height, the share of open ground, and the units for each
tile of open ground. A reading over no tile returns no value rather than zero.

**The units for each open tile divide by the open ground, not by the cell.** A
unit cannot stand on water, so a field that borrowed the tile count would
report a lower crowd than the ground carries. That is the case ADR-0024 D4
names, and it is in the code rather than in a comment.

**The ground contribution is computed once.** The first cut read the ground of
every tile every frame, and that is the whole-world sweep ADR-0068 calls a
design mistake. The step went from about 5 ms to about 59 ms on an empty world
of 281 600 tiles, and the head-up display test failed because the step time
stopped fitting its column. The ground does not change for the life of a
world, so its contribution is read when the level is built and combined into
every rebuild after that.

**The rebuild is parallel above the thread count and serial below it.** A
level with fewer cells than the caller has threads costs more in starting them
than the cells cost. The rule reads the cell count and the thread count, both
of which the caller supplied, and holds no constant of its own.

**The invariant is scoped to a barrier.** The tile total holds at every
moment, because the ground does not change. The unit total holds only when the
derived structure describes the arena. A spawn made between two frames leaves
the level as stale as the structure it was built from, which is the documented
state and not a defect. An earlier version asserted the unit total
unconditionally and failed on a despawn that no frame had yet seen.

**The remaining cost is what reserved row 0025 is for.** A full rebuild each
frame costs about 2.6 ms on this development machine for a world of 281 600
tiles. The registry reserves a row for the two update paths and their
threshold, and the note that closed item 0008 said that row waited on a
measurement. There is one now, and an item carries it.[^3]

**Level 2 is not built.** The combination of every cell is available and is
what level 2 would hold for a world of one region. A second array, with its
own geometry, waits for a reader that needs one.

## References

[^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^2]: Recurring defect shapes, shape 3. `.claude/rules/recurring-defects.md`
[^3]: Backlog item 0043. `docs/backlog/proposed/0043-decide-how-a-cell-is-repaired.md`
