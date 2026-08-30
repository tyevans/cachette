# ADR-0006: An event is plain data and applying it is pure

Status: Draft

## Context

The engine hashes the whole world each frame and compares the hash against a
stored value. A hash reads the bytes of a structure, including any byte the
compiler inserted as padding and never wrote.

Uninitialised padding holds whatever was in that memory. The hash then
differs between two runs that are otherwise identical, and the test reports a
determinism defect that does not exist.

A boolean has a further problem. Its representation permits values that no
correct program produces, and reading one is undefined behaviour.

## Decision

**Every event type is plain data with a declared layout.**

- The type uses the C representation, so the field order is fixed.
- Padding is declared as an explicit field, so every byte is written.
- No field is a boolean. A single-byte integer carries the same information
  with a defined representation for every value.
- The type satisfies the plain-data traits, so it can be viewed as bytes
  safely.

**Applying an event is a pure function of the event and the state it
touches.** It reads no clock, makes no allocation that affects the result,
and calls no code outside the engine. In particular it never calls Python.

This claim covers any type whose bytes are hashed or written to the log, not
only types named "event".

## Consequences

**Adding a field to an event means recounting the padding.** This is
mechanical and a test catches it, because the size and alignment are asserted.

**An event cannot hold a pointer or a growable collection.** Variable-length
material goes in a side arena and the event holds an index into it.

**A state hash means what it says.** A mismatch is a real difference in the
world and never an artefact of unwritten memory.

## References

[^1]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/draft/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^2]: Report 03, the event log and determinism. `docs/research/reports/03-event-sourcing-cqrs-determinism.md`
