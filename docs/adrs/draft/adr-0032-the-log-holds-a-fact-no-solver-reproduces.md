# ADR-0032: The log holds a fact no solver reproduces, never derived state

## Context

The engine writes a log each frame. A watcher reads the log to learn what
happened, and the thread-count test compares two logs byte for byte.[^1]

The question this record answers is what goes in it.

The engine holds two kinds of state that change during a frame. The first kind
changes because something chose: a unit took a resource from a tile, a tile
changed hands, a unit died. Nothing recomputes those from the previous frame,
because each depends on a keyed draw, on a command, or on a comparison that
picked one of several admissible outcomes.

The second kind changes because a solver ran. An influence field, a tile value
pass and a rate application are each a function of the previous state and of
the content parameters. Every solver in this project runs a fixed iteration
count, so it produces the same answer whenever it runs again from the same
input.[^2]

The pull toward logging the second kind is strong and it is not foolish. A
watcher that wants to see an influence field change would rather read events
than run the solver again. The field covers every cell of the world, so
recording it records the largest thing in the simulation on every frame, and
the log then grows with the world rather than with what happened.

No measurement of that growth exists on the target platform, and every cost
figure in this project is derived.[^3] The argument here does not need one: the
second kind is reproducible by construction, so recording it stores an answer
the engine can compute.

## Decision

### D1. The log holds a fact that no solver reproduces

An entry in the log reports something the engine could not derive again from
the previous state alone. A resource taken from a tile, a holder changed, a
unit spawned and a unit reaped are of this kind.

The test is not whether the change was interesting. It is whether running the
frame again from the previous state, with the same seed, would produce the
value without being told it.[^4]

### D2. A derived field writes no event

A solver that computes a field over the world writes nothing to the log. The
field is state, the reader may read the state, and a reader that wants the
history of the field runs the solver again from a state it holds.

This is what the log gives up, and the record states it plainly rather than
leaving it to be discovered. **The log names the cause of a change to a derived
field. It does not hold the arithmetic of it.**

### D3. A discontinuity of a derived field is a fact

A field crossing a threshold is not the field. The crossing changes a discrete
outcome, so a later frame branches on it, and no reader recovers which frame it
happened in by looking at the field.

A threshold crossing is therefore logged, and the field it crossed is not.[^5]

### D4. The log is cleared at the frame barrier

The log holds one frame. The engine clears it at the start of the next.

A reader takes what it wants before the next step. Keeping more than one frame
is a later addition, and this record neither makes it nor forecloses it: an
event is plain data, and applying one is pure, so a retained log is still
replayable.[^6]

## The alternatives this rejects

**Log everything that changed.** Every write to every field emits an event, and
the log is a complete account of the frame. The project rejects this because
the derived fields are the largest state in the world and they are reproducible,
so the log would grow with the size of the world on every frame rather than
with what the frame did.

**Log a derived field as one bulk event that carries an array.** This is the
same volume with fewer entries. The research raised it as an open question for
the trade network, and this record answers it: the flux is derived, so it is
not logged in either form.[^7]

**Log nothing, and let a reader diff two states.** A diff says what differs. It
does not say which unit took the resource or which faction took the tile, and
those are exactly the facts a watcher wants and no diff recovers.

**Keep the log across frames so that a reader never misses one.** The project
rejects this now because nothing asks for it, and D4 says what it would cost to
add. A record that promised retention would describe a capability nothing
invokes.[^8]

## Consequences

**You cannot audit a derived value from the log.** Answering why an influence
value or a tile value changed needs the solver, run again from a state. This is
the cost the record accepts, and D2 states it.

**A contributor adding a system must classify its output.** A system that
computes a field adds no event. A system that resolves a choice adds one. The
question is asked once, when the system is written, and getting it wrong is not
visible at review time.

**A reader that wants a frame must read it before the next one.** D4 makes the
log a frame-lifetime thing, so a control plane that steps twice and then reads
has lost the first frame.

**A threshold is a design decision with a log consequence.** D3 means that
moving a threshold changes what the log reports, so a threshold is part of the
observable interface and not only an internal constant.

**Nothing fails when this record is broken.** No check looks for an event
emitted by a solver. A reviewer reads the system and asks whether the value
could have been recomputed. This record is enforced by review alone, and a
reader should not assume otherwise.[^8]

## References

[^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^2]: ADR-0005, a solver runs a fixed iteration count, never a convergence test, decision D1. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
[^3]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^4]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^5]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
[^6]: ADR-0006, an event is plain data and applying it is pure, decision D2. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
[^7]: Report 11, resource and trade flow, the open questions. `docs/research/reports/11-resource-and-trade-flow.md`
[^8]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
