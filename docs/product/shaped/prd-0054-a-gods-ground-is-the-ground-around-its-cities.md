---
id: 0054
title: A god's ground is the ground around its cities
status: Shaped
created: 2026-09-05
---

# PRD-0054 — A god's ground is the ground around its cities

## Who this is for

A developer who builds a strategy game on this engine, and who plays a god in
that game. A god directs a faction. The god wants the ground its faction holds
to mean something: a place the faction settled and improved, and not a stain
that spreads wherever its people walk.

A modeller needs this later, to study how a settlement pattern fills a world.
A researcher does not need it.

## What the person cannot do today

**A god cannot say why its faction holds a tile.** The world already gives each
tile one holder, and the holder is one faction or nobody.[^1] The world also
grows a holding during a run. The ground grows outward from wherever a unit
stands, and it keeps growing until it meets another holding, or ground that
refuses it. A holding therefore has no centre and no reason.

The costs follow.

**A god cannot lose ground by losing a city.** The ground a faction holds does
not depend on anything the faction owns. A faction whose every settlement is
gone still holds the ground, and it holds that ground for the rest of the run.

**A god cannot grow its ground by choice.** The ground grows by itself, at a
rate the god does not set, toward places the god did not choose. A god that
wants more ground has no act that gives it more ground.

**A god cannot see a frontier that means anything.** Two holdings meet where
their growth met, and that line records nothing about either faction. Nothing
the god built moved it.

**A unit builds anywhere.** The project owner answered that a unit builds only
on ground its own faction holds, and nothing checks it. A unit of one faction
finishes an improvement on ground another faction holds, or on ground nobody
holds.

**A god cannot make ground its own by using it.** A faction sends its people
to the same places outside its own ground, run after run, and holds none of
it. Use costs the faction and earns it nothing, so a border never moves toward
the way a god plays.

**A god cannot tidy the ground it has almost surrounded.** A piece of ground
that one faction rings on every side belongs to nobody. The god sees a hole in
its own ground, and no act of the game closes it.

## What good looks like

Each statement below can be checked.

- A tile that is far from every city a faction owns is held by nobody, unless
  another faction's city is near it, or a faction uses it, or one faction rings
  it.
- Ground held by a faction that owns no city is held by nobody after the next
  step.
- An improvement that is finished inside a faction's ground extends how far
  that ground reaches from the city, and the extension stops at a bound.
- A unit that orders a build outside the ground its own faction holds is
  refused, unless the build is a road. A road may be built on ground nobody
  holds.
- A settler stands on ground nobody holds and founds a city there. On the next
  step, the ground around the city is held by the settler's faction.
- A faction that plays itself founds new cities during a run, without a call
  from the developer.
- Where the ground of two cities overlaps, each tile is held by exactly one
  faction, and the same seed gives the same holder at every thread count and
  on every run.
- A watcher reads the holder of a tile before a city is founded and after it.
  The two readings differ.
- A faction that sends its people over the same ground, tick after tick, comes
  to hold that ground. It holds it even where no city it owns is near.
- Ground that a faction stops visiting is held by nobody again, after a stated
  number of ticks. A faction cannot keep ground it has abandoned.
- One unit that crosses a piece of ground far from every city of its faction,
  once, gives that faction nothing. The reading of the holder is unchanged.
- A piece of ground that one faction rings on every side comes to belong to
  that faction. A piece of ground that two factions ring, or that nobody
  rings, belongs to nobody.

## What this does not do

- It does not decide what happens to ground that one god trades to another.
  Traded land is a separate need, and that need already states it.[^2] This
  record states that ground far from every city of its holder is not held, and
  the reader of the traded land need must weigh that.
- It does not model a siege. A city that an enemy surrounds keeps its ground
  until the city is gone. Taking a city is a separate need.
- It does not model a wall or any structure that keeps a unit out. What a
  guest may do on ground another god holds is a separate need.
- It does not model diplomacy over ground. Whether two gods agree a border is a
  separate need.
- It does not decide what a city is worth or what founding one costs. That is
  a rule of the downstream game.
- It does not decide how the terrain shapes the ground, and it does not keep
  the rule that a faction never holds water. A piece of water that one faction
  rings on every side is a piece of ground that one faction rings. Whether a
  hill shortens how far a city reaches is a separate question.
- It does not decide whether a settler is used up when it founds a city.

## What it costs at the target scale

Two cost drivers matter, and this record changes which one dominates.

Today the growth of a holding costs the edge of the holding, and the edge grows
with the area held. At the target population the held ground reaches a large
share of the world, so a cost that follows the holding follows the world.[^3]

Under this record the ground follows the cities. A city holds a bounded area
around it, so the whole held ground is bounded by the number of cities
multiplied by the largest area one city can hold. These properties follow.

- Deciding who holds the ground costs the cities multiplied by the area one
  city can reach. It does not cost the world.
- Clearing ground that no city reaches costs the ground that was held, not the
  ground that exists.
- Refusing a build costs one read of the holder for each unit that orders a
  build.
- Founding a city costs one write for each tile the new city reaches.
- Every result combines exactly in any order, so an aggregate of the held
  ground does not depend on how the work was divided.

No cost figure appears here, because the one measurement this project has was
taken on a development machine and not on the target.[^4] The statements above
state a shape, not a number.

## Which blockers govern this

- **One blocker governs every cost figure here.**[^4] It says which figures
  are measured and which are derived.
- **One blocker governs every quantity this record needs.**[^5] The rules of
  the downstream game are one paragraph. How far a city reaches, how much one
  improvement extends it, how long use takes to win ground, how long disuse
  takes to give it back, and how far a ring reaches into the ground it
  surrounds are all rules of that game. This record states no distance and no
  duration. The values are parameters, and the balance register holds their
  rows when a pass writes them.
- **One blocker stays untouched.**[^6] It asks whether an improvement changes
  hands when the ground under it does. Traded land that carries an improvement
  is refused while the question is open, and this record does not close it.
  This record is cited where traded land meets a city's reach, because ground
  that leaves a faction's reach leaves its holding, and an improvement on it
  meets the same open question.

## References

[^1]: PRD-0006, a place belongs to somebody. `docs/product/accepted/prd-0006-a-place-belongs-to-somebody.md`
[^2]: PRD-0051, a god trades land. `docs/product/accepted/prd-0051-a-god-trades-land.md`
[^3]: Findings register, FND-285. `docs/FINDINGS.md`
[^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^5]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^6]: Blockers register, BLK-036. `docs/BLOCKERS.md`
