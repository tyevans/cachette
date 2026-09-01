# ADR-0080: A depleted deposit recovers by ageing the stored take, never by a pass over the world

## Context

A tile of this world holds a stock of each resource kind. A unit takes an
amount from the tile it stands on, and the amount the tile holds falls. Until
now the amount never rose again, so every resource in the world was a budget
that only fell.

The world stores no map of stocks. The stock a tile started with is a pure
function of the world seed and the tile address, in the same way the ground
is.[^1] The engine stores only what units took, and it stores that for a tile
only when somebody took from it.[^2] A world in which nothing was gathered
stores nothing at all.

Two facts make the choice hard.

The first is the target scale. The world holds many millions of tiles, and a
rule that visits each of them on each tick pays the world for every deposit
that nothing is touching. The engine refused that shape once already, for
gathering.[^3]

The second is determinism. One binary must give one answer at any thread
count, and the state hash covers the whole world on every frame.[^4] A recovery
that ran in a different order, or that rounded, would break that quietly. A
defect of this shape repeats perfectly, so both determinism tests pass over
it.[^5]

Two values that recovery needs are judgements and not measurements. How long a
kind takes to recover, and whether a deposit that reached nothing recovers at
all, are open rows that the owner may answer either way.[^6] [^7] No
measurement exists on the target platform, so this record states a shape and no
cost figure.[^8]

## Decision

### D1. Recovery ages the stored take, and never grows an amount

The world holds a record that a unit took from one deposit. Recovery makes that
record smaller. The amount a deposit holds stays what the generator gave it,
less what the ledger still says was taken, so a smaller stored take is a fuller
deposit.

Recovery therefore adds no storage. It creates no entry for a tile that nobody
gathered from, and it never raises a stock above what the generator gave that
tile. The bound follows from the arithmetic, not from a check that a later
change could drop.

### D2. The cost of recovery follows the depleted set, and never the tile count

The recovery pass reads the stored takes and nothing else. It takes no grid, no
extent and no tile count, so it cannot read a tile that holds no stored take.

A world in which nothing was gathered holds no stored take, so one pass over it
does no work, at any tile count. Two worlds that differ only in extent, and
that hold the same worked deposits, do the same work.

This forbids the obvious alternative. A pass that steps every deposit on every
tick is rejected by this decision, and a contributor may not add one for
convenience.

The depleted set must be able to shrink as well as grow. An entry that owes
nothing describes a deposit that is whole again, and such an entry is a fact
the world no longer needs. Removing it is separate work, and this decision
requires only that the shape permits it.

### D3. Recovery is exact whole-number arithmetic, in the order the ledger holds

The elapsed ticks divide by the period of the kind, and the result is a whole
number of units. No float and no fraction takes part, so a total over the
deposits combines the same in any order.[^9]

The pass walks the depleted set in key order, which is the order the ledger
holds its entries in. The order never comes from a thread, and never from the
order in which the entries arrived.[^10]

The remainder of the division survives. The pass advances the clock of an entry
by the whole periods it spent, and never to the tick it ran at. A pass that
advanced the clock to the tick would recover nothing at all when it ran on
every tick and the period was longer than one tick.

Recovery draws no random number. It reads the stored take, the tick and the
period of the kind, and nothing else.

### D4. Recovery runs before the gather resolve of the same frame

A unit takes what the deposit holds at the tick it gathers on. The step
therefore ages the stored takes before it resolves the orders to gather, so the
resolve reads the recovered amount and never a stale one.[^3]

A read of the world moves nothing. The amount a deposit holds at a tick is a
function of the stored take, the tick and the period, so two readers at one tick
get one answer.

### D5. The recovery period is a parameter of the resource kind, in one place

Each kind states one period, or states that it does not recover. A caller
replaces the whole rule set rather than one value, so no second site holds a
period and nothing can disagree with the first. A second declaration site with
no check between the copies is the defect shape this project meets most
often.[^11]

The absent case is a real case. At least one kind states no period, so the
engine carries the case that a deposit never recovers from the first day, rather
than gaining it later and then discovering that the shape does not hold.[^6]

A period is stated in simulated time and converted to ticks in one place. A
period stated in ticks alone would state something false the moment the span of
a tick moved.[^12]

The values of the periods are not part of this decision. The owner may change
one without superseding this record, because a period is a parameter and this
record states the shape that holds whatever the parameter is.[^6] [^7]

## Consequences

The project cannot add a rule that raises the stock of a tile directly, because
no stock is stored to raise. Anything that must make a tile richer than the
generator made it needs a different mechanism, and that mechanism needs its own
record.

The project cannot step every deposit on a tick. A feature that needs a
neighbourhood, such as a recovery that reads whether a neighbouring deposit
still holds something, does not fit this decision as written and must state the
cost it adds.[^7]

The state hash of every world that has gathered changes, because the stored
take now carries the tick it was brought up to date at. The golden files move
with this change.

A conservation check can no longer compare the stored take against what the
units hold, because recovery gives a part of the take back to the tile. The
check reads the returned total as a second term.

A deposit that has recovered fully is not different from a deposit that nobody
touched. A watcher cannot tell the two apart, and neither can a gatherer.

## References

[^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D1. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^2]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^3]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decisions D1 and D3. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
[^4]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^5]: Testing rules, section 2. `.claude/rules/testing.md`
[^6]: Decisions register, DEC-049. `docs/DECISIONS.md`
[^7]: Decisions register, DEC-050. `docs/DECISIONS.md`
[^8]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^9]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^10]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^11]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^12]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
