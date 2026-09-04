---
id: 0380
title: Run two factions together through the movement pass
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0030]
blocked-by: [BLK-080]
---

## Why

A fight resolves at the tile, and a tile contest needs both factions on one
tile.[^1] The measurement that settled the granularity placed the armies on
shared tiles directly, through the placement call, which skips the admission
rule and skips the movement pass.[^2]

**So the arrangement the measurement used is one the engine has never produced
for itself.** The admission rule reads the capacity of the ground and not the
faction, so nothing refuses a mixed tile. Ordinary ground holds 8 units, and a
tile already full of one faction offers no room.[^3]

A contest that never fires costs nothing and does nothing. This item measures
how often a running world reaches a contested tile, so that item 0345 sizes its
pass against a number rather than a hope.

## What is missing before this can be refined

- A seed set the control plane can name. A unit takes its direction from a field
  over cells, and no call sends one army at another, so nothing can drive the
  case today. Item 0342 holds that work.
- Whether the measurement counts contested tiles, contested blocks, or both.
- Whether a unit that meets an enemy should stop. That is a rule the project
  owner has not stated, and it changes the answer.

## References

[^1]: Decisions register, DEC-144. `docs/DECISIONS.md`
[^2]: Blockers register, BLK-080. `docs/BLOCKERS.md`
[^3]: Findings register, FND-392. `docs/FINDINGS.md`
