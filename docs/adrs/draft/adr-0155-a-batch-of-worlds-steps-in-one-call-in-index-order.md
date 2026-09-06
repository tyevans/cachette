# ADR-0155: A batch of worlds steps in one call, in index order

## Context

A learner needs many worlds. It plays one faction, it takes one decision at a
time, and it learns from the return of a whole game. One world gives one game
at a time, so a training run wants many worlds running together. A product
record asks for that learner.[^1] Two design documents work the shape out, and
each names this record.[^2] [^3]

The engine already allows many worlds. A process holds one interpreter and many
worlds, and no mutable process-wide state reaches simulated state.[^4] One
record already asks that the world interface stay compatible with stepping many
worlds in one call.[^5] Nothing states what that call must do.

The shortest implementation steps one world for each call, and lets Python loop
over the worlds. A contributor would reach for it, because it needs no new type.
Two facts refuse it. **The number of crossings would grow with the number of
worlds.** The boundary is meant to carry an instruction and an answer, and the
count of crossings is meant not to follow the size of the thing worked on.[^6]
A learner that plays a thousand worlds would then cross the boundary a thousand
times for one decision. **Python would own the parallelism.** The interpreter is
released for the whole step, so a Python thread pool over many worlds would
release and reacquire the lock once for each world, and the worlds would finish
in an order the operating system chose.[^7]

The next shortest implementation steps the worlds on a thread pool and returns
the results as each world finishes. It is faster to write and it is wrong. The
answer would then depend on which world finished first, and the determinism
record outranks every other constraint.[^8]

## Decision

**One call steps every world of a batch. The call writes each world's result
into the row that world's index names.**

### D1. A batch holds N worlds, and one call steps all of them

A batch type holds N worlds. One call steps every world of the batch. Sibling
calls read every world of the batch and act on every world of the batch, each in
one crossing.

Each call writes the result of a world into the row that the world's index
names. World zero is the first row of every returned array. A thread pool inside
the call may run the worlds in any order, and the order it ran them in reaches
no output.

The iteration order of the batch is therefore explicit and fixed by the index,
not by the schedule.[^9] The partition comes from the data, and each world
writes only its own row, so no two threads write one place.[^10] [^11]

A reviewer finds a violation when a result reaches a row by completion order,
when a returned array holds fewer rows than the batch holds worlds, or when the
batch reorders itself.

### D2. The interpreter is released for the whole batch call

Each batch call releases the interpreter once, around the whole call, and
reacquires it when the call returns. No Python code runs while any world of the
batch steps.[^7] The released region receives no interpreter token, and the
compiler enforces that.[^12]

One call therefore holds one release, whatever N is. A call that released for
each world would put a Python-visible point between two worlds of one batch
step.

A reviewer finds a violation when a batch call releases more than once, or when
anything inside the released region touches the interpreter.

### D3. A world in a batch is the same world a caller builds alone

A world in a batch is built from a seed in the same way a caller builds one
world. Its state hash after K ticks is the same hash it would have if a caller
had built it alone and stepped it K times. Membership of a batch changes no
field of a world and enters no hash.

**A batch is an arrangement of calls, and not a kind of world.** Nothing in the
world knows N. No world of a batch reads another world of the batch, and no
process-wide value crosses between them.[^13]

A reviewer finds a violation when a world holds a field that names its batch,
when a hash moves because N changed, or when one world of a batch reads
another. A test builds one world alone, builds the same seed inside a batch,
steps both the same number of ticks, and compares the two hashes.

### D4. Threads for one world and worlds for one call are both parameters

A world steps on T threads, and a batch holds N worlds. A call may give each
world T threads in turn, or give N worlds one thread each, or take any split
between them. This record fixes neither N nor T. Both are parameters that a
caller sets, and the answer is the same at every setting.[^8]

**The cost shape follows the ticks and the worlds.** The cost of a batch step
follows N times the cost of one frame at T threads. The cost of a batch read
follows N times the length the schema declares. No term follows the tile count
or the unit count, because every reader reads aggregates and every verb is
set-valued.[^14] [^15]

Every figure that would fill this shape is a reference table row, and every cost
figure of this project stays derived until the target platform measures it.[^16]
This record therefore states no figure. A local measurement misleads, because
the development machines and the target do not share a cache line width.[^17]

A reviewer finds a violation when a cost claim names a figure, or when a term of
the cost shape follows the population.

## The alternatives this rejects

**One call for each world.** Rejected because the number of crossings would then
follow the number of worlds, which is what the control plane record
forbids.[^6] It also puts the loop over worlds in Python, and the loop is the
thing Python must not do.

**A thread pool owned by Python.** Rejected because it releases the interpreter
once for each world rather than once for the batch, and because it gives the
results in completion order. It moves the ordering decision out of the engine,
where no test of this project can see it.[^8]

**Results in completion order.** Rejected because the answer would depend on the
schedule. The determinism record outranks the speed this would buy, and a sort
by index at the end is the same cost as writing by index in the first
place.[^9]

**One shared world with many factions playing.** Rejected because a training run
needs independent games, not one game with many seats. A shared world also
couples the seeds, so two samples would not be independent.

**A batch as a separate kind of world.** Rejected because a world that behaves
differently inside a batch cannot be replayed alone. The golden hash would then
depend on N.

## Consequences

**A learner makes one call for each decision, whatever N is.** The batch is the
unit the boundary carries, so a thousand worlds cost one crossing for a step,
one for a read and one for an action.

**The engine owns the parallelism over worlds.** The thread pool lives inside
the call. A caller sets N and T and reads no thread from Python.

**A world stays replayable alone.** A world taken from a batch replays from its
seed and its action log, and gives the hash the batch gave. That is a
consequence of D3, and not a second decision.

**The determinism tests reach the batch.** The thread-count test runs one tick
of one batch at more than one T, and compares the results byte for byte. A test
that ran at one thread count would prove nothing.[^18]

**A failure in one world must not lose the others.** A world that fails reports
through its own row, in the way a failing thread of a parallel stage
does.[^19]

**This record fixes no N and no T.** A caller that wants a recommendation reads
the reference tables, and a table row changes when a measurement changes
it.[^16]

## References

[^1]: PRD-0056, a learner plays one faction against the controllers. `docs/product/REGISTRY.md`
[^2]: Design, a learner plays one faction against the controllers, section 9. `docs/superpowers/specs/2026-09-05-reinforcement-learning-design.md`
[^3]: Design, one environment core serves every learning stack, section 5. `docs/superpowers/specs/2026-09-05-reinforcement-learning-interfaces-design.md`
[^4]: ADR-0047, many worlds live in one interpreter, decision D1. `docs/adrs/draft/adr-0047-many-worlds-live-in-one-interpreter.md`
[^5]: ADR-0047, many worlds live in one interpreter, decision D4. `docs/adrs/draft/adr-0047-many-worlds-live-in-one-interpreter.md`
[^6]: ADR-0040, Python is a control plane, not a data plane, decision D2. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^7]: ADR-0042, the interpreter is released for the whole step, decision D1. `docs/adrs/draft/adr-0042-the-interpreter-is-released-for-the-whole-step.md`
[^8]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^9]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^10]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^11]: ADR-0009, parallel stages write disjoint outputs, decision D3. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^12]: ADR-0042, the interpreter is released for the whole step, decision D2. `docs/adrs/draft/adr-0042-the-interpreter-is-released-for-the-whole-step.md`
[^13]: ADR-0047, many worlds live in one interpreter, decision D2. `docs/adrs/draft/adr-0047-many-worlds-live-in-one-interpreter.md`
[^14]: ADR-0096, cost follows the lattice, not the population. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
[^15]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decision D2. `docs/adrs/draft/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^16]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^17]: Target platform costs. `docs/reference/graviton-costs.md`
[^18]: Testing Rules, section 1. `.agents/rules/testing.md`
[^19]: ADR-0009, parallel stages write disjoint outputs, decision D4. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
