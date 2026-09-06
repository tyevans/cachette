---
id: 0505
title: Keep a builder on the tile it builds until the work is done
status: complete
created: 2026-09-05
implements: [ADR-0152 D3, ADR-0159 D3, ADR-0165 D1, ADR-0165 D2, ADR-0165 D3]
changes: []
creates: [ADR-0165]
serves: [PRD-0055]
blocked-by: []
---

## Why

**A builder adds one unit of work to a tile and then walks away.** The build
pass adds the build rate of a unit to the site the unit stands on, on every
tick that the unit holds a build order.[^1] The movement pass read no build
order. A unit that builds therefore moved in the same way as a unit that does
not, and it left the tile on the next tick.

**A project finishes only when a random walk returns to one tile enough
times.** The first level of a road asks for eight work, and one builder adds
one.[^2] In a measured run at extent 96 with three factions, the units of one
faction stood on a tile their own plan zoned for 253 ticks of 800, and 214 of
those ticks carried the matching build order. The work reached 31 separate
tiles. No tile passed seven of the eight that finish a road.[^4]

**The plan bound is not the lever.** A smaller bound gives a wandering unit
fewer tiles to stand on, and a larger bound spreads the same work over more
tiles.[^4] No value of the bound makes a random walk finish a build.

**This is not the release that item 0502 holds.** That item frees a unit the
project order sent, so that the faction can aim it again.[^5] A freed unit that
still walks away from the tile it builds finishes nothing either. The two are
one chain and two rules, and this one states what a unit does after it arrives.

## Impact review

**Governed by.** ADR-0091 D1 fixes movement to a field over cells and forbids a
search that starts at a unit.[^6] ADR-0095 D1 and D2 state why that shape was
chosen.[^7] ADR-0152 D3 and D5 state that the plan bounds where a unit may
build and that the controller orders every unit standing on a zoned tile.[^8]
ADR-0159 D1 and D3 state that a unit standing on a project takes the build
order whether or not it is idle.[^9] ADR-0150 D4 and ADR-0151 D2 state the two
rules that permit a build.[^10] [^11] ADR-0145 D1 and D2 state that a build
rate of zero means the unit cannot build.[^12]

**Changes.** None. No record said what a unit does after it takes a build
order, so nothing had to be superseded.

**Creates.** ADR-0165, a build order holds a unit on its tile, and the hold is
derived and never stored. The registry row was allocated before the file was
written.[^13]

**Blockers.** None. BLK-050 governs the plan bound, and this item reads no
value of the bound. The work introduces no balance value, because the hold is
bounded by the work the row asks for and by the rate of the unit's type.

**Precedent.** FND-545 holds the chain from the seed to the stall.[^4] Backlog
item 0039 holds the shape this item must avoid, which is a unit that cannot
move.[^14] Recurring defect shape 1 governs the rule that one function states
the build rule and every path calls it.[^15]

## Done when

- The movement pass leaves a unit alone while the unit stands on the work its
  own build order names.
- Nothing stores the hold, so no release rule can leak.
- A unit whose type adds no work is never held.
- The sweep of 200 seeds, at the extent, seeds and tick count the register
  names, finishes more projects at every seed than it did before.
- A test drives the engine for each release path, and each test goes red when
  the matching clause is removed.
- The two determinism tests pass at 1, 2 and 12 threads.

## Outcome

**The hold is derived in the movement pass and stored nowhere.** The pass asks,
for each unit, whether the tile it stands on is work that its own order names
and that its own type can advance. One function answers that question, and the
build pass and the movement pass both call it.[^15] A unit that carries no
order costs one byte of one column and no lookup.

**Nothing clears the hold, because nothing writes it.** The answer stops being
true on the tick the work finishes, on the tick the plan drops the project, on
the tick the ground changes hands, and on the tick a caller revokes the order.
A unit that dies leaves no state behind.

**A unit that adds no work is never held, and that clause is load-bearing.**
Three of the shipped type rows build at zero, and the controller orders every
unit of its faction that stands on a zoned tile. Without the clause one of those
units would stand on its tile for the rest of the run.

**A stored hold was rejected on a timing argument and not only on a leak
argument.** The build pass runs after the movement pass in the step, so a flag
it wrote would be one tick old when movement read it. The record holds the
reasoning.[^16]

**The reading moved, and the reading it was measured against did not
reproduce.** The register holds the before and after tables, and it holds a
separate correction: the 200 seed table of FND-545 does not reproduce at the
tree that recorded it.[^17] [^18]

**Registers.** ADR-0165 is in the registry as a draft. DEC-274 opens the
question of whether a held builder starves before it finishes. FND-549 holds
the reading, and FND-550 holds the correction to FND-545.

## References

[^1]: The build pass and the build intents. `crates/cachette-core/src/world.rs`
[^2]: Balance register, the road work by level and the build rate. `docs/reference/balance.md`
[^4]: Findings register, FND-545. `docs/FINDINGS.md`
[^5]: Backlog item 0502. `docs/backlog/proposed/0502-let-a-faction-re-aim-its-project-order-and-keep-its-plan-live.md`
[^6]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^7]: ADR-0095, a behavioural strategy arrives as a field over cells, never as a search from a unit, decisions D1 and D2. `docs/adrs/draft/adr-0095-a-behavioural-strategy-arrives-as-a-field-over-cells.md`
[^8]: ADR-0152, a faction plans its roads and zones with one solver, decisions D3 and D5. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
[^9]: ADR-0159, a project order names one category for each unit and one seed set for the faction, decisions D1 and D3. `docs/adrs/accepted/adr-0159-a-project-order-names-one-category-and-one-seed-set.md`
[^10]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D4. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^11]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D2. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
[^12]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decisions D1 and D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
[^13]: ADR Registry. `docs/adrs/REGISTRY.md`
[^14]: Backlog item 0039, a rejected unit is not stuck. `docs/backlog/proposed/0039-a-rejected-unit-is-not-stuck.md`
[^15]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
[^16]: ADR-0165, a build order holds a unit on its tile, and the hold is derived and never stored, the alternatives. `docs/adrs/draft/adr-0165-a-build-order-holds-a-unit-on-its-tile.md`
[^17]: Findings register, FND-549. `docs/FINDINGS.md`
[^18]: Findings register, FND-550. `docs/FINDINGS.md`
