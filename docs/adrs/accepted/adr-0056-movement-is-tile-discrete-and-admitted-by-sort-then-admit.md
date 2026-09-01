# ADR-0056: Movement is tile-discrete and admitted by sort-then-admit

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
of the entity identity.[^14]

## Decision

### D1. A unit occupies exactly one tile

A unit holds a tile index. It holds no sub-tile coordinate and no velocity
vector. A move takes a unit from its tile to one adjacent tile, or it does
not happen. There is no fractional position.

Speed is a progress accumulator against the step cost of the tile, not a
distance for each tick. The accumulator is an integer, and it clamps. An
unclamped accumulator overflows and enters the state hash.[^6]

### D2. A move is an intent, and a separate admission step grants it

A unit does not move itself. It writes an intent, and an intent names the
unit and the tile it wants. Writing the intent is a pure read of the world.

The intent carries no source tile. The arena already holds where a unit
stands, and the identity is how admission reads it. A second copy of the
source tile is one fact in two places, with nothing that fails when the two
disagree.

A later step admits the intents. The two steps never interleave, so no unit
sees a half-applied world.

### D3. Admission sorts by a stable key, then admits in that order

Admission sorts the intents by target tile, then by the unit's identity.
Each target tile then owns one contiguous segment, and the segments are
disjoint. The sort is the engine's key vector sort, and the identity is the
final key field, so no two intents tie.[^7]

Admission then scans each segment in its sorted order and admits until the
target tile reaches its capacity.

**Admission reads the occupancy of a target tile from the derived
unit-to-tile structure**, which the barrier rebuilt before the intents were
drawn.[^13] It does not carry a per-tile array of its own. A dense array over
every tile would be faster and would be a second declaration of where units
stand, and no record chooses one.

**A unit that leaves a tile releases room in that tile, and only an admitted
departure counts.** An intent is not a departure. A unit that intends to
leave and is then rejected at its own target has not left, so the room it
appeared to release was never released.

The distinction is the whole of this decision, and getting it wrong is
invisible. Take three tiles in a line. The middle tile and the far tile are
both full. The unit in the middle intends to move to the far tile and is
rejected. The unit in the first tile is admitted into the middle tile on the
strength of a departure that never happened, and the middle tile ends the
tick above its capacity.

**That failure is deterministic.** The same intents give the same wrong
answer at every thread count, so the thread-count test passes and the state
hash matches its golden file. Neither determinism test can see a capacity
violation. Only a test that asserts the invariant can.[^11]

**Admission runs a fixed number of passes.** Each pass admits what it can
against the room that the previous pass confirmed. A pass admits no unit that
a previous pass admitted. The count is content, declared before the frame
runs, and the engine never runs to a fixpoint: a fixpoint needs a convergence
test, and a solver in this project runs a fixed count.[^12]

The project rejects the alternative of ordering the tiles so that a departure
always precedes an arrival. No such order exists. Two units that swap
adjacent tiles each release room for the other, and a ring of units around a
closed path does the same. A cycle has no admissible order.

**A departure is applied after the scan, not inside it.** The segments are
disjoint by target tile, so every write addressed to a target tile is free of
contention. A write addressed to a source tile is not: the units leaving one
tile are scattered across many segments, because they chose different
targets. Departures are therefore a separate reduction over the admitted set,
keyed on the source tile.

### D4. Capacity is a data-driven property of the terrain

Each terrain type carries a capacity. The engine reads the capacity from
the terrain table. The engine holds no capacity constant of its own.

The capacity of a crossing terrain is higher than the capacity of ordinary
terrain. That difference is a design lever, and the movement calibration
depends on it.[^8]

This record states no capacity value. The values follow from the tile scale,
and the scale constants table holds them.[^9] [^10]

This record does not decide how the engine stores an occupancy count, and no
record does. Whether occupancy is read from the derived structure or held in
a dense array over every tile is a storage decision, and the work that needs
it writes the record.[^13]

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
[^5]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^6]: Findings register, FND-011, the progress accumulator overflows. `docs/FINDINGS.md`
[^7]: ADR-0007, content supplies a key vector, never a comparator. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^8]: Findings register, FND-037, a crossing time needs the terrain multiplier. `docs/FINDINGS.md`
[^9]: Blockers register, BLK-001 and BLK-009, both resolved. `docs/BLOCKERS.md`
[^10]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^12]: ADR-0005, a solver runs a fixed iteration count, never a convergence test. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
[^13]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^11]: Testing Rules, a determinism test cannot tell correct from consistently wrong. `.claude/rules/testing.md`
[^14]: ADR-0014, entity identity is an index plus a generation. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
