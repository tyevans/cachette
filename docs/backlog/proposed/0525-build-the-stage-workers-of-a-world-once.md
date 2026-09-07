---
id: 0525
title: Build the stage workers of a world once
status: proposed
created: 2026-09-07
implements: []
changes: []
creates: []
serves: [PRD-0056]
blocked-by: [BLK-007]
---

## Why

**A world's own step builds a thread for each parallel stage, and it does that
on every tick.** The engine opens a scope in many passes. Each scope builds one
thread for each thread the caller asked for, runs the pass, and joins them. A
tick therefore builds tens of threads for one world, whatever the thread count
is.

**This is measured, and it is most of the thread building the engine does.** A
fixed workload of twelve worlds at twelve batch workers and one thread for each
world was traced on a development machine. The batch owned about one build in
forty. The remaining builds all came from a world's own step. A finding holds
the counts and the command that produced them.[^1]

**The system time that started this line of work is still unexplained.** A
training run on the target platform spent a large share of the machine in the
kernel. Removing the batch's own thread builds moved that share by an amount
too small to read on a development machine. Whether the stage scopes explain
the rest is not established. It is a hypothesis with a count behind it and no
cost measurement behind it.

## Impact review

**This item is proposed, and refining it is the work.** The review below states
what a refinement must answer, and it does not answer them.

**Governed by.** ADR-0001 D1 holds that one binary gives one answer at any
thread count. ADR-0004 D1 holds that iteration order is explicit and never
comes from a thread. ADR-0009 holds that parallel stages write disjoint
outputs, and its rules on how a stage divides its work are what any shared pool
must keep. ADR-0071 D2 states that one pass takes no thread count at all, so a
change that gives every pass a shared pool must say whether it touches that
record.

**Creates.** A record, if the work proceeds. A pool that lives for the life of
a world is a choice a contributor could reasonably make otherwise, and where it
lives decides whether the core crate holds process-wide mutable state. ADR-0047
D2 forbids a process-wide value that crosses between worlds, so a refinement
must say whether the pool is per world or per process, and how that answer
respects it.

**Blockers.** BLK-007 governs every cost figure. **The measurement that would
justify this work does not exist.** The cost of a thread build on the target
platform is not measured, and the count of builds for one tick at the target
scale is not measured either. A refinement takes both before it changes code.

**Serves.** PRD-0056.

## What the work does

Decide whether a world can keep the workers its stages use, rather than build
them for each stage of each tick. Say where the pool lives, and how a stage
divides its work over it without taking its order from the schedule.

## What good looks like

The thread count of the process does not move across a tick. The state hashes
do not move. The determinism tests pass at every thread count. A measurement on
the target platform states what the change bought.

## What it does not do

It does not change what any stage computes, and it does not change how any
stage divides its work.

It does not change the batch. One item holds that, and it is complete.[^2]

## References

[^1]: Findings register, FND-638. `docs/FINDINGS.md`
[^2]: Backlog item 0524, keep the workers of a batch across the step. `docs/backlog/complete/0524-keep-the-workers-of-a-batch-across-the-step.md`
