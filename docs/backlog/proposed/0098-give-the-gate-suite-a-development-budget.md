---
id: 0098
title: Give the gate suite a development budget
status: proposed
created: 2026-08-31
implements: []
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
quantity has had no owner because the rule that governs the first is silent
about it.

## The decision this waits on

**Whether the project keeps a performance path for the development machine at
all.** The decision register holds the question, the options and a
recommendation.[^2] This item builds option 2 and should not start before the
owner takes it.

## What the work does

1. State a budget for the gate suite on a development machine, in the
   reference tables and never in a record.[^3]
2. Make the figure visible when the suite runs, so a change that exceeds the
   budget is seen rather than absorbed.
3. Reduce the golden state hash cost. Read the test before choosing how: the
   scenarios are a table, the world extents in that table drive the cost, and
   whether every scenario needs every frame is the first question.

## What this must not do

- **It must not weaken the test to make it fast.** The golden test is one of
  the two determinism gates, and a scenario removed to save time is coverage
  removed.[^4] If a scenario goes, say what it covered and what now covers it.
- It must not state a development figure as evidence about the target. A local
  measurement misleads on this project in a specific way: the development
  machines and the target have different cache line sizes.[^5]
- It must not put a figure in a decision record.[^3]

## The development machines are not one machine

The project develops on x86-64 and on Apple Silicon, and it says so.[^6] The
two do not perform alike. The project owner reports that the engine runs much
faster on Apple Silicon.

A budget therefore names the machine it was taken on, or it means nothing. A
single number for "a development machine" would be the same error as a single
number for "a machine", one level down.

Apple Silicon is the closer of the two to the target in one way that matters:
it is arm64, so it exercises the same code generation. It is the further away
in another: it uses a 128-byte cache line against the target's 64, which is
exactly the difference that makes a local measurement mislead on false sharing
and alignment.[^5] **An Apple Silicon figure is not evidence about the target**,
and this item must not let one become so.

## Done when

- The decision register records the owner's answer.
- A budget for the suite exists in the reference tables, marked as local to a
  development machine.
- The suite reports its own cost against that budget.
- The golden test costs less than it does today, and the commit body states
  the before and after figures and the command that produced them.
- No determinism scenario was removed without a statement of what replaced it.
- Every figure names the machine that produced it.
- `just check` exits 0.

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Decisions register, DEC-033. `docs/DECISIONS.md`
[^3]: Decision Record Scope, section 4.1. `.claude/rules/adr-scope.md`
[^4]: Testing Rules, section 1. `.claude/rules/testing.md`
[^5]: Project orientation, the target platform. `CLAUDE.md`
[^6]: Project orientation, the target platform. `CLAUDE.md`
