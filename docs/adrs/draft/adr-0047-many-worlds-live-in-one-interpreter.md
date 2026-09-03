# ADR-0047: Many worlds live in one interpreter

## Context

A user of this engine wants more than one simulation at once. A researcher
sweeps a parameter and runs the variants together. A developer compares a
change against the world it changed. Both want the runs in one process, because
one process shares the loaded module and lets one script collect every result.

Two shapes answer that want. The process can hold several interpreters, one for
each run, or it can hold one interpreter and several world objects.

The first is not available. The interpreter binding library does not support a
module loaded into a second interpreter, and adding support needs a redesign of
its interface rather than a fix.[^1] The standard library now makes a second
interpreter easy to reach from Python, so a user will try it and will meet an
import error rather than a clear refusal.

The second is available and it is nearly free, because the step already
releases the interpreter for its whole duration.[^2] Two worlds stepping in two
threads therefore run their simulation work at the same time.

The condition on the second shape is the part that is easy to lose. Two worlds
are independent only if the engine keeps nothing that both of them use and
either of them changes. That condition is invisible: a global counter, a cached
table or a registry filled at first use all compile, all pass a
single-world test, and all couple two worlds together.

## Decision

### D1. A process holds one interpreter and many worlds

A world is an object. A caller builds as many as it wants. Each holds its own
storage, its own configuration and its own random seeding, and no world reads
another.

Two worlds built from one seed and stepped differently diverge, and that is the
property a test can assert.

### D2. No mutable process-wide state reaches simulated state

Nothing that the simulation reads to produce a value is process-wide and
mutable. No mutable static, no registry filled at first use, and no cache that
a step writes.

**The engine holds one mutable process-wide counter and it is named here.** An
arena mints a process-wide identity so that a structure built from one arena
cannot silently answer a question about another. That counter is never read to
produce a simulated value, so it enters no state hash and no log. A record that
claimed the engine held nothing of the kind would be false, and a reader who
found the counter would stop trusting the record.[^3]

A table that is built once and never changed afterwards is not process-wide
mutable state under this decision. It may be shared, because sharing something
nobody writes couples nothing.

### D3. The bindings refuse a second interpreter, and the project says why

A module loaded into a second interpreter fails. The project does not pretend
otherwise, does not work around it, and states the limit in its documentation
so that a user who meets the error knows it is expected rather than broken.

The limit belongs to the binding library. When that library supports a second
interpreter, this decision is what a contributor revisits.

### D4. The world interface must stay compatible with stepping many worlds in one call

An operation that steps several worlds together is one crossing for many steps
and it is the shape the research audience wants. It is not built, and this
record does not build it.

What this decision does is forbid a world interface that would make it
impossible: a world may not require that it is the only world, and a step may
not assume it is the only step running.

## The alternatives this rejects

**One world for each process.** A caller runs several processes and collects
results across them. The project rejects this as the only answer because it
gives up the shared module, the shared script and the shared memory that make a
parameter sweep convenient. It remains available to a user who wants isolation,
and this record does not stop anybody using it.

**Several interpreters in one process.** The project rejects this because the
binding library refuses it. This is a rejection by circumstance rather than by
argument, and D3 says so.

**A global world, with the current world selected by a setting.** This is the
shape a single-world engine grows into, and it is what D2 forbids. The project
rejects it because it makes every world reachable from every other, and because
the coupling appears only when two worlds run at once.

**A shared cache keyed by world.** A process-wide table with one entry for each
world would let a system look up per-world data without the world being passed
in. The project rejects it because it is D2's failure with a key attached: two
worlds still contend on one structure.

## Consequences

**Everything a step needs is reached through the world.** A system cannot pull a
value from a process-wide place. That makes some signatures wider than they
would otherwise be, and that is the price.

**A contributor cannot add a lazily built global table.** This is the most
natural thing to add and the hardest to undo, which is why it is a decision and
not a guideline. Undoing one means finding every reader.

**Two worlds may step at the same time.** This follows from the interpreter
being released for the whole step, and it is the payoff of that record and this
one together.[^2]

**A user who reaches for a second interpreter gets an error the project did not
write.** D3 accepts that and answers it with documentation, because the project
cannot improve the message.

**Nothing checks D2.** No gate looks for a mutable static. A reviewer looks for
one, and the one that exists is named in D2 so that finding it is not read as
evidence that the rule lapsed.

## References

[^1]: Report 05, the Rust and Python boundary, sub-interpreters and multiple worlds. `docs/research/reports/05-rust-python-boundary.md`
[^2]: ADR-0042, the interpreter is released for the whole step, decisions D1 and D3. `docs/adrs/draft/adr-0042-the-interpreter-is-released-for-the-whole-step.md`
[^3]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
