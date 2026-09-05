---
id: 0052
title: An upgrade wears and a worker repairs it
status: Accepted
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
- An enemy wears a built thing. A unit at war with the holder stands on the
  tile. The thing loses condition each tick the unit stands there.[^1]
- A built thing at zero condition is gone. It leaves the world in the same
  way as a thing destroyed on purpose. A watcher cannot tell the two apart
  afterwards.
- A worker repairs a built thing. The order to build on a tile that already
  holds a built thing raises its condition. Full condition is the ceiling.
- Repair costs the worker what building costs. A faster builder repairs
  faster.
- A watcher reads the condition of a built thing on the map. The watcher can
  tell a worn thing from a new one.
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
- It does not ask a worker to know the difference. A worker told to build on
  a worn thing repairs it.
- It does not make wear a choice. A faction cannot wear a thing faster by
  wanting to.
- It does not add kinds of built thing. Which kinds exist, and whether one of
  them is a way to win, are rules of the downstream game.[^2]
- It does not make weather harm anything other than a built thing. What
  weather does to a store, a unit or a site is a separate need.

## What it costs at the target scale

The cost driver is the number of built things and the area the weather
occupies. It is not the size of the world.

Wear from weather costs the built things on wet ground. Wear from an enemy
costs the built things an enemy stands on. Repair costs the builders at work.
None of those grows with the world.

Three properties follow. A solution must have all three.

- The wear pass costs the built things under a condition that wears them. A
  built thing under a clear sky and no enemy costs nothing.
- What the world remembers about a built thing grows with the count of built
  things and with nothing else.
- Removal at zero costs what a destroy costs today, because it is one.

No cost figure appears here. One blocker governs every cost figure this
project holds, and it says which figures are measured and which are
derived.[^3]

## Which blockers govern this

- **One blocker governs every cost figure here.**[^3] Every cost statement
  above states a shape and not a number.
- **One blocker holds whether a built thing changes hands with the ground under
  it.**[^4] The project owner holds it. This record states that the holder of
  the tile holds the thing. It states nothing about the moment the holder
  changes.
- **One blocker holds the rules of the downstream game.**[^2] The rate at
  which an enemy wears a thing is a rule of that game. So are the full
  condition of each kind and the work each kind takes. The engine holds a
  value for each, and this record states none of them.
- **One blocker holds what weather is worth.**[^5] How fast wet ground wears a
  built thing is one of the values it governs. This record states no rate.

This record depends on three needs. A unit changes the ground it stands on.
The world holds weather. Two factions can be at war. The first two exist, and
the third is a need beside this one.[^6] [^7] [^1]

## References

[^1]: PRD-0049, a god declares war and makes peace. `docs/product/accepted/prd-0049-a-god-declares-war-and-makes-peace.md`
[^2]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^3]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^4]: Blockers register, BLK-036. `docs/BLOCKERS.md`
[^5]: Blockers register, BLK-130. `docs/BLOCKERS.md`
[^6]: PRD-0008, a unit changes the ground it stands on. `docs/product/accepted/prd-0008-a-unit-changes-the-ground-it-stands-on.md`
[^7]: PRD-0004, the world has weather that a watcher can read. `docs/product/accepted/prd-0004-the-world-has-weather-that-a-watcher-can-read.md`
