---
id: 0316
title: Give the panel one standard that a new panel registers with
status: complete
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0003]
blocked-by: []
---

## Why

**The viewer draws one panel and there is no way to add a second.** The panel
is one list of lines, built by one method of one readout. A person who wants to
show a character, a per-faction count, or the log of the last tick has to add
a section to that list. Every such addition edits the same file and the same
method, so two people cannot add two sections at once.

**The layout rules exist and they are not written down.** The panel knows its
width, its padding, its line height and the pixel room a value gets. Each of
those is a constant in the drawing file, and a new section learns them by
copying an old one.

**Two line kinds write past the right edge and nothing stops them.** A row cuts
its value. A note and a founding row do not, so the length of the text is the
only thing that keeps them inside the panel. That is the class item 0300 names.

## Done when

- One module states the panel geometry, the colours and the cut. No second
  file declares the width, the padding or the line height.
- Every text a panel writes goes through one writer that takes a right edge.
  A caller cannot write past it, whatever the text says.
- A new panel is one new file. It states its own name and its own lines, and
  it registers in one list.
- A test renders a panel whose text is longer than any panel width and asserts
  that no pixel outside the panel rectangle changed.
- The test can fail. Put an uncut write back and watch it notice.

## Impact review

**Governed by.** ADR-0067 D1 holds that the viewer reads the world and never
writes to it, and D3 puts the floating point boundary at the viewer. ADR-0070
D1 holds that the panel reports what the drawing pass read and starts no pass
of its own over the world. ADR-0070 D2 holds that a number the panel cannot
afford is stated as absent and never as a zero. ADR-0094 D2 holds that the
caller owns the pixels.

**The standard belongs in Rust, and the discovery is why.** The panel is laid
out in the viewer crate. The control plane owns the loop, the camera and the
pixel memory, and it asks for one frame. It never places a glyph. A panel
standard written in Python would therefore describe a layout that nothing in
Python performs, which is a capability nobody invokes.[^1]

**This work contradicts no record.** It moves declarations that already exist
into one module and adds a registration list. The cost rule of ADR-0070 D1
binds each new panel, so the registration list carries that obligation in its
own documentation.

**This work creates no decision that needs a record.** The three tests of the
scope rule fail on it: a future contributor could not reasonably lay the panel
out somewhere other than where the drawing is, the arrangement is cheap to
change, and the reasoning is visible in the module.[^2] The panel width and the
line height are a mechanism, not a constraint on the project.

**Blockers.** None govern a value here. The panel width is a property of the
glyph table and of the window, not a budget.

**Registers.** This item closes backlog item 0300. It opens no blocker and it
records one finding, because the deck changes what a reader must know before
adding a panel.

## References

[^1]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
[^2]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`

## Outcome

**The standard exists in the viewer, and every panel registers with it.** One
module declares the panel geometry, the colours, the cut mark and the line
kinds. It holds one list of the registered panels, and the frame draws the deck
by walking that list. A panel is one file that states its own name, its own
title and its own lines.

**There is one draw path.** The head-up display reads the width, the padding,
the line height and the value column from the standard, so no second file
declares them. Every line kind writes through one writer that takes a right
edge, so a caller cannot write past the panel.

**The keys come from the list.** The control plane asks the engine for the
panel names and gives each one a function key in that order, so a panel that
joins the list gets a key with no edit to the control plane.

**A run without a window can now draw a panel of the deck.** The demonstration
takes the panel names on the command line, which is how the panels of item 0318
were proved outside a window.

The work landed under an earlier commit and this item was never moved. A
finding records that, because an item nobody moves states work that is open
when it is done.[^3]

## References

[^3]: Findings register, FND-526. `docs/FINDINGS.md`
