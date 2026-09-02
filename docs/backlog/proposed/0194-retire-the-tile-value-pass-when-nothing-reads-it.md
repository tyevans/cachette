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

**No reader decides anything from it any more.** It had three. The viewer read
it for the colour of a tile, and item 0188 changed the colour to the food
stock.[^2] The level 1 summary averaged it and the `forage` option scored that
average; items 0183 and 0184 replaced that with the food of the cell.[^3] [^4]
No option row names the stub field, so the pass now computes a number that
decides nothing, over every tile on every tick, at a target scale of 16.7
million tiles.

**What remains is not a reader.** The summary still accumulates the field and
folds it into the state hash. The world holds the column and exposes it
through two readers and a copy. The control plane receives it as an array.
Removing the pass means removing all of that, which is why this is an item and
not a deletion.

The decisions register holds the general question of how the project finds a
value that nothing reads. This item is the instance that will be ready
first.[^5]

## What the work might do

The shape is open. The field is hashed into the state, so removing it changes
every golden state hash, and the golden files must be written again.

The questions this item must answer before it is refined:

- Whether anything still reads the field, checked by a search of the whole
  tree rather than by a list of files somebody thought of. The match arm that
  scores the stub field is still in the option scorer with no option row that
  names it, and the item must say whether that arm goes with the field.
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
[^3]: Backlog item 0183. `docs/backlog/complete/0183-carry-the-food-of-a-cell-into-the-level-1-summary.md`
[^4]: Backlog item 0184. `docs/backlog/complete/0184-score-the-forage-option-against-food.md`
[^5]: Decisions register, DEC-074. `docs/DECISIONS.md`
