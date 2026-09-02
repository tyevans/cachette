---
id: 0171
title: Build the first level without a pass over every tile
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: [PRD-0003]
blocked-by: []
---

## Why

The product record for the ground states a cost shape: building a world must
not cost a pass over every tile before the first frame, so a developer who
changes a seed sees the new world at once.[^1] The record is `Accepted` and
the engine does not meet it.

Item 0112 removed one of the passes. The tile value field now generates a
tile when a reader asks for one, and building a world stores nothing for
it.[^2]

**The build still passes over every tile twice, and the pyramid makes both
passes.** The first level of the pyramid holds one summary for each block.
Building it reads the ground of every tile of every block. The world then
closes its build by rebuilding the moving part of every cell, and that sums
the tile value of every tile.

A third cost is a proportional allocation rather than a pass. The holder
column is one entry for each tile, and the world allocates it when it is
built. That column is a dense column by decision, so it is not a defect. It
is named here because a reader who measures the build will find it.[^3]

## What is missing before this is refined

- **The repair is not chosen.** Two shapes are available and neither is
  free. A cell can be built when a reader first asks for it, which moves the
  cost to the first read rather than removing it. A cell can also be derived
  from a coarser generated quantity, which needs a summary the generator can
  give without visiting a tile. Read the level record before choosing.[^4]
- **The barrier rebuild is a separate question from the build.** A rebuild at
  every barrier is a whole-world sweep on every frame, and the ground record
  calls such a sweep a design mistake.[^5] The item must say whether it
  covers the barrier or only the build.
- **The governing records are not named.** The review must name the records
  that govern the pyramid and the level 1 rebuild.

## Done when

Not yet stated. The item is not refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: PRD-0003, a developer sees a world worth looking at, what it costs at the target scale. `docs/product/accepted/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
[^2]: Backlog item 0112. `docs/backlog/complete/0112-build-a-world-without-a-pass-over-every-tile.md`
[^3]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^4]: ADR-0022, level 0 is the only truth, and every level above it is derived. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^5]: ADR-0068, terrain is generated from the seed and is never stored as a map, the consequences. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
