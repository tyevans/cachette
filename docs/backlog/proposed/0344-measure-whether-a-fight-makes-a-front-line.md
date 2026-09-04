---
id: 0344
title: Measure whether a fight makes a front line
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0030]
blocked-by: [BLK-052]
---

## Why

A design sketch for combat resolves a fight for each level 1 cell, as a small
table over unit types. A cell summarises a block of tiles, and the block edge is
a power of two set by one constant in the bridge.

**A fight resolved for a whole block kills units across the whole block.** So
the casualties may not form a front line, and an army may read as a smear. Two
factions have never been run into contact in this engine, so there is no
evidence either way. One blocker holds that gap.[^1]

The measurement is cheap and it decides between two designs. Building either one
first risks throwing it away.

## What the measurement is

The research report states the method in full.[^2] In outline: seed two factions
on opposite sides of a world, run them to contact, and report the band of tiles
that holds the middle 90 percent of the casualties. Compare that band against
the block edge.

**Do not copy the demonstration world.** That world is chosen to look right
rather than to produce an edge value, and this project has recorded the cost of
that twice.[^3]

**Put the defect back.** Resolve at the cell on purpose and confirm the band
widens to the block edge. A fixture that cannot show the bad case is measuring
itself.

## What is missing before this can be refined

- A casualty. Nothing kills a unit in a fight today, so the measurement needs a
  provisional resolution to measure. The item must say whether that provisional
  rule is thrown away afterwards.
- Whether the run happens on the target platform. This measures a shape rather
  than a cost, so a development machine may serve.

## References

[^1]: Blockers register, BLK-052. `docs/BLOCKERS.md`
[^2]: Research report 21, what a god needs from this engine, section 4.2. `docs/research/reports/21-what-a-god-needs.md`
[^3]: Testing Rules, section 2a. `.claude/rules/testing.md`
