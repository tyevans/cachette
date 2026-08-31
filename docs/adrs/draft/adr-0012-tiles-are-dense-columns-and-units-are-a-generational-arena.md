# ADR-0012: Tiles are dense columns and units are a generational arena

Status: Draft

## Context

The world holds two populations, and they have opposite shapes.

A tile always exists. The world allocates every tile once, and the tile count
never changes while the simulation runs. A tile is never created and never
destroyed. Almost every tile field is present on almost every tile.

A unit is created and destroyed. The unit population changes every frame. A
unit is sparse against the tile count: most tiles carry no unit at all.

One storage mechanism can hold both. Several engines make a tile an entity and
give it components, which keeps one query language for the whole world.[^1]
The project must decide whether it does the same, because the answer reaches
every later storage decision and every zero-copy view.

This record states the split. The record that fixes the entity shapes assumes
the split and does not state it.[^2]

## Decision

### D1. A tile is not an entity

Tile data lives outside the entity storage. A tile has no identity, no
generation, and no row in the entity location table.

A tile is addressed by its tile index alone.[^3] A tile lookup derives its
storage position from the address by arithmetic, and never consults a table.
There is nothing to resolve, because there is no identity to check.

The derivation is arithmetic whichever order the tiles sit in. A block order
adds a shift and a mask; it adds no lookup.[^7]

### D2. Each tile field is its own dense column

A tile field is one contiguous array with one element for each tile. The
engine stores a structure of arrays, not an array of structures.

A system that reads one field reads one array. A system that reads no field
touches no memory for that field. A new field adds a column and changes no
existing column.

This record fixes that a tile field is a column. It does not fix the width of
a column, the encoding of a boolean field, or the form a rare field takes. A
separate record holds those.[^4]

### D3. A unit is an entity in the generational arena

A unit lives in the entity storage. It carries an identity that pairs a slot
index with a generation.[^5] Its columns are the columns of its shape.[^2]

The arena holds every one of the four fixed shapes. Tile columns hold none of
them. A tile upgrade is an entity in the arena, and the tile side of the split
holds only the marker that says which tiles carry one.[^4]

### The alternative this rejects

**Make every tile an entity.** One query language would then cover the whole
world, and a tile field would be a component like any other. Python would
select a tile and a unit through one mechanism.

The project rejects it. A tile needs no identity, because its address is
stable and total. Giving a tile an identity buys a generation and a location
entry for a thing that is never created and never destroyed. The location
table would grow with the tile count, which is the largest count in the
project. The saving is a language, and the cost is the largest table in the
engine.

The reverse alternative is also rejected. **Storing a unit in a dense column
indexed by tile** would give a unit an address instead of an identity. A unit
moves, so its address would change every time it moves, and every reference to
it would have to move with it.

## Consequences

**Tile data is the zero-copy path, and unit data is not.** A tile field is one
flat array, so a caller can view it without a copy. The entity arena is
chunked for each shape, so a caller reads one view for each shape or takes a
copy.[^2]

**A tile field cannot be optional.** Every column carries an element for every
tile, so a field that few tiles need still costs a full column. A rare field
therefore needs a sparse form, and choosing that form is a design step for
each new field.[^4]

**The engine holds two storage mechanisms and two vocabularies.** A system
that joins a tile to a unit must cross the split. The bridge that crosses it
is a separate structure with its own record.[^6]

**A whole-world tile operation has a predictable cost.** The work is a linear
pass over one array, with no indirection and no resolution. This is the
property the split exists to preserve.

**The split cannot move later without a supersession.** Every system reads a
tile through an index and a unit through an identity. Merging them afterwards
would rewrite every system.

## References
[^1]: Report 01, the entity component system core and the memory layout, sections 1 and 8. `docs/research/reports/01-ecs-and-memory-layout.md`
[^2]: ADR-0066, entity storage holds four fixed shapes. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^3]: ADR-0017, the world is a rhombus, so a tile index is raw axial. `docs/adrs/draft/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
[^4]: ADR-0015, a tile column is narrow, with bitplanes and sparse side tables. `docs/adrs/REGISTRY.md`
[^5]: ADR-0014, entity identity is an index plus a generation. `docs/adrs/draft/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^6]: ADR-0018, the unit-to-tile bridge is three structures, and units stay sorted by tile. `docs/adrs/draft/adr-0018-the-unit-to-tile-bridge-is-three-structures-and-units-stay-sorted-by-tile.md`
[^7]: ADR-0016, tiles are stored in block-tiled order at the aggregation block size. `docs/adrs/REGISTRY.md`
