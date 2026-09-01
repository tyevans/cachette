---
id: 0107
title: Decide how a faction stores what it observes
status: proposed
created: 2026-08-31
implements: []
changes: []
creates: []
serves: [PRD-0001]
blocked-by: [BLK-001, BLK-005]
---

## Why

The project commits to hidden information. A faction must see a tile only
when one of its own units observes that tile, and a faction that has never
observed a tile must read nothing about it.[^1]

Nothing in the engine holds that. Every reader sees the same world. No
backlog item tracked this need before this one.

The register settles the shape of the model. The project keeps two layers.
One layer says that a faction has observed a tile at some time. The other
says that a faction observes the tile now. The game shows the two
differently, so both are needed.[^2]

The storage is the hard part. A bit for each tile for each faction pays the
whole world for every faction that exists, and the need rejects that shape.
Storage must grow with the area a faction has observed.

## What is missing before this is refined

This item names a decision, not an answer. The impact review needs these
first.

- **The record number is not allocated.** Only the registry allocates it,
  and this item does not hold review rights over the registry.[^3] Add the
  row before this item moves to `refined/`.
- **Two blockers govern the values.** The tile scale is open, so a sight
  radius has no unit.[^4] The maximum faction count is open, so the cost
  argument stays parametric.[^5] Express both parametrically. Do not invent
  either.
- **The interaction with the pyramid is unresolved.** The need states that a
  level 1 and a level 2 answer must not leak what the tiles beneath them
  hide. Whether a faction reads its own projection, or reads the shared one
  through a filter, is the decision this record must make.

## Done when

Not yet stated. The item is not refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Product record PRD-0001. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
[^2]: Decisions register, DEC-004. `docs/DECISIONS.md`
[^3]: ADR Registry. `docs/adrs/REGISTRY.md`
[^4]: Blockers register, BLK-001. `docs/BLOCKERS.md`
[^5]: Blockers register, BLK-005. `docs/BLOCKERS.md`
