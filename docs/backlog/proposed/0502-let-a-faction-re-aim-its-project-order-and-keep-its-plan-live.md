---
id: 0502
title: Let a faction re-aim its project order and keep its plan live
status: proposed
created: 2026-09-05
implements: [ADR-0159 D1, ADR-0159 D3, ADR-0152 D1, ADR-0152 D2]
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

**A unit that has delivered its load has no project to walk to.** The move pass
reads the destination plane of a unit before it reads the option row, so a unit
the order steered keeps walking the plane it was given. One test frees the
units it measures through the stop verb, because nothing in the engine frees
them.[^1] [^4]

**The road chain repair touches neither cause.** That repair made the build
rule refuse an order on a tile the faction zones for another category. The plan
figures barely moved.[^1] The plan still saturates, and the seed set is still
written once.

## What is missing before this is refined

A refiner answers these before this item leaves `proposed/`.

- **What releases a unit from a project send, and when.** A release on
  delivery, a release each time the plan changes, and a release on a schedule
  are three behaviours with three costs. Name the one the records support, or
  say which record must change.
- **Whether the send is a re-aim or a repeat.** ADR-0159 D3 gives the faction
  one seed set.[^3] Say whether the stage rewrites that seed set on every tick
  from the current plan, or whether it sends only the units it released. A
  rewrite costs the seed set every tick; a send costs the units it touches.
- **What clears a project that nothing can finish.** A project the plan holds
  and no unit can build occupies a row of the bound for the rest of the run.
  Say whether the solver clears it, whether a project expires, or whether the
  release alone is enough. ADR-0152 D1 bounds the plan and D2 gives the solver
  a fixed iteration count, so a clearing pass must fit both.[^2]
- **Whether the bound is the defect or the symptom.** BLK-050 governs the plan
  bound.[^5] A larger bound delays saturation and does not prevent it, so say
  what the bound is for once the plan cycles.
- **Which pass owns the release, and whether it is one arena walk.** This item
  touches `fn step`, so one worker holds it at a time. A release that walks the
  units once for the whole tick is a set-valued route. A release that searches
  for each project is not.
- **What a watcher reads to see that the plan cycles.** Four census rows do not
  count what their names claim, and two of them count the last tick alone.[^1]
  Say which rows this item repairs and which it leaves.
- **Whether the eighth seed is the same defect.** One seed of eight finishes no
  project at all. A separate item holds that question, and the refiner says
  whether this work closes it or leaves it open.[^6]

## Done when

Not written. Refine the item first.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, the road chain finished nothing in a run. `docs/FINDINGS.md`
[^2]: ADR-0152, a faction plans its roads and zones with one solver, decisions D1 and D2. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
[^3]: ADR-0159, a project order names one category for each unit and one seed set for the faction, decisions D1 and D3. `docs/adrs/draft/adr-0159-a-project-order-names-one-category-and-one-seed-set.md`
[^4]: The carrying test. `crates/cachette-core/tests/carrying_a_load_home.rs`
[^5]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^6]: Backlog item 0504. `docs/backlog/proposed/0504-find-why-one-seed-of-eight-finishes-no-project.md`
