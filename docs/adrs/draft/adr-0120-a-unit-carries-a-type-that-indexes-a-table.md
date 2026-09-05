# ADR-0120: A unit carries a type, and the type is an index into a table the world is built with

## Context

A unit in this engine is a row of a column set. It carries a generational
identity, a tile address, a faction, a carried load, a gather order, a build
order, a need, a deficit, a home site and an intent.[^1] It carries nothing
that says what kind of unit it is.

The project owner stated an acceptance test for combat: one tank still kills
four bowmen. **That test is not expressible today**, because a tank and a
bowman are the same thing to this engine. Nothing on a unit distinguishes
them, so no pass can treat them differently.

The project already states the shape that the answer must take. A unit type is
an index into a shared table. An upgrade set is an interned identifier. Types
parameterise the verbs, and types do not multiply the verbs.[^2]

The alternative that a contributor reaches for is a Rust enumeration of unit
kinds, with a match in each pass that reads one. The engine already holds one
such enumeration, for upgrade kinds, and an open backlog item exists to remove
it, because a game cannot add a kind to it.[^3] A second enumeration would
repeat that mistake in the subsystem where a game most wants its own
vocabulary.

The research that surveyed what a downstream game needs from this engine names
the type as the first of five things the engine must represent before any
contest exists.[^4]

## Decision

**A unit carries a type. The type is an index into a shared table that the
world is built with. The table holds data and never code.**

### D1. The table is data, and a lookup in it is not a callback

The world holds one table. A row of it states the properties that a pass reads
for a unit of that type. The table holds no function, no closure and no
dispatch, so a pass that reads it takes the same branches for every type.

A caller fills the rows it wants and leaves the rest at their zero value. Every
value of a row is a capability column, and a zero in a column means that the
type cannot do what the column names. A later record widens this decision and
states that rule.[^10] A row that nobody filled is therefore a unit that can do
nothing, and the default table the world is built with fills the rows a
demonstration needs.

This is the design principle applied to one table. A verb takes the type as a
parameter. A verb is not written twice because two types exist.

### D2. The table is dense, square in the type count, and fixed in length

The table is an array indexed by the type. Its length never changes while a
world runs, so a lookup is one indexed read and never a search.

**The length is a structural property and not a budget.** The resolution of a
meeting reads the table for each ordered pair of types, so its cost follows the
square of the length.[^5] The length is therefore small on purpose. The code
declares it once, and no record states the number, because a number that a
content choice can move does not belong in a record.[^6]

### D3. The unit carries the index, and never a copy of a row

The column holds the type number. It holds no copy of the attack, of the
armour, or of any other property of the row.

A copy would be the table in a second place, with nothing that fails when the
two disagree. A world that changed a row would then hold units that fight by
the row the table used to have. That is the defect shape this project meets
most often, and the rule against it is written.[^7]

The consequence is that a pass which wants a property of a unit reads the type
column and then the table. That is two indexed reads and no branch.

### D4. The type is a column of the unit arena

The type is part of the unit shape. It is therefore a column of the arena that
holds units, and not a side table keyed on the identity.

The arena is a struct of arrays, so a column is additive.[^8] A side table
keyed on the identity would cost a lookup on a path that has none today, and it
would hold a second answer to the question of which units exist.

The column enters the whole-world state hash, because a type decides what a
later frame does.[^9] A dead slot holds the default type, in the way every
other column of the arena is cleared when a unit dies.

## Consequences

**A game states its unit vocabulary as data.** It fills rows of the table
before it runs the world. It adds no Rust and it recompiles nothing.

**The number of types a world may hold is bounded, and the bound is small.**
That is the price of the resolution cost following the square of it. A game
that wants many kinds of unit must express the difference in the values of a
row rather than in the number of rows.

**A row holds only what a pass reads.** The table gains a field when a pass
needs one, and not before. A field that nothing reads is a declared capability
that nothing invokes, which this project treats as a defect shape.[^7]

**The state hash moves when the type column is added.** Every stored golden
hash covers the unit columns byte for byte, so a new column changes every one
of them. The change is recorded in the commit that made it.

**This record decides no value.** It says that the table exists and what its
shape is. What a row holds for a tank, and what it holds for a bowman, is
content that a caller supplies.

## References

[^1]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^2]: Project orientation, the design principles. `CLAUDE.md`
[^3]: Backlog item 0348, make the upgrade catalogue a table the world is built with. `docs/backlog/proposed/0348-make-the-upgrade-catalogue-a-table-the-world-is-built-with.md`
[^4]: Research report 21, what a god needs from this engine, section 4.5. `docs/research/reports/21-what-a-god-needs.md`
[^5]: ADR-0121, a meeting between two factions resolves at the tile, decision D3. `docs/adrs/draft/adr-0121-a-meeting-between-two-factions-resolves-at-the-tile.md`
[^6]: Decision Record Scope, section 4.1. `.claude/rules/adr-scope.md`
[^7]: Recurring Defect Shapes, shapes 1 and 3. `.claude/rules/recurring-defects.md`
[^8]: ADR-0012, tiles are dense columns and units are a generational arena, decision D3. `docs/adrs/accepted/adr-0012-tiles-are-dense-columns-and-units-are-a-generational-arena.md`
[^9]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^10]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/draft/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
