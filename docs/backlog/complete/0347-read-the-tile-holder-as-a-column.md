---
id: 0347
title: Read the tile holder as a column
status: complete
created: 2026-09-03
implements: [ADR-0053]
changes: []
creates: []
serves: [PRD-0031]
blocked-by: []
---

## Why

The control plane reads who holds a tile one address at a time, as one entry of
a report built for that address. The engine holds the holders as one dense
column over the tiles, and it holds one faction mask for each block.[^1]

So a caller who wants to draw a map of who holds what, or to ask about a
region, walks the world from Python. That is the loop the boundary rule
forbids, and the engine already returns tile values as one array, so the shape
of the answer exists.

This is smaller than the presence relation and it does not replace it. The
relation answers whether anybody is present. This answers where the ground is.

## Impact review

**This is a separate change from the presence relation, and the two share no
code.** The presence relation reads the holder column inside Rust and never
crosses it. This item copies the column across the boundary. They were judged
together and built together because they serve one product record, and each
stands alone.

**Governed by.** ADR-0053 D2 gives a tile one holder, which is a faction or
nobody, and makes exclusivity a property of the storage. ADR-0040 D1 and D2
forbid a crossing for each entity. ADR-0044 asks a call site to declare whether
it copies. ADR-0107 D2 puts the prose of a published member in the Rust doc
comment.

**Creates.** No record. The three-condition test fails at the first condition.
A dense column already exists, and returning it as one array is the only
workable shape for a whole-world read. Nothing about the choice needs the right
to refuse a change.

**The three open questions from the proposal are answered.**

The call returns the whole world and never a window. The engine already returns
the tile value column that way, so the two arrays index alike. The radius
ceiling of a window census guards a scan that derives a count; this call copies
a column that already exists, so the same argument does not apply.

The column names nobody with 65535. A faction number counts from zero and the
world holds at most 63 factions, so 65535 can never name one. A separate null
value would be a second declaration of the same fact.

The selector does not absorb it. A selector returns a described set of
entities, and this is a dense array over the tiles. Building it now does not
build the same thing twice.

**Blockers.** None. BLK-007 governs every cost figure, so this item states
none.

**Serves.** PRD-0031.

## What the work does

Copy the holder column across the boundary as one array of unsigned 16-bit
integers, one entry for each tile, in row-major order.

## What good looks like

A new world reads as all nobody. A world in which a faction holds ground reads
that faction's number at those tiles. The column and the single-address report
agree, because they are two statements of one fact. The call raises nothing,
because a holder changes only inside a step.

## What it does not do

It does not answer for a window. It does not say what stands on the ground.

## References

[^1]: Findings register, FND-361. `docs/FINDINGS.md`
