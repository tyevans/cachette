---
id: 0275
title: Size the panel picture to the panel
status: complete
created: 2026-09-03
implements: [ADR-0070]
changes: []
creates: []
serves: [PRD-0005]
blocked-by: []
---

## Why

**The picture that holds every number was cutting numbers off.** The recipe
that writes the panel to a file exists so that a person reads the whole panel
without a display. Its picture was a constant 1340 pixels tall, the panel had
grown past that, and the last sections fell off the bottom.

**The panel does say that it was cut.** It writes a notice as its last line.
That notice sits at the foot of the picture, which is the last place anyone
generating a picture looks, and nothing fails. So the recipe reported success
while losing the sections it exists to show.

**A constant cannot hold.** The panel grows with the faction count, with the
number of foundings, and with every section that a count switches on. A height
that fits today cuts tomorrow, silently, and the next person to add a section
has no way to know.

## Done when

- The picture asks the panel how tall it needs to be and resizes to it.
- No caller of the panel holds a height constant as the answer.
- The control plane's picture does the same, through the same reader.

## How it came out

**One list is the only statement of what the panel holds.** The line list was
already built in one place and then cut to the canvas. The build and the cut
are now separate, so a caller can ask for the height the whole list needs
while the drawing still cuts to the canvas it was given. Nothing is stated
twice.

**The answer can move, so the caller asks again.** A taller picture paints
more units, and a section that a count switches on adds lines, so the height
the panel asks for after a resize may be larger than the one it asked for
before. Both callers loop, bounded, and fail loudly rather than looping
forever if it never settles. Two passes settle it in practice.

**The measurement says how far off the constant was.** The picture now comes
out 1853 pixels tall against the constant's 1340. Raising the constant to 1500
had been tried and changed nothing visible, which is what made the height look
like the wrong explanation; it was the right explanation and the raise was too
small. An earlier item gave the panel a way to reach past the window, and this
one removes the constant that decided how far.[^1]

## References

[^1]: Backlog item 0133, the panel is longer than the window. `docs/backlog/complete/0133-let-a-watcher-reach-a-panel-longer-than-the-window.md`
