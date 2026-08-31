# ADR-0056: Movement is tile-discrete and admitted by sort-then-admit

Status: Draft

## Context

The engine moves up to one million units on a hex world. A unit needs a
position, and two units must not occupy the same space without limit.

A continuous position is the usual choice. A unit then holds a sub-tile
coordinate and a velocity. Two units that want the same space must
negotiate. Reciprocal velocity obstacles solve that negotiation, and the
crowd report costs the method at this scale.[^1]

A continuous position also breaks two project rules. A sub-tile coordinate
and a velocity are fractional quantities, and simulated state holds no
floating point number.[^2] A negotiation between neighbours depends on the
order in which the engine visits them, and iteration order must be
explicit.[^3]

Congestion is the harder half of the problem. Many units want one tile.
The engine must decide which of them enter it. The decision must give the
same answer at any thread count, because one binary gives one answer at any
thread count.[^4] A parallel first-come rule cannot give that answer,
because arrival order is thread order.

A derived bridge structure answers which units stand on a tile. The bridge
is rebuilt at each barrier, so a system that moves a unit does not maintain
it.[^5] The unit arena itself is never sorted, because the slot index is half
of the entity identity.[^11]

## Decision

### D1. A unit occupies exactly one tile

A unit holds a tile index. It holds no sub-tile coordinate and no velocity
vector. A move takes a unit from its tile to one adjacent tile, or it does
not happen. There is no fractional position.

Speed is a progress accumulator against the step cost of the tile, not a
distance for each tick. The accumulator is an integer, and it clamps. An
unclamped accumulator overflows and enters the state hash.[^6]

### D2. A move is an intent, and a separate admission step grants it

A unit does not move itself. It writes an intent record. The intent names
the source tile, the target tile, and the unit's stable key. Writing the
intent is a pure read of the world.

A later step admits the intents. The two steps never interleave, so no unit
sees a half-applied world.

### D3. Admission sorts by a stable key, then admits in that order

The admission step runs four ordered sub-steps.

1. Reduce the intents by source tile. This gives the departure count of
   each tile.
2. Sort the intents by target tile, then by the unit's stable key. Each
   target tile then owns one contiguous segment.
3. Admit the intents of a segment in their sorted order, until the target
   tile reaches its capacity. The departure count of the target tile
   releases room in the same tick. Reject the remaining intents.
4. Write the accepted positions, then the departures, then the arrivals.

The sort is the engine's stable integer sort. The key ends in a unique
identifier, so no two intents tie.[^7] The segments are disjoint, so the
admission scan runs in parallel without an atomic operation.

Sub-step 1 is not an optimisation. Without it a column of units in a
corridor blocks itself, because the tile ahead still looks full.

### D4. Capacity is a data-driven property of the terrain

Each terrain type carries a capacity. The engine reads the capacity from
the terrain table. The engine holds no capacity constant of its own.

The capacity of a crossing terrain is higher than the capacity of ordinary
terrain. That difference is a design lever, and the movement calibration
depends on it.[^8]

This record states no capacity value. The values follow from the tile
scale, and the scale constants table holds them.[^9] [^10] The count array
that stores the occupancy of a tile bounds the capacity, because the count
is one byte for each tile.

### D5. A rejected unit is not stuck

The engine counts the rejections of a unit. Above a threshold the unit
takes a lateral step, or it marks its plan stale and plans again. The
engine draws the lateral step from the keyed generator, on the tuple of
system, frame, entity and draw. It never draws from thread-local state.

## Consequences

**Collision avoidance disappears as a subsystem.** The admission rule
replaces it. The engine never runs a velocity negotiation.

**Congestion is exact and reproducible.** The same intents give the same
admissions at one thread and at twelve.

**The renderer must interpolate.** A unit jumps between adjacent tiles. The
renderer smooths that jump between ticks. This is the one visible cost of
the decision, and no arithmetic settles whether it looks right.[^1]

**A formation cannot hold a shape below tile resolution.** A wedge or a
line of one tile width is not expressible.

**The capacity value cannot enter engine code.** A contributor who writes a
capacity literal in the movement kernel violates D4.

## References

[^1]: Report 10, crowd simulation and unit movement. `docs/research/reports/10-crowd-and-movement.md`
[^2]: ADR-0002, simulated and aggregated state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^3]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^4]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^5]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier. `docs/adrs/draft/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^6]: Findings register, FND-011, the progress accumulator overflows. `docs/FINDINGS.md`
[^7]: ADR-0007, content supplies a key vector, never a comparator. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^8]: Findings register, FND-037, a crossing time needs the terrain multiplier. `docs/FINDINGS.md`
[^9]: Blockers register, BLK-001 and BLK-009, both resolved. `docs/BLOCKERS.md`
[^10]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^11]: ADR-0014, entity identity is an index plus a generation. `docs/adrs/draft/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
