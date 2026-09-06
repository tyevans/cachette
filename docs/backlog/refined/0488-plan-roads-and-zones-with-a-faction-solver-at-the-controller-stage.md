---
id: 0488
title: Plan roads and zones with a faction solver at the controller stage
status: refined
created: 2026-09-05
implements: [ADR-0152 D1, ADR-0152 D2, ADR-0152 D3, ADR-0152 D4, ADR-0152 D5, ADR-0144 D2, ADR-0144 D3, ADR-0144 D5, ADR-0005 D1, ADR-0004 D1, ADR-0009 D1, ADR-0001 D4]
changes: []
creates: []
serves: [PRD-0055]
blocked-by: [BLK-007, BLK-050]
---

## Why

**A road is laid where a unit stands.** The controller draws one category and
orders every idle unit of the faction to build it. Each unit builds under its
own feet, so a road appears where gathering left the unit. The road is the one
kind that the own-ground refusal exempts, so those roads also appear on ground
nobody holds.[^1] Two sites of one faction are joined only by accident, and a
god can neither read a plan nor write one.

The product asks for the opposite. A road follows the faction's plan, the plan
follows what the faction lacks, and a god zones a project of its own.[^2] A
decision record states the shape.[^3]

Each faction gains a bounded plan of projects, one tile and one category each,
held in tile order and hashed. A solver at the controller stage writes the plan
in a fixed number of passes. A road project is a deterministic path between two
of the faction's places. One verb writes a project, and the solver and a Python
caller both call it. The build verb refuses a road on a tile that no plan
zones. After the solver has written, the controller sends each idle unit to the
nearest project through the existing build verb.

**This item touches `controller.rs` and `fn step` in `world.rs`. Only one
worker may hold it at a time.** It waits for item 0486 to merge, because a
project names a category that only the upgrade table resolves.

## Impact review

### Which records govern this work

**ADR-0152 D1** holds that a faction holds a bounded plan of projects, that a
project is one tile and one category, that the plan is simulated state inside
the whole-world hash, and that the plan is held in ascending tile order with
one project for each tile. A project past the bound is dropped and counted.

**ADR-0152 D2** holds that one solver writes the plan at the controller stage,
in a fixed pass count, from the sites the faction holds, the deposits it knows
and the sites that are unconnected. It forbids a convergence test, a time
budget, and a pass over the units or the tiles.

**ADR-0152 D3** holds that a road project joins two of the faction's places
along the shortest path over the ground, that a tie breaks by the lower tile
index at every branch, and that the search is bounded by a radius.

**ADR-0152 D4** holds that one verb writes a project, that the solver and a
Python caller both call it, that it refuses and counts, and that a second verb
clears a project.

**ADR-0152 D5** holds that the controller sends each idle unit to the nearest
project by hex distance, that a tie breaks by the lower tile index, that a busy
unit is not moved, and that the assignment goes through the build verb.

**ADR-0151 D2** holds that a build order names a category and no level, and
that the engine resolves the row from the ground. A project therefore holds a
category index and never a level. **ADR-0151 D1** holds that the table is data
the world is built with, so the plan names a row of that table.[^4]

**ADR-0150 D4** holds that a build off the builder's own ground is refused
unless the kind is a road.[^1] This item leaves that rule in place and adds a
second rule beside it. **ADR-0144 D2** holds that no verb exists for the
controller alone. **ADR-0144 D3** holds that a refused command is dropped and
counted. **ADR-0144 D5** holds that the commands sort by faction and then by
draw index.[^5]

**ADR-0004 D1** holds that iteration order is explicit. **ADR-0009 D1** holds
that a parallel stage writes disjoint outputs. **ADR-0005 D1** holds that a
solver runs a fixed iteration count. **ADR-0001 D4** holds that the two
determinism tests protect the claim.[^6] [^7] [^8] [^9]

### Which registers open or close

**Balance.** This item adds five rows: the plan bound, the projects one pass
writes, the solver pass count, the path relaxation pass count and the road
search radius. Every row is `unset, pass 10`. Stage two writes a provisional
value into each row and fills its derivation column.

**Blockers.** No blocker closes. BLK-050 governs the plan bound, the projects
one pass writes, the solver pass count and the road spacing, because the rules
of the downstream game are not written down.[^10] BLK-007 governs the cost of
the solver stage and the cost of the road work.[^11] Both stay open, and every
cost figure here stays derived.

**Findings.** This item opens one finding for each defect that a fixture hides
and one for the disagreement stated below. FND-320 records that nothing
regenerates the Python type stub, so stage two edits it by hand in the same
commit. FND-051 and FND-048 record that a fixture built for realism hides a
defect, so every fixture here is built for its extreme.[^12]

**Decisions.** One open choice arrives with this item: whether the road stays
exempt from the own-ground rule. The recommendation is below.

**Registry.** ADR-0152 stays a draft. This item accepts nothing.

### One disagreement inside ADR-0152

**ADR-0152 D3 and ADR-0152 D4 cannot both hold as written.** D3 says a road
project runs along the path between two places, and that path crosses ground
the faction does not hold. D4 says the write verb refuses a tile the faction
does not hold. Under D4 the verb refuses most tiles of every road path.

Stage two writes the verb so that it refuses a tile the faction does not hold,
**unless the category is a road**. That is the same exemption ADR-0150 D4
already states for the build.[^1] The item records the disagreement as a
finding. A reviewer of ADR-0152 decides whether the exemption belongs in D4.
This item does not edit either record.

### What a project is

A project is one tile index and one category index. It is a plain-data row with
`repr(C)` and declared padding, in the form the campaign row already takes.[^13]

The plan is one bounded register for each faction, in the form of the campaign
register. It holds a fixed number of rows for each faction, and the number is
the plan bound. The rows of one faction stay in ascending tile order. A tile
holds one project, because a tile carries one upgrade.[^14] A write for a tile
that a project already names replaces that project.

The register hashes byte for byte into the whole-world hash, faction by
faction and tile by tile.[^9] Two worlds that differ only in a plan never hash
the same. A write past the bound is dropped and counted in the census.

### What the solver reads, and which readers exist

**The sites the faction holds.** `SettlementArena::iter`, `faction`, `tile` and
`address` exist. The seat is `World::seat`. The cost follows the settlements of
the faction and never the tile count.

**What each site lacks.** `World::faction_stores` gives the store of a faction
for each resource kind. `World::shortfall_log` gives the sites whose upkeep
could not pay. `World::store_capacity_raise` gives what a store can still hold.
Each exists today, and each is bounded.

**Which of its places are unconnected.** **No reader exists.** Nothing derives
connectivity, and nothing indexes the roads. `World::finished_upgrade` answers
for one tile, and `World::upgrade_sites` returns the sparse entries.

Without a reader, the solver calls a site unconnected when the path it already
computed toward the seat holds one tile that carries no finished road. The test
is one read for each tile of that path. It costs the path and never the world.
This item adds no connectivity index. A union over the sparse road entries at
the barrier is the alternative, and it stays an open question.

**Where its deposits are.** **No reader exists.** A tile stock is generated
from the seed, and only what was taken is stored.[^15] `World::original_stock`
and `World::tile_stock` answer for one tile. No aggregate names the deposits of
a faction.

Without a reader, the solver reads the generated stock of the tiles it already
visits inside the bounded window of a road search. It plans a road toward a
deposit only when that deposit sits inside the window. A deposit past the
window is invisible to the plan, and this item states that rather than hiding
it. This item adds no deposit index and opens no pass over the world.

**The ground the faction holds.** `Holding::tiles_held_by` and `World::holds`
exist. The cost follows the ground held, which ADR-0150 already bounds by the
cities and their reach.[^1]

**No read here starts a pass over the units or over the tiles.** The solver
reads the settlements of one faction, the stores of one faction, the ground of
one faction, and one bounded window for each path.

### What a road project is

A road project joins the seat to a site, a site to a site, or a site to a
deposit. The near end and the far end are two tiles.

**The window.** The window holds every tile whose hex distance to the near end
is at most the search radius. A far end outside the window yields no project.
The window size follows the radius, which is a balance value.

**The path.** The solver holds one cost array over the window. The cost of the
near end is zero, and every other cost is the maximum. A tile that admits no
unit takes no cost and joins no path.

The solver then runs exactly the path relaxation pass count. Each pass visits
the window in ascending tile index. For each tile it reads the six neighbours
in the fixed order the hex module gives, and it takes the lowest pair of the
step cost and the neighbour tile index. **A tie therefore takes the lower tile
index at every branch.** The step cost is what the ground charges.

**The pass count never changes with the input.** No pass stops early, no pass
tests whether anything changed, and no pass reads a clock.[^8] The same window
runs the same number of passes at every load and at every thread count.

The solver then walks from the far end back to the near end. At each step it
takes the neighbour with the lowest pair of cost and tile index. The walk is
bounded by the window size. A far end that still holds the maximum cost after
the passes is unreachable inside the budget, and no project is written.

Every tile of the walk becomes one project with the road category, written
through the verb in ascending tile order.

**Why this is deterministic.** Every read is an array read at a fixed index.
The neighbour order is fixed by the hex module. No container iteration order
and no thread completion order reaches the result.[^6] [^7]

**Why a priority queue is refused.** A best-first search pops in an order the
container decides, and it runs until the frontier settles. That is a
convergence test, and a solver runs a fixed iteration count.[^8]

### Why this fixes what the owner saw

The owner saw roads that sprawl and sites that nothing joins. The cause has two
parts. The controller draws one category and orders every idle unit of the
faction to build it, so a unit builds wherever gathering left it. The road is
exempt from the own-ground refusal, so those builds also land on ground nobody
holds.[^1] Nothing reads what the faction lacks, and nothing writes a plan a
god can read.

**After this item no unit lays a road outside a project.** The build verb and
the build intent pass both refuse a road on a tile that no plan zones, and each
refusal is counted. The controller sends each idle unit to the nearest zoned
project instead of leaving it where it stands. A road then marks what the
faction planned, and not where its units happened to be.

### Whether the road stays exempt from the own-ground rule

ADR-0150 D4 exempts a road from the own-ground refusal deliberately. A road is
how a faction reaches ground it does not yet hold, and a settler walks it.[^1]

**Recommendation: keep the exemption, and let the plan be the second bound.**
A road then needs two things: the exemption from the ground rule, and a project
that zones the tile. The plan alone cannot replace the exemption, because the
write verb of ADR-0152 D4 refuses a tile the faction does not hold. Under that
rule a faction could zone no road off its own ground, so it could never reach
new ground, and the reason ADR-0150 D4 exists would be lost.

The plan is the tighter bound of the two. It says which unheld tiles a road may
cross, and the exemption only says that unheld tiles are allowed at all.
Together they answer the owner's complaint without removing the reach.

**This item does not edit ADR-0150.** A reviewer of ADR-0150 and ADR-0152
decides. The item records the recommendation and the disagreement above.

### The open questions this item still holds

1. **The unconnected test.** The path test above costs one path. A union over
   the sparse road entries at the barrier would answer for every site at once.
   Which one the engine keeps is open, and the item builds the cheaper one.
2. **The deposit reader.** A faction cannot plan toward a deposit it cannot
   see. Whether the engine gains a bounded deposit index, and who maintains it,
   is open.
3. **The assignment shape.** ADR-0152 D5 chooses a distance for each idle unit
   against each project. A flow field over the faction's ground would cost the
   ground and not the population. That is a later record.
4. **The road spacing.** Two projects may zone two roads one tile apart.
   Whether a spacing rule belongs in the solver is open, and BLK-050 governs
   the value.
5. **Whether a plan survives the loss of a site.** A project stays in the plan
   when the site that asked for it dies. Nothing clears it today.

## Done when

- A faction holds a bounded plan register. A row is a plain-data pair of a tile
  index and a category index, with `repr(C)` and declared padding. The rows of
  one faction are in ascending tile order, and one tile holds one project.
- The plan register hashes into the whole-world state hash, and a test proves
  that two worlds differing only in a plan hash differently.
- A write past the plan bound is dropped, and the census counts it.
- One verb writes a project, and one verb clears a project. The solver and a
  Python caller both call the same two verbs, and neither verb reads who called
  it.
- The write verb refuses a tile outside the world, a tile the faction does not
  hold unless the category is a road, and a category the ground does not suit.
  It counts each refusal.
- The solver runs at the controller stage, once for each faction the controller
  evaluates. It reads the settlements, the stores and the held ground of that
  faction, and nothing else.
- The solver runs exactly the solver pass count. The count is the same for
  every input, and a test asserts the number of passes at two different worlds.
- The solver holds no convergence test, no early exit and no clock read.
- A road project runs along the path the relaxation gives. A tie takes the
  lower tile index, and a far end past the search radius yields no project.
- The build verb and the build intent pass both refuse a road on a tile that no
  project zones, and both call one function that states the rule.
- After the solver writes, the controller issues one build order for the idle
  units of each faction. Each unit goes to the nearest project by hex distance,
  a tie takes the lower tile index, and a unit that is not idle is not moved.
- The census holds two new rows: the projects zoned and the projects finished.
- The Python binding reads a faction's plan and calls both verbs. The type stub
  is edited by hand in the same commit, and a pytest file drives each from the
  boundary.
- The solver returns the same plan at 1, 2 and 12 threads.
- A faction with two unconnected sites zones a road between them, and the path
  is the same on every run.
- Two paths that tie on cost resolve by the lower tile index, and the fixture
  builds the tie rather than hoping for one.
- Two projects at one hex distance from one unit resolve by the lower tile
  index, and the fixture builds the tie.
- A plan at its bound drops the next project, and the census count rises.
- A pair of places past the search radius yields no project.
- Each defect is put back once, and the test goes red: drop the path tie rule,
  drop the assignment tie rule, stop the passes when nothing changed, leave the
  plan out of the hash, skip the unzoned refusal, and drop the bound. The
  commit body names each defect and the test that caught it.
- The two determinism tests pass, and the golden state hash files are
  regenerated in the same commit with the reason in the commit body.
- `cargo fmt --check`, the clippy gate on both crates, the named test binaries
  and the pytest file run green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D4. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^2]: PRD-0055, a god raises the ground its people hold, and sees what stands there. `docs/product/shaped/prd-0055-a-god-raises-the-ground-its-people-hold-and-sees-what-stands-there.md`
[^3]: ADR-0152, a faction plans its roads and zones with one solver. `docs/adrs/draft/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
[^4]: ADR-0151, an upgrade is a category with a ground fit and a level, decisions D1 and D2. `docs/adrs/draft/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
[^5]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decisions D2, D3 and D5. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^6]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^7]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^8]: ADR-0005, a solver runs a fixed iteration count, decision D1. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
[^9]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^10]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^11]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^12]: Findings register, FND-320, FND-051 and FND-048. `docs/FINDINGS.md`
[^13]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
[^14]: ADR-0090, a tile upgrade is stored sparsely, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
[^15]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D1. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
