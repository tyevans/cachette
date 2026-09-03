# ADR-0042: The interpreter is released for the whole step

## Context

The engine runs its simulation across threads and must give one answer at any
thread count.[^1] The control plane is Python, and a Python interpreter
serialises the code that runs under it.

A step that holds the interpreter for its whole duration blocks every other
Python thread in the process. A step that takes and releases it repeatedly pays
for each crossing and, worse, opens a window in which other Python code runs
while the simulation is part way through a frame.

The simulation crate cannot call Python, because it does not depend on the
interpreter binding.[^2] That settles what the simulation may do. It does not
settle what the bindings do around the simulation, and the bindings are where
the interpreter is held or released.

A build with no global interpreter lock does not remove the question. Such a
build still stops every thread to collect garbage, and a thread attached to the
interpreter that runs long native work delays that stop for everybody.

## Decision

### D1. The bindings release the interpreter once, around the whole step

The step method releases the interpreter before any simulation work begins and
takes it back after the work finishes. The release happens once for each step.

It never happens inside a loop over entities, over systems or over frames, for
the same reason the boundary is not crossed once for each entity.[^3]

### D2. The released region receives no interpreter token, and the compiler enforces it

The closure that runs the simulation may not capture an interpreter token,
because the token is not safe to send across threads and the release function
requires that it can be. The compiler refuses the capture.

This is the second mechanism, and it exists because one is not enough. The
crate split says the simulation cannot name Python.[^2] This says the region
around the simulation cannot carry Python either. The two catch the same
mistake from opposite directions.

### D3. A frame is a function of the state and the inputs that were fixed before it began

No input reaches the simulation after the step begins. Python cannot supply one,
because no Python code runs. Another Rust caller cannot supply one, because a
world is entered through one exclusive hold for the duration of the step.

This is the property that D1 and D2 exist to produce, and it is what the
thread-count test is comparing when it compares two logs.[^1] A frame that
could take an input mid-way would still pass a single-threaded test and would
give a different answer under load.

### D4. Events reach Python after the step, in one batch

A system never calls Python to report what it did. The step collects what
happened into the log, and the control plane reads the log after the step
returns.[^4]

Batched delivery is what makes D1 and D3 compatible with a program that still
observes the frame. Without it, the only way to see a frame would be a callback
inside it.

### D5. The release still happens on a build with no global interpreter lock

The release is not a workaround for one interpreter design. A thread that holds
the interpreter delays a collection pause for every other thread, whatever the
build. The bindings release in both cases and the code does not branch on which
build it is.

## The alternatives this rejects

**Hold the interpreter for the step.** This is what happens if nobody writes
the release. It is correct and it makes every other Python thread in the
process wait for the simulation. The project rejects it because running two
worlds in two threads is a stated goal of the interface.[^5]

**Release and take the interpreter around each system.** This would let a
control plane observe the frame as it runs. The project rejects it because
that observation is exactly the mid-step callback that the crate split makes
impossible, and because each crossing costs and the count would grow with the
number of systems.

**Release for the step and let a callback opt back in.** A caller would register
a function that the engine calls between systems. The project rejects it for
the same reason, and notes that it cannot be built without removing D2.

**Rely on the crate split alone.** The simulation cannot call Python already, so
a reader might conclude the release is redundant. It is not: without the
release the step holds the interpreter, and D3 would still hold while the goal
in the alternatives above would not.

## Consequences

**No Python code runs while a frame runs.** A control plane that wants to
influence a frame must decide before the step, and a control plane that wants
to observe one must read after it.

**A world is entered exclusively for the step.** Two Python threads may hold two
worlds and step both at once. Two Python threads may not step one world at
once; the second waits.

**The parallel work inside the released region touches no Python.** This is a
condition of the decision, not a consequence of it, and it is what the crate
split guarantees.[^2]

**A future feature that wants to interleave Python with a frame is foreclosed.**
Any such feature supersedes this record and ADR-0041 together, and it gives up
the property in D3.

**Nothing measures the cost of a release on the target platform.** One blocker
holds every cost figure this record would state, and it says which figures a
run has measured.[^6] The argument above is about how many times the boundary
is crossed, not about what a crossing costs.

## References

[^1]: ADR-0001, one binary gives one answer at any thread count, decisions D1 and D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^2]: ADR-0041, a crate split enforces the boundary at compile time, decisions D1 and D2. `docs/adrs/draft/adr-0041-a-crate-split-enforces-the-boundary-at-compile-time.md`
[^3]: ADR-0040, Python is a control plane, not a data plane, decision D2. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^4]: ADR-0032, the log holds a fact no solver reproduces, never derived state, decision D4. `docs/adrs/draft/adr-0032-the-log-holds-a-fact-no-solver-reproduces.md`
[^5]: ADR-0047, many worlds live in one interpreter, decision D1. `docs/adrs/draft/adr-0047-many-worlds-live-in-one-interpreter.md`
[^6]: Blockers register, BLK-007. `docs/BLOCKERS.md`
