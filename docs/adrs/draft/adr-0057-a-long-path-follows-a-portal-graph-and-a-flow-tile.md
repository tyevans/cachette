# ADR-0057: A long path follows a portal graph and a flow tile, never a per-unit search

Status: Draft

## Context

Units cross the world in large numbers. A direct answer gives each unit its
own path search. That answer does not survive the target scale, because the
cost grows with the number of units and with the length of the path at the
same time.

A second direct answer builds one cost field over the whole world for each
destination. That field is as large as the world, and the engine would
rebuild it whenever the world changes.

The project already holds a rule that governs this choice. A set-valued
command permits a cheaper algorithm, so the engine must choose an algorithm
that uses the whole set rather than a loop over one unit at a time.

Movement is tile-discrete, and a unit chooses one adjacent tile in each
step.[^1] A unit therefore needs a local cost gradient, not a full path.

## Decision

### D1. A long path is a coarse route plus a local field

The engine splits the problem in two.

A portal graph spans the world. A node is a portal, which is a passable
edge between two neighbouring chunks. An edge joins two portals of the same
chunk. A search over this graph gives the route as a sequence of portals.
The graph is small, because a chunk holds many tiles and few portals.

A flow tile holds the local field. It covers one chunk. It stores the cost
of each tile in the chunk against one exit portal, and it stores the
preferred direction of each tile. A unit reads the flow tile of the chunk
it stands in.

### D2. The engine never runs a path search for each unit over a long path

A per-unit search over the whole world is forbidden. Many units that share
a route share one portal route and one set of flow tiles.

A search inside one chunk is not a long path, and this record does not
forbid it.

### D3. The flow tile cache is keyed on the chunk and the exit portal

The cache key is the pair of the chunk identifier and the exit portal
identifier. The key does not name the command that asked for the field, and
it does not name the destination.

Two commands with different destinations that leave a chunk by the same
portal share one flow tile. This is the whole value of the key.

### D4. The flow tile stores the cost field, not only the direction

A unit scores each of the six neighbours from the cost field and from the
current occupancy of the neighbour. A direction byte alone cannot answer
that score, because a congested unit must be able to prefer a neighbour
that the direction byte does not name.

The engine applies the occupancy term at the moment of choice. It never
applies it inside the flow tile build. The flow tile therefore stays
independent of occupancy, and the cache survives a tick in which units
move.

### D5. A coarse field biases the route, it never replaces the portal graph

A field at the summary level may adjust the edge costs of the portal graph.
It does not answer connectivity. A coarse cell hides a one-tile gap and a
one-tile wall, so a coarse field answers a route across them
wrongly.[^2]

## Consequences

**A unit cannot hold a private route.** A route belongs to a group of units
that share it. A behaviour that needs one unit to take a different path
must express that as a different exit portal, not as a private search.

**The cost of movement follows the number of live routes, not the number of
units.** Adding units to an existing route is close to free. Adding a new
destination is not.

**A stale flow tile is possible.** The world changes, and a cached field
outlives the change. The engine must invalidate a flow tile when the chunk
that it covers changes.

**The route quality is not optimal.** A hierarchical route is longer than
the true shortest path. The project accepts that error.[^3]

**The cache is a sizing risk.** The hit rate depends on how many distinct
routes are live at once. Nothing in the project answers that number
yet.[^2]

## References

[^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit. `docs/adrs/draft/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^2]: Report 10, crowd simulation and unit movement. `docs/research/reports/10-crowd-and-movement.md`
[^3]: Botea, Muller and Schaeffer, Near optimal hierarchical path-finding, Journal of Game Development, 2004.
