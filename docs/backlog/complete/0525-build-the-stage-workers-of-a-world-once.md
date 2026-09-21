---
id: 0525
title: Build the stage workers of a world once
status: complete
created: 2026-09-07
implements: [ADR-0001 D1, ADR-0004 D1, ADR-0009 D1, ADR-0047 D2]
changes: []
creates: []
serves: [PRD-0056, PRD-0002]
blocked-by: [BLK-007]
---

## Why

A world's step opens a new thread scope for each parallel stage on every
single tick. In a multithreaded run, each scope spawns worker threads, runs
the chunked pass, and joins them. Consequently, a single tick executes dozens
of thread spawns and joins, accounting for approximately 39 out of every 40
thread builds across the engine.[^1]

This per-stage thread turnover creates substantial operating system kernel
overhead and scheduler contention, which degrades simulation throughput and
RL training velocity.[^3] Retaining a persistent worker pool on the world
instance eliminates repeated thread creation while preserving deterministic,
disjoint chunk execution.

## Impact review

**Governed by.** ADR-0001 D1 mandates that one binary produces identical
results at any thread count. ADR-0004 D1 requires explicit iteration order
independent of thread completion. ADR-0009 requires parallel stages to write
disjoint outputs. ADR-0047 D2 strictly forbids process-wide mutable state that
crosses between world instances.

**Design decisions.**
1. The stage worker pool is owned by `World` and scoped to its lifecycle,
   satisfying ADR-0047 D2. No global or process-wide pool is used.
2. Single-threaded configurations execute passes directly on the calling
   thread without dispatch overhead.
3. Passes partition work into deterministic contiguous slices and dispatch to
   persistent workers via channel/barrier synchronization.

**Changes.** None to decision records.

**Creates.** None.

**Blockers.** BLK-007 governs cost figures on the target platform. Performance
improvements will be validated via the Graviton benchmark harness.

**Precedent.** FND-638 confirmed that 39 in 40 thread builds originate from
within a world's own step scopes rather than the batch harness. Item 0524
successfully retained batch workers across steps; this item brings the same
retention to intra-world stages.[^2]

**Conflict surface.** `crates/cachette-core/src/world/mod.rs`,
`crates/cachette-core/src/world/step.rs`, and
`crates/cachette-core/src/parallel.rs`.

## Done when

- `World` constructs a persistent worker thread pool sized to its configured
  parallelism at initialization.
- Parallel simulation stages (`influence`, `weather`, `rates`, `holding`)
  execute over the persistent worker pool without opening per-stage thread
  scopes.
- OS thread creations drop to zero during steady-state simulation stepping.
- Determinism tests pass byte-for-byte across 1, 2, and 12 threads.
- World state hashes match golden baseline values exactly.

## Outcome

The world now owns a persistent `StagePool` constructed at initialization or
retained on the first step. Parallel stages execute their disjoint chunks
across the persistent worker threads and calling thread without opening
per-stage thread scopes. Single-threaded configurations execute inline on the
calling thread with zero dispatch overhead. Operating system thread creations
drop to zero during steady-state simulation stepping. Thread equivalence tests
pass byte-for-byte across 1, 2, and 12 threads.

## References

[^1]: Findings register, FND-638. `docs/FINDINGS.md`
[^2]: Backlog item 0524, keep the workers of a batch across the step. `docs/backlog/complete/0524-keep-the-workers-of-a-batch-across-the-step.md`
[^3]: PRD-0056, a learner plays one faction against the controllers. `docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md`
