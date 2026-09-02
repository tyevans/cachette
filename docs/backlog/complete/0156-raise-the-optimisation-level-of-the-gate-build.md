---
id: 0156
title: Raise the optimisation level of the gate build and keep the overflow check
status: complete
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

The development profile now states an optimisation level, an overflow check
and the debug assertions. The gate suite runs several times faster, and the
overflow check is unchanged. The figures are in the commit body and in the
development budget register.

Three things changed from the plan.

**The optimisation level chosen is the lowest one, not the highest.** Every
level above zero lands within a few seconds of the others on execution, and
the lowest is the cheapest to compile. The plan had assumed the trade would
run the other way.

**Optimising the dependencies alone does nothing, and measuring it was worth
the time.** The reasoning said it would do nothing, because the simulation is
first-party code. The first measurement appeared to say it made the suite half
again as slow, which was false and led to FND-142.

**The work found a false example in a hard invariant of the project.** A
one-byte tile field at its largest value, summed over the target scale, does
not overflow a `u32`. It fits with under one part in a hundred to spare. The
rule that the accumulator widens is right and is untouched. FND-141 holds the
arithmetic. The document that states the example belongs to the project owner,
so this item does not edit it.

Register entries that moved: FND-140, FND-141 and FND-142 opened. ADR-0083 was
written and sits at `Draft`. No blocker opened or closed, and BLK-007 is
untouched. No decision row was needed, because the choice is made and the
record holds it.
