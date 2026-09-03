---
id: 0269
title: Map the large arrays with huge pages
status: complete
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

Measure one frame at the target scale on the target platform, with and without
huge pages, and record both rows.

**The measurement changed the kernel setting rather than the engine**, and
that was the cheapest form of the question. The same binary, from the same
commit, on the same machine, in three processes, with the kernel backing its
large anonymous mappings either at 4 kB or at 2 MB. A code change would have
answered the same question and would have cost an allocation path that another
item now holds.[^5]

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

**That estimate was wrong by a factor of five.** The measured resident set grew
by 27,889,664 bytes, which is 2.65 percent. The register holds the figure and a
finding holds the correction.[^1] [^6]

## What it does not do

It does not change any data layout. Three other items do that, and each is
worth measuring on its own so that the results do not mix.[^2] [^3] [^4]

## Outcome

**A frame costs 3.9 percent less on huge pages, and the saving sits in three
stages.** The register holds every figure, the machine, the commit and the
huge page setting each row was taken under.[^1]

The frame fell from 835,978,143 nanoseconds to 803,042,781 at the target
scale, at 12 threads, with the units scattered. The change merge, the holding
spread and the barrier bridge rebuild account for the whole of it, and every
other stage moved by less than half a millisecond. **Those are the three
passes that write scattered over a large array**, which is where this item
predicted a translation cost would appear.

**The setting reached the process**, and the register says so with a figure
rather than an assumption: 719,323,136 bytes of the resident set sat on huge
pages, which is 66.6 percent of it. Under both other settings it was zero.

**This is one run for each condition.** Two settings the engine cannot tell
apart differ by 0.64 percent, so read that as the noise of the apparatus. The
effect is about six times it.

**The item is answered and the saving is not captured.** A machine setting is
not something the engine controls where it runs, so taking this 3.9 percent
portably means asking for the pages in the allocation. That is a separate
item, and it is separate because it touches the allocation path this item
promised not to.[^5]

**What made this item measurable was not this item.** The stage table came
first, and without it this run would have reported one number falling by 3.9
percent with no way to say where.[^7]

## References

[^1]: Target platform costs, the stage split. `docs/reference/graviton-costs.md`
[^2]: Backlog item 0266, order the unit arena by cell. `docs/backlog/refined/0266-order-the-unit-arena-by-cell.md`
[^3]: Backlog item 0267, hold the exit direction on the tile. `docs/backlog/complete/0267-hold-the-exit-direction-on-the-tile.md`
[^4]: Backlog item 0268, hold the cell index on the unit. `docs/backlog/proposed/0268-hold-the-cell-index-on-the-unit.md`
[^5]: Backlog item 0290, ask for a huge page in the allocation. `docs/backlog/proposed/0290-ask-for-a-huge-page-in-the-allocation.md`
[^6]: Findings register, FND-278. `docs/FINDINGS.md`
[^7]: Backlog item 0289, price every stage of a frame by name. `docs/backlog/complete/0289-price-every-stage-of-a-frame-by-name.md`
