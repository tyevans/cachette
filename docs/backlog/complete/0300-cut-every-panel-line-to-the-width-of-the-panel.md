---
id: 0300
title: Cut every panel line to the width of the panel
status: complete
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0003]
blocked-by: []
---

## Why

**Two of the panel's line kinds write past the right edge and nothing stops
them.** A row states a label on the left and a value against the right edge,
and the drawing cuts the value to the space that is left. A note and a founding
row do not. A note draws its whole text from the left margin, and a founding
row draws its whole value, so either one runs off the panel and over the map
when the text is long enough.

**This is a class and not an instance.** It was found when one note of 37
characters overflowed, against a longest existing note of 31. The 31 fits, so
nothing had failed yet. Every note in the panel today is under the bound by
accident rather than by a rule, and the next author of a note has nothing to
tell them what the bound is.

**The panel width is not a constant a writer can consult.** It follows the
glyph size and the panel width, so the safe length is a derived figure. A
comment stating "keep this under 31 characters" would be a second declaration
site for a value the drawing already computes, which is the first recurring
defect shape.[^1]

## Done when

- Every line kind the panel draws is cut to the width the panel has, including
  the note and the founding row.
- The cut happens where the drawing knows the width, not where an author counts
  characters.
- A test writes a note longer than any panel width and asserts that nothing is
  painted outside the panel.
- A test can fail. Put the uncut path back for one line kind and watch the test
  notice.

## What makes this hard

**A cut is not always the right answer for a note.** A row has a label and a
value, and cutting the value keeps the label, so the reader still knows what
was cut. A note is one run of text with no label, so a cut note may say
something different from what it meant. The work has to decide whether a note
wraps to a second line or is cut, and a wrapped note changes the height of the
panel, which the height calculation and the cut both read.

**The height and the drawing must agree.** One list states what the panel
holds, and the height, the cut and the painting are all derived from it, so a
wrap that changes a line's height has to change the height calculation in the
same place.[^2]

## What is already known

The overflow was confirmed by disabling one section rather than by reading the
code, so the cause is measured and not inferred. The note that overflowed was
shortened as a local repair. That repair is why nothing is visibly wrong today
and it is not a fix.

## Outcome

**The cut moved into one writer, and every line kind now goes through it.** The
writer takes a left edge and a right edge, cuts the text to the room between
them in whole glyphs, and returns whether it cut. A caller cannot ask it to
write past the right edge. The bound follows from the panel width and the glyph
table, so no author has to know a character count and no comment states one.

**The question about wrapping a note was answered by not wrapping it.** A
wrapped note changes the height of the line, and the height, the cut and the
painting are all derived from one list.[^2] A cut note is reported rather than
wrapped. The check that finds a line the panel must cut now covers every line
kind, so a note that would be cut fails a test instead of reaching a reader.

**One note was over the bound and nothing had failed.** It read "panel has no
count of the world." at 32 characters against 30 characters of room, and the
stored layout picture shows its ink two glyphs into the padding. It now reads
"panel counts no other tile." That is the local repair the item said was not a
fix, and it is correct here because the class closed first.

**The panel geometry now has one declaration site.** The width, the padding,
the line height, the value column and every colour moved into the panel
standard. The head-up display reads them from there, so no second copy can
drift.[^1]

**Two tests hold it.** One draws every line kind with a text many times the
panel width and asserts that no pixel outside the rectangle moved. The other
writes the same text through the bare canvas, which is the uncut path the panel
used to take, and asserts that the ink does escape. Without the second, a test
that only looked outside the rectangle would pass on a panel that drew nothing.

A finding records what the panel believed about its own cut.[^3]

## References

[^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
[^3]: Findings register, FND-321. `docs/FINDINGS.md`
