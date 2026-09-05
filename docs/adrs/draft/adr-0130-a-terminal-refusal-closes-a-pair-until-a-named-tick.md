# ADR-0130: A terminal refusal closes an ordered pair until a named tick

**This file is a copy of ADR-0127.** A number collision restored the three trade records under a second set of numbers, thirty-nine minutes after the first set. The registry names ADR-0127 as the record that replaces this one. Cite ADR-0127. This file stays because the registry keeps the file of a replaced record.

## Context

The project owner asked for contractual trades with counteroffers, and he named
one act in particular: "no and no more counteroffers". That act is different
from a plain no, and the difference is the reason this record exists.

**A plain no ends one negotiation.** The parties are free at once, and either
of them may open a new one on the next call.

**"No more counteroffers" ends the asking.** It is a statement about the
future, and the engine has to represent that future somehow.

One of the two players in the downstream game is a language model. A language
model that receives a refusal and cannot tell whether asking again is allowed
will ask again. It will keep asking, because nothing in the answer says to
stop. A refusal that reads the same as a closed door is a loop, and the loop
costs a token budget rather than a frame.

So the closure must be a fact the engine holds, and the caller must be able to
read it. **A rule that only the control plane knows is a rule that a second
control plane will not follow**, and the downstream game runs one player for
each faction.

A closure with no end is a poor fit for a game. Two players that fall out for
ever hold a permanent dead pair, and the game has no way back that does not
reach into the engine. A closure that any party can clear is not terminal at
all.

## Decision

**A terminal refusal ends the negotiation and closes the direction the other
party would open, until a tick that the refusing party names.**

### D1. A refusal closes nothing, and a terminal refusal closes a direction

Two verbs, and the difference between them is the whole point.

The refusal verb ends the live negotiation. The pair is idle afterwards, and
either party may open a new one on the next call.

The terminal refusal verb ends the live negotiation and writes a tick into the
row that the other party would use to open one. An offer made before that tick
is refused, and the refusal states the tick.

**The refusing party names the duration.** The engine invents no number. A
player that wants a short pause names a short one, and a player that wants a
permanent break names a duration that saturates the clock. A duration of zero
is refused, because a closure that closes nothing is a refusal wearing another
name.

### D2. The closure is directional, and it belongs to the party that wrote it

A closes the direction B would open. B cannot open a negotiation toward A until
the closure ends.

**A may still open one toward B.** A closed its own door to being asked. It
made no promise of silence of its own, and a rule that made one would turn
every refusal into a mutual embargo that neither party chose.

Only A clears the closure early. Nothing B does shortens it: not a message, not
a unit standing on A's ground, not a gift. That is what makes it terminal
rather than a delay.

### D3. The closure is readable, and every refusal of an offer names its tick

The row states the tick at which the direction opens again. A caller reads it
before it offers.

An offer that a closure refuses raises an error that states the tick. The
caller therefore learns the rule from the refusal itself and needs no second
read to act on it.

**A refusal that says only "no" is a defect of this design and not a style
question.** The whole reason the closure is engine state is that a caller can
act on it, and a caller that cannot see it is back where it started.

### D4. Time is the only thing that ends a closure, apart from the party that wrote it

No other event clears it. A new unit, a change of ground, a settled contract
elsewhere and a change of ruler all leave it standing.

The engine holds one clock, and a tick is a fact that every party reads the
same way. A closure keyed on anything else would need a rule that says which
change counts, and every such rule is a game rule that nobody has stated.

## Consequences

**A player can lock another player out of trade for the life of a run.** The
duration saturates, so a party that names the largest duration has closed the
direction for ever in practice. That is a power the game gives a player, and it
is the power the project owner asked for.

**A closure survives a change of circumstances.** Two players that fall out and
then have every reason to deal again must wait, or the refusing party must open
the direction on purpose. Nothing in the world does it for them.

**A pair can be half closed.** A holds a closed door toward B while B holds an
open one toward A. A caller that reads only one row of a pair reads half the
state, and the read is shaped to return both.

**The closure is one number in a row that already exists.** It costs nothing
that the negotiation plane did not already cost.[^1]

**A default reuses this mechanism rather than inventing a second one.** A
contract that fails at its deadline closes the direction against the party that
did not deliver, and the duration comes from the contract itself.[^2]

## References

[^1]: ADR-0126, a trade negotiation is engine state, and the words are not, decision D1. `docs/adrs/draft/adr-0126-a-trade-negotiation-is-engine-state.md`
[^2]: ADR-0128, a contract moves a quantity only when a unit carries it onto the ground of the other party, decision D4. `docs/adrs/draft/adr-0128-a-contract-moves-a-quantity-only-when-a-unit-carries-it.md`
