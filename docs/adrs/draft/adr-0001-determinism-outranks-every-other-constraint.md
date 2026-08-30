# ADR-0001: Determinism outranks every other constraint

**Status:** Draft
**Date:** 2026-08-30
**Depends on:** nothing. This is the root record.

## Context

Cachette simulates a world of about 16.7 million tiles and about one million
individual units. The engine runs headless on servers. A Python control plane
issues commands; a Rust core runs the simulation.

The project must choose which non-functional property outranks the others.
This record chooses determinism, and every other record follows from it.

**Determinism means the same inputs produce byte-identical state.** It is not
a feature. It is the property that makes five other things possible.

**Replay.** A recorded run reproduces exactly. A defect that appeared once
appears again.

**Debugging.** Without determinism, a defect at one million entities is found
by guessing. With it, the failing tick is reproduced and examined.

**Testing.** The engine can hash its whole state each frame and compare
against a stored file. It can run one tick at one thread and at twelve threads
and compare the results byte for byte. Neither test is possible without
determinism, and no other test covers a parallel simulation this size.

**Research use.** Reinforcement learning and agent-based modelling need
reproducible environments. This is the audience the project is best placed to
serve, and determinism is the entry fee.

**Rollback and network play.** Neither is in scope. Both become possible
later, and neither can be added to a non-deterministic engine at any price.

The cost of choosing determinism is paid at the start. The cost of adding it
later is a rewrite. That asymmetry is the argument.

## Decision

### D1 — The target is bit-exact for one binary, at any thread count

The engine produces identical state for the same inputs, on one compiled
binary, on one architecture, regardless of how many threads run.

Cross-platform bit-exactness is **not** a target. It is an architectural
option kept open, not a promise kept now.

This is the right target because the engine runs on a server fleet the project
controls.[^1] Development happens on other architectures, so a defect
reproduced locally may differ in its last bit from the same defect in
production. That is accepted.

### D2 — No floating point in simulated or aggregated state

Simulated state, aggregated state, and anything that enters a state hash uses
integer or fixed-point arithmetic. Floating point is permitted only in
rendering, in telemetry, and at the Python boundary for values that never
return to the simulation.

Four independent lines of research reached this conclusion.[^2]

**Exact associativity.** An aggregate must combine in any order and give one
answer. Float addition is not associative, so a float sum is not a monoid. A
summary pyramid built on float sums drifts away from its source as the
recombination order varies with which blocks are dirty. The drift is silent
and slow.

**Parallel fold order.** With integer accumulators, the order in which threads
combine partial results cannot change the answer. With floats it can, so the
schedule leaks into the result.

**Modifier composition.** A stack of multiplicative modifiers applied in a
different order gives a different number in floating point. Fixed-point makes
the order a schema property rather than a runtime hazard.

**Exact conservation.** Resource transport computes a flux once for each edge,
subtracts it at one end, and adds it at the other. In integer arithmetic the
pair cancels exactly, for any rounding rule. Conservation cannot fail. In
floating point it must be maintained by care.

The rule has repeatedly turned out to be a correctness win rather than a cost.
Kinship coefficients are dyadic rationals, so fixed point represents them
exactly to twelve generations. Integer addition and bitwise OR are exactly
commutative, so scatter-add needs no ordering. Saturating byte addition is
exactly associative, so influence aggregation needs no special case.

### D3 — Simulation arithmetic goes through one module, enforced by a lint

All arithmetic on simulated state routes through a `sim_math` module. A lint
rejects raw floating-point operations in simulation crates.

The lint must ban reassociating operations by name, including
`f32::algebraic_add` and its siblings, which stabilised in Rust 1.98 and
permit exactly the reassociation this record forbids.[^3]

This boundary cannot be retrofitted. Adding it later means auditing every
line.

### D4 — Fixed-point is Q16.16

One scale, project-wide. Sixteen integer bits and sixteen fractional bits in
an `i32`. Where range demands it, `i64` with the same fractional width.

A second scale was proposed and is rejected: its only purpose was keeping a
multiply inside 32 bits, and the target architecture runs 64-bit integer
arithmetic at full rate.[^4]

Pyramid accumulators are always `i64`. A one-byte tile field summed over 2^24
tiles reaches 2^32 exactly, which overflows a `u32`. The widening happens at
the first summary level, not the second.

### D5 — Random numbers come from a counter, not from state

Every draw is keyed on the tuple of system, frame, entity, and draw index. A
counter-based generator maps that key to a value.

Thread-local generator state is forbidden. It makes the result depend on which
thread served which entity, which is exactly what the schedule must not
control.

The mixer is written in-project with known-answer tests rather than taken from
a dependency, because the available crates are not maintained at a level this
warrants.[^5]

### D6 — Iteration order is explicit and stable

Every parallel reduction, every set iteration, and every traversal has a
declared order.

- Sort keys are total. Where a natural key can tie, an entity identifier is
  appended so no tie remains.
- Thread completion order is never used.
- Work-stealing order is never used.
- Hash-map iteration order is never used in simulation code.
- Hierarchical descent visits children in fixed index order.

### D7 — Commutative reductions need no order; the rest need slots

Integer addition and bitwise OR are exactly commutative and associative.
Scatter-add and scatter-or therefore give the same answer under any thread
order, and may use atomics freely.

Minimum, maximum, and first-wins do **not** have this property. They require
indexed output slots or a sorted reduction.

Prefer disjoint outputs over atomics regardless. The target architecture has a
weak memory model, so an atomic costs a real barrier where a stronger model
would not.[^1]

### D8 — Solvers run a fixed number of iterations, never a convergence test

Every iterative solver — diffusion, flow, relaxation, input-output — runs a
count fixed at compile time or from content.

A convergence test is a data-dependent branch. It makes the iteration count
depend on the data, which makes the result depend on timing under any
scheduling variation, and it makes cost unpredictable.

Where a solver might not converge, the fixed count is chosen from a proven
error bound, and failure to converge inside it is a modelled outcome rather
than a runtime condition.

The same rule governs search. A tree search takes a node budget, never a time
budget. A time budget makes the same tick explore fewer nodes on a busy
machine and produce a different answer.

### D9 — Events are plain data and applying them is pure

Every event type is `bytemuck::Pod`: `repr(C)`, explicit padding, no `bool`.
Use `u8` in place of `bool`.

Undeclared padding holds uninitialised bytes, which enter a state hash and
produce nondeterminism that has no cause in the simulation.[^6]

The function that applies an event to state is pure. Given the same prior
state and the same event, it produces the same next state, with no reference
to wall-clock time, thread identity, allocation address, or iteration order.

This is what makes replay possible, and it is what allows derived state to be
recomputed rather than stored.

### D10 — A comparator from content is a key vector, not a function

Content may not supply a comparison function. A content-authored comparator
can be intransitive, and an intransitive comparator makes sort output depend
on the sorting algorithm. Appending an identifier does not repair a cycle.

Content supplies an ordered vector of integer key extractors, with an implicit
final key of entity identifier. Totality and transitivity then hold by
construction, and the sort becomes a radix sort.[^7]

### D11 — Two tests protect this record

**Thread-count equivalence.** Run one tick at one thread, at two, and at
twelve. Compare the event log byte for byte. This is the highest-value test in
the project.

**Golden state hash.** Hash the whole simulation state each frame and compare
against a stored file. D9's padding rule exists so this test does not produce
false failures.

Both tests must exist before the first solver is written. A determinism defect
found by these tests takes minutes to locate. The same defect found by
observation takes days.

## Consequences

### What this buys

Replay, byte-exact. Reproducible defects. Two tests that cover a parallel
simulation no other test could. A research-grade environment. A path to
rollback and network play that stays open at no ongoing cost.

Several correctness properties arrive free, listed under D2.

### What this costs

Every numerical method must be expressed in integer or fixed-point
arithmetic. Most published numerical work assumes floating point, so a method
often has no integer literature and must be derived.[^8]

Fixed-point needs a declared scale, stated error bounds, and attention to
rounding bias. One such bias has already been found and fixed.[^9]

Solvers cannot adapt their effort to the data. A fixed iteration count is
sometimes wasteful and sometimes insufficient, and the bound must be proven
rather than observed.

The `sim_math` boundary is friction on every arithmetic line in the
simulation.

### What this forecloses

Adaptive numerical methods that vary their work with the data.

Any dependency that uses floating point in a path reaching simulated state,
including most numerical and machine-learning libraries. A learned policy may
run at a decision tier, but its output must quantise before it enters
simulated state.

Cross-platform bit-exactness is not foreclosed, but it is not delivered. The
remaining gap is transcendental functions supplied by the platform library.

## Notes

Absent literature is not impossibility. The project nearly abandoned a solver
on a citation search that found no integer formulation, then derived that the
integer form is exact and more reproducible than the float form.[^8] When a
method has no integer literature, derive it before abandoning it.

## References

[^1]: ADR-0002, Target platform and value types. `docs/adrs/draft/`
[^2]: Findings register, entries FND-001 and FND-006. `docs/FINDINGS.md`
[^3]: Rust 1.98 release notes, `f32::algebraic_add` stabilisation, 2026-08-20. https://blog.rust-lang.org/
[^4]: Research report 07, target platform and value types. `docs/research/reports/`
[^5]: Research report 03, event sourcing, CQRS and determinism. `docs/research/reports/`
[^6]: Research report 05, the Rust and Python boundary. `docs/research/reports/`
[^7]: Research report 14, character graph and inheritance. `docs/research/reports/`
[^8]: Findings register, entry FND-009. `docs/FINDINGS.md`
[^9]: Findings register, entry FND-012. `docs/FINDINGS.md`
