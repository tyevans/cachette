---
id: 0238
title: Decide per cell and need rather than per unit
status: complete
created: 2026-09-02
implements: [ADR-0096 D1, ADR-0096 D3, ADR-0096 D4, ADR-0064 D1, ADR-0064 D4, ADR-0064 D5]
changes: []
creates: [ADR-0098]
serves: [PRD-0009]
blocked-by: []
---

## Why

**The engine computes one answer many times.** The choice pass scores the fixed
option set for each live unit. The score is the unit's drive multiplied by a
weight and by the value the option reads from the unit's level 1 cell.

The engine holds one weight profile for every unit alive, and no unit carries a
type or a profile of its own.[^1] So the inputs to a choice are the unit's cell
and the unit's need, and nothing else. **Two units in one cell with the same need
always get the same answer, and the engine computes it twice.**

The record on cost binds this: the engine computes one answer once for every
reader that would compute the same answer, and a unit reads rather than
computes.[^2]

**The pass has two serial phases as well, and they are the part a thread count
cannot help.** It collects every live unit into one list before any thread
starts, and it applies the results afterwards by walking that list. Both grow
with the population. A finding holds the reading that found them.[^3]

## Impact review

**Governed by.** ADR-0096 D1 states that the cost of a pass follows the lattice
and never the population, and that the claim binds the work that decides rather
than the work that applies. Its D3 states that the partition axis of a parallel
pass is the cell, and that a pass which must end by touching units applies its
results in the order the lattice produced them. Its D4 states that the engine
computes one answer once for every reader that would compute the same answer.

ADR-0064 governs the choice itself. Its D1 fixes the scoring method, its D3 the
score floor, its D4 the interval and the stagger key, its D5 the tie-break, and
its D6 the levels the pass may read and write. **This work keeps every one of
them.** It changes where the score happens and not how it is computed.

**One sentence of ADR-0064 D1 is false after this work, and ADR-0096 D4 already
says so.** That sentence states that the cost of the pass is the option count
times the population and nothing else. It is a consequence the record derived,
not a decision it made. How the project repairs a stale consequence inside an
accepted record with dependents is an open question that a register holds, and
this item does not settle it.[^5]

**Creates.** ADR-0098. Quantising a need is a decision that ADR-0096 does not
make. The scope rule gives three conditions and all three hold.[^6] A contributor
could reasonably choose a different resolution, a different representative, or an
exact scheme. Choosing wrongly costs behaviour that a later reader cannot recover
from the table, and the reasoning that picks a resolution is invisible in the
code. The record also carries the statement that the behaviour changes, which the
gate does not.[^7]

**Blockers.** BLK-007. Every cost figure in this project is derived and nothing
is measured on the target platform. The bucket count is therefore chosen for
behaviour rather than for cost, and the reference table states the derivation and
names the blocker.[^4]

**Product record.** PRD-0009 states that a unit acts on the world it can see.
This work does not change what a unit sees. It changes how many times the engine
computes the answer.

**Precedent.** The register records that a fixture which models the typical case
supplies no extreme, so the assertion never receives the input that would fail
it.[^8] That is what happened here to the golden corpus, and a finding holds
it.[^9]

## Done when

- The choice pass walks the level 1 cells and never the live unit list. The
  serial collect over the population is gone.
- Each thread owns a contiguous range of cells, and the join reads the slots in
  slot order, so the result takes its order from the lattice.
- A cell holds one answer for each bucket of need, and a unit reads it.
- The bucket count sits in the reference table with its derivation, and not in
  the record.
- A test drives the engine to a need whose bucket changes the answer, and
  asserts that the unit took the answer of the bucket. The fixture asserts that
  it reached that need.
- A test holds the ceiling: a cell scores at most the bucket count, whatever the
  reader count.
- The three thread counts still agree, and the golden files are read rather than
  assumed.

## What this item does not do

**It does not remove the interval.** ADR-0064 D4 decides it, and ADR-0096 says
its removal has a frame consequence that nobody has measured under the new
shape. A separate item holds it.[^10]

**It does not repair the incompatible field ranges.** A weight is a preference
multiplied by a unit conversion that nobody has written down, and the register
holds it.[^11] This work moves the field read from the unit to the cell, so a
normalisation added later is computed once for each cell. It makes that repair
cheaper and it does not make it.

**It does not touch the other unit passes.** Movement, gathering, consumption and
the build pass all still walk the population. ADR-0096 governs each of them and
each needs its own item.

## Outcome

Done. The pass walks the lattice, the answer table holds one answer for each cell
and each bucket, ADR-0098 is written at `Draft`, and the reference table holds
the bucket count. **No golden hash moved**, and a finding records that no golden
scenario reaches the case the change alters.[^9] The interval stays, and item
0242 holds the question of removing it.[^10]

**One decision opened.** The measurement register holds a table of the collapse a
cell would show against the number of need buckets in play, and at the target
density the median cell holds about as many units as the engine holds
buckets.[^13] So the sharing may buy nothing in the worst case. The lazy fill
makes that a weak loss rather than a cost, because a cell scores the smaller of
the units it holds and the bucket count. The measurement that settles the bucket
count does not exist, and DEC-097 holds the choice.[^14]

**No figure in this item was measured.** The target platform run that motivated
it is in the measurement register, and nothing here was re-run on that
platform.[^12]

## References

[^1]: Findings register, FND-251. `docs/FINDINGS.md`
[^2]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decisions D2 and D4. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
[^3]: Findings register, FND-252. `docs/FINDINGS.md`
[^4]: Budgets and costs, the choice pass. `docs/reference/budgets.md`
[^5]: Decisions register, DEC-096. `docs/DECISIONS.md`
[^6]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^7]: ADR-0098, the choice is decided for each cell and each bucket of need. `docs/adrs/draft/adr-0098-the-choice-is-decided-for-each-cell-and-each-bucket-of-need.md`
[^8]: Testing rules, section 2a. `.claude/rules/testing.md`
[^9]: Findings register, FND-258. `docs/FINDINGS.md`
[^10]: Backlog item 0272, decide whether the choice interval can go. `docs/backlog/proposed/0272-decide-whether-the-choice-interval-can-go.md`
[^11]: Findings register, FND-233. `docs/FINDINGS.md`
[^12]: Target platform costs. `docs/reference/graviton-costs.md`
[^13]: Target platform costs, would the choice pass collapse if it decided for each cell. `docs/reference/graviton-costs.md`
[^14]: Decisions register, DEC-097. `docs/DECISIONS.md`
