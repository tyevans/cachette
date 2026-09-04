---
id: 0401
title: Decide how a sent unit gets around a barrier the field cannot see
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0020]
blocked-by: []
---

## Why

The control plane sends a set of units to a set of tiles. The engine seeds a
field at those tiles, spreads a reach outward over the level 1 cells, and every
unit of the set reads the direction of its own cell and steps.[^1]

**A unit stopped at a shoreline and never arrived.** It was not frozen. It took
a step on almost every frame, it wandered inside the cell it was stuck in, and
it never crossed the water in front of it. The finding holds the measurement.[^2]

**The cause is the pitch of the field and not a defect.** A cell covers a block
of tiles. The field says which way a block should go, and the ground of one tile
of that block is not a fact the block carries. A record states that consequence
and says plainly that a rule which routes a unit around an obstacle is a
different claim, which needs a field that reaches further than one
neighbour.[^3]

So the verb is honest and narrower than a caller expects. A god who orders a
congregation to a mountain across a lake watches it walk to the shore and stay
there.

## What is missing before this can be refined

- Whether the answer is a second field at the pitch of a tile, a rule that
  removes a cell from the reach when its own ground blocks the direction it
  holds, or nothing at all.
- What the answer costs. A field at tile pitch follows the tile count, and the
  blocker that governs every cost figure is open.[^4]
- Whether a caller should be able to read that a set is stuck. Nothing today
  reports it, and a caller sees movement and no progress.
- Whether the same answer serves the return field, which has the same shape and
  the same barrier.[^5]

## References

[^1]: ADR-0125, the control plane names the seed set of a destination field, decision D1. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
[^2]: Findings register, FND-411. `docs/FINDINGS.md`
[^3]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, the consequences. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^5]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decision D1. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
