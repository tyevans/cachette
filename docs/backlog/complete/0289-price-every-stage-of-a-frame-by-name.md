---
id: 0289
title: Price every stage of a frame by name
status: complete
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

**Sixty-two percent of the cost of a unit had no name.** The frame is a
sequence of private methods on the world. Nothing on the public interface
calls one of them, so a pass could only be priced by running a whole frame
with the pass switched off and taking the difference. Three switches exist.
The split they produced named the choice at 71 milliseconds, the bridge
rebuild at 26 and the economy at 6, and it left 170 milliseconds in one
residual.[^1]

**Every optimisation above that residual was chosen blind.** Four items in the
backlog propose a change to the layout or to the allocation, and each says to
measure the result. A measurement that reports a frame cost cannot say which
pass improved, so a change that helps one pass and hurts another reports the
sum and hides both.[^2] [^3] [^4] [^5]

This is the half of item 0237 that pays in measurement rather than in
checking, and that item states the two halves separately.[^6]

## Impact review

**Governed by.** ADR-0001 binds every change to the step: one binary gives one
answer at any thread count.[^7] ADR-0005 D1 forbids the clock in the
simulation.[^8]

**Neither is contradicted, and the reasons are structural rather than
argued.** The table reads a clock and adds two integers to a static. No pass
reads either integer, so no pass can branch on one. The table is behind a
feature that is off by default, and without the feature a span is an empty
type, so a shipped binary holds no clock read and no store. The step takes the
same branches in both builds.

The clock allowance sits on one function inside the feature, in the same way
the benchmark already holds one. The lint still covers every other line.

**Creates no record.** The scope rule gives three conditions and one fails: an
instrument that a feature switches off binds nothing, and a future contributor
who chose otherwise would choose a different instrument rather than a different
constraint.[^9] The declaration of what a stage reads and writes is a
constraint, and it stays in 0237 where a record may be needed.

**Blocked by nothing.** BLK-007 says no measurement exists on the target
platform. A harness has existed since the register was written, and this item
uses it.[^10]

## What the work did

A module names every pass of a frame in one macro list, and derives the
enumeration, the name, the count and two declarations from that list. The step
opens a span over each pass. The benchmark gains a mode that resets the table,
runs nine frames, and writes one row for each stage.

**The list and the step are compared by a test rather than by a reader.** One
frame must open each stage exactly as many times as the stage declares. A pass
that loses its span reports zero and the test names it. That was checked by
deleting one span and watching the test name that stage, which is the only
proof that a test reaches the case it was written for.[^11]

**The mode reports two rows that are not stages.** One is the sum of the
stages. The other is the wall time of the same frames, measured from outside.
The difference between them is the part of the step that no span covers, and
reporting it is what stops a future reader from assuming the stages are
exhaustive when a new pass has been added without one.

## What it does not do

It does not declare what a stage reads or what it writes, and it does not make
ADR-0009 checkable. That is item 0237, it is an architectural claim rather
than an instrument, and a declaration that nothing checks against the code is
the defect shape this project meets most often.[^12]

It gives no speedup. It makes one measurable.

## Outcome

**The measurement register holds the split, taken on the target platform.**[^1]
The residual divides. The figures, the machine, the commit and the huge page
setting are in the register, and the commit body holds the command that took
them.

## References

[^1]: Target platform costs. `docs/reference/graviton-costs.md`
[^2]: Backlog item 0266, order the unit arena by cell. `docs/backlog/refined/0266-order-the-unit-arena-by-cell.md`
[^3]: Backlog item 0267, hold the exit direction on the tile. `docs/backlog/complete/0267-hold-the-exit-direction-on-the-tile.md`
[^4]: Backlog item 0268, hold the cell index on the unit. `docs/backlog/complete/0268-hold-the-cell-index-on-the-unit.md`
[^5]: Backlog item 0269, map the large arrays with huge pages. `docs/backlog/complete/0269-map-the-large-arrays-with-huge-pages.md`
[^6]: Backlog item 0237, declare what each stage reads and writes. `docs/backlog/proposed/0237-declare-what-each-stage-reads-and-writes.md`
[^7]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^8]: ADR Registry, row 0005. `docs/adrs/REGISTRY.md`
[^9]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^10]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^11]: Testing rules, sections 1 and 2a. `.claude/rules/testing.md`
[^12]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
