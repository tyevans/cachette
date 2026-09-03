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

**This register holds target platform figures only.** Every row below describes
how the engine performs on the target, which is AWS Graviton. A second register
holds the one local figure the project keeps: how long the gate suite takes on
a development machine.[^7] The project owner decided to keep the two paths with
different standing, and a figure from one is never evidence about the other.[^8]
Do not add a development machine figure to this file.

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
| Mountain crossing | 50 s | BLK-007 | Owner decision, accepting the recalibrated figure |
| Ordinary step multiplier | 1 | BLK-007 | The calibration baseline. The dwell is derived over ordinary ground |
| Mountain step multiplier | 2 | BLK-007 | Derived from the ratio of the two accepted crossing times. Nobody decided it directly |
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
| Relation depth | 12 generations | BLK-004 | The depth at which the character report shows every step of the recursion is exact in Q16.16 |
| Record of descent ceiling | 4,194,304 rows | BLK-004 | Sixteen times the character ceiling, above the dead-to-living ratio the character report derives at 500 simulated years |
| World shape | Rhombus | BLK-014 | Owner decision. A tile index is a raw axial pair |
| Maximum factions | 63 | BLK-013 | Owner decision. One bit for each faction in a 64-bit mask, with one value reserved for no faction |

The tile upgrade fraction picks sparse storage over dense storage. The
character layer figure is derived by scaling, not measured. BLK-007 still holds
every cost figure in this project.

The step multiplier scales the step cost of a tile, and it is the third
quantity that a crossing time depends on. Capacity and dwell are the other
two.[^10] A crossing derivation that omits the multiplier gives a confident
wrong answer. The multiplier is content, and it sits in the terrain table
beside the terrain capacity, because the capacity and the multiplier describe
the same tile.[^11]

**The mountain multiplier is derived and nobody decided it directly.** The
project accepted an ordinary crossing of 12.5 seconds and a mountain crossing
of 50 seconds, and the ratio of the two figures is the multiplier.[^12] The
validated range runs from the ordinary multiplier to the mountain multiplier.
Ordinary ground is the floor, because the dwell was derived over ordinary
ground. The mountain multiplier is the ceiling, because the mountain crossing
is the longest crossing the project has accepted. Ground that wants a larger
multiplier needs an accepted crossing time that would justify it.

The forest kind and the hill kind carry the ordinary multiplier. No accepted
crossing time distinguishes them from level ground, and inventing an
intermediate value would state a figure that nobody derived. The register
holds the open choice.[^13]

The relation depth bounds the recursion that computes the relation between two
characters. Each step of that recursion halves a value, so the smallest term
the recursion reaches is two to the power of the negative depth. The Q16.16
scale holds sixteen fractional bits, so a depth below sixteen keeps every value
exact and no step rounds.[^9] The character report recommends a depth of six
for a gameplay test and names twelve as the ceiling that keeps the arithmetic
exact. The project takes the ceiling, because the cost at twelve is bounded by
the pairs the memoised recursion visits and the report shows that bound is
small.[^9]

The record of descent holds one row for each character the world has ever
created, so the living ceiling alone does not bound it. The character report
derives a dead count of about eight times the living count over 500 simulated
years at a mean lifespan of 60 years.[^9] Sixteen times the character ceiling
is the next power of two above that, and it is the value the code states. The
figure is derived by scaling, not measured.

The world shape and the faction ceiling are decided, not derived. The rhombus
removes the coordinate conversion that an offset index pays on every tile
access. The faction ceiling makes a relation one plane and a presence set one
word.

## Commodity constants

A commodity is a kind of good that a settlement stores, that the transport
solve moves, and that an individual carries. The project owner fixed three
limits on 31 August 2026.[^8] The three limits bound different things, so they
do not conflict.

| Constant | Value | What it bounds | How reached |
|---|---|---|---|
| Commodities that may exist | 64 | Existence | Owner decision. A presence mask is one `u64`, and 64 `i64` values fill exactly 8 cache lines on the target |
| Commodities in the transport solve | 16 | Participation | Owner decision, from the trade and flow report. Cache residency during the flow solve |
| Commodities an individual carries | 8 | Carriage | Owner decision, at the top of the range the agency report gave |

**Existence, participation and carriage are separate.** A commodity that exists
does not have to enter the transport solve. The commodities outside the solve
stay local to a settlement. A commodity an individual carries is a third,
smaller set again.

The cache line claim behind the first row is a property of the target platform,
which uses a 64-byte cache line. It is not a measurement. BLK-007 holds every
cost figure in this project, and these three values are decided, not derived
from a measurement.

## The choice pass

A unit scores a fixed option set and takes the highest score. Two parameters of
that pass are budget parameters and not design knobs, so they live here.[^5]

| Parameter | Value | Blocker | How reached |
|---|---|---|---|
| Score floor | 16,384 in the Q16.16 scale | BLK-007 | Report 16, section 3.7. One quarter of one unit of weighted need |
| Choice interval | 32 ticks | BLK-007 | Report 16, section 3.5. A power of two, at the low end of the range the owner asked for |
| Stagger key | The level 1 cell index, mixed | BLK-007 | Report 16, section 3.5, and FND-023 |

**The floor decides the mover count.** A unit whose highest score is below the
floor holds what it was doing and does not move. Without the floor, a world in
which every option scores near zero gives every unit the same option, and the
whole population walks one way.[^6] The movement stage is sized for a part of
the population, so a change to the floor changes the frame budget.

**The interval is a power of two**, so the phase test is a mask and not a
division. The engine takes it as a parameter of the world, and the value above
is the default.

BLK-007 holds all three rows. The derivations come from a research report and
nobody has measured them on the target platform.

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
- A figure taken on a development machine. The local register holds those.[^7]

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
[^5]: ADR-0064, a unit chooses by scoring a small fixed option set, decisions D3 and D4. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^6]: Findings register, FND-014. `docs/FINDINGS.md`
[^7]: Development budgets, the local register. `docs/reference/development-budgets.md`
[^8]: Decisions register, DEC-033 and DEC-001. `docs/DECISIONS.md`
[^9]: The character graph and inheritance, sections 2.3 and 3.6. `docs/research/reports/14-character-graph-and-inheritance.md`
[^10]: Findings register, FND-037. `docs/FINDINGS.md`
[^11]: Decisions register, DEC-017. `docs/DECISIONS.md`
[^12]: Decisions register, DEC-008. `docs/DECISIONS.md`
[^13]: Decisions register, DEC-093. `docs/DECISIONS.md`
