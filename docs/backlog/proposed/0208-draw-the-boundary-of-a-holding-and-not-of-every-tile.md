---
id: 0208
title: Draw the boundary of a holding and not of every tile
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: [PRD-0006]
blocked-by: []
---

## Why

A held tile takes a mix of its holder's colour over the ground. A held tile
whose neighbour has another holder takes a border in nearly the pure colour, so
that a watcher sees where one holding meets another.[^1]

**The border carries information only while holdings form regions.** When
several factions interleave at tile scale, almost every held tile borders a
differently held one. Almost every tile then draws its border, the borders
cover a large part of the picture, and the ground is lost between them. The
border is no longer a border. It is a second fill.

The finding holds the measurement, and it records a repair that was tried and
removed: scaling the border weight with the tile size improves the picture and
does not touch the cause, because the driver is the density of the holdings and
not the zoom.[^2]

## What the work might do

The shape is open. The border should mark the outer edge of a contiguous
holding rather than the edge of every held tile.

The questions this item must answer before it is refined:

- What "contiguous" means when the drawing pass visits tiles in window order
  and never walks a region. A pass that grew a region would be a pass over the
  world, which the panel record forbids for the reporting and which the
  drawing already avoids.[^3]
- Whether the engine already holds what is needed. The holding module reports
  the holder of a tile and the faction mask near an address, and a boundary may
  fall out of the mask without a walk.
- Whether the answer is a border at all. A watcher may read holdings better
  from the fill alone at low zoom, with the border appearing only where a
  reader is looking closely.
- What a test asserts. A test needs a world whose holdings interleave and a
  world whose holdings form regions, and it must distinguish them.

## Done when

Filled in when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: PRD-0006, a place belongs to somebody. `docs/product/accepted/prd-0006-a-place-belongs-to-somebody.md`
[^2]: Findings register, FND-201. `docs/FINDINGS.md`
[^3]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
