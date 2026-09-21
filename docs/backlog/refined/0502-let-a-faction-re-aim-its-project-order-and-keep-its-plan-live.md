---
id: 0502
title: Let a faction re-aim its project order and keep its plan live
status: refined
created: 2026-09-05
implements: [ADR-0159 D1, ADR-0159 D3, ADR-0152 D1, ADR-0152 D2, ADR-0152 D5, ADR-0125 D4]
changes: []
creates: []
serves: [PRD-0055]
blocked-by: [BLK-050]
---

## Why

**A faction fills its plan to the bound early in a run and never clears a
project.** A run of the demonstration world at extent 96 with three factions
zoned 123 projects in 800 ticks and dropped 4677. Each of the three plans held
the bound from the twenty-first tick onward, so every write after that was a
drop. The findings register holds the reading and the command that produced
it.[^1]

A plan that never changes is not a plan. The solver runs on every tick and
writes what the faction should build next.[^2] The register refuses the write,
because the register is full of projects that nothing finishes. The subsystem
therefore runs, reports a number, and reaches nothing.

**Nothing releases a unit that a project order sent.** The order sends the idle
units of a faction toward the seed set the plan gives, and a sent unit is no
longer idle.[^3] The stage skips that unit on every later tick. The seed set is
written once, on the first tick a plan holds a project, and the stage never
aims at a later plan. Instrumentation of the controller stage over sixty ticks
found one send, at the first tick, and none after it.

**A newly idle unit overwrites the seed set and steers all units away.** When a
single idle unit is sent, the stage overwrites the destination seeds of the
faction's plane with that single project. Units already walking toward other
projects lose their target and turn toward the new seed, leaving earlier
projects unbuilt.

**Projects that cannot be built stay in the plan for ever.** An upgrade on a
tile that already holds a different category cannot be built, because a tile
holds one upgrade. `sweep_finished` sweeps only completed matching upgrades,
leaving impossible projects in the plan to consume the bound permanently.

**This item holds the last step of the way between two settlements.** The
solver plans that way, and a probe of the demonstration world over four seeds
at six hundred ticks reports about forty road projects for each faction that
holds two settlements.[^7] The same probe reports no pair of settlements that
a standing road joins. Cycling the plan and re-aiming the units lets the faction
complete its ways and join its places.

## Impact review

**Governed by.** ADR-0152 D1 bounds the plan to a fixed limit, and D2 gives the
solver a fixed iteration count.[^2] ADR-0152 D5 directs idle units to their
nearest project.[^2] ADR-0159 D1 and D3 state that the project order builds a
shared seed set for the faction from the projects its units chose, and that
sent units climb that shared field.[^3] ADR-0125 D4 releases a sent unit when
its destination plane stops steering it.[^8] ADR-0168 holds a builder on its
tile until the work completes.[^9]

**Changes.** None. No decision record is superseded. The change realizes the
cycling and re-aiming behavior specified by ADR-0152 and ADR-0159.

**Creates.** None.

**Blockers.** BLK-050 governs downstream balance values, including the plan
bound (`PLAN_BOUND_DEFAULT = 40`) and the solver pass counts.[^5] The bound is
not changed; cycling the plan keeps it active within the bound.

**Precedent.** FND-496 records the initial demonstration run where the road
chain finished nothing.[^1] FND-545, FND-549, and FND-550 record the seed sweep
and builder hold defect repaired by item 0505.[^9] FND-572 and FND-576 record
releasing sent units upon arrival or when the plane leads nowhere.[^8]

## Done when

- `sweep_finished` clears any project whose tile holds another upgrade
  category, via `plan.clear`, in addition to sweeping completed upgrades via
  `plan.finish`.
- `project_partition` and `controller_take_projects` assemble destination seeds
  from all units assigned to projects on that plane, including newly sent idle
  units and units currently climbing the plane.
- When an idle unit is sent, the destination seed set preserves the targets of
  other units walking on the plane rather than overwriting them.
- When a faction's plan holds no remaining projects, the destination plane's
  seed set is cleared, allowing `release_sent_units` to release the walking
  units to resume ordinary actions.
- The 8-seed demonstration world test in `crates/cachette-core/tests/plan.rs`
  passes, and the closed seed count is maintained or raised.
- Unit tests verify:
  1. An unbuildable project on a tile holding another category is cleared from
     the plan during the solver pass.
  2. Sending a newly idle unit preserves the seed destinations of existing
     walking units.
  3. Sent units re-aim toward remaining projects when a targeted project is
     finished or cleared.
- Determinism tests pass at 1, 2, and 12 threads.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-496. `docs/FINDINGS.md`
[^2]: ADR-0152, a faction plans its roads and zones with one solver, decisions D1, D2 and D5. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
[^3]: ADR-0159, a project order names one category for each unit and one seed set for the faction, decisions D1 and D3. `docs/adrs/accepted/adr-0159-a-project-order-names-one-category-and-one-seed-set.md`
[^4]: The carrying test. `crates/cachette-core/tests/carrying_a_load_home.rs`
[^5]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^6]: Backlog item 0504. `docs/backlog/complete/0504-find-why-one-seed-of-eight-finishes-no-project.md`
[^7]: The road join probe. `crates/cachette-core/examples/road_join_probe.rs`
[^8]: ADR-0125, the control plane names the seed set of a destination field, decision D4. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
[^9]: Backlog item 0505. `docs/backlog/complete/0505-keep-a-builder-on-the-tile-it-builds-until-the-work-is-done.md`
