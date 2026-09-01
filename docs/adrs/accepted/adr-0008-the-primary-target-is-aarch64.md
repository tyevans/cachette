# ADR-0008: The primary target is aarch64

## Context

The engine runs on a server. Which server is not a detail that a project can
leave until later, because three things the engine cannot change afterwards
follow from it: the width of a cache line, the strength of the memory model,
and the vector instruction set that a compiler may assume.

Development happens on x86-64 and on Apple Silicon. Neither is the machine the
engine runs on, and each differs from it in a way that misleads. Apple Silicon
uses a 128-byte cache line and the server uses 64, so a local measurement of
false sharing reports the wrong answer. x86-64 has a strong memory model, so a
program that is wrong on a weak one runs correctly on it for years.

A project that names no primary target gets the strongest assumptions of
whichever machine its contributors happen to own, and finds out which of them
were wrong on the machine that matters.

## Decision

### D1. The primary target is `aarch64-unknown-linux-gnu`

**The engine targets that triple, and every other platform is a development
target.** A development target must compile and must pass the tests. It is not
where a cost figure comes from and it is not what an assumption about the
hardware may be taken from.

The continuous integration checks that the core and the Python binding compile
for the primary target on every change. The viewer is excluded, because it
opens a window and links a C library that the check does not carry.

### D2. A cost figure from a development machine is labelled as one

**No figure taken on a development machine is stated as a cost of this
project.** A measurement on the target platform is the only kind that counts,
and the blocker that says none exists governs every figure in the
project.[^1]

A figure from a development machine may be recorded, in a commit message or in
a backlog item, and it must say where it came from. The comparison it supports
is between two versions of the same code on the same machine, never between
this project and a budget.

### D3. Vector code compiles for the baseline of the target, and the engine
holds no dispatch

The primary target has a vector instruction set in its baseline. **When the
engine gains vector code, that code compiles for the baseline and the engine
selects no implementation at run time.**

A run-time dispatch would mean two implementations of one calculation. Two
implementations of one calculation in a project that hashes its state each
frame is two answers, and the difference appears as a state hash that depends
on the machine.[^2]

**No vector code exists yet.** This decision is written so that a reviewer can
refuse a dispatch table when the first of it arrives, not because any exists
to refuse.

## Consequences

**A local performance measurement proves nothing about the project.** It can
show that one version is faster than another on one machine. It cannot show
that the engine meets a budget, and no such claim may be made from it.

**False sharing cannot be tested where the work is done.** The development
machines have a different cache line, so a layout that shares a line on the
target may not share one locally. That defect waits for a benchmark on the
target.

**A weak memory model is the assumption everywhere.** A stage that relies on
one thread seeing another's write without an explicit ordering is wrong on the
target and may pass on a development machine. Another record states the rule
that avoids the question.[^3]

**The project cannot take a dependency that does not build for the target.**
The compile check is on every change, so the cost of finding out is one
change and not one release.

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^3]: ADR-0009, parallel stages write disjoint outputs. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
