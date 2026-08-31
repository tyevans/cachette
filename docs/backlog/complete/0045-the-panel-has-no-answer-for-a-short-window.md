---
id: 0045
title: The panel has no answer for a window shorter than itself
status: complete
created: 2026-08-31
---

The panel's height follows its content. The content grows with the faction
count and with each section added to it, and nothing bounds it against the
window.

A canvas shorter than the panel cuts the bottom off. The drawing itself is
safe, because every write clips at the canvas edge, so nothing is corrupted
and nothing panics. What breaks is the claim: `bounds` states a rectangle that
the panel did not paint, and a caller that trusts the rectangle reads the
wrong answer.

The section that reports the region under the crosshair reached this. The test
window was 560 pixels tall and the panel grew past it, so the rectangle test
failed. The fixture now matches the height the demonstration binary opens,
which tests the panel rather than the gap.

Three options, and the impact review must weigh them.

**State the intersection.** `bounds` takes the canvas and reports the part of
the rectangle that fits. The claim becomes true and the content is still cut.

**Drop whole sections.** The panel omits a section that does not fit, in a
declared order, and says that it did. A reader then knows something is
missing.

**Scroll the panel.** The person moves through the content. This is the most
work and the only one that loses nothing.

The demonstration binary opens a window 720 pixels tall and the panel fits
inside it today, so nothing a person runs is wrong now. The gap is that
nothing fails when it stops fitting.

## Impact review

**Governed by.** ADR-0070 D2 says a number the panel cannot afford is absent
and never estimated, and that the panel labels what each number is so a reader
learns when it has none. A section cut off the bottom of the window is a
number the panel silently does not have, which is the failure that decision
exists to prevent. ADR-0067 D2 keeps the layout in the viewer.

**Changes.** No record changes. The work implements ADR-0070 D2 in a case that
decision did not name.

**Creates.** No record. A layout is not a constraint.

**Blockers.** None.

**The option chosen is a fourth one.** The list above has three: state the
intersection, drop whole sections, or scroll. The first leaves the reader
unaware. The third is the most work. The second is right in spirit, and the
panel already has the mechanism for it: one list of lines is the only
statement of what the panel holds, and both the height and the painting are
derived from that list.

**Drop trailing lines, and say so on the last one.** The readout learns the
height of the canvas, and the list it builds stops before it would overflow.
The height is summed from the shortened list, so the rectangle the panel
states is one it paints, and the claim in the drawing code stays true without
changing its signature. The last line says the panel was cut, so a reader
knows a number is missing rather than absent.

This is section-blind: it cuts at a line rather than at a heading. A section
half shown is still legible, because every row carries its own label, which is
the discipline ADR-0070 D2 already imposes.

## Outcome

The readout learns the height of the canvas, and the list of lines it builds
stops before it would overflow. The height is summed from the shortened list,
so the rectangle the panel states is one it paints, and the claim in the
drawing code holds without any change to its signature.

**The last line says the panel was cut.** A number that is missing and says so
is a number a reader knows to look elsewhere for. A number that is missing in
silence is the failure ADR-0070 D2 forbids for a number the panel cannot
afford, and a number below the edge of the window is exactly that.

**The cut is section-blind on purpose.** It stops at a line rather than at a
heading, so a section can be half shown. Every row carries its own label,
which is the discipline the record already imposes, so half a section is still
legible.

**One list stayed the only statement of what the panel holds.** The shortening
happens where that list is built, so the height and the painting cannot
disagree. Cutting at the drawing instead would have been one fact in two
places.

Two tests. One builds a window shorter than the panel and asserts the stated
rectangle shrinks and fits. The other repeats the rectangle test on a cut
panel, because the claim it checks is exactly the one the cut could break.
Removing the shortening fails both.
