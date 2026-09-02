---
id: 0165
title: Repair the two register defects the ADR-0083 review found
status: proposed
created: 2026-09-01
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

A review of ADR-0083 found two defects that the record itself does not carry.
Both are in registers, and both are the shape where one fact sits in more than
one place with nothing that fails when the copies disagree.[^1]

**The development budget register holds the comparison its own text
forbids.**[^2] The register says a row is a snapshot and does not support a
comparison against a row taken hours earlier. Its own warm rows are 153 s at
`opt-level 1` against 435 s with no optimisation, and the commit that recorded
them says the two runs are not next to each other in time. The 435 s figure is
a third-hour sample of a suite that measured 263 s in the first hour of the
same session.[^3] A reader who compares the two rows gets an inflated ratio.
The paired evidence exists and sits in a commit body, where the register cannot
reach it. FND-149 holds this.[^4]

**The sweep that corrected the accumulator example reached one document and
left six.** FND-141 corrected the arithmetic in the project owner's document. A
whole-tree search finds the old example in two accepted records, in three
source comments and in one complete backlog item. The rule that an accumulator
widens is right in all six places, and the example beside it is false. FND-150
holds this.[^5]

## Impact review

**Governed by.** ADR-0002 D3 states that an accumulator widens, and it is one
of the six sites that states the false example. ADR-0008 D2 governs which
figures belong to a development machine and which belong to the target.

**Changes.** ADR-0002 and ADR-0053 are accepted, and each holds the false
example inside a decision or beside one. An accepted record does not change
except in status, so this item does not edit them. It states the choice and
asks the owner to make it: a repair in place, a correction recorded elsewhere,
or a record that supersedes.

**Creates.** No record, unless the owner chooses supersession.

**Blockers.** None, and BLK-007 stays open. Every figure here describes a
development machine.

**Precedent.** FND-142 gives the rule for a paired figure. FND-141 gives the
corrected arithmetic. The commit rule says a sweep is done when a whole-tree
search comes back clean, and that the command goes in the commit body.[^6]

## Done when

- The development budget register no longer invites a comparison between two
  rows taken hours apart. Either it holds a paired row, or the row that no
  comparison may use says so beside the figure.
- The paired execution figures are in the register, not only in a commit body.
- The three source comments state the arithmetic that FND-141 holds, or state
  the rule without an example.
- The complete backlog item is left alone or annotated, and the choice is
  stated.
- The two accepted records have an owner's decision recorded against them, and
  the item says which of the three routes was taken.
- A whole-tree search for the phrasings of the example comes back clean, and
  the command is in the commit body.

## References

[^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^2]: Development budgets, the gate suite budget. `docs/reference/development-budgets.md`
[^3]: Findings register, FND-142. `docs/FINDINGS.md`
[^4]: Findings register, FND-149. `docs/FINDINGS.md`
[^5]: Findings register, FND-150. `docs/FINDINGS.md`
[^6]: Commit Message Rules. `.claude/rules/commits.md`
