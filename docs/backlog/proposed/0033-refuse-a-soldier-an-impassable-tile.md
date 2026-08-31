---
id: 0033
title: Refuse a soldier an impassable tile
status: proposed
created: 2026-08-30
implements: []
changes: []
creates: []
serves: [PRD-0003]
blocked-by: []
---

The terrain says whether a unit may stand on a tile. Nothing reads that
answer. A soldier walks into water, and no test fails.

This is the inert-capability shape: the project declares a capability, tests
it directly, and nothing invokes it. The test must start at the engine.

The work makes the movement system reject an impassable target, and makes a
spawn onto an impassable tile an error. A test then drives a stepping world
and asserts that no soldier ever stands on water.

The item is held out of the terrain milestone because the movement system and
the admission rule were owned by other agents at the same time. The impact
review must name the movement record and say how admission and passability
compose, because a tile that refuses a unit and a tile that is full are two
different refusals.
