# ADR-0149: A faction's trade board is simulated state that any faction may read

## Context

A faction that has a surplus and no partner cannot trade. The negotiation verbs
name the other party, so a god offers blind and most offers are refused. The
product record asks for a post that names no partner: what the faction offers,
what it wants, and what it asks in return.[^1] The design calls the table of
those posts the board.[^2]

This record exists because a contributor could reasonably build the board in
two other places. **The control plane could hold it.** A board is a statement
and not a mechanism, so a Python dictionary keyed by faction would serve a
human player. **The board could be private to a pair.** A post could be shown
only to the faction it was posted toward, in the way a negotiation is held
between two parties.

The choice costs more to change later than to record now. The board is inside
the state hash, so a later move of the board to the control plane or a later
change of its layout changes every golden file and needs a supersession. The
code shows that the board is hashed and does not say why it is inside the
simulation at all. That reasoning is what this record carries.

Two forces fix the answer. **A faction that acts on its own reads the board.**
The faction controller runs inside the step and acts only through the caller's
verbs.[^3] A board held outside the simulation would be unreadable to it, and a
controller that could not read a want could not offer where a want is posted.
**A post is a public statement.** The product record says every god reads every
post, and that secrecy is a separate need.[^1]

## Decision

**A faction's board is simulated state. Any faction reads any board at no cost.
A write replaces the whole board. The board is bounded by a world parameter,
and the controller writes its rows only through the verb that Python uses.**

### D1. The board is simulated state and every byte enters the hash

The world holds one table of boards, one block of rows for each faction. Every
field of every row is a whole number.[^4] Every byte of every row enters the
state hash, in faction order, and a declared padding byte is always zero.[^5]
The Python read is a copy of the rows and never the store.

Two worlds that differ in one row of one board have different hashes. A world
in which no faction has posted hashes as it did before the board existed, so a
golden file of such a world does not move.

A reviewer finds a violation when a board, or a copy that a pass reads, lives
outside the world, when a row holds a value that the hash does not fold, or
when a padding byte can hold anything other than zero.

### D2. A world parameter bounds the board

One faction holds a fixed number of rows. The row count is a balance value and
the balance register holds it.[^6] The table holds the row count times the
faction count and nothing else. It never grows with the population and never
grows with the number of posts, because a write that names more rows than the
bound is refused and changes nothing.

A change to the bound empties every board, because the blocks are laid out by
the bound. That is a consequence of the layout and not a choice this record
defends. A later record may lay the table out another way and keep this
decision.

A reviewer finds a violation when a board holds more rows than the bound, when
the bound is read from anything other than the world, or when a write past the
bound raises the row count.

### D3. A write replaces the whole board

The write verb takes the full list of rows for one faction. It writes them in
the order given and empties the rest of the block. There is no verb that adds
one row, and no verb that removes one row.

The reason is order. A partial update needs a rule for which slot a new row
takes and which row leaves when the board is full. Two partial updates to one
board in one tick would then need an order between them, and that order would
be a second place where determinism can fail.[^5] A whole-board write has no
such rule. The board after the write is the list the caller gave, and nothing
else.

A refused write changes nothing. The verb checks every row before it writes
any, so a caller reads the board after a refusal and finds it as it was.

A reviewer finds a violation when a verb changes one row of a board without
naming the rest, or when a refused write leaves a board partly written.

### D4. Any faction reads any board at no cost to standing

The read verb takes a faction number and answers that faction's board to any
caller. Reading passes no presence gate, moves no relation, and writes no
event. The engine holds no notion of who is asking.

**A board is not a negotiation.** A negotiation is between two parties, and
whether a third party may read it is a question the project owner holds.[^7]
That blocker still governs the negotiation, and this record does not close it.
A board differs from a negotiation in one way that decides the matter. A
negotiation names the other party. A board names nobody. A post that only one
faction could read would be an offer to that faction, and the offer verb exists
for that.

The product record makes the same distinction. It asks that every god read
every post, and it states that hiding a post is a separate need.[^1]

A reviewer finds a violation when a read of a board depends on who asks, when a
read moves a relation or a standing, or when a read starts a pass over the
world.

### D5. The controller writes its rows only through the verb that Python uses

When the faction controller posts a board, it calls the same write verb that a
Python caller calls, and the same refusals apply.[^3] No write path exists for
the controller alone.

**No controller writes a board yet.** The design says the controller writes its
rows from its site economies on a schedule, and the schedule is a balance value
that nobody has set.[^2] [^6] The write verb exists and Python calls it. The
controller does not. A reader of the code sees a board that only the control
plane fills.

A reviewer finds a violation when the controller writes a row through any path
other than the write verb, or when the write verb checks who is calling it.

## The alternatives this rejects

**A board in the control plane.** Python would hold the posts and hand them to
each player. Rejected because the controller runs inside the step and cannot
read Python state, so a faction that acts on its own could never find a want.
Rejected also because two control planes over one world would hold two boards,
and the world would not know which one a controller acted on.

**A board private to a pair.** A post would be readable only by the faction it
was posted toward. Rejected because a post toward one faction is an offer, and
the offer verb already exists. Rejected also because the engine holds no
notion of who is asking, and a notion of the asker would be an authentication
model.[^7]

**An unbounded list of posts.** A faction would post as many rows as it likes.
Rejected because the table would then grow with behaviour and not with a
constant, and the cost of a read would grow with it. The product record asks
that the whole market be bounded by two constants.[^1]

**A per-row write verb.** A caller would add or remove one row. Rejected under
D3, because a partial update needs a slot rule and an order between updates,
and a whole-board write needs neither.

## Consequences

**A faction can be found.** A faction posts a want without naming a partner,
and any other faction reads it. That is the need this record serves.[^1]

**The board is in every golden file that holds a post.** A change to the row
layout or to the hash fold moves every such golden file. A reader who wants to
change the layout writes a record that supersedes this one.

**A post makes no promise.** A board is a statement. The negotiation and the
contract are the only things that bind, and a faction that posts and then
refuses the offer that answers it has broken nothing. A row names a resource
kind and a quantity, and a contract that answers it may be priced in any kind
the consideration tag set names.[^8]

**A change to the bound empties every board.** A game that resets the row
count mid-run loses every post. That is the price of a layout that holds one
block for each faction.

**A controller that posts is later work.** The write path is fixed by D5, and
the schedule waits on a balance value. Until that work arrives, only the
control plane fills a board.

**Secrecy is a separate need.** A game in which a player hides a post is a
different game, and this record does not serve it. The blocker on who may read
a negotiation stays open and governs the negotiation alone.[^7]

## References

[^1]: PRD-0050, a god advertises what it will trade. `docs/product/accepted/prd-0050-a-god-advertises-what-it-will-trade.md`
[^2]: Design, the living world game layer, section 4. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^3]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^4]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^5]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^6]: Balance register, board size and advertisement schedule. `docs/reference/balance.md`
[^7]: Blockers register, BLK-121. `docs/BLOCKERS.md`
[^8]: ADR-0147, a contract consideration is a tagged kind, decision D1. `docs/adrs/accepted/adr-0147-a-contract-consideration-is-a-tagged-kind.md`
