---
id: 0019
title: A unit goes somewhere it cannot see
status: Shaped
created: 2026-09-02
---

# PRD-0019 — A unit goes somewhere it cannot see

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

## What this does not ask for

**It does not ask for a path.** Nothing here requires that a unit route around
an obstacle, take the shortest way, or arrive in a stated number of ticks. A
unit that moves generally toward a place satisfies this need.

**It does not ask for per-unit destinations.** A developer choosing a place for
each unit individually is a control plane looping over entities, which the
project forbids.[^2] A place given to a set of units answers this need.

**It does not ask that a unit knows it has arrived.** What a unit does when it
gets somewhere is a separate question, and the engine has no concept of arrival
today.

**It does not ask for two units to want different things.** That is a real gap
and it is a different one: the engine holds one weight profile for the whole
world, so every unit alive scores the options identically. A finding records
it.[^3]

## What it costs at the target scale

The engine holds far more tiles and units than a script can visit, and the
scale constants table holds the figures.[^4]

The cost that matters is which term it follows. An answer computed for each
unit costs the population. An answer computed for each block of tiles costs the
block count, and the block count does not change when units are born.

**No figure is stated here.** No measurement exists on the target platform, and
every cost figure in this project is derived rather than measured.[^5] A record
that quoted one would be quoting an estimate.

## Which blockers govern it

**No measurement exists on the target platform.** Every cost claim about this
need is derived.[^5]

**One open choice governs the mechanism.** Whether a plane over the block
lattice may carry a value from one tick to the next is open, it already blocks
another record, and the shape of the answer here depends on it.[^6] [^7]

**The record that would carry this need is written and binds nothing yet.** It
states that the answer is a field over cells and never a search from a unit, and
it is a draft.[^8]

## References

[^1]: PRD-0009, a unit acts on the world it can see. `docs/product/accepted/prd-0009-a-unit-acts-on-the-world-it-can-see.md`
[^2]: ADR-0040, Python is a control plane, not a data plane, decision D2. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^3]: Findings register, FND-241. `docs/FINDINGS.md`
[^4]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^5]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^6]: Decisions register, DEC-067. `docs/DECISIONS.md`
[^7]: Decisions register, DEC-095. `docs/DECISIONS.md`
[^8]: ADR-0095, a behavioural strategy arrives as a field over cells, never as a search from a unit. `docs/adrs/draft/adr-0095-a-behavioural-strategy-arrives-as-a-field-over-cells.md`
