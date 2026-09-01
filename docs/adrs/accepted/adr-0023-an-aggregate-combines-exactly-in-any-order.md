# ADR-0023: An aggregate combines exactly, in any order

## Context

A level 1 cell is the combination of the level 0 tiles it covers, and a level
2 cell is the combination of the level 1 cells it covers.[^1] The engine
therefore folds a set of values into one value, many times a frame, across
many threads.

A fold over a set has no order unless something gives it one. The engine
splits the work across threads, so two runs may combine the same values in
different groupings. A parallel fold and a serial fold must agree, and two
parallel folds at different thread counts must agree.[^2]

The requirement this places on the combine operation is stronger than it
looks. It is not enough that the operation be associative in the usual
informal sense. It must be **exactly** associative: combining `(a, b)` then
`c` must give the identical value to combining `a` then `(b, c)`, bit for bit.

Floating point addition fails this. It is associative to within a rounding
error, and a rounding error is exactly what a state hash compares. A pyramid
built on float sums drifts away from level 0 as the grouping varies with which
blocks are dirty. This is one of the four independent reasons the project
holds no floating point in simulated or aggregated state.[^3] [^4]

An incremental update asks for more still. To change one tile and repair the
summaries above it without rereading its siblings, the engine must remove the
old contribution and add the new one. Removal needs an inverse. A structure
with an associative operation and an identity is a monoid; a monoid with an
inverse is a group. **A monoid is enough to build the pyramid upward. It is
not enough to update the pyramid incrementally.**[^5]

## Decision

### D1. A summary field carries an exactly associative combine operation

**Every field of a summary declares a combine operation that is exactly
associative and has an identity value.** Exactly associative means that any
two groupings of the same values produce identical results, bit for bit, and
not to within a tolerance.

A field whose combination is not exactly associative is not a summary field.
It is a value computed at read time from fields that are.

### D2. The combine operation is commutative

**The combine operation gives the same result whatever order the values arrive
in.** Associativity alone permits an operation whose result depends on the
order of the operands, and the engine does not fix that order across a
parallel fold.

A field that needs an order carries the ordering key inside its value, so that
the combination reads the key rather than the arrival. The pair of a value and
a stable tiebreak key combines commutatively where the value alone does not.

### D3. The combine operation is exact integer or fixed-point arithmetic

**No combine operation uses a floating point type.** The arithmetic goes
through the arithmetic module, in the same way every other piece of simulation
arithmetic does.[^3] [^6]

An accumulator is wide enough that no sum it can hold overflows. A field
stored narrowly at level 0 is widened at level 1, because a narrow field
summed over the tile count of the target world exceeds a narrow
accumulator.[^7]

### D4. A field that has an inverse is declared as such, and only such a field
may be updated incrementally

**A summary field declares whether its combine operation has an inverse.** A
field with an inverse forms a group, and the engine may repair its summaries
by removing the old contribution and adding the new one.

**A field without an inverse is repaired by rebuilding from the level below.**
A minimum, a maximum and a bitwise union are associative and have identities,
and none of them has an inverse: knowing that a child left a set does not say
what the parent becomes.

A field that is derived at read time from group-valued fields inherits their
incremental update. A mean stored as a sum and a count is one such field, and
a mean stored as a mean is not a summary field at all.

### D5. The equality between a level and the level below is a test, not a
comment

**A test recomputes a cell from the level below and compares it with the
stored cell.** The two must be identical. The test runs at more than one
thread count, because a fold that agrees with itself proves nothing.[^2]

A property test states the algebra directly: the combination of a set is
independent of how the set was grouped, and independent of the order the
groups arrived in.

## Consequences

**Some statistics cannot be summary fields.** A median, a dominant value and a
distinct count are not exactly associative over their natural representation.
Each is available at read time from a field that is: a bucketed histogram
combines exactly and answers all three approximately.

**A field costs its accumulator width at every cell of every level.** The cell
count of the pyramid is small against level 0, so the width of the summary is
what the pyramid costs, not the number of cells.[^8]

**An incremental update is an optimisation with a proof obligation.** It must
give the answer a rebuild would give, and D5 is where that is checked. A field
that cannot meet it rebuilds, and rebuilding is always correct.

**A contributor cannot add a summary field by writing an accumulate
function.** The field must state its combine, its identity, and whether it has
an inverse, and a field that cannot state them is not a summary field.

## References

[^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^3]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^4]: Findings register, FND-001. `docs/FINDINGS.md`
[^5]: Findings register, FND-002. `docs/FINDINGS.md`
[^6]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^7]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^8]: Research report 02, the level of detail pyramid. `docs/research/reports/02-hex-grid-and-lod-pyramid.md`
