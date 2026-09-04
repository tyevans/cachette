# ADR-0141: A weather pass moves water and never scales it

## Context

A product record states four properties that a weather condition must hold. It
must conserve what it should conserve, so that what leaves one place arrives at
another exactly, with no loss to rounding and no gain. It must be bounded. It
must not fall below zero. It must give the same answer from one seed, at every
thread count, on every run.[^1]

The obvious kernel does not hold the first property. The influence field
relaxes by taking a weighted average of a cell and its neighbours, and the
weights sum to less than one, so an unforced field falls to nothing.[^2] That is
correct for influence, which is a reach and not a quantity. A quantity that
leaks has no account, and no reader can say where the missing part went.

**The engine already holds one quantity that must balance.** A resource is
generated on a tile, taken by a unit, carried, and delivered. The world keeps
running totals for the load that a dead unit took out of the world and for what
every delivery moved, because conservation must still balance and the world
records where the quantity went rather than letting it disappear.[^3] The
conservation check is what fails when the accounts disagree.

Weather also has to end. A storm that conserves and never leaves would fill the
world and stay. So conservation and decay both have to hold at once, and a
kernel that scales the quantity cannot do both.

## Decision

### D1. A spread pass is a gather of exact integer transfers

**A cell hands each neighbour a truncated integer share of what it holds, and
the receiver adds that same integer.** Both ends of an edge compute the share
from the same input plane with the same truncating division, so the two agree
exactly. The remainder stays with the giver.

The pass is written as a gather. A cell computes what it keeps and what each
neighbour hands it, and it writes only itself, so a parallel pass writes
disjoint output and needs no atomic operation.[^4] The cells are visited in
ascending index and the neighbours in direction order, and both orders are
fixed.[^5]

The share is chosen so that a cell with the largest possible number of
neighbours still hands away less than it holds. The field therefore never goes
below zero, whatever the input.

The alternative is the weighted average that the influence solve uses. It is
rejected because it loses quantity that nothing accounts for.

### D2. The account is exact, and a check reports it

**The field holds two running totals beside the two planes.** One counts every
drop that has ever entered the air. The other counts every drop that has ever
left the ground. The sum of the air, the ground and the second total equals the
first total at every moment.

The world invariant check reads this, in the way it reads the resource
account.[^3] A pass that scaled the quantity rather than moving it fails the
check, and nothing else would report it.

Decay is expressed inside the account rather than against it. Water leaves the
air onto the ground, and it leaves the ground into the second total. Nothing is
lost, and a storm still ends.

### D3. The solve runs a fixed number of passes

**The solve runs the same number of spread passes whatever the field holds and
whatever the thread count.** It reads no clock, it tests no residual, and it
takes no branch on whether a storm exists.

A convergence test makes the pass count depend on the arithmetic, and the
parallel reduction it invites makes the result depend on the thread count. The
project already made this decision for the influence solve, and this record
takes the same one for the same reason.[^2]

The one branch the solve does take is on whether any water exists at all. That
branch reads a fact of the world and not a residual, and it changes no answer:
a field with no water spreads nothing, whether the passes run or not.

## Consequences

The engine cannot express a weather quantity that is not conserved. A rule that
wants water to appear must raise the running total in the same operation, so
every source is visible in one place.

Truncation makes a small quantity stand still. A cell holding fewer drops than
the divisor hands away nothing, so ground that is barely wet stays barely wet
until something else moves it. That is exact rather than approximate, and it is
the price of an integer transfer.

The field cannot be made to decay faster by scaling. A designer who wants a
storm to end sooner changes the share that leaves, and the share that leaves is
still accounted.

The spread is isotropic. There is no wind, so a storm spreads the same way in
every direction. Nothing in the engine reads a direction of weather, and a
decision register row holds the choice.[^6]

## References

[^1]: PRD-0004, the world has weather that a watcher can read, what good looks like. `docs/product/accepted/prd-0004-the-world-has-weather-that-a-watcher-can-read.md`
[^2]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
[^3]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D5. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^4]: ADR-0009, parallel stages write disjoint outputs, decisions D1 and D2. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^5]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^6]: Decisions register, DEC-236. `docs/DECISIONS.md`
