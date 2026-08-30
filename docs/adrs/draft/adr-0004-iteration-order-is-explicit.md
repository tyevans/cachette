# ADR-0004: Iteration order is explicit, and unordered reductions need slots

Status: Draft

## Context

A parallel loop finishes in an unpredictable order. A hash map iterates in an
order that depends on its internal state. Both produce a correct answer for
an operation that does not care about order, and a different answer for one
that does.

The distinction is not obvious at the call site, which is what makes it
dangerous.

## Decision

### D1. Iteration order is explicit

**Every iteration that feeds a result has an explicit, stable order.** No
result depends on thread completion order or on hash iteration order.

Two cases follow, and the difference decides how the result is combined.

### D2. An order-free reduction combines in any order

Integer addition and bitwise OR are order-free, because they are associative
and commutative on exact values. A parallel sum of integers needs no ordering
work.

### D3. A reduction that is not order-free needs a slot

Minimum, maximum, and first-wins all depend on order when values tie. Each
such reduction writes into a slot indexed by a stable key, and the combine
step reads the slots in index order.

### D4. A sort uses a stable key

A sort uses a stable key, never a comparison whose result can vary.

## Consequences

**A slot array costs memory.** A reduction that needs slots allocates one
entry for each parallel unit of work, and that is the price of the ordering.

**A hash map cannot be iterated to produce a result.** It may be used as a
lookup. Producing an ordered result from one needs an explicit sort.

**A reviewer can find a violation.** The rule is mechanical: if the operation
is not associative and commutative, look for the slots.

## References

[^1]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/draft/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^2]: Report 06, algorithms and scheduling. `docs/research/reports/06-algorithms-and-scheduling.md`
