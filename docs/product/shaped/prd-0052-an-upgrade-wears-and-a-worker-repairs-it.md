---
id: 0052
title: An upgrade wears and a worker repairs it
status: Shaped
created: 2026-09-05
---

# PRD-0052 — An upgrade wears and a worker repairs it

## Who this is for

A developer who builds a strategy game on this engine, and who needs what a
faction builds to need keeping.

A modeller needs this second. A modeller wants a population that must spend
labour to stand still, because that is what a real population does.

## What the person cannot do today

A developer cannot make a built thing decay.

A unit builds a road or a terrace, and the thing stays built. Nothing weathers
it and nothing damages it. The only way it leaves the world is that somebody
destroys it on purpose. A world that has been built up stays built up for as
long as the run lasts.

This has three costs.

A faction has nothing to defend. An army that reaches an enemy road cannot
harm the road. War changes who holds the ground and changes nothing on it.

Weather is a nuisance and not a threat. A storm wets the ground and slows a
unit. It ruins nothing, so a faction has no reason to build against it.

Labour has no floor. A faction that has built what it wants has nothing left
for its builders to do. A population that must maintain what it holds always
has work.

## What good looks like

Each statement below can be checked.

- Every built thing has a condition. A new build is in full condition.
- Weather wears a built thing. A built thing on wet ground loses condition
  each tick the ground stays wet.
- An enemy wears a built thing. A unit whose faction is at war with the
  holder, standing on the tile, loses the thing condition each tick.
- A built thing at zero condition is gone. It leaves the world by the same
  path as a thing destroyed on purpose. A watcher cannot tell the two apart
  afterwards.
- A worker repairs a built thing. The order to build on a tile that already
  holds a built thing raises its condition. Full condition is the ceiling.
- Repair costs the worker what building costs. A faster builder repairs
  faster.
- A watcher reads the condition of a built thing on the map. The watcher can
  tell a worn thing from a new one.
- The engine holds a built thing that slows an enemy. It holds a built thing
  that stores more goods. It holds a built thing whose completion is a win.
- The same seed gives the same wear and the same repairs, at every thread
  count, on every run.

## What this does not do

- It does not decide what happens to a built thing when its ground changes
  hands. The project owner holds that question, and it is open.
- It does not name the rates. How fast a storm wears a road is a value of the
  downstream game. So is how much a worker repairs in a tick.
- It does not give a built thing an owner apart from the ground. Who holds
  the tile holds the thing.
- It does not decide how condition is stored. That is an architectural
  question, and it belongs in a decision record.
- It does not add a repair act. Repair is the build act on a built tile.
- It does not make wear a choice. A faction cannot wear a thing faster by
  wanting to.

## What it costs at the target scale

The cost driver is the number of built things and the area the weather
occupies. It is not the size of the world.

Wear from weather costs the built things on wet ground. Wear from an enemy
costs the built things an enemy stands on. Repair costs the builders at work.
None of those grows with the world.

Three properties follow. A solution must have all three.

- The wear pass costs the built things under a condition that wears them. A
  built thing under a clear sky and no enemy costs nothing.
- Condition is one small value per built thing. The storage grows with the
  count of built things and with nothing else.
- Removal at zero costs what a destroy costs today, because it is one.

No cost figure appears here. One blocker governs every cost figure this
project holds, and it says which figures are measured and which are
derived.[^1]

## Which blockers govern this

- **One blocker governs every cost figure here.**[^1] Every cost statement
  above states a shape and not a number.
- **One blocker holds whether a built thing changes hands with the ground under
  it.**[^2] The project owner holds it. This record states that the holder of
  the tile holds the thing. It states nothing about the moment the holder
  changes.
- **One blocker holds the rules of the downstream game.**[^3] Every rate of
  wear is a rule of that game. So are the full condition of each kind and the
  work each kind takes. The engine holds a value for each, and this record
  states none of them.

This record depends on a unit changing the ground it stands on, and on the
world holding weather. Both exist.[^4] [^5]

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Blockers register, BLK-036. `docs/BLOCKERS.md`
[^3]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^4]: PRD-0008, a unit changes the ground it stands on. `docs/product/accepted/prd-0008-a-unit-changes-the-ground-it-stands-on.md`
[^5]: PRD-0004, the world has weather that a watcher can read. `docs/product/accepted/prd-0004-the-world-has-weather-that-a-watcher-can-read.md`
