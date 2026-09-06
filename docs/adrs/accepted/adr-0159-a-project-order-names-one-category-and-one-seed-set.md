# ADR-0159: A project order names one category for each unit and one seed set for the faction, and the field decides where a unit arrives

## Context

A faction holds a bounded plan of zoned projects. One solver writes the plan at
the controller stage, and a unit builds only inside a project.[^1] [^2] After
the solver writes, the controller issues one order for each faction, through
the verbs a Python caller also holds.[^3]

**The record that states the plan also stated the assignment, and it stated it
as a per-unit destination.** It said that each idle unit goes to the project
nearest to it by hex distance.[^4] A reader takes that as a promise: this unit
walks to that project.

**The engine moves a unit by a field, and one faction climbs one plane.** The
control plane names a set of units, a set of tiles and one plane. The engine
takes the level 1 cell of each tile, gives every one of those cells a reach of
zero on that plane, and spreads the reach outward. A unit reads the entry of
its own cell and steps.[^5] [^6] The plane is indexed by the faction, at the
pitch of a cell.[^7] No unit reads a neighbouring cell, and no unit computes a
bearing toward an address of its own.[^8]

A per-unit destination therefore needs a plane for each unit, or a search from
each unit. The second is refused by the record that fixes where a direction
comes from.[^8] The first is a loop over entities in all but name, and the
project holds that Python is a control plane and never a data plane.[^9] A
set-valued command exists to buy a cheaper algorithm, and a flow field in place
of many path searches is the example the project gives.[^10]

The plan record saw this. It says in its own consequences that a field is the
cheaper assignment, that the field would need the plan as its seed set, and
that a record which replaced the assignment would be a later record.[^11] This
is that record. A register holds the disagreement and its evidence.[^12]

## Decision

**A project order names one category for each unit and one seed set for the
faction. One faction climbs one field, and the field decides which project a
walking unit reaches. This record does not promise that a named unit reaches a
named project.**

### D1. The assignment decides a category and a seed set, and nothing else

The order reads every unit of the faction, and it sends only the idle ones. A
unit that is not idle is never sent.

A unit that already stands on a project of its own faction takes a build order
for the category that project names, whether that unit is idle or not. The
build verb refuses a tile that no project zones, so the order is safe to give.
Every other idle unit joins the set that the order sends.

For each unit of that sent set, the assignment takes the project nearest to it
by hex distance, and the lower tile index wins a tie. That choice decides two
things. It decides the category the unit builds when it arrives, and it puts
the chosen project into the seed set the faction climbs. The seed set is the
set of projects that the units of the order chose. The engine holds each cell
of that set once and in ascending order, so the order of the set decides
nothing.[^5] [^13]

The assignment runs through one reader, and the controller calls that reader.

A reviewer finds a violation when a unit is assigned by anything but hex
distance and tile index, when a busy unit joins the sent set, when the order
bypasses the verbs a caller holds, or when the assignment is written twice.

### D2. The record promises no named unit reaches a named project

The engine seeds one plane at every cell of the seed set and relaxes a reach
outward from the whole set at once. A cell takes one more than the smallest
reach of its neighbours, and the direction of a cell is the neighbour whose
reach is smaller.[^6] Every unit of the faction on that plane therefore walks
toward the seed that is nearest to its own cell.

**The field distributes the units.** A unit whose nearest project by hex
distance is not its nearest seed by reach walks to the second one. It builds
what it finds there, because a unit standing on a project of its faction takes
the build order for the category that project names.

**The stronger promise would forbid the cheaper algorithm.** A promise that
this unit reaches that project is a destination for each unit, and the engine
would then pay a plane for each unit or a search from each unit. The cost of
one field follows the cells, and it does not follow the units, so one call that
sends a million units costs what one call that sends one unit costs.[^5] That
is the cheaper algorithm the set-valued command exists to permit, and this
record keeps it.[^10] [^9]

**The rule stays testable where it decides.** The assignment is one reader, and
a test drives that reader for the category it names and for the tie it breaks.
The walk is the answer of the field, and the records that state the field test
it.[^6] [^8]

A reviewer finds a violation when a test asserts that a named unit reached a
named project, when the engine allocates a plane for a unit, or when a unit
computes a route of its own.

### D3. Several projects give several seeds, and the nearest one is a reach in cells

Every project that any unit of the order chose becomes a seed of the one plane.
Two projects inside one level 1 cell are one seed, because the field is at the
pitch of a cell.[^7]

The reach counts cells, and it counts them over ground that admits a unit. A
cell that admits no unit carries no reach and joins no path.[^6] The nearest
seed is therefore the nearest in cells over passable ground. It is not the
nearest in hex distance, and the two disagree wherever the ground is broken. A
project that a lake separates from a unit is not near that unit, whatever the
hex distance says.

**The word "nearest" holds for the set and not for a unit.** Every unit walks
toward a nearest project, in the reach the field measures. No unit is promised
the project that the assignment named for it.

A reviewer finds a violation when a second field is derived for one faction,
when a seed set holds one cell twice, or when a reach crosses a cell that
admits nobody.

## The alternatives this rejects

**A plane for each unit.** Rejected because the field is at the pitch of a cell
and one plane covers the whole lattice. A plane for each unit multiplies that
lattice by the population, and the record that admits a plane for each faction
admits it because the faction count is small.[^7]

**A search from each unit toward the project it was given.** Rejected because
movement takes its direction from a per-cell field and never from a per-unit
search.[^8] It also multiplies a path search by the population, which the plan
record rejected for the same reason.[^11]

**One call for each unit, each naming one seed.** Rejected because a caller
that names a plane again replaces the seed set of that plane, so the last call
would steer every unit sent before it.[^5] It is also a per-entity loop at the
boundary, and the control plane rule refuses that.[^9]

**Keeping the per-unit wording and calling the engine wrong.** Rejected because
three records state the movement form the engine has, and the wording
contradicts all three.[^5] [^6] [^8] A record the code contradicts is worse
than no record, because it lies with the authority of a record.[^14]

## Consequences

**The assignment of the plan record no longer states what happens.** This
record changes that one decision. Every other decision of the plan record
stands: the bounded plan, the solver, the road path, and the one verb that
writes a project.[^1] [^2] [^15] [^16]

**A test asserts the reader and never the walk.** A test that watched a named
unit arrive at a named project would pin the field rather than the rule, and it
would go red when the reach of a cell changed for a reason nobody argued about.

**The seed set is rebuilt whenever the order runs.** A caller that names one
plane again replaces the seed set of that plane, and every unit already sent
then climbs the new field.[^5] A project that no unit chose is not a seed, so a
plan may hold a project that nothing walks toward.

**One plane is one seat, and two orders cannot both hold it.** A faction climbs
one plane, so an order that seeds it takes the seat from whatever seeded it
before. The engine gives the seat to the campaign today, and no record states
that precedence. This record states the constraint that forces the choice, and
it does not make the choice.

**A god still reaches the field.** The plan is the seed set, so a caller that
zones a project has moved the field of that faction. The unit that walks to it
cannot tell a project of a god from one the solver wrote.[^16]

**Nothing here names a value.** The plan bound and the pass counts are balance
values behind the game rules blocker, and every cost figure is derived and
behind the cost blocker.[^17] [^18]

## References

[^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D1. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
[^2]: ADR-0152, a faction plans its roads and zones with one solver, decision D2. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
[^3]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^4]: ADR-0152, a faction plans its roads and zones with one solver, decision D5. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
[^5]: ADR-0125, the control plane names the seed set of a destination field, decision D1. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
[^6]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decision D2. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
[^7]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decision D3. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
[^8]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^9]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^10]: Project orientation, the design principles. `CLAUDE.md`
[^11]: ADR-0152, a faction plans its roads and zones with one solver, the consequences. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
[^12]: Findings register, FND-493. `docs/FINDINGS.md`
[^13]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^14]: Definition of Done, section 3. `.agents/rules/definition-of-done.md`
[^15]: ADR-0152, a faction plans its roads and zones with one solver, decision D3. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
[^16]: ADR-0152, a faction plans its roads and zones with one solver, decision D4. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
[^17]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^18]: Blockers register, BLK-007. `docs/BLOCKERS.md`
