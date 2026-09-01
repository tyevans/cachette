---
id: 0011
title: A unit is born, holds a job, and dies
status: Accepted
created: 2026-08-30
---

# PRD-0011 — A unit is born, holds a job, and dies

## Who this is for

A developer who builds a strategy game on this engine, and who needs the
population to be a consequence rather than a setting.

## What the person cannot do today

A developer cannot let the population answer to the world.

Units are spawned at the start and persist. The number of units is a number
somebody chose. Nothing the world does changes it, and nothing about a unit
changes over its life.

This has three costs.

The developer cannot make a decision expensive. A unit that cannot die costs
nothing to risk, so no choice about a unit carries weight and no loss can
happen.

The developer cannot make prosperity visible. A faction that gathers well and
one that gathers badly hold the same units for ever. Success has no
expression, so the world has no winners and no decline.

The developer cannot make a unit specific. Every unit is interchangeable, so
a watcher cannot follow one. A story needs somebody it is about, and an
interchangeable unit cannot be that.

## What good looks like

Each statement below can be checked.

- A unit can come into existence during a run, from a rule the world applies.
- A unit can cease to exist during a run, and what it carried is accounted
  for.
- A unit holds a job, and the job determines what it does.
- A unit's job can change during its life.
- A unit needs something to continue. Failing to get it has a consequence a
  watcher can see.
- A unit ages, and age changes something about it.
- The population responds to the world. A faction with more resources
  supports more units, and a faction with fewer supports fewer.
- The same seed gives the same births and the same deaths, at every thread
  count, on every run.
- A watcher can follow one unit through its life, and can ask what it is
  doing and why.
- The identity of a unit is never confused with the identity of a unit that
  came before it in the same place.

## What this does not do

- It does not model a family. Whether a unit has parents is a separate
  question.
- It does not model a person. A unit holds a job, a need and an age. It does
  not hold a personality.
- It does not decide the jobs. Farmer, soldier, builder and trader are
  candidates.
- It does not give a unit a career it chooses. A unit is assigned. Choosing
  belongs with unit behaviour.
- It does not model disease, war casualties or starvation as separate
  systems. It needs one rule by which a unit can end.
- It does not require a settlement. Where a unit is born is a design question
  and it depends on what holdings turn out to be.

## What it costs at the target scale

Two cost drivers matter.

The first is the churn. Units arriving and leaving change the entity storage
every tick, and the identity of a unit must survive it. The project already
holds this constraint: identity is a slot and a generation, and a slot is
reused after the generation advances. A rule that reuses a slot without
advancing the generation would let a new unit answer to the identity of a
dead one. This record depends on that not happening.

The second is the accounting. A population that is summed by walking every
unit pays the population for a number.

These properties follow.

- Birth and death cost the number of units that were born and died, not the
  number that exist.
- A count of the population is read at a cost that does not grow with the
  population.
- What a unit needs is an exact integer quantity, so a sum over units is the
  same in any order.
- A dead unit's identity is never reissued. A reference to it resolves to
  nothing, never to a different unit.
- Births and deaths are ordered by a stable key, never by thread completion
  order.

No cost figure appears here, because the one measurement this project has was
taken on a development machine and not on the target.[^1] That measurement
established the shape that matters: the term that grows with the number of
things dominates the term that grows with the number of tiles, and this
record makes the number of things vary.[^2]

## Which blockers govern this

- **No measurement exists on the target platform.**[^1] Every cost statement
  above states a shape, not a number.

The unit ceiling is answered, so this record states no population limit.[^3]

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Findings register, FND-049. `docs/FINDINGS.md`
[^3]: Blockers register, BLK-013. `docs/BLOCKERS.md`
