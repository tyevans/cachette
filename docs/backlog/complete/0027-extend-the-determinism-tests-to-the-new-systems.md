---
id: 0027
title: Extend the determinism tests to the new systems
status: complete
created: 2026-08-30
implements: [ADR-0001 D5, ADR-0004 D1, ADR-0056 D3]
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

The two determinism tests cover a tile value update. They must also cover the
entity arena, the keyed behaviour draw, and the admission step.

The perturbed build must fail each new assertion. A determinism test with no
proven failure mode is decoration.

## What already covers what

**The entity arena.** The thread-count test runs a world that holds soldiers,
despawns part of them, and compares the event log, the state hash and the live
count. The perturbed build fails it.

**The keyed behaviour draw.** Two tests read the fields of the key rather than
the repeatability of the draw: one proves the frame is in the key, one proves
the entity generation is. The perturbed build has a third, which drops the row
from the terrain lattice key.

**The admission step.** The thread-count test gained a scenario whose units
contend for their targets, and the golden file gained a crowded scenario. This
is the gap: neither has a proven failure mode. The perturbation that exists
reverses the slot join, and admission sorts its intents, so the sort absorbs
the perturbation and the assertion cannot fail.

## What the work does

1. The perturbed build makes admission scan each target's intents in the order
   they arrived rather than in the sorted order.
2. A probe test asserts that the admitted set then differs between one thread
   and twelve.
3. The perturbed build must fail the thread-count test for this reason, and
   the probe test asserts that it does.

## Impact review

**Governed by.**

- ADR-0056 D3: admission sorts by a stable key and admits in that order. The
  perturbation is exactly the rule this decision states, removed.
- ADR-0004 D1: iteration order is explicit. The joined intent order is the
  slot order, and the existing perturbation reverses it, so an admission that
  reads the joined order inherits the thread-count dependence.
- ADR-0001 D5: one binary gives one answer at any thread count.

**Changes.** No record changes.

**Creates.** No record. The work adds a test-only switch and a test.

**Blockers.** None.

**Precedent.** The testing rule says a determinism test must be able to fail,
and that a test nobody has seen fail is a test nobody has checked.[^1]

## Outcome

The perturbed build makes admission sort by the target alone, so each target
still owns one contiguous segment and the order inside a segment is the order
the intents arrived in. The slot probe reverses the join order, which is
visible only above one thread, so who enters a full tile now follows the
thread count.

Two probe tests were added. One asserts that the positions differ between one
thread and twelve. The other asserts that the probe changed who was admitted
and not how many, because a probe that also changed the count would let the
thread-count test fail on the population rather than on the order.

The thread-count scenario whose units contend now fails under the perturbed
build, which is what the item asked for.

**The sound build absorbs the slot perturbation, and that is the point.** The
sort takes the order from the key values, so reversing the join changes
nothing. The perturbation had to remove the sort to be visible at all, which
is the clearest statement of what the sort buys.

## References

[^1]: Testing rules, sections 1 and 2. `.claude/rules/testing.md`
