---
id: 0450
title: Read a unit type and the table from Python
status: complete
created: 2026-09-03
implements: [ADR-0120 D1, ADR-0120 D2, ADR-0120 D3, ADR-0040 D1, ADR-0043 D1, ADR-0085 D1, ADR-0085 D3, ADR-0107 D2]
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

A unit carries a type, and the type indexes a shared table of an attack and an
armour. The resolution of a meeting reads that table, and it refuses an attacker
whose attack does not exceed the armour of the defender. One tank therefore
beats any number of bowmen.

The control plane could write the table and give a set of units a type. **It
could read neither back.** A caller that cannot read the table cannot choose a
type, because the numbers in the rows are content that the caller itself wrote
and then lost. A caller that cannot read the type of a unit cannot tell whether
a command landed.

This item is the two reads. It builds no verb and it changes no engine
behaviour.

## Impact review

**Governed by.** ADR-0120 D1 makes a type an index into a shared table, and D2
makes that table data the world holds. D3 keeps the default at row zero. ADR-0040
D1 keeps the boundary carrying an instruction and an answer, never the
population. ADR-0043 D1 makes the shape of an interface follow the tier of what
it names, so a write over soldiers takes a set and a read of one soldier stays
singular. ADR-0085 D1 and D3 make an identity opaque and make the engine resolve
it against the generation. ADR-0107 D2 puts the prose in the Rust doc comment.

**Changes.** None. No record is superseded.

**Creates.** No record. The reads state no constraint that the records above do
not already state, so section 1 of the record scope rule refuses one.

**Blockers.** None.

**Register.** DEC-250 records how a caller learns the width of the table.

**Precedent.** Recurring defect shape 1 governs the width. A second call that
answered the number would be one value in two declaration sites, with nothing
that fails when they disagree.

## Done when

- A caller reads the type of one soldier, and the read refuses a stale identity.
- A caller reads the whole table, with the attack and the armour of each row,
  and the doc comment states the fixed-point scale of both.
- The width of the table has one declaration site that a caller can read.
- A test at the Python boundary proves that one tank beats a set of bowmen.
- The whole check command runs green.

## Outcome

Done. The boundary gained two reads. One answers the type of one soldier. One
answers the table as two columns, and the length of a column is the width.

**The tank is reachable from Python.** A test writes a bowman row and a tank
row, gives 32 units of one faction the bowman row and one unit of another
faction the tank row, and steps. The tank ends every bowman and never falls. A
control test runs the same meeting with no row written and nobody falls.

The write verbs were already bound when this work started. The item therefore
covers the reads alone, and the test covers the whole path.

## References

None.
