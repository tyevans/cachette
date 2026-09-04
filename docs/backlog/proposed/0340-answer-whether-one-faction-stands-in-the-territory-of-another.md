---
id: 0340
title: Answer whether one faction stands in the territory of another, in one call
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0031]
blocked-by: []
---

## Why

A downstream game gates a conversation between two players on presence. One may
speak to another only while one of its own units stands in the other's
territory. The engine holds every part of that answer and exposes none of it as
one call.

A tile carries one holder. A unit carries a faction and a tile. The control
plane reads the holder of one address at a time, and it cannot list the units of
a faction at all, so the only route today is a loop over the population.

**The answer for the whole world is one set of factions for each faction.** The
world admits at most 63 factions, so the whole relation is a small fixed number
of words. Deriving it rides on the pass that already visits every unit and the
tile it stands on. The findings register holds the reasoning.[^1] The decisions
register holds the options.[^2]

## What is missing before this can be refined

- The choice between a derived relation, the selector, and both, which the
  decisions register holds.[^2]
- The record that states where the relation lives and that it is derived rather
  than stored. A research report names it.[^3]
- Whether the game's rule is symmetric. One blocker holds it, and the engine can
  answer either way.[^4]

## References

[^1]: Findings register, FND-362. `docs/FINDINGS.md`
[^2]: Decisions register, DEC-141. `docs/DECISIONS.md`
[^3]: Research report 21, what a god needs from this engine, section 6.2. `docs/research/reports/21-what-a-god-needs.md`
[^4]: Blockers register, BLK-050. `docs/BLOCKERS.md`
