---
id: 0107
title: Decide how a faction stores what it observes
status: complete
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

**Closed as already done. The decision this item asks for is written and
accepted.** An audit read the registry and the records on 5 September 2026.

**The decision is that fog storage grows with the observed area.** ADR-0059
holds it and the registry gives that row the status `Accepted`. A second
accepted record holds the observation table.[^6] [^7]

**This item asked for a decision, and nothing else.** Its scope is the choice of
how a faction stores what it observes. That choice is made.

**Read this closure narrowly. It does not close fog of war.** No code implements
the decision. A search for the word over the core crate returns one footnote and
no store, no reader and no pass. A reader who finds an accepted record about fog
would reasonably assume something stores fog, and nothing does.

**Item 0108 holds the observation pass**, and it stays open. Its one
prerequisite, the storage decision, is now satisfied, so it can be refined. The
observation plane is the last gate before a learner can be trained, because the
harness must never show a learner anything a player of that faction could not
see.[^8]

## References

[^1]: Product record PRD-0001. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
[^2]: Decisions register, DEC-004. `docs/DECISIONS.md`
[^3]: ADR Registry. `docs/adrs/REGISTRY.md`
[^4]: Blockers register, BLK-001. `docs/BLOCKERS.md`
[^5]: Blockers register, BLK-005. `docs/BLOCKERS.md`
[^6]: ADR-0059, fog storage grows with observed area. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
[^7]: ADR Registry, rows 0059 and 0154. `docs/adrs/REGISTRY.md`
[^8]: Backlog item 0108, let a unit observe the tiles around it. `docs/backlog/proposed/0108-let-a-unit-observe-the-tiles-around-it.md`
