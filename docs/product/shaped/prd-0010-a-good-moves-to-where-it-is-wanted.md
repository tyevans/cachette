---
id: 0010
title: A good moves to where it is wanted
status: Shaped
created: 2026-08-30
---

# PRD-0010 — A good moves to where it is wanted

## Who this is for

A developer who builds a strategy game on this engine, and who needs places
to depend on each other.

A modeller needs this most of all. A modeller studies where a quantity goes
and what it is worth, and this is the record that produces both.

## What the person cannot do today

A developer cannot connect two places.

A resource is taken where it is found and carried by the unit that took it.
Nothing moves a good from where it is plentiful to where it is scarce, so
every place stands alone.

This has two costs.

The developer cannot make scarcity matter. A place that lacks a resource
simply lacks it. Nothing follows, because nothing can arrive. A shortage with
no possible relief is a fact about the map, not a situation.

The developer cannot make a place strategic. A route matters when something
travels it. With nothing travelling, a mountain pass is terrain, a border is
a line, and cutting a road achieves nothing.

## What good looks like

Each statement below can be checked.

- A good moves between places without a unit being told to carry it.
- Where a good moves depends on where it is scarce and where it is plentiful.
- Nothing is created and nothing is lost in transit. A conservation check
  over the world balances to zero.
- A good has a value that responds to what is available and what is wanted.
- Terrain and improvements change what a route costs, and the cost changes
  where the goods go.
- A route can be cut. Blocking a place changes what arrives, and a watcher
  can see the change.
- The same seed and the same world give the same flows, at every thread
  count, on every run.
- A watcher can see what is moving and where it is going.

## What this does not do

- It does not simulate a merchant. This record moves goods. A unit whose job
  is trading belongs with employment.
- It does not model a market with participants. A value here is what the
  world computes, not what somebody offers.
- It does not model money. Whether a currency exists is a separate question.
- It does not require agreement between factions. Whether two factions trade
  at all is diplomacy, and it is a separate need.
- It does not decide the goods. It needs a mechanism that works for one good,
  and that does not change shape when a second arrives.
- It does not give a good an owner in transit.

## What it costs at the target scale

The cost driver is the number of routes and sources, not the number of goods
in motion and not the size of the world.

The wrong shape is explicit and the project has already named it. Trade that
finds a path for each cart pays a search for each thing that moves, and the
number of things that move is the number this record intends to grow. This
record rejects that shape.

These properties follow.

- Movement is solved once over the whole network, not once for each thing
  that moves. The cost grows with the network, not with the quantity flowing
  through it.
- The solver runs a fixed number of iterations. It does not stop on a
  convergence test and it does not stop on a time budget.
- Quantity is conserved exactly. The arithmetic is integer or fixed point, so
  a flow that splits and rejoins arrives whole.
- A value is a computed exact quantity, not a floating point average.
- Nothing is ordered by thread completion order.

No cost figure appears here, because the one measurement this project has was
taken on a development machine and not on the target.[^1] That measurement
established the shape that matters: the term that grows with the number of
things dominates the term that grows with the number of tiles.[^2]

## Which blockers govern this

- **No measurement exists on the target platform.**[^1] Every cost statement
  above states a shape, not a number.

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Findings register, FND-049. `docs/FINDINGS.md`
