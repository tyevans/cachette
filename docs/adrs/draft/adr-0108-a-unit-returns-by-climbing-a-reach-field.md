# ADR-0108: A unit returns by climbing a reach field seeded at every site of its faction

## Context

A unit in this engine takes one step each frame. The direction comes from a
per-cell field: the engine ranks the six neighbours of every level 1 cell
once, for each option, and a unit reads one entry.[^1] A level 1 cell
summarises one block of tiles.[^2] The engine ranks a neighbour on a value
that the cell carries, and that value is a summary field.[^3]

**A site is not a summary field.** A cell carries how much of its ground is
open, how high it is on average, how much food it holds and how crowded it is.
It does not carry where one unit belongs. So "go to my home site" cannot be
another row of the exit field.

That gap has a cost. A pass moves a carried load into the store of the unit's
home site, and it fires only while the unit stands on the tile of that site.
Nothing steers a unit there, so the delivery is built and almost
unreachable.[^4]

A separate record already decides the shape of the answer. **A strategy that
names a place takes its direction from a field over cells and never from a
search that starts at a unit**, and a quantity computed from a unit's own
position toward a unit's own destination is such a search, whatever it is
called.[^5] That record also states that several destinations seed one field,
so one derivation serves a set of units with a set of destinations.[^6]

**What that record leaves open is what the field holds.** It says a strategy
is seeded at its destination and spread outward. It does not say what spreads,
how far, or what stops it. The register holds the question of whether such a
field carries between frames, and it recommends deriving from nothing and
seeding densely.[^7]

This record answers those questions for the first strategy the project needs.

### Two fields already solve over this lattice

The influence field holds one plane for each faction over the level 1 cells,
and a solve relaxes it a fixed number of passes each frame.[^8] [^9] It is the
obvious thing to climb, and nothing sets a source in it today.

The exit field holds one direction for each cell and each option, and it is
derived again at every rebuild of level 1.[^10]

A reader must know which of the two a return direction resembles, because the
answer decides whether the direction states a fact of its own.

## Decision

### D1. The return direction is a field over cells, with one plane for each faction

The engine derives one direction for each faction and each level 1 cell. A
unit that holds the option which carries a load home reads the entry of its
own cell and its own faction, and it steps in that direction.

**No unit reads a neighbouring cell. No unit scores a neighbour. No unit
computes a bearing from its own address toward its own site.** The direction
belongs to the cell and to the faction, and every unit of that cell and that
faction reads one answer.

### D2. The field ranks a reach, and the reach is the number of cells to the nearest seed

Every live site seeds the plane of its faction. A seed cell has a reach of
zero. Each relaxation pass gives a cell one more than the smallest reach of
its neighbours, when that is smaller than the reach the cell already holds.

The direction of a cell is the first neighbour, in ascending direction index,
whose reach is strictly smaller than the reach of the cell. **The lowest
direction index therefore wins a tie**, which is the order that every other
walk over the neighbours of a hex uses.[^11]

**The reach is a whole number of cells and never a diffused quantity.** A
diffusion saturates near a strong source and falls to nothing far from it, so
the gradient it offers depends on how far away the reader is. A count of cells
gives the same answer at every distance, it costs an integer comparison rather
than a fixed-point multiply, and it needs no argument about a decay.

**A cell that admits no unit is not a candidate, and it carries no reach.** No
summary field says whether a unit may stand in a cell, so a cell of open water
would otherwise carry the reach across a lake and send a whole block at a
coast it can never cross. The rule reads the open tile count, which is the
count the open share already reads, so it states no second rule.[^12] [^13]

### D3. The field is indexed by the faction, and that is admitted at the pitch of a cell

A summary field indexed by the faction would multiply the tile side of the
world by the faction count, and an accepted record refuses that.[^14]

This field is at the pitch of one level 1 cell, and it is the second field at
that pitch to hold one plane for each faction.[^8] The refusal applies to the
tile side of the world, where the faction count multiplies the largest array
the engine holds. It does not apply to a lattice whose cell count is the tile
count divided by the tiles of a block.

**A reader who knows only the refusal reads this field as a violation of it.**
That is why the distinction is stated here rather than left to be inferred.

### D4. The field is derived again at every rebuild of level 1, and it carries nothing between frames

The engine clears the reach and derives every direction from the summaries
that the rebuild produced. Nothing accumulates.

Level 0 stays the only source of truth, and the field states no fact of its
own.[^15] It is a pure function of level 0 and of the live sites, in the way
that the exit field is.[^16]

**This answers the open question rather than waiting for it.** The question of
whether a plane above level 0 may carry the state of a solver is open, it
blocks the acceptance of the influence record, and the register recommends
deriving from nothing for a strategy field.[^7] A field that carries nothing
needs no answer from it.

### D5. The relaxation runs a fixed pass count, and the count is the reach

The solve reads no residual and tests no convergence. It runs the same number
of passes whatever the field holds.[^17]

The pass count is therefore the reach, in cells. A cell further than that from
every site of its faction holds no direction, and a unit standing there keeps
the behaviour it already has.

**Several sites seed one plane, so the limit binds on the spacing of sites and
not on the size of the world.**[^6] A faction whose sites are spread across the
world reaches most of it. A faction with one site in a far corner does not, and
that is the case the reach limit refuses to serve.

## The alternatives this rejects

**Compute a bearing from the unit's tile toward its own home tile.** It is
exact, it costs a constant number of integer operations, and it is what a
contributor writes first because the movement pass already reads one unit at a
time. It is rejected because a quantity computed from a unit's own position
toward a unit's own destination is a search under the record that governs
every strategy, whatever it costs.[^5] It also gives the engine a second
source of directions, so a reviewer can no longer answer the question "what
fixes the direction of a step" by naming one field.

**Climb the influence plane of the faction.** The plane exists, the solve
already runs, and a site is the natural thing to put a source at. It is
rejected on D4: the influence plane carries between frames, so a return
direction taken from it would depend on a plane that states a fact appearing
nowhere at level 0, which is the open question this record declines to
answer.[^7] It is rejected on D2 as well: the plane saturates near a source and
falls to nothing far from one, so the gradient it offers a distant unit is a
few counts of a narrow integer.

**Add a summary field that rises near a site.** A unit would climb it with the
exit field and no new field would exist. It is rejected because a summary
combines by addition, and the record that governs summary fields says a
direction is neither extensive nor intensive.[^3] The value that rises near a
site is also not a fold of the tiles of a block, so it is not a summary of
anything.

**Store the direction for each site rather than for each faction.** It would
send a unit to its own home rather than to the nearest site of its faction. It
is rejected on cost: the plane count would follow the site count, which the
content sets and which grows with the world. The consequence is that a unit may
be steered to a site that is not its home, and a register row holds it.[^18]

**Rank the reach with a strict comparison against the cell itself, and let a
cell with no lower neighbour keep a direction.** It is rejected for the same
reason the exit field refuses it: a cell that is already the best of its
neighbourhood would send every unit onto worse ground for ever.[^19] A seed
cell therefore holds no direction, and the unit falls back to a keyed
draw.[^20]

## Consequences

**A cell moves as a block.** Every laden unit of one cell and one faction takes
one direction. The engine cannot give two of them different directions,
because the mechanism that would do it is the search D1 forbids.

**The last block is a random walk.** The field ends at the cell that holds the
site, and the tile of the site is one tile of that block. A unit that arrives
in the right cell holds no direction, so the keyed draw carries it the rest of
the way.[^20] A field at block pitch cannot answer a tile, and the findings
register already holds that as a general statement.[^21]

**A unit may be steered to a site that is not its home.** The field carries the
direction of the nearest site of the faction, and the delivery still needs the
tile of the unit's own home site. A faction with one site does not meet this. A
faction with two does, and the register holds the row.[^18]

**The engine gains one array, indexed by the cell and by the faction, and one
relaxation over it.** Neither follows the population. No figure appears here,
because one blocker governs every cost figure this project holds.[^22]

**A strategy added later pays for its own plane.** The derivation is indexed by
the cell and by the strategy, so a project that wanted many strategies would
pay for each of them whether any unit held it or not. That consequence belongs
to the record that chose the shape, and this record does not remove it.[^5]

**Nothing enforces D1 against a later pass.** A contributor who computes a
direction from a unit's own address gets no failure from any gate. A reviewer
reads what the derivation is indexed by.

## References

[^1]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^2]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^3]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D3. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^4]: Findings register, FND-317. `docs/FINDINGS.md`
[^5]: ADR-0095, a behavioural strategy arrives as a field over cells, never as a search from a unit, decision D1. `docs/adrs/draft/adr-0095-a-behavioural-strategy-arrives-as-a-field-over-cells.md`
[^6]: ADR-0095, a behavioural strategy arrives as a field over cells, never as a search from a unit, decision D3. `docs/adrs/draft/adr-0095-a-behavioural-strategy-arrives-as-a-field-over-cells.md`
[^7]: Decisions register, DEC-095. `docs/DECISIONS.md`
[^8]: ADR-0060, an influence map is stored as a shared basis, decision D1. `docs/adrs/draft/adr-0060-an-influence-map-is-stored-as-a-shared-basis.md`
[^9]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
[^10]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D2. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^11]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^12]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D5. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^13]: ADR-0024, every summary field is declared extensive or intensive, decision D4. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
[^14]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D3. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^15]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^16]: ADR-0095, a behavioural strategy arrives as a field over cells, never as a search from a unit, decision D4. `docs/adrs/draft/adr-0095-a-behavioural-strategy-arrives-as-a-field-over-cells.md`
[^17]: ADR-0005, a solver runs a fixed iteration count, decision D1. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
[^18]: Decisions register, DEC-112. `docs/DECISIONS.md`
[^19]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D4. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^20]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D6. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^21]: Findings register, FND-315. `docs/FINDINGS.md`
[^22]: Blockers register, BLK-007. `docs/BLOCKERS.md`
