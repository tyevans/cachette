# ADR-0007: Content supplies a key vector, never a comparator

Status: Draft

## Context

Content decides how things are ordered: which target a unit prefers, which
job it takes, which of two sites wins a tie.

The direct way to express that is a comparison function supplied by content.
A comparison function is code. It can be inconsistent, it can order two items
differently depending on when it is called, and a sort given an inconsistent
comparison produces an order that depends on the algorithm.

The engine cannot check a supplied function for consistency.

## Decision

### D1. Content supplies a key, never a comparison function

A content author declares an ordered vector of key fields. The engine
extracts the key from each item and sorts by the key.

### D2. The last key field is a stable identifier

Every key field is an exact integer, and the final field of every key is a
stable identifier, so no two items ever tie.

### D3. The engine never calls content code from inside a sort

A sort runs on extracted keys only.

## Consequences

**Some orderings are harder to express.** An ordering that genuinely needs to
compare two items against each other, rather than score each item on its own,
must be re-expressed as a computed key. Some cannot be, and those orderings
are not available.

**The ordering is total and reproducible.** Because the last key field is a
unique identifier, the sort has exactly one correct output.

**The sort can be a radix sort.** An integer key permits an algorithm that a
comparison function forbids, so the restriction buys speed as well as
correctness.

## References

[^1]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/draft/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^2]: Report 04, the selector engine and verbs. `docs/research/reports/04-selector-engine-and-verbs.md`
