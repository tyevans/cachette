---
id: 0267
title: Hold the exit direction on the tile
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

**A unit reads its direction through a chain of four lookups.** It reads its
own tile, turns the tile into a layout key, turns the key into a block, and
reads the cell summary of that block. Each step lands somewhere the previous
step did not predict, and a review of the choice pass confirmed all four
against the source.[^1]

The direction is one value for each cell and each option. Nothing stops the
engine from writing it once for each tile instead, at the moment the field is
derived. A unit then reads one array at its own tile index and the chain
disappears.

**The trade is space for time, and the space is available.** A world at the
target scale holds 876 MB resident on a machine that holds 32 GB, so the
engine uses about three percent of the target machine.[^2] One byte for each
tile and each option is about 67 MB.

**This does not change what the field means.** The exit field is still derived
from the lattice, one direction for each cell, and the cost of deriving it
still follows the cell count and not the population.[^3] This item changes
where the answer is stored for reading, not how it is computed.

## What the work does

Write the derived direction to a tile-indexed array at the same moment the
field is derived. Read it from the tile in the movement pass. Keep the
cell-indexed field as the thing that is computed.

## What good looks like

The movement pass reads one array at the unit's tile index. A benchmark on the
target platform shows the unit passes cost less, and shows the derive cost
grew by the write. The determinism tests pass at every thread count, because a
tile-indexed write from a cell-indexed derive must have a fixed order.

## What it costs at the target scale

About 67 MB, as one byte for each of 16,777,216 tiles and each of four
options. The derive gains a write over every tile, which is a pass over the
tile count that the derive did not have. **That cost is the risk of this item
and it must be measured, not assumed.** A derive that now costs a tile pass
may lose more than the movement pass gains.

## What it does not do

It does not decide whether the memory trade is open in general. One open row
holds that.[^4]

It does not change the derive, the tie-break order, or any property the field
records state.[^3]

## References

[^1]: Findings register, FND-252. `docs/FINDINGS.md`
[^2]: Target platform costs, the resident memory rows. `docs/reference/graviton-costs.md`
[^3]: ADR-0091, movement takes its direction from a per-cell field. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^4]: Decisions register, DEC-105. `docs/DECISIONS.md`
