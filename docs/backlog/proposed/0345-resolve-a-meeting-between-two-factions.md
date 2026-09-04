---
id: 0345
title: Resolve a meeting between two factions
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0030]
blocked-by: [BLK-052]
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
- Item 0344 must report. It decides whether the resolution sits at the tile or
  at the cell.[^3]
- Two choices must close: where the fight resolves, and whether the threshold is
  hard.[^4] [^5]
- The records the report names. The determinism half of this needs a written
  constraint, because a later contributor who wants a smoother fight will reach
  for a draw for each unit and nothing will refuse it.[^6]
- A test for each field of the draw key. A draw keyed on the tile and not on the
  frame kills the same units for ever, and both determinism tests pass while it
  does.[^7]

## References

[^1]: Research report 21, what a god needs from this engine, section 4.1. `docs/research/reports/21-what-a-god-needs.md`
[^2]: ADR-0106, a cohort serves whole rations to a keyed subset, decisions D1 and D2. `docs/adrs/draft/adr-0106-a-cohort-serves-whole-rations-to-a-keyed-subset.md`
[^3]: Blockers register, BLK-052. `docs/BLOCKERS.md`
[^4]: Decisions register, DEC-144. `docs/DECISIONS.md`
[^5]: Decisions register, DEC-145. `docs/DECISIONS.md`
[^6]: Research report 21, what a god needs from this engine, section 6.2. `docs/research/reports/21-what-a-god-needs.md`
[^7]: Testing Rules, section 2. `.claude/rules/testing.md`
