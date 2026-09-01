---
id: 0014
title: Everyone needs somewhere to live
status: Shaped
created: 2026-08-31
---

# PRD-0014 — Everyone needs somewhere to live

## Who this is for

A developer who builds a strategy game on this engine, and who needs a
population to be somewhere in particular.

## What the person cannot do today

A developer cannot limit where a population can be.

A unit stands on a tile and belongs to nothing. Any number of units can occupy
a region, and no place holds anybody. A settlement is therefore a name on a
map rather than a thing that contains people.

This has three costs.

The developer cannot make growth cost anything. A population that needs
nothing built in order to exist grows for free, so building is never the thing
that limits a faction.

The developer cannot make a place worth defending. A place that holds nobody
loses nobody. Taking it removes ground from an enemy and nothing else, so no
attack can hurt a population.

The developer cannot make crowding happen. A hundred units and ten thousand
units in one region behave the same. Pressure to spread out cannot exist, so a
population never has a reason to move.

## What good looks like

Each statement below can be checked.

- A place holds a stated number of people, and that number follows from what
  has been built there rather than from the size of the ground.
- A unit lives somewhere. A watcher can ask a unit where it lives and get an
  answer.
- The number of people who live in a place is a fact the world reports. A
  watcher reads it without walking every unit.
- A population larger than the places that hold it produces a consequence a
  watcher can see and name.
- The consequence goes away when more places are built, and it goes away when
  people leave.
- A population stops growing, or grows more slowly, when there is nowhere for
  a new person to live.
- A place can be lost or destroyed, and the people who lived there stop living
  there at the same moment.
- A watcher can see where people live and can see how full each place is.
- The same seed and the same world give the same places, the same residents
  and the same crowding, at every thread count, on every run.

## What this does not do

- It does not decide how a dwelling sits on the ground. Whether shelter is a
  property of a tile or a property of a settlement is an architectural
  question, and this record states the need instead.
- It does not model an interior. A dwelling has a capacity. It has no rooms
  and nothing happens inside it.
- It does not model quality. Every place to live shelters one person as well
  as any other.
- It does not decide what building a place to live costs, or who builds it.
  Building belongs with improvements.
- It does not give a dwelling an owner, a rent or an heir. Who inherits a
  dwelling belongs with family.
- It does not model a unit without a home as a different kind of thing. A unit
  that lives nowhere is still a unit.
- It does not decide whether a unit may move to another place. Migration is a
  separate need, and it depends on what crowding turns out to cost.
- It does not require a unit to be at home. Where a unit stands and where it
  lives are two different facts.

## What it costs at the target scale

The cost driver is the number of places to live and the number of units that
hold one. It is not the number of tiles.

Two shapes are rejected. A slot for a dwelling on every tile pays the world
for the ground that holds nobody. A residency query that walks the population
to count who lives in a place pays the population for one number, and a
watcher asks for that number often.

These properties follow.

- Occupancy is kept as a count and updated by the change. It is not recomputed
  by a sweep over the units.
- A unit's residence is read at a bounded cost that does not grow with the
  population and does not grow with the size of the world.
- Storage grows with the number of places that exist, not with the number of
  tiles. A world with no dwellings stores none.
- Capacity and occupancy are exact integers, so an aggregate over many places
  combines to the same total in any order.
- Assignment of units to places, and eviction from them, are ordered by a
  stable key, never by thread completion order.
- Losing a place updates every resident of it as one set-valued operation, not
  as a loop in the control plane.

No cost figure appears here, because the one measurement this project has was
taken on a development machine and not on the target.[^1] That measurement
established the shape that matters: the term that grows with the number of
things dominates the term that grows with the number of tiles.[^2]

## Which blockers govern this

- **No measurement exists on the target platform.**[^1] Every cost statement
  above states a shape, not a number.

The settlement count is answered, so this record states no number of
places.[^3] The population is answered, and it counts everybody rather than
the soldiers alone, so everybody in it needs somewhere to live.[^4]

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Findings register, FND-049. `docs/FINDINGS.md`
[^3]: Blockers register, BLK-005. `docs/BLOCKERS.md`
[^4]: Blockers register, BLK-003. `docs/BLOCKERS.md`
