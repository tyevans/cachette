# ADR-0018: The unit-to-tile bridge is derived, and it rebuilds at the barrier

## Context

A tile is addressed by an index, and a unit is named by an identity.[^1] The
two populations therefore live in separate storage, and nothing in either one
answers the question that movement asks first: which units stand on this tile.

A unit holds the tile it occupies as a field, so the map from a unit to a tile
is direct. The reverse map is not stored.

The reverse map is what a system asks for whenever it must act on the units
that share a tile. Any system that joins a tile to the units on it needs it,
and none of them can build it from the unit columns at the moment of the
question without scanning the whole population. Admission is the first such
system: it must know what already occupies a target tile.[^2]

The unit population is sparse against the tile count. The tile count is fixed
and large. The unit count is smaller and changes every frame. Any structure
that grows with the tile count therefore spends most of its size recording
that a tile is empty.

The bridge partitions the world by the same block that the level of detail
pyramid aggregates over.[^9] One partition serves both. A change to the block
size therefore changes two subsystems, and neither may choose it alone.

The reverse map is derived. Level 0 is the only source of truth, so the bridge
must be rebuildable from the unit columns alone.

## Decision

### D1. The bridge is its own arrays, and it owns none of the units

The bridge holds a key array, a unit array, a block range array, and a block
occupancy bitplane. D5 states what the bitplane is for.

The bridge is wholly derived. It holds no fact that the entity columns do
not already hold, and destroying it loses nothing. It reorders nothing that
it does not own: the arena is not sorted to build it, because the slot index
is half of the identity and never moves.[^7]

The key array holds one bridge key for each occupying unit. The unit array
holds the matching entity identities in the same order. The block range array
holds one start and one length for each block.

The two parallel arrays stay parallel. Nothing reorders one without reordering
the other.

### D2. The bridge key orders a block together, and the engine derives it

The bridge key is a block-major ordering of the tile address. The tiles of one
block occupy one contiguous run of the key space. That run is what makes a
block range a start and a length rather than a list of runs.

The engine derives the key from the tile address by shifts and masks. It never
stores the key on the unit and never stores it on the tile. The tile address
stays the tile address.[^1]

When the tile storage order is already block-major, the derivation changes
nothing and the bridge key is the tile index itself.[^3] This record does not
fix the tile storage order. It fixes that the bridge key is derived, so that
the two orderings cannot disagree. A stored second copy would be a value
declared twice with nothing to fail when the copies diverge.

### D3. The bridge rebuilds at the frame barrier by a sort on the key

The engine rebuilds the whole bridge once for each frame, at the barrier. It
sorts the occupying units by the bridge key. This record does not name the
algorithm. The key is an exact integer, so the choice is open, and another
record holds it.[^11]

The sort is total. Units that share a bridge key break the tie on the
identity, taken as one integer, so the order is fixed and no two runs
disagree.[^4] The identity is opaque, so this record does not say which of
its parts sorts first; it says only that the whole value is the tie-break and
that no two live entities share one.[^7] The key is a
vector of exact integer fields whose last field is a stable identifier, which
is the form the engine sorts by everywhere.[^10] The bridge
holds no result whose order came from a thread finishing first.

The engine never updates the bridge incrementally while systems run. An
incremental update would need a write from every system that moves a unit,
and the merge order of those writes is exactly the nondeterminism the project
cannot carry.

### D4. A per-tile query reads the block range, then searches inside it

To find the units on one tile, the engine reads the range for the block that
holds the tile, then searches that range for the tile key. The range covers
one block, so the search is bounded by the block size and not by the unit
count.

A system that needs many per-tile answers within one block may build a dense
index for that block on demand. That index is scratch. Nothing stores it
between frames.

### D5. A bitplane marks each block that holds at least one unit

The bitplane is one bit for each block. A query that descends the level of
detail pyramid tests the bitplane and skips an empty block without reading its
range.

### The alternative this rejects

**An offset array over every tile, in the usual compressed sparse row form.**
This gives a constant-time per-tile lookup with no search, and it is the
standard structure for this problem.

The project rejects it for the offset array, not for the per-tile array. A
per-tile array of counts already exists, because admission needs the
occupancy of a target tile and its departure count in the same tick.[^2]

What the compressed sparse row form adds is an offset array that must be
exact everywhere. Its rebuild repairs every entry once for each frame, even
where nothing moved, so the per-frame cost follows the tile count rather than
the work. The block range array is exact at the block and silent below it, so
its rebuild cost follows the occupied blocks. The search inside a block is
what buys that.

**A list or a map for each tile** is also rejected. It allocates for each
occupied tile, it scatters the payload, and a map introduces an iteration
order that no key fixes.[^4]

## Consequences

**The bridge covers the mobile shape only.** Of the four fixed entity shapes,
the bridge indexes the soldier.[^5] A settlement and a tile upgrade are fixed
to a tile, so their tile field is already the answer and they need no rebuild.
A living character carries no tile position, so it is absent from the bridge.

**The bridge is derived state, and a test can prove it.** Two rebuilds from
the same unit columns must give the same arrays, at any thread
count. A property test also checks that every occupying unit appears exactly
once in the bridge, and that its bridge key matches its tile field.

**A per-tile answer costs a search, not a subscript.** Every caller pays the
search. A caller that cannot pay it must batch its queries by block, which is
the access pattern the whole bridge assumes.

**The rebuild cost follows the unit count.** The sort is the dominant term,
and it is bounded by the number of occupying units. The block range array is
the only part that follows the block count, and a block covers many tiles. No
figure appears here, because no measurement exists on the target platform.[^6]

**Movement reads the bridge and never writes it.** Admission resolves
contention for a target tile from the bridge as it stood at the barrier.[^2] A
move applied during the frame does not change what a later system reads in the
same frame. Every system therefore sees one consistent occupancy for the whole
frame.

**Every identity in the bridge is live for the whole frame.** The rebuild
runs after the structural apply at the barrier, and no entity dies while
systems run, so the unit array names only live entities.[^7] A reader still
resolves before it acts, because the identity is the only handle it has, but
the resolution cannot fail during the frame.

The ordering inside the barrier is therefore a decision and not an
implementation detail. Rebuilding before the structural apply would leave a
dead identity in the array for the whole frame, and every caller would pay a
branch that this ordering removes.

**The evidence for the block form is a research report.** The report
compares the offset array against the block form and recommends the block
form.[^8] The block that the bridge uses is the aggregation block of the level
of detail pyramid, so the bridge and the pyramid share one partition.[^9]

## References

[^1]: ADR-0017, the world is a rhombus, so a tile index is raw axial. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
[^2]: ADR-0056, movement is tile-discrete and admitted by sort then admit. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^3]: ADR-0016, tiles are stored in block-tiled order at the aggregation block size. `docs/adrs/REGISTRY.md`
[^4]: ADR-0004, iteration order is explicit, and unordered reductions need slots. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^5]: ADR-0066, entity storage holds four fixed shapes. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^6]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^7]: ADR-0014, entity identity is an index plus a generation. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^8]: Report 01, the entity component system core and the memory layout, section 9. `docs/research/reports/01-ecs-and-memory-layout.md`
[^9]: Report 02, the hex grid and the level of detail pyramid, sections 3.4 and 7.4. `docs/research/reports/02-hex-grid-and-lod-pyramid.md`
[^10]: ADR-0007, content supplies a key vector, never a comparator. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^11]: ADR-0071, the bridge rebuild orders on one thread, decision D1. `docs/adrs/accepted/adr-0071-the-bridge-rebuild-orders-on-one-thread.md`
