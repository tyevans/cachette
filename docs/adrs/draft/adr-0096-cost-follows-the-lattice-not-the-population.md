# ADR-0096: Cost follows the lattice, not the population, and a unit is a reader

## Context

This engine holds a world of tiles and a population of units, and the scale
constants table holds both figures.[^1] The tiles are summarised into a lattice
of level 1 cells, and each cell covers one block of tiles.[^2]

The project orientation states a design principle: a set-valued command permits
a cheaper algorithm, and it names a flow field over a set against one path
search for each member. **That principle has never been written as a record.**
A principle in an orientation binds nobody. It cannot be cited in a review, it
states no consequence, and nothing refuses a change that ignores it.

The engine is built the other way. A frame is a sequence of passes, and most of
them walk every live unit and do a unit's worth of work for each one.

**Two shapes of pass exist in the engine today, and they behave differently.**
A pass over the tiles divides the tiles into contiguous ranges. A pass over the
units divides the units into contiguous ranges of the arena. The first shape
scales with threads. The second does not, and it stops improving well before
the thread count runs out. The measurements are on the target platform. A
reference table holds every row with the machine and the commit that produced
it.[^3] One blocker still governs the cost figures that no benchmark
reached.[^4] **No figure appears in this record**, because a measurement can
change and a record must not hold material that changes.[^5]

### Why the obvious explanation is wrong

The natural reading is that the unit passes contend: that they write where
another thread writes, and that contention does not parallelise. An accepted
record already forbids exactly that.[^6]

**The unit passes do not violate it.** The choice pass gives each thread a
contiguous chunk of the live units and one output slot of its own, and it joins
the slots in index order. It obeys the rule that a parallel stage writes only to
memory that no other thread writes, and it obeys the rule that the partition is
derived from the data rather than from the schedule.[^6] [^7]

It scales poorly anyway, and the reasons are visible in the source rather than
in a profile.[^8] The pass collects every live unit into one list before any
thread starts, and it applies the results by walking them afterwards. Both
are serial and both grow with the population. Between them, each thread reads a
unit's tile, converts it to a cell, reads that cell's summary and reads that
unit's need. The arena is in spawn order and never compacts, so consecutive
units in a chunk touch scattered tiles, scattered cells and scattered needs. A
record already states that condition and declines to claim the locality as a
property of the engine.[^9]

**So obeying the rule about writes is necessary and it is not sufficient.** The
unit passes partition on the axis that satisfies that rule and destroys
locality, because the axis they partition on is the unit.

## Decision

### D1. The cost of a pass follows the lattice, and never the population

**A pass does an amount of work proportional to the number of cells, and adding
units does not add work to it.**

This is the claim, and it is checkable without a profiler. A reviewer reads what
the pass is indexed by. If the population appears in the count, the pass is the
shape this record refuses, whatever it is called and however it is threaded.

A pass that must touch every unit still exists, and D2 says what such a pass may
do. The claim binds the work that decides, not the work that applies.

### D2. A unit is a reader, not an actor

**A unit reads an answer that the lattice already holds. It does not compute
one.**

A unit may read one entry and apply it to itself: take a step, take a value,
take an intent. It may not search, may not score a neighbourhood, may not walk
a set of candidates, and may not compute a quantity that the unit standing
beside it would compute the same way.

The two subsystem records that exist are instances of this decision rather than
neighbours of it. Movement takes its direction from a per-cell field and never
from a per-unit search.[^10] A behavioural strategy arrives as a field over cells
and never as a search from a unit.[^11] Both are D2 applied to one subsystem, and
a future subsystem that needs the same shape does not need a third record.

### D3. The partition axis of a parallel pass is the cell, and passing the rule about writes is not enough

A parallel pass divides the lattice, and each unit of parallel work owns a
contiguous region of it.

**The rule that a stage writes only where no other thread writes is necessary
and it is not sufficient.**[^6] A pass can satisfy that rule completely, as the
choice pass does, by partitioning on the unit index, and it then reads a
scattered set of tiles, cells and columns. The rule governs correctness under a
weak memory model. It does not choose the axis.

The axis is the cell, because the cell is the one index under which the reads
and the writes are both contiguous. A partition by unit index makes the writes
contiguous and the reads scattered. A partition by cell makes both contiguous,
and it satisfies the write rule as a side effect rather than as a constraint to
be met.

**A pass that must end by touching units applies its results in the order the
lattice produced them**, and not by walking a list of units and scattering into
the columns.

### D4. The engine computes one answer once for every reader that would compute the same answer

Where many units would compute one value, the engine computes it once and the
units read it.

**This does not change what any unit does.** A unit still takes the option that
the fixed set and its own state select, and the record that governs the choice
still governs it.[^12] What changes is where the work happens. Two units whose
inputs are equal receive one answer that was computed once, rather than two
answers computed twice.

The engine holds one weight profile for every unit alive, and no unit carries a
type or a profile of its own, so today the inputs to a choice are the unit's
cell and the unit's need and nothing else.[^13]

**The need enters the key as a bucket, and the width of the bucket is the
mechanism and not a detail of it.** A unit takes its ration with its cohort,
and every unit of one cohort gains the same amount, so a need follows the
cohort and not the unit.[^14] The distinct keys of a cell are therefore bounded by
the cohorts that the cell holds. That bound is far above one, so the exact need
does not collapse a cell to a single answer. It is also far below the units in
the cell, so the population is not the bound either. The exact key collapses a
little, and this decision needs a key that collapses to the lattice.

**The width is a parameter, and this record does not set it.** A measurement
now holds the spread of the needs of one cell in a world that consumes, and a
register holds it with the fixture that produced it.[^3] A wide bucket makes
two units of different need act alike. A narrow one approaches one answer for
each unit. The item that implements this decision carries that choice.[^15]

A value computed for each distinct pair of a cell and a need bucket is computed
once for every unit that shares it.

**One sentence of the accepted choice record becomes false.** That record states
that the cost of the choice pass is the option count times the population and
nothing else. Under this decision it is not. That sentence is a consequence the
record derived, not a decision it made. A closed decision now says that the
project strikes such a sentence in place rather than superseding the record, and
the registry holds the rule.[^16] [^17] The strike belongs to the commit that
accepts this record.

## The alternatives this rejects

**Keep the per-unit passes and buy scale with threads.** The engine would divide
the same work further. It is rejected because the serial phases at each end of a
unit pass grow with the population and no thread count removes them, and because
the scattered reads are a property of the axis rather than of the thread count.

**Keep the per-unit passes and hide the cost with a schedule.** The choice runs
at an interval today, so only a fraction of the population scores in a frame.
This is rejected as an answer to cost. An interval does not make a pass cheaper;
it makes it run less often, and it buys that by letting a unit act on a reading
that is as old as the interval. **A unit reacting late is a behaviour nobody
chose, and it should not be the price of a cost that a better axis removes.**

**Order the arena by tile so that a per-unit pass reads contiguously.** This
would keep the axis and fix the locality. It is rejected because it needs the
arena to compact, and compaction invalidates every identity that names a
slot.[^18] The project chose the generational identity deliberately, and this
record does not reopen it.

**Declare the read set and the write set of every stage, and check the ordering
mechanically.** This is a good idea and it is not this record. It is a separate
claim: it would make part of an existing accepted record checkable. That record
says in its own consequences that a reviewer can see an atomic and a lock, and
that the compiler refuses the violation a reviewer cannot see. The part that
neither the reviewer nor the compiler covers is the ordering.[^19] A record
holds one claim, and mixing a claim about cost with a claim about observability
produces a record that neither of them can be rejected from separately.[^20] An
item carries it.[^21]

## Consequences

**What the project can no longer do, and can refuse at review.** A per-unit
search, a per-unit scatter into shared columns, and a pass whose count includes
the population are each a design defect that a reviewer may refuse when it is
written. **They are not performance details to be measured later.** This is the
whole value of the record: it moves the refusal from a benchmark to a review.

**The engine as it stands does not satisfy this record.** Most passes walk the
population. This record is written against the engine and not from it, and a
reader must not take it as a description. The work that closes the gap is
filed.[^21] [^15]

**A pass may still touch every unit, and the record says which part is bound.**
Applying an answer to a unit is per-unit by necessity. Deciding the answer is
not. A reviewer separates the two and applies D1 to the second.

**The choice interval becomes a choice rather than a necessity.** The interval
exists because scoring every unit every frame is expensive. Under D4 the
expensive part is computed per cell and per need, so the reason for the interval
weakens. This record does not delete it, because the interval is a decision of
an accepted record and its removal has a frame-budget consequence that nobody
has measured under the new shape.[^22] An item holds it.[^15]

**Two units that share a cell and a need cannot be told apart by the engine.**
D4 makes that explicit rather than incidental. It is already true, for a
different reason: nothing gives two units different weights.[^13] A project that
later wants two such units to differ must give them different inputs, and the
lattice will then compute one answer for each distinct input rather than one for
each unit.

**A subsystem record about a field is an instance of this record and not a
precedent.** A third subsystem that needs a field cites this record, rather
than copying either of the two that exist.[^10] [^11] **Neither of those two
cites this record today.** Both were written before it, and both are drafts,
so a later edit may add the citation. This record does not claim the citation
exists.

**Nothing enforces this record.** No check counts what a pass is indexed by. A
reviewer applies D1 by reading the pass, and a contributor who writes a per-unit
decision gets no failure. A reader must treat this as unenforced.[^22]

## References

[^1]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^2]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^3]: Target platform costs, the register of measured figures. `docs/reference/graviton-costs.md`
[^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^5]: Decision Record Scope, section 4.1. `.claude/rules/adr-scope.md`
[^6]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^7]: ADR-0009, parallel stages write disjoint outputs, decision D3. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^8]: Findings register, FND-252. `docs/FINDINGS.md`
[^9]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D4. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^10]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^11]: ADR-0095, a behavioural strategy arrives as a field over cells, never as a search from a unit, decision D1. `docs/adrs/draft/adr-0095-a-behavioural-strategy-arrives-as-a-field-over-cells.md`
[^12]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D1. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^13]: Findings register, FND-251. `docs/FINDINGS.md`
[^14]: The pass that feeds every unit from what its cohort received. `crates/cachette-core/src/cohort.rs`
[^15]: Backlog item 0238. `docs/backlog/complete/0238-decide-per-cell-and-need-rather-than-per-unit.md`
[^16]: Decisions register, DEC-096. `docs/DECISIONS.md`
[^17]: ADR Registry, repairing a derived consequence is not an amendment. `docs/adrs/REGISTRY.md`
[^18]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^19]: ADR-0009, parallel stages write disjoint outputs, the consequences. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^20]: Decision Record Scope, section 2. `.claude/rules/adr-scope.md`
[^21]: Backlog item 0237. `docs/backlog/proposed/0237-declare-what-each-stage-reads-and-writes.md`
[^22]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
