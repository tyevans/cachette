# ADR-0126: A trade negotiation is engine state, and the words are not

## Context

A downstream game asks two players to make contractual trades with each other,
and to counter an offer before they agree. The project owner named the shape:
an offer, a counteroffer, and a terminal refusal that says no and asks for no
more counteroffers.

The engine holds four entity shapes and a faction. A faction owns units and
ground, and the engine keeps a running total of both for each one.[^1] The
world holds at most 63 factions, and that ceiling was chosen so that a relation
between factions is one plane.[^2]

**A trade has two halves and they are not the same kind of thing.** The
negotiation is a conversation. The contract is an obligation that the world
enforces, because goods move when it says so. The contract is simulated state,
because a later frame reads it and moves a quantity from it.

The question this record answers is where the conversation lives. A control
plane could hold it. The conversation is small, it changes between frames, and
one of the players is a language model that already keeps a transcript.

**Two facts refuse that.** A contract binds future delivery, so it is engine
state whatever happens to the conversation. And the acceptance that turns a
conversation into a contract is the last act of the conversation. Splitting
them puts one fact in two places, with nothing that fails when the copies
disagree. That shape has a local instance in this project already.[^3]

A second question follows. The project owner stated one rule of the game in
full: a player sends a message to another player only while one of its own
units stands in that player's territory. A trade is a thing two players say to
each other, so the rule either governs it or it does not, and nobody has
written down which.

A research report says that the engine should hold the gate and hold nothing
else about a conversation.[^4] It also says that the engine holds no text, no
channel, no delivery and no ordering between players, and that it should hold
none of them, because none of it is simulated state and all of it would have to
be hashed if it were.[^5]

## Decision

**The engine holds what two factions agreed. It holds nothing that either of
them said.**

### D1. The negotiation and the contract are one plane over ordered pairs of factions

One row holds one ordered pair. The row for the pair `(A, B)` holds the
negotiation that A opened toward B, the terms as they now stand, what each
party has delivered, and the status.

The plane never follows the population. Its size is the square of the faction
ceiling, and the ceiling is a property of the mask that holds a relation
between factions.[^2] A world with one unit and a world at the target
population hold the same plane.

The plane enters the state hash. A contract that a later frame reads is
simulated state, and two worlds that hold the same tiles and different
contracts must give different hashes.[^6]

The plane holds no row until a party speaks. A world in which nobody traded
allocates nothing and folds nothing into the hash. This is the shape the sparse
upgrade store already uses.[^7]

**A walk over the plane reads it in pair order.** No pass over it reads a hash
order and no pass over it names a thread.[^8]

### D2. The terms are engine state. The words are control-plane state

The engine holds the consideration of each side, the term, the deadline and
the status. A consideration is a tagged kind, and a resource kind with a
quantity is one of the kinds.[^11] Every field of every kind is an exact
integer, so no term of a contract is a floating point number.[^9]

The engine holds no text. There is no channel between two players, no message
body, no ordering of messages, and no delivery. A player that wants to say why
it refused says it in the control plane, beside the verb it called.

This is the same split the project already states for a message between two
players.[^5]

### D3. Every speech act passes the presence gate, and a bound contract does not

An offer, a counteroffer, an acceptance, a refusal and a terminal refusal each
require that a live unit of the speaker stands on ground the listener holds.
The engine refuses the act otherwise, and it says so.

**A contract, once bound, needs no presence.** An obligation outlives the
messenger who carried it. Requiring presence for delivery would make a contract
unenforceable the moment a player pulled its units back, which is the opposite
of what a contract is for.

The gate reads primary state. It walks the unit column and reads the holder of
the tile each unit stands on. It stores no answer, so it is not a second copy
of a fact that a derived relation also holds.[^3]

### D4. One unordered pair holds at most one live negotiation

Two players discuss one thing at a time. A pair with a live negotiation, or a
live contract, refuses a new offer in either direction.

The bound is what keeps the plane readable. Without it a pair holds two
conversations with two sets of terms, and the answer to "what are we
discussing" needs a rule that nobody has stated.

The plane stays ordered all the same, because the row records which party
opened the pair, and the terminal refusal of the next record is directional.

### D5. The engine answers any pair, and privacy belongs to the control plane

The engine holds no notion of who is asking. It answers the status of any
ordered pair to any caller.

Adding a notion of the asker would be an authentication model, and the engine
is not the place for one. A game that keeps a negotiation private between its
two parties hands each player a view of its own rows, and the read is shaped
for that: it takes a faction and answers the rows that faction is a party to.

**This is a statement about the engine and not about the game.** Whether a
player may learn of a negotiation it is not party to is a rule of the game, and
a blocker holds it.[^10]

## Consequences

**A negotiation cannot hold a reason.** A player that refuses says nothing in
the engine about why. Every explanation is control-plane text, and two players
that disagree about what was said have no engine record to settle it.

**A pair cannot run two deals at once.** A player that wants food for wood and
stone for wood from one partner makes one contract and then another.

**A player with no unit in another player's territory cannot trade with it at
all.** The gate is the whole diplomacy of the game, and it applies here without
exception. A player that withdraws its units ends every conversation it could
have.

**The gate costs one column read for each live unit, for each speech act.** A
speech act happens between frames and the cost does not enter a frame. A
derived presence relation would make the same answer one bit, and this decision
does not depend on which of the two answers it.[^4]

**The plane is a fixed size and a small one, so nothing about trade can be made
slow by a large world.** The cost of the whole diplomatic state of a world does
not change when a unit is born.

**A contract enters the state hash, so a golden hash moves when a world
trades.** A world that never traded is unaffected, because the plane holds no
row.

## References

[^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^2]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^3]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^4]: Research report 21, what a god needs from this engine, section 2. `docs/research/reports/21-what-a-god-needs.md`
[^5]: Research report 21, what a god needs from this engine, section 5.3. `docs/research/reports/21-what-a-god-needs.md`
[^6]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^7]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
[^8]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^9]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^10]: Blockers register, BLK-121. `docs/BLOCKERS.md`
[^11]: ADR-0147, a contract consideration is a tagged kind, decision D1. `docs/adrs/accepted/adr-0147-a-contract-consideration-is-a-tagged-kind.md`
