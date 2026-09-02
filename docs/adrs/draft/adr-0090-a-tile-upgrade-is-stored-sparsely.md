# ADR-0090: A tile upgrade is stored sparsely, as the difference from the generated world

## Context

A tile upgrade is a thing a unit built on a tile. It is one of the four fixed
entity shapes: sparse, attached to a tile, and not to a mobile entity.[^1]

The world holds no memory of anything a unit did to it until something stores
one. Every tick starts from the state the generator made, so work that takes
many ticks cannot exist, and holding ground for a long time gains nothing.

The ground of this world is a pure function of the seed and the tile
address, and the engine stores no terrain map.[^2] The stock of a tile is the
same shape, and the engine stores only what a unit took from it.[^3] Both
halves of the world therefore already refuse a dense array over the tiles.

An upgrade is different from both. No function of the seed can produce it,
because it is a fact that a unit created. The engine must store it.

The target scale is 16.7 million tiles. The owner answered how many of them
carry an upgrade, and the answer is a small fraction.[^4] The scale constants
table holds the figure.[^5]

A progress accumulator that several units add to must combine exactly, in any
order, because the threads that hold those units finish in no fixed order.[^6]
Floating point addition is not associative, so it cannot hold this.[^7]

An upgrade changes a property of a tile. The ground already states one such
property, the number of units a tile holds, and a record says that property is
data the movement system reads rather than a rule the movement system
states.[^8]

## Decision

### D1. The engine stores one entry for each improved tile, and nothing for any other

**Storage grows with the number of upgrades, not with the number of tiles.**

A world in which nobody built holds no entry at all. The memory cost follows
the building, and a read of an unimproved tile pays one indirection and finds
nothing.

The entries are held sorted by tile index, so a lookup is a binary search and
the order never depends on which unit built first.[^6] Entries are merged in
ascending runs, never inserted one at a time.

**The advance pass reads the builders and the entries. It takes no grid and no
tile count.** A world of any size in which one unit builds costs what the same
build costs in a small world. This is the property that a dense store would
lose, and the engine reports the count of entries it read so that a test can
assert it.

### D2. An unfinished build is stored, and its progress is a clamped whole number

A build takes more than one tick. The work that has gone into a tile is stored
between ticks, so a unit that stops and starts again continues rather than
restarts, and a unit that dies leaves the work it did.

The accumulator is 64 bits wide and every contribution is a whole
number.[^7] [^9] Several units contribute in one tick and the total is the
same however the threads produced them.

**The accumulator is clamped at the work its kind asks for.** An unclamped
accumulator lets a builder bank surplus that nothing can spend, and the
surplus overflows a narrow field and enters the state hash. The project has
already recorded that defect.[^10] The bound is folded from the catalogue
rather than written down a second time.[^11]

The progress is simulated state that a later frame reads, so both the kind and
the progress enter the whole-world hash.[^12]

### D3. One function composes the generated property and the upgrade row

The ground states how many units a tile holds. A finished upgrade may state a
larger number, and the larger of the two wins. **One function reads both
tables, and every caller that enforces the capacity of a tile calls it.** A
second rule beside it would be one fact in two places, and nothing would fail
when the two disagreed.[^11]

**A caller that asks a different question reads the table its question is
about.** A bound on the work a site opens follows the ground alone. A founding
that seats a group over new ground follows the ground alone as well. Neither
enforces a capacity, because admission is the one rule that does.[^13] A
register row holds whether such a caller should compose instead.[^14]

**An upgrade states no capacity of its own.** A made way is ground that a unit
crosses quickly, and the project already holds the capacity of such ground in
the scale constants table.[^5] The terrain module owns the capacity table, and
the upgrade row reads the value from there rather than restating it.

**Ground that admits nobody stays closed.** An upgrade changes how many units a
tile holds. It never changes whether the tile holds anybody. Every caller that
asks only about passability therefore reads the ground and stays correct, and
the derived levels above level 0 do not have to learn about upgrades.[^15]

**A site under construction changes nothing.** Only a finished upgrade reaches
the composition.

A tile carries one upgrade. Two upgrades on one tile would make decision D4 a
question with more than one answer.

### D4. Destroying an upgrade is removing the entry

The tile returns to the world the generator made. Nothing else stores a
property of an improved tile, so removing the entry is the whole of the
return, and no second copy can survive it.

The entry is what says the tile was improved. An absent entry means "nothing
has happened here", which is true.

## Consequences

**A world costs nothing for the tiles nobody built on.** The cost is the
building, and it is bounded by the fraction the owner named.[^4]

**A read of a tile property costs a binary search.** The ground answers in
arithmetic and the upgrade answers in a search over the improved set. A caller
that sweeps every tile pays that search on every tile, and the project has no
measurement of what it costs on the target platform.[^16]

**The engine cannot place an upgrade by hand into the generated world.** An
upgrade exists because a unit built it, or because the control plane removed
one. There is no authored content path, and adding one is a new decision.

**An upgrade does not decay.** Nothing takes progress away and nothing wears a
finished upgrade out. Upkeep is a rate attached to a site, and a rule that
spent one on an upgrade is a new decision.[^17]

**The advance pass cannot be told to skip a tick.** It runs on every tick, and
its cost is the builders plus the entries. A schedule would be a new decision,
and it would need the same shape the rate schedule has.[^17]

**A builder on a finished upgrade wastes its work.** The clamp of D2 absorbs
the contribution and nothing tells the builder to stop. A rule that stopped it
is a behaviour decision and this record does not make it.

**The advance is order-free, so a determinism probe over its order asserts
nothing.** The contribution of a tile is a count of builders times a whole
number, and integer addition does not depend on order.[^6] The sorted order
decides only which kind takes a tile that carries none, and the sort is the key
vector sort that the project already uses.[^18] The two determinism tests still
cover the stored result, because the map reaches the state hash.

## Alternatives rejected

**A dense array of one upgrade for each tile.** This is the obvious shape, and
it is the shape the ground and the stock both already refused. It pays the
whole world for the tiles nobody built on, and the fraction the owner named
makes almost all of that payment waste. A record already rejected a dense
per-tile count for the same reason.[^19]

**A dense array of one bit for each tile, beside the sparse entries.** The bit
would say whether a tile carries an upgrade, so a read could skip the search.
It is rejected because it holds the same fact twice, and nothing would fail
when the bit and the entry disagreed.[^11] The absent entry already says it.

**Store the finished world rather than the difference.** The engine would hold
the capacity of every improved tile, not the upgrade that produced it. It is
rejected because the tile could then not return to what it was: nothing would
hold the value the generator gave, and a destroyed upgrade would have to guess
it. Storing the difference makes the return exact.

**Give the progress a narrow field.** A narrower accumulator is smaller and the
sparse set is small. It is rejected because the project has already lost a
frame to a progress accumulator that overflowed a narrow field, and the
accumulator is the one place this record cannot afford to be clever.[^10]

**Let a tile carry several upgrades.** A set of upgrades on one tile composes
more richly. It is rejected because no need has asked for it, and because it
makes the return of D4 ambiguous: destroying one of several upgrades has no
single answer for what the tile becomes.

## References

[^1]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^2]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
[^3]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^4]: Blockers register, BLK-006. `docs/BLOCKERS.md`
[^5]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^6]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^7]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^8]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^9]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^10]: Findings register, FND-011. `docs/FINDINGS.md`
[^11]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^12]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^13]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity, decision D2. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
[^14]: Decisions register, DEC-081. `docs/DECISIONS.md`
[^15]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^16]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^17]: ADR-0062, production and upkeep are rates attached to a site, decision D4. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
[^18]: ADR-0007, content supplies a key vector, never a comparator, decision D1. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^19]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity, decision D3. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
