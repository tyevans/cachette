---
id: 0098
title: Give the gate suite a development budget
status: complete
created: 2026-08-31
implements: [ADR-0008 D2, ADR-0001 D4, ADR-0001 D5]
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

The gate suite is the thing a contributor runs many times a day, and its cost
is paid on the development machine. Nothing owns that cost.

The golden state hash test grows each time a subsystem enters the state hash.
Four subsystems entered it in one session. The test now dominates a debug run
of the suite, and no gate, no register and no budget noticed.

Every cost figure in this project belongs to the target platform, and one open
blocker states that no measurement exists there.[^1] That rule is about how the
engine performs. It is not about how long a contributor waits, and the second
quantity had no owner because the rule that governs the first is silent about
it.

The project owner has now answered that question. The project keeps two
performance paths with different standing.[^2] The target owns every claim
about how the engine performs. The development machine owns one local budget:
how long the gates take. This item builds the second path.

## Impact review

**Governed by.**

- **ADR-0008 D2** requires that a cost figure from a development machine is
  labelled as one. Every figure this item records must name its machine.
- **ADR-0008 D1** names the primary target. A development figure is not
  evidence about that target, and this item must not let one become so.
- **ADR-0001 D4** requires the two determinism tests. The golden state hash
  test is one of them. Making it cheaper must not remove it.
- **ADR-0001 D5** requires that both determinism tests can fail. The
  perturbation probe proves it. Any change to the golden test must leave the
  probe passing.

**Changes.** No record changes. ADR-0008 D2 already states the constraint this
work obeys, so nothing is superseded.

**Creates.** No record. The decision this item carries out is a register
decision, not a new binding constraint. A future contributor could choose to
treat a local figure as target evidence, but ADR-0008 D2 already forbids that,
so a second record would state a claim the project holds.[^3]

**Blockers.** None blocks this item. BLK-007 states that no measurement exists
on the target platform, and it stays open, because it is about the target.[^1]
This item measures the development machine, which BLK-007 does not cover.

**Registers.** Two register files now hold the split. The target register holds
target figures only and points at the local one. The local register holds the
gate suite budget, with the value blank until a measurement exists.[^4] [^5]

**Precedent.** The findings register records that one fact stored in two places
decays when nothing fails on disagreement. The budget figure has one home, and
the suite reads it from that home rather than holding a second copy.

**Product record.** None. This item serves the contributor, not a recorded
need.

## What the work does

1. Measure the gate suite on a named development machine, and record the
   figure and the allowance in the local register.[^5] The register holds the
   blank rows and the command that fills them.
2. Make the figure visible when the suite runs, so a change that exceeds the
   budget is seen rather than absorbed.
3. Reduce the golden state hash cost. Read the test before choosing how: the
   scenarios are a table, the world extents in that table drive the cost, and
   whether every scenario needs every frame is the first question.

## What this must not do

- **It must not weaken the test to make it fast.** The golden test is one of
  the two determinism gates, and a scenario removed to save time is coverage
  removed.[^6] If a scenario goes, say what it covered and what now covers it.
- It must not state a development figure as evidence about the target. A local
  measurement misleads on this project in a specific way: the development
  machines and the target have different cache line sizes.[^7]
- It must not put a figure in a decision record.[^3]
- It must not add a target figure to the local register, or a local figure to
  the target register.

## The development machines are not one machine

The project develops on x86-64 and on Apple Silicon, and it says so.[^7] The
two do not perform alike. The project owner reports that the engine runs much
faster on Apple Silicon.

A budget therefore names the machine it was taken on, or it means nothing. A
single number for "a development machine" would be the same error as a single
number for "a machine", one level down.

Apple Silicon is the closer of the two to the target in one way that matters:
it is arm64, so it exercises the same code generation. It is the further away
in another: it uses a 128-byte cache line against the target's 64, which is
exactly the difference that makes a local measurement mislead on false sharing
and alignment.[^7] **An Apple Silicon figure is not evidence about the target**,
and this item must not let one become so.

## Done when

- A budget for the suite exists in the local register, with a value, a
  machine, an architecture, a build profile and a date.
- The suite reports its own cost against that budget.
- The golden test costs less than it does today, and the commit body states
  the before and after figures and the command that produced them.
- No determinism scenario was removed without a statement of what replaced it.
- The perturbation probe still fails the golden test on a perturbed build.
- Every figure names the machine that produced it.
- No target figure appears in the local register, and no local figure appears
  in the target register.
- `just check` exits 0.

## Outcome

**Done.** The local register now holds three rows for one named development
machine: the measured cost of the gate suite, a budget of that cost plus an
allowance of one fifth, and the cost of the golden state hash test.[^5] Each
row names the machine, the architecture, the build profile and the date. No
row is evidence about the target platform, and the target register gained
nothing.[^4]

**The suite reports its own cost.** A wrapper times the gates and prints the
figure against the budget row for the architecture that runs it. It reads the
budget from the register, so the figure has one home. It reports and does not
fail, because a wall clock figure on a loaded machine is not a gate.[^6]

**The golden test costs a quarter of what it did.** Two scenarios exist for
their extent, not for their duration, and both ran the full frame count. Their
count is now eight. No scenario was removed, and no hash line changed when the
golden files were recorded again: the two files lost their trailing lines and
kept every line they had. The commit body holds the figures and the machine.

**Both determinism tests can still fail.** The golden test fails on the
perturbed build, and the probe recipe now runs it, which it did not before.
The thread-count test still runs at one, two and twelve threads.

BLK-007 stays open. It is about the target, and this item measured a
development machine.

Two findings were recorded.[^8] The cost of a wide scenario is its extent
times its duration. The probe recipe covered one determinism test of the
two.

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Decisions register, DEC-033, option 2. `docs/DECISIONS.md`
[^3]: Decision Record Scope, sections 1 and 4.1. `.claude/rules/adr-scope.md`
[^4]: Budgets and costs, the target register. `docs/reference/budgets.md`
[^5]: Development budgets, the local register. `docs/reference/development-budgets.md`
[^6]: Testing Rules, section 1. `.claude/rules/testing.md`
[^7]: Project orientation, the target platform. `CLAUDE.md`
[^8]: Findings register, FND-097 and FND-098. `docs/FINDINGS.md`
