---
id: 0423
title: Measure what the conversion pass costs on the target platform
status: proposed
created: 2026-09-03
---

## Why

The step now runs a conversion pass on every frame. The pass opens a stage, so
the stage cost table names it and a benchmark reports it. Nobody has run that
benchmark on the target platform, so the cost of the pass is derived and not
measured.

The derivation says the pass costs the occupied tiles multiplied by the faction
count, plus one read for each unit on those tiles. Two terms of that are worth
checking against a measurement. One blocker governs every cost figure in this
project, and it says which are measured and which are derived.[^1] A register
holds what the target platform has measured so far.[^2]

**The faction count multiplies the per-cell read.** The pass reads what every
faction holds at a cell before it decides anything. A world at the faction
ceiling reads sixty-three values for each cell it meets. The reader holds the
last cell, so it pays that once for each cell rather than once for each tile,
and how well that holds depends on how the tiles of one block map onto cells.

**A frame that converts somebody rebuilds the derived unit structure.** The
faction setter raises the arena revision, so the refresh after the pass does a
full rebuild. That rebuild is the largest single stage in the engine. A world
whose field is still moving pays it on every frame until the field settles, and
nobody has measured how many frames that is. The record that decides the pass states the derivation in its own consequences.[^3]

## What it needs before it can be refined

A benchmark run on the target platform, with the conversion pass reached rather
than skipped. The existing benchmark world sets no influence source, so it
converts nobody and measures the walk alone. The measurement needs both cases:
a settled world that converts nobody, and a moving world that converts on every
frame.

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Target platform costs. `docs/reference/graviton-costs.md`
[^3]: ADR-0133, a unit converts to the faction that leads the influence field at its cell, the consequences. `docs/adrs/draft/adr-0133-a-unit-converts-to-the-faction-that-leads-the-field.md`
