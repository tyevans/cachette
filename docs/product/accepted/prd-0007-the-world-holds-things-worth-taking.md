---
id: 0007
title: The world holds things worth taking
status: Accepted
created: 2026-08-30
---

# PRD-0007 — The world holds things worth taking

## Who this is for

A developer who builds a strategy game on this engine, and who needs the
units to want something.

A modeller needs this soon after. A modeller studies where a quantity comes
from and where it goes, and this is the first quantity the world produces.

## What the person cannot do today

A developer cannot give a unit a reason to be anywhere.

The world holds ground, and ground is the same everywhere in the one way that
matters: it offers nothing. A unit therefore has no reason to prefer one tile
to another, so movement carries no meaning however well it works.

This has two costs.

The developer cannot make a place valuable. Terrain makes a place different.
It does not make a place worth going to. Without value, a map has texture and
no geography, because geography is where the good places are and what it
takes to reach them.

The developer cannot make a unit's time matter. A unit that gathers nothing
spends nothing and gains nothing, so a tick costs it nothing. Every later
need depends on this one: a wage needs something to pay, a trade needs
something to trade, and a want needs something to want.

## What good looks like

Each statement below can be checked.

- A tile can hold an amount of a resource, and different tiles hold different
  amounts.
- A unit standing on such a tile can take from it, and the amount on the tile
  falls by exactly what the unit took.
- Nothing is created and nothing is lost. What leaves a tile arrives
  somewhere, exactly, with no loss to rounding and no gain.
- A unit carries what it took, and a watcher can ask what a unit carries.
- A resource can run out. A tile that holds nothing gives nothing.
- Terrain influences what a tile holds. A resource is not spread evenly.
- The same seed gives the same resources in the same places, at every thread
  count, on every run.
- A watcher can see where the resources are, and can see them being taken.
- Two units cannot take the same unit of resource. A watcher can see that
  this holds.

## What this does not do

- It does not decide which resources exist. Food, wood, stone and ore are
  candidates. This record needs a quantity that behaves correctly, not a
  catalogue.
- It does not make a unit decide to gather. A unit that is told to gather
  gathers. Choosing to gather belongs with unit behaviour.
- It does not consume anything. A unit that carries food does not eat it.
  Consumption arrives with unit lives.
- It does not move a resource between places. Carrying is not trading.
- It does not build anything. A mine that raises what a tile yields belongs
  with improvements.
- It does not give a resource a price. A price needs an exchange, and that is
  trading.
- It does not regrow a resource. Whether a deposit refills is a separate
  question, and it depends on what depletion turns out to cost.

## What it costs at the target scale

The cost driver is the number of units gathering, not the number of tiles
that hold a resource.

A world that steps every deposit each tick pays the world for the deposits
that nothing is touching. A gathering rule that resolves one unit at a time
against a shared tile pays for a conflict that is rare. Both are the wrong
shape, and this record rejects both.

These properties follow.

- The cost of gathering grows with the number of units that gather, not with
  the number of deposits and not with the size of the world.
- Two units taking from one tile resolve by a rule that runs over the whole
  set, not by a lock and not by a retry. A set-valued command permits a
  cheaper algorithm, and this is one.
- The amount on a tile is an exact integer quantity, so a sum over tiles is
  the same in any order.
- What a unit carries is exact. A conservation check over the world balances
  to zero.

No cost figure appears here, because the one measurement this project has was
taken on a development machine and not on the target.[^1] That measurement
established the shape that matters: the term that grows with the number of
things dominates the term that grows with the number of tiles.[^2]

## Which blockers govern this

- **One blocker governs every cost figure here.**[^1] It says which figures
  are measured and which are derived. Every cost statement above states a
  shape, not a number.

The tile capacity and the world shape are answered, so this record states
neither of them parametrically.[^3]

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Findings register, FND-049. `docs/FINDINGS.md`
[^3]: Blockers register, BLK-013. `docs/BLOCKERS.md`
