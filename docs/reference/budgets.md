# Budgets and Costs

This document is a **register**. It holds the cost and storage figures for the
project. The figures change. A decision record cites this document; a decision
record does not hold a figure.[^1]

The registry names this document as the place for the byte budget table, the
per-tick cost budgets, and every constant that an unanswered question
governs.[^2]

**Every figure in this project is derived, not measured.** No code exists, so no
benchmark has run. Mark each figure with how it was derived. When a measurement
replaces a derivation, say so in the row and give the commit.

## Status

No measured figure is recorded. The foundation crate exists, and no benchmark
harness runs on the target platform.

The scale constants below are decided or derived, not measured. Each was held
here because a blocker governed it. Those blockers are now closed.

## Scale constants

The project owner fixed these on 30 August 2026. Each row names the blocker
that held it and says how the value was reached.

| Constant | Value | Blocker | How reached |
|---|---|---|---|
| Tile edge | 80 m | BLK-001 | Owner decision, from the report 17 calibration |
| World extent | about 330 km across | BLK-001 | Derived from the tile edge at 16.7 million tiles |
| March rate | 24 km in a simulated day | BLK-001 | Historical rate, held fixed through the calibration |
| Dwell | 2 ticks | BLK-001 | Derived from the tile edge and the march rate |
| Ordinary crossing | 12.5 s | BLK-001 | Approved calibration, consistent at an 80 m tile |
| Crossing-terrain capacity | 16 units | BLK-001, BLK-009 | Derived from dwell 2 at the approved crossing time |
| Ordinary tile capacity | 8 units | BLK-009 | Owner decision, stored as `u8` |
| Tiles crossed in a simulated day | 300 | BLK-012 | March rate divided by the tile edge |
| Ticks in a simulated day | 600 | BLK-012 | Tiles crossed multiplied by the dwell |
| Simulated time in one tick | 2.4 minutes | BLK-012 | A simulated day divided by the ticks in it |
| Real time for a simulated day | 60 s | BLK-012 | Ticks in a day at 10 ticks for each second |
| Total population | 1,000,000 | BLK-003 | Owner decision. Soldiers are a fraction of it |
| Living characters | 50,000 | BLK-004 | Owner decision, inside the report recommendation |
| Character ceiling | 262,144 | BLK-004 | Hard ceiling, two to the eighteenth |
| Character layer at the target | about 85 MB | BLK-004 | Linear scaling from the character report. Not measured |
| Settlements | 5,000 | BLK-005 | Owner decision, confirming the report assumption |
| Tiles carrying an upgrade | fewer than one in twenty | BLK-006 | Owner decision, agreeing with the report estimate |
| World shape | Rhombus | BLK-014 | Owner decision. A tile index is a raw axial pair |
| Maximum factions | 63 | BLK-013 | Owner decision. One bit for each faction in a 64-bit mask, with one value reserved for no faction |

The tile upgrade fraction picks sparse storage over dense storage. The
character layer figure is derived by scaling, not measured. BLK-007 still holds
every cost figure in this project.

The world shape and the faction ceiling are decided, not derived. The rhombus
removes the coordinate conversion that an offset index pays on every tile
access. The faction ceiling makes a relation one plane and a presence set one
word.

## What belongs here

- Per-tick and per-frame cost budgets.
- The byte budget table, for each entity tier and each pyramid level.
- Memory totals at the target scale of 16.7 million tiles and one million
  units.
- Throughput figures and latency figures, once measured on the target platform.
- A constant that a blocker governs, held here until the blocker closes.[^3]

## What does not belong here

- A structural constant of the target platform, such as the cache line size.
  That is a property of the platform the project chose. It belongs in the
  record that chose the platform.
- A decision. A budget is an input to a decision, not a decision.

## Figures still held in a record

Four draft records still hold derived cost figures in their bodies. They must
move here when those records are next revised. The record check carries the
list, and the check fails when an entry in that list matches nothing, so the
list cannot go stale.[^4]

| Record | Kind of figure |
|---|---|
| ADR-0003 | Two cache hit rate percentages |
| ADR-0005 | Allocation and cache miss costs for each frame, and the frame budget |
| ADR-0006 | Boundary call costs, and two percentage splits |

Moving a figure here is not a free edit. An accepted record does not change
except in status.[^2] Move a figure as part of the change that supersedes the
record, or while the record is still a draft.

## Format for a row

Give the name, the value, the unit, how it was derived, and the date. Give the
target platform for any figure that depends on the hardware. Cite the source in
a footnote.

## References

[^1]: Decision Record Scope. `.claude/rules/adr-scope.md`
[^2]: ADR Registry. `docs/adrs/REGISTRY.md`
[^3]: Blockers register. `docs/BLOCKERS.md`
[^4]: The record check baseline. `scripts/adr-volatile-baseline.txt`
