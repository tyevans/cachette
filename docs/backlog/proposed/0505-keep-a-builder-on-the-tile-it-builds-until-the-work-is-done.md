---
id: 0505
title: Keep a builder on the tile it builds until the work is done
status: proposed
created: 2026-09-05
implements: [ADR-0152 D3, ADR-0159 D3]
changes: []
creates: []
serves: [PRD-0055]
blocked-by: [BLK-050]
---

## Why

**A builder adds one unit of work to a tile and then walks away.** The build
pass adds the build rate of a unit to the site the unit stands on, on every
tick that the unit holds a build order.[^1] The movement pass reads no build
order. A unit that builds therefore moves in the same way as a unit that does
not, and it leaves the tile on the next tick.

**A project finishes only when a random walk returns to one tile enough
times.** The first level of a road asks for eight work, and one builder adds
one.[^2] A faction founds with two people, so a faction has two builders.[^3]
In a measured run at extent 96 with three factions, the units of one faction
stood on a tile their own plan zoned for 253 ticks of 800, and 214 of those
ticks carried the matching build order. The work reached 31 separate tiles. No
tile passed seven of the eight that finish a road. The findings register holds
the reading and the command.[^4]

**One seed in ten therefore finishes no project at all.** A sweep of 200 seeds
of the demonstration world, run 800 ticks each, finished no project at 19 of
them. The seeds that finish none share nothing else. They seat the same four
factions, they hold the same settlements, and they zone the same number of
projects as the seeds that finish many.[^4]

**The plan bound is not the lever.** A bound of 4 in place of the default made
the reading worse, and 107 of 160 seeds then finished nothing, because a
smaller plan gives a wandering unit fewer tiles to stand on.[^4] A larger bound
spreads the same work over more tiles. No value of the bound makes a random
walk finish a build reliably.

**This is not the release that item 0502 holds.** That item frees a unit the
project order sent, so that the faction can aim it again.[^5] A freed unit that
still walks away from the tile it builds finishes nothing either. The two are
one chain and two rules, and this one states what a unit does after it arrives.

## What is missing before this is refined

A refiner answers these before this item leaves `proposed/`.

- **Which record states the rule, and whether a new one is needed.** A unit
  that stands still while it builds is a movement rule and a decision. The
  movement record fixes how a unit moves, and no record says that an order
  holds a unit in place.[^6] Say whether this is a decision of the plan record
  or a new record of its own.
- **What ends the hold, and what bounds it.** A unit held by an order it can
  never satisfy is stuck, and the register already holds an item about a unit
  that cannot move.[^7] Name the conditions that release it: the build
  finishes, the project goes, the ground changes hands, or a bound of ticks
  passes. A hold with no bound is a hold for ever.
- **Whether the hold belongs to the engine or to the controller.** A learner
  drives the same verbs as the built-in controller.[^8] Say whether the engine
  refuses to move a unit that builds, or whether the controller stops giving it
  a step. The first binds every caller. The second leaves a learner free to
  waste its units.
- **What the hold costs the movement pass.** The pass reads no order today. A
  read of one column for each live unit is one more column in the walk, and the
  target scale is one million units.[^9]
- **Which reading proves the change.** The sweep over 200 seeds is the
  measurement that found this, and it is the measurement that shows the
  change.[^4] State the bar: how many seeds of the sweep must finish a project.

## Done when

Not written. Refine the item first.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: The build pass and the build intents. `crates/cachette-core/src/world.rs`
[^2]: Balance register, the road work by level and the build rate. `docs/reference/balance.md`
[^3]: Balance register, the founding group. `docs/reference/balance.md`
[^4]: Findings register, FND-545. `docs/FINDINGS.md`
[^5]: Backlog item 0502. `docs/backlog/proposed/0502-let-a-faction-re-aim-its-project-order-and-keep-its-plan-live.md`
[^6]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^7]: Backlog item 0039. `docs/backlog/proposed/0039-a-rejected-unit-is-not-stuck.md`
[^8]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^9]: Project orientation, the target scale. `CLAUDE.md`
