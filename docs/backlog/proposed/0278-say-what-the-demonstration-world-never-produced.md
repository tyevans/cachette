---
id: 0278
title: Say what the demonstration world never produced
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

**A subsystem that produces no instance is indistinguishable from a subsystem
that nothing draws.** Both give a front end an empty set. The project has now
paid for that twice: item 0216 and item 0240 each found a chain that was
complete in the engine and never ran, and each cost an audit to find.

An audit found three more.[^1] The demonstration world holds four settlements
with sixteen ranked positions between them, and after two hundred ticks no
unit holds one. It holds no character at all. It holds no improvement at all.
Nothing in the run says so. A person watching sees a world that looks busy,
and a person reading the panel sees the sections that have content.

The demonstration already reports one fact of this kind. It says whether each
founded ground carries its group, and that line exists because the world fed
every unit for as long as it had existed and nobody noticed.[^2] **That report
covers one subsystem.** This item asks for the general form.

The rules name this shape. A capability that nothing invokes passes its own
test and ships inert, and the question to ask is who is obliged to invoke
it.[^3] For these three the answer is the engine, and the engine does not.

## What is missing before this is refined

- The impact review.
- **Where the census lives.** The figures come from public readers of the
  engine, so any caller can take them. Whether the demonstration prints them,
  whether a command writes them, or whether both read one function is an
  arrangement question, and the item must say which serves the guard rather
  than describing a module move.
- **What the list of subsystems is, and what keeps it current.** A hand-written
  list of things to count is a second declaration site for what the engine
  holds, and it goes stale the next time a subsystem lands.[^4] The item must
  say what fails when the two disagree, or say plainly that nothing does and
  why that is acceptable here.
- **Whether a zero fails or reports.** A zero is correct today for three
  subsystems, so a gate that failed on one would be red from the first commit.
  A report that nobody reads is the defect this item is about. The item must
  choose, and the choice is the substance of it.
- Whether the count belongs to the demonstration or to the engine. "How many
  positions does a unit hold" is a fact about the world. Item 0241 raises the
  same question for the founding report and neither should answer it alone.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-269. `docs/FINDINGS.md`
[^2]: Findings register, FND-232. `docs/FINDINGS.md`
[^3]: Testing rules, section 5. `.claude/rules/testing.md`
[^4]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
