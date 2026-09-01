---
id: 0109
title: Decide how the world holds a condition that moves
status: proposed
created: 2026-08-31
implements: []
changes: []
creates: []
serves: [PRD-0004]
blocked-by: [BLK-007]
---

## Why

The project commits to a world that changes on its own. The world must hold
at least one condition that varies over the map and over time, that no unit
acts on, and that a rule advances each tick.[^1]

The engine holds no such condition. Terrain is fixed for the life of a
world, so every situation a unit meets was placed by the generator.

The register settles the shape. A procedural base carries the cheap part,
and a simulated perturbation buys the feedback the project needs. The
register also states that the project builds this only if it builds
weather.[^2] The product record is now accepted, so the project builds it.

This decision needs a record because a future contributor could reasonably
choose a stored field instead, and because the choice governs determinism.
A condition that combines under any order is a determinism constraint, and
determinism is the one property this project cannot recover.

## What is missing before this is refined

- **The record number is not allocated.** Only the registry allocates it,
  and this item does not hold review rights over the registry.[^3] Add the
  row before this item moves to `refined/`.
- **The condition is not chosen.** Rain, wind, temperature and snow are
  candidates. The product record needs one that works, not a catalogue. The
  review chooses one and says why.
- **No measurement exists on the target platform.**[^4] The record states a
  cost shape, not a figure.

## Done when

Not yet stated. The item is not refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Product record PRD-0004. `docs/product/accepted/prd-0004-the-world-has-weather-that-a-watcher-can-read.md`
[^2]: Decisions register, DEC-006. `docs/DECISIONS.md`
[^3]: ADR Registry. `docs/adrs/REGISTRY.md`
[^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
