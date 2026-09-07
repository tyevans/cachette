# ADR-0191: The workers of a batch outlive the step

## Context

A batch holds many worlds and steps every one of them in one call.[^1] The
caller gives the batch two counts. One says how many worlds the batch steps at
a time, and the other says how many threads one world may give to its own
step. An earlier record makes both of them parameters of the caller, and it
fixes neither.[^2]

That record says nothing about how long a worker lives. The first
implementation built the workers inside the step and joined them at the end of
it, so one tick built a whole worker set and destroyed it. A learner steps the
batch once for each tick of a decision, and an episode holds thousands of
ticks, so the count of builds followed the count of ticks.

**A thread is not free to build.** The kernel maps a stack, it makes a task,
and it takes it apart again. That work is system time, and system time is time
the simulation does not get. The cost of one build is small, and the count is
what makes it matter.

A second force pushes the other way. A worker that lives across calls holds the
worlds between calls, so the batch must keep the world objects alive for the
whole life of the worker. A worker that lives also needs an answer for a world
that panics, because a panic that ends a worker would leave the batch waiting
for a report that never arrives.

A third force is the machine. A training run now starts one process for each
strategy, and several of those processes sit on one machine. A pool that reads
the core count of the machine would give each process the whole machine, and
the processes would then fight for it. The caller already knows how many
workers it wants, because it passes that count today.

The determinism record outranks every other constraint here.[^3] A worker that
lives across calls must not let the order of a result follow the schedule, and
the batch must give one answer at any worker count.

## Decision

**A batch builds its workers when a caller builds the batch, and keeps them
until the batch goes. The caller states how many workers there are, and the
batch reads nothing else.**

### D1. The workers live as long as the batch, and no step builds one

A batch builds its workers once. A step sends one order to each worker and
reads one report from each. No step builds a thread, and no step ends one. The
batch ends every worker when the batch itself goes.

The count of thread builds therefore follows the count of batches, and not the
count of ticks.

A reviewer finds a violation when a step builds a thread, when a step joins a
thread, or when a batch leaves a worker running after the batch goes. A test
counts the threads of the process between two steps and asserts that the count
does not move.

### D2. The caller states the worker count, and the batch reads no core count

The caller gives the worker count to the batch when it builds the batch. The
batch never reads the parallelism of the machine, and it holds no default that
stands in for the caller.

The batch builds one worker for each world when the caller asks for more
workers than the batch holds worlds. The batch reports the count it built, so
the caller can read what it got rather than assume it.

**The worker count is declared once.** It is not a parameter of the step as
well, because a step that carried a second count would have to reconcile it
with the pool, and one of the two copies would then decide nothing.[^4]

A reviewer finds a violation when the binding reads the core count of the
machine, when a step takes a worker count, or when the built count is not
readable.

### D3. The batch sorts on the index, and the sort is the only thing that orders a result

Each worker reports the results of the worlds it holds, and each result carries
the index of its world. The batch collects every report and sorts the whole
collection on that index.

**The batch reads the reports in a fixed worker order, and that order still
does not decide anything.** Each worker holds a strided run of indices, so the
collection is out of order until the sort puts it right. Nothing else fixes the
order.[^5] [^6]

A reviewer finds a violation when a result reaches a row by the order a worker
finished, or by the order the batch read the reports.

### D4. A world that panics reports at the call that caused it, and the worker lives

A worker catches a panic of a world it steps, and it reports the panic instead
of dying. The batch reads every report of a call before it acts on a failure,
so no report of one call reaches the next call.

A worker that dies anyway closes its own channel, and the batch then refuses
the call rather than waiting. Each worker owns a channel of its own, which is
what makes that refusal possible.

A reviewer finds a violation when a panic in one world blocks the batch, and
when a report of one call is read by another.

### D5. The batch serialises two callers that step it at the same time

The workers are shared, so two callers that stepped one batch at the same time
would read each other's reports. The batch takes a lock for the whole parallel
section, and the second caller waits.

This costs nothing in the intended use, because the control plane holds the
interpreter and steps one batch at a time.[^7]

A reviewer finds a violation when two callers can pair a report with the wrong
call.

## The alternatives this rejects

**Build the workers in the step, as before.** Rejected because the count of
thread builds then follows the count of ticks, and a training run holds
thousands of ticks in one episode. The batch already exists to stop a count
from following the size of the work.[^8]

**Take a thread pool library.** Rejected because the batch needs no work
stealing. The worlds are independent and the assignment is a fixed stride, so
there is nothing to steal and nothing to balance. A work-stealing scheduler
would also bring an ordering that the determinism record forbids as a source of
order, and the project would then depend on the sort to undo it.[^5] The
library would add a dependency tree to a crate that has few, and it would buy a
property this call does not use.

**One pool for the whole process.** Rejected because a training run starts one
process for each strategy. A process-wide pool sized from the machine would
give every process the whole machine. A process-wide pool sized by the first
caller would take its size from whichever caller ran first, which is a value
declared in a place nobody can find.[^4]

**A default worker count from the parallelism of the machine.** Rejected for
the same reason. The caller already passes the count, so a default would be a
second declaration site that silently outranks nothing and confuses everyone.

**Keep the worker count on the step and rebuild the pool when it changes.**
Rejected because it makes the count a parameter in two places, and because a
step that rebuilds a pool is the cost this record removes, hidden behind a
condition.

**Let a panic end the worker.** Rejected because the batch would then wait for
a report that never comes. A failure a caller cannot attribute to one index is
worse than a failure that stops the call.[^9]

## Consequences

**A caller that changes the worker count builds a new batch.** The count is
fixed for the life of the batch. The control plane already builds a new batch
when the set of running worlds changes, so this costs it nothing.

**The batch holds the worlds for as long as the workers live.** A world of a
batch is not collected while the batch holds it. That was already true, and it
now also holds for the workers.

**A batch is a resource, and it must go.** The workers end when the batch goes.
A caller that keeps a batch it does not use keeps its threads too.

**The saving is bounded by the share of thread builds the batch owns.** The
engine builds threads inside a world's own step as well, once for each parallel
stage. This record removes the builds the batch owns and none of the others, so
the saving is a share and not the whole of it. What that share is on the target
platform is a measurement, and the blocker that governs cost figures still
holds.[^10]

**The order test must still be able to fail.** The sort is the only thing that
orders a result, so a build that removes the sort must turn the order test
red.[^11]

## References

[^1]: ADR-0155, a batch of worlds steps in one call, in index order, decision D1. `docs/adrs/accepted/adr-0155-a-batch-of-worlds-steps-in-one-call-in-index-order.md`
[^2]: ADR-0155, a batch of worlds steps in one call, in index order, decision D4. `docs/adrs/accepted/adr-0155-a-batch-of-worlds-steps-in-one-call-in-index-order.md`
[^3]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^4]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
[^5]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^6]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^7]: ADR-0042, the interpreter is released for the whole step, decision D1. `docs/adrs/draft/adr-0042-the-interpreter-is-released-for-the-whole-step.md`
[^8]: ADR-0040, Python is a control plane, not a data plane, decision D2. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^9]: ADR-0009, parallel stages write disjoint outputs, decision D4. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^10]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^11]: Testing Rules, section 1. `.agents/rules/testing.md`
