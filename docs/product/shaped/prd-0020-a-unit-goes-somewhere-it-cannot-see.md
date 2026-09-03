---
id: 0020
title: A unit goes somewhere it cannot see
status: Shaped
created: 2026-09-02
---

# PRD-0020 — A unit goes somewhere it cannot see

## Who this is for

A developer who builds a strategy game on this engine, and who needs a unit to
have somewhere to be.

This need sits directly on top of the one the whole project points at. That one
asks that a unit acts on the world it can see.[^1] This one asks that a unit can
act on a part of the world it cannot see from where it stands.

## What the person cannot do today

**A developer cannot make a unit go anywhere in particular.**

A unit reads the block of tiles it stands in, scores a small fixed set of
options against what that block holds, and takes the highest. Every option
is a gradient: it names one quantity the block summarises, and the unit
prefers more of it.

A place is not a quantity. A unit standing away from its home reads a summary
that says nothing about where its home is, so no score can prefer the direction
of home over any other direction. The developer has no way to express it.

This has three costs.

**A developer cannot write a story with a return in it.** Leaving, doing
something and coming back is the shape of most of what a unit does in a
strategy game. The going out works. The coming back cannot be said.

**A developer cannot connect two things the world already has.** Units gather
from the ground and settlements hold a store. Nothing carries what was gathered
from the first to the second, because carrying it means going to a named place.

**A developer cannot show a watcher that a unit belongs anywhere.** A unit that
wanders away and never returns reads as a unit with no home, whatever the
engine records about where it lives.

## What good looks like

Each statement below can be checked.

- A unit given a place to be moves toward that place across ground it cannot
  see from where it started.
- A unit that is given no such place behaves as it does today, and nothing
  about it becomes worse.
- Two units with the same place to be, standing in different parts of the
  world, both arrive, and neither is given a route by the control plane.
- A unit whose destination stops existing does not walk to where it used to be.
- A watcher can ask why a unit stepped the way it did and get an answer from
  the engine, in the same way it can ask why a unit chose what it chose.
- The same seed and the same world send the same units the same way, at every
  thread count, on every run.
- The work the engine does to answer this does not grow when the population
  grows.

## What this does not do

**It does not say how the engine answers it.** The need is that a unit reaches
a place it cannot see. Whether the engine computes that once for the whole world
or once for each unit, and what it stores to do so, is not this record's
business and this record states none of it.

**It does not ask for a path.** Nothing here requires that a unit route around
an obstacle, take the shortest way, or arrive in a stated number of ticks. A
unit that moves generally toward a place satisfies this need, and a unit that
takes a longer way than a person would still satisfies it.

**It does not give a unit a destination it can name.** A unit does not hold the
identity of a place, ask where that place is, or carry a target. The need is met
when a unit ends up somewhere, not when a unit knows where it is going. A
developer who wants a unit to report its destination is asking for something
else.

**It does not ask for per-unit destinations.** A developer choosing a place for
each unit individually is a control plane walking the population, which the
project forbids.[^2] A place given to a set of units answers this need in full.

**It does not ask that a unit knows it has arrived.** What a unit does when it
gets somewhere belongs to whatever it does there. The engine has no concept of
arrival, and this need does not create one.

**It does not ask for two units to want different things.** This is the
exclusion worth stating loudest, because it is the one a reader will assume is
included. The engine holds one weight profile for every unit alive, so two units
in one cell with the same need always choose alike, and a finding records
it.[^3] **Letting a unit reach a place does not let two units disagree about
which place.** They are separate needs with separate answers, and folding them
together would let this record claim a gap it does not close.

## What it costs at the target scale

The engine holds far more tiles and units than a script can visit, and the
scale constants table holds the figures.[^4]

The cost that matters is which term it follows. An answer computed for each
unit costs the population. An answer computed for each block of tiles costs the
block count, and the block count does not change when units are born.

**No figure is stated here.** No measurement exists on the target platform, and
every cost figure in this project is derived rather than measured.[^5] A record
that quoted one would be quoting an estimate.

## Which blockers govern this

**One open choice could stop the whole approach, and it is live.** Whether a
plane over the block lattice may carry a value from one tick to the next is
open. The project's own rule is that every level above the tiles is derived from
the tiles, and a plane that carries a value from the last tick is not derived
from anything below it. That question already blocks one record from being
accepted, and it was raised against a different plane before this need
existed.[^6]

**It matters here because reach is what this need buys.** A value that is built
fresh each time reaches only as far as the work spent building it. A value that
carries reaches further every tick. If the carried form is refused, a unit far
from its destination reads nothing and this need is met over short distances and
not long ones. A second open row holds that consequence for this case.[^7]

**No measurement exists on the target platform.** Every cost claim about this
need is derived rather than measured, so the statement above about which term
the cost follows is an argument and not a result.[^5]

**Nothing here is blocked by an unanswered question the owner holds.** The three
above are engineering questions. The need itself is stated and does not wait on
anybody.

## References

[^1]: PRD-0009, a unit acts on the world it can see. `docs/product/accepted/prd-0009-a-unit-acts-on-the-world-it-can-see.md`
[^2]: Project orientation, the design principles. `CLAUDE.md`
[^3]: Findings register, FND-251. `docs/FINDINGS.md`
[^4]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^5]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^6]: Decisions register, DEC-067. `docs/DECISIONS.md`
[^7]: Decisions register, DEC-095. `docs/DECISIONS.md`
