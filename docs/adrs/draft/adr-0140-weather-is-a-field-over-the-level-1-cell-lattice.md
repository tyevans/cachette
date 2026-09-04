# ADR-0140: Weather is a field over the level 1 cell lattice

## Context

A product record asks the world to hold a condition that varies over the map
and over time, without any unit acting on it.[^1] The world holds terrain, and
terrain does not move. Every condition a unit meets is therefore fixed when the
generator runs.

The engine simulates a hex world at two levels of detail. Level 0 holds
individual tiles and units, and it is the only truth. Level 1 summarises a
square block of tiles at city scale, and it is derived at the barrier of every
step.[^2] The block edge is a power of two, so one level 1 cell covers a block
of tiles rather than a handful of them.

**The engine has just decided the opposite question for a fight.** A meeting
between two factions resolves at the tile and never at a level 1 cell, because
a cell covers a whole block and a fight resolved there kills units spread over
all of them.[^3] A measurement at level 1 granularity found the smear
directly.[^4]

That answer does not carry over, and this record says why rather than copying
it. Three forces pull the other way.

**A storm is larger than a block.** A fight is an event between two units
standing on one tile, so a cell is coarser than the thing being resolved. A
weather field varies slowly over distance, so a cell samples it rather than
smearing distinct events. Two units in one cell genuinely stand in the same
weather.

**A field at tile pitch costs the whole world on every frame.** The product
record rejects that shape by name: what the update costs must grow with the
area the condition occupies, not with the size of the world.[^1] The lattice is
smaller than the world by the square of the block edge.

**The project already solves a field at this pitch.** The influence field is a
plane over the level 1 cell lattice, it relaxes against its neighbours, and it
carries what the last solve left.[^5] A second machine for the same shape would
be a second way to do one thing.

## Decision

### D1. The weather field is a plane over the level 1 cell lattice

**A cell of the lattice holds the weather, and a tile holds none.** A reader
that asks about a tile is answered from the cell that covers it, so two tiles
of one cell answer the same.

The field is not a summary. A cell of it holds what the last solve left there,
and that value appears nowhere at level 0. The field is simulated state and it
enters the state hash, in the way the influence field does.[^5] [^6]

The alternative is a plane over the tiles. It is rejected on the cost shape the
product record states, and on the fact that nothing in the engine would read
weather at tile resolution.[^1]

### D2. The field allocates nothing until water enters the world

**A world in which nothing has happened stores no weather.** The planes are
allocated by the first thing that puts water into the air, and a solve over an
empty field does its source pass and stops.

This is the storage half of the same cost shape. It is weaker than the product
record asks for, because one drop of water anywhere allocates the whole
lattice. The consequences say so plainly.

### D3. The solve runs after level 1 rebuilds, and a reader takes what the
previous frame left

**The solve is the last field stage of the step, after the derived level it
reads was rebuilt.** It reads the height and the water share of each cell from
the summaries that the rebuild produced. A solve placed earlier would answer
from a level 1 that the frame had not yet rebuilt.[^2]

A simulation pass that reads the weather therefore reads the ground as the
previous solve left it. That is the same relation that movement has to the exit
field, and it is what keeps the read out of the write.

## Consequences

The engine cannot express weather that differs between two tiles of one cell. A
game that wants a shower over one tile cannot have one. That is the price of
the cost shape, and it is the mirror of the price the fight record refused to
pay.

The storage claim is weaker than the product record asks for. The record asks
that storage grow with the area the condition occupies, and this grows with the
lattice as soon as any water exists. The lattice is smaller than the world by
the square of the block edge, so the shape is right and the bound is not tight.
A sparse field over an active set would be tight, and nothing has priced
either.[^7]

The step gains one stage. It costs the lattice on every frame, whatever the
weather is doing, and the stage cost table can price it.[^8] No measurement
exists, and one blocker governs every cost figure in this project.[^7]

A world with no water and no god never allocates the field and never spreads
anything. The cost of a calm inland world is one keyed draw for each cell.

## References

[^1]: PRD-0004, the world has weather that a watcher can read, what it costs at the target scale. `docs/product/accepted/prd-0004-the-world-has-weather-that-a-watcher-can-read.md`
[^2]: ADR-0022, level 0 is the only truth, and every level above it is derived, decisions D1 and D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^3]: ADR-0121, a meeting between two factions resolves at the tile, decisions D1 and D2. `docs/adrs/draft/adr-0121-a-meeting-between-two-factions-resolves-at-the-tile.md`
[^4]: Findings register, FND-402. `docs/FINDINGS.md`
[^5]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
[^6]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^7]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^8]: The stage cost table. `crates/cachette-core/src/stage.rs`
