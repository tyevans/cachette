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

**Next number: BLK-035**

[^ALLOC]: Findings register, FND-038. `docs/FINDINGS.md`

## Open

### BLK-007 — Most cost figures are still derived on the target platform

**Owner:** engineering. **Blocks:** confidence in most cost figures in most
reports.

**This row narrowed on 3 September 2026. It did not close.** A benchmark now
lives in the repository, and a script runs it on a Graviton instance and
destroys the instance afterwards. Every axis of the sweep is a parameter. Two
runs measured five quantities, on two instances of different size. A register
holds every row, both machines and both commits.[^MEASURED]

**What is now measured.** The cost of one frame against the tile count and
against the unit count, up to 16,777,216 tiles and 1,000,000 units, at 1, 2,
4, 12 and 16 threads. The cost of building a world. The cost of the hash of
the whole world. The resident memory of a world. Whether one frame at the
target scale trips an integer overflow check, which it does not. Every figure
was taken on a world that holds no settlement and no character.

**What keeps this row open.**

1. **The figures in the research reports.** A cache hit rate, an allocation
   count for each frame, and the cost of a call across the language boundary
   are all derived, and the benchmark measures none of them.
2. **Any figure for a world that holds settlements or characters.** The
   measured world holds neither, so the rate pass, the consumption pass and
   the position pass did no work and the character arena was empty. Every
   measured frame figure and the memory figure are lower bounds.
3. **Any figure for a stage inside a step.** The benchmark reaches the public
   interface, and the passes inside a step do not.

**The machine size no longer holds this row open, and the reason is a
result.** A run on 16 hardware threads gives a frame of 500 milliseconds at
the target scale against a budget of 100, and the unit passes stop gaining
above 12 threads. A larger machine is not the missing measurement.[^FLOOR]

**Do not read a document that says no measurement exists as current.** That
sentence was true when it was written, and about ninety documents hold it.
This row is the current statement. A finding records how far the sentence
spread and why it was not swept.[^SPREAD]

### BLK-034 — May a unit build on ground another faction holds, and who may destroy an upgrade?

**Owner:** the project owner. **Blocks:** any rule that ties an upgrade to a
faction, and the verb that removes one.

The engine now stores an upgrade on a tile and lets a unit build it.[^BLK34A]
It asks nothing about who holds the tile. Any live unit builds on any tile it
stands on, and the control plane removes an upgrade from any tile by address.

**This row narrowed on 3 September 2026. It did not close.** The project owner
answered the first question and the first half of the second.

**A unit builds only on ground that its own faction holds.** Building is
restricted by the holder of the tile. The holder column already names who
holds each tile, so the rule reads a value the engine stores.[^BLK34B]

**Anyone may destroy an upgrade.** Destruction is not restricted by the
holder. The owner stated that the reasons to destroy one vary, so the engine
permits the act and does not encode a motive.

Two questions stay open.

1. **Does destruction take work, or is it instant?** It is instant today, and
   it is a control-plane call rather than something a unit does. The answer
   decides whether a unit needs a verb for it.
2. **Does an upgrade change hands when the ground does?** The holder of a tile
   moves as units move, and the upgrade does not move with it.

Work continues without the two answers. The engine states the storage and the
arithmetic, and neither depends on a faction. A rule invented here would be a
content decision made by the wrong person, and it would reach the state hash.


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

The engine takes the group size and the faction at the founding call, so a
caller founds one group or several. The run-level call founds one group for
each faction the world holds, and the demonstration calls it.[^FOUNDDIST]

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

[^MEASURED]: Target platform costs. `docs/reference/graviton-costs.md`
[^SPREAD]: Findings register, FND-223. `docs/FINDINGS.md`
[^FLOOR]: Findings register, FND-224. `docs/FINDINGS.md`
[^SCALE]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^SHAPES]: ADR-0066, entity storage holds four fixed shapes. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^SHAPE]: ADR Registry, row 0017. `docs/adrs/REGISTRY.md`
[^TILEIDX]: Findings register, FND-042. `docs/FINDINGS.md`
[^TIMING]: Movement timing note. `docs/research/movement-timing.md`
[^FOUND]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
[^SEP]: Decisions register, DEC-037. `docs/DECISIONS.md`
[^FOUNDDIST]: ADR-0076, a founding keeps a fixed distance from the foundings before it. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
[^BLK34A]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
[^BLK34B]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
