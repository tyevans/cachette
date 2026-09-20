---
id: 0279
title: Let a golden scenario reach the position pass
status: refined
created: 2026-09-02
implements: [ADR-0001 D4]
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**The position table folds into the state hash and no golden scenario reaches
the pass that writes it.** A change to who works where therefore moves no
golden file, and the guard that exists to notice a changed simulation notices
nothing.

This was measured rather than assumed. The seating pass was added, it seats
sixteen units in the world the demonstration builds, and every golden file
matched without being recorded again. Two reasons combine, and each is enough
on its own.

- The settling schedule has a default period of ten frames. The founding
  scenario is a wide scenario and runs eight, so it never reaches a settling
  frame.
- The scenarios that run long enough spawn their units directly rather than
  founding them, so no unit names a home site and no site has an applicant.

The golden state test is one of the two tests the project cannot lose.[^1] A
scenario set that misses a whole pass is the same weakness that item 0179
records for the build pass, seen in a second subsystem.[^2]

## What the tree already holds

**The scenario reaches the pass with real applicants, read on 5 September
2026.** The gathering scenario founds a settlement on a food deposit and homes
half the units it spawns. With the default schedule period, three ticks of the
run reach the seating pass, and the homed units are applicants. The scenario
stocks food only, so positions for the other kinds open and there is somewhere
to seat.[^3]

**Nothing asserts that a seat was written.** The scenario reaches the pass and
then says nothing about the result, so it measures the fixture. No mutation
proves that the golden file guards the seating.

**That assertion is the whole of the remaining work.** Put the defect back and
watch the test stay green, because that is the only proof the scenario reaches
the case.[^4]

## The same gap was found and closed for the promotion pass

The promotion work met this exactly. All eight golden files moved when it
landed, and none of the eight promoted anybody: the files moved because new
unit columns entered the state hash, which happens whatever the pass does.
FND-293 records it.[^5]

**The repair there was three lines in the scenario, and it is the shape this
item should copy.** The gathering scenario already states its own recovery
periods, on the stated ground that a period is a parameter of the kind and the
engine holds no test value. It now states its own promotion threshold in the
same place, at a value that world reaches, and asserts that it produced a
character. Two mutations confirm the file guards the behaviour.

## Impact review

**Governed by.** ADR-0001 D4 requires the golden state hash test.[^1] A
scenario that misses a pass cannot detect simulation changes in that pass.

**Changes.** None.

**Creates.** None.

**Blockers.** None.

**Serves.** None.

**Fixture choice.** The gathering scenario already founds a settlement and
assigns homes to units.[^3] All work commodities share one slot. The food store
exceeds the default target of 1.0, so the site opens no position without a
stated target. Setting a wood target above the store opens positions and seats
the units. The scenario runs 32 frames at the default 10-frame settling period.
The test needs no added frames, no schedule change, and no new scenario row.
The gate suite budget remains unaffected.[^6]

## Done when

- The gathering scenario in the golden state hash test asserts
  `census(&world, "seats_filled") > 0`.
- Suppressing unit seating causes the assertion to fail.
- The recorded golden state hash file matches without drift.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^2]: Backlog item 0179, give a golden scenario a build. `docs/backlog/complete/0179-give-a-golden-scenario-a-build.md`
[^3]: The gathering scenario of the golden state hash. `crates/cachette-core/tests/golden_state_hash.rs`
[^4]: Testing Rules, section 2a. `.agents/rules/testing.md`
[^5]: Findings register, FND-293. `docs/FINDINGS.md`
[^6]: Development budgets, the gate suite budget. `docs/reference/development-budgets.md`
