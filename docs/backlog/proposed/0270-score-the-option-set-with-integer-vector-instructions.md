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

**The toolchain decides which vector paths are available, and the portable one
now is.** The project pinned a stable release when this item was written, and
the portable vector library is not in stable. The project has since moved to a
dated nightly, and that move was made for this item.[^5] [^6] Four routes are
open and they are not equal.

1. **Let the compiler vectorise the loop.** This needs no new interface, no
   unsafe code and no target-specific code. It works on every target the
   project builds for, including the development machines. It is not
   guaranteed: a loop vectorises or it does not, and a later compiler may
   change its mind.
2. **Write the pass against the portable vector library.** This states the
   width in the source rather than hoping the compiler finds it, it is safe
   code, and it works on every target. It needs a feature attribute, and the
   library carries no stability promise, so a later date may change its shape.
3. **Write target-specific intrinsics for the target platform.** These are in
   the standard library for the target architecture and are stable, but they
   are unsafe, they are specific to one architecture, and they need a scalar
   path for every other target and a check that the two agree.
4. **Pin a newer processor generation to reach the wider vector unit.** This
   decides which machines the engine runs on and is an architectural decision,
   not an implementation detail.

**This item still takes route 1 first, and route 2 is now the fallback rather
than route 3.** Shape the pass so that the compiler can vectorise it, and
measure whether the compiler did. If it did not, route 2 is the next step: it
buys the guarantee that route 3 buys, and it costs no unsafe code and no second
implementation. Route 3 should now be argued from a measurement showing that
route 2 was not enough as well. Route 4 needs a record.

## What the work does

Shape the scoring loop so that the compiler can vectorise it: a fixed trip
count, no branch inside the body, and inputs read from contiguous slices.
Then measure whether it did.

## What good looks like

The scores are identical to the scores before the change, byte for byte, for
a large sample of units. The determinism tests pass at every thread count. A
measurement on the target platform says what the pass costs before and after.

**The result may be that the compiler does not vectorise it.** That is a
result and the item is done when it is recorded. A loop shaped for a vector
unit that no vector unit reaches is still a loop with fewer branches in it.

An equality assertion is required for route 2 and not for route 1, because
route 1 keeps one implementation and the compiler is obliged to preserve its
meaning. If this item ever grows a second implementation, it grows the
equality test with it.

## What it does not do

It does not add unsafe code, target-specific code, or a second
implementation. It does not pin a processor generation.

It does not use the portable vector library. Enabling that library was the
toolchain move, and it is done.[^6] Using it is route 2, and route 2 is what
this item falls back to when the measurement says route 1 was not enough.

It does not vectorise any other pass. The choice pass is the one with a fixed
option count and a shared profile.

It does not remove the scattered reads that feed the pass. Three items do
that, and this one gains little until they land, because a vector unit fed one
scattered value at a time is a scalar unit with extra steps.[^4]

## References

[^1]: ADR-0012, tiles are dense columns and units are a generational arena. `docs/adrs/accepted/adr-0012-tiles-are-dense-columns-and-units-are-a-generational-arena.md`
[^2]: Target platform costs, the stage split. `docs/reference/graviton-costs.md`
[^3]: ADR-0002, state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^4]: Backlog item 0266, order the unit arena by cell. `docs/backlog/refined/0266-order-the-unit-arena-by-cell.md`
[^5]: The pinned toolchain. `rust-toolchain.toml`
[^6]: ADR-0097, the toolchain is a dated nightly. `docs/adrs/draft/adr-0097-the-toolchain-is-a-dated-nightly.md`
