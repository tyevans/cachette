---
id: 0299
title: Make the bridge rebuild cost less than a sixth of a frame
status: complete
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
The bridge rebuild was measured at 31.4 ms of a 177.9 ms frame on the target
platform, accounting for nearly a sixth of the total frame time.[^1] The ordering
pass alone represented 63% of that time, driven by indirect index sorting and an
18% permutation gather pass.[^2]

ADR-0071 D2 fixes the rebuild to a single thread to preserve determinism and
avoid thread contention.[^4] DEC-111 provides the path forward: replace the full
sort and separate uniqueness scan with a direct radix sort on key-unit pairs with
adjacent-key deduplication.[^3]

## Impact review

**Governed by.** ADR-0018 D3 establishes that the bridge is derived at the barrier.
ADR-0071 D2 requires single-threaded ordering. DEC-111 governs the radix sort and
adjacent deduplication.

**Changes.** None to decision records.

**Creates.** None.

**Blockers.** DEC-111 is closed by this implementation.

**Precedent.** FND-301 recorded the 31.4 ms rebuild measurement and identified
indirect index sorting as the primary cost driver.

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

Adopted DEC-111 and replaced the indirect sorting pass with a direct radix sort on (key, unit) pairs, writing directly into destination memory during the final radix cycle to eliminate the permutation gather pass. Folded block range indices into the digit 0 radix histogram pass, and eliminated the duplicate sort pass by tie-breaking entity bits within matching tile runs and inspecting adjacent keys. All 9 CI checks passed cleanly. Merged in PR #69.

## References

[^1]: Target platform costs, every stage of a frame. `docs/reference/graviton-costs.md`
[^2]: Findings register, FND-301. `docs/FINDINGS.md`
[^3]: Decisions register, DEC-111. `docs/DECISIONS.md`
[^4]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^5]: PRD-0002, a developer watches the world run. `docs/product/shipped/prd-0002-a-developer-watches-the-world-run.md`
