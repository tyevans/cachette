---
id: 0268
title: Hold the cell index on the unit
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

**Every unit pass recomputes which cell a unit stands in.** It reads the unit's
tile, turns the tile into a layout key, and turns the key into a block. A
review of the choice pass named these two steps as two of four scattered
reads.[^1]

**That naming is wrong, and a measurement says so.** The two steps read no
memory. They are one remainder and one quotient by the world width, and the
width is a runtime value, so both are a hardware division. The division is the
largest single part of the chain, and reading a stored cell index costs a small
fraction of deriving one.[^5]

The answer changes only when the unit moves. The engine recomputes it on every
pass that needs it, on every frame.

**The same measurement refused the item that sat above this one.** That item
proposed a tile-indexed exit direction, and it lost on both halves of its
trade.[^6] This item is not the smaller half of that win. It is the part of the
chain that costs, and it costs 4 MB rather than 67 MB.

**The trade is space for time, and the space is small.** One cell index for
each of 1,000,000 units is about 4 MB. A world at the target scale holds
876 MB on a machine that holds 32 GB.[^2]

## What the work does

Store the cell index in the unit columns. Write it where the unit's tile is
written, so the two cannot disagree without one write being wrong.

## What good looks like

A unit pass reads the cell index rather than deriving it. A test proves the
stored index and the derived index agree for every live unit, and the
invariant pass fails when they do not.

## What it costs at the target scale

About 4 MB. A unit costs 89 bytes today, so this is a small addition to the
smaller of the two large allocations.

## The risk this item carries

**This is a second declaration site for one value, which is the defect shape
this project records most often.**[^3] The tile is the truth and the cell
index is derived from it. A write that updates one and not the other is
silent, and the value is read by the pass that decides where a unit goes.

The item is worth doing only with a check that fails when the two disagree.
That check is not optional and it is not a follow-up. Write it first.

## What it does not do

It does not decide whether the memory trade is open in general. One open row
holds that.[^4]

## References

[^1]: Findings register, FND-252. `docs/FINDINGS.md`
[^2]: Target platform costs, the resident memory rows. `docs/reference/graviton-costs.md`
[^3]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^4]: Decisions register, DEC-105. `docs/DECISIONS.md`
[^5]: Findings register, FND-282. `docs/FINDINGS.md`
[^6]: Findings register, FND-281. `docs/FINDINGS.md`
