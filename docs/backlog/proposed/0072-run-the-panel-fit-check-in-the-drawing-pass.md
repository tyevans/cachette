---
id: 0072
title: Run the panel fit check in the drawing pass
status: proposed
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

## References

[^1]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
