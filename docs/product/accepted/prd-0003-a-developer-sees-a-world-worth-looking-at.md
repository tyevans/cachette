---
id: 0003
title: A developer sees a world worth looking at
status: Accepted
created: 2026-08-30
---

# PRD-0003 — A developer sees a world worth looking at

## Who this is for

A developer who builds a strategy game on this engine, and who has already
seen the world run.

The other two audiences do not need this. A modeller reads numbers and does
not care what a tile looks like. A researcher compares hashes and does not
look at the world at all. Only the game developer must judge whether the
world is a place, and that judgement is made by looking.

## What the person cannot do today

A game developer cannot tell one part of the world from another.

Every tile is the same kind of tile. Nothing distinguishes the place a unit
stands from the place it came from. The developer therefore cannot answer the
first question anyone asks of a world simulation: where is this happening.

Three further things follow, and each of them blocks work.

The developer cannot judge whether the world is worth building a game on. A
world of identical tiles proves that the engine steps. It does not show that
the engine holds a place.

The developer cannot see a defect that has a position. A unit that walks into
the sea, a region that no unit can reach, a border drawn in the wrong place:
each of these is obvious against a varied world and invisible against a flat
one.

The developer cannot start any of the later work. A territory needs somewhere
to be. A deposit needs somewhere to sit. A route needs something to go
around. Each of those needs asks the world what a tile is, and the world has
no answer.

## What good looks like

Each statement below can be checked.

- Every tile has a kind. The kinds are few and a person can name them.
- Every tile has a height. Two tiles beside each other have close heights, so
  the world reads as ground rather than as noise.
- Every kind occupies a part of the world. No kind is absent, and no kind
  covers everything.
- A person who looks at the world sees regions, not speckle. Water gathers.
  High ground gathers.
- Every tile states whether a unit may stand on it.
- The same seed gives the same world, on every machine, at every thread
  count, however the reader visits the tiles.
- A different seed gives a different world.
- A world that runs does not change its ground. A unit crosses the world; the
  world does not move under it.
- The developer chooses a world by choosing a seed, and needs nothing else.

## What this does not do

- It does not decide what a tile costs to cross. That is a movement question,
  and an open choice governs it.[^1]
- It does not put anything on a tile. A resource, a settlement, a road and a
  claim are each a later need.
- It does not change with the season or the weather. The ground is fixed for
  the life of a world.
- It does not let a person edit the world, load one from a file, or paint
  one. The seed is the whole input.
- It does not model rivers, coastlines as objects, or any feature that spans
  more than one tile.
- It does not decide how a viewer colours the world. What the ground is, and
  what it looks like, are separate questions.
- It does not serve the control plane with a per-tile query loop. A person who
  wants to count tiles asks for a count.

## What it costs at the target scale

The target is 16.7 million tiles. Two costs matter, and they pull against
each other.

The first is memory. A world that keeps a value for every tile pays for every
tile, and it pays whether or not anything reads the tile. Several fields for
each tile, at the target count, is a cost the project must justify against
what it buys.

The second is time. A world that computes a tile when a reader asks for it
pays nothing to hold the world, and pays again every time a reader asks. A
reader that sweeps the whole world pays the full price each sweep.

The need does not choose between them, because that is an architectural
decision. It states the shape both must meet.

- Building a world must not cost a pass over every tile before the first
  frame. A developer who changes a seed sees the new world at once.
- Reading one tile must cost a bounded amount of work that does not grow with
  the size of the world.
- The cost of the ground must not grow with the number of units, and the cost
  of the units must not grow with the ground.

No cost figure appears here, because nobody has measured one on the target
platform.[^2] The shape of the growth is the requirement. The figure is not.

## Which blockers govern this

- **No measurement exists on the target platform.**[^2] Every cost statement
  above states a shape, not a number. A memory figure and a per-tile time are
  both measurements, so this record states neither.

The question that governed the world shape is answered, and the question that
governed the tile edge is answered, so this record states neither of them
parametrically.[^3]

## References

[^1]: Decisions register, DEC-017. `docs/DECISIONS.md`
[^2]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^3]: Blockers register, BLK-001 and BLK-014. `docs/BLOCKERS.md`
