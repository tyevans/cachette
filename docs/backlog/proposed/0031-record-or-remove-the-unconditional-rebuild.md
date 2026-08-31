---
id: 0031
title: Record the unconditional bridge rebuild, or make it conditional
status: proposed
created: 2026-08-31
---

The step rebuilds the whole bridge every frame, whether or not any soldier
moved. ADR-0018 D3 sanctions a per-frame rebuild, but it argues from the
merge order of incremental writes, not from frequency. It does not consider
the case where the arena revision has not moved since the last rebuild, which
the bridge already tracks and could test in one comparison.

The finding is not that the rebuild is expensive. No figure is measured, and
BLK-007 governs every cost figure in this project. The finding is that
"rebuild unconditionally" and "rebuild when the revision moved" are two
options, the code chose one, and nothing records the choice. A future
contributor could reasonably choose otherwise, which is the first of the
three conditions for needing a record.

Either add the comparison, or record the choice. A line in a backlog item is
enough if the project judges the claim below the threshold for a record.

Found by the review of item 0020.
