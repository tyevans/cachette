---
id: 0343
title: Give a soldier a type that indexes a table
status: complete
created: 2026-09-03
implements: [ADR-0120 D1, ADR-0120 D2, ADR-0120 D3, ADR-0120 D4]
changes: []
creates: [ADR-0120]
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

## Impact review

**Governed by.** ADR-0066 D1 fixes the four entity shapes, and a unit type is a
column of one shape rather than a fifth shape. ADR-0012 D3 makes the unit arena
a struct of arrays, so a column is additive. ADR-0014 D1 and D2 keep the slot
index half of an identity, so the column is indexed by the slot and never
compacted. ADR-0002 D1 keeps every value in the table an exact integer or a
fixed-point value. ADR-0001 D4 puts the column and the table in the whole-world
state hash, because both decide what a later frame does. ADR-0011 makes the type
a newtype rather than a bare byte.

**Changes.** None. No record is superseded.

**Creates.** ADR-0120, a unit carries a type, and the type is an index into a
table the world is built with. The registry row was allocated before the record
was written.

**Blockers.** None. The values a row holds are content that a caller supplies,
so no value here waits on information the project does not have.

**Precedent.** Recurring defect shape 1 governs the column. A copy of a table
row on a unit would be one fact in two places, and nothing would fail when the
copies disagreed, so the column holds the index alone.

## Done when

- A unit carries a type, and the type indexes a shared table the world holds.
- The table holds an attack and an armour for each type, in the fixed-point
  scale, and it refuses a negative value and a row it does not hold.
- The column reaches the state hash, and the invariant check fails when a type
  names no row and when a dead slot carries a type.
- The control plane writes a row of the table and gives a set of units a type,
  all or nothing.
- The whole check command runs green.

## Outcome

Done. The unit type module holds the identifier, the row and the table. The
soldier arena gained the type column, and the column reaches the state hash and
the invariant check. The world holds one table and exposes a writer for a row.

The Python boundary gained two calls. One writes a row of the table. One gives
a set of units a type, and it resolves every identity and checks the type
before it writes anything.

**The table is square in the type count, and the code declares the width.** The
resolution of a meeting reads it for each ordered pair, so the cost of a fight
follows the square of that number.[^2]

Every stored golden hash moved, because the unit columns reach the hash byte for
byte and the arena gained a column. The commit body holds the recording command.

**No register row opened or closed for this item alone.** The findings and the
decisions that this work moved belong to item 0345, which built the contest that
reads the table.

## References

[^1]: Project orientation, the design principles. `CLAUDE.md`
[^2]: ADR-0121, a meeting between two factions resolves at the tile, decision D3. `docs/adrs/draft/adr-0121-a-meeting-between-two-factions-resolves-at-the-tile.md`
