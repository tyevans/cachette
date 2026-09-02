---
id: 0101
title: Assert the terrain gradient of a holding
status: complete
created: 2026-09-01
implements: [ADR-0053 D5, ADR-0004 D1]
changes: []
creates: []
serves: [PRD-0006]
blocked-by: []
---

## Why

A product record states that terrain influences holding, and that a holding
does not spread the same way across every kind of ground.[^1] The behaviour is
real. The test named for it does not defend it.

The test asserts that water is never held. It then asserts that the claim
threshold of plain is below hill, and hill below mountain. That second
assertion reads three constants and exercises no behaviour. The test also
computes the held share for each terrain kind and discards the result without
asserting on it.

The gradient survives on the constants alone. This item first stated that
flattening the claim threshold to one value for every passable kind leaves the
test green. That is wrong, and the work corrected it: the flattened thresholds
fail the old test on the line that compares two constants, which says nothing
about the behaviour. Stop the rule from reading the threshold and leave the
constants ordered, and the old test passes with the gradient gone.[^2] [^7]

A review measured the behaviour directly over 40 ticks. It recorded the
terrain of every unheld tile beside a holding, stepped the world, and recorded
whether the tile was taken. Plain took every tile offered. Hill took about a
third. Mountain took a small fraction. Water took none. The gradient is in the
system, and no assertion holds it there.

## What the work does

1. The test counts outcomes rather than reading constants. For each passable
   terrain kind it counts the tiles offered to a holding and the tiles taken.
2. The test asserts the order of the conversion rates across the kinds, not
   the order of the thresholds.
3. The test states the case it reached, so a fixture that offers no hill and
   no mountain fails rather than passes.
4. The discarded per-kind shares are asserted on or removed. A computed value
   that nothing reads is a comment that the compiler checks.

## Impact review

**Governed by.** ADR-0053 D5 states that the ground decides how much support a
claim must raise, and that open water admits no holder. ADR-0004 D1 requires
an explicit iteration order, which the measurement respects by reading the
offer before the step and the outcome after it. ADR-0002 D1 forbids a floating
point number in the crate, so the comparison of two shares cross-multiplies
rather than divides.

**Changes.** None. This item changes a test. The holding rule is untouched.

**Creates.** No decision record. The three-condition test of the scope rule
answers no on the second condition: the choice between an order and a bound is
a choice inside one test file, and changing it later costs one edit.[^5] The
reasoning sits in the test beside the assertion, which is condition three.

**Blockers.** None. BLK-007 governs every cost figure, and this item states
none. The tick count is a parameter of the test.

**Precedent.** FND-080 records the instance. FND-075 and FND-078 record the
same shape in two other subsystems: one fact checked in a place that cannot see
it go wrong.[^2] [^3] [^4]

### The three questions

**How many ticks the count needs to be stable.** Twenty. The fixture was
measured for sixty ticks. The order of the four shares first holds at tick 5
and holds at every tick from there to tick 60. Twenty sits inside that band
with margin on both sides, and it leaves every kind a count in the hundreds.
The figure is a property of this fixture on a development machine. It is not a
figure about the target, so no blocker governs it.

**Order, or a bound for each kind.** Order. The rule states an order and
nothing else: each step upward asks for one more supporter. A bound would fix
the fixture as well as the rule, because the share a kind gives up depends on
how many of its tiles the run reached and how many neighbours they had. Tuning
a threshold moves every bound and moves no order. The order is therefore the
claim the rule makes, and the durable one.

**Its own file, or beside the holding tests.** Beside them. The measurement
reuses the world fixture, the address list, the garrison helper and the frame
runner. A second file cannot share those without a third site to hold them,
and one fact in more than one place is the shape this project keeps
correcting.[^6] The run costs about three seconds in a debug build, which does
not pay for a split.

## Done when

- The test counts offered and taken tiles for each passable terrain kind.
- The test asserts the order of the conversion rates.
- The rule is stopped from reading the claim threshold, the test fails, and
  the commit body says so. **This replaces the flattening experiment the item
  first named.** Flattening the thresholds makes the old test fail on the line
  that compares two constants, which proves nothing about the behaviour. The
  finding records the correction.[^7]
- The test asserts that it reached every passable kind, and fails when a
  fixture offers none of one.
- `just check` runs green.

## Outcome

**The test counts rather than reads.** For each tick it records every passable
tile that lies open beside a holding, steps the world, and records whether the
tile was taken. A tile a soldier stands on after the step is counted in
neither column, because presence outweighs the six neighbours together and
would measure the garrison rather than the ground.

**The assertion is the order of the four shares.** Level ground gives up more
than forest, forest more than hill, and hill more than mountain. The
comparison cross-multiplies the two counts, so it holds no floating point
number. The three constant reads are gone, and so are the shares that the old
test computed and discarded.

**The fixture was built for the distribution, not copied.** A survey of the
world took the address whose neighbourhood holds the four passable kinds in
the most even mix, and the address with the most hill and mountain. A garrison
starts at each. The world of the demonstration binary offers no mountain
inside the run, and the test fails against it by name.

**The old test kept the part that was behavioural.** The water refusal is now
its own test, with the shore fixture that reaches it.

**Two experiments, and a correction.** Flattening the claim threshold fails
the new test. It would also have failed the old one, through the constant
comparison rather than through any behaviour, so it is not the experiment that
separates them. Leaving the thresholds ordered and stopping the rule from
reading them fails the new test and passes every assertion the old one made.
The finding records this.[^7]

**The register moved.** One finding opened. No blocker and no decision changed,
and no record was written.


## References

[^1]: PRD-0006, a place belongs to somebody. `docs/product/accepted/prd-0006-a-place-belongs-to-somebody.md`
[^2]: Findings register, FND-080. `docs/FINDINGS.md`
[^3]: Findings register, FND-075. `docs/FINDINGS.md`
[^4]: Findings register, FND-078. `docs/FINDINGS.md`
[^5]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^6]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^7]: Findings register, FND-133. `docs/FINDINGS.md`
