---
id: 0101
title: Assert the terrain gradient of a holding
status: proposed
created: 2026-09-01
implements: [ADR-0053 D2, ADR-0004 D1]
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

Flatten the claim threshold to one value for every passable kind and the test
stays green. The gradient survives on the constants alone.[^2]

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

Not done. This item is `proposed/` and refining it is the work.

The reviewer must answer how many ticks the count needs to be stable, and
whether the assertion is on the order of the rates or on a bound for each
kind. An order is the weaker claim and the more durable one, because it does
not move when a threshold is tuned.

The reviewer must also decide whether this test belongs beside the holding
tests or in its own file, given that it runs a world for many ticks.

**Blockers.** BLK-007 governs every cost figure, so this item states none. The
tick count is a parameter of the test, not a figure about the target.

**Precedent.** FND-080 records this instance. FND-075 and FND-078 record the
same shape in two other subsystems: one fact checked in a place that cannot
see it go wrong.[^2] [^3] [^4]

## Done when

- The test counts offered and taken tiles for each passable terrain kind.
- The test asserts the order of the conversion rates.
- The claim threshold is flattened to one value for every passable kind, the
  test fails, and the commit body says so.
- The test asserts that it reached hill and mountain, and fails when a fixture
  offers neither.
- `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: PRD-0006, a place belongs to somebody. `docs/product/accepted/prd-0006-a-place-belongs-to-somebody.md`
[^2]: Findings register, FND-080. `docs/FINDINGS.md`
[^3]: Findings register, FND-075. `docs/FINDINGS.md`
[^4]: Findings register, FND-078. `docs/FINDINGS.md`
