---
id: 0526
title: Shard one generation across worker processes
status: complete
created: 2026-09-07
implements: [ADR-0001 D2, ADR-0155 D1]
changes: []
creates: [ADR-0192]
serves: [PRD-0056]
blocked-by: []
---

## Why

A training run of one strategy uses a fraction of the target machine. The
section between two decisions runs in one interpreter, and every engine worker
of that process waits for it. More processes fill the machine and more workers
inside one process do not. Running one process for each strategy gives parallel
experiments, and it does not make one experiment faster.

## Impact review

**Governed by.** ADR-0001 D2 forbids a result that takes its order from which
worker finished first, so the combination of the shard scores must sort on a
stable key. ADR-0155 D1 gives the batch its index order, and a shard keeps that
order inside itself. ADR-0154 owns the observation and action layouts, and this
work states none of its own. ADR-0040 keeps Python a control plane, and a
worker process still sends one action integer for one faction.

**Changes.** None. No record changes.

**Creates.** ADR-0192, a generation is scored in shards and combined in
candidate order. The registry row is allocated.

**Blockers.** BLK-007 governs every cost figure. The two throughput figures in
the target register are measurements the project owner took on the target
platform, and the work states no figure of its own.

**Precedent.** Recurring defect shape 1 forbids deriving the process count or
the worker count from the core count, because the caller already states both.
The testing rule requires a determinism test with a proven failure mode.

**Serves.** PRD-0056, a learner plays one faction against the controllers.

## Done when

- A run states how many worker processes score one generation, and how many
  engine workers each process gives its batch. Neither comes from the core
  count.
- A worker rebuilds its candidates from the run seed and the generation
  number. No candidate policy crosses to a worker.
- The combined result sorts on the candidate index.
- A run at one shard and a run at four shards reach the same weights from one
  seed, position for position.
- A perturbed combination order moves those weights, so the comparison above
  can fail.
- A shard that dies ends the generation and says so.
- Each worker process holds the matrix libraries to one thread.

## Outcome

Done. The trainer opens a pool of worker processes when the shard count is
above one, and it scores the generation in this process when it is one. The
worker receives the centre, the generation number and the pair range it owns.
The equivalence test ran at one, two and four shards and the weights matched
exactly. The order perturbation moved them, and the fault switch ended the run.

The combination refuses a shard set that does not cover the population, so a
lost shard cannot train on a subset in silence.

No register row opened or closed. One measured section was added to the target
platform register, from figures the project owner took.
