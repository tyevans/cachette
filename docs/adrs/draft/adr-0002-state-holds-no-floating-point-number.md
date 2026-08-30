# ADR-0002: Simulated and aggregated state holds no floating point number

Status: Draft

## Context

Floating point addition is not associative. Adding the same three numbers in
a different order can give a different sum.

The engine sums values in parallel and combines the partial results. The
order in which the partial results arrive depends on the thread count and on
which thread finished first. A floating point sum therefore changes with the
thread count.

The engine also summarises the world into levels. A summary must equal the
sum of the parts it summarises. With floating point it does not, and the
error grows with the number of parts.

## Decision

### D1. State holds no floating point number

**No value in simulated state or in an aggregate is a floating point number.**

Values are integers, or fixed-point numbers held in integers. The fixed-point
scale is Q16.16: sixteen bits of whole part and sixteen bits of fraction,
held in a 32-bit integer, with a 64-bit accumulator above it.

### D2. All simulation arithmetic goes through one module

The module is the only place that defines the operations. A lint bans the
reassociating floating point operations everywhere else, including
`f32::algebraic_add` and its relatives.

### D3. An accumulator widens

A byte-wide field summed over millions of tiles overflows a 32-bit
accumulator, so an aggregate accumulates in 64 bits.

### D4. Floating point is allowed outside simulated state

It is allowed in rendering, in a statistic reported for a human to read, and
in a test that compares against a reference. None of those feed back into the
world.

## Consequences

**Content authors write integers.** A rate is stated in fixed-point units and
not as a decimal fraction. This is a real cost in ergonomics, and it is paid
once at the content boundary rather than every frame.

**Division needs care.** Q16.16 division loses precision differently from
floating point division, and the arithmetic module states the rounding rule
so that every call site rounds the same way.

**A physical formula must be re-derived in fixed point.** Some are harder in
fixed point, and one may prove impractical. That cost is accepted.

**An aggregate becomes exactly correct.** The sum of a region equals the sum
of its tiles, with no drift, which is what makes the upper levels usable as
an index rather than as an estimate.

## References

[^1]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/draft/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^2]: IEEE 754-2019, clause 5. https://ieeexplore.ieee.org/document/8766229
[^3]: Report 13, the field operator algebra. `docs/research/reports/13-field-operator-algebra.md`
