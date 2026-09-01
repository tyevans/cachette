---
id: 0040
title: Pin the two call sites that give a change its barrier
status: refined
created: 2026-08-31
implements: [ADR-0018 D3, ADR-0056 D3]
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

A spawn or a despawn made between two frames is a structural change that has
passed no barrier. It leaves the derived unit structure stale. Admission reads
the occupancy of a target from that structure, so the structure must describe
the arena before the intents are admitted.

The register answers where that change gets its barrier. The step opens by
rebuilding a stale structure, so the step gives the caller's change the
barrier it never had.[^1] The cost is a revision comparison on a frame that
changed nothing. One operation therefore has two call sites. The rebuild at
the end of the step stays last, and it stays the barrier of that frame.

**This item no longer asks for a record.** It asks for the mechanism that
holds the two call sites in place.

## Impact review

**Governed by.** ADR-0018 D3 states that the derived structure rebuilds at the
barrier, after the structural apply, and calls that ordering a decision.
ADR-0056 D3 states that admission sorts by a stable key and then admits, and
admission is the reader that the stale structure would mislead.

**Changes.** No record changes.

**Creates.** No record. The scope rule gives three conditions, and a decision
needs a record when all three hold.[^2] The second condition fails. The
objection a reviewer would raise is about the two call sites of one function,
and moving the rebuild is cheap, so a record would buy the right to refuse a
change that costs little to make. The first condition holds, because a
contributor could ask the caller to rebuild instead. The third condition is
weak, because the reasoning already sits at the call site and in the register.
Promote the row to a record when a second structural apply lands inside the
frame. The ordering between two applies is a real decision, and a comment is
not the mechanism this project accepts for that class of fact.

**Blockers.** None.

**Precedent.** Item 0030 found one operation with two callers, where the
second hid the first and repaired a wrong order quietly.[^3] This item leaves
two call sites in place on purpose, so it must not repeat that shape. The
difference is that the rebuild refuses a stale structure rather than repairing
one silently, and a refusal is visible.

**Product record.** None.

## What fails if somebody changes it back

Removing the call at the top of the step is the change to guard against. The
engine is obligated to rebuild, not the caller, so the test starts at the
engine and not at the derived structure.[^4]

A test drives the public step through this sequence:

1. Build a world and run one step, so the structure is fresh.
2. Despawn a unit outside any frame. The despawn passes no barrier.
3. Run the next step, and assert that it succeeds and that admission saw an
   occupancy that counts no dead unit.

The item as first written said that this call site has no test that could
fail, because a caller who forgets to rebuild is served rather than refused.
**That reading looks wrong.** The structure carries the arena revision it was
built from, and a read against a later revision returns a stale error rather
than a wrong answer. A step that opened without the rebuild would therefore
fail the step, not answer wrongly. This work must settle which of the two is
true.

Settle it by putting the defect back. Remove the call at the top of the step
and run the suite. If nothing fails, the test does not reach the case and the
test is the work. If the step fails, record which error it raises, because
that error is the mechanism and the test must assert on it.[^5]

## Done when

- The call at the top of the step is covered by a test that starts at the
  public step.
- The test has been proven able to fail, by removing the call and watching the
  suite go red. The commit body names the command and the result.
- The comment at each call site says which barrier it serves, and neither
  comment names the other as the winner.
- DEC-021 stays a register row. No record is written.
- The findings register holds the outcome of the claim above, if the claim was
  wrong.
- The whole check command runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Decisions register, DEC-021. `docs/DECISIONS.md`
[^2]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^3]: Backlog item 0030. `docs/backlog/complete/0030-enforce-the-barrier-ordering.md`
[^4]: Testing Rules, section 5. `.claude/rules/testing.md`
[^5]: Testing Rules, section 2a. `.claude/rules/testing.md`
