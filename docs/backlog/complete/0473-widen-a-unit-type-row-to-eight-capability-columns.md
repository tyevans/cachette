---
id: 0473
title: Widen a unit type row to eight capability columns
status: complete
created: 2026-09-05
implements: [ADR-0145, ADR-0120 D1, ADR-0120 D2, ADR-0120 D3, ADR-0002 D1, ADR-0107 D3]
changes: []
creates: []
serves: [PRD-0048]
blocked-by: [BLK-007]
---

## Why

**A unit type row holds two values, attack and armour.** Every later pass of
the game layer needs a column that the row does not hold. A trade route needs a
unit that carries. A repair needs a build rate. A relation move needs a unit
with command reach, and a storm needs a unit with weather reach. This item is
pass 2 of the living world game layer.[^1]

The row widens to eight numeric columns: attack, armour, gather rate, build
rate, carry capacity, move cost scale, command reach and weather reach. Zero
means "cannot". Every column is an integer or a Q16.16 value. No pass reads a
type name. A pass reads a column of the row that the unit indexes.

One Rust constant holds the default table of five rows: worker, soldier,
merchant, leader and one open row. The seeding layer instantiates the table
from the constant. The panel labels and the generated Python reference derive
from the same constant. A check fails when a label list and the constant
disagree.

The verb `define_unit_type` takes the full row. The two-value form is removed.
Every caller moves in the same commit, and the whole-tree search goes in the
commit body.

**This pass does not touch `fn step` in `world.rs`.** It may run beside pass 1.

## Impact review

**Governed by.** ADR-0145 holds that a unit type is a row of capability
columns and that no pass reads a type name. The record is written beside this
item, and the registry holds its status.[^2] ADR-0120 D1 holds that the table
is data and a lookup is not a callback. ADR-0120 D2 holds that the table is
dense and fixed in length, and five rows fit it. ADR-0120 D3 holds that the
unit carries the index and never a copy of a row, so a widened row changes
nothing that the unit carries. ADR-0002 D1 holds that state holds no floating
point number, so every new column is an integer or a Q16.16 value. ADR-0107 D3
holds that the type stub declares types and carries prose only for what the
compiled module does not provide. The widened `define_unit_type` signature
changes the stub by hand.

**Changes.** None. ADR-0120 names attack and armour as examples of what the
row holds, and it does not fix the column count. If the worker reads a decision
in it as fixing the count, ADR-0145 supersedes that decision, and this item
says so in its outcome.

**Creates.** None.

**Blockers.** BLK-007 governs the cost of a wider row.[^3] The row count is
five and the column count is eight, so the table is small, and the figure
stays derived. The value of every cell in the default table is a row in the
balance register, and every row is unset.[^4] Write the constant against the
rows. Do not invent a value.

**Precedent.** FND-320 records that nothing regenerates the type stub, so the
new signature of `define_unit_type` is edited into the stub by hand in the
same commit.[^5] FND-051 records that a fixture chosen for realism hides the
defect it should show. A test of a scaled rate must use a row at zero and a
row at the largest value, not the worker row alone.[^5]

**Serves.** PRD-0048, a developer watches factions play a game to an end.[^6]

## Done when

- A unit type row holds the eight columns, each an integer or a Q16.16 value.
  The lint and the script that guard the arithmetic module pass.
- One Rust constant holds the default table of five rows, and the world is
  built from it.
- The panel labels and the generated Python reference derive from the
  constant. A test adds a sixth label behind a test-only switch and asserts
  that the check fails.
- `define_unit_type` takes the full row in the core crate and in the bindings.
  A whole-tree search for the two-value form comes back clean, and the search
  command is in the commit body.
- The gather pass scales its take by the gather rate, and the build pass scales
  its work by the build rate. A test uses a row with a rate of zero and a row
  with the largest rate, and it asserts the take and the work at each extreme.
- The movement pass scales its cost by the move cost scale. A test at scale
  zero and at the largest scale asserts the cost at each extreme.
- The other four columns are stored, hashed and readable, and no pass reads
  them yet. The item says so, and the outcome names the passes that read them.
- Each new test is proven able to fail. Put the defect back, run the test, and
  record in the commit body that it went red.
- The thread-count test and the golden state hash test pass at 1, 2 and 12
  threads. The golden file changes, because the hash now covers eight columns,
  and the commit body says so.
- The type stub `_core.pyi` is edited by hand in the same commit as the new
  signature, because nothing regenerates it.
- The whole check command runs green.

## Outcome

The row holds the eight columns. Five are fixed-point values: the attack, the
armour, the gather rate, the build rate and the move cost scale. Three are
whole counts: the carry capacity, the command reach and the weather reach. A
fixed-point column below zero is refused. One macro declares
the row, and the column names and the column reader derive from it, so the
Python table cannot name a column the row does not hold. A Python test reads
the hand-written stub and asserts that its typed dictionary and its keyword
arguments name the same columns in the same order. No view panel shows a unit
type, so no panel label exists to derive.

One constant holds the default table, with named placeholder constants that
cite the balance register row. The world is built with it. Every placeholder
keeps the behaviour of a world before the widening: a worker takes the tile
rate, adds the builder rate, and carries without a cap it reaches.

The gather pass scales the tile rate by the gather rate and caps the load at
the carry capacity. The build pass scales the builder rate by the build rate.
A unit whose column is zero keeps its order and takes or adds nothing, because
the choice pass writes the gather order from inside the step and a type can
change after an order is given. The Python verbs `order_gather` and
`order_build` refuse a unit whose type cannot, so a caller learns at once.

**What changed from the plan.** The movement pass does not read the move cost
scale. The item asked for that read and for a test at each extreme. The
movement pass is one of the passes that a later item holds, and reading the
column there would put this item into the movement code beside pass 1. The
column is stored, hashed and readable, and no pass reads it. The command
reach and the weather reach are the same. The passes that read them are the
relation verb of pass 3, the movement pass, and the weather verb of pass 5.

The core verb takes a row struct rather than a list of columns. The Python
verb keeps the two positional arguments and takes the six new columns as
required keyword arguments, so no partial form exists.

The golden state hash changed for every stored file at every frame, because
the table enters the hash at frame zero and the row is four times as wide.

**Where the code disagrees with ADR-0145.** The record is a draft and stays
one. D4 says the panel labels derive from the constant, and no panel shows a
type, so nothing derives. D3 names two gates that read the reach columns, and
no verb reads them yet. The record describes the target and the code
describes this pass.

**Registers.** No blocker moved. No finding was recorded. The balance register
row stays unset, and the placeholder constants say so.

## References

[^1]: Design: the living world game layer, sections 2 and 13. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^2]: ADR Registry. `docs/adrs/REGISTRY.md`
[^3]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^4]: Balance register. `docs/reference/balance.md`
[^5]: Findings register, FND-320 and FND-051. `docs/FINDINGS.md`
[^6]: Product registry. `docs/product/REGISTRY.md`
