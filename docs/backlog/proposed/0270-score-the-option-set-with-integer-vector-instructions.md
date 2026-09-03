---
id: 0270
title: Score the option set with integer vector instructions
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

**The choice pass is the shape a vector unit is built for, and it runs one
unit at a time.** It scores a fixed set of four options, reads one weight
profile that every unit shares, multiplies a drive by a weight and a field
value, and takes the highest score. The columns it reads are already a
structure of arrays.[^1] The choice is 71 milliseconds of the unit cost at the
target scale.[^2]

**The invariant against floating point is what makes this safe.** Float
addition is not associative, so a vector reduction over floats gives a
different answer for a different lane count, and a golden state hash would
refuse it. Every quantity here is integer or fixed point in the Q16.16
scale.[^3] Integer arithmetic is exact and associative, so a vector result is
bit-identical to the scalar result at any width, in any lane order.

**The constraint that looks like a cost is what buys this.** A simulation that
held floats could not vectorise this pass and keep a state hash.

**The target platform has two vector paths and they are not the same choice.**
One is portable across every generation of the target. The other is wider and
exists only on the later generations. Choosing the wider one decides which
machines the engine runs on, which is an architectural decision and not an
implementation detail. **This item does not make that choice.** It measures
the portable path first, because a gain there is available on every target and
needs no decision from anybody.

## What the work does

Score the option set for several units at once, on the portable vector path.
Keep the scalar path and prove the two agree.

## What good looks like

A test asserts that the vector path and the scalar path give identical scores
for a large sample of units, byte for byte, not within a tolerance. The
determinism tests pass at every thread count. A measurement on the target
platform says what the pass costs before and after.

**The equality test is the deliverable, not the speed.** A vector path that is
faster and disagrees anywhere is a defect that the golden hash will find later
and expensively.

## What it does not do

It does not choose the wider vector path or pin a processor generation. If the
portable path pays, a record decides whether the wider one is worth the
narrowing.

It does not vectorise any other pass. The choice pass is the one with a fixed
option count and a shared profile.

It does not remove the scattered reads that feed the pass. Three items do
that, and this one gains little until they land, because a vector unit fed one
scattered value at a time is a scalar unit with extra steps.[^4]

## References

[^1]: ADR-0012, tiles are dense columns and units are a generational arena. `docs/adrs/accepted/adr-0012-tiles-are-dense-columns-and-units-are-a-generational-arena.md`
[^2]: Target platform costs, the stage split. `docs/reference/graviton-costs.md`
[^3]: ADR-0002, state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^4]: Backlog item 0266, order the unit arena by cell. `docs/backlog/proposed/0266-order-the-unit-arena-by-cell.md`
