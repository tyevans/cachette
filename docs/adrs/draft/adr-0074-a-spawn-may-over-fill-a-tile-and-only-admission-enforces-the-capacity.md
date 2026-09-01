# ADR-0074: A spawn may over-fill a tile, and only admission enforces the capacity

## Context

A tile holds a limited number of units. The capacity is a property of the
terrain, and the engine reads it from the terrain table.[^1]

Two parts of the engine put units on a tile. Admission moves a unit that
already stands in the world onto a tile it asked for.[^2] A spawn places a new
unit. A founding is a spawn that places a group over one disc of tiles.

Admission reads the occupancy of a target tile and grants no intent that would
carry the tile above the capacity of its ground.[^2] A spawn reads no
occupancy at all. It refuses impassable ground and it refuses a faction the
world does not hold, and then it places the unit. Two foundings whose discs
overlap therefore put a tile above its capacity, and the engine accepts it.

**The code is silent about whether that is a decision or an oversight.** A
reader of a spawn sees an absent check. An absent check reads as a defect, and
one reader treated it as one. The reasoning that makes the absence correct
lives nowhere the artefact can hold it, so this record holds it.

**Neither determinism test can see an over-full tile.** The same placements
give the same result at every thread count, and the state hash matches its
golden file. Only a test that asserts the invariant can speak about it, and
the invariant is the subject of this record.[^3]

**Two accepted records disagree about whether a dense per-tile count exists.**
The bridge record rejects an offset array over every tile. Part of its stated
reason is that a per-tile array of counts already exists, because admission
needs the occupancy of a target tile and its departure count in the same
tick.[^4] The movement record states the opposite: admission reads the
occupancy from the derived unit-to-tile structure and carries no per-tile
array of its own.[^2] Both records are accepted, and source files cite both.
The findings register holds the pair.[^5] One of the two claims must give way,
and this record says which.

## Decision

### D1. A spawn places a unit without reading the tile capacity

A spawn refuses impassable ground, and it refuses a unit the world cannot
hold. It does not refuse a tile that is at its capacity. A caller that places
a group over a disc may carry a tile above the capacity of its ground, and the
engine accepts the placement.

The capacity is a rule that movement obeys. It is not an invariant of the
world at rest.

**A spawn therefore has no capacity outcome.** No caller of a spawn gains a
refusal case for a full tile, and no caller must decide what to do with a unit
the engine declined to place.

### D2. Admission is the only enforcer, and what it guarantees is monotone

Admission grants no intent that raises a tile above the capacity of its
ground. That is the whole of the guarantee, and it is narrower than it looks.

The guarantee is that **no tile gains a unit beyond its capacity**. It is not
that no tile is ever above its capacity. A tile that a spawn over-filled stays
over its capacity until its units leave. It may not rise, and it may not rise
above its capacity from below.

**An over-full tile drains and never fills.** Admission computes the room of a
target tile by subtracting the occupancy from the capacity, and the
subtraction saturates rather than wrapping.[^2] A tile above its capacity
therefore offers no room and admits nobody, while the units standing on it may
still depart. The over-fill relaxes toward the capacity and never away from
it.

**That drain is why D1 is safe, and it is a consequence of this decision
rather than an accident of the arithmetic.** A future change that gave an
over-full tile room, by any signed arithmetic or by any rule that lets a tile
refill toward some other bound, would violate this decision.

### D3. The engine holds no dense per-tile occupancy count

The engine stores no array of counts over every tile. Admission reads the
occupancy of each target tile from the derived unit-to-tile structure, which
the barrier rebuilt before the intents were drawn.[^2] The derived structure
is the one declaration of where units stand.

**The movement record holds the true claim, and the bridge record's aside is
false.**[^2] [^4] The bridge record rejects an offset array over every tile
because such an array must be exact everywhere before any query is correct, so
its rebuild repairs every entry once for each frame, even where nothing moved.
That reason stands on its own. The per-tile array of counts that the record
names beside it never existed, it does not exist now, and the rejection never
needed it. A reader who meets the two claims must land here, and this record
says the bridge record was wrong about the mechanism and right about the
rejection.

The project rejects the dense count for two reasons.

**Nothing would call it.** Its purpose was a constant-time capacity check for
a spawn, and D1 removes that check. Admission already has an answer it trusts.
A structure that the engine declares and nothing invokes is inert, and its own
test would pass while nothing reached it.[^6]

**It would be a second declaration of where units stand.** One fact in two
places, with nothing that fails when the copies disagree, is the defect shape
this project meets most often.[^6] The dense count would need a check that
fails when it disagrees with the derived structure, and that check would need
a proof that it can fail. That is the price of a second site, and no caller is
asking to pay it.

This record claims no cost figure for either option. Every cost figure in this
project is derived rather than measured.[^7]

### D4. A caller that wants a tile filled to its capacity counts its own placements

A founding fills each tile of its disc to the capacity of that tile's ground.
It reaches that number by asking the terrain and by counting the units it
placed itself. It reads no occupancy of the world, and it makes no second
declaration of where units stand.

A caller that must not over-fill therefore carries that rule itself. The
engine does not carry it, and no caller may assume it.

## Consequences

**A watcher may see a tile above its capacity, and that is a state of the
world rather than a fault.** The viewer already reports it.[^8] A reader of
the world may not assume that every tile is at or below the capacity of its
ground.

**The product statement that no tile holds more units than its capacity allows
is false, and it will stay false.** That statement belongs to a shipped
product record, and it needs repair against this decision.[^9]

**An invariant test must assert the monotone form.** A test that asserts that
no tile is ever above its capacity fails on a legitimate world. A test must
assert that no tile rises above its capacity, and that an over-full tile does
not rise.[^3]

**The occupancy storage question that the movement record deferred is now
answered, and the answer is that no new storage exists.**[^2] The world
carries no per-tile array, and the storage of the world does not grow with the
tile count on account of the occupancy.

**A later contributor who wants a spawn to refuse must supersede this record.**
The change is not local. It needs an occupancy answer that is valid outside a
frame, a refusal outcome at every caller of a spawn, and a decision about what
a founding does with a placement it could not make.

## References

[^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^2]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^3]: Testing Rules, a determinism test cannot tell correct from consistently wrong. `.claude/rules/testing.md`
[^4]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^5]: Findings register, FND-081. `docs/FINDINGS.md`
[^6]: Recurring Defect Shapes, shapes 1 and 3. `.claude/rules/recurring-defects.md`
[^7]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^8]: ADR-0067, the viewer reads the world and never writes to it, decision D1. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^9]: PRD-0002, a developer watches the world run. `docs/product/shipped/prd-0002-a-developer-watches-the-world-run.md`
