---
id: 0269
title: Map the large arrays with huge pages
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

**The engine holds about 456 MB of tile arrays and reads them in an order
nothing predicts.** At the default page size of 4 KB that is more than one
hundred thousand pages. No translation buffer holds a working set of that
size, so a scattered tile read can pay a page walk on top of a cache miss.

**The target platform supports a larger page.** The engine can ask the
operating system to back a large allocation with 2 MB pages, which reduces the
number of translations by a factor of five hundred and twelve.

**This is the cheapest experiment the project has.** It changes no algorithm,
no data layout and no arithmetic. It cannot affect determinism, because it
changes where the memory sits and not what any pass computes or in what order.

**It may explain part of a cost nobody can name.** The stage split leaves 170
milliseconds of the unit cost unattributed, which is 62 percent of it.[^1] A
translation cost would appear in exactly that way: spread across every pass
that touches a large array, and invisible to a split that measures stages.

## What the work does

Ask for huge pages for the large allocations. Measure one frame at the target
scale on the target platform, with and without, and record both rows.

## What good looks like

The measurement register gains a row for each condition, on the same machine
and the same commit. The frame cost falls, or it does not, and the register
says which.

**A result of no change is a result.** This item is done when the figure
exists, not when the figure is favourable.

## What it costs at the target scale

Nothing in memory that the engine does not already hold. A huge page can waste
space at the end of an allocation, and at these sizes that waste is under one
part in two hundred.

## What it does not do

It does not change any data layout. Three other items do that, and each is
worth measuring on its own so that the results do not mix.[^2] [^3] [^4]

## References

[^1]: Target platform costs, the stage split. `docs/reference/graviton-costs.md`
[^2]: Backlog item 0266, order the unit arena by cell. `docs/backlog/refined/0266-order-the-unit-arena-by-cell.md`
[^3]: Backlog item 0267, hold the exit direction on the tile. `docs/backlog/proposed/0267-hold-the-exit-direction-on-the-tile.md`
[^4]: Backlog item 0268, hold the cell index on the unit. `docs/backlog/proposed/0268-hold-the-cell-index-on-the-unit.md`
