# ADR-0151: An upgrade is a category with a ground fit and a level, and a build order names the category

## Context

A tile upgrade is the mark a unit leaves on a tile. The engine stores one entry
for each improved tile and nothing for any other, and a tile carries at most one
upgrade.[^1] The entry holds a kind and a progress accumulator. The kind is an
enumeration in the core crate with two variants, a road and a terrace. Each
variant carries a fixed work, a fixed capacity and a fixed yield bonus, written
as a match arm in code.

The project states the opposite rule. Unit types and upgrades are data and not
code, and types parameterise the verbs rather than multiplying them.[^2] The
unit type already follows the rule: a type is an index into a table the world is
built with, a row is numeric capability columns, a zero means cannot, and no
pass branches on a type name.[^3] [^4] The upgrade kind does not follow it. The
decisions register holds that gap as an open choice, and a backlog item asks to
close it.[^5] [^6]

Three needs now press on the enumeration at once. The product asks that an
upgrade suit the ground under it, that an upgrade have a level a watcher reads
from the map, and that a level raise what the ground yields or holds.[^7] The
wear work adds a wall, and the game end adds a wonder and a store.[^8] [^9]
Each of those is a new variant, a new match arm in every table, and a new
branch in the controller that draws a kind from the high bits of one draw.

The ground has five kinds, and every kind is a pure function of the seed and the
tile index.[^10] A deposit on a tile is generated the same way.[^11] So the
ground a tile offers is known at the moment an order arrives, and the engine
pays one read for it. The entry is the stored change over that generated base,
which is the shape every tile field takes.[^12]

**The shortest path adds variants.** Five variants become eight, each table
gains three arms, and a level becomes a second variant beside the first, so a
terrace at level two is a kind of its own. A level then multiplies the variants,
and a category of three levels on two ground kinds is six arms in every table.

**The second shortest path adds a level to the entry and keeps the enumeration.**
The entry gains a level field, and every effect table takes the level as a
second argument. The ground fit then has nowhere to live except a third match
in the build verb, and a game that wants a shrine still needs a release.

This record chooses a third shape. It is the shape the unit type already has.

## Decision

**An upgrade is a row of a table that the world is built with. A row is one
category at one level, and it names the ground it fits, the work it takes and
what it changes. A build order names a category. The engine resolves the row
from the ground under the tile and the level that stands there, and it refuses
when no row fits.**

### D1. The upgrade table is data, one row for each category and level

The table has one row for each pair of a category and a level. A row holds the
ground it fits, the work that finishes it, and the columns a pass reads. Today
those columns are the yield bonus and the capacity. A column exists when a pass
reads it, and not before.[^13] A row with a category that fits no ground and a
level that no row reaches cannot be written, because the fit and the level are
part of the key.

The ground fit is stated over the ground kinds, and it may require that the
tile yields a resource. A row that fits every kind fits every kind. A row that
fits no kind is not a row.

Every value in a row is a whole number.[^14] A category and a level are indices.
No column is a name and no column is a flag. Zero in a column means the row
does not change the thing the column names.[^4]

The world is built with the table, from one constant in the core crate, in the
form the unit type table takes.[^15] The table enters the whole-world hash byte
for byte, so two worlds built with different tables never hash the same.[^16]
The values of each row are balance values and live in the balance register,
never in this record.[^17]

A reviewer finds a violation when a work, a fit or an effect is written in a
match arm, or when a row gains a column that no pass reads.

### D2. A build order names a category, and the engine resolves the row

The build verb takes a category and a set of tiles. It takes no level. For each
tile the engine reads the ground kind and the resource the tile yields, reads
the entry that stands there, and looks up the row for the category at the next
level. The next level is one when the tile carries no upgrade of that category,
and one above the level that stands there otherwise.

**The engine refuses when no row fits.** No row for the category at the next
level means the category is at its top, or does not exist. A row whose ground
fit excludes the tile means the ground does not suit it. A tile that carries an
upgrade of another category is refused too, because a tile carries one
upgrade.[^1] Each refusal drops the tile from the set and counts in the
census, in the form the controller record fixes for a refused command.[^18]

The resolution costs one read of the ground, one search of the sparse entries
and one read of the table. It costs no pass over the world.

A reviewer finds a violation when a caller names a level, when a build order
starts a build that no row fits, or when the refusal is silent.

### D3. A level is raised in place, and an entry never has a sibling

The entry on a tile holds the category, the level that stands there and the
work done toward the next level. When the work done reaches the work of the
next row, the level rises by one and the work done returns to zero. The entry
stays one entry. No second entry is written for the higher level, and the
storage of an upgrade does not grow with its level.[^1] The entries stay in
tile order, so a raise never moves an entry and never depends on which unit
built first.[^19]

The work done is a whole number in a wide accumulator, and several units add
to it in one tick in any order.[^14] [^20] It is clamped at the work of the
next row, so a builder cannot bank surplus past a level.[^21] At the top level
there is no next row, so the clamp is zero and a builder there adds nothing.

**Repair comes before a raise.** The wear work gives an entry a condition, and
the build order on a worn entry raises the condition first.[^8] The work
reaches the next level only from full condition. One order therefore means one
thing at every state of the entry: make this tile better. The condition and
the work done are two fields, because they answer two questions, and each has
its own clamp.

Destroying an upgrade removes the entry, at whatever level it stood.[^22] The
tile returns to the ground the generator made.

A reviewer finds a violation when a tile holds two entries, when a level is
stored anywhere but the entry, or when a raise costs an allocation.

### D4. No pass branches on a category, and a pass reads a column

A pass that asks what an upgrade does reads the column of the row that the
entry indexes. The capacity reader reads the capacity column. The gather
resolve reads the yield column. A pass never compares a category index to a
constant, and never names a category.[^4]

The one function that composes the ground and the upgrade keeps its shape.[^23]
It reads the capacity column of the finished row instead of a match arm. Every
caller that enforced the capacity through it still does.

A reviewer finds a violation when a pass holds a branch on a category or on a
level.

### D5. The viewer reads the category and the level, and paints them apart

The viewer reads the category and the level of every entry it draws. Two
entries that differ in category are drawn so that a watcher can tell them
apart. Two entries that differ in level are drawn so that a watcher can tell
them apart. The viewer reads and never writes.[^24]

This is a constraint on the view and not on a palette. Which colour, which
glyph or which shape stands for which row is a choice of the drawing pass, and
a game may change it. What a game may not do is draw two rows the same.

A reviewer finds a violation when two rows share one drawing, or when the
drawing pass reads anything but the category and the level to choose it.

### D6. The road, the terrace, the wonder, the store and the wall are rows

Every upgrade the engine knows is a row of the table, and none is a variant.
The road and the terrace that exist today become rows. The wall that the wear
work adds, and the wonder and the store that the game end adds, are written as
rows and never as variants.[^8] [^9]

The sparse storage record does not state the enumeration.[^1] It says that an
entry holds a kind, that a tile carries one entry and that the clamp folds from
the catalogue. Every one of those statements holds under this record, with the
row in place of the kind. This record therefore supersedes nothing. The change
is to code alone, and the sparse storage record stays as it is.

## The alternatives this rejects

**An enumeration with one variant for each kind, as today.** Rejected because a
level multiplies the variants, because every effect is a match arm in code, and
because a game cannot add a kind without a release. The project already refused
this shape for the unit type, for the same reasons.[^3]

**A free-form kind that a game names at run time.** A caller would define a
kind by name, with its own work and effect. Rejected because a name is not a
key a pass can read, because an effect a caller states is content-supplied
behaviour and not a value, and because the hash would then cover a string. The
unit type rejected a name column for the first reason.[^4]

**One level only, with a second category for the better thing.** A game would
write a terrace and a great terrace as two categories. Rejected because the
order between them would live in the caller and not in the table, because the
build verb could not resolve the next step, and because a watcher could not
read a level that the table does not hold.

**A level stored as a second entry on the tile.** Rejected because a tile
carries one upgrade, and because destroying an upgrade would then have more
than one answer.[^1]

**A raise that needs a second verb.** Rejected because the product asks that
the order that builds also raises, and because a second verb is a second rule
set that the controller and the caller could apply differently.[^7] [^18]

## Consequences

**The golden state hash moves.** The table enters the hash, the entry gains a
level and the kind becomes a category index. Every stored golden hash changes,
and the commit that lands the table records the change.

**The wall, the wonder and the store are designed against rows.** The two items
that add them write rows and read columns, and neither adds a variant.[^8] [^9]
The item that adds the wall waits for the table.

**A category has a top.** The top is the highest level with a row, and nothing
states it twice. A build order at the top is refused, and the refusal is
counted.

**The controller builds by category.** The demonstration controller draws a
category from the high bits of its one draw, in the way it draws a kind
today.[^18] A category the ground does not suit is refused and counted, and the
controller learns nothing from it. A controller that chose by the ground is a
behaviour decision and this record does not make it.

**A column that no pass reads must not be written.** A cross cost column is a
candidate, because the product asks what a road does to a unit, and a movement
pass may one day read it. The column arrives with the pass that reads it.[^13]

**The Python type stub gains a category argument and loses a kind.** A caller
that passed a kind number passes a category index. The commit that lands the
table searches the tree for every caller and names the search.

**The decisions register closes its row.** The open choice between a variant
and a row is answered by this record, and the item that lands the table closes
the row in the same commit.[^5]

**Nothing here names a value.** Which categories exist, how many levels each
has, which ground each fits and what each level changes are rules of the
downstream game, and a blocker holds them.[^25] The engine holds a default
table for the demonstration and the balance register holds its rows.[^17]

## References

[^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decisions D1 to D4. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
[^2]: Project orientation, the design principles. `CLAUDE.md`
[^3]: ADR-0120, a unit carries a type, and the type is an index into a table the world is built with, decisions D1 and D3. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
[^4]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decisions D1 and D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
[^5]: Decisions register, DEC-143. `docs/DECISIONS.md`
[^6]: Backlog item 0348. `docs/backlog/proposed/0348-make-the-upgrade-catalogue-a-table-the-world-is-built-with.md`
[^7]: PRD-0055, a god raises the ground its people hold, and sees what stands there. `docs/product/shaped/prd-0055-a-god-raises-the-ground-its-people-hold-and-sees-what-stands-there.md`
[^8]: Backlog item 0475. `docs/backlog/proposed/0475-give-an-upgrade-a-condition-that-armies-wear-and-workers-repair.md`
[^9]: Backlog item 0479. `docs/backlog/proposed/0479-end-the-game-on-domination-wealth-wonder-or-renown.md`
[^10]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
[^11]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D1. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^12]: ADR-0088, a tile field is a generated base and a stored change, decision D1. `docs/adrs/draft/adr-0088-a-tile-field-is-a-generated-base-and-a-stored-change.md`
[^13]: Recurring Defect Shapes, shape 3. `.agents/rules/recurring-defects.md`
[^14]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^15]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D4. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
[^16]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^17]: Balance register, the upgrade rows. `docs/reference/balance.md`
[^18]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decisions D2 and D3. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^19]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^20]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^21]: Findings register, FND-011. `docs/FINDINGS.md`
[^22]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D4. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
[^23]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D3. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
[^24]: ADR-0067, the viewer reads the world and never writes to it, decision D1. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^25]: Blockers register, BLK-050. `docs/BLOCKERS.md`
