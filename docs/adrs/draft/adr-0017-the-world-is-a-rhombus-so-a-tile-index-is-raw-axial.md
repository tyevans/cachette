# ADR-0017: The world is a rhombus, so a tile index is raw axial

Status: Draft

## Context

Every system in this engine reaches a tile through an index. A tile index is
therefore the one address that every later storage decision inherits.

A hex grid has several coordinate systems. Axial coordinates name a tile by
two numbers and support vector arithmetic: the six neighbours are six fixed
offsets, and adding two axial coordinates gives a third. Offset coordinates
name a tile by a column and a row. They tile a rectangle with no wasted cell,
and they do not support vector arithmetic, because the neighbour offsets
change from one row to the next.[^1]

The two systems disagree about the world shape, and that is the real
question. A raw axial array is a parallelogram in world space. Storing a
rectangular world in one means allocating a bounding parallelogram, and the
shear wastes about half of it. A project that wants a rectangular world
therefore stores an offset index and converts on every tile access.[^1]

The project owner fixed the shape.[^2] This record states what follows from
it, so that a later contributor who reaches for an offset conversion can see
that the choice was made and why.

## Decision

### D1. The world is a rhombus, and a tile index is a raw axial pair

A tile address is an axial pair. Deriving the storage index from the address
is arithmetic on the two components, and nothing else.

**No tile access converts a coordinate.** A conversion function between an
axial address and an offset address does not exist in the engine. Its
presence in a tile access path is the violation this record lets a reviewer
find.

This record does not state the index function. The order in which tiles sit
in memory is a separate claim, and the record that holds it may choose a
block order rather than a row order.[^4] Both derive an index from the same
axial address by arithmetic, which is what this record constrains.

### D2. The neighbours are six fixed offsets, and the edge does not wrap

The six neighbours of a tile are the tile address plus six constant axial
offsets. The offsets are the same for every tile, which is the property that
an offset index does not have.

A neighbour outside the world is absent. The world does not wrap. A wrapping
world would make the tile index a ring rather than a range, and every
distance would need a shortest-arc rule.

### D3. Every coordinate is an exact integer

An axial component, an index, and a distance are all integers. No coordinate
is a fixed-point value and none is a floating point value, so the arithmetic
that derives one is exact.[^3]

### D4. The engine stores the shape, and the viewer draws it

A rhombus in the index space is a parallelogram on the screen. The viewer
applies the skew when it maps a tile to a screen position.

The engine never holds a screen position and never applies the skew. A
viewer is one consumer of the world among several, and a projection that
suits one display does not suit another.

## Consequences

**A block is a rhombus, so query pruning is looser.** The aggregation block
inherits the index space, so a block is a parallelogram in world space rather
than a near-rectangle. A conservative bounding radius around a longer, thinner
block admits more false positives, so a radius query descends into more
subtrees than a rectangular block would need. The report measures the aspect
ratios of both.[^1] This is the price of removing the conversion, and it is
paid by the query path rather than by every tile access.

**A rectangular viewport is not a rectangular index range.** A viewer that
shows a rectangle of the world reads a sheared range of the index space. The
viewer computes that range. The engine does not.

**The world cannot become rectangular later without a supersession.** The
index shape reaches every storage decision above it, so this is the kind of
choice that costs more to change than to make. That is why it is recorded.

**A tile access loses a conversion, and the engine gains a shape it must
explain.** A person who looks at the world sees a parallelogram and asks why.
The answer is this record.

## References

[^1]: Report 02, the hex grid and the level of detail pyramid, sections 1.2 and 3.4. `docs/research/reports/02-hex-grid-and-lod-pyramid.md`
[^2]: Blockers register, BLK-014. `docs/BLOCKERS.md`
[^3]: ADR-0002, simulated and aggregated state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^4]: ADR-0016, tiles are stored in block-tiled order at the aggregation block size. `docs/adrs/REGISTRY.md`
