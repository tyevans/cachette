---
id: 0008
title: A unit changes the ground it stands on
status: Accepted
created: 2026-08-30
---

# PRD-0008 — A unit changes the ground it stands on

## Who this is for

A developer who builds a strategy game on this engine, and who needs the
world to keep a record of what the units did to it.

## What the person cannot do today

A developer cannot let a unit leave a mark.

Units move across the world and take from it. The world does not change
because of them. A unit that walks a route a thousand times leaves the same
ground it found, so the world holds no memory of any effort spent on it.

This has two costs.

The developer cannot make effort accumulate. Every tick starts the world in
the state the generator made, adjusted only by what was removed. Work that
takes many ticks to finish cannot exist, so nothing a unit does can be large.

The developer cannot make a place become good. A tile is as valuable as the
generator made it. A faction that has held ground for a long time is
therefore in the same position as one that arrived, and holding ground gains
nothing.

## What good looks like

Each statement below can be checked.

- A tile can carry an improvement that no generator placed.
- A unit builds an improvement over several ticks, and a watcher can see the
  work in progress.
- Unfinished work persists. A unit that stops and returns continues from
  where the work stood.
- An improvement changes what the tile does. It raises what the tile yields,
  or changes what it costs to cross, or both.
- An improvement can be destroyed, and the tile returns to what it was.
- A watcher can see improvements on the map, and can tell them from terrain.
- The same seed and the same actions give the same improvements, at every
  thread count, on every run.
- Two units can work on one improvement, and the work they contribute adds
  exactly.

## What this does not do

- It does not decide which improvements exist. A road, a farm, a mine and a
  wall are candidates. This record needs a mechanism, not a catalogue.
- It does not make a unit decide to build. A unit that is told to build
  builds. Choosing belongs with unit behaviour.
- It does not model a building with an inside. An improvement is a property
  of a tile.
- It does not give an improvement an owner separate from the tile's holder.
- It does not require a cost in resources. Whether building consumes what a
  unit carries is a design question, and it depends on what gathering yields.
- It does not model decay. Whether an improvement degrades without upkeep is
  a separate need, and it belongs with production and upkeep.

## What it costs at the target scale

The cost driver is the number of improvements under construction, not the
number of tiles that could hold one.

A world that steps every tile to advance construction pays the world for the
tiles where nothing is being built. That is the wrong shape, and this record
rejects it.

These properties follow.

- The cost of advancing construction grows with the number of sites under
  construction. It does not grow with the size of the world.
- The storage of improvements grows with the number of improvements, not with
  the number of tiles. A world with no improvements stores none.
- Progress is an exact integer quantity, so contributions from several units
  sum the same in any order.
- A finished improvement is read at the cost of reading a tile. The thing
  that is read every tick is the result, not the progress.

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
