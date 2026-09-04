---
id: 0345
title: Resolve a meeting between two factions
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0030]
blocked-by: [BLK-080]
---

## Why

Two factions can stand on one place and nothing happens. The engine holds no
contest, and a downstream game names attacking as one of the six things its
players must do.

The sketch is a table over unit types rather than a fight for each pair of
units. A type whose effect does not exceed the defender's threshold contributes
exactly zero, so a sum of zeroes stays zero and no number of weak attackers ever
reaches a strong defender. That is the project owner's acceptance test, and the
threshold satisfies it structurally rather than by tuning a constant.[^1]

**Casualties are whole units served to a keyed subset.** The project already
holds the rule: a cohort serves whole rations to a keyed subset, never an equal
share to everybody, and the subset is the ordinals rotated by a keyed
offset.[^2] The arithmetic module already floors a share and leaves the
remainder to the caller. One keyed draw serves a whole group, and a draw for
each unit is what this reuses the rule to avoid.

## What is missing before this can be refined

- Item 0343 must land. Nothing here is expressible until a unit has a type.
- One choice must close: whether the threshold is hard.[^5] The project owner
  owns it.
- **The granularity is settled.** A fight resolves at the tile. The band that
  holds the middle 90 percent of the casualties is 1 tile wide at the tile and
  up to 30 tiles wide at the level 1 cell, and about seven in ten of the
  casualties of a cell resolution stand on a tile that holds no enemy.[^4]
- **What a contest reads is open, and it is new.** Ordinary ground holds 8
  units, and the admission rule reads the capacity and not the faction, so an
  army packed to that capacity cannot be entered and a same-tile rule never
  fires against it. A decision row holds it.[^3]
- **Nothing has driven two factions onto one tile through the movement pass.**
  The measurement placed them there directly. Item 0380 holds that gap.
- The records the report names. The determinism half of this needs a written
  constraint, because a later contributor who wants a smoother fight will reach
  for a draw for each unit and nothing will refuse it.[^6]
- A test for each field of the draw key. A draw keyed on the tile and not on the
  frame kills the same units for ever, and both determinism tests pass while it
  does.[^7]

## References

[^1]: Research report 21, what a god needs from this engine, section 4.1. `docs/research/reports/21-what-a-god-needs.md`
[^2]: ADR-0106, a cohort serves whole rations to a keyed subset, decisions D1 and D2. `docs/adrs/draft/adr-0106-a-cohort-serves-whole-rations-to-a-keyed-subset.md`
[^3]: Decisions register, DEC-170. `docs/DECISIONS.md`
[^4]: Findings register, FND-390. `docs/FINDINGS.md`
[^5]: Decisions register, DEC-145. `docs/DECISIONS.md`
[^6]: Research report 21, what a god needs from this engine, section 6.2. `docs/research/reports/21-what-a-god-needs.md`
[^7]: Testing Rules, section 2. `.claude/rules/testing.md`
