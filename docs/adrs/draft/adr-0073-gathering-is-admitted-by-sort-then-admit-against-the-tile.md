# ADR-0073: Gathering is admitted by sort-then-admit against the tile

## Context

A unit standing on a tile takes an amount from the stock of that tile. Several
units can stand on one tile, so several units can name one deposit in one
frame. A deposit can run out part way through, so the engine must decide which
of them takes the last of it.

That decision is the whole problem. Every other part of gathering is
arithmetic.

A lock on the tile would answer it, and the answer would follow which thread
reached the lock first. A retry loop would answer it, and the answer would
follow the same thing. Both give one binary two answers, which is the property
this project cannot lose.[^1]

The engine already meets this problem once. Several units can name one target
tile in one frame, and the target has a capacity. Movement resolves it by
sorting the intents on a bounded key and then admitting each contiguous segment
in that sorted order.[^2] Nothing in that shape is specific to movement.

The target scale is a million units. A rule that visits a deposit once for each
unit that wants it pays for a conflict that is rare. A rule that steps every
deposit each tick pays the world for the deposits that nothing is touching.

The engine derives a map from a tile to the units on it, and it rebuilds that
map at the frame barrier.[^3] A unit moves during the frame, so what tile a
unit stands on is settled at the barrier and not before it.

## Decision

### D1. The command names a unit and a kind, and the step resolves the whole set

An order to gather is a fact stored on the unit: a kind, or nothing. The
command that sets it does no work. The step resolves every order of the frame
in one pass over the sorted intents.

The cost therefore follows the number of units that gather. It does not follow
the number of deposits, and it does not follow the size of the world. A deposit
that nothing is touching costs nothing.[^4]

The resolve never takes a lock and never retries. Two units that name one
deposit sit in one segment of the sorted intents, and the sort decides between
them.

### D2. The intents sort by the deposit, then by the identity of the unit

The ordering field packs the tile index and the resource kind into one bounded
integer. The tie-break field is the identity of the unit, which pairs a slot
index with a generation.[^5] No two intents tie, so the order has exactly one
correct output.[^6]

The tie-break is the identity and never the slot index. A slot is reused after
a unit dies, so a new unit would otherwise inherit the position that the dead
unit held in a contest for a deposit. The project has already recorded one
defect of that shape, where a random draw was keyed on the slot rather than on
the identity.[^7]

The sort runs on one thread. The result therefore depends on the key values
alone, and never on which thread finished first.[^8]

### D3. The resolve runs after the barrier of its frame

A unit takes from the tile it stands on, and the frame may have moved it. The
resolve therefore runs after the structural changes of the frame have passed
their barrier.[^3]

The resolve is not itself a barrier and does not create one. It writes a load
into a unit column and an amount into the depletion ledger. Neither moves a
unit, so nothing derived from where the units stand goes stale.

### D4. What a unit carries is a column of the unit, not a side table

The load lives in the column set of the unit shape, beside its tile and its
faction.[^9] It is plain data, so it enters the state hash without a
conversion.

A reused slot starts with an empty load. A load left behind by the dead unit
would arrive in the hands of the next unit, and the conservation rule would
then balance against a unit that never gathered.[^10]

Only the resolve raises a load. A caller that could raise one directly would
break conservation in silence, so no public call does.

## Consequences

**Who takes the last of a deposit is decided by identity.** A reviewer can
check the rule by reading the key. A unit with a lower identity beats a unit
with a higher one, on every machine and at every thread count.

**A unit that reaches an empty deposit takes nothing and reports nothing.** The
log holds one event for each grant, so a refused unit is visible as an absence.
A later need for an explicit refusal is a new decision.

**The resolve grants a whole rate or what is left, never a share.** The engine
does not divide a deposit between the units that want it. Division would need a
rounding rule, and a rounding rule over an integer amount is where a resource
gets created or lost.

**A unit gathers from wherever the frame left it.** A unit told to gather while
it also moves will take from a different tile each frame. Making a unit stay
still to gather is unit behaviour, and it is not this record.

**The gather rate is one number for every unit and every ground.** A rate that
depended on the unit type or on the ground is a content decision that no need
has been recorded for.

## Alternatives rejected

**Lock the tile and let the units contend.** This is the shape that every
engine without a determinism constraint uses. It is rejected because the
winner is whichever thread arrived first, and that is not a function of the
world.

**Give each unit its share of the deposit.** A deposit of seven divided between
three units needs a rounding rule, and the remainder has to go somewhere. It is
rejected because every such rule either loses the remainder or invents one, and
conservation is the invariant this subsystem exists to hold.

**Step every deposit each tick and hand out what it produces.** This makes the
cost follow the number of deposits, which is the number of tiles. It is
rejected on the shape of the cost: the term that grows with the number of
things dominates the term that grows with the number of tiles.[^11]

**Resolve gathering before the movement of the frame.** This removes the
dependence on the barrier and lets the resolve run beside the intent half of
movement. It is rejected because a unit would then take from the tile it left,
which is a wrong answer that repeats perfectly and that no determinism test can
see.

**Break the tie on the order the intents arrived in.** The intents are built in
slot order, which is stable, so this looks sound. It is rejected because the
build runs on many threads and joins their outputs, and because the slot order
is not the identity order after a slot is reused.

## References

[^1]: ADR-0001, one binary gives one answer at any thread count, decision D5. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^2]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^3]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^4]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4, a draft record. `docs/adrs/draft/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^5]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^6]: ADR-0007, content supplies a key vector, never a comparator, decision D2. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^7]: Testing Rules, a determinism test cannot tell correct from consistently wrong. `.claude/rules/testing.md`
[^8]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^9]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^10]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D5, a draft record. `docs/adrs/draft/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^11]: Findings register, FND-049. `docs/FINDINGS.md`
