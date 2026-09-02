---
id: 0156
title: Raise the optimisation level of the gate build and keep the overflow check
status: refined
created: 2026-09-01
implements: []
changes: []
creates: [ADR-0083]
serves: []
blocked-by: []
---

## Why

The project owner ran the gate suite and said it is too slow. It is. The
suite compiles the Rust tests in the development profile, and that profile
does not optimise. The engine steps a world of hundreds of thousands of
tiles, so unoptimised code is where nearly all of the wall clock goes.
Measurement puts the Rust tests at the great majority of the suite, and the
rest of the gates at a few seconds together.

Raising the optimisation level of the development profile is the fix. The
trap is that the obvious version of it, switching the gate to the release
profile, also turns off the integer overflow check. That check is a real net,
and losing it would be invisible.

## Impact review

**Governed by.** ADR-0001 D4 requires the two determinism tests, and both
must still pass under the new profile. ADR-0002 D3 requires an accumulator to
widen, and the overflow check is what catches an accumulator that does not.

**Changes.** None. No accepted record states a build profile.

**Creates.** ADR-0083, the gate build checks every integer overflow. The
registry row is allocated. The record states the constraint, never the level,
because the level is a cost figure and the register owns it.

**Blockers.** None. BLK-007 stays open and is untouched: no figure here is
evidence about the target platform, and the register that holds these figures
says so.

**Precedent.** FND-099 records that a contended run of this suite is not a
measurement. Every figure this item records comes from a machine with no
other work on it.

**Serves.** No product record. The need is a stated request from the project
owner, and the development budget register is what owns the cost.

## Done when

- The development profile states its optimisation level and states that it
  checks for overflow.
- A test overflows an integer on purpose and asserts that it panics, and the
  test fails when the check is turned off.
- A test asserts that an accumulator too narrow for a level panics, and that
  the widened accumulator holds the same sum exactly.
- The two determinism tests pass under the new profile.
- ADR-0083 exists, and the registry holds its row.
- The development budget register holds a row for the new cost, measured on
  an isolated machine, with the machine, the architecture, the profile and
  the date beside it.
- The whole check command runs green.

## Outcome

Filled in when the item moves to `complete/`.
