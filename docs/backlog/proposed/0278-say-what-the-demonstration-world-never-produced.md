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

An audit found three more, and it named three zeros in the demonstration
world: no unit held a ranked position, no character existed, and no improvement
existed.[^1] Nothing in the run said so. A person watching sees a world that
looks busy, and a person reading the panel sees the sections that have content.

**Two of the three zeros are closed, and the item is worth more now than when
it was written.** The demonstration world was driven again on 3 September 2026,
for 400 ticks. At tick 200 it held 32 ranked positions with 16 of them filled,
and 26 of its founded people had become characters. At tick 400 it held 34
characters. It still holds no improvement at all, and it will hold none until
something makes a unit choose to build.[^2]

**The measurement is the argument for the item, not against it.** Two of the
three counts this item quoted moved without anybody restating them, and both
moved in the same direction: a chain that was inert began to run and no line of
the demonstration announced it. A guard that only reports a zero would have
said nothing on either day. **A guard that reports the count says what changed,
and it is the count and not the zero that this item should ask for.**

The demonstration already reports one fact of this kind. It says whether each
founded ground carries its group, and that line exists because the world fed
every unit for as long as it had existed and nobody noticed.[^3] **That report
covers one subsystem.** This item asks for the general form.

The rules name this shape. A capability that nothing invokes passes its own
test and ships inert, and the question to ask is who is obliged to invoke
it.[^4] For the improvement the answer is the engine, and the engine does not.

## What is missing before this is refined

**The open questions are answered.** The design of the living world game layer
answered each one on 5 September 2026, and item 0472 holds the work.[^6] [^7]

- **Where the census lives.** In the engine, as a reader named
  `subsystem_census()`. The demonstration prints it at its end, and the gate
  reads the same reader. The name `census` was taken three times, so the reader
  carries the longer name.
- **What the list of subsystems is, and what keeps it current.** One Rust table.
  Each row names the subsystem and the reader that counts it. The reader
  derives from the table, so a list written by hand does not exist and nothing
  can disagree with it.[^5]
- **Whether a zero fails or reports.** Both, at two sites. The reader reports a
  count. A gate drives the demonstration world for a tick count that the
  balance register holds and asserts that every count is nonzero.[^8] The gate
  is not red from the first commit, because pass 1 of the game layer makes a
  faction build, so the improvement count is nonzero when the gate lands.
- **Whether the guard reports a count or a zero.** A count.
- **Whether the count belongs to the demonstration or to the engine.** The
  engine. A count of what the world holds is a fact about the world.

**What keeps this item in `proposed/`.** Item 0472 delivers the census as one
of its five parts, and its impact review and its checkable statements cover it.
A second refined item for the same deliverable would be one fact in two places,
and the two would drift. This item closes when 0472 closes. If 0472 is split
and the census falls out of it, this item takes the census and is refined then.

## Done when

- Item 0472 is complete, and its outcome names the census.
- The demonstration prints one count per subsystem at its end.
- The gate asserts that every count is nonzero on the demonstration world after
  the census tick count.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-269. `docs/FINDINGS.md`
[^2]: Backlog item 0180, let a unit choose to build. `docs/backlog/proposed/0180-let-a-unit-choose-to-build.md`
[^3]: Findings register, FND-232. `docs/FINDINGS.md`
[^4]: Testing rules, section 5. `.claude/rules/testing.md`
[^5]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^6]: Design: the living world game layer, section 10.1. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^7]: Backlog item 0472, run a faction controller inside the step and end the game on territory. `docs/backlog/refined/0472-run-a-faction-controller-inside-the-step-and-end-the-game-on-territory.md`
[^8]: Balance register. `docs/reference/balance.md`
