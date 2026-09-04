---
id: 0032
title: A god knows what its ground is rich in
status: Shaped
created: 2026-09-03
---

# PRD-0032 — A god knows what its ground is rich in

## Who this is for

A developer who builds a strategy game on this engine, and whose game rewards a
side for holding varied ground rather than plentiful ground.

The engine already gives each place an owner, and it already gives each place a
stock of the things a unit gathers.[^1] [^2] This record is about a different
question. It asks how many *different* precious things a side holds, and it does
not ask how much of anything a side holds.

One real project needs it. In that game a god directs a congregation, and the
god wants a reason to spread rather than to dig. The need is stated here in
general terms, because any game with sides and ground can reward variety.

## What the person cannot do today

**A developer cannot put a precious thing on a place, and cannot ask how varied
a side's holdings are.**

The engine holds three things a unit gathers. A unit takes an amount of one of
them into a store, and every place carries a stock of each. That catalogue is
fixed when the engine is built. A developer cannot add a fourth thing, and a
developer cannot choose where a thing occurs, because the engine generates the
stock of a place from the world seed.

This has four costs.

**A developer cannot author a world.** A game that wants a rare spice in one
valley and a rare stone in another has no way to say so. The world is what the
seed makes it.

**A developer cannot reward variety.** The engine reports amounts. Two sides
that hold the same amount of one thing and different numbers of different
things read the same.

**A developer who counts variety outside the engine loses the arithmetic.** A
score kept in the control plane is a score the engine cannot combine over a
region, and the control plane would have to visit each place to build it. That
is the loop this project forbids.[^3]

**A score kept outside the engine is not part of the world.** Two runs of the
same world would not be comparable, because the score would not be in what the
engine compares.

## What good looks like

Each statement below can be checked.

- A developer names any number of precious things when it builds a world, and
  names where each one occurs, in one call.
- A place carries any number of them at once, including none and including all
  of them.
- A developer asks how many different precious things stand on one place, in
  one region, on the ground one side holds, and in the whole world.
- The answer for a region is exactly the number of different things on the
  places of that region. It is never an average and never an estimate.
- Two worlds that carry different precious things give different answers. Two
  that carry the same give the same answer.
- The same world gives the same answer at every thread count and on every run.
- A world in which the developer named nothing costs nothing to hold.
- A developer who names a thing the engine cannot address meets an error that
  names the ceiling, rather than a world that quietly holds one thing fewer.
- The score is part of the world, so two worlds that differ in it are reported
  as different worlds.

## What this does not do

**It does not make a precious thing gatherable.** A unit does not carry one,
does not deliver one, and does not store one. The three gatherable things stay
what they are, and this need adds none to them.

**It does not say how much.** A place carries a precious thing or it does not.
A game that wants a quantity is asking for something else.

**It does not change how anybody behaves.** The engine reports the score and
changes no rule because of it. What the score is worth is the game's business,
until the project owner says otherwise.[^4]

**It does not generate the placement.** The developer says where each thing
occurs. The engine invents nothing and draws nothing.

**It does not let the placement change.** A developer names the world once. A
thing that appears and disappears is a different need.

**It does not name the things.** A precious thing is a number. What the game
calls it is the game's business.

**It does not trade.** Two sides exchanging anything is a separate need.

## What it costs at the target scale

The engine holds far more places and units than a script can visit, and the
scale constants table holds the figures.[^5]

**The engine holds nothing for a place that carries nothing.** A world in which
the developer named no precious thing holds no entry at all, in the same way
that a world in which nobody built holds no built thing. The cost follows what
the developer named, and not the size of the world.

**What one place carries is one machine word.** The reference table states the
side ceiling and the reason for it: a set of sides is one word.[^5] A set of
precious things is one word for the same reason, so the ceiling on the
catalogue is that width, and reading what a place carries costs one read
whatever the number of things on it.

**The score is a population count of that word.** It is an exact whole number,
so combining two of them is exact and order-free, and no fraction and no
rounding enters.

**Combining two regions is a union of sets.** That has an identity, it is
associative, and it is commutative, so the answer does not depend on how the
work was split or on how many threads ran it.

**Deriving the region answer costs what the developer named.** The pass walks
the places the developer named and no others, so a sparsely seeded world costs
almost nothing.

**No figure is stated here.** One blocker governs every cost figure in this
project, and it says which figures are measured and which are derived.[^6] The
statements above are arguments about which term the cost follows, not results.

## Which blockers govern this

**One blocker governs every cost claim above.**[^6] Nothing here was measured.

**One blocker holds what the score should change.**[^4] The project owner
suggested that variety could change how a settlement staffs its work, and he
said that he did not know. Until that closes, the engine reports the score and
changes no rule. **This record is shaped without that answer on purpose.** The
need above is to know, and knowing is useful to a game whatever the engine
later does with it.

**One blocker holds whether the ceiling is enough.**[^7] A catalogue wider than
one machine word costs a second word for every seeded place, and it changes
what the engine compares between two runs. That choice is cheaper now than
later.

**One blocker governs the game this need came from.** The rules of that game
are one paragraph.[^8] The need above is stated in general terms so that it
does not wait on them.

**Nothing here waits on a question the project owner holds about the engine
itself.** The rule that gives a place its owner is decided and built. The rule
that gives a place its stock is decided and built, and this need does not touch
it.

## References

[^1]: PRD-0006, a place belongs to somebody. `docs/product/accepted/prd-0006-a-place-belongs-to-somebody.md`
[^2]: PRD-0007, the world holds things worth taking. `docs/product/accepted/prd-0007-the-world-holds-things-worth-taking.md`
[^3]: Project orientation, the design principles. `CLAUDE.md`
[^4]: Blockers register, BLK-110. `docs/BLOCKERS.md`
[^5]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^6]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^7]: Blockers register, BLK-111. `docs/BLOCKERS.md`
[^8]: Blockers register, BLK-050. `docs/BLOCKERS.md`
