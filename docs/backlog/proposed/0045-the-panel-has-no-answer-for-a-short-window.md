---
id: 0045
title: The panel has no answer for a window shorter than itself
status: proposed
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
