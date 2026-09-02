---
id: 0164
title: Review ADR-0083 for acceptance
status: complete
created: 2026-09-01
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

ADR-0083 states that the gate build checks every integer overflow. It is a
draft, so nothing may cite it as binding.

The change it accompanies is merged. The workspace manifest sets an
optimisation level, an overflow check and the debug assertions on the
development profile, and a test file asserts the check. The record can be read
against the code rather than against the intent, which is the first thing a
review here must do.

The record also sits beside three fresh findings about measurement. One says a
one-byte tile field over the target scale does not overflow a `u32`. One says
two runs of the gate suite hours apart are not comparable. A review must check
whether the record repeats the first and leans on an unpaired figure of the
second kind.

## Impact review

**Governed by.** The registry states who reviews and what a delegated review
must do that a second reader would do for free: read the record against the
code, be an agent that did not write it, and state what it tried to reject.
The scope rule states the test for whether a decision needs a record, and the
categories that must not appear in one.

**Changes.** ADR-0083 moves from `Draft` to `Accepted`, or the review returns
it with objections. Nothing else changes.

**Creates.** None.

**Blockers.** None. BLK-007 stays open and is untouched. Every figure this
review reads describes a development machine, and none is evidence about the
target platform.

**Precedent.** FND-141 holds the corrected accumulator arithmetic. FND-142
holds the rule that a figure taken hours from its comparison is a figure about
the machine. Both were opened by the work this record accompanies, so both are
tests the record must pass rather than material it may cite loosely.

## Done when

- An agent that did not write ADR-0083 has read both decisions against the
  manifest, the gate recipes and the test file.
- The review states what it tried to reject, and why each objection failed or
  held.
- The review states whether the record repeats the false example FND-141
  corrected.
- The review states whether any figure the record leans on was paired.
- The review names every claim that only a run can settle, and marks it
  unverified.
- The registry row holds the outcome.

## Outcome

**ADR-0083 stays at `Draft`.** The constraint is sound and three sentences are
not. The review holds the detail and the replacement text for each.[^1]

The record was compiled against nothing. Another worker held the machine, so
no `cargo` command ran. The review names four claims that rest on the commit
body of the merged change and marks each unverified. The five record and
register checks were run, because they compile nothing, and all five pass.

**Both decisions have code behind them.** The manifest declares the profile,
the test profile inherits it, and the gate runs the workspace tests. The test
file holds three assertions, and both operands of every overflow pass through
`black_box` so the compiler cannot fold the sum.

**The record does not repeat the false example.** This was the first thing
checked. ADR-0083 states the widening rule and gives no arithmetic, and the
test that supports it sums a two-byte field, which does overflow.

**The magnitude the record leans on is paired.** The context says the suite
gets several times faster. The paired evidence is in the commit body: the two
profiles were alternated back to back and gave 429 s and 430 s against 84 s and
79 s for the workspace tests.

**Three sentences must change.** One says the test compiles out of the release
build, and one of the three tests has no attribute and runs there. One says
turning the debug assertions off removes the check as well as the test, and the
manifest sets the check explicitly, so it stays. One says the optimisation
level bought the larger of the two savings, and nobody timed the suite with the
check off.

**Seven objections were attempted.** Five failed. One holds against the
register and not against the record. One holds weakly against a footnote and
does not stand alone.

**The review found two register defects.** FND-149 records that the development
budget register holds the comparison its own text forbids. FND-150 records that
the sweep which corrected the accumulator example reached one document and left
six, two of them accepted records. Item 0165 carries both repairs.

Register entries that moved: FND-149 and FND-150 opened. No blocker opened or
closed, and BLK-007 is untouched. No decision row was needed.

## References

[^1]: Review 0164, the gate build profile record. `docs/reviews/0164-the-gate-build-profile-record.md`
