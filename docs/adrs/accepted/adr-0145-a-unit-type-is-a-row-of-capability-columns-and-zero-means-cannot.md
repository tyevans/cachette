# ADR-0145: A unit type is a row of capability columns, and zero means cannot

## Context

A unit carries a type, and the type is an index into a shared table that the
world is built with. The table holds data and never code.[^1] Before this
record a row held an attack and an armour, and the contest was the only pass
that read it.

The game layer needs more from a type. A merchant carries a load under a
contract and a worker does not. A leader may move a relation and a worker may
not. A faction inflicts weather only while it holds a unit that can. Each of
these is a question a pass asks about a unit, and each has the same answer
shape: can this type do that, and how much.[^2]

**The shortest path is a branch on a name.** A pass that asks "is this a
merchant" reads a name and takes a branch. The project already refused that
shape for the type itself, because an enumeration of kinds is code that a game
cannot extend.[^1] The refusal is not visible in the table, though. A
contributor who reads a row of contest values sees nothing that says a new
question must be a new column and not a branch.

**The second shortest path is a flag column.** A row would hold a bit that says
"can carry" beside a number that says how much. Two columns then hold one fact,
and nothing fails when a row says it can carry nothing.

This record widens ADR-0120 D1. That decision says a row states the properties
a pass reads. This record says what shape those properties take and what a zero
in one means.

## Decision

**A unit type is one row of numeric capability columns. A pass reads the
column for the question it asks. A zero in a column means the type cannot do
the thing the column names.**

### D1. Every question a pass asks about a type is one column, and every column is numeric

A row holds one column for each capability a pass reads. Each column is a
whole number or a fixed-point value in the project scale.[^3] No column is a
name, a flag or a code.

A pass reads the column of the row that the unit indexes. It never reads a
type name and never branches on a type index.

A reviewer finds a violation when a pass compares a type index to a constant,
or when a row gains a field that is not a number.

### D2. Zero means cannot, and there is no separate flag

A column at zero means the type cannot do what the column names. A carry
capacity of zero means the unit never carries. A gather rate of zero means the
unit takes nothing from a tile. A build rate of zero means the unit adds no
work to a site.

One number therefore answers both questions, whether and how much. No row holds
a flag beside a rate.

**A row that nobody filled is a unit that can do nothing.** This is the change
to ADR-0120 D1. Before this record a world whose table nobody filled behaved as
a world with no types at all. After it, a caller that wants a unit to gather
fills a gather rate. The default table the world is built with fills the rows a
demonstration needs.

### D3. A gate on a capability reads the faction's units, never a per-faction flag

When a faction may do something only while it holds a unit of a capable type,
the gate reads the type column of the units the faction holds. It reads no
separate per-faction flag, because a flag would be a second copy of a fact that
the unit columns already hold.[^4]

The row holds a command reach column and a weather reach column for this
gate. No verb reads either column yet. A verb that gates a faction power on one
of them must read the column of the units the faction holds, and the record
that defines the verb cites this decision for the gate. A column that no
verb reads is a declared capability that nothing invokes, and the record that
defines the verb closes that gap or removes the column.[^4]

### D4. The default table is one constant, and every other listing derives from it

One constant in the core crate holds the default table, and the seeding fills
the world from it. The row is declared once, and the column names and the
column reader derive from that declaration. A hand-written copy of the column
list, such as the Python type stub, is permitted only when a test compares it
to the declaration and fails when the two disagree.[^4]

The values of each row are balance values. They live in the reference tables
and never in this record.[^5]

### D5. The verb that defines a type takes the whole row

One verb defines a type and it takes every column. There is no partial form. A
caller that gave part of a row would leave the rest at zero, which under D2
defines a unit that cannot do what the missing columns name, and the caller
would not know it had done so.

## The alternatives this rejects

**A branch on a type name.** Rejected because it is code that a game cannot
extend, and because a table of names has no boundary: any question about a type
becomes a new branch in a pass.[^1]

**A flag column beside each rate.** Rejected because two columns hold one fact
and nothing fails when they disagree.[^4]

**A per-faction flag for each gated power.** Rejected for the same reason. The
units a faction holds already answer the question, and a flag that said
otherwise would be the second copy.

**Several tables, one per subsystem.** Rejected because a unit then indexes
several tables with one number, and the tables must agree on the length and on
the meaning of each index. One table, one row, one index.

## Consequences

**A game adds a capability by adding a column.** Every pass that reads the
column is a pass, and no pass gains a branch. The table gains a column when a
pass needs one and not before, because a column nothing reads is a declared
capability that nothing invokes.[^4]

**The resolution cost still follows the square of the type count.** This
record adds columns and not rows, so the bound that ADR-0120 D2 states is
unchanged.[^6]

**A caller cannot define a type in part.** The whole-row verb is the only
verb, so every caller that defined a type moves when a column is added. The
commit that adds a column searches the tree for the callers and names the
search in its body.

**A zero row is a unit that can do nothing, not a unit with no type.** A test
that builds a world with an empty table and expects a unit to gather now fails.
That is the correct failure.

**The state hash moves when a column is added.** A copy of the table enters
the hash byte for byte, so every stored golden hash changes. The commit records
the change.

## References

[^1]: ADR-0120, a unit carries a type, and the type is an index into a table the world is built with, decisions D1 and D3. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
[^2]: Design, the living world game layer, section 2. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^3]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^4]: Recurring Defect Shapes, shapes 1 and 3. `.agents/rules/recurring-defects.md`
[^5]: Budgets and costs. `docs/reference/budgets.md`
[^6]: ADR-0121, a meeting between two factions resolves at the tile, never at a level 1 cell, decision D3. `docs/adrs/draft/adr-0121-a-meeting-between-two-factions-resolves-at-the-tile.md`
