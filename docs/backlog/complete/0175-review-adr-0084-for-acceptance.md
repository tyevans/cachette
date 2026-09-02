---
id: 0175
title: Review ADR-0084 for acceptance
status: complete
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

ADR-0084 states that the world reserves the unit columns at construction and
refuses a spawn past the reservation. It is a draft, so nothing may cite it as
binding.

The change it accompanies is merged. The world settings hold a reservation
field, the arena takes it, the columns and the free queue reserve at
construction, and a founding that runs out of slots undoes itself. The record
can be read against the code rather than against the intent, which is the
first thing a review here must do.

The record sits beside three findings that bear on it directly. One records
that the world reserved nothing. One records that a refused founding left a
settlement standing. One records that the settings field cost 82 struct
literals rather than 25. A review must check whether the record repeats a
claim any of the three corrected.

## Impact review

**Governed by.** The registry states who reviews and what a delegated review
must do that a second reader would do for free: read the record against the
code, be an agent that did not write it, and state what it tried to reject.
The scope rule states the test for whether a decision needs a record, and the
categories that must not appear in one.

**Changes.** ADR-0084 moves from `Draft` to `Accepted`, or the review returns
it with objections. Nothing else changes.

**Creates.** None.

**Blockers.** None. BLK-007 stays open and is untouched. BLK-003 is resolved,
so the population the reservation defaults to is answered.

**Precedent.** FND-135 holds what the world reserved and what the project
believed it reserved. FND-144 holds that a refused founding is not
hash-neutral. FND-145 holds the price of the settings field. All three were
opened by the work this record accompanies, so all three are tests the record
must pass rather than material it may cite loosely.

## Done when

- An agent that did not write ADR-0084 has read all four decisions against the
  arena, the world, the founding and the reservation test.
- The review states what it tried to reject, and why each objection failed or
  held.
- The review states whether the reservation has one declaration site.
- The review states whether the record extends the open row that holds the
  settlement and character arenas.
- The review states whether the record claims a refused founding leaves no
  trace.
- The review names every claim that only a run can settle, and marks it
  unverified.
- The registry row holds the outcome.

## Outcome

**ADR-0084 stays at `Draft`.** All four decisions hold against the code. Two
sentences do not. The review holds the detail and the replacement text for
each.[^1]

**Nothing was compiled.** Three other workers held the machine, so no `cargo`
command ran. The review names the claims that rest on a run and marks each
unverified. The six document checks were run, because they compile nothing,
and all six pass.

**Eight objections were attempted. Six failed.** The record cites the scale
constants table rather than naming a population, it leaves the settlement and
character arenas to the open row, its title states one claim, its hash claim
matches the hash function, and its length is below both reference medians.

**Two sentences must change.** One says a refused founding leaves nothing
behind, and FND-144 records that it leaves open slots and advanced
generations, which the state hash covers. One says the code states no
population of its own, and the code states one million as a literal.

**The review opened FND-168.** The phrase that FND-144 corrected is still
alive in the doc comment above the function that undoes a founding.

Register entries that moved: FND-168 opened. No blocker opened or closed, and
BLK-007 is untouched. No decision row was needed.

## References

[^1]: Review 0175, the unit reservation record. `docs/reviews/0175-the-unit-reservation-record.md`
