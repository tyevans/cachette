---
id: 0488
title: Plan roads and zones with a faction solver at the controller stage
status: proposed
created: 2026-09-05
implements: [ADR-0152 D1, ADR-0152 D2, ADR-0152 D3, ADR-0152 D4, ADR-0152 D5, ADR-0144 D1, ADR-0144 D2, ADR-0005 D1, ADR-0004 D1, ADR-0001 D4]
changes: []
creates: []
serves: [PRD-0055]
blocked-by: [BLK-007, BLK-050]
---

## Why

**A road is laid where a unit stands.** The controller orders a whole faction
to build, and every idle unit builds under its own feet. Two sites of one
faction are joined only by accident, and a god can neither read a plan nor
write one. The product asks that a road follow the faction's plan, that the
plan follow what the faction lacks, and that a god may zone a project of its
own.[^1] A decision record states the shape.[^2]

Each faction gains a bounded plan of projects, one tile and one category each,
held in tile order and hashed. A solver at the controller stage writes the
plan in a fixed number of passes over the sites the faction holds, the deposits
it knows and the sites that no finished road joins to the seat. A road project
is the shortest path over the ground between two of the faction's places, with
ties broken by the lowest tile index and a search bounded by a radius. One verb
writes a project, and the solver and a Python caller both call it. The build
verb refuses a road on a tile that no plan zones. After the solver has written,
the controller issues one build order for the idle units of the faction, and
the verb sends each unit to the nearest project by hex distance.

The plan bound, the pass count and the search radius are rows of the balance
register under BLK-050. This item adds the rows with a provisional value and a
derivation.

**This item touches `controller.rs` and `fn step` in `world.rs`. Only one
worker may hold it at a time.** It depends on item 0484, because the solver
reads the ground the faction holds and the sites that are unconnected, and on
item 0486, because a project names a category that only the table resolves.

## What is missing before this is refined

- The impact review, decision by decision, against ADR-0152 and ADR-0144. The
  review must say which aggregates the solver reads, and prove that none of
  them is a pass over the units or the tiles.
- What "unconnected" reads. A site is unconnected when no finished road joins
  it to the seat, and the review must say how that is derived at the barrier
  without a search on every tick.
- The verb signatures: write a project, clear a project, and the refusal for
  a road on an unzoned tile. The Python type stub is edited by hand in the same
  commit.
- The per-field tests and the extreme the fixture reaches: two paths that tie
  on cost, so the tile index rule is proven; two projects at equal hex
  distance from one unit, so the assignment tie is proven; a plan at its
  bound, so the drop is counted; and a pair past the radius, so no project is
  written.
- Whether the assignment of ADR-0152 D5 is the distance for each idle unit
  against each project, or a field, and what the population term costs at the
  target scale. The record chooses the distance and names the field as a later
  record.
- The "Done when" statements, in the shape of item 0472: the two determinism
  tests at 1, 2 and 12 threads, the defect put back and the test red, and the
  golden hash regenerated in the same commit with the reason in its body.[^3]

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: PRD-0055, a god raises the ground its people hold, and sees what stands there. `docs/product/shaped/prd-0055-a-god-raises-the-ground-its-people-hold-and-sees-what-stands-there.md`
[^2]: ADR-0152, a faction plans its roads and zones with one solver, and a unit builds only what the plan zones. `docs/adrs/draft/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
[^3]: Findings register, FND-320. `docs/FINDINGS.md`
