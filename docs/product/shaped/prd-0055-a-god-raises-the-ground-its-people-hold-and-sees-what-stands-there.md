---
id: 0055
title: A god raises the ground its people hold, and sees what stands there
created: 2026-09-05
---

# PRD-0055 — A god raises the ground its people hold, and sees what stands there

## Who this is for

A developer who builds a game in which a god directs a congregation. The god
is a person or a language model, and it acts through the control plane. The
god holds ground, and it wants the ground to become better than the world made
it.

A watcher who reads the map serves second. A watcher must see what a faction
built, and how far it has gone, without a query.

## What the person cannot do today

A god cannot make an upgrade fit the ground.

A unit builds a road or a terrace on any tile it stands on. Ground that yields
a resource and ground that yields nothing take the same upgrade. A terrace on
bare rock and a terrace on a field are one thing. The world holds ground of
five kinds, and an upgrade ignores all of them.

A god cannot raise an upgrade. A built thing is finished or it is not. A
faction that holds a tile for the whole run holds the same terrace it built on
the first day. There is no second stage, so long tenure gains nothing after
the first build.

A watcher cannot read an upgrade. The map shows that a tile carries a built
thing. It does not show which thing, and it does not show how far the thing
has gone.

A god cannot plan a road. Each unit builds where it stands, so a road is a
mark of where a unit happened to be. Two sites of one faction are joined by a
road only by accident. Nothing reads what the faction lacks and lays a road
toward it.

This has three costs.

The ground does not matter. A faction that holds a river valley and a faction
that holds a mountain hold the same choices, so there is no reason to want one
place over another.

Tenure does not matter. A faction cannot invest in a place over time, so the
world holds no record of a long occupation.

The map does not tell a story. A watcher cannot see a faction grow richer, and
a god cannot see whether its orders were carried out.

## What good looks like

Each statement below can be checked.

- An upgrade suits the ground under it. A build order for an upgrade that does
  not suit the ground is refused, and the refusal is counted.
- Some ground yields a resource, and an upgrade exists that raises what that
  ground yields. Some ground yields nothing, and an upgrade exists for it too.
- An upgrade has a level. A watcher reads the level from the map without a
  query.
- Two levels of one upgrade look different on the map. Two upgrades of one
  level look different on the map.
- A level raises what the ground yields, or raises what the ground holds, or
  both. A higher level raises it more.
- The order that builds an upgrade also raises it. A god does not need a
  second order to reach the next level.
- A road is laid where the faction's plan says, and not where a unit happens
  to stand.
- A plan follows what the faction lacks. A faction with two unconnected sites
  plans a road between them. A faction with no unconnected site plans no road.
- A god may write a plan of its own, and a unit builds what that plan zones.
- The same seed and the same orders give the same upgrades, the same levels
  and the same roads, at every thread count, on every run.

## What this does not do

- It does not decide wear. Whether a level falls when nobody keeps it is the
  need beside this one.[^1]
- It does not decide territory. Who holds a tile is a need beside this one,
  written at the same time as this record. What happens to an upgrade when
  the holder changes is a blocker the project owner holds.[^2]
- It does not decide what a road costs to cross. A road changes how many
  units cross a tile. What it does to the pace of a unit is a movement need.
- It does not name the upgrades. Which categories exist, how many levels each
  has and which ground each suits are rules of the downstream game.[^3]
- It does not name the colours or the shapes. That the map tells two things
  apart is the need. How it tells them apart is a view choice.
- It does not make a plan clever. A plan that follows what the faction lacks
  is enough. A plan that predicts what the faction will lack is a later need.
- It does not let a unit refuse the plan. A unit told to build a zoned project
  builds it.

## What it costs at the target scale

The cost driver is the number of upgrades and the number of projects a
faction has planned. It is not the size of the world.

A faction holds a bounded plan, so the cost of planning follows the faction
count and the plan bound. It does not follow the population, and it does not
follow the tile count.

Four properties follow. A solution must have all four.

- Resolving which upgrade fits a tile costs a read of the ground and a read
  of the upgrade catalogue. It costs no pass over the world.
- Raising a level costs what a build costs today. A level is not a second
  built thing, so the storage of an upgrade does not grow with its level.
- Planning costs the faction count times a bounded plan. A world with more
  tiles plans no more roads.
- Drawing a level costs what drawing an upgrade costs today. The watcher reads
  what the engine holds and computes nothing.

No cost figure appears here. One blocker governs every cost figure this
project holds, and it says which figures are measured and which are
derived.[^4]

## Which blockers govern this

- **One blocker governs every cost figure here.**[^4] Every cost statement
  above states a shape and not a number.
- **One blocker holds the rules of the downstream game.**[^3] The categories,
  the level count of each, the ground each suits, the work each level takes
  and what each level raises are values of that game. The engine holds a value
  for each, and this record states none of them.
- **One blocker holds whether an upgrade changes hands with the ground under
  it.**[^2] The project owner holds it. A level on a tile whose holder changes
  is one instance of that question, and this record states nothing about it.

This record extends one accepted need, that a unit changes the ground it
stands on.[^5] That need rejected a catalogue. This record asks for one, as
data of the downstream game, and it asks that the ground and the level shape
what the catalogue holds.

## References

[^1]: PRD-0052, an upgrade wears and a worker repairs it. `docs/product/accepted/prd-0052-an-upgrade-wears-and-a-worker-repairs-it.md`
[^2]: Blockers register, BLK-036. `docs/BLOCKERS.md`
[^3]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^5]: PRD-0008, a unit changes the ground it stands on. `docs/product/accepted/prd-0008-a-unit-changes-the-ground-it-stands-on.md`
