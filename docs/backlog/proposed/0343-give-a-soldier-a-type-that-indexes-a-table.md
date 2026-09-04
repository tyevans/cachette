---
id: 0343
title: Give a soldier a type that indexes a table
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0030]
blocked-by: []
---

## Why

A soldier carries a generational identity, a tile address, a faction, a carried
load, a gather order and a build order. **It carries no type.**

The project owner's acceptance test for combat is that one tank still kills four
bowmen. Nothing about that test is expressible until a unit has a type, because
a tank and a bowman are the same thing to this engine.

The project already states the shape: a unit type is an index into a shared
table, and types parameterise the verbs rather than multiplying them.[^1] The
unit arena is struct-of-arrays, so a column is additive rather than a rewrite.

This item is the state, not the contest. It builds no fight and it decides no
combat rule.

## What is missing before this can be refined

- What the table holds. A type that indexes nothing is a number nobody reads.
- Whether the type reaches the choice pass. Every unit alive shares one weight
  profile today, so two units in one cell with the same need choose alike.
- Whether the unit arena reorder should land first. One item holds that work,
  and adding a column to the unit row while the layout is moving costs the work
  twice.[^2]

## References

[^1]: Project orientation, the design principles. `CLAUDE.md`
[^2]: Backlog priority index, item 0266. `docs/backlog/PRIORITY.md`
