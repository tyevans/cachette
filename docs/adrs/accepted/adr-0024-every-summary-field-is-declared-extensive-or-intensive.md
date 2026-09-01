# ADR-0024: Every summary field is declared extensive or intensive

## Context

A summary field combines the values of the cells below it.[^1] Two kinds of
quantity behave differently under that combination, and mixing them up gives
an answer that is wrong without being obviously wrong.

An **extensive** quantity scales with the extent it covers. A count of units,
a stockpile of grain and an area of forest are extensive. Combining two of
them adds them. A region that covers twice the ground holds about twice as
much.

An **intensive** quantity does not scale with the extent. A height above sea
level, a temperature, a rate of growth and a fraction of ground under forest
are intensive. Adding two of them produces a number that means nothing. The
right combination is an average, and an average must be weighted by the extent
each part covers, or the answer favours whichever part had fewer tiles.

The failure is quiet. Both mistakes produce a number of the right type, in a
plausible range, that varies plausibly as the world changes. A summed height
grows as the region grows and looks like a mountain range. An unweighted mean
of two cells, one covering four tiles and one covering four hundred, looks
like an average and is not one.

The engine cannot infer which kind a field is. A `u32` holding a count and a
`u32` holding a percentage are the same type, and the combination that is
right for one is wrong for the other.

There is a further trap. An intensive field cannot be stored as an intensive
value and combined, because a mean of means is not a mean. It must be stored
as the extensive parts it is computed from, and divided at read time. A field
declared intensive is therefore a statement about how it is read, and what is
stored is a pair of extensive fields.

## Decision

### D1. A summary field declares its kind where it is registered

**Every field of a summary declares itself extensive or intensive at the point
it is registered, and the declaration is not optional.** A field with no
declaration is a compile-time error, not a field that gets a default.

The declaration sits with the field, not with the code that reads it. A reader
that has to know which kind a field is has been given the wrong interface.

### D2. An extensive field combines by its own operation, and no weight is
applied

An extensive field combines by the operation it declared, which is exactly
associative and commutative.[^2] The engine applies no weighting, because the
extent is already inside the value.

### D3. An intensive field is stored as extensive parts and divided at read
time

**An intensive field is never stored as the value it reports.** It is stored
as the numerator and the denominator that produce it, both extensive, and both
combined by their own operations.

The division happens when a caller reads the field, and never before. A mean
is stored as a sum and a count. A fraction is stored as a part and a whole.

This is what makes the weighting automatic. A cell that covers four hundred
tiles contributes four hundred to the denominator, so the read of a combined
cell is weighted by construction and no separate weight exists to get wrong.

### D4. The denominator is the extent that the field is defined over, and not
always the tile count

A field defined over every tile has the tile count as its denominator. A field
defined over a subset does not: a mean elevation of dry land is a sum over dry
tiles divided by a count of dry tiles, and the count of dry tiles is itself an
extensive summary field.

The denominator is declared with the field. A field that borrows a denominator
it did not declare is the same defect as a field that declares no kind.

### D5. A division at read time is exact integer or fixed-point arithmetic

The read is arithmetic on simulated state, so it holds no floating point
number and goes through the arithmetic module.[^3] [^4] A division by a zero
denominator returns no value rather than a zero: a mean over no tiles is not
zero, and reporting it as zero is a false answer that a caller cannot
distinguish from a true one.

## Consequences

**A summary is wider than it looks.** An intensive field costs two accumulators
rather than one. That cost is real, and the alternative is a field that is
quietly wrong.

**A caller cannot write an intensive value.** There is nowhere to put it. A
system that wants to change a mean changes the tiles it is a mean of, which is
what the truth rule already requires.[^5]

**Two fields can share a denominator.** A mean elevation and a mean moisture
over the same extent divide by the same count. Whether the engine stores that
count once is a storage question, and this record does not settle it.

**A field whose kind is genuinely ambiguous is two fields.** A quantity that
is a total in one question and a rate in another is registered twice, once
each way, rather than converted at the call site by whoever remembers to.

## References

[^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^2]: ADR-0023, an aggregate combines exactly, in any order, decisions D1 and D2. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^3]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^4]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^5]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D3. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
