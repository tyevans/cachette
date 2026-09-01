# Blockers (Register)

This document is a **register**. It lists work that is stopped and names what
must happen to start it again.

A blocker needs **information** the project does not have. Compare
`DECISIONS.md`, which lists choices that need judgement. If work can continue
under a stated assumption, it is a decision, not a blocker.

Numbers are permanent. Never reuse one. A resolved blocker keeps its row.

| Field | Meaning |
|---|---|
| Blocks | What cannot proceed |
| Owner | Who can resolve it |
| Status | `Open`, `Resolved`, or `Dropped` |


## Allocating a number

**Claim the next number below before you write the row.** Increment it in the
same change that adds the row.

A writer that numbers a row by reading the last row collides with any other
writer working at the same time. That happened, and it is recorded as
precedent.[^ALLOC]

**Next number: BLK-019**

[^ALLOC]: Findings register, FND-038. `docs/FINDINGS.md`

## Open

### BLK-007 — No measurement exists on the target platform

**Owner:** engineering. **Blocks:** confidence in every cost figure in every
report.

Every number in the research is derived, not measured. The research agenda
names benchmarking on Graviton as blocking most of its own conclusions.

This blocker cannot be resolved by a decision. It needs a benchmark harness
and a machine.

## Resolved

### BLK-018 — How many groups found a world, and does every faction found one?

**Resolved.** Every faction founds one group. A run begins with one founding
for each faction the world holds.

The product record named one group and one group for each faction as the two
candidates and declined to choose between them.[^FOUND] The owner chose the
second. The two produce different games: one founding gives a run with one
society on an empty map, and one for each faction gives a run in which the
factions meet, on a tick that follows from how far apart the engine placed
them.

**A third shape was considered and deferred.** One faction that fractures into
several as the run proceeds. It was set aside because a fracture needs a rule
for why a society splits, and no record holds one. It is a later question, and
it does not block the founding rule: a run that founds one group for each
faction can still gain a fracture rule afterwards.

The engine already takes the group size and the faction at the founding call,
so a caller founds one group or several without an engine change. The
demonstration founds one and must now found one for each faction.

**Two questions this answer does not settle, and the work needs both.** How
far apart two foundings must be, and whether a second founding may widen its
sample when it fails. The founding record refuses a sample that widens until
it succeeds, so the second question is a real constraint and not a
detail.[^SEP] Both are judgement rather than missing information, so they are
decisions and not blockers.

### BLK-013 — Maximum faction count

**Resolved.** The ceiling is 63. A faction is one bit in a 64-bit mask, and
one value is reserved for no faction. The transposed level 0 grid is
therefore affordable, a relation is one plane, and a presence set is one
word. The value is in the scale constants table.[^SCALE]

### BLK-014 — The world shape

**Resolved.** The world is a rhombus. A tile index is a raw axial pair, so no
tile access converts a coordinate. The cost falls on the viewer: a rhombus is
a parallelogram on the screen, so the viewer applies the skew and the engine
does not. The registry row for the tile index was written for an offset
index and now states the rhombus claim.[^SHAPE] The finding records the
correction.[^TILEIDX]

### BLK-001 — Tile scale, and therefore world extent

**Resolved.** The tile edge is 80 metres. The world is regional, about 330 km
across. Dwell is 2 and crossing-terrain capacity is 16, which stays inside
`u8`. The 12.5-second ordinary crossing holds. The project gives up the
continental extent, not the crossing time, so no per-tick rate re-bakes. The
parametric movement constants resolve against these values, and the values are
in the scale constants table.[^SCALE]

### BLK-002 — Name three archetypes you expect to exist

**Resolved.** Four fixed shapes exist, so entity storage keeps the archetype
machinery. The shapes are the soldier, the settlement, the living character,
and the tile upgrade. A record holds the claim and its consequences.[^SHAPES]

### BLK-003 — Is one million the whole population, or one million soldiers?

**Resolved.** One million is the whole population. Soldiers are a fraction of
it, and civilians are not separate entities on top of the million. Every
storage figure in the needs report and the agency report holds as written.

### BLK-004 — Target living character population

**Resolved.** The target is 50,000 living characters, inside the range the
character report recommends. The hard ceiling of 262,144 stays. The layer cost
is in the scale constants table, and it is derived by scaling, not
measured.[^SCALE]

### BLK-005 — Settlement count

**Resolved.** The world holds 5,000 settlements. This confirms the assumption
in the entity economy report, so every storage figure in that report holds.

### BLK-006 — Tile upgrade fraction

**Resolved.** Fewer than one tile in twenty carries an upgrade, which agrees
with the entity economy report estimate. Tile upgrades therefore use sparse
storage, not one slot for each tile. A read pays one indirection.

### BLK-012 — What does one tick represent in simulated time?

**Resolved by derivation, once BLK-001 answered what a tile represents.**

The tile edge is 80 metres, so a march rate of 24 km in a simulated day
crosses 300 tiles. Each tile costs a dwell of 2 ticks, so a simulated day is
600 ticks. One tick is therefore 2.4 simulated minutes.

The engine runs at 10 ticks for each second, so a simulated day passes in one
minute of real time.[^TIMING] A content author who writes a per-tick rate now
has the figure they need. The derived values are in the scale constants
table.[^SCALE]

The derivation assumes the march rate applies to ordinary ground at dwell 2.
It is arithmetic on constants the owner approved, not a new decision.

### BLK-008 — Upkeep per unit or per formation

**Resolved.** A unit is an individual soldier. The three-tier split makes it
affordable: individual decay, pooled consumption, aggregate decisions.

### BLK-009 — Tile capacity

**Resolved.** Eight units, stored as `u8`, with capacity as a data-driven
parameter. Crossing terrain raises it to 16.

### BLK-010 — Do formations exist as entities

**Resolved.** Formation membership is an ownership column plus a reverse
index. A formation is not a spatial region: a region is not stable under
movement, so a move order would change its own recipient set across frames.

### BLK-011 — Promoted soldier lineage

**Resolved.** A promoted soldier gets no invented ancestry. He founds a new
house, his kinship to everyone is zero, and he cannot inherit a title by
blood. A title holder may **appoint** him. His children inherit from him
normally.

[^SCALE]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^SHAPES]: ADR-0066, entity storage holds four fixed shapes. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^SHAPE]: ADR Registry, row 0017. `docs/adrs/REGISTRY.md`
[^TILEIDX]: Findings register, FND-042. `docs/FINDINGS.md`
[^TIMING]: Movement timing note. `docs/research/movement-timing.md`
[^FOUND]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
[^SEP]: Decisions register, DEC-037. `docs/DECISIONS.md`
