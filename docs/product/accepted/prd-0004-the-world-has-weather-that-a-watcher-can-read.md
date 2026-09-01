---
id: 0004
title: The world has weather that a watcher can read
status: Accepted
created: 2026-08-30
---

# PRD-0004 — The world has weather that a watcher can read

## Who this is for

A developer who builds a strategy game on this engine, and who needs the
world to act on the units rather than only hold them.

A modeller needs this second. A modeller studies how a condition spreads and
how it decays, and weather is the first such condition the world will hold.
A researcher does not need it at all.

## What the person cannot do today

A developer cannot make the world change on its own.

The world holds terrain, and terrain does not move. A tile is a forest and
stays a forest. Every condition a unit meets is therefore fixed when the
world is generated, so no situation can arise that the generator did not
already place there.

This has two costs.

The developer cannot make a place matter differently at different times. A
river crossing is either passable or not. It cannot be passable in one season
and closed in another, so the map holds no history and no timing.

The developer also cannot get a story out of the world. A story needs a
change that nobody chose. Terrain gives the world variety, and variety is not
change. Until something moves across the map by itself, every event in the
simulation traces back to a unit, and the world is scenery.

## What good looks like

Each statement below can be checked.

- The world holds at least one condition that varies over the map and over
  time, without any unit acting on it.
- The condition changes by a rule the world applies each tick. A table of
  values read by date does not satisfy this.
- The condition conserves what it should conserve. What leaves one place
  arrives at another, exactly, with no loss to rounding and no gain.
- The condition is bounded. It does not grow without limit and it does not
  fall below zero.
- Terrain influences the condition. A height, a slope or a tile kind changes
  what the condition does there.
- The condition influences a unit. A unit that stands in it behaves
  differently from a unit that does not, and a developer can point at the
  difference.
- The same seed gives the same weather, at every thread count, on every run.
- A watcher can see the condition on the map and can tell it apart from the
  terrain beneath it.
- The condition costs nothing when nothing is happening. A calm map does not
  cost what a storm costs.

## What this does not do

- It does not model real weather. It models a condition that behaves
  plausibly. Fidelity to a physical atmosphere is not the need.
- It does not decide which conditions the world holds. Rain, wind,
  temperature and snow are candidates. This record needs one that works, not
  a catalogue.
- It does not give a unit a forecast. A unit meets the weather where it
  stands. Prediction is a separate need, and it belongs with unit behaviour.
- It does not tie weather to a calendar. A season is a way of driving the
  condition. This record does not require one.
- It does not put weather in the pyramid. Whether an aggregate carries the
  condition upward is a separate question, and it depends on what reads it.
- It does not damage a unit. Weather that kills belongs with unit lives.

## What it costs at the target scale

The cost driver is the area the condition occupies, not the area of the
world.

A field that updates every tile of the world each tick costs the whole world
every tick, whatever the weather is doing. The world is far larger than any
storm. That is the wrong shape, and this record rejects it.

Three properties follow. A solution must have all three.

- What the update costs grows with the area the condition occupies. It does
  not grow with the size of the world.
- The storage grows with the same area, not with the world. An empty map
  stores nothing.
- The condition combines exactly under any order, so the aggregate a
  projection takes from it does not depend on how the work was divided.

No cost figure appears here, because the one measurement this project has
was taken on a development machine and not on the target.[^1] That
measurement did establish the shape that matters: the cost that grows with
the number of things, rather than with the number of tiles, is the cost that
dominates.[^2]

## Which blockers govern this

- **No measurement exists on the target platform.**[^1] Every cost statement
  above states a shape, not a number. A tick budget, a memory figure and an
  update rate are all measurements, so this record states none of them.

This record depends on terrain. The world now gives every tile a kind and a
height. This record states no terrain value, so no terrain decision can make
it false.

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Findings register, FND-049. `docs/FINDINGS.md`
