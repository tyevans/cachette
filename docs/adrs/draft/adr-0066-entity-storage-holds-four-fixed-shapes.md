# ADR-0066: Entity storage holds four fixed shapes

Status: Draft

## Context

An archetype is the exact set of component types that an entity carries. It is
not a category of unit. A unit type is an index into a shared table and an
upgrade set is a bitmask, so neither of those varies the component set.

Entity storage can hold one shape or several. The choice is structural. One
shape means a generational struct-of-arrays arena: parallel columns and a
generational free list, with no archetype graph, no table move, and no chunk.
Several shapes mean one column set for each shape, an archetype graph, and
chunking, because a chunk is where per-chunk metadata hangs.

The project could not make this choice from the code, because the answer
depends on what the world contains. A blocker held it until the project owner
named the shapes.[^1]

## Decision

**Entity storage holds four fixed shapes, and each shape gets its own set of
columns.**

The shapes are the soldier, the settlement, the living character, and the tile
upgrade.

- A soldier is mobile, carries needs, and belongs to a formation.
- A settlement is fixed to a tile and holds pooled stores.
- A living character carries opinion and kinship, and carries no tile
  position.
- A tile upgrade is sparse and attaches to a tile, not to a mobile entity.

A structural change is a move between column sets, so the batched tombstone
and compact path applies to it.[^2] Chunking applies with it, and the chunk
size is a compile-time constant that the project measures on the target
platform.[^3]

**The shapes do not vary at run time.** The engine never builds an archetype
from a component set it discovers while running. A shape that is not one of
the four is a compile-time error, not a run-time table.

### The alternative this rejects

A single arena holding one shape is smaller, needs no archetype graph, and
keeps a whole-column zero-copy view for the Python boundary. The project
rejects it because one shape cannot hold four kinds of entity without giving
every entity the union of all four component sets. A living character would
then carry a tile position and a needs vector that it never reads, and the
character layer is the tier with the least room for waste.

## Consequences

**A zero-copy view no longer spans the whole population.** A chunked layout
has no flat per-component array, so a caller reads one view for each shape, or
takes a copy. The project recorded this cost before it knew the shapes, and
the boundary record states what copies at each call site.[^4] [^5]

**Tile data is unaffected.** Tiles are dense columns and are not in the entity
arena, so a tile field stays one contiguous array. Tile data is therefore the
honest demonstration of the zero-copy path.

**The project maintains an archetype graph it would otherwise not write.** The
graph has no upstream and no external test suite, and every structural change
path goes through it.

**A fifth shape is a decision, not a configuration.** Adding one means a new
column set and a new node in the graph. That cost is deliberate: it keeps the
run-time archetype out of the design.

## References

[^1]: Blockers register, BLK-002. `docs/BLOCKERS.md`
[^2]: ADR-0020, structural change batches at the barrier and applies by tombstone and compact. `docs/adrs/REGISTRY.md`
[^3]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^4]: Findings register, FND-003. `docs/FINDINGS.md`
[^5]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
