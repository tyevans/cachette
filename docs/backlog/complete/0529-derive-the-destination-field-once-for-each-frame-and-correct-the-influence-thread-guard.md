---
id: 0529
title: Derive the destination field once for each frame, and correct the influence thread guard
status: complete
created: 2026-09-08
implements: [ADR-0125 D1, ADR-0144 D2, ADR-0001 D4, ADR-0087 D1]
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**The simulation holds 96 to 99 percent of the wall clock of a training run,
and a report measured where one tick of that clock goes.**[^1] It named two
defects, and this item is both of them.

**The destination field was derived four times in one frame.** The barrier of
the frame derived it once. The built-in controller then applied a project
order, a crossing order and a settling order, and each of those calls the send
verb, which ended by deriving the whole field again. The report measured the
four derivations at 71 percent of a late training frame.

**The influence solve started threads to avoid threads.** The relaxation takes
a single-threaded path when the cell count is at most the thread count. A world
of 48 tiles a side holds four level 1 cells, so at one thread the guard read
four against one and started one thread for each faction and each pass. The
trainer runs one thread for each world, so every training frame paid it.

## Impact review

**Governed by.** ADR-0125 D1 states that the control plane names the seed set
of a destination plane and that the engine derives the field from it. ADR-0144
D2 states that the controller emits every command through a verb a caller can
also call, and that no verb checks who calls it. ADR-0001 D4 states that one
binary gives one answer at any thread count. ADR-0087 D1 states that an
influence solve runs a fixed iteration count.

**How the work honours them.** The send verb keeps its signature and its
meaning: a caller outside a step finds the field derived when the call
returns. The step sets one flag while the controller runs, and it derives the
field once before it returns. The flag states a fact about the frame and not
about the caller, so the verb asks nothing about who called it. The influence
guard gains one condition and drops no pass, so the iteration count is
unchanged.

**Changes.** None. No record stated when the field derives.

**Creates.** None. The reasoning that a future contributor needs is the
tension between one derivation and a stale read, and a finding carries it
with its measurement.[^2] The scope rule asks all three of its tests to hold
before a record is written, and the second does not: the change is cheap to
reverse, and nothing in the project would refuse a change on the strength of
a record here.[^3]

**Blockers.** BLK-007 governs every cost figure this item states. Every figure
is a development-machine figure and none belongs in the target platform
register.

**Precedent.** FND-029 is the finding that put the derivation inside the verb,
and the work keeps it rather than deleting it.[^2]

## Done when

- One frame derives the destination field once.
- A caller outside a step reads a field that describes the seed set the world
  holds.
- The influence relaxation starts no thread when the caller asks for one.
- The whole-world state hash is unchanged, and it is the same at 1, 2, 4 and
  12 threads.
- A test drives the engine and fails when the field is wrong, and a test-only
  switch proves that the test can fail.

## Outcome

**Both defects are fixed, and the state hash did not move.** The hash is the
same before and after the change at 1220 ticks and at 2500 ticks, and it is
the same at 1, 2, 4 and 12 threads. The derived field enters no hash, and no
pass between the barrier and the end of the step reads it, so no behaviour
depends on when the derivation runs.

**The report offered two shapes of fix and the work took neither exactly.**
Collecting the sends saves three of the four derivations. Deferring the
derivation to its first reader saves all four, and it changes what a caller
reads between two steps, which is the case FND-029 records. Moving the one
derivation to the end of the step saves all four and changes what no reader
sees, because the step derives the field before it returns.

**The influence guard needed one condition.** The report named it, and three
independent sweeps in that report agree on the cause.

Measured on one pinned performance core of a development machine, at 48 tiles
a side, three factions, one thread. A late frame falls from 15.22 to 6.94
milliseconds, an early frame from 3.85 to 3.51, and the mean frame over an
episode of 2500 ticks from 13.29 to 6.65. The findings register holds the
readings and the run-to-run spread.[^2]

## References

[^1]: Report 39, what one tick of the training world costs. `docs/research/reports/39-what-one-tick-of-the-training-world-costs.md`
[^2]: Findings register, FND-029, FND-664 and FND-665. `docs/FINDINGS.md`
[^3]: Decision Record Scope, section 1. `.agents/rules/adr-scope.md`
