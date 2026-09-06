---
id: 0501
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

**A faction's plan fills to its bound on the twenty-first tick of a run and
never changes again.** A run of the demonstration world at extent 96 with three
factions zoned 143 projects in 800 ticks and dropped 4657. The three plans held
forty projects each, which is the bound, from tick 21 onward. Every write after
that was a drop.[^1]

Two causes sit behind it, and the repair of the build rule that closed the road
chain touches neither.[^1]

**Nothing releases a unit from a project send.** The order sends the idle units
of a faction toward the seed set its plan gives, and a sent unit is no longer
idle.[^2] The stage therefore skips it on every later tick, so the seed set is
written once, on the first tick a plan holds a project, and it is never
re-aimed at the plan the solver has since written. Instrumentation of the
controller stage over sixty ticks found one send, at tick one, and none after
it.

**A project that nothing can build stays in the plan.** The solver writes a
path tile whether or not that tile already carries an unfinished upgrade of
another category. A tile carries one upgrade, so such a project can never be
finished, and it holds a row of the bound for the rest of the run.[^3] After
200 ticks of the run above, 33 of one faction's 40 projects sat on a tile that
carried another category.

**Two census rows do not count what their names claim.** A write past the bound
raises both `projects_dropped` and `projects_refused`, so the two are not
disjoint. `projects_refused` also counts every build refusal of the world and
not only a refused plan write. A reader cannot tell the three apart.[^1]

## Impact review

Not done. This item is `proposed/` and the review is the work that refines it.
The records that will govern it are named above.

## Done when

Not written. Refine the item first.

## References

[^1]: Findings register, FND-496. `docs/FINDINGS.md`
[^2]: ADR-0159, a project order names one category for each unit and one seed set for the faction, decisions D1 and D3. `docs/adrs/draft/adr-0159-a-project-order-names-one-category-and-one-seed-set.md`
[^3]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D2. `docs/adrs/draft/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
