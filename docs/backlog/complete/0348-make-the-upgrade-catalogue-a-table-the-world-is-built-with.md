---
id: 0348
title: Make the upgrade catalogue a table the world is built with
status: complete
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0030]
blocked-by: []
---

## Why

The engine holds two upgrade kinds, a road and a terrace, as a Rust enumeration.
Each carries a fixed work cost and a fixed effect on the tile.

A game that wants a shrine, a temple or an altar cannot add one. It must change
the engine and take a new release.

**The project already states the opposite rule.** Unit types and upgrades are
data and not code, an upgrade set is an interned identifier, and types
parameterise the verbs rather than multiplying them.[^1] The enumeration is the
one place that rule is currently testable, and it fails there. The decisions
register holds the options.[^2]

A lookup in a table is not a callback, so nothing about the frame or about
determinism changes.

## What is missing before this can be refined

- The choice in the decisions register must close.[^2]
- What an entry may declare. A work cost is easy. An arbitrary effect on a tile
  is content-supplied behaviour, and the choice pass already states that a
  content-supplied weight is a value in a table and never a function.
- How the catalogue reaches the state hash. Two worlds built with different
  catalogues must not compare equal by accident.
- Whether it lands before or after item 0341. A binding written against an
  enumeration is a binding that changes when the catalogue arrives.

## Outcome

**Item 0486 did this work, and this item needs none of its own.** The upgrade
kind is a table the world is built with. A row is one category at one level,
and it holds the ground it fits, the work it takes and the columns a pass
reads. A caller writes a row through the boundary, and the table enters the
whole-world hash.[^3]

The three open questions are answered. A row declares whole-number columns and
never a function. The table enters the hash byte for byte, so two worlds built
with different tables never compare equal. Item 0341 landed first, and item
0486 moved the boundary it wrote.

## References

[^1]: Project orientation, the design principles. `CLAUDE.md`
[^2]: Decisions register, DEC-143. `docs/DECISIONS.md`
[^3]: Backlog item 0486. `docs/backlog/complete/0486-turn-the-upgrade-kind-into-a-table-of-category-ground-fit-and-level.md`
