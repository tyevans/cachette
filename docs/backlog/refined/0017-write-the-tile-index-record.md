---
id: 0017
title: Write the tile index record for a rhombus world
status: refined
created: 2026-08-30
implements: []
changes: []
creates: [ADR-0017]
serves: [PRD-0002]
blocked-by: []
---

## Why

Every system in the engine reaches a tile through an index. The index shape
is the one decision that every later storage decision inherits, and a future
contributor could reasonably choose otherwise, because offset coordinates are
the common choice for a hex grid.

The project chose a rhombus world, so a tile address is a raw axial pair and
a tile access needs no coordinate conversion. That is a constraint a reviewer
can find a violation of: a conversion function in a hot path is the
violation.

## Impact review

**Governed by.** ADR-0002 D1 makes every coordinate an exact integer.
ADR-0004 D1 requires an explicit stable order, and a tile index is what makes
tile order stable.[^1] [^2] Registry row 0016 says tiles are stored in
block-tiled order at the aggregation block size, and this record must not
contradict it: the axial pair is the address, and the block-tiled order is
how addresses are laid out in memory.[^3]

**Changes.** None. Row 0017 is `Proposed` with no file. Item 0016 corrects
the row text before this item writes the file.

**Creates.** ADR-0017. The number is already allocated.

**Blockers.** BLK-014 is answered by item 0016 and must be closed before this
record is written. The record states the shape as a decision and cites the
blocker row for the reasoning. BLK-007 governs any cost figure, so the record
states none.[^4]

**Precedent.** FND-040 records that a reused decision number sends a reader to
the wrong claim. This record is new, and the number was never accepted under
its old claim, so nothing cites the old text.[^5]

**Serves.** PRD-0002.

## Done when

- The record states one claim: the world is a rhombus, so a tile index is a
  raw axial pair and no tile access converts a coordinate.
- The record has numbered decisions, so a later record and a source file can
  cite one.
- The record states the forces, the alternative rejected, and the
  consequences. The rejected alternative is odd-r offset indexing, and the
  record says what it costs and what it buys.
- The record states a consequence for the renderer: a rhombus is a
  parallelogram on the screen, so the viewer applies the skew, and the engine
  does not.
- The record holds no cost figure, no version, no count and no module
  arrangement.
- The registry row moves to `Draft`, which an author may set.
- The record check passes.

## Outcome

Filled in on completion.

## References

[^1]: ADR-0002, simulated and aggregated state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^2]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^3]: ADR Registry. `docs/adrs/REGISTRY.md`
[^4]: Blockers register. `docs/BLOCKERS.md`
[^5]: Findings register, FND-040. `docs/FINDINGS.md`
