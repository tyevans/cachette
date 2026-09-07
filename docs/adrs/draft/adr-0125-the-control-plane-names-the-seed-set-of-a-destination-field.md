# ADR-0125: The control plane names the seed set of a destination field

## Context

A unit in this engine takes one step each frame. The direction comes from a
field over the level 1 cells, and never from a search that starts at the
unit.[^1] A level 1 cell summarises one block of tiles. A field holds one
direction for each cell, and every unit of that cell reads one entry.

Two such fields exist. The exit field ranks the six neighbours of a cell on a
summary field, and it steers a unit that chose an option.[^2] The return field
spreads a reach outward from every live site of a faction, and a laden unit
climbs it home.[^3]

**The return field is the mechanism a caller needs, and its seed set is
closed.** The record that holds it seeds the plane of a faction at the live
sites of that faction and at nothing else.[^4] A control plane that wants a set
of units at a mountain, at a frontier, or at a place a player chose cannot say
so, because none of those is a site.

That gap is the whole of the failure. The relaxation, the tie-break, the
refusal of a cell that admits nobody and the per-unit read are all built and
all correct. Only the question of who names the seeds is open, and the register
holds it.[^5]

A research report ranks this first among the things a downstream game asks
for. It states that the repair is a decision before it is a change, and that
two of the six verbs a god needs fall out of it: move a set of units somewhere,
and gather a set of units in a place.[^6]

**A per-unit route is not available and this record does not want it.** A
quantity computed from a unit's own position toward a unit's own destination is
a search, whatever it costs and whatever it is called. The record that governs
every strategy refuses it.[^7]

## Decision

### D1. The control plane names a set of tiles, and the engine seeds a plane at every one of them

A caller passes a set of unit identities and a set of addresses in one call.
The engine takes the level 1 cell of each address, gives every one of those
cells a reach of zero on one plane, and spreads the reach outward. The
direction of a cell is the neighbour whose reach is smaller.

**One field serves the whole set.** The cost follows the cell count and not the
number of units, so a call that sends a million units costs what a call that
sends one costs. This is the cheaper algorithm that a set-valued command exists
to permit, and it is not a batched per-unit loop.[^8]

The seed set is a set. The engine holds each cell once and in ascending order,
so two calls that name one set of addresses in two orders derive one field.[^9]

**The relaxation is the one the return field already runs.** The reach, the
fixed pass count, the strict comparison, the lowest direction index as the
tie-break, and the refusal of a cell whose open tile count is zero all come
from the record that states them.[^10] A second copy of that rule would be one
rule in two places, with nothing to fail when the copies disagree.[^11]

### D2. A unit carries the plane it obeys, and it carries no address

A unit holds one small number. The number says which destination plane steers
it, or that nobody sent it anywhere.

**The unit carries no address, no route, and no distance.** Those are what a
per-unit search would need, and the movement record forbids the search.[^1]

**A destination replaces the field that steers the step, and it replaces
nothing else.** An order from the control plane is not a score, so it does not
join the option set and it does not compete with one. The unit still chooses an
option, still gathers what that option gathers, and still eats.

**A sent unit reads no intent.** The choice pass writes an intent only on the
frame that the cell of the unit chooses, so a unit that has chosen nothing yet
holds none.[^12] A sent unit that waited for an intent would stand still until
its cell next chose, and a caller who gave an order would watch nothing happen.

### D3. The caller names the plane, and the engine allocates none

A world holds a fixed number of destination planes. The caller says which one
carries an order. A caller that names a plane again replaces the seed set of
that plane, and every unit already sent to it climbs the new one.

**An allocator would be state.** A free list of planes must be hashed, must be
freed, and must answer what happens when it runs out. A number the caller names
answers all three by holding nothing.

The count is a parameter of the world. No record holds the value and no
measurement chooses it, and one blocker governs every cost figure this project
holds.[^13]

**Re-aiming a live plane is the behaviour a caller wants.** A god that moves
its congregation on names the same plane with new seeds, and no unit needs a
second order.

### D4. A sent unit steps where its plane says, and the engine releases it when the plane says nothing

Two fields answer a sent unit. The fine field holds one direction for each tile
of a block that holds a seed. The coarse field holds one direction for each
cell. The fine field wins over the coarse one. A unit reads one entry of each,
keyed on its own tile and its own cell, so it still searches nothing.[^1]

The pair gives one of three answers. It names a neighbour, and the unit steps
onto it. It says the unit stands on a seed tile, so the unit arrived. It says
nothing at all, so the plane leads the unit nowhere. The last answer covers
three cases: the plane holds no seed, the cell is further from every seed than
the pass count reaches, and ground that admits nobody cuts the cell off.[^10]

**A sent unit never draws a direction from its plane.** It steps by the field,
or it takes no step. The draw made an arrived unit wander about the block it had
reached, and that wandering was the only thing that carried a load home in the
demonstration world. The findings register holds both measurements.[^16] [^19]

**The engine releases the unit on the second answer and on the third.** A unit
that arrived, and a unit whose plane leads nowhere, both hold a destination that
steers nothing. A sent unit reads no option row, so a unit that keeps such a
destination neither gathers nor delivers for the rest of the run.[^19]

**A field at block pitch cannot answer a tile, and the fine field can.** An
earlier form of this decision left the order in place for the caller to clear,
because the engine could not tell that a unit had arrived. The fine field
answers at the pitch of one tile, and the seed offset is how it says that the
unit stands on the tile the caller named.[^16]

**The release clears the destination and nothing else.** The home site, the
intent and the load of the unit all stand. The control plane may still stop an
order itself, and stopping an order that already ended changes nothing.

**The release is decided from the tile the unit stands on after the step of the
frame.** It writes the destination of one unit for each unit, and no write is
read by another, so the walk order decides nothing and the result is the same at
any thread count.[^9]

**A refused step is not a lost plane.** The field may name a neighbour that the
ground under one unit refuses. That unit stays sent and takes the keyed draw
that the movement record already states, because the refusal repeats exactly and
a unit that only stayed put would stay put for ever. A unit against a shoreline
is the case that proved it.[^14] [^15] [^16]

## The alternatives this rejects

**Leave the seeds at the live sites.** This is the state the return field
describes, and it costs nothing. It is rejected because it answers no verb that
a caller needs: a mountain is not a site, and a place a player chose is not a
site.[^6]

**Give a unit a destination it carries, and compute a bearing toward it.** It
is exact, it costs a constant number of integer operations for each unit, and
it is what a contributor writes first. It is rejected because it is the search
that the strategy record forbids.[^7] It also gives the engine a second source
of directions, so a reviewer can no longer answer "what fixes the direction of
a step" by naming a field.

**Make the destination an option of the choice set.** The option set already
steers a step, so a sixth row would need no new mechanism. It is rejected
because an order is not a preference. An option wins by scoring above the
others, so a unit that wanted food more than it wanted the destination would
ignore the order, and a caller who gave an order would get a suggestion.

**Let the engine allocate a plane and return its number.** The call site would
be shorter. It is rejected on D3: an allocator is state that must be hashed and
freed, and it must answer what happens when the planes run out.

**Clear the order when a unit reaches a seed cell.** A caller would not have to
stop an order that finished. It is rejected because the engine cannot tell that
a unit arrived. The field is at the pitch of a block, and the tile a caller
named is one tile of that block.[^16]

**Widen the return field instead of adding a second field.** One field would
hold both, indexed by the faction and by the destination together. It is
rejected because the two answer different questions and are seeded by different
callers. The return field is seeded by the world from its own sites, and this
one is seeded by the control plane. A reader of one plane could not say which
of the two wrote it.

## Consequences

**A cell moves as a block.** Every unit of one cell that holds one plane takes
one direction. A caller cannot send half a cell one way and half the other,
because the mechanism that would do it is the search D2 forbids.[^1]

**The last block is answered at tile pitch.** The reach of the coarse field
ends at the cell that holds the seed, and the tile the caller named is one tile
of that block. The fine field resolves that block, so a unit reaches the tile
and the engine then releases it.[^16] [^19]

**A caller sends a set toward a place, and the engine does not promise that the
set arrives.** A cell steers a block of tiles, and the water in front of one unit
of that block is not a fact the block carries. A unit behind such a barrier walks
to it and then wanders beside it. It is not frozen, and it does not get past. The
findings register holds the measurement, and a backlog row holds the gap.[^17]
[^18]

**An order beyond the reach ends at once.** The relaxation runs a fixed pass
count, so a cell further than that from every seed of its plane holds no
direction.[^10] The engine releases a unit there rather than making it wander,
so a caller who sends a set across the world must send it again from nearer.

**The engine gains one array, indexed by the cell and by the plane, and one
relaxation over it.** Neither follows the population. No figure appears here,
because one blocker governs every cost figure this project holds.[^13]

**A unit is either sent or steered by its own option, and never both.** A
caller that sends a set of gatherers stops them gathering by choice, and it
must stop the order to give the choice back.

**Nothing enforces D1 or D2 against a later pass.** A contributor who computes
a direction from a unit's own address gets no failure from any gate. A reviewer
reads what the derivation is indexed by, and reads whether a unit column holds
an address.

## References

[^1]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^2]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D2. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^3]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decision D1. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
[^4]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decision D2. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
[^5]: Decisions register, DEC-142. `docs/DECISIONS.md`
[^6]: Research report 21, what a god needs from this engine, sections 1.1 and 1.4. `docs/research/reports/21-what-a-god-needs.md`
[^7]: ADR-0095, a behavioural strategy arrives as a field over cells, never as a search from a unit, decision D1. `docs/adrs/draft/adr-0095-a-behavioural-strategy-arrives-as-a-field-over-cells.md`
[^8]: Project orientation, the design principles. `CLAUDE.md`
[^9]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^10]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decisions D2 and D5. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
[^11]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^12]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D4. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^13]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^14]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^15]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D6. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^16]: Findings register, FND-315. `docs/FINDINGS.md`
[^17]: Findings register, FND-411. `docs/FINDINGS.md`
[^18]: Backlog item 0401, decide how a sent unit gets around a barrier the field cannot see. `docs/backlog/proposed/0401-decide-how-a-sent-unit-gets-around-a-barrier.md`
[^19]: Findings register, FND-572. `docs/FINDINGS.md`
