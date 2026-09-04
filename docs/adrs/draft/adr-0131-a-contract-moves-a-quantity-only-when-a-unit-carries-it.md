# ADR-0131: A contract moves a quantity only when a unit carries it onto the ground of the other party

## Context

A contract between two factions binds each of them to deliver a quantity of a
resource to the other. The question is how the quantity gets there.

**The obvious answer is a transfer between two stores.** The engine holds a
pooled store for each settlement, and moving a quantity from one to another is
two writes and an account. It is exact, it is order-free at one store, and it
would be a short pass.

**The engine already moves a quantity, and it does not move it that way.** A
unit gathers into a carry column, walks home, and the delivery pass moves the
load into the store of its own site when it stands on that site's tile. The
resource loop had no sink before that pass existed, and the record that opened
it says why the store may not simply rise.[^1]

The downstream game is about presence and territory. Its one stated rule is
that a player speaks to another player only while one of its own units stands
in that player's territory. A trade whose goods appear in a store without
anybody carrying them makes the map decorative for the whole economy.

**A contract must also be able to fail.** A contract that cannot fail is not a
contract. So the record must say what happens when a party does not deliver,
and it must say it in a way that costs the defaulting party something the game
can see.

Two constraints bound the answer. A quantity that vanishes without a record
breaks the conservation equality, and nothing else fails when it does.[^2] And
two units delivering into one store meet a saturating add, which is not
order-free, so the transfers need a stated order.[^3]

## Decision

**A contract moves nothing. A unit carries the quantity onto the tile of a
settlement the other party holds, and the engine transfers it there.**

### D1. There is no transfer between two stores

No verb and no pass moves a quantity from the store of one settlement into the
store of another. The only path from one faction's goods to another faction's
goods is a unit that carries a load.

A unit of the debtor that carries the resource the contract names, and that
stands on the tile of a settlement the creditor holds, delivers what it carries
against the debt.

This is the same transfer the engine already performs for a unit standing on
its own site. The pass differs in the site it looks at and in the ledger it
credits.

**A contract therefore costs the debtor a journey.** The distance between two
players is a real cost of trading with them, and it falls out of the map rather
than out of a rate the engine invents.

### D2. A delivery is admitted by sort, then by transfer, and it never passes the debt

The pass collects the deliveries by walking the unit slots, then orders them by
the settlement and then by the identity of the unit, and only then transfers.
That is the order the ordinary delivery already uses.[^4] [^5]

The quantity moved is the smallest of three: what the unit carries, what the
party still owes, and what the store can still hold. A quantity the store
cannot hold stays in the carry, and the unit delivers it on a later tick. A
quantity above the debt is never moved, so a contract moves what it named and
no more.

The pass runs on the calling thread and writes one store at a time in a stated
order. It names no thread and it depends on no thread count.[^5]

### D3. The pass runs after the ordinary delivery and before the rate pass

It reads where each unit stands, so it runs after the barrier that the movement
of the frame passed. It moves a quantity, so it runs before the rate pass and
before the consumption pass, which two records require of anything that moves
one.[^6] [^7]

It changes no structure. It writes a store, a carry and a contract row, and it
moves no unit, so it is not a barrier and it needs none.

### D4. A contract fails at its deadline, and what already moved stays where it arrived

The contract carries a term, and the acceptance turns the term into a deadline.
The deadline is checked after the delivery of that tick, so a contract whose
deadline is this tick gets this tick's delivery.

A contract with a debt on either side at the deadline fails. The engine records
the failure and names the pair.

**Nothing is returned.** A quantity that already arrived stays in the store it
arrived at. Taking it back would need a transfer that no unit carried, which
D1 refuses, and a partial delivery that vanished would break the conservation
equality.[^2]

**The party that did not deliver loses the direction it would ask on, for as
long as the contract ran.** The closure is the same mechanism a terminal
refusal uses.[^8] The duration is the term of the contract itself, so no
balance figure decides it and no measurement can make it stale. A party that
breaks a long promise waits as long as the promise ran.

When both parties owe, both lose their direction.

## Consequences

**A player cannot trade with a partner it cannot reach.** A contract with a
faction on the far side of the world binds a journey that the game must make
possible. This is a real limit and it is the one the design chose.

**A player can bind itself to a contract it cannot keep.** The engine checks no
stock at acceptance, because the goods may be gathered during the term. A
player that promises more than it can carry defaults, and the default is what
tells it so.

**A contract can settle in either order and in any number of instalments.** The
debt falls as loads arrive, and the contract settles on the tick both debts
reach zero.

**The delivery reads a settlement, so a party with no settlement can receive
nothing.** A contract toward a faction that loses its last settlement runs to
its deadline and fails, and whether the game wants that is a rule nobody has
stated.[^9]

**A default costs the defaulting party nothing but time.** There is no
reputation, no fine and no seizure. Whether the game wants a heavier cost is
open.[^10]

**The pass costs a walk over the unit slots on every frame in which any
contract is bound.** A world with no bound contract does no work at all,
because the plane holds no row and the pass returns at once.[^11]

## References

[^1]: ADR-0062, production and upkeep are rates attached to a site, decision D2. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
[^2]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D5. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^3]: ADR-0062, production and upkeep are rates attached to a site, decision D3. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
[^4]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D2. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
[^5]: ADR-0004, iteration order is explicit, decisions D1, D3 and D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^6]: ADR-0062, production and upkeep are rates attached to a site, decision D5. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
[^7]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D5. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
[^8]: ADR-0130, a terminal refusal closes an ordered pair until a named tick, decision D1. `docs/adrs/draft/adr-0130-a-terminal-refusal-closes-a-pair-until-a-named-tick.md`
[^9]: Blockers register, BLK-120. `docs/BLOCKERS.md`
[^10]: Decisions register, DEC-212. `docs/DECISIONS.md`
[^11]: ADR-0129, a trade negotiation is engine state, and the words are not, decision D1. `docs/adrs/draft/adr-0129-a-trade-negotiation-is-engine-state.md`
