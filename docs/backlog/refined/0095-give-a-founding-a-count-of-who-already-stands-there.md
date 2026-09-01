---
id: 0095
title: Give a founding a count of who already stands there
status: proposed
created: 2026-08-31
implements: []
changes: []
creates: []
serves: [PRD-0012]
blocked-by: []
---

## Why

The founding fills each tile of its place to the capacity of that tile's
ground, and it reads no count of who already stands there.

The reason is in two register rows. A spawn does not read the capacity at
all, so a caller may over-fill a tile and the engine accepts it.[^1] The
derived structure that holds a tile occupancy is rebuilt at the frame barrier,
so between two frames it does not describe the arena, and the step is the
thing that refreshes it.[^2] A founding that read that structure would be a
third call site for a rebuild the step already owns.

The result is correct for the case the engine meets today. One founding, in an
empty world, fills nothing twice. Two foundings whose discs overlap can put a
tile above its capacity, and movement then only ever takes units off that tile,
because admission never raises a tile above its capacity.

This is acceptable and it is not invisible: it is a caller mistake that the
simulation absorbs. It stops being acceptable when the engine founds more than
one group, which is a separate item.

## What the work does

1. A founding knows how many units already stand on a tile before it adds one.
2. No tile leaves a founding above the capacity of its ground.
3. The answer does not add a third call site for the rebuild that the step
   owns.

## What is missing before this is refined

- **Which register row this closes.** The question is the spawn one, the
  barrier one, or both.[^1] [^2] Read them and say which.
- **Whether the answer is storage.** One row offers a dense occupancy count,
  one byte for each tile at the target scale, and calls it the storage
  decision that the bridge record defers.[^1] That is an architectural
  decision and it needs a record, not a register row.
- **Whether the founding can count for itself.** A founding touches one disc.
  It could carry its own count over that disc alone, at no storage cost and
  with no rebuild, and that would be correct for any number of foundings in
  one call. It would not be correct across two separate calls. Decide whether
  that is enough.

## Done when

- A test founds two groups whose places overlap and asserts that no tile ends
  above the capacity of its ground.
- The register rows say what this work settled.
- `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Open decisions register, DEC-020. `docs/DECISIONS.md`
[^2]: Open decisions register, DEC-021. `docs/DECISIONS.md`
