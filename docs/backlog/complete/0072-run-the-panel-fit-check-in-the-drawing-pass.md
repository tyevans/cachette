---
id: 0072
title: Run the panel fit check in the drawing pass
status: complete
created: 2026-08-31
serves: [PRD-0002]
---

The panel cuts a value that does not fit its column. A cut value states a
number other than the one it was given, and it does so in silence.

The viewer holds a check that finds a cut value. One test calls it. Nothing
in the drawing path calls it, so no run ever asks whether it cut something.
The check has a proven failure mode and a fixture chosen for the case, which
is more than most checks have. It is still a capability that only a test
invokes, and the recurring defect rule names that shape.[^1]

The test walks one world through twenty-four zoom steps. A row whose value
grows with something that world does not reach would overrun in a run and
pass in the test.

Two shapes would close it. The drawing pass can refuse to paint a value it
must cut, and paint a marker instead. A debug assertion can fire on a cut.
The first is visible to the person watching, which is what the panel is for.

Refine this against the head-up display record.

## Outcome

**The drawing asks on every line of every frame, and it paints a mark when the
answer is yes.** The item named two shapes. This takes the first, because it is
visible to the person watching, and a debug assertion would fire only in a
build nobody runs the demonstration in.

The check is no longer a separate function that something must remember to
call. **The cut and the check are now one act.** The writer that puts text on
a panel returns whether it cut, and it paints the mark itself, so no drawing
path can cut a line without saying so. There is nothing left to forget.

The mark is a small block in one colour, against the right edge of the line it
belongs to. The cut text stops short of it, so the mark never sits over a
glyph. The colour is in the panel's ink key, so a stored layout picture shows
the mark rather than an unknown character.

**The item's concern about the fixture is answered by removing the fixture from
the path.** It said that a row whose value grows with something one test world
does not reach would overrun in a run and pass in a test. That is still true of
any test. It no longer matters, because the run itself now reports the cut.

Two tests hold it. One draws a panel whose every line is longer than the panel
and asserts that the mark appears. The other draws a panel that fits and
asserts that the mark does not, because a mark on every frame says nothing.

## References

[^1]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
