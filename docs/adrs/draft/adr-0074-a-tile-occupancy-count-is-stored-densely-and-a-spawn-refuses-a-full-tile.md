# ADR-0074: A tile occupancy count is stored densely, and a spawn refuses a full tile

## Context

A tile holds a limited number of units. The capacity is a property of the
terrain, and the engine reads it from the terrain table.[^1] The engine must
therefore answer one question often: how many units stand on this tile now.

Two parts of the engine ask it. Admission asks it once for each target tile,
when it decides which units enter a tile that many units want.[^1] A spawn asks
it when it places a unit into the world. A founding is a spawn that places many
units over one disc of tiles.

Until now only admission asked it, and no record said where the answer comes
from. The movement record states that admission reads the occupancy from the
derived unit-to-tile structure, which the barrier rebuilt before the intents
were drawn. The same record then says that no record chooses between that read
and a dense array over every tile, and it leaves the storage open.[^2]

A spawn cannot use the derived structure. The structure describes the arena as
the last barrier left it, so between two frames it does not describe the arena.
A spawn runs outside the frame. A spawn that read the structure would either
read a stale answer or force a rebuild that the step already owns.

A spawn that counts for itself is worse. Counting the units on a tile by a scan
of the arena costs the whole population for each placement, and the target
scale does not permit that.[^3]

The result today is that a spawn reads no count at all. Two foundings whose
discs overlap put a tile above its capacity, and the engine accepts it.
Movement then only ever takes units off that tile, because admission never
raises a tile above the capacity of its ground.

**Neither determinism test can see this.** The same placements give the same
over-filled tile at every thread count, so the thread-count comparison passes,
and the state hash matches its golden file. Only a test that asserts the
invariant can find it.[^4]

**Two accepted records already disagree about whether a dense count exists.**
The bridge record rejects an offset array over every tile, and part of its
reason is that a per-tile array of counts already exists, because admission
needs the occupancy of a target tile and its departure count in the same
tick.[^5] The movement record says that admission carries no per-tile array of
its own.[^2] Both records are accepted, and source files cite both. One of the
two claims must become true.

## Decision

### D1. The engine holds a dense count of the units on each tile

The engine stores one count for each tile of the world. The count is one byte.
It is dense: the storage covers every tile, whether or not a unit stands there.
One byte for each tile is a structural property of this choice, not a budget.

The count is simulated state. It is an integer, and no floating point number
enters it.[^6] It saturates rather than wraps, because a wrap turns an
over-filled tile into an empty one and puts a false value in the state hash.

**This replaces the clause of the movement record that says admission reads the
occupancy from the derived structure and carries no per-tile array of its
own.**[^2] It also replaces the paragraph of that record that says no record
decides how the engine stores an occupancy count.[^1] The rest of the movement
record stands. A unit still occupies exactly one tile, a move is still an intent
that a separate admission step grants, admission still sorts by a stable key,
and capacity is still a property of the terrain.

**This makes the bridge record's stated reason true.** That record rejected an
offset array over every tile on the ground that a per-tile array of counts
already exists.[^5] It did not exist. It does now, and the rejection stands on
the reason it always gave.

**This record closes a contradiction between two accepted records. It does not
open one.** A reader who finds the bridge record and the movement record
disagreeing about a per-tile count must land here. This record is the answer to
that disagreement, and the movement record holds the claim that gives way.

### D2. The dense count is not an offset array, and the rebuild argument does not reach it

The bridge record rejects an offset array over every tile because the array
must be exact everywhere before any query is correct. Its rebuild therefore
repairs every entry once for each frame, even where nothing moved, so the cost
follows the tile count rather than the work.[^5]

The dense count is not that structure. Nothing rebuilds it. Admission
increments it where a unit arrives and decrements it where an admitted unit
departs, so the write cost follows the moves, not the tiles. A tile where
nothing happened is never touched.

The dense count is a summary. The derived structure remains the answer to which
units stand on a tile, and this record does not replace it.[^5]

### D3. A spawn refuses a tile that is at capacity

A spawn reads the count and the capacity of the ground. It places the unit when
the count is below the capacity. It refuses when the count has reached it.

**A refusal is an outcome that the caller receives.** A spawn does not drop a
unit silently. A founding that cannot fill a tile learns which placements were
refused, and it decides what to do.

The capacity comes from the terrain table. A spawn holds no capacity value of
its own, and a capacity literal in a spawn violates the movement record.[^1]

The check is a read of one count and one table entry. It costs the same for one
placement and for a founding of many, so a founding needs no count of its own
and makes no second declaration of who stands where.

### D4. Only an admitted departure decrements the count

An intent is not a departure. A unit that intends to leave a tile and is then
rejected at its target has not left, and the room it appeared to release was
never released.[^2]

The count obeys that rule. Admission decrements the source tile of a unit it
admitted, and it decrements nothing for a unit it rejected.

Getting this wrong is invisible and deterministic. Three tiles stand in a line.
The middle tile and the far tile are both full. The unit in the middle intends
to move to the far tile and is rejected. If the count decremented on the intent,
the unit in the first tile enters the middle tile on the strength of a departure
that never happened, and the middle tile ends the tick above its capacity. The
same intents give the same wrong answer at every thread count.[^4]

### D5. A check fails when the count and the derived structure disagree

The count is a second place that says where units stand. The derived structure
is the first. One fact in two places, with nothing that fails when the copies
disagree, is a silent failure: both sites read back correctly and only one is
right.

The engine therefore carries a check that compares the count against the derived
structure after a barrier, over a world with movement in it. A comment that
names which copy wins is not that check, and this record does not permit one.

The check must be proved able to fail. Perturb one count behind a test-only
switch, and the check must then fail.[^4]

### D6. The write order is fixed by the sort, not by the threads

Admission already sorts the intents by target tile and then by the identity of
the unit, and each target tile owns one contiguous segment. The segments are
disjoint, so an increment addressed to a target tile meets no other writer.[^2]

A decrement is not addressed that way. The units leaving one tile are scattered
across many segments, because they chose different targets. A decrement is
therefore a separate reduction over the admitted set, keyed on the source
tile.[^2] A stable key fixes the order of every parallel result. Thread
completion order and work-stealing order fix nothing.[^7]

## Consequences

**The world carries storage that grows with the tile count.** This is the cost
of the decision, and it is the reason the alternative was tempting. The scale
constants table holds the tile count that the storage covers.[^3]

**Capacity becomes an invariant of the world, not a rule that movement obeys.**
A reader of the world may now trust that no tile is above the capacity of its
ground. Before this record, only a tile that movement had touched carried that
guarantee.

**A caller must handle a refusal.** A spawn that used to succeed may now fail.
Every caller of a spawn gains a case it did not have, and a caller that ignores
the refusal loses units without knowing it.

**The engine gains a second declaration of where units stand, deliberately.**
D5 is what makes that safe. If the check in D5 is ever removed, this decision
becomes the defect it was written to avoid.

**A constant-time admission becomes possible.** Admission no longer searches
the derived structure for the occupancy of a target tile. This record claims no
figure for that, because every cost figure in this project is derived rather
than measured.

**The count must be seeded when the world is built.** A world that loads units
without building the count starts with a count that says nothing stands
anywhere, and every spawn then succeeds. The barrier question governs when a
structural change made outside a frame becomes visible, and it is open.[^8]

## References

[^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^2]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^3]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^4]: Testing Rules, a determinism test cannot tell correct from consistently wrong. `.claude/rules/testing.md`
[^5]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^6]: ADR-0002, simulated and aggregated state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^7]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^8]: Open decisions register, DEC-021. `docs/DECISIONS.md`
