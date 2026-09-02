---
id: 0166
title: Clear the footnote baseline
status: proposed
created: 2026-09-01
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

The footnote check fails on four things: a marker with no definition, a label
defined twice, one source under two labels, and a definition nothing cites. The
documents that already break those tests are in a baseline, so the gate passes
over them.[^1]

The baseline is falsifiable, so it can only shrink. It does not shrink by
itself. Each line is a document that states something a reader cannot follow.

Most of the baseline is one shape: a document writes a reference section that
its body never cites. The seven records numbered 0001 to 0007 do this, and they
predate the documentation rule.[^2] Several reviews and completed items do it
too.

## What the work does

Take a group of the baseline at a time and repair it. Take a line out of the
baseline in the same change. The check fails when a line matches nothing, so a
repair without the deletion turns the gate red, and a deletion without the
repair does the same.

The groups differ in cost, and they are not one item's work:

1. **The two index documents and the two registers.** A definition left behind
   when the row that cited it moved. Cheapest.
2. **The reviews and the completed items.** Writing a marker into prose that
   describes finished work. Ask first whether the definition should go instead.
3. **The findings register, one source under many labels.** This is a change to
   how the register is written, not a repair of one entry.
4. **The seven early records.** An accepted record does not change except in
   status, so this needs a decision before it needs an edit.[^3]

## The questions this item must answer before it is refined

**Whether group 4 is work at all.** The rule that a record does not change
after acceptance may simply mean those records keep their reference sections as
they are, and the baseline holds them for good. If so, say it in the baseline
and close that group rather than carrying it.

**Whether group 3 is a decision.** Reusing one marker across a register of this
size makes an entry harder to read on its own, and the register is written to
be read one entry at a time. That trade is a decision, not a repair.

## Done when

Filled in when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: The footnote baseline. `scripts/footnote-baseline.txt`
[^2]: Project orientation, the documentation rules. `CLAUDE.md`
[^3]: Definition of Done, update the registers. `.claude/rules/definition-of-done.md`
