---
id: 0095
title: Give a founding a count of who already stands there
status: refined
created: 2026-08-31
implements: [ADR-0074 D1, ADR-0074 D3, ADR-0074 D4, ADR-0074 D5, ADR-0074 D6]
changes: [ADR-0056 D3, ADR-0056 D4]
creates: []
serves: [PRD-0012]
blocked-by: []
---

## Why

The founding fills each tile of its place to the capacity of that tile's
ground, and it reads no count of who already stands there. One founding, in an
empty world, fills nothing twice. Two foundings whose discs overlap can put a
tile above its capacity, and movement then only ever takes units off that tile,
because admission never raises a tile above the capacity of its ground.

The project owner has answered the register row that held this open. A spawn
must refuse a tile that is at capacity, and the engine holds a dense occupancy
count of one byte for each tile at the target scale.[^1] The answer is against
the recommendation of the row, which was to let a spawn over-fill. The owner
chose deliberately, so this item implements the answer and does not reopen it.

The answer also settles a deferral. The movement record states that admission
reads the occupancy of a target tile from the derived unit-to-tile structure,
and that no record chooses between that and a dense array over every tile.[^2]
The dense count is that array. It replaces the derived read for admission and
it gives a spawn a constant-time check.

## Impact review

**Governed by.** The movement record binds this work in four places. D1 says a
unit occupies exactly one tile, so a count is the whole of the occupancy of a
tile.[^3] D3 fixes the admission order and states where admission reads the
occupancy from, and that clause is what this work replaces.[^2] D3 also states
that only an admitted departure releases room, and the dense count must obey
that: a rejected intent is not a departure, so nothing decrements on an
intent.[^2] D4 says capacity is a property of the terrain and that the engine
holds no capacity constant of its own, so the refusal reads the terrain table
and never a literal.[^4]

The bridge record binds the other half. It rejects an offset array over every
tile because the rebuild of that array repairs every entry once for each frame,
whether or not anything moved.[^5] A dense count is not an offset array. It
carries no structure that must be exact everywhere before a query is correct,
and it is maintained by the arrival and the departure rather than rebuilt. The
work must say so where a reader will meet the objection.

The determinism record binds the maintenance. The count is written by the
admission step and by a spawn, and a parallel write to one tile must give the
same answer at any thread count.[^6] Admission already sorts by target tile and
scans disjoint segments, so an arrival write is free of contention. A departure
is a separate reduction keyed on the source tile.[^2] A spawn runs outside the
frame and the barrier question governs when its write becomes visible.[^7]

**Changes.** The movement record, in two places: D3's clause that admission
carries no per-tile array of its own, and D4's paragraph that says no record
decides how the engine stores an occupancy count. The new record states that it
replaces both.[^8] The movement record keeps its number and its status. A
supersession replaces a claim, not a file, and the registry annotation follows
acceptance rather than the draft.

**Creates.** No record. The record this work needed now exists as a draft, and
it states that a tile occupancy count is stored densely and that a spawn
refuses a tile at capacity.[^8] The registry holds its number and its
status.[^11] The record is a draft, so a reviewer must accept it before this
work merges, and the reviewer must not be the agent that wrote it.

**Blockers.** No blocker governs a value this work needs. The capacity values
come from the terrain table and the scale constants hold them.[^9] The cost of
the array at the target scale is derived, not measured, like every cost figure
in this project.

**Precedent.** Two accepted records already disagree about whether a per-tile
count exists. The bridge record rests part of its rejection of the offset array
on a per-tile array of counts that already exists, because admission needs the
occupancy of a target tile and its departure count in the same tick.[^5] The
movement record says admission carries no per-tile array of its own.[^2] This
work makes the first one true, and the new record states that repair.[^8] Say
so in the commit body as well, because a reader who finds the two claims will
otherwise read the new record as the cause of the conflict rather than the end
of it.

**Precedent.** One value in two places, with nothing that fails when the copies
disagree, is the shape this project meets most often. The dense count is a
second declaration of where units stand: the derived structure is the first. A
check must fail when the two disagree, and a comment that names the winner is
not that check.

## What the work does

1. The engine holds a count of the units that stand on a tile, one byte for
   each tile, and it is dense over the world.
2. A spawn reads that count and refuses a tile that is at the capacity of its
   ground. A refusal is an outcome the caller can see, not a silent drop.
3. Admission reads and writes that count instead of reading the derived
   structure, and it keeps the rule that only an admitted departure releases
   room.[^8]
4. A founding places no unit on a tile that is at capacity, and it needs no
   count of its own.

## Done when

- A test founds two groups whose places overlap and asserts that no tile ends
  above the capacity of its ground.
- A test spawns into a full tile through the public interface and asserts that
  the spawn is refused and that the count did not change.
- A test asserts that the dense count agrees with the derived structure after a
  barrier, over a world with movement in it. This is the check that catches the
  two copies disagreeing.
- A test proves the previous one can fail. Perturb one count behind a test-only
  switch and assert that the agreement test then fails.
- The rejected-departure case has a test: a unit whose own intent is rejected
  releases no room, and the tile it stands on does not exceed its capacity.
- The thread-count test and the golden state test pass. Neither can see a
  capacity violation on its own, so they are not the evidence here.[^10]
- A test builds a world with units already in it and asserts that the count
  describes them before any frame runs. A count that starts empty makes every
  spawn succeed.[^8]
- A reviewer who did not write it has accepted ADR-0074, and the registry says
  so.[^11]
- The register rows say what this work settled.
- `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Open decisions register, DEC-020. `docs/DECISIONS.md`
[^2]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^3]: ADR-0056, decision D1. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^4]: ADR-0056, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^5]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^6]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^7]: Open decisions register, DEC-021. `docs/DECISIONS.md`
[^8]: ADR-0074, a tile occupancy count is stored densely, and a spawn refuses a full tile. `docs/adrs/draft/adr-0074-a-tile-occupancy-count-is-stored-densely-and-a-spawn-refuses-a-full-tile.md`
[^9]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^10]: Testing Rules, a determinism test cannot tell correct from consistently wrong. `.claude/rules/testing.md`
[^11]: ADR Registry. `docs/adrs/REGISTRY.md`
