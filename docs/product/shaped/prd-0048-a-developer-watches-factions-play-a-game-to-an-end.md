---
id: 0048
title: A developer watches factions play a game to an end
status: Shaped
created: 2026-09-05
---

# PRD-0048 — A developer watches factions play a game to an end

## Who this is for

A developer who builds a strategy game on this engine. The developer must see
the engine play a whole game before writing a player onto it.

A modeller needs this second. A modeller wants a run that has an outcome, so
that two runs can be compared by what they produced. A researcher needs it
third. A run that ends is a run that can be repeated to the same end.

## What the person cannot do today

A developer cannot watch a game. The developer can watch a world.

The demonstration steps a world and draws it. A unit gathers where the engine
sends it. No faction plans, and no faction fights for a reason. Nothing wants
anything, so nothing is at stake. A run has no end other than the moment the
watcher closes it.

This has three costs.

The developer cannot judge the engine as a game. Every mechanism the engine
holds is visible only as motion. The developer cannot tell whether trade,
weather, contest and building add up to a contest that somebody can win.

The developer cannot tell whether a mechanism ever fires. A subsystem that
nothing drives stays quiet, and a quiet subsystem looks the same as a working
one.

The developer cannot show the engine to anybody. A watcher who is not a
developer sees units move and asks what is happening. The honest answer today
is nothing.

## What good looks like

Each statement below can be checked.

- Every faction in a run acts on its own. No Python code chooses for it.
- A faction acts only through the acts a caller can make. A watcher can name
  the act a faction made, and a caller could have made the same act.
- A faction works toward a win. The engine names at least four ways to win,
  and a run can end by each of them.
- A run ends. A reader states the winner, the way it won and the tick it won
  on. Before the end, the reader states that the run has not ended.
- The run ends once. A world that has ended keeps stepping, and the end does
  not change.
- A watcher sees the game as it happens. A declaration, a deal, a storm, a
  repair and the end each appear when they happen. Each names the faction it
  concerns.
- A reader states the standing of one faction toward each way to win.
- A reader states, for each mechanism the engine holds, how many times it
  fired in the run. A count of zero is a fact a developer can act on.
- No faction is a player. A faction can be marked as controlled from outside,
  and nothing sets the mark in this work.
- The same seed gives the same game and the same winner, at every thread
  count, on every run.

## What this does not do

- It does not give a person a way to play. A player hook is a separate need.
  This record asks only for the place where that hook will stand.
- It does not make a faction clever. A faction that acts on its own and acts
  badly satisfies this record. Balance is a separate need.
- It does not name the four ways to win. Which acts count toward a win is a
  rule of the downstream game, and the rule is not written.
- It does not decide how a faction chooses. That is an architectural question,
  and it belongs in a decision record.
- It does not stop the world at the end. A watcher may keep watching.
- It does not tell a faction anything its own units cannot see.

## What it costs at the target scale

The cost driver is the number of factions, not the number of units and not the
number of tiles.

A faction chooses from what the engine already summarises about the world. A
faction that walked its own population to decide would cost the population
every tick. One million units make that the wrong shape. This record rejects
it.

Three properties follow. A solution must have all three.

- What a faction spends to choose grows with the number of factions. Each
  faction makes a fixed count of choices. The cost does not grow with the
  population and it does not grow with the world.
- What a faction reads to choose is a reading the engine already keeps. No
  reading costs a pass over the world.
- What a faction does costs what the act already costs. A faction that orders
  a set of units pays what a caller pays for the same order.

No cost figure appears here. One blocker governs every cost figure this
project holds, and it says which figures are measured and which are
derived.[^1]

## Which blockers govern this

- **One blocker governs every cost figure here.**[^1] Every cost statement
  above states a shape and not a number.
- **One blocker holds the rules of the downstream game.**[^2] What a faction
  wants, what counts as a win, and how long a game lasts are rules of that
  game. The engine holds a value for each, and the blocker says that no owner
  chose them. This record states none of them.

This record depends on three needs that exist. A faction holds ground.[^3] A
faction deals with another faction.[^4] The world holds weather.[^5]

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^3]: PRD-0006, a place belongs to somebody. `docs/product/accepted/prd-0006-a-place-belongs-to-somebody.md`
[^4]: PRD-0034, two players hold each other to a future delivery. `docs/product/shaped/prd-0034-two-players-hold-each-other-to-a-future-delivery.md`
[^5]: PRD-0004, the world has weather that a watcher can read. `docs/product/accepted/prd-0004-the-world-has-weather-that-a-watcher-can-read.md`
