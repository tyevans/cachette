# ADR-0005: The log holds commands and facts, never derived state

**Status:** Draft
**Date:** 2026-08-30
**Depends on:** ADR-0001, determinism as the primary constraint.

## Context

Cachette simulates about 16.7 million tiles and about one million units. A
Python control plane issues commands. A Rust core runs the simulation. The
core runs many threads and must produce identical state at any thread count.

The root record makes two rules that this record builds on.[^1] First, an
event is plain data: `repr(C)`, declared padding, no `bool`. Second, the
function that applies an event to state is pure. The same prior state and the
same event give the same next state. That purity is what permits replay, and
it is what permits the engine to recompute derived state instead of storing
it.

This record fixes the shape of the log. It answers four questions. What does
the engine record? How does a parallel frame produce one ordered log? How does
a command that fails for part of a set report the failure? How does the engine
save and restore a world that is too large to copy inside one frame?

The scale sets the shape. A conservative load is 100,000 events for each
frame, and a heavy load is one million.[^2] At that rate the storage method is
not an implementation detail. It decides whether the frame budget survives.

## Decision

### ADR-0005 D1 — Classic event sourcing is rejected

Classic event sourcing stores one heap-allocated polymorphic object for each
fact. The engine does not use it. The reason is arithmetic, not taste.

**Allocation.** A matched allocate and free pair costs about 20 to 50
nanoseconds on a general allocator. At 100,000 events for each frame that is
2.0 to 5.0 milliseconds, against a frame budget of 16.6 milliseconds. The
allocator is also shared, so sixteen emitting threads contend on it.

**Cache misses.** A polymorphic handle is a fat pointer. To read the event the
apply loop follows the pointer to an arbitrary heap address. Each read is a
probable cache miss of about 80 to 100 nanoseconds. At 100,000 events that is
a further 8 to 10 milliseconds. The two costs together exceed the whole
frame.

**Dispatch.** Each virtual call is an indirect branch. The predictor cannot
predict it when event types are mixed. The indirect call also blocks inlining
and blocks vectorisation of the apply loop.

At one million events for each frame the method is about one hundred times
over budget. No tuning recovers that. The failure is structural.

### ADR-0005 D2 — Events live in type-segregated arenas of plain data

Each event type owns an append-only array of plain data. There is no
polymorphic container, and no single enumeration with a wide variant.

One push costs an amortised bounds check, a store, and a length increment,
which is 1 to 2 nanoseconds. The apply step becomes one tight loop for each
type, over a contiguous array, with no dispatch. That loop vectorises. The
method is 25 to 50 times cheaper for each event than the rejected one.[^2]

Reserve every arena at startup from a measured high-water mark. A frame then
allocates nothing. Give each arena a hard capacity, and report an overflow as
a rejected command, never as a panic. Because the event types have no
destructor, clearing an arena is one store to a length field.

Split a wide event type into parallel arrays only when a consumer reads a
subset of the fields. Most event types are 8 to 32 bytes and fit in half a
cache line, so one interleaved array is usually correct. Measure before you
split.

**The price of this decision is open extensibility.** Every event type is
named at compile time, or registered in a table at startup. A plugin cannot
introduce an event type at run time. For an engine with about thirty verbs
this is the correct trade.

### ADR-0005 D3 — The log holds commands and discontinuous facts only

The log records a command that entered the frame, and a fact that changes
state discontinuously. Examples of a discontinuous fact are a spawn, a death,
an ownership change, a construction completion, and a threshold crossing.

A discontinuous fact is one that no solver reproduces from prior state,
because it depends on a command, on a random draw, or on a comparison that
changes a discrete outcome.

### ADR-0005 D4 — A derived field update is never logged

Influence propagation, fog updates, weather, and trade flux are pure functions
of the prior state and of the content parameters. Both the prior state and the
parameters are already in the log or in the snapshot. Every solver runs a
fixed iteration count, so replay recomputes each derived field exactly.[^1]

This resolves the open question that the trade research raised.[^3] That
report asked whether the level 1 flux is one event for each arc, at up to 3.1
million events and 37.7 megabytes for each tick, or one bulk event that
carries an array. The answer is neither. The flux is derived state. The engine
records the commands and the discontinuous facts that changed the trade
network, and the solver reproduces the flux on replay.

**State the cost plainly. You cannot answer "why did this value change" from
the log alone.** An audit of a derived value needs the solver to run again
from the last snapshot. The log gives the cause of the change but not the
arithmetic of it. This is the accepted price of not paying 37.7 megabytes for
each tick to record work that is already reproducible.

### ADR-0005 D5 — Threads write to local buffers and the barrier concatenates them in a fixed order

Each worker appends to its own arena for each event type. No worker writes to
a shared arena, so no emit path needs an atomic operation.

At the frame barrier the engine concatenates the buffers by a stable key,
which is the worker index. It never appends in completion order.[^1]
Work-stealing completion order varies with cache state, with other load on the
machine, and with the core count. An output that depends on it fails perhaps
once in a thousand runs, which is the worst available failure mode.

Partition the work by data and not by time. A region maps to a worker by a
fixed rule, so the assignment does not change with the worker count.

### ADR-0005 D6 — Commands queue during the Python phase and seal at the barrier

No command enters a frame after the seal. The frame is then a pure function of
the sealed state and the sealed command set.

Every queued command carries a priority class, an issuer identifier, and a
monotonic sequence number assigned when it is queued. **The sort key is
priority, then issuer, then sequence.** That key is total and stable. It never
reads a thread identity, a clock, or a pointer address.

The issuer field is not optional. Plain issue order is well defined only when
one thread issues every command. A Rust system also queues commands, and a
later release may allow more than one Python thread. Two bytes remove the
ambiguity now, and no later change can add it cheaply.

**Internal commands use a separate queue.** The engine applies the external
queue first and then the internal queue. It never interleaves them. An
internal command may generate a further internal command, so the cascade
carries a bounded depth. The engine reports the bound as a rejection. An
unbounded cascade is a source of frame-time spikes and of non-termination.

Validation and application stay separate phases. A command handler reads state
and emits events. It never mutates state. This gives parallel validation
without locks, and it keeps the apply step pure.

### ADR-0005 D7 — Rejection reporting uses a closed enumeration, a count array, and a frame-stamped set

A set-valued command returns a summary. It does not raise an exception for
each failure, and it does not return a list of identifiers.

The result holds four things. The count of affected entities. The count of
rejected entities. An array of counts indexed by reason code. A handle to the
set of rejected entities.

**Reason codes are a closed enumeration of small integers.** A string reason
costs an allocation for each rejection. Python maps the code to a message.

**The rejected set is a bit set over the selected set, not a list of
identifiers.** A bit set over one million entities is 128 kilobytes, needs no
allocation, and chains directly into the next selector.

**The set handle carries a frame stamp and is valid for exactly one frame.**
Entities die. The engine rejects a stale handle with a clear error rather than
reading a freed row.

**A command is all-or-nothing for each entity, never for each field.** A
handler that partly applies to one entity and then fails leaves state that
replay cannot reproduce. Validate the entity fully, then apply it fully.

### ADR-0005 D8 — Snapshots copy dirty chunks, not the world

A full copy of the world takes about 27 milliseconds, which is longer than one
frame.[^4] Chunk-level copy-on-write, driven off the dirty bit set that the
summary pyramid already maintains, is therefore **mandatory and not an
optimisation**. This is a case where the number is the decision.

In a typical frame far less than one percent of tiles change, so the copy
costs well under one millisecond.

The chunks hold no pointers, so a snapshot is a byte copy and a restore is a
byte copy back. No traversal and no per-entity allocation occur. This is the
largest single benefit of the storage layout, and it is a reason for that
layout rather than a consequence of it.

The same mechanism is what a future rollback needs. A ring of recent chunk
deltas over one base snapshot gives a cheap restore to any recent frame. The
engine does not build the ring now. It does not foreclose it either.

### ADR-0005 D9 — The log stays transient, and retention stays additive

The engine discards the log at the end of the frame, after the export to
Python.[^5] Retention costs about 3.2 megabytes for each frame at the
conservative event rate, which is about 192 megabytes for each second and
about 11.5 gigabytes for each minute.[^2] That number, and not processor
cost, is the argument.

Retention buys rollback, time travel, and audit. None is in scope now.

Retention stays a later addition at no ongoing cost, because the events are
already serialisable plain data and the apply step is already pure. The
decision can be revisited against the storage figure above, and against a
measured compaction ratio. Column encoding of the event arrays is expected to
reduce the stored volume by three to ten times, because the arrays are already
columnar and the identifiers in one frame are near-sorted.[^2]

### ADR-0005 D10 — Three ideas from domain-driven design survive; the object model does not

Keep three ideas, because they cost nothing and they help.

**The ubiquitous language.** Name each verb and each event after the domain.
This makes the Python interface clear at no run-time cost.

**The command and event split.** A command is a request and it can fail. An
event is a fact and it cannot fail. This split is what makes the apply step
pure, and a pure apply step is what makes replay possible.

**Explicit invariant boundaries.** Write down what each region guarantees.
That statement is the proof that the parallel pass is safe.

Reject the rest, and be blunt about why.

**Aggregate roots.** A root object is a pointer to a graph. It forces pointer
chasing and it destroys the column layout that the engine depends on.

**Repositories.** A repository hides the storage layout. The storage layout is
the design. Hiding it removes the only thing that makes the engine fast.

**Domain services behind dynamic dispatch.** This is the same indirect branch
that ADR-0005 D1 rejects, in a different costume. Use static dispatch, or a
function-pointer table indexed by a small integer.

**Value objects that check an invariant on every write.** A wrapper type that
compiles away is fine. A check on every write blocks vectorisation.

**One aggregate for each entity.** This is the worst of them. It converts one
pass over an array of a million rows into a million transactions.

### ADR-0005 D11 — The aggregate is the region, never the entity

An aggregate is the smallest unit of state that one transaction locks and
keeps consistent. An entity is a row in an array. A row holds no method and
enforces nothing. So the aggregate is the region, which is a block of level 0
chunks, and the aggregate boundary is the parallelism boundary.

**This holds for a region-local invariant only.** Classify every invariant
when you write it, into one of three classes.

A **region-local** invariant is checked inside the parallel region pass. Tile
occupancy and local resource caps are of this class. This is the cheap case.

A **global scalar** invariant needs its own phase. A faction-wide unit cap is
of this class. Two regions each spawn a unit, each sees a valid local state,
and the global total then exceeds the cap. Check it in a serial reduction
after the parallel pass, or reserve from a budget before the parallel pass.
A reservation must follow the sorted command order and never the thread order.

A **cross-region** invariant, such as a unit that moves from one region to
another, needs a separate two-phase or serial pass after the parallel pass. It
must not run inside the parallel region pass.

### ADR-0005 D12 — The authoritative save format is hand-written

The save file holds the raw chunk bytes and a small header. The header carries
a format version, an endianness marker, and a checksum. The reader rejects a
mismatch with a clear error.

**The engine does not depend on any serialisation library's byte format for
the authoritative save.** The layout is defined in this project and the reader
is written by hand. A library version bump can then never invalidate a saved
world. This costs about two hundred lines and removes a whole class of future
failure.

A general serialisation library adds nothing inside the process, because the
chunks are already pointer-free and a byte copy is already correct. Such a
library belongs at the file edge for small metadata, and at a future network
edge.

## Consequences

### What this buys

An event costs 1 to 2 nanoseconds to emit rather than 20 to 50, and the apply
loop is a contiguous vectorisable pass rather than a chain of indirect calls.
The frame allocates nothing.

The log stays small, because the largest volume in the simulation, the derived
fields, never enters it. The trade subsystem writes commands and facts rather
than 37.7 megabytes of flux for each tick.

The frame is a pure function of the sealed state and the sealed command set.
Two tests then cover the whole parallel engine: a thread-count comparison of
the log, byte for byte, and a golden state hash.[^1]

A snapshot costs well under one millisecond and needs no traversal.

Rollback, time travel, and audit stay reachable later at no cost now.

### What this costs

**You cannot audit a derived value from the log.** To answer why a stock, an
influence value, or a fog value changed, you must restore a snapshot and run
the solver again. The log names the cause but does not hold the arithmetic.

Every event type is declared at compile time or registered at startup. There
is no run-time extensibility of the event set.

Every command carries a priority, an issuer, and a sequence number, and every
parallel merge concatenates by worker index. That discipline applies to every
emit path with no exceptions.

Every invariant must be classified into one of the three classes of ADR-0005
D11 when it is written. A misclassified global invariant fails silently under
parallel execution.

The save reader is project code and the project maintains it.

### What this forecloses

An event log that answers arbitrary questions about derived state without a
solver run.

A plugin that adds an event type at run time.

Any command handler that mutates state during validation, and therefore any
handler that reports failure by unwinding rather than by returning a summary.

Per-field partial application of a command to one entity.

A full snapshot inside one frame, at any scale near the target.

## References

[^1]: ADR-0001, Determinism as the primary constraint, decisions D6, D8, D9 and D11. `docs/adrs/draft/adr-0001-determinism-outranks-every-other-constraint.md`
[^2]: Research report 03, event sourcing, CQRS and determinism, sections 2, 5 and 8. `docs/research/reports/`
[^3]: Research report 11, resource and trade flow, open question OQ-E. `docs/research/reports/`
[^4]: Findings register, entry FND-020. `docs/FINDINGS.md`
[^5]: Decisions register, entry DEC-007. `docs/DECISIONS.md`
