---
id: 0043
title: Decide how a level 1 cell is repaired
status: proposed
created: 2026-08-31
---

Level 1 rebuilds every cell every frame. The registry reserves row 0025 for the
claim that the pyramid carries two update paths chosen by a threshold, and the
note that closed the record-writing item said that row waited on a measurement,
because a cost decision with no figure is a record that states an intent.

There is a figure now. A full rebuild costs about 2.6 milliseconds for a world
of 281 600 tiles, on a development machine, in the step that already costs
about 5 milliseconds without it. The commit that added level 1 holds the table.

Three paths, and the impact review must weigh them.

**Rebuild every cell.** What the code does. Its cost follows the world and not
the change, so a frame that moved one unit pays for the whole world.

**Repair the cells that changed.** The combine operation has an inverse, and
the code already exposes it, so a cell can be repaired by removing the old
contribution and adding the new one. This needs a record of which cells
changed, and the research report warns against a per-tile dirty bitset and
recommends tracking dirtiness per chunk.

**Fold the sweep into the stage that already runs.** The tile update already
walks every tile in parallel. A per-thread partial summary, merged by cell
index, would cost one sweep rather than two. The merge is addition, which is
commutative, and the key is an index rather than a thread, so the result stays
independent of the thread count.

Do not refine this before a measurement exists on the target platform.[^1] The
figure above is from a development machine and the cache line differs.

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
