# ADR-0093: The window shows what changes, and the record of a moment goes to the inspection path

## Context

The viewer draws a world and reports what it drew. It reads the world through
the public interface and writes nothing to it.[^1] Every number it states comes
from a value the engine already holds, a value the viewer computed for itself,
or a count the drawing pass produced while it painted.[^2]

Those two records say where a number may come from. They do not say how many
numbers the window may show, and nothing did.

**The window filled with numbers until it could not hold them.** Each addition
was correct on its own. Each one named a real quantity, took its value from an
allowed source, and passed every check. Together they made a panel taller than
the window, which cut at the foot and stated that it had cut. The numbers below
the cut were not reachable at all, and among them was the naming of the colours
that the product record asks the window to give.[^3]

Two repairs were tried before this record. Ordering the sections so that the
most useful came first bought one placement and could not buy a second, because
the next section forced the same choice again.[^4] Making the panel scrollable
was recommended and was not built, because the project owner asked for
something else.[^4]

**The engine keeps growing, and the window does not grow with it.** The engine
now holds households, influence, descent, tile upgrades, ranked positions at a
site, and a closed loop from a survey to a production rate to a store to a
ration to a death. Every one of those is a candidate for a row. A window has a
fixed height, so the set of candidates grows and the space does not.

The project owner ruled on the shape: the window shows only what changes moment
to moment, drawn over the map, and everything else moves to paths that no
window height bounds.[^5]

## Decision

### D1. A quantity earns a place in the window only if it changes moment to moment

A watcher looks at a window to see a world running. A quantity that holds still
tells them nothing while they watch, and it takes space from the map.

**The test is what the quantity does, not how interesting it is.** The extent
of the world never changes. The seed never changes. A founding happens once,
before the first frame. None of those may sit in the window, however much a
reader wants them.

A quantity that changes only when the watcher moves the camera is a third case,
and it does not pass. It changes because the reader acted, not because the
world did.

### D2. A quantity the reader consults rather than watches goes behind one hold

Some quantities are neither continuous nor irrelevant. A reader consults them
to orient themselves, then goes back to watching. The naming of the colours is
one. The position of the camera is another.

**One mechanism serves all of them, and it holds no state between frames.** The
caller passes what the keyboard says, in the same way it passes the camera, and
nothing reaches the engine.[^1] A layer that is hidden by default takes no
space from the map, so a quantity behind it costs nothing while a watcher
watches.

A second mechanism for a second class of quantity is the failure this record
prevents. The panel had one class and grew; two classes with two mechanisms
would grow twice.

### D3. Every other quantity goes to a path that no window height bounds

The window is not the only reader of the world. Two paths hold what the window
does not, and each answers a different reader.

**A rendered picture answers a person.** One command draws the whole report to
an image file at a height that never cuts. It needs no display, so a machine in
continuous integration produces it and a reviewer attaches it to a change.

**A protocol server answers an agent.** An agent reads structured values and
cannot read an image.

The two paths are not alternatives to each other. A path that answers a person
does not answer an agent, and the reverse. A quantity that leaves the window
goes to whichever path has the reader who wants it, and to both when both do.

### D4. The window names the path that holds the rest

A number that is missing and says so is a number a reader knows to look
elsewhere for. A number that is missing in silence is the failure the panel
record already forbids.[^2]

**The window states, always and without a key, the name of the command that
writes what it does not show.** A window that dropped a section and said
nothing would leave a reader believing the project holds no such number.

The name in the window and the recipe in the build system are one fact in two
places, so a check compares them.[^6]

### D5. One reading feeds every layout

The window and the rendered report are two arrangements of one reading of the
world. The reading happens once, against the world and the finished canvas.

**No layout may read the world for itself.** A layout that did would produce a
number the other layout does not have, and the two would disagree about the
same tick with nothing to catch it.

This also bounds the cost. A layout is a function of a reading, so adding a
layout adds no pass over the world.

## Consequences

**The window can no longer answer every question, and that is the point.** A
developer who diagnoses reads the rendered report. A developer who watches
reads the window. The two activities were served by one artefact and are now
served by two.

**A quantity that stops changing must leave the window.** The rule is about
behaviour, not about subject matter, so a field that becomes static as the
engine matures loses its place. That is a review question at every change, and
no check can see it.

**A shipped product record needed amendment, and more will.** The record that
asks the window to state a set of things was written when the window held one
panel. It now describes a window and a command. A product record follows a
design rather than constraining it, and the register holds that ruling.[^5]

**The reference layer is a single point of growth.** Everything a reader
consults collects behind one hold, and nothing bounds how much collects there.
The hold hides it from a watcher, so the pressure that produced this record does
not apply, and the pressure that replaces it is weaker. A future reviewer should
expect that layer to be the next thing that grew too large.

**A person and an agent can disagree about the world.** Two paths hold the
record now, and nothing checks that they report the same value for the same
tick. Both derive from the engine's public interface, so neither can invent a
number, but neither is compared against the other.

### Alternatives rejected

**Make the panel scrollable.** This keeps every number in the window and makes
the ones below the fold reachable. It was the recommendation before the owner
ruled.[^4] It answers the reachability question and leaves the growth question
open: a panel that scrolls has no bound at all, so nothing ever has to leave
it. It also spends the map, which is the thing the watcher came to see.

**Fold each section behind its own control.** A reader opens the one they want.
This gives one control for each section, so the mechanism grows with the
content, and a reader must learn which control holds which quantity.

**Show the panel in a second window.** The two artefacts then differ by
position rather than by content, and both compete for the same screen. It also
needs a second window on a machine that may have no display at all, which the
rendered picture does not.

**Let the window grow to fit the panel.** This was tried. The demonstration
window was made taller to reach further down the panel, and the panel then grew
again. A window that follows its content has no bound, and a display does.

## References

[^1]: ADR-0067, the viewer reads the world and never writes to it. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^2]: ADR-0070, the head-up display reports what the drawing pass read. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
[^3]: PRD-0005, a watcher can tell what is happening and why. `docs/product/shipped/prd-0005-a-watcher-can-tell-what-is-happening-and-why.md`
[^4]: Decisions register, DEC-078. `docs/DECISIONS.md`
[^5]: Decisions register, DEC-085. `docs/DECISIONS.md`
[^6]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
