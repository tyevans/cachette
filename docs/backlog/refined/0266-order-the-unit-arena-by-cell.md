---
id: 0266
title: Order the unit arena by cell
status: refined
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0002]
blocked-by: [BLK-007]
---

## Why

**The unit arena holds units in the order they spawned, and nothing ever
reorders it.** A slot is half of an identity, so a slot never moves. Two units
that stand in one cell therefore sit far apart in every unit column, once the
arena has drifted away from the order it was filled in.

**The figure this item was written against was the wrong figure.** The item
first claimed the arena order costs 2.11 times the unit cost at 12 threads,
from the packed and scattered rows of the target platform register. Both of
those rows spawn in ascending tile order, so the arena is in cell order in
both, and they differ in the density of the population and not in the arena
order. A finding records the correction and the fixture that separates the two
variables.[^1]

**The measured cost of a drifted arena is 1.24 to 1.45 on the unit half**, at
one and four threads, on a development machine that other work shared. A
figure from that machine is not evidence about the target platform.[^2]

**Half of that cost was the order of the walk, and that half is done.** The
movement pass walked the arena in slot order and read four tile-side values
for each unit, so a drifted arena made those reads random. It now walks the
order the bridge already holds, which is block-major tile order, and the reads
are ascending. The change cost the frame nothing, because the bridge sorts on
that key at the barrier whatever this pass does.[^3] A second finding records
it, and records that its gain could not be measured on the shared
machine.[^4]

**What is left is the residual: the unit columns themselves.** A pass in cell
order still reads them at scattered positions, because they are indexed by the
slot.

## Impact review

**Governed by.** ADR-0014 D1 holds that an identity is a slot index and a
generation, and its consequences hold that the engine can never compact the
slot index space. ADR-0014 D7 holds that the location table is a dense array
indexed by the slot. ADR-0012 D3 holds that every entity lives in the arena.
ADR-0004 D1 holds that iteration order is explicit. ADR-0009 D1 holds that
parallel stages write disjoint outputs.

**The work does not compact the slot index space, and must not.** It separates
the slot space, which holds the generation and never moves, from a row space,
which holds every payload column packed and in cell order. The identity
resolves through one more dense array indexed by the slot, which is what D7
already describes. A caller that starts from an identity then pays one more
dependent load, and that load is on the hot path.

**Changes.** None expected. If the indirection turns out to contradict a
reading of ADR-0014, that record is superseded rather than edited.

**Creates.** A record for the two-space arena, if the work proceeds. The
three-condition test passes for it: a contributor could reasonably index the
columns by the slot, the choice reaches every pass and every column accessor,
and the reasoning is not visible in the code. The registry row is allocated
before the work starts, not now, because the decision to do the work at all is
still open.

**Blockers.** BLK-007 governs it. No measurement of the residual exists on the
target platform, and neither does one of the reorder in the same process. The
reorder itself is priced on the development machine at 64 milliseconds for one
million units on one thread, against a frame budget of one hundred. **One side
of the comparison is measured and the other is not, and the unmeasured side is
the one that would justify the work.** The decisions register holds the
choice.[^5]

**Serves.** PRD-0002.

## What the work does

Give the unit columns an order that follows the cell, and keep it. Decide
whether the order is maintained on every change or rebuilt on an interval, and
say which in the implementation. A reorder of a packed arena costs 26
milliseconds and saves nothing, so an unconditional reorder on every frame
wastes the common case.

An entity identity must survive the move. Every column keyed on the arena slot
moves with it, and so does the reverse lookup.

## What good looks like

A pass over the units of one cell reads a contiguous run. A test proves that a
unit keeps its identity across a reorder. The determinism tests pass at every
thread count, and the golden state hash does not move, because a reorder
changes where things sit and nothing else.

**The measurement comes before the refactor, not after it.** A run on the
target platform states the residual and the reorder in one process. If the
residual is smaller than the reorder, this item closes with that result and no
code.

## What it does not do

It does not change what any pass computes. It changes where the inputs sit.

It does not divide the residual of the stage split. One item holds that
work.[^6]

## References

[^1]: Findings register, FND-273. `docs/FINDINGS.md`
[^2]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^3]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D1. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^4]: Findings register, FND-274. `docs/FINDINGS.md`
[^5]: Decisions register, DEC-110. `docs/DECISIONS.md`
[^6]: Backlog item 0237, declare what each stage reads and writes. `docs/backlog/proposed/0237-declare-what-each-stage-reads-and-writes.md`
