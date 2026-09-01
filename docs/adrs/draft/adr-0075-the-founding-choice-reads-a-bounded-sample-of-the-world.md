# ADR-0075: The founding choice reads a bounded sample of the world

## Context

A run begins with a small group of people in one place. The engine chooses
that place. It does not take the place from a caller, because a place a caller
named says nothing about the world the seed made.

The obvious way to choose is to score every tile and take the best one. That
way is correct, it is simple to write, and a contributor will write it. It is
also the shape this project has already paid for once. The first level of the
summary pyramid was built as a pass over every tile, and the measurement that
followed showed that the term which grows with the number of things dominates
the term which grows with the number of tiles.[^1]

The cost lands before the first frame, where a developer sees it as the time
between changing a seed and seeing the new world. The product record refuses
that cost and asks for a founding that reads a bounded part of the world.[^2]

The ground is a pure function of the seed and the tile address, and the engine
stores no map of it.[^3] Nothing is therefore cheap to read in bulk. Each tile
the chooser looks at is a generator call, so the number of tiles the chooser
looks at is the whole cost of the choice.

The choice is also a comparison, and a comparison needs an order. Two places
that a score cannot separate must still resolve to one answer, on every thread
count and in every visit order.[^4]

## Decision

### D1. The number of tiles the founding reads does not depend on the tile count

The founding draws a fixed number of candidate places, and reads a fixed
number of tiles around each one. Both numbers are properties of the choosing
rule. Neither is a function of the world extent.

A world of a hundred tiles and a world of sixteen million tiles therefore cost
the same choice. The chooser reports how many tiles it read, so a test asserts
the property rather than describing it.

The alternative is a pass over every tile, which gives the best place in the
world rather than the best place in a sample. It is rejected because the cost
falls entirely before the first frame, and because the sample answers the need:
the product record asks that the place be chosen for a reason a watcher can
read, not that it be the maximum.[^2]

A second alternative is a coarse pass over a summary level, then a fine pass
inside the winning cell. It is rejected for now because the summary level is
derived from level 0 and a world that has not stepped has nothing to summarise
about its people. It is not rejected forever, and a later record may take it.

### D2. The candidate places come from the keyed generator, and the key holds the world seed

Each candidate address is a draw from the counter-based generator, keyed on
the tuple of the system, the frame, the entity and the draw index.[^5] The
candidate ordinal fills the entity slot. The column and the row of one
candidate take different draw indices, so the two coordinates never correlate.

A different seed therefore gives a different sample and a different place. The
same seed gives the same place at any thread count, because the key is the
whole input.

### D3. Every score in the choice is an exact integer or a fixed-point value

The score of a candidate is a weighted sum of quantities the world already
holds as exact integers. No step of the sum uses a floating-point type, and
every step goes through the arithmetic module.[^6] [^7]

An inexact score would make the comparison depend on the order the terms were
added in, which is the property this project cannot lose.

### D4. Two candidates that score the same resolve by the tile index

The chooser orders the candidates by a key vector. The first field is the
score, ordered from high to low. The last field is the tile index, which is
unique inside the sample, so no two keys tie.[^4]

The order is therefore total, and it does not depend on the order the
candidates were drawn in or joined in. A test constructs the tie rather than
waiting for one to occur.

### D5. The chooser reports the properties that made the place the choice

The founding returns the candidate it chose, the quantities it read at that
place, and the candidates it rejected with theirs. A watcher asks why the
place was chosen and gets the answer from the engine.

The report is the output of the choice, not a second copy of it. Nothing
recomputes a score to answer the question, so no copy can disagree with the
choice that was made.[^8]

## Consequences

**The founding does not find the best place in the world.** It finds the best
place in its sample. A caller who wants the global maximum cannot have it, and
that refusal is the point of this record.

**A founding can fail.** A sample may hold no place that admits the group. The
founding reports that refusal instead of widening the sample, because a sample
that widens until it succeeds is a pass over every tile with extra steps.

**The sample size and the neighbourhood size are tuning knobs with a floor.**
Either one may change. Neither may become a function of the world extent
without superseding this record.

**A test asserts the cost.** The chooser reports the tiles it read, so a test
compares two worlds of different extent and asserts the same count. This makes
the constraint checkable without a timing assertion.[^9]

**The place is chosen for what the ground holds.** The score reads the ground
and the stock the ground carries, both of which are generated from the seed.
A property that is not a function of the seed cannot enter the score, because
the world has not stepped when the choice is made.

## References

[^1]: Findings register, FND-049. `docs/FINDINGS.md`
[^2]: PRD-0012, a world starts small and grows. `docs/product/shaped/prd-0012-a-world-starts-small-and-grows.md`
[^3]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1, a draft record. `docs/adrs/draft/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
[^4]: ADR-0004, iteration order is explicit, decision D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^5]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^6]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^7]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^8]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^9]: Testing rules, section 3. `.claude/rules/testing.md`
