# Budgets and Costs

This document is a **register**. It holds the cost and storage figures for the
project. The figures change. A decision record cites this document; a decision
record does not hold a figure.[^1]

The registry names this document as the place for the byte budget table, the
per-tick cost budgets, and every constant that an unanswered question
governs.[^2]

**Every figure in this document is derived, not measured.** Mark each figure
with how it was derived. When a measurement replaces a derivation, say so in
the row and give the commit.

**A measured figure does not live here.** A third register holds every figure
this project has measured on the target platform, with the machine, the commit
and the date that produced it.[^14] The two are separate files so that a reader
cannot take a derivation for a measurement.

**This register holds target platform figures only.** Every row below describes
how the engine performs on the target, which is AWS Graviton. A second register
holds the one local figure the project keeps: how long the gate suite takes on
a development machine.[^7] The project owner decided to keep the two paths with
different standing, and a figure from one is never evidence about the other.[^8]
Do not add a development machine figure to this file.

## Status

No measured figure is recorded **in this document**. A benchmark harness now
exists, and two runs on Graviton instances measured the cost of a frame, the
cost of building a world, the cost of the whole-world hash and the resident
memory of a world.[^14] None of those figures replaces a row below.

The scale constants below are decided or derived, not measured. Each was held
here because a blocker governed it. Those blockers are now closed.

## The frame budget

**One frame must cost 100 milliseconds or less at the target scale.** The
target scale is 16,777,216 tiles and 1,000,000 units, on the target platform.

| Figure | Value | Source | How reached |
|---|---|---|---|
| Frame budget | 100 ms | The project owner | Chosen, on 3 September 2026 |

**This figure is chosen, not derived.** No blocker governs it and no
derivation produces it. The project owner set it as a requirement. A reader
must not treat it as a quantity that a better derivation can move, and must
not adjust it to fit a measurement.

**The row exists because the figure did not.** Every cost figure in this
project was compared against 100 milliseconds, and the number appeared in no
register. It was recoverable only as the reciprocal of one sentence about
what a tick represents.[^16] A requirement that lives nowhere cannot be cited,
and it cannot be argued with either.

**The engine does not meet it.** The measurement register holds what one frame
costs today on the target platform.[^14]

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
character layer figure is derived by scaling, not measured. BLK-007 holds every
figure in this document, and the run that narrowed it measured none of
them.[^14]

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
which uses a 64-byte cache line. It is not a measurement, and a run on the
target has since read the same value from the machine.[^14] BLK-007 holds every
cost figure in this document, and these three values are decided, not derived
from a measurement.

## The choice pass

A unit scores a fixed option set and takes the highest score. Two parameters of
that pass are budget parameters and not design knobs, so they live here.[^5]

| Parameter | Value | Blocker | How reached |
|---|---|---|---|
| Score floor | 16,384 in the Q16.16 scale | BLK-007 | Report 16, section 3.7. One quarter of one unit of weighted need |
| Choice interval | 32 ticks | BLK-007 | Report 16, section 3.5. A power of two, at the low end of the range the owner asked for |
| Stagger key | The level 1 cell index, mixed | BLK-007 | Report 16, section 3.5, and FND-023 |
| Need bucket width | A shift of twelve in the Q16.16 scale, so 17 buckets | None | One tick of the default need decay, measured, see below |

**The floor decides the mover count.** A unit whose highest score is below the
floor holds what it was doing and does not move. Without the floor, a world in
which every option scores near zero gives every unit the same option, and the
whole population walks one way.[^6] The movement stage is sized for a part of
the population, so a change to the floor changes the frame budget.

**The interval is a power of two**, so the phase test is a mask and not a
division. The engine takes it as a parameter of the world, and the value above
is the default.

**The bucket width decides how finely the engine tells two needs apart.** The
choice is decided for each cell and each bucket of need, so two units whose
needs share a bucket receive one answer.[^16] **The width is the mechanism of
that decision and not a detail of it.** Unbucketed, the key is the exact need,
and the distinct keys in a cell are bounded by the cohorts standing in it. That
bound belongs to the content, because a world that gave every unit a site of its
own would put one key on every unit. The bucket is the bound the engine holds.

**The width is a parameter of the world, and no record sets it.** A review of the
record that governs the pass placed the choice on the item that implemented it,
and that item took a measurement.

**The derivation, and it is measured rather than argued.** The default need rule
takes a sixteenth of the need range off a unit on every tick. The width is that
amount, so a unit crosses one bucket in one tick. A finer bucket separates two
needs that the rule cannot separate inside a tick. A coarser one lets a need
change without the bucket changing, so the choice lags the need.

The measurement says what the finer bucket costs. In a world that consumes, at
the density the project states, the median cell holds about 75 units. At the
moment the needs are most spread, the matched width gives 17 distinct keys in
that cell and a width four times finer gives 41. **The finer bucket buys nothing
and it is not free.** The findings register holds the table, both placements it
is bounded by, and the fixture that produced it.[^19]

**The decay is a parameter of the need rule, so the two are coupled.** A caller
who changes the decay and leaves the width alone has unmatched them. That
coupling is why the world takes the width as a parameter rather than a module
holding it as a constant. A closed decision holds the reasoning.[^18]

**This row names no blocker, and that is deliberate.** The measurement behind it
is a distribution and not a cost. The simulation is deterministic integer
arithmetic, so the figures are the same on every machine. **What the pass costs
under this width on the target platform is not measured**, and the blocker below
covers that in the way it covers every cost figure.

The width is a power of two in the fixed-point scale, so the bucket of a need is
a shift and never a division.

BLK-007 holds the first three rows. The derivations come from a research report, and
the run that narrowed that blocker measured none of the three.

## What belongs here

- Per-tick and per-frame cost budgets.
- The byte budget table, for each entity tier and each pyramid level.
- Memory totals at the target scale of 16.7 million tiles and one million
  units.
- A figure that a decision needs and that no measurement has replaced.
- A constant that a blocker governs, held here until the blocker closes.[^3]

## What does not belong here

- A structural constant of the target platform, such as the cache line size.
  That is a property of the platform the project chose. It belongs in the
  record that chose the platform.
- A decision. A budget is an input to a decision, not a decision.
- A figure taken on a development machine. The local register holds those.[^7]
- A figure measured on the target platform. The measurement register holds
  those, because a row there carries a machine, a commit and a date that a row
  here does not.[^14]

## Figures still held in a record

A record that still holds a derived cost figure in its body must move it here
when the record is next revised.

**The claim that this list cannot go stale was wrong, and the list went
stale.** The record check carries a baseline of figures it tolerates, and that
baseline is what fails when an entry matches nothing.[^4] The baseline is
empty, so nothing checks the table below, and the table is prose like any
other.[^15]

**No record holds one today.** The three records this table named, which were
ADR-0003, ADR-0005 and ADR-0006, hold no figure of any kind. The work that
cleared them did not clear this table, and the table then named three records
as carrying figures they do not carry. A finding holds the case.[^15]

| Record | Kind of figure |
|---|---|
| None | |

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
[^14]: Target platform costs, the measurement register. `docs/reference/graviton-costs.md`
[^15]: Findings register, FND-242. `docs/FINDINGS.md`
[^16]: ADR-0098, the choice is decided for each cell and each bucket of need. `docs/adrs/draft/adr-0098-the-choice-is-decided-for-each-cell-and-each-bucket-of-need.md`
[^18]: Decisions register, DEC-097. `docs/DECISIONS.md`
[^19]: Findings register, FND-259. `docs/FINDINGS.md`
