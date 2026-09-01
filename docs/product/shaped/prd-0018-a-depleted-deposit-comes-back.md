---
id: 0018
title: A depleted deposit comes back
status: Shaped
created: 2026-08-31
---

# PRD-0018 — A depleted deposit comes back

## Who this is for

A developer who builds a strategy game on this engine, and who needs a place
that units worked to stay worth returning to.

A modeller reads this next. A modeller studies where a quantity comes from.
Today the world holds one source of every resource, and that source is the
world as the generator made it. This record adds the second source.

## What the person cannot do today

A developer cannot let the world repair itself.

A tile holds an amount of a resource. A unit takes from it, and the amount
falls. The amount never rises again. Every resource in the world is therefore
a fixed budget that the units spend down, and a run has one direction.

This has three costs.

The developer cannot make a place worth holding for a long time. A tile is
worth most on the tick a unit reaches it, and worth less on every tick after.
Ground that a faction has held and worked is worth less than ground it has
not touched, so holding ground is a loss.

The developer cannot make a run last. A world with a fixed budget ends when
the budget ends. Every need that follows from a standing population assumes a
supply that continues. Consumption without a supply is only a countdown.

The developer cannot state a rate of extraction that is too high. Nothing in
the world says that a hundred units on one wood tile take more than the tile
can carry, because the tile carries no opinion about time. A world that puts
resource back gives that statement a meaning, and over-extraction becomes a
thing a player can do wrong.

## What good looks like

Each statement below can be checked.

- A deposit that a unit took from holds more at a later tick than it held at
  the tick of the take, when nothing takes from it again.
- A deposit never holds more than the amount it started with. Recovery
  returns a deposit toward its starting amount and never past it.
- Recovery is exact. The amount recovered is a whole number, and a total over
  the world is the same in any order.
- Recovery creates nothing. What a deposit regains never exceeds what units
  took from it.
- A deposit that has recovered fully is not different from a deposit that
  nobody touched. A watcher cannot tell the two apart, and neither can a
  gatherer.
- Recovery does not depend on when a watcher asks. Reading the world does not
  change the world, and two watchers that ask at the same tick get one answer.
- Recovery differs by resource kind. A kind can recover quickly, slowly, or
  not at all, and the difference is a parameter of the kind.
- Whether a deposit that reached nothing recovers is a parameter, and the
  same parameter answers it for every tile of that kind. The engine does not
  decide it tile by tile.
- The same seed and the same gathering give the same amounts at the same
  ticks, at every thread count, on every run.
- A watcher can see a deposit recover, and can tell a recovering deposit from
  a full one.
- Gathering from a recovering deposit takes what the deposit holds at that
  tick, and no more.

## What this does not do

- It does not add a crop. A crop is something a unit plants, tends and
  harvests. This record covers what the world does with nobody acting. The
  distinction and the reasoning are in the cost section below.
- It does not add a plant, a species, or a catalogue of either. The record
  needs a quantity that returns correctly. It does not need a botany.
- It does not change what a tile can hold at most. The ceiling of a tile is
  what the generator gave it, and raising a ceiling belongs with improvements.
- It does not spread a resource. A recovered deposit stays on its own tile. A
  forest does not grow into the plain beside it.
- It does not tie recovery to the weather or to a season. The world has no
  season, and a recovery that waits for one waits for a need nobody stated.
- It does not make a unit prefer a full deposit. Choosing where to gather
  belongs with unit behaviour.
- It does not model an animal, a herd, or anything that moves while it grows.
- It does not let a faction ruin ground permanently. Whether heavy extraction
  lowers what a tile can ever hold again is a separate need.

## What it costs at the target scale

**Growth looks like a pass over every tile, and it must not be one.** The
world holds 16.7 million tiles. A rule that visits each of them on each tick
to raise an amount pays the world for every deposit that nobody has touched,
and this project rejected that shape once already for gathering.

One fact about the world makes the cheap shape available. The starting amount
of every tile follows from the seed and the address, so the world stores no
amounts. It stores only what units took, and it stores that for a tile only
when somebody took from it. A world where nothing was gathered stores nothing
at all.

Recovery is therefore not growth of an amount. It is the removal of a fact.
The world holds a record that a unit took from one tile, and recovery makes
that record smaller until it goes. A deposit that nobody touched has no such
record, so recovery has no work to do on it, at any tile count.

These properties follow, and this record requires them.

- The cost of recovery grows with the number of deposits that units have
  depleted. It does not grow with the number of tiles and it does not grow
  with the size of the world.
- The set of depleted deposits shrinks as well as grows. A deposit that
  reaches its starting amount stops being a fact the world holds, so the set
  does not grow without bound over a long run.
- The amount a deposit holds at a given tick follows from what was taken,
  when it was taken, and the parameters of the kind. Nothing else. The world
  can therefore answer the question when it is asked, and does not have to
  revisit a deposit on a tick when nobody asks about it.
- Every amount is an exact whole number, so a total over tiles combines the
  same in any order, at any thread count.
- Recovery adds no storage for a tile that nobody gathered from.

**Why a crop is a different need, and is not in this record.** Recovery is a
property of the world that happens with nobody acting, and its cost follows
the depletion history. A crop is a thing a unit plants on a chosen tile,
tends over ticks, and harvests. That is an act on a site, and the engine
already carries acts on a site: an improvement is built over ticks on a tile,
and a site already produces and consumes at a rate. A crop would reach for
those, and this record would reach for none of them. Two needs with different
audiences, different cost drivers and different mechanisms are two records.
This record is the first, because a crop that grows in a world where nothing
else grows is a special case of a rule the world does not have. **No number
is reserved for the crop record.** The project reserves a number when it
writes the record, and reserving one now would state an intent as a fact.

No cost figure appears here. The one measurement this project holds was taken
on a development machine and not on the target platform, and every figure in
the project is derived rather than measured.[^1] The statements above give a
shape, not a number. The shape that matters is the one the project already
found: the term that grows with the number of things dominates the term that
grows with the number of tiles.[^2]

## Which blockers govern this

- **No measurement exists on the target platform.**[^1] Every cost statement
  above states a shape, not a number.

No other blocker governs this record. The tile edge, the world extent and the
simulated time in one tick are all answered, so a recovery period can be
stated in simulated time without inventing anything.[^3]

Two values this record needs are not blocked, because they need judgement and
not information. How long a deposit takes to return to its starting amount,
and whether a deposit that reached nothing recovers at all, are open rows in
the decisions register.[^4] [^5] This record states both as parameters and
states neither as a value.

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Findings register, FND-049. `docs/FINDINGS.md`
[^3]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^4]: Decisions register, DEC-049. `docs/DECISIONS.md`
[^5]: Decisions register, DEC-050. `docs/DECISIONS.md`
