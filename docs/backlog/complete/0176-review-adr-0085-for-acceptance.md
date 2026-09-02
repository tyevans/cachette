---
id: 0176
title: Review ADR-0085 for acceptance
status: complete
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

ADR-0085 states that an entity crosses to Python as one opaque identity that
the engine resolves. It is a draft, so nothing may cite it as binding.

The change it accompanies is merged. The event log gives Python a column of
whole identities, the bindings take identities back, and the engine resolves
each one against the arena before it acts. The record can be read against the
code rather than against the intent, which is the first thing a review here
must do.

The record sits beside a finding that bears on it directly. FND-148 records
that deleting the generation comparison in the resolution left the Python read
green, because the arena compares the generation a second time. A review must
check whether the record claims a protection that the reinserted defect
disproved.

## Impact review

**Governed by.** The registry states who reviews and what a delegated review
must do that a second reader would do for free: read the record against the
code, be an agent that did not write it, and state what it tried to reject.
The scope rule states the test for whether a decision needs a record, and the
categories that must not appear in one.

**Changes.** ADR-0085 moves from `Draft` to `Accepted`, or the review returns
it with objections.

**Creates.** None.

**Blockers.** None. BLK-007 stays open and is untouched, because the record
states no cost figure.

**Precedent.** FND-148 holds what a test above two checks can and cannot
cover. The testing rule holds the case where a random draw was keyed on the
slot index rather than on the identity, which is the failure this record
exists to prevent at the boundary.

## Done when

- An agent that did not write ADR-0085 has read all four decisions against the
  value types, the world, the bindings, the type stub and the agent server.
- The review states what it tried to reject, and why each objection failed or
  held.
- The review states whether the record claims an enforcement that nothing
  performs.
- The review states whether the record's claim survives FND-148.
- The review names every claim that only a run can settle, and marks it
  unverified.
- The registry row holds the outcome.

## Outcome

**ADR-0085 is accepted.** All four decisions hold against the code, and no
objection held.[^1]

**Nothing was compiled.** Three other workers held the machine, so no `cargo`
command ran. The review names the claims that rest on a run and marks each
unverified. The six document checks were run and all six pass.

**Six objections were attempted. All six failed.** The record makes no claim
about test coverage, so FND-148 does not reach it. It claims no enforcement of
the no-loop rule, which ADR-0043 reserves and nobody has written. Its width
claim follows from the identity layout rather than from a budget. Its
alternatives section names the wrapper and the two-column form and rejects
both.

**The record moved to `accepted/`, and the sweep repaired seven files.** The
registry row holds `Accepted`, and the priority index no longer lists it as
waiting.

Register entries that moved: none. No finding, blocker or decision row was
needed.

## References

[^1]: Review 0176, the Python identity record. `docs/reviews/0176-the-python-identity-record.md`
