---
id: 0031
title: A god knows whose ground its people stand on
status: Shaped
created: 2026-09-03
---

# PRD-0031 — A god knows whose ground its people stand on

## Who this is for

A developer who builds a strategy game on this engine, and whose game turns on
where one side's people are standing.

The engine already gives each place an owner, and it already gives each unit a
side.[^1] This record is about the question that joins the two. It asks whether
the people of one side are standing on the ground of another side.

One real project needs it. In that game a god directs a congregation, and a god
may send a message to another god only while one of its own units stands in
that god's territory. The need is stated here in general terms, because any
game with sides and ground asks the same question.

## What the person cannot do today

**A developer cannot ask whether one side is present on another side's
ground.**

The engine holds every part of the answer. Each tile carries the side that
holds it. Each unit carries a side and stands on a tile. Each side carries a
running count of the ground it holds.

The control plane can reach one tile at a time. It reads a report for one
address, and the owner is one entry of that report. It reads the position of
one unit, for one identity that the caller already holds.

**Nothing lists the units of a side.** The control plane reads a population
count and no identities. So a developer must keep every identity the engine
ever handed back, in a list, for the life of the run.

This has three costs.

**The developer writes the loop the project forbids.** Asking the question
means visiting each unit, then visiting the tile under it. The control plane is
not a data plane, and a loop over the population is exactly what that rule
exists to prevent.[^2]

**The cost follows the population, and the question is asked often.** A game
that gates a conversation on presence asks it whenever a side wants to speak.

**The developer cannot tell a stale answer from a fresh one.** The engine
derives the position of a unit at a barrier. A caller assembling the answer by
hand has no way to know which barrier each part came from.

## What good looks like

Each statement below can be checked.

- A developer asks whether any unit of one side stands on ground that another
  side holds, and gets the answer in one call.
- The call visits no unit from Python and builds no object for each unit.
- The answer covers every pair of sides in the world, so a developer asking
  about all of them pays what a developer asking about one pays.
- The size of the answer does not change when the population changes.
- The answer states the world at the last barrier, and a caller who changed the
  population without stepping is refused rather than given a stale answer.
- The same world gives the same answer at every thread count and on every run.
- A developer who asks the question about a side that does not exist meets an
  error that names the side.

## What this does not do

**It does not say which units are standing there.** The need is answered when a
developer knows that somebody is present. Naming the units is a different need,
it is a set-valued read rather than one answer, and it costs a different thing.
A developer who wants the list is asking for something else.

**It does not carry a message.** Two sides exchanging anything is the game's
business, and the engine holds no channel, no text and no delivery. The engine
answers whether the game may allow the exchange, and nothing else.

**It does not say for how long.** The answer is about the present state of the
world. A game that wants presence over a period keeps that itself.

**It does not define territory.** The engine already decides which side holds a
tile, by a rule that reads the ground. This record changes none of that and
adds no second notion of ownership.

**It does not answer for a place the caller names.** The question is about the
ground of a side, not about an address, a radius or a shape. A developer who
wants a window already has one.

**It does not give a side a name.** A side is a number. What the game calls it
is the game's business.

## What it costs at the target scale

The engine holds far more tiles and units than a script can visit, and the
scale constants table holds the figures.[^3]

**The size of the answer is fixed by the side ceiling and by nothing else.**
The world admits a bounded number of sides, and the reference table states both
the ceiling and the reason it was chosen: a set of sides is one machine word.[^3]
The answer to this need is one such set for each side. So it is a small fixed
number of words, whatever the world holds.

**The cost of deriving it follows the population once, and it rides on work the
engine already does.** The engine already visits every unit and the tile it
stands on, once for each barrier, to keep the derived position of the
population. Reading the owner of that tile adds one read for each unit and
allocates nothing.

**The combining step is exact and order-free.** Joining two partial answers is
a union of sets. That has an identity, it is associative, and it is
commutative, so the result does not depend on how the work was split.

**No figure is stated here.** One blocker governs every cost figure in this
project, and it says which figures are measured and which are derived.[^4] The
statements above are arguments about which term the cost follows, not results.

## Which blockers govern this

**One blocker governs every cost claim above.**[^4] Nothing here was measured.
The claim that the answer is a fixed size is a property of the side ceiling and
not a measurement. The claim that deriving it costs one read for each unit is a
derivation from the passes the engine already runs.

**One blocker governs the game this need came from.** The rules of that game
are one paragraph.[^5] The need above is stated in general terms so that it
does not wait on them. If the game turns out to gate on something other than
presence, this record is still the right answer to the question it asks, and it
is the wrong need to have answered first.

**Nothing here waits on a question the project owner holds about the engine.**
The rule that gives a tile its owner is decided and built. The derived position
of the population is decided and built.

**One thing is open and it does not block this.** Whether a built thing changes
hands when the ground under it does is unanswered.[^6] This need reads the
owner of the ground and never reads what stands on it, so the answer does not
change either way.

## References

[^1]: PRD-0006, a place belongs to somebody. `docs/product/accepted/prd-0006-a-place-belongs-to-somebody.md`
[^2]: Project orientation, the design principles. `CLAUDE.md`
[^3]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^5]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^6]: Blockers register, BLK-036. `docs/BLOCKERS.md`
