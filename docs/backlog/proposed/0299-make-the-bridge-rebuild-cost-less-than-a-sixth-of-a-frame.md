---
id: 0299
title: Make the bridge rebuild cost less than a sixth of a frame
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

**The bridge rebuild is the largest single stage in the engine.** It costs
31,394,191 nanoseconds of a 177,862,658 nanosecond frame, which is 17.65
percent, at 16,777,216 tiles and 1,000,000 units scattered on 12 threads. Every
stage above it in the table divides into named parts that other items hold. This
one does not.[^1]

**It takes no thread count, and an accepted record is why.** ADR-0071 D2 gives
the rebuild one thread, and it argues that splitting a radix histogram across
threads needs a combine in a fixed order and a placement from that combine, each
of which is a place where a result could take its order from a thread. The
record also says the rebuild "does not earn" a thread. That phrase is a claim
about cost, it was written before any measurement existed on the target
platform, and the measurement now contradicts it.[^2]

**Where the time goes.** From a probe on the development machine, at the target
scale, in the last of ten frames. Read the shares, not the figures: the machine
was loaded.

| Part | Share |
|---|---|
| Order the keys | 63 percent |
| Follow the permutation | 18 percent |
| Walk the arena and build the keys | 17 percent |
| Rebuild the block ranges | 2 percent |

## What is missing before this is refined

- **Whether the record changes.** The argument in ADR-0071 D2 is untouched by
  the measurement, so an item that parallelises the ordering pass must supersede
  the record rather than work around it. An item that makes the ordering pass
  cheaper on one thread does not.
- **What the ordering guard costs, separately from the ordering.** Every
  ordering pass sorts its whole key set a second time to check that no
  identifier repeats. The narrower property the engine needs is free after the
  radix. That is a decision with options and a recommendation in the register,
  and it may be most of the difference between this stage and the same stage
  without a guard.[^3]
- **Whether the counting pass and the block ranges are one thing.** The rebuild
  sorts by a key whose high part is the block, and then scans the sorted keys
  again to derive the start and length of every block. A counting pass by block
  produces both, and the ranges would then cost nothing.
- **Whether the permutation can be avoided.** The pass builds an order, then
  gathers the keys and the units through it, which is one scattered read for
  each unit. Writing the pairs directly during the last radix pass would remove
  the gather.
- **What a rebuild that is not whole-world would need.** The record requires a
  rebuild at the barrier rather than an incremental update, because the merge
  order of per-system writes is the nondeterminism the project cannot
  carry.[^4] An item that proposes anything incremental must meet that.

## Done when

Not yet stated. The item is not refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Target platform costs, every stage of a frame after admission stopped searching. `docs/reference/graviton-costs.md`
[^2]: Findings register, FND-301. `docs/FINDINGS.md`
[^3]: Decisions register, DEC-111. `docs/DECISIONS.md`
[^4]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
