---
id: 0421
title: Put the two quantity passes into the stage table
status: refined
created: 2026-09-03
implements: [ADR-0001 D4, ADR-0009 D1, ADR-0062 D5, ADR-0128 D3]
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

The step opens a stage around each pass it runs, and the frame cost table reads
those stages. A pass that opens no stage is absent from every cost report the
project takes, and its cost falls silently into the gap between two neighbours.

**Two passes move a quantity and neither opens a stage.** The delivery pass
moves a carried load into the store of a unit's own site. The contract
settlement pass moves a carried load into the store of another faction's site
and then fails every contract that reached its deadline. A finding records
both.[^1]

Nothing fails today. The cost table adds up to less than a frame and no check
compares the two. A contributor who plans against the table plans against a
frame that is missing two passes.

The repair must add both at once. Adding one leaves the other, and a table that
covers one of two passes of one kind is harder to read than a table that covers
neither.

## Impact review

**Governed by.** ADR-0001 D4 holds that one binary gives one answer at any thread
count. The stage cost table uses atomic integers behind a crate feature, reads a
clock, and writes to statics that no simulation pass reads, so adding spans cannot
change a simulation result at any thread count.[^2] ADR-0009 D1 holds that
parallel stages write disjoint outputs.[^3] Neither the delivery pass nor the
contract settlement pass takes a thread count. Both run on the calling thread and
write one store at a time in a deterministic order, so both declare `false` for
the threaded column.[^4] ADR-0062 D5 holds that production and upkeep are rates
attached to a site, and that the delivery pass moves quantities into site stores
before the rate pass and consumption pass run.[^5] ADR-0128 D3 holds that a
contract moves a quantity only when a unit carries it onto the ground of the
other party, running directly after ordinary delivery and before the rate
pass.[^6]

**Changes.** None to decision records. The stage table is an internal
instrumentation table rather than a separate architectural claim.

**Creates.** None. No new decision record is needed.

**Serves.** PRD-0002 (A developer watches the world run: stage-level profiling
and pass accounting accuracy).[^7]

**Blockers.** None (`blocked-by: []`).

## Done when

- `declare_stages!` in `crates/cachette-core/src/stage.rs` declares `Deliver` and
  `TradeSettle` with `takes_threads = false`, `entries_for_each_frame = 1`, and
  `is_nested = false`.
- `crates/cachette-core/src/world/step.rs` opens `Stage::Deliver` around
  `self.deliver(threads)` and `Stage::TradeSettle` around
  `self.settle_trades(threads)`.
- The existing test `one_frame_opens_every_declared_stage_exactly_as_often_as_it_declares`
  in `crates/cachette-core/tests/stage_cost.rs` passes with both stages present.
- Integration tests in `crates/cachette-core/tests/stage_cost.rs` assert that
  `Stage::Deliver` and `Stage::TradeSettle` record exactly one entry per frame
  and have non-nested status.
- A test proves able to fail when either stage span is omitted.
- The whole check command runs clean.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-431. `docs/FINDINGS.md`
[^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^3]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^4]: The stage cost table. `crates/cachette-core/src/stage.rs`
[^5]: ADR-0062, production and upkeep are rates attached to a site, decision D5. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
[^6]: ADR-0128, a contract moves a quantity only when a unit carries it onto the ground of the other party, decision D3. `docs/adrs/draft/adr-0128-a-contract-moves-a-quantity-only-when-a-unit-carries-it.md`
[^7]: PRD-0002, a developer watches the world run. `docs/product/shipped/prd-0002-a-developer-watches-the-world-run.md`
