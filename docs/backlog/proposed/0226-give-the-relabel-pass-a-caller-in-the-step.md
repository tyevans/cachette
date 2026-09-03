---
id: 0226
title: Give the relabel pass a caller in the step
status: proposed
created: 2026-09-02
implements: [ADR-0021 D3]
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

The record of descent carries two Euler interval labels over the father
forest. A pass rebuilds them, and after it runs an ancestry question costs two
integer comparisons and a patrilineal line is one span.

**Nothing in the engine calls that pass.** A test calls it. A world that steps
never does. A world left to run therefore holds labels for the characters it
started with, and for nobody born since. Every question about a later character
answers nothing.

This is the inert-capability shape: the mechanism has a test, the test drives
the mechanism directly, and no caller reaches it.[^1] The capability is
correct and it ships unreached.

The work that added the labels could not close this. The frame schedule lives
in a file that work did not own.

## What the work does

1. Call the relabel pass from the step, at the character-tier barrier.
2. Choose the cadence and say what decided it. The research runs the rebuild
   once for each simulated year, and it gives the walk cost per node.[^2] The
   scale constants give the ticks in a simulated day.[^3] **The cadence
   is derived and not measured.** No run has priced the walk.[^4]
3. Drive the engine, not the pass: step a world through a birth and assert
   that the new character answers a patrilineal question afterwards.[^5]

## Impact review

**Governed by.** ADR-0021 D3 makes the descent columns struct-of-arrays
because the deciding pass reads few columns for each row. This is that
pass.[^6] ADR-0004 D1 fixes the visit order, and the pass already takes its
order from the record rather than from a caller.[^7] ADR-0001 D4 governs what
enters the state hash: the labels are derived from the parent edges, so they
stay out of it, and adding a caller must not change that.[^8]

**Changes.** No record.

**Creates.** No record, unless the cadence turns out to be a choice a
contributor could reasonably make otherwise. The cadence may be a number in the
reference tables. If the schedule also shows why it sits where it sits, the
code is the record.[^9]

**Blockers.** BLK-007 governs the cadence, because the cost of the walk is
derived. State the cadence parametrically against the ticks in a simulated day
rather than as a tick count nobody derived.

**Serves.** No product record directly. It is what makes a dynasty question
answerable in a running world.

**Precedent.** The project has shipped a capability that nothing invoked. The
rule that came out of it says to ask who is obligated to call this.[^1] Here
the answer is the engine.

## What fails if somebody changes it back

- A test steps a world through a birth and asks a patrilineal question about
  the new character. Removing the call fails it, because the answer becomes
  absent.
- A test asserts that the state hash does not move when the pass runs and
  nothing else changed. A caller that put a derived label into the hash fails
  it.

Remove the call and watch the test stay green before claiming it covers the
case. A test that builds an arena and calls the pass itself proves nothing
about the engine.[^5]

## Done when

- The step calls the relabel pass, and the cadence is stated where a reader
  can find it.
- A test drives a stepping world and reaches the labels through it.
- The state hash does not move because the pass ran.
- The tests above exist and each has been proven able to fail.
- The whole check command runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Recurring defect shapes, shape 3. `.claude/rules/recurring-defects.md`
[^2]: The character graph and inheritance, section 3.4. `docs/research/reports/14-character-graph-and-inheritance.md`
[^3]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^5]: Testing Rules, section 5. `.claude/rules/testing.md`
[^6]: ADR-0021, a layout claim names one structure and one pass, and never a tier, decision D3. `docs/adrs/draft/adr-0021-layout-follows-the-access-pattern.md`
[^7]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^8]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^9]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
