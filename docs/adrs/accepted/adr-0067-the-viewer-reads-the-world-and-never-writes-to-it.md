# ADR-0067: The viewer reads the world and never writes to it

## Context

The project needs a person to see the simulation. A developer cannot tell a
working engine from a broken one by reading a hash, and cannot show the engine
to anyone else.[^1]

A viewer is the first thing the project builds that is not the simulation. It
is therefore the first place where the rules that protect determinism meet
code that does not need them. Rendering wants a floating point coordinate, a
screen position, an interpolation between two ticks, and a frame rate. The
simulation permits none of those.[^2] [^3]

The danger is not that a viewer uses floating point. The record already allows
that outside simulated state.[^2] The danger is that a value computed for the
screen flows back into the world, or that the engine grows a field it holds
only because a viewer wanted it. Both are silent. Neither test the project has
would catch the second one.

The project must therefore state where the boundary sits, once, so that a
reviewer can find a crossing.

## Decision

### D1. The drawing and the reporting read the world through the public interface, and never write to it

Every function that draws the world, or that reports a number about it, holds
a shared reference. It calls no method that takes a mutable reference. It
never spawns, moves, or removes an entity, and it never advances a tick.

A drawing that could write to the world would put a person's choice of what to
look at into the simulated state.

**The program that owns the loop is not bound by this.** It builds the world
it is going to show, and it steps the engine before it draws, which D4
requires of it. The constraint is on the path from the world to the picture,
not on the program that owns both ends of it. A decision stated over the whole
crate would forbid the loop that another decision here requires.

### D2. The engine holds no value that exists for the viewer

The world holds no screen position, no colour, no camera, no zoom level, and
no frame count. It holds no field that exists because something draws it.

The viewer derives every such value from what it reads. A value that the
viewer needs and the world does not have is the viewer's to compute.

This is the decision a reviewer can check. A field named for a display, in the
engine, is the violation.

### D3. Floating point begins at the viewer boundary and never returns

The viewer may use floating point freely. Rendering is outside simulated
state, and the record that bans floating point allows it there.[^2]

No value that has been a floating point number may be passed back to the
engine, in any form. A viewer that converts a screen position to a tile
address must round to an exact integer before it names a tile, and the
conversion belongs to the viewer.

### D4. The viewer runs after the step, on the stepping thread

One loop steps the engine, then draws. The viewer does not run while the
simulation runs, and the simulation does not wait for a drawing to finish
before it can start the next step.

The project rejects the alternative for now, which is an engine on its own
thread publishing a frame that the viewer reads. That design honours the
product record's requirement that a viewer never slows the engine, and it
needs a snapshot mechanism that no record holds.[^4] Writing that record to
serve a demonstration is the wrong order.

**The consequence is that the drawing rate and the tick rate are the same
number.** This record states it rather than leaving it to be discovered. When
the project needs them separated, that is a new record, and this decision is
what it supersedes.

### D5. The viewer lives in its own crate

The viewer is a crate that depends on the core. The core does not depend on
the viewer, and it never will.

The dependency direction is what makes D2 checkable by a compiler rather than
by a reviewer: the core cannot name a type that only a viewer needs, because
it cannot see one.

## Consequences

**The engine gains nothing from having a viewer.** No field, no method that
exists only to be drawn. A viewer that needs something the world does not
expose is an argument for a public reader, not for a new field.

**A rendering defect is never a simulation defect.** The two determinism tests
do not cover the viewer, and they do not need to. A wrong picture of a right
world is a bug in one crate.

**The viewer cannot show what the world does not hold.** It cannot interpolate
between two ticks unless it keeps its own copy of the previous one, because
the world holds one tick at a time. Interpolation is the viewer's memory, not
the engine's.

**The rhombus is the viewer's problem.** The world is a parallelogram in the
index space, and the skew that puts it on a screen belongs here.[^5]

**The demonstration is bounded by the drawing rate.** D4 ties the two rates
together, so a slow drawing slows the simulation in the demonstration binary.
That is acceptable for a demonstration and unacceptable for anything else,
which is why D4 names what would supersede it.

## References

[^1]: PRD-0002, a developer watches the world run. `docs/product/shipped/prd-0002-a-developer-watches-the-world-run.md`
[^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D4. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^3]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^4]: ADR-0036, a snapshot copies dirty chunks, not the world. `docs/adrs/REGISTRY.md`
[^5]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D4. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
