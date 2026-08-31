---
id: 0031
title: Record the unconditional bridge rebuild, or make it conditional
status: complete
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

## Impact review

**Governed by.** ADR-0018 D3 sanctions a rebuild each frame and argues from
the merge order of incremental writes, not from frequency. It neither requires
the rebuild nor forbids the test.

**Changes.** No record changes.

**Creates.** No record. The work removes the choice rather than recording it.

**Blockers.** BLK-007 governs every cost figure, so this item states none.

## Outcome

The step removed the choice rather than recording it.

The admission work had already added a second rebuild site, at the top of the
step, guarded by a comparison of the arena revision. The tree therefore held
one conditional rebuild and one unconditional one, for one operation. Both now
go through one function and one rule: rebuild when the arena has moved since
the last rebuild, and not otherwise.

**A structure that already describes the arena is the structure a rebuild
would produce**, so the skip trades no guarantee. The barrier rebuild is still
last in the step, and the ordering ADR-0018 D3 states is unchanged.

**No saving is claimed.** A frame in which any unit moved rebuilds as before,
and in a world that is not stalled every frame moves someone. The value of the
change is one rule where there were two.

A test drives the stalled path: a world of one tile, whose only tile has every
neighbour outside the extent. No draw ever names a tile, the arena never
moves, and the derived structure must still answer after eight frames.

**A one-tile island does not exist in this generator.** The record pictures a
unit hemmed in by water; the ground is coherent and makes no such tile in a
world of nine thousand. The world of one tile is the shape that exists.

