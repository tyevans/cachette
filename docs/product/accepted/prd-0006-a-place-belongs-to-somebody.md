---
id: 0006
title: A place belongs to somebody
status: Accepted
created: 2026-08-30
---

# PRD-0006 — A place belongs to somebody

## Who this is for

A developer who builds a strategy game on this engine, and who needs the
world to hold sides that a player can recognise and act against.

A modeller needs this later, to study how a holding grows and where it stops.
A researcher does not need it.

## What the person cannot do today

A developer cannot say who anything belongs to.

A unit carries a faction, and that is the whole of it. A faction is a label
on a unit. It is not a label on the world. No tile belongs to anybody, so the
world has no sides, only units that happen to differ.

This has three costs.

The developer cannot express a boundary. Two groups can stand next to each
other, and nothing in the world says that one of them is somewhere it should
not be. Every idea that depends on a boundary is therefore unavailable:
trespass, defence, expansion, and loss.

The developer cannot give a unit a place to belong to. A unit has no home, so
it has nothing to return to, nothing to leave, and nothing to lose. A unit
that cannot lose anything cannot produce a story.

The developer cannot make a decision matter beyond the unit that made it. A
faction that owns nothing is not affected by what its units do. Nothing
accumulates, so nothing has consequences.

## What good looks like

Each statement below can be checked.

- A tile can be held by a faction, or by nobody.
- A watcher can see who holds a tile, and can see where one holding meets
  another.
- A holding changes during a run, by a rule the world applies. A holding
  fixed at generation does not satisfy this.
- What a faction holds is a fact the world can report. A developer can ask
  what a faction holds without walking every tile.
- Holding is exclusive. No tile is held by two factions at once, and the
  world's own invariant check re-derives the census, the held list and the
  block masks from the holder column and refuses a disagreement.
- Terrain influences holding. A holding does not spread the same way across
  every kind of ground.
- The same seed gives the same holdings, at every thread count, on every run.
- Exactly one value names the faction that owns a tile, and every interface
  that reports a tile's owner reports that value. A comparison between a
  unit's faction and a tile's owner is then meaningful.

## What this does not do

- It does not model government. A kingdom here is a holder of ground and a
  side in a conflict. Law, succession and title are not the need.
- It does not decide how a holding spreads. Growth from a centre, claim by
  presence and claim by force are all candidates. This record needs one rule
  that works.
- It does not give a faction a decision. A faction holds ground. It does not
  choose. Choosing belongs with unit behaviour.
- It does not model diplomacy. Whether two factions are at peace is a
  separate need, and it arrives with trading.
- It does not require a capital, a settlement or a building. A holding is
  ground. A place that produces something belongs with improvements.
- It does not give a unit an obligation to its faction. Employment and duty
  are a separate need.

## What it costs at the target scale

Two cost drivers matter, and they pull in opposite directions.

The first is storage. A holder for each tile costs the world, whatever the
number of factions. The second is the query. A faction that must walk the
world to learn what it holds pays the world for an answer whose size is the
holding.

A solution must not pay the world twice. These properties follow.

- What a faction holds can be answered at a cost that grows with the holding,
  not with the size of the world.
- The storage does not grow with the number of factions multiplied by the
  size of the world. A plane for each faction has that shape, and this record
  rejects it.
- The spread of a holding costs the area that changed, not the area that
  exists.
- A holding combines exactly under any order, so an aggregate of it does not
  depend on how the work was divided.

No cost figure appears here, because the one measurement this project has was
taken on a development machine and not on the target.[^1] That measurement
established the shape that matters: the term that grows with the number of
things dominates the term that grows with the number of tiles.[^2]

## Which blockers govern this

- **One blocker governs every cost figure here.**[^1] It says which figures
  are measured and which are derived. Every cost statement above states a
  shape, not a number.

The faction ceiling is answered, so this record states no faction count.[^3]

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Findings register, FND-049. `docs/FINDINGS.md`
[^3]: Blockers register, BLK-014. `docs/BLOCKERS.md`
