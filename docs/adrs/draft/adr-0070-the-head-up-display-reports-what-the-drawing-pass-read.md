# ADR-0070: The head-up display reports what the drawing pass read

Status: Draft

## Context

The viewer draws a world and a panel of numbers over it. The panel states the
tick, the camera position, the counts of what is on the screen, and the cost
of the last steps.

A separate record already fixes two boundaries. The viewer holds a shared
reference to the world and never writes to it. The engine holds no field that
exists because something draws it, so a value the viewer wants is the
viewer's to work out.[^1]

Neither boundary says what the viewer may spend to work a value out. That gap
is the subject here, and it is not academic. Half of the numbers a panel wants
are counts of units, and a count of units is a loop over units.

The engine holds one million units at the target scale.[^2] A count written
the obvious way reads every one of them, every frame, to print a number in a
corner. The drawing does not: it reads the engine's spatial structure, tests
an occupancy bit for each block, and touches only the blocks the window
covers.[^3] A panel that scanned the population would therefore cost more than
the picture it labels.

Three properties make the failure silent.

- The scan is correct. It gives the right number every time.
- The scan is in the viewer, so no compiler and no crate boundary sees it.
- The demonstration binary steps and draws in one loop, so the added cost
  appears as a slower simulation rather than as a slower panel.[^4]

The project has a record of this shape. A value that is read back correctly
and costs more than it is worth is the recurring defect the rules name
first.[^5]

A future contributor will reach for the scan, because it is the shorter code
and it works. The constraint must be written, or the reviewer has nothing to
refuse it with.

## Decision

### D1. The panel adds no pass over the world

Every number the panel states comes from one of three sources.

- A value the engine gives at once, such as the tick or the number of live
  units.
- A value the viewer computes from its own state, such as the camera
  position, the zoom, or the extent the window covers.
- A count the drawing pass produced while it painted.

The panel starts no loop over the units and no loop over the tiles. A count of
what is on the screen is incremented by the code that paints the thing it
counts, so the count costs one addition on a path that already runs.

This is the decision a reviewer can check. A loop over the population inside
the viewer's reporting code is the violation, whatever it returns.

### D2. A number the panel cannot afford is absent, never estimated

D1 puts some numbers out of reach. The number of units each faction holds in
the whole world is one of them, because nothing knows it without reading every
unit.

The panel does not show such a number. It does not sample, extrapolate from
the window, or cache a figure from an earlier frame and present it as current.

The panel labels what each number is instead. A count of the window says that
it counts the window. A reader who wants a count of the world learns that the
panel does not have one, which is true, rather than reading a figure that is
close.

An estimate is worse than an absence here. The panel exists so that a
developer can check the picture against a number. A number that is nearly
right defeats the only reason to show it.

## Consequences

**The panel's cost follows the window.** It grows with the pixels, not with
the world and not with the population. The drawing already grows that way.
D1 extends the same shape from the drawing to the reporting, so the two halves
of a frame cannot diverge.

**The drawing pass carries the counters.** The pass that paints a unit is the
pass that counts it, so the two cannot disagree. A second pass would be one
fact in two places, and nothing would fail when the copies differed.[^5]

**The panel is bounded by what the drawing already touches.** A number about a
part of the world that is off the screen is not available, and D2 forbids
inventing it. Showing one needs a new engine reader, or a new structure the
engine already maintains. It never needs a new engine field.[^1]

**A reader must read the labels.** The panel states counts of the window next
to counts of the world, and only the label separates them. That is the price
of refusing the estimate, and it is the smaller price.

### Alternatives rejected

**Scan the units each frame and report the world.** This gives the fuller
panel. It is rejected because the cost grows with the population, the growth
is invisible in a loop that steps and draws together, and the numbers it adds
are not the ones a person watching a window needs. A person watching a window
asks about the window.

**Keep a running census in the engine, updated as units spawn and move.** This
is cheap to read and it is rejected outright. It is a field the engine would
hold because something draws it, which the boundary record forbids.[^1] The
engine must gain nothing from having a viewer.

**Sample the population and scale the sample.** Rejected under D2. A sampled
count is an estimate, and an estimate cannot be checked against the picture,
which is the whole purpose of the panel.

## References

[^1]: ADR-0067, the viewer reads the world and never writes to it, decisions D1 and D2. `docs/adrs/draft/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^2]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^3]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D5. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^4]: ADR-0067, the viewer reads the world and never writes to it, decision D4. `docs/adrs/draft/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^5]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
