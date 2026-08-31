# ADR-0071: The bridge rebuild orders on one thread

## Context

The unit-to-tile bridge maps a tile to the units that stand on it. The engine
does not store that map. It derives the map once for each frame, at the frame
barrier, by an order on a key.[^1]

The key is block-major. The high part names the block that holds the tile, and
the low part names the tile inside the block.[^2] The key therefore lies in a
dense range that the world extent bounds. The engine knows that bound before it
reads a single unit.

Content supplies an ordered vector of exact integer key fields, and never a
comparison function.[^3] The last field is a stable identifier, so no two keys
tie and the order has exactly one correct output.[^4] That record already
states the consequence that an integer key permits a radix sort.[^5]

The general order in this engine runs on many threads. It divides the keys into
chunks, orders each chunk into its own slot, and then merges the runs in slot
index order. Slot index order does not depend on which thread finished
first.[^6]

That arrangement is sound, and it is the wrong shape for the bridge. It pays
for two things the bridge does not need. It pays a thread for each chunk, and
the operating system charges for a thread whether the chunk is large or small.
It also pays a comparison for each step of the order, when the key is a bounded
integer that a counting pass can place directly.

A measurement of the rebuild on a development machine started this record. The
rebuild is a large part of what a unit costs in one step, and the cost has a
fixed part that is absent when no unit exists. The figures are in the commit
that made the change, because a figure decays and a record does not.[^7] No
measurement exists on the target platform, so the figures are evidence about a
shape, not about a budget.[^8]

## Decision

### D1. The bridge orders by a radix pass on the bounded key

The engine orders the bridge with a radix sort on the ordering field. The
caller states a ceiling for that field. The sort reads the ceiling and derives
the number of passes from it. Nothing derives the ceiling from the data, so the
cost of an order does not change when the data changes.

A key above the stated ceiling is a caller mistake. The sort refuses the whole
set and names the key. It does not widen itself to fit.

### D2. The bridge rebuild runs on one thread

The rebuild takes a thread count from its caller and orders on one thread. It
still refuses a thread count of zero, because zero threads is a caller mistake
whatever the algorithm.

A radix pass is a scan of the whole set with one shared histogram. To run it on
many threads, the engine would have to split the histogram, combine the parts
in a fixed order, and then place each item at an offset that the combine
produced. That is more machinery than the whole rebuild costs today, and every
piece of it is a place where a result can take its order from a thread.

The engine parallelises what a thread earns. The bridge rebuild does not earn
one.

### D3. A radix pass is stable, and the identifier replaces the input order

A radix pass keeps the order that the caller gave for items that share a digit.
The input order is the order that the arena holds, which is the slot order. The
slot order is not part of the key, so it must not reach the output.

After the passes, the engine finds each run of items that share the whole
ordering field, and it orders that run by the stable identifier. The output
then depends on the key values alone. It is the same permutation that the
general order gives for the same keys.

The identity is the tie-break, and the identity is the whole value.[^4] This
decision does not change that. It changes only how the engine reaches the same
answer.

## Consequences

**The rebuild gives one answer at any thread count, for a stronger reason than
before.** The old reason was that the merge reads the slots in index order. The
new reason is that no second thread exists. A reviewer can check the new reason
by reading the call graph.

**The bridge no longer scales with the thread count.** A world that grows far
past the target unit count will reach a point where one thread is too slow. At
that point the project must either parallelise the radix pass under D2, or
partition the rebuild by block. Either is a new decision and needs a record.

**The sort module now holds two orders.** The general order takes a key vector
of any width. The bounded order takes two fields and a ceiling. A property test
holds them together: for the same keys, the two give the same permutation. If
that test ever fails, one of the two is wrong and neither may be trusted.

**A caller must know the bound of its key.** A caller that cannot state a
ceiling uses the general order. The bridge can state one, because the block
partition of the world fixes it.

## Alternatives rejected

**Keep the comparison order and reduce the thread count.** This removes part of
the fixed cost and none of the cost for each unit. The comparison order is the
larger part.

**Order the whole key vector by radix.** The identity field is a wide integer,
so a radix over it costs many more passes than the ordering field does. It is
slower than the comparison order it would replace.

**Break the tie on the slot index rather than on the identity.** The slot index
is unique among live units, so this order is total, and it removes the second
step of D3. It is rejected. A slot is reused after a unit dies, so a new unit
would inherit the position that the dead unit held in a contest for a tile. The
project has already recorded one defect of that shape, where a random draw was
keyed on the slot rather than on the identity.[^9]

**Update the bridge as each unit moves.** The engine rebuilds the whole bridge
rather than updating it, because an incremental update needs a write from every
system that moves a unit, and the merge order of those writes is the
nondeterminism this project cannot carry.[^1] That reasoning is unchanged.

## References

[^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^2]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D2. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^3]: ADR-0007, content supplies a key vector, never a comparator, decision D1. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^4]: ADR-0007, content supplies a key vector, never a comparator, decision D2. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^5]: ADR-0007, content supplies a key vector, never a comparator, the consequences. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^6]: ADR-0004, iteration order is explicit, decision D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^7]: Commit Message Rules. `.claude/rules/commits.md`
[^8]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^9]: Testing Rules, a determinism test cannot tell correct from consistently wrong. `.claude/rules/testing.md`
