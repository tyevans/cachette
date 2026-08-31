---
id: 0017
title: Work is assigned to the people who can do it
status: Shaped
created: 2026-08-31
---

# PRD-0017 — Work is assigned to the people who can do it

## Who this is for

A developer who builds a strategy game on this engine, and who needs a place
to respond to what it lacks.

## What the person cannot do today

A developer cannot move people between kinds of work.

A unit holds a job. The job never changes, and nothing in the world can change
it. A place that runs out of food holds the same number of farmers it held
before, so a place cannot respond to its own situation.

This has three costs.

The developer cannot make a shortage produce a response. A shortage is
information, and information that nobody acts on changes nothing. Every rule
that produces a shortage therefore ends in a number that nothing reads.

The developer cannot make a choice cost anything. Every soldier is a farmer
who is not farming. Without an assignment that names the trade, a faction can
have every kind of worker at once, and no decision about its people is a
decision.

The developer cannot express an intent from the control plane. A person who
wants more soldiers must name the units that become soldiers, and naming units
one at a time is the thing the control plane must never do.

## What good looks like

Each statement below can be checked.

- A place holds a number of positions of each kind, and a watcher can ask what
  they are.
- The number of positions responds to what the place has and lacks. A place
  short of food moves towards more of the work that produces food.
- A unit holds a position by a rule, and a watcher can ask why that unit holds
  that position.
- A unit's job changes during its life when what the place needs changes.
- Not every unit can hold every position. A property of the unit limits which
  positions it can take.
- A position can go unfilled, and the place shows a consequence a watcher can
  see.
- One command changes what a set of places prefers, and the assignment
  follows. The command names no unit.
- The assignment for the whole world runs with no loop in the control plane.
- The same seed and the same world give the same assignments, at every thread
  count, on every run.

## What this does not do

- It does not decide which jobs exist. Farmer, soldier, builder and trader are
  candidates. This record needs an assignment that works, not a catalogue.
- It does not model a wage, a contract or a labour market. What a job pays
  belongs with goods and value.
- It does not model a skill that grows with practice. Fitness for a job is a
  property a unit has, not a history it accumulates.
- It does not let a unit refuse. A unit that is assigned takes the position.
- It does not move a unit to where the work is. Whether a unit travels to its
  job is a separate need.
- It does not model a career, a rank or a promotion within a kind of work.
- It does not give a unit a preference of its own. A unit that decides for
  itself belongs with unit behaviour. This record decides for the place.
- It does not decide which place a unit belongs to. Where a unit lives is a
  separate need, and this record reads the answer.

## What it costs at the target scale

The cost driver is the number of units whose assignment changes, not the
number of units that exist.

Three shapes are rejected. An assignment that scores every unit against every
position pays the population multiplied by the positions. An assignment
recomputed from nothing every tick pays the population every tick for an
answer that is nearly the same as the last one. A loop in the control plane
that assigns unit by unit pays the population in the slowest language in the
project, and the project already forbids it.

These properties follow.

- Assignment runs as one set-valued operation for each place, over the units
  that place holds. It is not a loop over units.
- The steady state is cheap. A world where nothing changed does almost no
  assignment work.
- Assignment does not run every tick. The interval is a parameter, and the
  cost falls as the interval rises.
- The cost grows with the number of places and with the number of units that
  move. It does not grow with the size of the world.
- Every quantity in the decision is an exact integer or a fixed-point value,
  so two candidates for one position compare the same way whatever order the
  work ran in.
- Ties break by a stable key, never by thread completion order and never by
  work-stealing order.
- One command from the control plane sets the preference for a set of places.
  The control plane names a set. It never names a unit.

No cost figure appears here, because the one measurement this project has was
taken on a development machine and not on the target.[^1] That measurement
established the shape that matters: the term that grows with the number of
things dominates the term that grows with the number of tiles, and this record
must keep the changing part of that term small.[^2]

## Which blockers govern this

- **No measurement exists on the target platform.**[^1] Every cost statement
  above states a shape, not a number.

The settlement count is answered, so this record states no number of
places.[^3] The population is answered, and it counts everybody rather than
the soldiers alone, so the assignment covers the whole of it.[^4]

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Findings register, FND-049. `docs/FINDINGS.md`
[^3]: Blockers register, BLK-005. `docs/BLOCKERS.md`
[^4]: Blockers register, BLK-003. `docs/BLOCKERS.md`
