---
id: 0299
title: Make the bridge rebuild cost less than a sixth of a frame
status: refined
created: 2026-09-03
implements: [ADR-0018 D3, ADR-0071 D2]
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

The bridge rebuild is the single largest stage in the engine, consuming
31.4 ms of a 177.9 ms frame (17.65%) at target scale (16.7 million tiles,
1,000,000 scattered units) on 12 threads.[^1] [^5]

Profilers show where this time is spent: 63% is ordering keys, 18% is following
the permutation gather, 17% is walking the arena to construct keys, and 2% is
rebuilding block ranges.[^2] Because ADR-0071 D2 requires the bridge rebuild to
execute on one thread to prevent thread-order nondeterminism, performance must
be achieved by optimizing the single-threaded algorithm rather than adding
threads.[^4]

## Impact review

**Governed by.** ADR-0018 D3 mandates that the unit-to-tile bridge is derived
and rebuilt deterministically at the frame barrier. ADR-0071 D2 assigns the
rebuild to one thread. DEC-111 evaluates duplicate checking strategies.

**Design choices.**
1. DEC-111 is resolved by checking uniqueness directly on adjacent sorted keys
   following the radix sort, removing the expensive full secondary sort pass.[^3]
2. The final radix pass writes (key, entity) pairs directly to destination
   memory, eliminating the separate 18% permutation gather pass.
3. Block range starts and counts are accumulated during the radix histogram
   pass, eliminating the post-sort block scan.
4. ADR-0071 D2 is preserved without modification: the pass remains single-threaded
   and strictly deterministic.

**Changes.** None to decision records.

**Creates.** None.

**Blockers.** None.

**Precedent.** FND-301 revealed that bridge rebuild was the largest unoptimized
stage in the engine frame budget.

**Conflict surface.** `crates/cachette-core/src/bridge.rs` and
`crates/cachette-core/src/sort.rs`.

## Done when

- Uniqueness checking is performed by adjacent inspection post-radix, eliminating
  the duplicate full sort pass.
- Key and entity pairs are written directly during the final radix cycle,
  eliminating the separate permutation gather.
- Block range indices are emitted during histogram calculation.
- Target scale bridge rebuild execution time decreases by at least 40% (measuring
  under 18 ms on 12-thread target profile).
- Event logs and world state hashes remain byte-for-byte identical across all test
  fixtures.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Target platform costs, every stage of a frame. `docs/reference/graviton-costs.md`
[^2]: Findings register, FND-301. `docs/FINDINGS.md`
[^3]: Decisions register, DEC-111. `docs/DECISIONS.md`
[^4]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^5]: PRD-0002, a developer watches the world run. `docs/product/shipped/prd-0002-a-developer-watches-the-world-run.md`
