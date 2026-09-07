---
id: 0524
title: Keep the workers of a batch across the step
status: complete
created: 2026-09-07
implements: [ADR-0155]
changes: []
creates: [ADR-0191]
serves: [PRD-0056]
blocked-by: [BLK-007]
---

## Why

**The batch built a worker set for every tick and took it apart again.** The
step opened a scope, built one thread for each worker, joined them all, and
returned. A learner steps the batch once for each tick of a decision, and an
episode holds thousands of ticks, so the count of thread builds followed the
count of ticks.

A training run on the target platform reported that the machine spent a large
share of its time in the kernel rather than in the simulation. Thread builds
are kernel work, and the shape of the call made them look like the cause.

**Nobody had proved that.** The item was taken to remove the churn and to
measure what share of it the batch owned.

## Impact review

**Governed by.** ADR-0155 D1 holds that one call steps every world of a batch
and writes each result into the row that the world's index names. ADR-0155 D2
holds that the call releases the interpreter once, around the whole call.
ADR-0155 D4 holds that the worker count and the thread count are both
parameters of the caller. ADR-0004 D1 holds that iteration order is explicit.
ADR-0009 D1 holds that parallel stages write disjoint outputs. ADR-0001 D1
holds that one binary gives one answer at any thread count. ADR-0041 holds
that the core crate carries no interpreter binding, and the work stays in the
binding crate.

**Changes.** None. Every decision of ADR-0155 stands.

**Creates.** ADR-0191. The three-condition test passes for it. A future
contributor could reasonably build one pool for the whole process, or size a
pool from the parallelism of the machine. The choice reaches every training
process on a machine that runs several of them, and reversing it later means
changing the shape of the call. The reasoning is not visible in the code,
because the code only shows that the caller passes a count.

**Blockers.** BLK-007 governs every cost figure. The saving was measured as a
count of thread builds on a development machine, and no cost of it was taken
on the target platform.

**Serves.** PRD-0056.

## What the work does

The batch builds its workers when a caller builds the batch, and keeps them
until the batch goes. The worker count moves from the step to the constructor,
so it is declared once. The batch reports the count it built.

A step sends one order to each worker and reads one report from each. The
batch sorts the whole collection on the world index, which is the only thing
that orders a result.

## What good looks like

The thread count of the process does not move between two steps of one batch.
The state hashes of a run do not move. The order test goes red when a
test-only build removes the sort.

## What it does not do

**It does not remove the thread builds the batch does not own.** A world's own
step opens a scope for each parallel stage, and it does that once for each
tick. That is a separate item.[^1]

It does not change what any world computes.

## References

[^1]: Backlog item 0525, build the stage workers of a world once. `docs/backlog/proposed/0525-build-the-stage-workers-of-a-world-once.md`
