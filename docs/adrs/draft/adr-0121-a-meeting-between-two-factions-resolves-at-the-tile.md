# ADR-0121: A meeting between two factions resolves at the tile, never at a level 1 cell

## Context

Two factions can stand on one place in this engine and nothing happens. No
pass contests the meeting, and a downstream game names attacking as one of the
things its players must do.[^1]

A design sketch for the contest resolved the fight for each level 1 cell, as a
small table over unit types. A level 1 cell summarises a block of tiles, and
the block edge is a power of two that one constant sets.[^2] A fight resolved
for a whole cell therefore ends units spread over every tile of that block.

**The risk is that an army smears rather than forming a front line.** A player
who watches a battle expects the casualties to lie where the two sides touch.
A cell-level resolution has no way to express that, because a cell holds no
position inside itself.

The engine already splits this way for movement, and the split works. The exit
field gives one direction to a whole cell, and admission enforces the tile
capacity one tile at a time.[^3] The derived unit structure lists the units
standing on one tile, and it rebuilds at every barrier, so the input a tile
resolution needs already exists and costs the frame nothing more.[^4]

A blocker held the measurement that settles the granularity by evidence rather
than by argument.[^5] **The project owner decided the granularity ahead of it,
and the measurement then agreed with him.** It reports the width of the band
of tiles that holds the casualties, for each granularity, and the register
holds its outcome.[^5]

## Decision

**A meeting between two factions resolves at the tile. A field over cells
decides only where an army goes.**

### D1. Contact is adjacency, and the resolution writes to the units of one tile

A unit reaches every unit of another faction that stands on its own tile, and
every such unit on the six tiles beside it.

**Co-occupation alone is the wrong rule, and it would make the pass fire
almost never.** Admission refuses a step onto a tile that stands at its
capacity, and it reads the capacity rather than the faction of the units
standing there.[^3] An army that filled a tile could therefore never be
entered, and a rule that needed two factions on one tile would never fire
against exactly the case a fight is about. A measurement found that, and the
register holds it.[^13]

**The alternative was rejected.** Admission could gain a rule that a unit of
another faction may enter a full tile. That supersedes an accepted record, and
it makes the capacity mean nothing at the moment it matters, because the tile
would then hold a crowd that no rule bounds.[^3] The capacity exists to bound
that crowd.

The pass walks the derived unit structure. The units of one tile lie in one
contiguous run of it, so the pass reads a tile without searching for it.[^4]
For each tile it also reads the six neighbours of that tile, in the fixed
order the hexagonal geometry declares. **That is a pass over tile pairs and not
a search from a unit**, so it does not reach the mechanism the movement record
forbids.[^11] The neighbour count is six, and it is a property of the
geometry.

The defenders of one resolution are the units of one tile. Every unit is a
defender exactly once, in the pass over the tile it stands on, so nothing is
resolved twice and no unit is counted against itself.

A tile is contested when some unit within reach belongs to a faction that some
unit on the tile does not, and when at least one of the two factions is in the
war band toward the other. The relation between two factions is one signed
integer, and a later record states the band and the gate.[^15] The pass tests
both conditions from the group lists it has already built, and skips every tile
that fails either. The cost of the contest
therefore follows the occupied tiles and never the tile count of the world.

### D2. The pass is a structural change, and it applies after its own barrier

The resolution removes units. A removal is a structural change, so the pass
marks the units that fell and ends them afterwards, in one ascending scan of
the slots. It never ends a unit inside the parallel walk.[^6]

Each thread marks into its own plane, and the planes join by a bitwise union.
A union is commutative and associative, so the joined plane is the same at any
thread count.[^7] The threads do not own disjoint output ranges here, because
two tiles of two threads can hold units whose slots share one word.

The derived unit structure names a dead identity after the pass, so the step
rebuilds it before anything else reads it.[^8]

### D3. The resolution reads a table for each ordered pair of unit types

The pass counts the units of each faction and each type on the tile, and then
reads the type table once for each ordered pair of those counts. **It never
loops over pairs of units.**

The cost of one tile therefore follows the square of the type count, which is
small and fixed, and the count of groups on a tile, which the tile capacity
bounds.[^9] A pairwise fight would follow the square of the population of the
tile, and it would give a game no way to add a unit kind without adding a
rule.

The consequence is that the resolution holds no arrangement. A table over
counts cannot express that one unit stands behind another, so a heavy unit
does not shield the units behind it. That is the price of not running a fight
for each pair, and this record accepts it.

### D4. The pass writes an event for each unit that fell

A fight that nobody can read is a fight that nobody can repair. The pass
appends one event for each unit it ended, and the event names the tick, the
unit, the tile, the faction and the type.

The event is plain data with a declared layout and declared padding, in the
way every event type of this project is.[^10] The log holds the units of one
frame, in ascending slot order, and the pass clears it at its start.

## Consequences

**The engine cannot resolve a fight more coarsely than a tile.** A game that
wants a battle over an area sends units to the tiles of that area and the
engine resolves each one.

**A fight cannot be finer than a tile either.** Two units on one tile meet;
the engine holds no position inside a tile and this record adds none.

**The cost of the contest follows the occupied tiles.** A world that is not at
war pays one walk over its units, six neighbour reads for each occupied tile,
and no table work at all. A resolution that fired only on co-occupation would
pay less and would never fire.

**A unit cannot stand out of reach of an enemy that is beside it.** There is no
posture that refuses a fight, and this record adds none. The relation between
the two factions is the only thing that refuses one.[^15] A decision holds the
question of a posture.[^14]

**Level 1 keeps the job it is good at.** It decides where an army goes. This
record takes nothing from the field that steers movement.[^11]

**The measurement that would have refuted this record exists, and it agrees
with it.** The register states the band width at each granularity and the
method that produced it.[^5]

## References

[^1]: Research report 21, what a god needs from this engine, section 1.5. `docs/research/reports/21-what-a-god-needs.md`
[^2]: Research report 21, what a god needs from this engine, section 4.2. `docs/research/reports/21-what-a-god-needs.md`
[^3]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^4]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D2. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^5]: Blockers register, BLK-052. `docs/BLOCKERS.md`
[^6]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^7]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^8]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^9]: ADR-0120, a unit carries a type, and the type is an index into a table the world is built with, decision D2. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
[^10]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
[^11]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^13]: Findings register, FND-402. `docs/FINDINGS.md`
[^14]: Decisions register, DEC-146. `docs/DECISIONS.md`
[^15]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D4. `docs/adrs/draft/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
