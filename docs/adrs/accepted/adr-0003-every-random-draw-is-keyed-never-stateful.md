# ADR-0003: Every random draw is keyed, never stateful

## Context

A generator that holds state produces its next value from its previous value.
Two threads that share one generator race for it. Two threads that each hold
their own generator produce different streams depending on how the work was
divided.

Either way, the result depends on the thread count, which contradicts the
primary constraint.

## Decision

### D1. Every draw is keyed

**Every random draw comes from a counter-based generator, keyed on the tuple
of the system, the frame, the entity, and the draw index.**

The draw is a pure function of that key. It reads no state and it writes no
state.

The same entity, in the same frame, in the same system, drawing for the same
purpose, gets the same number however the work was scheduled. An entity that
is processed on a different thread gets the same number.

### D2. No thread-local random state exists

No thread-local random state exists anywhere in the simulation. A
thread-local generator is the single most common way a simulation loses
determinism, because it is correct on one thread and wrong on two.

## Consequences

**A draw costs more than reading a counter.** A counter-based generator does
real work for each value. The cost is accepted because the alternative is not
available.

**A system must name its draws.** A system that makes two draws for one
entity in one frame must distinguish them by index. Reusing an index gives
the same number twice, which is a defect that a test cannot easily see.

**Replay needs no random state in the save file.** The key reproduces the
draw, so a snapshot stores no generator state.

## References

[^1]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^2]: Report 03, the event log and determinism. `docs/research/reports/03-event-sourcing-cqrs-determinism.md`
