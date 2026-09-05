# ADR-0152: A faction plans its roads and zones with one solver, and a unit builds only what the plan zones

## Context

A road is laid where a unit stands. The build verb takes a set of units and a
category, and each unit adds work to the tile under it. The demonstration
controller draws a build order for a whole faction from one keyed draw, so
every idle unit of the faction builds where it happens to be.[^1] Two sites of
one faction are joined by a road only by accident, and nothing reads what the
faction lacks.

The product asks for the opposite. A road is laid where the faction's plan
says, a plan follows what the faction lacks, and a god may write a plan of its
own that a unit then builds.[^2]

The controller record fixes where a plan can be made. A faction controller is
one system at one fixed stage of the step, it reads bounded aggregates, it acts
only through the verbs a caller can call, and its evaluation cost never follows
the population or the tile count.[^3] A solver runs a fixed iteration count and
never a convergence test.[^4] Every parallel result is ordered by a stable
key.[^5] Whatever a later frame reads enters the whole-world hash.[^6]

**The shortest path plans per unit.** Each unit would search for the nearest
site and walk toward it, laying road as it goes. This is the shape the engine
has now, with a destination. It multiplies a path search by the population,
and the plan lives nowhere, so a god cannot read it and cannot write it.

**The second shortest path plans once, from the seed.** A road network could be
a generated field, in the way the ground is. It costs nothing to store and it
never follows a need. It is rejected below.

The upgrade table gives a plan something to name. A project names a category,
the engine resolves the row from the ground, and the same build verb raises the
level.[^7]

## Decision

**A faction holds a bounded plan of zoned projects. One solver writes the plan
at the controller stage, in a fixed iteration count, from what the faction
holds and what it lacks. A caller writes a plan through the same verb. A unit
builds a road only inside a project, and an idle unit takes the nearest project
through the existing build verb.**

### D1. A faction holds a bounded plan of zoned projects

A plan is a list of projects. A project is one tile and one category. The list
has a fixed bound, and the bound is a balance value that lives in the balance
register and never in this record.[^8] A project past the bound is dropped, and
the drop is counted in the census.[^9]

The plan is simulated state. The step reads it, so it enters the whole-world
hash in project order.[^6] Two worlds that differ only in a plan never hash the
same.

The plan is held in ascending tile order. A tile appears in a plan once. A tile
cannot hold two projects, because a tile carries one upgrade.[^10]

A reviewer finds a violation when a plan grows without a bound, when a plan is
outside the hash, or when the order of a plan depends on which project was
written first.

### D2. One solver writes the plan, in a fixed iteration count

The solver runs at the controller stage, once for each faction the controller
evaluates.[^3] It reads three things: the sites the faction holds, the deposits
the faction knows, and which sites are unconnected. A site is unconnected when
no finished road joins it to the seat of the faction. Each of those is a
bounded aggregate that the engine already exposes or derives at the barrier. The
solver reads no unit and no tile outside the ground it holds.

The solver makes a fixed number of passes over those aggregates. Each pass
scores the candidate projects and writes the best into the plan. The pass
count is a balance value.[^8] There is no convergence test and no time
budget.[^4] A solver that ran until every site was connected would give a
different plan under a different load.[^11]

The solver follows what the faction lacks. A faction with an unconnected site
plans a road toward it. A faction with a deposit it cannot reach plans a road
toward the deposit. A faction whose stores fall short plans the category that
raises the yield of the ground it holds. A faction that lacks nothing plans
nothing.

A reviewer finds a violation when the solver starts a pass over the units or
over the tiles, when it stops on a condition, or when the plan of one faction
depends on the visit order of another.

### D3. A road project joins two of the faction's places along one deterministic path

A road project joins a site to the seat, a site to a site, or a site to a
deposit. The path is the shortest path over the ground between the two ends,
where the cost of a step is what the ground charges and a tile that admits
nobody is not on any path. When two paths tie on cost, the path through the
lower tile index wins at every branch. The search is bounded by a radius that is
a balance value, and a pair further apart than the radius yields no
project.[^8]

Every tile of the path is a project with the road category. No unit lays a road
outside a project. The build verb refuses a road order on a tile that no plan
zones, and counts the refusal.[^9]

The tie rule is the whole determinism of the path. A search that took the first
neighbour it visited would follow the order of a container, and that order is
not part of the data.[^5]

A reviewer finds a violation when a road is laid on a tile that no project
names, when a path search breaks a tie by anything but the tile index, or when a
search has no radius.

### D4. A caller writes a plan through the verb the solver uses

One verb writes a project into a plan. The solver calls it and a Python caller
calls it. The verb takes a faction, a tile and a category. It refuses a tile the
faction does not hold, a tile past the plan bound and a category the ground does
not suit, and it counts each refusal.[^9] [^7]

No verb exists for the solver alone.[^12] A god that zones a project by hand
puts it in the same list the solver writes to, and a unit cannot tell the two
apart. A caller may also clear a project, through one verb that both may call.

A reviewer finds a violation when the solver writes a plan by a path the
boundary does not expose, or when the verb reads who called it.

### D5. An idle unit takes the nearest project through the build verb

At the controller stage, after the solver has written, the controller issues
one build order for each faction through the existing build verb. The set is the
idle units of the faction. The verb sends each unit to the project nearest to
it by hex distance. When two projects tie, the lower tile index wins. A unit
that is not idle is not moved.

The order carries no level and no tile chosen by the unit. The unit builds what
the project names, and the engine resolves the row from the ground.[^7] A
project whose category the ground refuses stays in the plan and is counted as a
refusal on every tick, so a plan that asks for the impossible is visible.[^9]

The cost of the assignment follows the idle units of the faction times the plan
bound. The plan bound is fixed, so the cost follows the population and not the
world. That is the population term the controller record already assigns to the
command and not to the evaluation.[^3]

A reviewer finds a violation when a unit is sent to a project by anything but
hex distance and tile index, when a busy unit is reassigned, or when the
assignment bypasses the build verb.

## The alternatives this rejects

**Roads laid per unit, where a unit stands, as today.** Rejected because the
plan lives nowhere, so a god can neither read it nor write it, and because a
road is then a record of where units were rather than of what the faction
needed.[^2]

**A global road network generated from the seed.** Rejected because a generated
field cannot follow a need: the same seed gives the same roads whatever the
factions do, and a faction that founded nowhere near them gains nothing. A road
is a thing a unit made, and no function of the seed produces it.[^10]

**Roads that emerge from traffic.** A tile that many units cross would become a
road. Rejected because it is a per-tile counter over the whole world, which the
sparse storage record refuses, and because the result follows the movement pass
and not a plan, so a god still cannot write one.[^10]

**A per-unit path search to a destination.** Rejected because it multiplies a
search by the population and because the controller record forbids an
evaluation that starts a pass over the units.[^3]

**A solver that runs until every site is connected.** Rejected because a
convergence test gives a different answer under a different load.[^4] [^11]

## Consequences

**A faction pays for planning by its bound and not by its size.** The solver
cost follows the faction count, the pass count and the plan bound. No term
follows the tile count. The assignment cost follows the idle units, and it is
the command term the controller record already accepts.[^3]

**A god can zone.** The same verb that the solver uses is open to a caller, so a
player that wants a road where the solver would not lay one writes the project.
The unit that builds it cannot tell the two apart.

**A road cannot be laid on a whim.** A build order for the road category on an
unzoned tile is refused. A game that wants a wandering unit to lay road behind
it must zone the path first.

**The golden state hash moves.** The plan enters the hash, so every stored
golden hash changes. The commit that lands the plan records the change.

**The shortest path reads the ground and nothing else.** A road that already
stands lowers the cost of a step only through the ground composition the
sparse storage record fixes, so the path search reads one function and holds no
copy of the road network.[^10]

**A flow field is the cheaper assignment, and this record does not choose it.**
The assignment of D5 is a distance for each idle unit against each project. A
field over the faction's ground toward the nearest project would cost the
ground and not the population. The project already prefers a set-valued
algorithm over a per-entity loop.[^13] A record that replaced D5 with a field
would be a later record, and it would need the plan of D1 as its seed set.

**Nothing here names a value.** The plan bound, the pass count and the search
radius are balance values behind the game rules blocker, and every cost figure
is derived and behind the cost blocker.[^14] [^15]

## References

[^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D4. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^2]: PRD-0055, a god raises the ground its people hold, and sees what stands there. `docs/product/shaped/prd-0055-a-god-raises-the-ground-its-people-hold-and-sees-what-stands-there.md`
[^3]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D1. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^4]: ADR-0005, a solver runs a fixed iteration count, decision D1. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
[^5]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^6]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^7]: ADR-0151, an upgrade is a category with a ground fit and a level, and a build order names the category, decision D2. `docs/adrs/draft/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
[^8]: Balance register. `docs/reference/balance.md`
[^9]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D3. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^10]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decisions D1 and D3. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
[^11]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^12]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^13]: Project orientation, the design principles. `CLAUDE.md`
[^14]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^15]: Blockers register, BLK-007. `docs/BLOCKERS.md`
