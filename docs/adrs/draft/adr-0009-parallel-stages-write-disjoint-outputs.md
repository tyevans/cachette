# ADR-0009: Parallel stages write disjoint outputs, because the memory model is weak

Status: Draft

## Context

The engine runs its work across threads. It must give one answer at any thread
count, and it must give that answer on a machine with a weak memory
model.[^1] [^2]

The usual way to accumulate across threads is a shared value guarded by an
atomic operation or a lock. It is wrong here for two separate reasons, and
either alone would be enough.

**The order of arrival is the thread order.** A shared accumulator receives
contributions in whatever order the threads reach it, and that order changes
with the thread count, with the load on the machine, and with nothing at all.
For an operation that is not exactly associative the value then changes too.
For one that is, the value is stable and anything derived from the order is
not: which thread wrote last, which contribution won a tie, what the partial
sums looked like.

**A weak memory model gives no ordering that nobody asked for.** On the
primary target, a write by one thread is not visible to another without an
explicit ordering, and a program that relies on seeing one is wrong in a way
that a strongly ordered development machine hides for years.[^3]

The alternative that removes both problems at once is to give each thread its
own output and to combine the outputs afterwards, in an order the data fixes.

## Decision

### D1. A parallel stage writes only to memory that no other thread writes

**Each thread of a parallel stage owns its output.** It writes to its own slot,
its own chunk of an array, or its own buffer. No two threads write to one
location, and no thread reads a location another thread is writing.

The inputs may be shared, because a shared read is safe and needs no ordering.
The outputs are partitioned.

This is the decision a reviewer can check. An atomic accumulator, a mutex
around a running total, or two threads appending to one vector is the
violation. What the value ends up as does not matter.

### D2. The outputs are combined in an order the data fixes, never in the
order they finished

**A stage that produces one answer from many outputs combines them in a fixed
order.** The slot index, the chunk position, or a sort key fixes it. Thread
completion order and work-stealing order fix nothing, and neither may be
read.[^4]

A combine step that is commutative still declares its order, because a reader
should not have to prove commutativity to know what the code returns.

### D3. The partition is derived from the data, not from the schedule

**Which thread gets which part is a function of the input and the thread
count, and of nothing else.** A chunk is a contiguous range chosen by
dividing the work; a slot is named by its index. A thread never claims the
next available piece, because then the partition depends on which thread was
free, and the partition is what names the output.

A dynamic schedule may be faster on an uneven workload. It is not available
under this decision, and a stage with an uneven workload divides differently
rather than dynamically.

### D4. A thread that fails reports through its own output

**A stage where a thread can fail collects the failures the same way it
collects the results, through the per-thread output, and decides afterwards.**
A thread does not write an error into a shared place, and a stage does not stop
its siblings.

The stage picks one failure to return, by the same fixed order that combines
the results, so the error a caller sees does not depend on which thread failed
first.

## Consequences

**A parallel stage costs the memory of its outputs.** One buffer for each
thread, not one shared buffer. That is a real cost and it is the price of the
answer not depending on the schedule.

**Nothing in the simulation needs an atomic operation.** A contributor
reaching for one has either found a stage that does not fit this shape, which
is a design question, or has taken the shorter path, which this record
refuses.

**A stage cannot balance itself.** Work stealing is the standard answer to an
uneven division and it is not available. A stage that divides badly divides by
a different rule instead.

**The rule is cheap to check and impossible to check for.** A reviewer can see
an atomic and a lock. A reviewer cannot see a thread that read a location
another was writing, because the compiler already refuses that in safe Rust.
The decision is therefore mostly enforced by the language, and the part that
is not is the part about ordering, which is D2.

**A whole-world sweep divides into chunks and not into a work queue.** Every
parallel stage in the engine takes this shape today: the tile update, the
movement intents, the admission segment table, and the level 1 rebuild. Each
writes its own chunk and each is joined in index order.

## References

[^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^2]: ADR-0008, the primary target is aarch64, decision D1. `docs/adrs/draft/adr-0008-the-primary-target-is-aarch64.md`
[^3]: ADR-0008, the primary target is aarch64, the consequences. `docs/adrs/draft/adr-0008-the-primary-target-is-aarch64.md`
[^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
