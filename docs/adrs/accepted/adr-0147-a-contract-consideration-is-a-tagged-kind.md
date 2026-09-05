# ADR-0147: A contract consideration is a tagged kind

## Context

A contract between two factions binds each of them to deliver something to the
other. Before this record the something was a quantity of a resource, and a
unit carried it onto the ground of the other party.[^1] The negotiation and the
contract are one plane over ordered pairs of factions, and the delivery pass
reads that one plane.[^2]

The game layer needs two more things a faction can trade. A faction trades
land, so that a border moves by agreement and not only by war. A faction trades
a relation step, so that a treaty is a trade whose price is goodwill.[^3]

**The shortest path is a second table.** A land contract has a tile set and no
quantity, so a contributor writes a land plane beside the trade plane, and a
treaty plane beside that. Each plane then needs its own status machine and its
own delivery path.

Two facts refuse that. **The delivery pass reads one plane.** It walks the
rows, sorts the deliveries and transfers in a stated order.[^4] A second plane
needs a second pass, and the two passes must agree on the order in which a
contract settles, or a land contract and a resource contract on one pair settle
in an order that depends on which pass runs first. **One pair holds one live
negotiation.** That bound is what keeps the plane readable, and a second plane
breaks it silently, because the check that refuses a second offer reads only
the first plane.[^5]

This record widens ADR-0128 D1. That decision says the only path from one
faction's goods to another's is a unit that carries a load. This record keeps
that for a resource and states how the two kinds that no unit can carry
deliver.

## Decision

**Each side of a contract is one consideration. A consideration is a tagged
kind with content that the tag names. One plane holds every contract, one
status machine governs it, and one delivery pass settles it.**

### D1. A consideration is one tag and the content the tag names

The tag is one of a small closed set. The set holds a resource, a land set and
a relation step. The content of a resource is a resource kind and a quantity.
The content of a land set is a bounded set of tiles that the debtor of that
side holds. The content of a relation step is a step kind and an amount.

Every field of every kind is a whole number.[^6] Every side of every contract
is in the one plane, and every byte of every side enters the state hash. The
tag is the only thing a pass reads to find out what a side holds.

A reviewer finds a violation when a second table holds a contract of any kind,
when a side of a contract sits outside the state hash, or when a pass branches
on anything other than the tag to find the content.

### D2. A resource delivers as before, and a unit carries it

A resource consideration delivers exactly as ADR-0128 states. A unit of the
debtor carries the resource onto a settlement tile the creditor holds, and the
pass transfers it against the debt.[^1] Nothing in this record changes that
path.

### D3. A land set and a relation step deliver on full delivery of the other side, without a carrier

No unit can carry a tile or a feeling. A land set and a relation step therefore
have no journey. A side that no unit carries applies in the tick in which the
other side's debt reaches zero. It applies in the same pass, after every
carrier of that tick has delivered, and before the deadline check. The pass
walks the plane in pair order, so two land sets that name one tile apply in
pair order and in no other.[^2] When neither side is carried, both sides apply
on the first pass after the contract binds.

A land set applies by changing the holder of every tile in the set to the
creditor, in ascending tile index.

A relation step applies by moving the relation entry for the pair by the
step.[^7] **No pass moves a relation yet.** The relation matrix does not exist,
so the pass records that the side fell due, marks it delivered, and changes
nothing else. The record of the delivery is the event, and the event is what a
later pass replaces with the move.

This is the change to ADR-0128 D1. A quantity still moves only when a unit
carries it. A consideration that is not a quantity moves when the quantity it
was priced against has arrived.

### D4. The engine refuses a land side whose tiles its debtor does not hold

Every verb that states the terms checks every tile of a land side against the
holder column, for the debtor of that side. It refuses when any tile is held by
another faction, or when any tile is held by nobody. The list bound is a balance
value.[^8]

**Whether an upgrade on a traded tile goes with the tile is open, and the
project owner holds the question.**[^9] Until the blocker closes, the engine
refuses a land side whose tiles carry an upgrade. The refusal is written so
that one commit removes it when the answer arrives.

### D5. The status machine is unchanged

Offer, counter, accept, refuse, terminal refusal, close and reopen keep their
meanings and their code paths. A terminal refusal closes a direction for every
kind.[^10] A contract with a debt on either side at the deadline fails for
every kind, and what already moved stays where it arrived.[^11]

A land set and a relation step cannot be partly delivered, and neither owes
anything until the other side has arrived. A contract that prices land against
a resource therefore fails only when the resource side is short, and only the
party that owed the resource loses its direction.

## The alternatives this rejects

**A second plane per kind.** Rejected because the delivery pass depends on one
plane, and because the one-live-negotiation bound would then be checked
against one plane and broken by another.[^5]

**A generic quantity with a unit field.** Every consideration would be a
number and a code that says what the number is. Rejected because a tile set is
not a number and a code that says "these are tiles" is the tag with a worse
name.

**Land that moves tile by tile as units arrive.** A land set that transferred
one tile per carrier journey. Rejected because a tile is not carried, and
because the shape would put a holder change in the middle of a pass that
sorts by settlement.

**A treaty as a separate verb.** Rejected because a treaty is a trade with a
different consideration, and a separate verb would need a separate status
machine and a separate refusal path.

## Consequences

**A contract can be priced in anything the tag set names.** Land for grain, a
relation step for land, grain for grain. The status machine and the presence
gate apply to every one.

**The tag set is closed and small, and adding a kind is a change to this
record's set.** A new kind needs a delivery rule under D3, and a new kind that
a unit can carry needs a rule under D2.

**A land contract can change a border without a fight.** That is the need it
serves. The holder change repairs the derived parts of the holding at the
transfer, and the presence relation rebuilds at the end of the step as it does
for every holder change.[^12]

**A relation step is a promise the engine records and does not yet keep.** A
contract priced in a relation step settles, and the relation does not move.
The pass that adds the relation matrix must make the recorded delivery move
the entry. Until it does, a reader of the event log sees the debt paid and the
relation unchanged.

**An upgrade on traded ground blocks the trade for now.** That is a parameter
under an open blocker and not a decision of this record.[^9]

## References

[^1]: ADR-0128, a contract moves a quantity only when a unit carries it onto the ground of the other party, decision D1. `docs/adrs/draft/adr-0128-a-contract-moves-a-quantity-only-when-a-unit-carries-it.md`
[^2]: ADR-0126, a trade negotiation is engine state, and the words are not, decision D1. `docs/adrs/draft/adr-0126-a-trade-negotiation-is-engine-state.md`
[^3]: Design, the living world game layer, section 4. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^4]: ADR-0128, a contract moves a quantity only when a unit carries it onto the ground of the other party, decision D2. `docs/adrs/draft/adr-0128-a-contract-moves-a-quantity-only-when-a-unit-carries-it.md`
[^5]: ADR-0126, a trade negotiation is engine state, and the words are not, decision D4. `docs/adrs/draft/adr-0126-a-trade-negotiation-is-engine-state.md`
[^6]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^7]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D3. `docs/adrs/draft/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
[^8]: Balance register, the land list bound. `docs/reference/balance.md`
[^9]: Blockers register, BLK-036. `docs/BLOCKERS.md`
[^10]: ADR-0127, a terminal refusal closes an ordered pair until a named tick, decision D1. `docs/adrs/draft/adr-0127-a-terminal-refusal-closes-a-pair-until-a-named-tick.md`
[^11]: ADR-0128, a contract moves a quantity only when a unit carries it onto the ground of the other party, decision D4. `docs/adrs/draft/adr-0128-a-contract-moves-a-quantity-only-when-a-unit-carries-it.md`
[^12]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decision D1. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
