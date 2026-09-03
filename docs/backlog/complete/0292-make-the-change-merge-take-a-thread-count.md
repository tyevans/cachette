---
id: 0292
title: Make the change merge take a thread count
status: complete
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

**One serial pass is 14 percent of a frame at the target scale.** The change
merge costs 120.5 milliseconds of an 836 millisecond frame, on the target
platform, at 12 threads. It is the second largest stage of a frame and the
largest one that takes no thread count.[^1]

The pass joins the tile changes that the parallel tile scan produced, sorts
them by tile index, and merges one ascending run into the stored field. The
scan that produced those changes runs on every thread and costs 16.5
milliseconds. **The pass that tidies its output costs seven times the pass
that produced it.**

**A serial pass bounds every thread count above it.** The tile half of a frame
scales well, and the register measures it at 6.13 times on 16 threads. A fixed
120 milliseconds inside it puts a floor under the tile half that no core count
reaches.

## What is missing before this is refined

- **Which part costs.** The stage covers three things: the join into one
  vector, the sort, and the merge into the stored field. Nothing has divided
  them. The stage table takes a nested row, so dividing them is cheap.[^2]
- **Whether the sort is needed.** The ranges each thread reads are disjoint
  and ascending, so each slot is already sorted. A merge of sorted runs is not
  a sort, and the current code sorts the concatenation.
- **Whether the merge can be partitioned.** The stored field is written by
  tile index. Two threads that write disjoint index ranges write disjoint
  memory, which is what the parallel record asks for.[^3]
- **What fixes the order.** Whatever replaces this must give the same field at
  every thread count, and the two determinism tests must be run against it.[^4]
- **Whether the pass should exist.** The tile value field writes a random walk
  over every tile on every tick and no reader decides anything from it. Item
  0194 proposes removing that, and removing it would remove most of what this
  pass merges.[^5]

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Target platform costs, every stage of a frame by name. `docs/reference/graviton-costs.md`
[^2]: Backlog item 0289, price every stage of a frame by name. `docs/backlog/complete/0289-price-every-stage-of-a-frame-by-name.md`
[^3]: ADR-0009, parallel stages write disjoint outputs. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^4]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^5]: Backlog item 0194. `docs/backlog/proposed/0194-retire-the-tile-value-pass-when-nothing-reads-it.md`

## Outcome

**Closed without being done, because the stage it names no longer exists.**

This item asked the change merge to take a thread count. Another item made
the tile value field a dense array, and a third let the tile scan's workers
write that array directly, so the merge, the run it merged and the join that
fed it were all deleted rather than threaded.[^OUT1] [^OUT2]

A stage the target platform measured at 14.4 percent of a frame does not
run. Threading it would have made it cheaper; removing it made it free.

**The general form is worth more than the item.** An item that names a stage
is an item that assumes the stage should exist. This one was written from a
cost table, and a cost table is a map of where time goes rather than of what
causes it.[^OUT3]

[^OUT1]: ADR-0103, the tile value field stores a dense delta. `docs/adrs/draft/adr-0103-the-tile-value-field-stores-a-dense-delta.md`
[^OUT2]: Target platform costs, the stage table. `docs/reference/graviton-costs.md`
[^OUT3]: Findings register, FND-292. `docs/FINDINGS.md`
