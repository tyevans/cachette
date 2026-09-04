---
id: 0034
title: Two players hold each other to a future delivery
status: Shaped
created: 2026-09-03
---

# PRD-0034 — Two players hold each other to a future delivery

## Who this is for

A developer who builds a strategy game in which two players deal with each
other, and whose players are people or language models.

One real project is that developer. In it a player directs a congregation, and
the project owner asked for contractual trades between players, with
counteroffers, and with a refusal that says no and asks for no more
counteroffers.

The audience is also the player. A language model that plays one side reads the
state of a deal and decides what to say next. What that model can read is what
it can play.

## What the person cannot do today

**Two players cannot agree on anything.**

The control plane creates units, removes them, tells a set of units to collect
one kind of resource, founds a settlement, changes what a settlement prefers,
and sets how often one schedule runs. Nothing in that set involves two players
at once.

**Goods have no path between two players.** A unit collects from the ground and
delivers to its own side. There is no way for what one player holds to become
what another player holds, short of one of them removing the other's units.

**A promise cannot be recorded, so it cannot be broken.** A developer that
builds an agreement in its own program holds it outside the world. The world
then does not enforce it, does not fail it, and does not report it. Two
programs that disagree about what was agreed have nothing to appeal to.

**A refusal cannot say how final it is.** This is the sharpest gap, and it is
the one that costs the most. A player that says no and a player that says never
produce the same silence. A language model that receives a no and cannot tell
whether asking again is allowed will ask again, and it will keep asking,
because nothing in the answer tells it to stop. That loop costs a token budget
rather than a frame, so nothing in the engine ever notices it.

## What good looks like

Each statement below can be checked.

- A player offers a deal to another player, and the other player sees the terms
  without the two of them agreeing on a format first.
- The other player answers with different terms, and the first player sees
  those instead. Either of them can do this any number of times.
- Either player agrees, and the world then holds both of them to the terms.
- Goods actually move between the two players because they agreed, and a
  watcher can see the goods leave one side and arrive at the other.
- A player refuses, and the two of them may open a new deal at once.
- A player refuses and ends the asking, and the other player's next attempt is
  refused and says when it may ask again.
- The player that ended the asking may allow it again before then. Nothing the
  other player does shortens it.
- A deal that is not kept by its date fails, and both players can read that it
  failed and who did not deliver.
- A player reads everything it needs to decide in one call, and never by asking
  about one deal at a time.
- No word either player writes reaches the world.
- The same deals on the same world give the same result at every thread count.

## What this does not do

**It does not carry a conversation.** The world holds what was agreed and never
what was said. A reason, an argument and a threat are the game's business.

**It does not price anything.** No exchange rate, no market and no valuation.
The two players state quantities and the world holds them to those quantities.

**It does not make a player able to keep a promise.** A player may agree to
more than it can deliver. Finding that out is what the failure is for.

**It does not undo what already moved.** A deal that fails halfway leaves what
arrived where it arrived.

**It does not judge a player that fails.** There is no standing, no rating and
no memory beyond the pause that a failure causes.

**It does not let a player deal with someone it cannot reach.** The rule that
governs a message between two players governs this too, and this record does
not change that rule.

**It does not hide a deal from anybody.** Who may learn of a deal is a rule of
the game, and this record states none of it.

**It does not state how the world answers any of it.** Whether a deal is a
table, a column or an event is an architectural question, and this record
states none of it.

## What it costs at the target scale

**The cost of the whole thing must not follow the population.** A world with
one unit and a world at the target population must pay the same for holding
every deal in it. The number of players is bounded and the number of pairs of
players is bounded, so the state of every deal in a world is bounded before
anybody plays.[^1]

**Moving the goods is the part that costs a frame.** A delivery must be found
among the units, so its cost follows the units that are delivering and never
the units that exist. A world in which nobody agreed to anything must do no
work at all.

**A read must cross once.** A player that wants to know where it stands with
every other player makes one call and receives one answer for each pair. A read
that answered one pair at a time would make a player pay for the number of
players it deals with, in calls.

**No figure is stated here.** One blocker governs every cost figure in this
project, and it says which figures are measured and which are derived.[^2] The
statements above argue about which term a cost follows. They are not results.

## Which blockers govern this

**One blocker governs every cost claim above.**[^2]

**One blocker holds the rules of the game this need came from.**[^3] The owner
named the shape of a trade in one sentence. What a deal may be about, how long
one runs for, and what it costs to break one are unstated. Work continues,
because each of those is a number the caller supplies rather than a shape the
world fixes.

**One blocker holds what happens at the edge of a run.** Nobody has said what a
deal means when the other player has nowhere left to receive.[^4]

**One blocker holds who may see a deal.**[^5] The world answers about any pair
of players to anybody who asks, and a game that wants a deal private keeps it
private outside the world. Whether the game wants that is unstated.

**One blocker holds the scale.**[^6] Nobody has said how large the downstream
game runs, and the argument about which term a cost follows is priced at the
target population.

## References

[^1]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^2]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^3]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^4]: Blockers register, BLK-120. `docs/BLOCKERS.md`
[^5]: Blockers register, BLK-121. `docs/BLOCKERS.md`
[^6]: Blockers register, BLK-051. `docs/BLOCKERS.md`
