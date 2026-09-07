---
id: 0519
title: Step a batch of worlds in one call, in index order
status: proposed
created: 2026-09-06
implements: [ADR-0155 D1, ADR-0155 D2, ADR-0155 D3, ADR-0155 D4]
changes: []
creates: []
serves: [PRD-0056]
blocked-by: [BLK-007]
---

## Why

**A training run steps many worlds, and the boundary offers one.** The only step
on the boundary takes one world and one thread count.[^1] An accepted record
says a batch of worlds steps in one call, that the call writes each world's
result at that world's index, that the interpreter is released for the whole
batch, and that the thread count for one world and the world count for one call
are separate parameters.[^2] [^3] [^4] [^5]

**Nothing implements it.** A survey searched the tree for a batch step and found
only three comments about an unrelated use of the word.[^6]

**The mechanism the record needs already works.** The single step releases the
interpreter for its whole run, which is what the second decision of the record
asks the batch to do for its whole run.[^3] [^6] The record adds a batch type
beside the world type, and it says plainly that a world in a batch is the same
world a caller builds alone.[^4]

**This is throughput and nothing else.** A single-world loop must run before
this is worth building, because a batch of a loop that does not close buys
nothing. It is independent of the observation and of the action table in kind,
and it is dependent on them in value.

**One status risk sits on the record and it is not a design risk.** The registry
row of that record names a dependency that is still a draft.[^7] [^6] Refining
this item must read that dependency and say whether the draft binds anything
here.

## What is missing before this can be refined

- **What a failing world does to the batch.** The record says in its own
  consequences that a world that fails reports its failure and the others
  finish.[^2] The work must decide the shape of that report: a status column
  beside the results, a raised error that names the index, or a result value
  that says nothing happened. A failure a caller cannot attribute to one index
  is the worst of the three.

- **How the determinism tests reach the batch.** The record says the
  thread-count test runs one tick through the batch.[^2] A test that steps one
  batch and compares it against itself proves nothing.[^8] The work must say
  what it varies: the thread count for one world, the world count for one call,
  or both, and it must show that the test can fail before it claims coverage.

- **Whether one call may hold worlds of different parameters.** The record says
  a world in a batch is the same world a caller builds alone, and it does not
  say whether two worlds of one batch must share a lattice.[^4] A batch of
  unlike worlds is the honest reading, and it costs more to schedule. The work
  must decide and say which.

- **What the batch costs against the single step, and at which world count.**
  The record fixes no world count and no thread count, and it says a caller that
  wants a recommendation reads a reference table.[^2] No row exists. One blocker
  says every cost figure of this project stays derived until the target platform
  measures it, and the development machines mislead on false sharing because
  their cache line differs from the target.[^9] [^10] The work must add the rows
  unset, express the cost parametrically, and measure on the target platform
  rather than locally.

- **Whether the batch owns a thread pool, and where it lives.** The record says
  the engine owns the parallelism over worlds and rejects a pool owned by
  Python.[^2] The work must say whether the pool is built for each call or held
  by the batch, because a pool built for each call pays its cost at every step of
  a training run.

## Done when

Filled in when the item moves to `refined/`.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: The binding crate of the project. `crates/cachette-py/src/lib.rs`
[^2]: ADR-0155, a batch of worlds steps in one call, in index order. `docs/adrs/accepted/adr-0155-a-batch-of-worlds-steps-in-one-call-in-index-order.md`
[^3]: ADR-0155, a batch of worlds steps in one call, in index order, decision D2. `docs/adrs/accepted/adr-0155-a-batch-of-worlds-steps-in-one-call-in-index-order.md`
[^4]: ADR-0155, a batch of worlds steps in one call, in index order, decision D3. `docs/adrs/accepted/adr-0155-a-batch-of-worlds-steps-in-one-call-in-index-order.md`
[^5]: ADR-0155, a batch of worlds steps in one call, in index order, decision D4. `docs/adrs/accepted/adr-0155-a-batch-of-worlds-steps-in-one-call-in-index-order.md`
[^6]: Research report 31, the state of the learner surface. `docs/research/reports/31-the-state-of-the-learner-surface.md`
[^7]: ADR Registry. `docs/adrs/REGISTRY.md`
[^8]: Testing Rules, section 1. `.agents/rules/testing.md`
[^9]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^10]: Project orientation, the target platform. `AGENTS.md`
