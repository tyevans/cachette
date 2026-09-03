---
id: 0266
title: Order the unit arena by cell
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

**The unit arena holds units in the order they spawned, and nothing ever
reorders it.** Every unit pass then reads the tile column, the block of the
tile, the cell of the block and the need column at a position that the spawn
order fixed. Two units that stand in one cell sit far apart in memory.

**This is the largest measured cost in the engine that no item holds.** A
benchmark measured one frame twice at the target scale, once with the units
packed and once with them scattered, on the target platform. The unit cost
was 2.11 times higher scattered than packed at 12 threads. The whole frame
was 1.59 times higher. The register holds every row.[^1]

**The same layout explains why the unit passes do not scale.** They reach a
speedup of 1.88 at 12 threads and 1.85 at 16, so a larger machine makes them
worse. A review of the pass found that it obeys the rule about disjoint
parallel writes completely, and floors anyway, because it collects every live
unit serially, walks the result serially, and reads scattered positions in
between.[^2] [^3]

**The prize is not only speed.** A decision that follows the lattice asks for
the units of one cell together, and an arena in spawn order cannot supply
them without a gather.[^4]

## What the work does

Give the arena an order that follows the cell, and keep it. Decide whether
the order is maintained on every change or rebuilt on an interval, and say
which in the implementation.

An entity identity must survive the move. Any column keyed on the arena slot
moves with it, and the reverse lookup moves with it.

## What good looks like

A pass over the units of one cell reads a contiguous run. The packed and
scattered figures converge, because the engine no longer has a scattered
case. A test proves that a unit keeps its identity across a reorder, and the
determinism tests pass at every thread count.

## What it costs at the target scale

The reorder itself is a cost that the frame did not carry before, and this
item must measure it rather than assume it is small. A reorder of 1,000,000
units that runs every frame could exceed what it saves.

## What it does not do

It does not change what any pass computes. It changes where the inputs sit.

It does not divide the residual. The stage split leaves 170 milliseconds of
the unit cost unattributed, and one item holds that work.[^5]

## References

[^1]: Target platform costs, the packed and scattered rows. `docs/reference/graviton-costs.md`
[^2]: ADR-0009, parallel stages write disjoint outputs. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^3]: Findings register, FND-252. `docs/FINDINGS.md`
[^4]: ADR-0096, cost follows the lattice, not the population. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
[^5]: Backlog item 0237, declare what each stage reads and writes. `docs/backlog/proposed/0237-declare-what-each-stage-reads-and-writes.md`
