---
id: 0112
title: Build a world without a pass over every tile
status: proposed
created: 2026-08-31
implements: []
changes: []
creates: []
serves: [PRD-0003]
blocked-by: []
---

## Why

The product record for the ground states a cost shape: building a world must
not cost a pass over every tile before the first frame, so a developer who
changes a seed sees the new world at once.[^1]

The ground itself meets this. It allocates nothing and computes a tile only
when a reader asks for it.

The world does not. `World::new` loops once for each tile, draws a random
value for each one, and pushes into two vectors sized to the tile count. At
the target count of 16.7 million tiles that is a pass over the whole world
and two allocations proportional to it, paid before anything is drawn.

The two columns belong to the tile stub, not to the ground. The record's
statement is nevertheless false of the engine today, and a record the code
contradicts is worse than no record.[^2] This item was found by reviewing
the record against the code, and the record stayed in `accepted/` because of
it.

A second item already covers one of the two columns. It removes the stub
faction column, because a tile carries two values that name a faction and
the stub one means something different from the holder.[^3] This item covers
the remaining stub value column and the pass itself.

## What is missing before this is refined

- **The reader of the stub value is not surveyed.** The drawing pass reads
  `world.tile_values()`, and the state hash reads the stub columns, so every
  golden file moves. The review must name every call site from a whole-tree
  search, not from a list somebody thought of.[^4]
- **The governing records are not named.** The review must say which
  accepted records govern the tile stub and the state hash, and whether
  removing a column supersedes any of them.
- **The order between this item and 0084 is not decided.** Doing them
  together moves the golden files once instead of twice.

## Done when

Not yet stated. The item is not refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Product record PRD-0003. `docs/product/accepted/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
[^2]: Definition of Done. `.claude/rules/definition-of-done.md`
[^3]: Backlog item 0084. `docs/backlog/refined/0084-give-a-tile-one-faction-column.md`
[^4]: Commit Message Rules, after a sweep. `.claude/rules/commits.md`
