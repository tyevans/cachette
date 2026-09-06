# ADR-0165: The wealth bar stands above what one settlement can hold

## Context

The engine ends a game on one of four paths, and one reader watches each
path.[^1] The wealth reader totals the stock of a faction and compares the
total against a bar. The bar is a balance value, and a register row holds
its derivation.[^2]

**A store is a fixed-point value of a fixed width, so it has a ceiling.** A
settlement holds one store for each commodity, and each store saturates at
the top of its type.[^3] The stock of a faction is the sum of those stores,
so the stock of a faction that holds one settlement has a ceiling too. The
ceiling is a property of the type and of the commodity count. It is not a
value a game designer picks.

**The store rises and does not fall, so it reaches its ceiling.** A
settlement produces more than it spends in the world the demonstration
builds, and nothing in the engine caps a store below its type. A bar under
the ceiling is therefore crossed by a faction that founds one settlement and
then waits.

The project owner asked for a wealth bar far out of reach, so that domination
and territory decide a game. A previous pass answered by raising the register
row. It raised the row toward the ceiling, measured a later end tick, and
reported progress. The path still ended every game of a seeded sweep. The
finding holds the measurement and the reasoning.[^4]

**The register cannot answer this, and the row invites a reader to think it
can.** Every value the row may hold is under the ceiling, because a value at
or above the ceiling fires never. So every value the row may hold is crossed.
The question is not which value the bar takes. The question is what the
reader totals.

## Decision

### D1. The wealth bar stands above the stock one settlement can hold

The reader totals the stock of every commodity of every live settlement of a
faction, and the bar stands above the ceiling of one settlement.

A faction therefore reaches the bar by holding more settlements, and never by
waiting at the settlement it founded. Wealth becomes a claim about a domain
and not a claim about a warehouse.

A reviewer finds a violation when the bar is at or below the ceiling of one
settlement.

### D2. The bar is a whole multiple of the share one settlement carries

The engine states the bar as a multiple of a per-settlement share. The share
is the balance value, and the register row holds its derivation.[^2] The
multiple is the settlement count a wealth win asks a faction for, and it is
greater than one.

This keeps a designer's control of the bar and removes the trap. A designer
raises or lowers the share. The multiple states how many filled settlements a
wealth win is worth.

**This changes the earlier record, which says that every threshold a reader
compares against is a balance value.**[^1] A threshold may now be a product
of a balance value and a structural constant. The balance register still
holds every value a measurement can change, and the ceiling is not one of
those, because the type states it.

### D3. A compiled check fails when the bar falls back under the ceiling

The engine derives the ceiling from the width of a store and from the
commodity count. A compile-time assertion compares the bar against the
ceiling, and the build stops when the bar is not above it.

The check is the reason a future contributor cannot undo this record by
editing a number. It also fires when the commodity count rises, because a
higher count raises the ceiling. The build then stops until a writer derives
the bar again.

A reviewer finds a violation when the assertion is removed, or when it is
replaced by a comment that names the rule.

### D4. The total stays a 64-bit accumulator

The reader combines each store into a 64-bit accumulator, as an aggregate
must.[^5] The bar now stands above the range of one store, so a narrower
accumulator would wrap before the bar and the reader would never fire.

A reviewer finds a violation when the total is held in the type of a store.

## The alternatives this rejects

**Raise the register row again.** Rejected because every value the row may
hold sits under the ceiling, and a store that rises crosses each of them. The
finding measured the whole distribution against the ceiling.[^4]

**Widen the type of a store.** Rejected on two counts. It moves the ceiling
and does not remove it, so the same argument returns at the wider ceiling.
And a store is read by production, by upkeep, by the build queue and by
trade, so the change reaches every one of those and moves the state hash. The
wealth bar is a poor reason to widen the arithmetic of the economy.

**Raise the commodity count.** Rejected for the same shape. A count of two
doubles the ceiling and leaves it a ceiling. The commodity count is an
economic decision and not a win-condition one.

**Widen the accumulator the reader uses.** Rejected because the accumulator
is already 64 bits, and the ceiling is not in the accumulator. It is in the
store the accumulator reads. This alternative is recorded because two earlier
readings named it as the cheap answer, and it is not an answer at all.[^4]

**Drop the stock clause and leave the wonder clause.** Rejected because a
wealth win is a path the design names, and a path with no reader is a path
that does not exist.[^1] A bar out of reach of one settlement is still a bar
a faction can pass.

**Scale the bar by the settlements a faction owns.** Rejected because a bar
that rises as a faction grows is a bar the faction never approaches. It also
punishes the expansion that a wealth win is supposed to reward.

## Consequences

**A wealth win now asks for expansion.** A faction that holds one settlement
cannot win on stock, whatever it produces and however long the run is.

**Nothing in the engine founds a second settlement today, so the stock clause
fires in no seeded run.** The controller has no found option in its choice
set, and a finding records that.[^6] The clause is therefore correct and
quiet. It is not inert code in the sense the defect rule names, because the
reader runs on every tick and a test drives it over a faction that holds
more than one settlement. A reader that fires only for a world the controller
cannot yet reach is a bar, and the project asked for a bar.

**The wonder clause now carries the wealth-or-wonder path in a seeded run.**
The path still ends some games, and the share is a reading the balance
register holds.

**A fixture that reaches the bar founds more than one settlement.** A test
that set one store to the bar can no longer do so, because the bar is outside
the range of a store.

**The commodity count is now load-bearing for the win condition.** A change
to it stops the build until a writer derives the bar again. That is the
intended cost.

## References

[^1]: ADR-0148, a game end is recorded once and stops the controllers, decision D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^2]: Balance register, the stock target. `docs/reference/balance.md`
[^3]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^4]: Findings register, FND-543. `docs/FINDINGS.md`
[^5]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^6]: Findings register, FND-542. `docs/FINDINGS.md`
