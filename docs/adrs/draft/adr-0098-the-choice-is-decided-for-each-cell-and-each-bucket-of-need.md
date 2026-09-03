# ADR-0098: The choice is decided for each cell and each bucket of need

## Context

A unit chooses by scoring a fixed option set against the level 1 cell it stands
in, and it takes the highest score.[^1] Each score is one product of what the
unit wants by how much of that thing is near.

**The engine holds one weight profile for every unit alive.** No unit carries a
type and no unit carries a profile of its own, and the findings register records
that.[^2] So the inputs to a choice are the cell of the unit and the need of the
unit, and nothing else. Two units that stand in one cell with one need always
receive one answer, and the engine computes it twice.

A separate record binds this. The cost of a pass follows the lattice and never
the population, and the engine computes one answer once for every reader that
would compute the same answer.[^3] That record states the constraint. It does
not say how a continuous input becomes a shared answer, and it says in its own
text that the choice is the pass which must close the gap.

**A need is a fixed-point value, so it is nearly continuous.** The engine cannot
hold one answer for each value a need can take. It must therefore either compute
one answer for each unit, which is the shape the record refuses, or divide the
need range and hold one answer for each part of it.

**Both extremes are wrong, and the interesting question is between them.** A
coarse division makes two units with clearly different needs act alike. A fine
division approaches one answer for each unit and shares nothing. A contributor
could reasonably choose either, and the reasoning that picks a resolution is
invisible in the table that results.

**The choice of a representative is also a decision.** A part of the need range
must be scored at some need. The lower bound, the midpoint and the upper bound
are all defensible, and each gives a different answer at a boundary.

## Decision

### D1. The choice quantises the need into buckets, and one answer serves a bucket

The engine divides the need range into equal buckets. It computes one answer for
each cell and each bucket, and a unit reads the answer of its own bucket.

**The key gained a third term after this record was written.** An option may
now rank a bounded class of the state of the unit itself, so the key is the
cell, the bucket and the class. The claim of this decision is unchanged: the
key holds a bounded number of terms, and each of them is bounded by the engine
rather than by the content. A later record states the third term and the
reasoning for it.[^17]

**The width is the mechanism of this decision and not a detail of it.**
Unbucketed, the key is the exact need, and the number of distinct keys in a cell
is then bounded by the cohorts standing in that cell. A cohort is one site and
one faction, and its units draw one ration and hold one need exactly, so units of
one cohort share a key and units of two cohorts almost never do.[^12]

**That bound belongs to the content and not to the engine.** A world that gave
every unit a site of its own would put one key on every unit, and nothing in the
engine refuses such a world. A bound that a content author can remove cannot
carry D1, which asks for work that the population does not raise. **The bucket
gives a bound that the engine holds**, and that is why the key is a bucket rather
than a need.

**The width is a parameter of the world, and this record does not set it.** It
has a behavioural consequence and a cost consequence, and both are measured
rather than argued, so the reference table holds the value and its derivation
and this record holds neither.[^4] The bucket width is a power of two in the
fixed-point scale, so the bucket of a need is a shift and never a division.

**This changes behaviour, and the project accepts the change.** Two units whose
needs sit in one bucket now receive one answer where they could have received
two, and a measured world puts a different population in a cell at one width than
at another.[^12]

**The golden state hash does not move, and that is a fact about the scenarios
rather than about this decision.** No golden scenario reaches a need where the
bucket changes the answer, and the width was varied over most of its range
without moving a file. A finding holds the evidence and how it was
taken.[^16] **A reader must not take a green gate as proof that this decision is
inert.**

### D2. A bucket is scored at its lower bound

The engine scores a bucket at the need that opens it. The topmost bucket holds
the full need alone, so a unit that needs everything is scored at its exact need.

The lower bound is chosen because it makes the topmost bucket exact and because
it is the value that a shift already produces. A midpoint would need a second
derivation of the bucket width, and one value derived in two places is the defect
shape this project records.[^5]

### D3. The table fills as a unit asks for a bucket, and the fill changes no answer

A cell builds an empty table and scores a bucket the first time a unit asks for
it. A cell that holds few units therefore scores few buckets, and the table never
costs more than the per-unit pass it replaces. A cell that holds many units scores
at most the bucket count, so the deciding work has a ceiling that the population
cannot raise.

**The answer of a bucket depends on the bucket, on the cell and on the profile.**
It does not depend on which unit asked first, and it does not depend on how many
asked. The lazy fill is therefore invisible in the result, and the pass gives one
answer at any thread count.[^6]

A table belongs to one cell and to one thread. No thread reads a table that
another thread wrote, so the fill needs no ordering rule of its own.[^7]

### D4. An explanation scores the need that the pass scored

The engine answers a question about a choice by computing the scores again from
the world as it stands, because it stores no score.[^8] That recomputation takes
the bucket of the need and not the need.

An explanation that scored the exact need would report a winner that the unit did
not take, at every need where the bucket changes the answer. A watcher would then
read a correct engine as a broken one. The explanation reports the exact need and
the scored need together, so a reader can see the quantisation rather than
discover it.

## The alternatives this rejects

**Score each unit at its exact need.** This is what the engine did, it is exact,
and it is the shape the cost record refuses. It computes one answer for every
reader rather than once for all of them.

**Find the exact boundaries at which the answer changes.** Each score is monotone
in the need, so the winner is a step function of the need and the engine could
store the steps instead of a table. It is rejected because the fixed-point
multiply truncates. Two scores can therefore exchange the lead more than once
near a boundary, so the step function is not the small exact object the argument
assumes, and a search for the boundaries would be correct only by luck.

**Quantise the need in the column, so that the stored need is the bucket.** This
would make the choice exact against the stored value, and it would remove the
question of a representative. It is rejected because the need is a rate that a
consumption pass fills and a decay empties.[^9] Rounding it in storage would
round every arithmetic step that touches it, and the quantisation would leak out
of the choice into a subsystem that never asked for it.

**Fix the width in the module as a constant.** This is what the first
implementation did, and it is rejected. The width that matches the dynamics
depends on the rate at which a need moves, and that rate is a parameter of the
need rule.[^9] A constant would hold one half of a coupled pair and give a caller
no way to match the other. D1 therefore makes the width a parameter of the world,
in the same way the interval is.[^14]

**Choose the width from a cost figure.** The lazy fill of D3 caps the deciding
work at the per-unit cost whatever the width, so a width chosen for cost alone
would tie a behavioural parameter to a machine, and a blocker governs every cost
figure this project holds.[^15] The width was instead chosen against a measured
distribution of the needs of a cell, which is a property of the simulation rather
than of a machine, and the register holds it.[^13]

## Consequences

**The engine can no longer tell two units apart inside one bucket.** The cost
record already made that true for two units with one cell and one need; this
record widens it to a range of needs. A project that later wants finer behaviour
must either narrow the bucket or give the units different inputs.

**A test that asserts a unit's choice must state the bucket it depends on.** A
fixture that sets a need near a bucket boundary asserts a property of the
boundary and not of the option set. A test that wants the option set states a
need in the middle of a bucket.

**The width is now behaviour, and nothing fails when it moves.** A change to it
changes what units do, so it is not a tuning knob that a contributor may move to
make a benchmark look better. **No golden file moves with it**, and a finding
records that the width was varied over most of its range without moving one.[^16]
So the only thing that stops a contributor is a reviewer reading the argument for
the value, and a register holds that argument.[^13]

**Nothing enforces D1 against a new pass.** This record binds the choice pass. A
later pass that decides per unit gets no failure, in the same way that the cost
record gets none.[^10]

**The defect that four cell fields carry incompatible ranges is untouched, and it
becomes cheaper to repair.** A weight today is a preference multiplied by a unit
conversion that nobody has written down, and the findings register holds it.[^11]
This record does not change which fields an option reads or how they compare. It
does move the field read from the unit to the cell, so a normalisation added
later is computed once for each cell rather than once for each unit.

**How much D1 shares depends on the width, and the width is now measured rather
than guessed.** In a world that consumes, at the density the project states, the
needs of a cell spread widest while the stores empty and polarise afterwards. At
the widest spread the matched width shares about four readers to one answer, and
a width four times finer shares almost nothing.[^12] **D3 is what makes a badly
chosen width a weak loss rather than a cost**, because a cell scores the smaller
of the units it holds and the bucket count. A closed decision holds the
reasoning and what it rests on.[^13]

**The interval stays.** The cost record argues that the interval becomes a choice
rather than a necessity once the deciding work follows the lattice, and it
declines to delete it because the frame consequence has not been measured under
the new shape.[^10] This record makes no claim about the interval. The accepted
choice record still decides it.[^14]

## References

[^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D1. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^2]: Findings register, FND-251. `docs/FINDINGS.md`
[^3]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decisions D1 and D4. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
[^4]: Budgets and costs, the choice pass. `docs/reference/budgets.md`
[^5]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^6]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^7]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^8]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D2. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^9]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D1. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
[^10]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, the consequences. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
[^11]: Findings register, FND-233. `docs/FINDINGS.md`
[^12]: Findings register, FND-259. `docs/FINDINGS.md`
[^13]: Decisions register, DEC-097. `docs/DECISIONS.md`
[^14]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D4. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^15]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^16]: Findings register, FND-258. `docs/FINDINGS.md`
[^17]: ADR-0107, the choice key holds a bounded class of the unit's own state, decisions D1 and D2. `docs/adrs/draft/adr-0107-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
