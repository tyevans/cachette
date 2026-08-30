# ADR-0001: One binary gives one answer at any thread count

Status: Draft

## Context

This project simulates a world of many tiles and many units. It runs the
simulation in parallel across many cores.

A parallel program that adds results in the order the threads finish gives a
different answer on a different run. The difference is small at first. It
grows over many frames, because the output of one frame is the input of the
next.

A simulation that cannot repeat a run is a simulation nobody can study. A
researcher cannot defend a result. A developer cannot reproduce a defect
report. A save file cannot be trusted, because loading it gives a world that
drifts from the world that was saved.

Determinism cannot be added later. Every system must be written for it, and a
single system that ignores it destroys the property for the whole engine.

## Decision

**The engine produces the same result, byte for byte, for one binary at any
thread count.**

The scope is exact, and the scope matters.

- **One binary.** The same compiled artefact.
- **Any thread count.** One thread, two threads, or twelve give one answer.
- **Byte for byte.** The event log and the state hash match exactly. Not
  approximately, and not within a tolerance.

The engine does **not** promise a matching result across a different
processor, a different compiler, or a different version of a dependency.
Reproducing a run means keeping the binary, not only the seed.

This claim outranks every other constraint in the project. When a decision
would make the engine faster and break this claim, the claim wins. A record
that contradicts this one is wrong, whatever else it argues.

Two tests protect the claim, and both run in continuous integration.

1. **Thread-count equivalence.** The same tick runs at 1, 2 and 12 threads.
   The event logs are compared byte for byte. A mismatch names the first
   differing offset.
2. **The golden state hash.** The whole world is hashed each frame and
   compared against a stored file.

A test that cannot fail protects nothing. A build feature makes both tests
fail on demand, and continuous integration fails if either test passes while
that feature is on.

## Consequences

**The project cannot use a library that reassociates arithmetic**, however
fast it is, unless the reassociation is exact.

**The project cannot use a time budget** to decide how much work to do. Two
machines finish different amounts of work in the same wall-clock time.

**A performance change now needs a determinism argument.** This is a real and
permanent cost to the speed of development, and it is the price of the
property.

**A defect report becomes reproducible.** A seed and a binary reproduce the
run exactly, which is worth more than the cost above.

## References

[^1]: Report 03, the event log and determinism. `docs/research/reports/03-event-sourcing-and-command-flow.md`
[^2]: Report 07, target platform and value types. `docs/research/reports/07-target-platform-and-value-types.md`
[^3]: Decision Record Scope, the counter-test. `.claude/rules/adr-scope.md`
[^4]: Definition of Done. `.claude/rules/definition-of-done.md`
