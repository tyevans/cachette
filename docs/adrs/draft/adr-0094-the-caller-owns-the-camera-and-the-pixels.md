# ADR-0094: The caller owns the camera and the pixels, and one command fills them

## Context

A drawing turns a world into pixels. Today the program that owns the loop owns
everything about that: it builds the window, it holds the camera, it allocates
the canvas, and it calls the drawing. A second program that wants a picture
must be a Rust program, must link the same window library, and must repeat the
loop.

**This project already has a rule for a boundary of this shape.** The control
plane builds a selector and sends one command. The core resolves the selector
and runs the verb. The control plane never loops over entities, because a
per-entity crossing costs the boundary once for each entity, and the crossing
is the expensive part.

A camera is a selector. It names the tiles a picture covers, without naming
them one at a time. A frame is the set-valued command over that selector, and
it is the case the rule was written for: a command over a whole set permits an
algorithm that a per-item loop cannot use.

**The drawing has no such boundary today, so there is nowhere for a second
caller to stand.** The choice is between giving the control plane a way to walk
tiles, which the rule forbids, and giving it a way to ask for a whole frame,
which the rule requires.

A separate question is which renderer fills the pixels. That question is open
and this record does not answer it. The two are independent, and this one comes
first: with the boundary inverted, a later renderer change is a change behind
the boundary, and no caller learns of it.

## Decision

### D1. A frame is one command

A caller asks for a frame once. The command carries the world, a camera, and
somewhere to put the result. It carries no tile, no unit, and no entity.

**A per-tile crossing of this boundary is the violation.** So is a per-unit
crossing, and so is a call that returns a list the caller then walks in order
to draw. A reviewer looks for a loop on the caller's side of the boundary. If
one exists, the frame is not a command.

### D2. The caller owns the pixels, and the engine writes into them once

The caller supplies the memory. The engine writes each pixel of one frame into
it and returns. The engine allocates no frame, keeps no frame, and holds no
reference to the memory after the call ends.

**This is not a read of the world, and it must not be reasoned about as one.**
The rule that governs copying governs data leaving the world: a value the
engine holds, copied on the way out to a caller.[^1] An output buffer is the
opposite direction. Nothing the engine holds is copied out, because a pixel is
not a thing the world has. The world holds no colour and no screen
position.[^2] The frame is made during the call and belongs to the caller
before the call begins.

**Nor is it a write to the world.** The record that forbids the drawing from
writing forbids it from changing simulated state.[^3] The caller's memory is
not simulated state, is not reachable from the world, and does not enter any
state hash. The drawing still holds a shared reference to the world and still
calls no method that takes a mutable one. **A reviewer checks the mutable
borrow, not the word "write":** the world is borrowed shared, the caller's
buffer is borrowed mutably, and the two are different objects.

### D3. The camera is the caller's

The engine holds no camera. It is given one for the length of a call and keeps
nothing of it afterwards, which is what the record that bans viewer values in
the engine already requires.[^2]

**The consequence is that a frame is a pure function of a world and a camera.**
That is the property worth having, and it is why this record does not split.
Inverting the buffer while the engine kept the camera would leave the function
impure in its most interesting argument: two calls with the same world would
give different pictures, and a caller could not say why. **The split was asked
for and refused.** No version of this decision that keeps the camera in the
engine is a version anyone should be able to accept.

A pure function is testable at any camera, and it gives capability the loop
could not: a scripted flight over a world, a view an agent steers, and a
picture taken at a named camera that another run can reproduce exactly.

### D4. The render core carries no window library and no display connection

The code that fills a frame depends on nothing that opens a window, nothing
that talks to a display server, and nothing that requires a display to exist.

**This is the constraint, and it is not the crate boundary that serves it.**
The crate that holds the demonstration window depends on a window library
today. A caller that reached the drawing through that crate would carry a
dependency on the display protocols of one operating system in order to fill
an array in memory. A package built for the control plane would carry it to
every machine that installed the package, including every machine with no
display at all.

A reviewer checks the dependency list of whatever crate holds the fill, not the
name of the crate.

### D5. One renderer feeds every presenter

Every caller that shows a frame is a presenter: it owns a buffer and a camera,
asks for a frame, and puts the result somewhere. A window puts it on a screen.
A file writer puts it on a disk. The control plane puts it wherever the control
plane wants.

**A presenter draws nothing itself.** A presenter that read the world and drew
part of a frame would be a second renderer, and two renderers of one world
disagree about that world with nothing to catch it. This extends the rule that
one reading feeds every layout from the numbers to the pixels.[^4] The register
holds an instance of what the numbers cost when that rule was missing.[^5]

### D6. The cost of a frame is bounded by the size of the frame, never by a number the caller chooses

The work one frame does follows the pixels in the buffer. It does not follow
the tiles in the world, and a caller cannot make it follow them.

**This constraint exists because the inversion creates the hazard.** Nothing in
the arrangement it replaces could reach it. The viewer built its own camera and
held the scale to a floor before anything downstream saw it. Once the camera is
the caller's, nothing holds it. The tiles a frame covers go as the area of the
frame divided by the area of a tile, so as the scale falls the work rises
without bound while the buffer stays the size it was. A caller that asks for a
hundredth of a pixel per tile buys a picture of a few pixels that sweeps every
tile the world has.

**That is not a wrong picture. It is an unbounded cost that the caller
chose.** A record that defines this boundary and leaves that open defines a
boundary that admits it.

**The bound is derived.** Below one pixel for each tile, more than one tile
falls on the same pixel, so the tiles beyond the first cannot change what the
frame holds. The work below that scale is provably invisible. The bound follows
from the pixel lattice and from nothing else, so a later contributor cannot
move it because a picture looked better.

**The verb refuses, and the refusal names the bound.** It does not hold the
scale to the bound quietly. A silent hold returns a picture that does not match
the camera the caller asked for, and a caller cannot tell that from a picture
that does. **A wrong answer presented as a right one is the failure this
project keeps removing**, and a refusal costs a caller one error and no
confusion.

**A frame is not permitted to answer from a level the caller did not ask
for.** Reading a summary level below the bound would substitute one level for
another without saying so, and the record that governs the levels forbids
exactly that: the level a reader read is part of the answer.[^6] So the verb
cannot quietly change what it reads either. It refuses.

## Consequences

**The control plane can show a world without linking a window.** A frame is an
array. What the caller does with the array is the caller's business, and the
engine does not know.

**A renderer change is invisible to every caller.** A renderer that runs on a
graphics device reads one frame back and writes it into the caller's memory.
The read-back is one screen of pixels, which is bounded by the window and not
by the world, and no caller's code changes. **This record makes that choice
reversible, and it is why the boundary comes before the renderer.**

**The contract must not forbid a frame that never enters main memory.** A later
variant may hand the caller a handle to memory on a graphics device instead of
an array. This record does not build that and does not describe it. It requires
only that the shape chosen now does not make it a breaking change.

**The engine gains a way to be wrong in public.** A frame is now part of the
interface, so its size and its pixel format are a contract, and a caller that
supplies memory of the wrong size must be refused rather than trusted. A
refusal is cheap. A write past the end of a caller's array is not.

**The loop that owns the window is still a program, and it is still bound by
nothing here.** It builds a world, steps it, holds a camera, and asks for a
frame. The constraint of this record is on the path from the world to the
pixels, not on the program that owns both ends of it.

**Refusing below the bound is not the end state, and this record does not
build the end state.** A picture of a whole world at a few pixels for each
region is a legitimate thing to want, and the project already holds the
structure that answers it: level 1 summarises blocks of tiles, and a summary is
a pure function of level 0.[^7] Reading level 1 below one pixel for each tile
is this project's own principle that a set-valued command permits a cheaper
algorithm, rather than a batched loop over the tiles. That is a second decision
and a second body of work, and an item holds it.[^8] A reader who hits the
refusal should find that item, not a dead end.

**A frame is not free, and this record does not make it cheaper.** The cost of
one frame follows the tiles the camera covers. A separate item halved that
cost, by giving the drawing a reader that takes the ground the drawing already
holds.[^9] Moving the boundary neither made that repair nor depends on it, and
a reader of this record must not take the move as a repair.

## Alternatives rejected

**Give the control plane a reader for a tile and let it draw.** This is the
per-entity crossing the project forbids. A frame of a hundred thousand tiles
would cross the boundary a hundred thousand times, and the crossing costs more
than the drawing.

**Let the engine own the buffer and hand back a reference.** The engine would
then hold a frame between calls, and a caller would hold a reference into the
engine. The lifetime becomes the engine's problem, the allocation grows with
the number of callers, and a second caller either shares one buffer or forces
the engine to keep a table of them. A caller that owns its own memory has none
of these.

**Keep the camera in the engine and invert only the buffer.** Refused in D3. It
gives up the property that makes the rest worth having.

**Send the control plane a compressed image instead of pixels.** This adds a
codec to the engine, spends time on every frame to save memory the caller
already has, and gives the caller a thing it must decode before it can show.
The caller wanted pixels.

**Wait for the renderer decision first.** The renderer question is open and may
stay open. Inverting the boundary now is what makes that question answerable
later without a second migration, so the order is deliberate.

## References

[^1]: ADR-0044, what copies and what does not is declared at the call site. The registry gives its status and its file. `docs/adrs/REGISTRY.md`
[^2]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^3]: ADR-0067, the viewer reads the world and never writes to it, decision D1. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^4]: ADR-0093, the window shows what changes, decision D5. `docs/adrs/draft/adr-0093-the-window-shows-what-changes.md`
[^5]: Findings register, FND-198. `docs/FINDINGS.md`
[^6]: ADR-0022, level 0 is the only truth and every level above it is derived, decision D4. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^7]: ADR-0022, level 0 is the only truth and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^8]: Backlog item 0239, draw a world too small for its tiles from level 1. `docs/backlog/proposed/0239-draw-a-world-too-small-for-its-tiles-from-level-1.md`
[^9]: Backlog item 0210, generate the ground of a drawn tile once. `docs/backlog/complete/0210-generate-the-ground-of-a-drawn-tile-once.md`
