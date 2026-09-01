---
id: 0009
title: A unit acts on the world it can see
status: Shaped
created: 2026-08-30
---

# PRD-0009 — A unit acts on the world it can see

## Who this is for

A developer who builds a strategy game on this engine, and who needs a unit
to do something a watcher can explain.

This is the record the whole project points at. Every other need builds the
world that this one reads.

## What the person cannot do today

A developer cannot get behaviour out of a unit.

A unit draws a direction and steps. The draw is correct, repeatable and
meaningless. The unit does not read the world, so nothing in the world can
change what it does.

This has three costs.

The developer cannot produce a story. A story is a chain of causes, and a
random walk has none. A watcher who sees a unit move learns nothing, because
the movement would have been the same in any world.

The developer cannot test whether the world works. A world exists to be acted
in. Terrain, resources, holdings and weather are all inert until something
responds to them, so none of them can be shown to matter.

The developer cannot get emergence. Emergence is behaviour the designer did
not place there, and it comes from simple units responding to each other. A
unit that responds to nothing cannot produce it.

## What good looks like

Each statement below can be checked.

- A unit chooses its action by reading the world around it. A watcher can
  change the world and see the choice change.
- A unit prefers one option over another for a stated reason, and the reason
  is a value the world holds.
- A unit responds to another unit. A unit of another faction nearby changes
  what it does.
- A unit responds to the ground. Terrain, a resource or a holding changes
  what it does.
- A unit that has nothing to respond to still behaves sensibly. An empty
  world does not produce a stuck unit.
- The same seed and the same world give the same choices, at every thread
  count, on every run.
- A watcher can ask why a unit did what it did, and get an answer from the
  engine rather than a guess.
- Behaviour scales. The rule that chooses for one unit chooses for all of
  them, with no loop in the control plane.

## What this does not do

- It does not model a mind. A unit scores a small fixed set of options. It
  does not plan, remember or learn.
- It does not give a unit a long path. A unit acts on what is near it.
  Crossing the world is a separate need.
- It does not model a group decision. A unit chooses for itself. A group that
  acts together belongs with a later need.
- It does not give a unit a goal that outlives a tick. Persistent intent is a
  separate question, and it depends on what a single choice costs.
- It does not decide which options exist. Move, gather, build, fight and wait
  are candidates.
- It does not put a script in the world. A behaviour that names a specific
  situation is not what this record asks for.
- It does not require a unit to be right. A unit acts on what it can see, and
  what it can see may be wrong or incomplete.

## What it costs at the target scale

The cost driver is the number of units choosing, multiplied by the number of
options each one scores.

Two shapes are rejected. A unit that searches the world for its best option
pays the world for each unit. A rule that runs content code inside the choice
puts a call for each unit and each option on the hot path, and the project
forbids calling content code from inside a system.

These properties follow.

- The cost of choosing grows with the number of units, and the option set for
  each unit is a small fixed size known before the frame runs.
- What a unit reads to choose is bounded and near. It does not grow with the
  size of the world.
- A score is an exact integer or a fixed-point value, so a comparison between
  two options gives one answer whatever order the work ran in.
- Ties break by a stable key, never by thread completion order.
- The choice runs as one set-valued operation over all units, not as a loop
  that visits each unit in the control plane.

No cost figure appears here, because the one measurement this project has was
taken on a development machine and not on the target.[^1] That measurement
established the shape that matters: the term that grows with the number of
things dominates the term that grows with the number of tiles, and this
record adds units to that term.[^2]

## Which blockers govern this

- **No measurement exists on the target platform.**[^1] Every cost statement
  above states a shape, not a number.

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Findings register, FND-049. `docs/FINDINGS.md`
