---
id: 0002
title: A developer watches the world run
status: Shipped
created: 2026-08-30
---

# PRD-0002 — A developer watches the world run

## Who this is for

A developer who builds a strategy game on this engine, and who has not yet
decided to trust it.

The other two audiences do not need this first. A modeller studies numbers
and reads them from the control plane. A researcher reproduces a run and
compares hashes. Only the game developer needs to see the world move before
they will build on it.

## What the person cannot do today

A game developer cannot see the simulation.

The engine steps, hashes its state, and proves that two runs agree. Every one
of those statements reaches the developer as a passing test. None of them
reaches the developer as a world.

The developer therefore cannot tell a working engine from a broken one by
looking at it. A test says that two runs agree. It does not say that the
units went anywhere. It does not say that they stayed on the map. It does not
say that the world is the shape the developer expected. A defect that every
test passes stays invisible until someone draws the world.

The developer also cannot show the engine to another person. A hash convinces
a reviewer. It does not convince a collaborator, and it does not convince the
person who decides whether to fund the work.

## What good looks like

Each statement below can be checked.

- One command opens a window, and the world appears in it.
- The world in the window has the shape the project chose. A tile is where
  its coordinates say it is.
- Entities appear on the world. Each entity stands on a tile.
- The entities move between tiles while the developer watches. The developer
  gives no input.
- A viewer sees how full each tile is. It shows a tile that holds more
  entities than its capacity allows, and it marks that tile.
- The window shows the simulation. It does not show a copy. The code that
  moves the entities is the engine that the tests exercise.
- The same world, from the same seed, shows the same behaviour on every run.
- The window shows every step the engine takes. The developer misses none of
  them.
- A viewer that only watches asks the engine for no extra work. The engine
  does nothing for the sake of the picture.
- The engine gives the same results when no window is open.

## What this does not do

- It does not let the developer command anything. The developer watches. A
  control is a separate need.
- It does not model intent. The entities behave randomly. This record needs
  movement that a person can see. It does not need movement that means
  something.
- It does not show the world at more than one level of detail. One level
  proves that the world exists.
- It does not draw the whole world at the target scale. It draws a world
  small enough to watch.
- It does not decide how a shipped game presents the engine. It is a window
  onto the engine. It is not a user interface.
- It does not separate the drawing rate from the rate of the steps. The
  demonstration binary is excluded from the statements above by name. It
  draws every step, so a slow drawing slows the demonstration. This record
  accepts that for a demonstration.
- It does not serve a person who must watch a world that steps faster than a
  screen refreshes. That is a later need.
- It does not serve the control plane. A person who wants numbers reads them
  through Python.

## What it costs at the target scale

The cost driver is the number of entities that the viewer draws in one frame.
The screen bounds that number. The world does not.

The world holds far more tiles than a screen holds pixels. It holds far more
entities than a person can follow. A viewer that reads the whole world in
each frame therefore pays a cost that grows with the world, for a picture
whose detail the display bounds. That is the wrong shape, and this record
rejects it.

Two properties follow. A solution must have both.

- What the viewer reads grows with what the screen shows. It does not grow
  with the size of the world.
- The engine does no extra work for the viewer. A viewer that watches adds no
  cost to a step. The time a drawing itself takes is excluded, because the
  demonstration binary draws every step.

No cost figure appears here, because nobody has measured one on the target
platform.[^1] The shape of the growth is the requirement. The figure is not.

## Which blockers govern this

- **No measurement exists on the target platform.**[^1] Every cost statement
  above states a shape, not a number. A frame rate, a draw count and a memory
  figure are all measurements, so this record states none of them.

The two questions that governed the world shape and the faction ceiling are
answered, so this record states neither of them parametrically.[^2]

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Blockers register, BLK-013 and BLK-014. `docs/BLOCKERS.md`
