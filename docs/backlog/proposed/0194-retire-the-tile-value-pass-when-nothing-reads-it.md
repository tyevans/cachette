---
id: 0194
title: Retire the tile value pass when nothing reads it
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

The tile value field is a random walk. A pass draws a number for every tile on
every tick and adds one, subtracts one, or leaves the tile alone. No system
writes it for a reason and no system decides anything from it.[^1]

It had three readers. The viewer read it for the colour of a tile, and no
longer does: the colour now reads the food stock.[^2] The level 1 cell summary
averages it, and the `forage` option scores that average. Two items replace
both of those with food.[^3] [^4]

When those two land, nothing reads the field. A full pass over every tile on
every tick then computes a number that nobody looks at, at the target scale of
16.7 million tiles.

The decisions register holds the general question of how the project finds a
value that nothing reads. This item is the instance that will be ready
first.[^5]

## What the work might do

The shape is open. The field is hashed into the state, so removing it changes
every golden state hash, and the golden files must be written again.

The questions this item must answer before it is refined:

- Whether anything still reads the field, checked by a search of the whole
  tree rather than by a list of files somebody thought of.
- Whether the event log, which reports a tile change, has any other producer.
  If it does not, removing the field removes the only event the engine emits,
  and the Python control plane reads that log.
- Whether the census gate, which proves that building a world visits no tile
  of the value field, has a subject after the field goes.
- What replaces the field as the thing the state hash proves changes each
  tick.

## Done when

Filled in when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: What a unit does in a tick, section 1. `docs/research/what-a-unit-does-in-a-tick.md`
[^2]: Backlog item 0188. `docs/backlog/complete/0188-show-the-food-of-a-tile-and-the-reason-a-unit-chose.md`
[^3]: Backlog item 0183. `docs/backlog/proposed/0183-carry-the-food-of-a-cell-into-the-level-1-summary.md`
[^4]: Backlog item 0184. `docs/backlog/proposed/0184-score-the-forage-option-against-food.md`
[^5]: Decisions register, DEC-074. `docs/DECISIONS.md`
