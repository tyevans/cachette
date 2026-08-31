# ADR-0070: The head-up display reports what the drawing pass read

Status: Accepted

## Context

The viewer draws a world. It draws a panel of numbers over that world. The
panel states the tick and the camera position. It states the counts of what is
on the screen. It states the cost of the last steps.

A separate record already fixes two boundaries.[^1] The viewer holds a shared
reference to the world. It never writes to the world. The engine holds no
field that exists because something draws it. A value the viewer wants is
therefore the viewer's to work out.

Neither boundary says what the viewer may spend to work a value out. That gap
is the subject here. It is not academic. Half of the numbers a panel wants are
counts of units. A count of units is a loop over units.

The reference tables give the unit count at the target scale.[^2] A count
written the obvious way reads every one of those units. It does that every
frame, to print a number in a corner.

The drawing does not work that way. It reads the engine's spatial structure.
It tests an occupancy bit for each block. It touches only the blocks the
window covers.[^3] A panel that scanned the population would therefore cost
more than the picture it labels.

Three properties make the failure silent.

- The scan is correct. It gives the right number every time.
- The scan is in the viewer, so no compiler and no crate boundary sees it.
- The demonstration binary steps and draws in one loop.[^4] The added cost
  therefore appears as a slower simulation, not as a slower panel. That binary
  is the one a developer uses to judge how fast the engine is.

The project has a record of this shape.[^5] A value is read back correctly and
costs more than it is worth. That is the recurring defect the rules name
first.

A future contributor will reach for the scan. It is the shorter code and it
works. The constraint must be written. Without it, a reviewer has nothing to
refuse the scan with.

## Decision

### D1. The panel adds no pass over the world

**Every number the panel states comes from one of three sources.** There is no
fourth source.

- A value the engine gives at once. The tick is one. The number of live units
  is another.
- A value the viewer computes from its own state. The camera position, the
  zoom and the extent of the window are all of this kind.
- A count the drawing pass produced while it painted.

The panel starts no loop of its own. It does not loop over the units. It does
not loop over the tiles.

A count of what is on the screen is incremented where that thing is painted.
The count therefore costs one addition on a path that already runs.

This is the decision a reviewer can check. A loop over the population, inside
the viewer's reporting code, is the violation. What the loop returns does not
matter.

### D2. A number the panel cannot afford is absent, never estimated

D1 puts some numbers out of reach. One of them is the number of units each
faction holds in the whole world. Nothing knows that number without reading
every unit.

The panel does not show such a number. It does not sample. It does not
extrapolate from the window. It does not present a figure from an earlier
frame as a current one.

The panel labels what each number is instead. A count of the window says that
it counts the window. A reader who wants a count of the world learns that the
panel has none. That is true, and a figure that is merely close is not.

An estimate is worse than an absence here. The panel exists so that a
developer can check the picture against a number. A number that is nearly
right defeats the only reason to show it.

## Consequences

**The panel's cost follows the window.** It grows with the pixels, not with
the world and not with the population. The drawing already grows that way.
D1 extends the same shape from the drawing to the reporting, so the two halves
of a frame cannot diverge.

**The drawing pass carries the counters.** The pass that paints a unit is the
pass that counts it. The two therefore cannot disagree. A second pass would be
one fact in two places, and nothing would fail when the copies differed.[^5]

**The panel is bounded by what the drawing already touches.** A number about
an off-screen part of the world is not available. D2 forbids inventing it.
Showing one needs a new engine reader, or a structure the engine already
maintains. It never needs a new engine field.[^1]

**A reader must read the labels.** The panel states counts of the window next
to counts of the world. Only the label separates them. That is the price of
refusing the estimate, and it is the smaller price.

### Alternatives rejected

**Scan the units each frame and report the world.** This gives the fuller
panel. Three reasons reject it. The cost grows with the population. The growth
is invisible in a loop that steps and draws together. The numbers it adds are
not the ones a person needs. A person watching a window asks about the
window.

**Keep a running census in the engine, updated as units spawn and move.** This
is cheap to read, and it is rejected outright. The engine would hold the field
because something draws it. The boundary record forbids exactly that.[^1] The
engine must gain nothing from having a viewer.

**Sample the population and scale the sample.** Rejected under D2. A sampled
count is an estimate. An estimate cannot be checked against the picture, and
checking against the picture is the whole purpose of the panel.

## References

[^1]: ADR-0067, the viewer reads the world and never writes to it, decisions D1 and D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^2]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^3]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D5. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^4]: ADR-0067, the viewer reads the world and never writes to it, decision D4. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^5]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
