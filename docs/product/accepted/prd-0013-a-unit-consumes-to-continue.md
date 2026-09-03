---
id: 0013
title: A unit consumes to continue
status: Accepted
created: 2026-08-31
---

# PRD-0013 — A unit consumes to continue

## Who this is for

A developer who builds a strategy game on this engine, and who needs a unit's
continued existence to cost something.

A modeller needs this immediately after. A modeller studies where a quantity
goes, and this is the first need that removes a quantity from the world for a
reason.

## What the person cannot do today

A developer cannot make a unit spend anything.

A unit takes from a tile and carries what it took. Nothing takes it back. A
pile of food and no food are the same to the unit that holds them, so the
world produces without ever drawing down.

This has three costs.

The developer cannot make a surplus mean anything. A faction that gathers ten
times what another gathers is in the same position as the other, because
neither one needs any of it. Gathering therefore has no result.

The developer cannot make a shortage happen. A shortage is a demand that a
supply does not meet. With no demand, an empty store is a fact about a place
and not a problem for anybody.

The developer cannot give a place a carrying capacity. Land that feeds a
hundred and land that feeds ten thousand behave the same, so the map sets no
limit on anything.

## What good looks like

Each statement below can be checked.

- A unit takes a quantity from a stock at a stated interval, and the stock
  falls by exactly that quantity.
- Nothing is created and nothing is lost. What a unit consumed, plus what
  remains, equals what was produced. A conservation check over the world
  balances to zero.
- A unit that cannot take what it needs enters a condition, and a watcher can
  see the condition and name it.
- The condition gets worse while the shortage lasts, and it recovers when the
  shortage ends.
- A shortage that lasts long enough ends the unit.
- A place that produces less than its people consume runs its stock down, and
  a watcher can see the stock fall tick by tick.
- A place that produces more than its people consume accumulates, and the
  accumulation is bounded by something the world states.
- The same seed and the same world give the same consumption, the same
  shortages and the same deaths, at every thread count, on every run.
- A watcher can ask a unit what it consumed and what it lacked.

## What this does not do

- It does not decide what a unit consumes. Food is the candidate. This record
  needs a quantity that falls correctly, not a catalogue.
- It does not model a diet, a nutrient set, or a preference between two goods
  that both satisfy the need.
- It does not model hunger as a state of mind. The consequence of a shortage
  is a condition on a unit, not a feeling the unit has.
- It does not decide where the stock sits. Whether a unit draws from what it
  carries or from a shared store depends on where a unit lives, and that is a
  separate need.
- It does not make a unit act on the shortage. A unit that goes looking for
  food is choosing, and choosing belongs with unit behaviour.
- It does not produce anything. What refills a stock belongs with gathering
  and with improvements.
- It does not give the good a price. Value belongs with goods moving.
- It does not model a second need beside the first. Warmth, water and rest are
  candidates, and the mechanism must not change shape when one arrives.

## What it costs at the target scale

The cost driver is the number of units that consume. Every unit in the target
population consumes, so this is the largest per-unit rule the project has.

Two shapes are rejected. A rule that draws one unit at a time against a shared
store makes every unit in a place a writer to one location, and contention on
that location is the whole cost. A rule that runs every unit every tick pays
the population every tick for a quantity that changes slowly.

These properties follow.

- Consumption resolves as one operation over the whole set of units that draw
  from one store. It is not a loop over units and it is not a lock.
- The interval between draws is a parameter. The cost falls as the interval
  rises, and the project already holds that consumption is pooled rather than
  charged to each unit each tick.[^3]
- The cost grows with the number of units that consume. It does not grow with
  the size of the world and it does not grow with the number of stores.
- Every quantity is an exact integer, so a sum over units gives the same total
  in any order and a shortage is the same shortage however the work was
  divided.
- A unit that ends from a shortage is ordered by a stable key, never by thread
  completion order.
- A watcher reads the state of a store as a value the world already holds, not
  as a sum computed by walking the units.

No cost figure appears here, because the one measurement this project has was
taken on a development machine and not on the target.[^1] That measurement
established the shape that matters: the term that grows with the number of
things dominates the term that grows with the number of tiles, and every unit
is in that term here.[^2]

## Which blockers govern this

- **One blocker governs every cost figure here.**[^1] It says which figures
  are measured and which are derived. Every cost statement above states a
  shape, not a number.

The population is answered, and it counts everybody rather than the soldiers
alone, so this record states that every unit consumes and states no count.[^4]
The form of upkeep is answered, so this record states no per-unit charge for
each tick.[^3]

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Findings register, FND-049. `docs/FINDINGS.md`
[^3]: Blockers register, BLK-008. `docs/BLOCKERS.md`
[^4]: Blockers register, BLK-003. `docs/BLOCKERS.md`
