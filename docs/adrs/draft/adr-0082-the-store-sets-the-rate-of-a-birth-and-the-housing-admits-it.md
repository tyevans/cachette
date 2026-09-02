# ADR-0082: The store sets the rate of a birth and the housing admits it

## Context

The number of units in a world is a number a caller chose. A faction that
gathers well and one that gathers badly hold the same units for ever. Two
product records ask for the opposite. One says the population responds to what
a faction has.[^1] The other says growth slows when there is nowhere for a new
person to live.[^2]

Two independent limits act on one quantity. A contributor who meets them
separately gets a result that depends on which limit ran first, and this
project cannot carry an order dependence.[^3] A contributor who multiplies the
two, or who takes the smaller of the two, gets one answer in the easy case and
a different answer at the boundary. The easy case is where a test looks, so the
choice is invisible in the code and invisible in a passing suite.

The engine already holds the parts. A site holds a pooled store, and
production and upkeep are rates attached to the site.[^4] A unit ends when a
shortage carries its deficit to a bound, and that end applies in one ordered
scan at the frame.[^5] A spawn places a unit and reads no capacity of its
own.[^6]

**No code implements this record.** Nothing grows a population today. The
decisions below state what the work must satisfy.

## Decision

### D1. The store sets a rate, and the rate proposes a birth

A site proposes a birth at a rate. The rate reads the store of the site. A site
with no surplus proposes none.

A rate is the shape this project already uses for what a site produces and
spends, and a birth is one more thing a site produces.[^4] The interval at
which the rate applies is a parameter of the schedule, never a constant of the
kernel.[^7]

This record states no value for the rate. The values are content, and no
content pipeline exists.[^8]

### D2. The housing capacity admits, and it never scales

The free places of a site admit the proposals of D1, in a stated order, until
no place is free. An admitted proposal becomes a birth. A refused proposal is
discarded, and it is not carried to the next application.

**The capacity is a bound, not a factor.** It never multiplies the rate, and it
never reduces it. A site with free places grows at the rate its store sets. A
site with no free place grows not at all. Between the two there is no third
behaviour, so no reader must ask which limit bit.

This is the shape the project already uses when a limited thing is asked for by
more askers than it holds: propose, sort by a stable key, admit while room
remains.[^9]

**The key orders the proposals within a site, and the site alone cannot do it.**
A site with one free place and more than one proposal in one frame is the case
that decides the shape, and every proposal at that site carries the same site
value. A key constant across the things it orders is a grouping and not an
order, so the sort would leave the choice to whatever produced the sequence. The
key is therefore the site and then an index that distinguishes the proposals of
one site, and the second part is what admits exactly one when one place
remains.

**The two limits therefore compose by one operation with one answer**, and
neither limit is applied twice.

### D3. A birth spawns a unit, and no headcount is added to

A birth creates a unit in the entity arena. Nothing adds to a cohort headcount,
and nothing adds to a count of the population as a separate act.

A cohort headcount is derived from the columns of the units, and it is rebuilt
after a structural change.[^5] A birth that also incremented a headcount would
declare the same population twice, and nothing would fail when the two
disagreed.[^10]

The new unit takes a slot, and its identity never resolves as the unit that
held that slot before it.[^11]

**The admitted births are applied to the arena in one ordered scan, in the order
of the admission key of D2.** The slot a new unit takes therefore follows from
the world and not from a thread.

This is not a second key. A slot comes from a free list, and the pass that ends
a unit runs immediately before growth in the same frame, so the slots that
deaths freed this frame are the slots that births take. Which newborn takes
which slot is decided by the order the births are applied in, and the slot is
part of an identity.[^11] An order left to the collection that happened to hold
the proposals would be stable, would pass both determinism tests, and would
still be unrecorded, so the next contributor who parallelises the apply or
changes that collection would move every identity downstream of a birth with
nothing to say what the order had been.

The key of D2 already orders every admitted birth totally, because a site is
distinct from another site and an ordinal is distinct within a site. The end of
a unit is applied the same way, in one ordered scan at the frame, so growth and
death take the same shape.[^5]

### D4. Growth runs after the pass that ends a unit, and its draw is keyed

The proposal and the admission run after the pass that removes a starved unit,
in the same frame. A place that a death freed this frame is free this frame,
and the occupancy that the admission reads is the settled one.

The draw that a birth needs is keyed on the tuple of system, frame, entity and
draw. **The site fills the entity slot, and the index of the proposal within the
site fills the draw slot.**[^12] Two sites in one frame draw different values,
one site draws a different value in the next frame, and two proposals of one
site in one frame draw different values.

The last of those three is the one that is easy to lose. The site is the actor,
so it belongs in the entity slot, and a second proposal from the same actor in
the same frame has nothing left to distinguish it unless the draw slot carries
the ordinal. The project has already met this and decided it the same way for
the founding, where the faction is the actor and the candidate ordinal moved to
the draw index.[^19] No thread-local state takes part.[^13]

A draw keyed on the wrong field draws the same wrong value on every thread and
every run, so both determinism tests pass while the defect stands. A test for
each field of the key is what finds it.[^14]

**A test for each field cannot find a field that the key is missing.** Three
tests are needed here and not two: change the frame, change the site, and take
two proposals of one site in one frame and assert that they draw different
values. The third test is the one that fails when the draw slot carries no
ordinal, and the other two pass whether the ordinal is there or not.

## Consequences

**A population can collapse and not recover, and that is a content question
rather than a defect of this record.** A birth adds a mouth to the same store.
Under the default need rule a unit whose need reaches zero cannot climb back,
so a shortage that empties a need is fatal.[^15] Growth under that rule pushes
a site into a shortage that no later plenty repairs. The register holds the
open choice, and this record states no default.[^16]

**A test of growth must state the rates it needs.** A fixture that takes the
default rule measures the default rule. Growth lives at the extremes: a full
site, an empty store, and one free place with more than one proposal.[^17]

**Two limits can no longer be tuned independently.** A designer who wants
housing to slow growth rather than stop it must supersede D2. The rate is the
only place a slowdown can live.

**A refused proposal leaves no trace.** Nothing accumulates pressure to be born
later, and nothing records that a site wanted to grow and could not. A caller
that wants that pressure adds it as state, and that is a decision.

**A spawn keeps its silence about capacity.** Growth counts the free places
itself, in the way that a caller which must not over-fill carries the rule
itself. The spawn path gains no refusal.[^18]

## References

[^1]: PRD-0011, a unit is born, holds a job and dies. `docs/product/accepted/prd-0011-a-unit-is-born-holds-a-job-and-dies.md`
[^2]: PRD-0014, everyone needs somewhere to live. `docs/product/accepted/prd-0014-everyone-needs-somewhere-to-live.md`
[^3]: ADR-0001, one binary gives one answer at any thread count, decision D5. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^4]: ADR-0062, production and upkeep are rates attached to a site, decision D1. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
[^5]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D4. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
[^6]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity, decision D1. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
[^7]: ADR-0062, production and upkeep are rates attached to a site, decision D4. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
[^8]: Decisions register, DEC-034. `docs/DECISIONS.md`
[^9]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^10]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^11]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^12]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^13]: ADR-0003, every random draw is keyed, never stateful, decision D2. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^14]: Testing Rules, a determinism test cannot tell correct from consistently wrong. `.claude/rules/testing.md`
[^15]: Findings register, FND-089. `docs/FINDINGS.md`
[^16]: Decisions register, DEC-044. `docs/DECISIONS.md`
[^17]: Testing Rules, a fixture supplies the input. `.claude/rules/testing.md`
[^18]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity, decision D4. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
[^19]: Decisions register, DEC-038. `docs/DECISIONS.md`
