---
id: 0109
title: Decide how the world holds a condition that moves
status: complete
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
- **One blocker holds the cost figures this record would state.**[^4] The
  record states a cost shape, not a figure.

## Done when

Not yet stated. The item is not refined.

## Outcome

**Closed as already done. The decision this item asks for is written.** An audit
read the registry and the records on 5 September 2026.

**The condition is water, and it moves over the level 1 cell lattice.** Two
records hold that choice: one fixes the field on the lattice, and one states
that a pass moves the water and never scales it. The registry allocated both
rows.[^5] [^6]

**This item asked for a decision, and nothing else.** That choice is made, and
the passes that follow from it exist under items 0110 and 0111.

**Both records are still `Draft`.** Acceptance is a review step, and the record
priority index holds it. It is not work this item can do, because an author may
not accept a record.

## References

[^1]: Product record PRD-0004. `docs/product/accepted/prd-0004-the-world-has-weather-that-a-watcher-can-read.md`
[^2]: Decisions register, DEC-006. `docs/DECISIONS.md`
[^3]: ADR Registry. `docs/adrs/REGISTRY.md`
[^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^5]: ADR-0140, weather is a field over the level 1 cell lattice. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
[^6]: ADR-0141, a weather pass moves water and never scales it. `docs/adrs/draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md`
