---
id: 0049
title: The world holds things worth taking
status: complete
created: 2026-08-31
implements: [ADR-0002 D1, ADR-0003 D1, ADR-0004 D1, ADR-0012 D3, ADR-0014 D1, ADR-0018 D3, ADR-0056 D3, ADR-0068 D1]
changes: []
creates: [ADR-0072, ADR-0073]
serves: [PRD-0007]
blocked-by: [BLK-007]
---

## Why

A unit has no reason to prefer one tile to another. The ground differs, and
every tile offers the same thing, which is nothing. A developer therefore
cannot make a place valuable and cannot make a tick cost a unit anything.

This item gives a tile a stock of a resource, and gives a unit a way to take
from it. It is the first quantity the world produces.

## Impact review

**Governed by.**

- ADR-0002 D1 forbids a floating point number in simulated state. Every
  amount here is an exact integer, and a sum over tiles is exact in any
  order.
- ADR-0003 D1 keys every random draw on the tuple of system, frame, entity
  and draw index. The generated stock takes a new system identifier, a
  constant frame, the tile address as the entity, and the resource kind as
  the draw index.
- ADR-0004 D1 fixes the iteration order of every result. The gather resolve
  reads its intents in a sorted order and never in a thread completion
  order.
- ADR-0012 D3 holds a unit in a generational arena of dense columns. What a
  unit carries becomes a column of that arena.
- ADR-0014 D1 pairs a slot index with a generation. The gather sort breaks
  its tie on the identity and never on the slot.
- ADR-0018 D3 rebuilds the derived unit structure at the barrier. The gather
  resolve reads that structure, so it runs after the barrier of its frame.
- ADR-0056 D3 resolves a contested tile by sort-then-admit. The gather
  resolve takes the same shape against the stock of a tile.
- ADR-0068 D1 generates the ground from the seed and stores no map. The
  stock follows it.

**Changes.** None. No accepted record changes.

**Creates.** Two records, and both rows are in the registry before this item
starts.

- ADR-0072 states that a stock is generated and that only what was taken is
  stored.
- ADR-0073 states that gathering is admitted by sort-then-admit against the
  tile.

**Blockers.** BLK-007 governs every cost figure. This item states the shape
of the cost and quotes no budget. No blocker governs a value this item needs,
so nothing here is parametric.

**Precedent.**

- FND-048 records that a determinism test cannot see a broken invariant. The
  conservation rule therefore joins the world invariant check, and a test
  asserts it directly.
- FND-051 records that a fixture chosen for realism hides the defect it
  should show. The contention fixture asserts that the demand exceeds the
  stock rather than assuming it.
- FND-054 records that a test world smaller than the lattice spacing holds
  one terrain. A resource test reads an extent wider than that spacing.

## Done when

- A tile holds a stock of each resource kind, and the stock is a pure
  function of the seed and the tile address.
- The terrain kind sets what a tile can hold. Water holds nothing, and a
  forest holds more wood than a plain.
- A world in which nothing was gathered stores no stock at all.
- A unit told to gather takes from the tile it stands on, and the stock falls
  by exactly what the unit took.
- What left every tile equals what every unit carries, plus what left the
  world with a dead unit. The world invariant check holds this.
- A tile that holds nothing gives nothing.
- Two units that contend for one nearly empty deposit resolve by the sort.
  The thread-count test covers that case and asserts that the fixture
  produced it.
- Each field of the stock draw key has its own test.
- A watcher reads the stock of a tile, reads what a unit carries, and reads
  an event for each amount taken.
- The whole check command runs green.

## Outcome

**Done as planned.** The stock of a tile is generated from the seed, the tile
address and the ground. The engine stores what was taken, in a sparse ledger,
and a world that gathered nothing stores nothing. The gather resolve sorts the
intents on a bounded key and grants each segment in that order. Conservation
joined the world invariant check.

**Changed from the plan.** Three things.

The gather rate is one number for every unit and every ground, and it opened a
decision row rather than a record.

A unit that dies takes its load out of the world, so conservation could not be
stated over the tiles and the live units alone. The world holds a register of
what departed, and the equality includes it.

The plan expected a fixture assertion over the demand and the supply. That
assertion needed a model of the rate that the test does not hold, and it
refused a fixture that was correct. The assertion is now over the outcome.

**Registers.** FND-057 records the fixture assertion. FND-058 records that a
probe build perturbs every subsystem at once, so a companion test that holds
everything else fixed is not always available. DEC-022 opens the gather rate.
BLK-007 stays open, and this item quotes no cost figure.

**Registry.** ADR-0072 and ADR-0073 exist as drafts. The author may not accept
them.
