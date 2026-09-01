# ADR-0005: A solver runs a fixed iteration count, never a convergence test

## Context

An iterative solver usually stops when the change between iterations falls
below a threshold. The number of iterations then depends on the values, and
with floating point it can depend on the order of the arithmetic.

A solver that stops on a time budget is worse. Two machines do different
amounts of work in the same time, so the answer depends on the machine.

## Decision

### D1. A solver runs a fixed iteration count

**Every iterative solver in the simulation runs a fixed number of
iterations.** The count is content, declared before the frame runs.

### D2. No solver stops on a convergence test or a time budget

No solver uses a convergence test to decide when to stop. No solver uses a
time budget.

### D3. A bounded search is bounded by a node budget

Where a search must be bounded, it is bounded by a node budget and not by
elapsed time. A node budget is a count, so it is the same on every machine.

### D4. A solver reports progress, it never reacts to it

A solver may report how far it got. It may not change how much it does in
response.

## Consequences

**The engine sometimes does more work than it needed to.** A field that
settles in three iterations still runs the declared count.

**The engine sometimes returns a less converged answer** than a convergence
test would have produced. The content author chooses the count, and choosing
it too low is a content defect that shows as a visibly wrong result.

**The frame cost becomes predictable**, because the work is known before the
frame starts. This makes a static schedule possible.

## References

[^1]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^2]: Report 06, algorithms and scheduling. `docs/research/reports/06-algorithms-and-scheduling.md`
[^3]: Report 13, the field operator algebra. `docs/research/reports/13-field-operator-algebra.md`
