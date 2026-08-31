---
id: 0058
title: Build an improvement over several ticks
status: refined
created: 2026-08-31
implements: [ADR-0066 D1, ADR-0066 D2, ADR-0002 D1, ADR-0002 D3, ADR-0004 D2]
changes: []
creates: []
serves: [PRD-0008]
blocked-by: [0053]
---

## Why

The world holds no memory of anything a unit did to it. Every tick starts from
the state the generator made. Work that takes many ticks cannot exist, so
nothing a unit does can be large, and holding ground for a long time gains
nothing.

This item lets a unit leave a mark. It is the fourth of the four fixed entity
shapes, and it is the last one that no code reaches.

## What the work does

1. A tile can carry an upgrade that no generator placed. Storage grows with
   the number of upgrades, not with the number of tiles.
2. An upgrade under construction holds a progress accumulator. Several units
   contribute to it and the contributions add exactly.
3. Unfinished work persists between ticks.
4. A finished upgrade changes what the tile yields or what it costs to cross.
5. An upgrade can be destroyed, and the tile returns to what it was.

## Impact review

**Governed by.**

- ADR-0066 D1. The tile upgrade is one of the four shapes: sparse, attached to
  a tile, not to a mobile entity.
- ADR-0066 D2. Creating and destroying an upgrade is a structural change, so
  it goes through the batched path.
- ADR-0002 D1 and D3. Progress is an exact integer and the accumulator widens.
  **FND-011 records that the progress accumulator overflows**, so this is the
  named case rather than a general caution.[^1]
- ADR-0004 D2. Contributions from several units combine in any order, because
  integer addition is order-free.
- ADR-0056 D4 states that capacity is a data-driven property of the terrain.
  An upgrade that changes what a tile costs to cross changes that property, so
  it must go through the same data path and not through a second one.[^2] [^3]
  [^4]

**Changes.** No record changes.

**Creates.** No record. The sparse storage decision is already made: BLK-006 is
resolved, fewer than one tile in twenty carries an upgrade, and tile upgrades
therefore use sparse storage with one indirection on read.[^5] Row 0015
reserves the claim for a narrow tile column with sparse side tables, and this
work is a reader of that claim rather than a second one.[^6] If the work finds
a decision that neither holds, **that record is a deliverable of this item and
the number comes from the registry**, not from the author.

**Blockers.** BLK-007 governs every cost figure, so this item states none. The
upgrade fraction comes from the scale constants table.[^7]

**Precedent.** FND-011 gives the exact defect this work must not repeat.[^1]
Shape 3 of the recurring defect rule applies: an upgrade that nothing builds
and nothing reads is an inert capability, so the test starts at the engine and
drives a unit that builds.[^8]

**Serves.** PRD-0008. It does not decay an upgrade without upkeep; PRD-0008
sends that to production and upkeep, and item 0055 holds the rate mechanism
that would carry it.

**Conflict surface.** `crates/cachette-core/src/upgrade.rs` is new.
`crates/cachette-core/src/world.rs` at the step, the state hash and the
invariant check; `crates/cachette-core/src/terrain.rs` at the crossing cost
read; `crates/cachette-view` gains an upgrade layer. **It touches the terrain
read that movement uses**, so it does not run beside any item that changes
movement. No other item in this plan touches `upgrade.rs`.

## Done when

- A tile carries an upgrade that no generator placed, and a world with no
  upgrades stores none.
- Two units contribute to one upgrade and the progress is the exact sum, at 1,
  2 and 12 threads.
- A property test asserts that the accumulator does not overflow at the
  largest progress the content can reach, and the test names the bound.
- Unfinished work persists. A test stops a unit, restarts it, and asserts that
  the work continued rather than restarted.
- A finished upgrade changes what the tile yields or costs, and the change is
  read through the same path that reads the terrain.
- Destroying an upgrade returns the tile to the value it had, and a test
  asserts the exact return.
- Advancing construction costs the sites under construction. A test with one
  site in a large world runs in the same time as one site in a small world, or
  the item says why that test was not written.
- The engine drives the build. No test constructs an upgrade directly and
  calls it covered.[^8]
- No cost figure appears in the code or in a comment.
- `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-011. `docs/FINDINGS.md`
[^2]: ADR-0066, entity storage holds four fixed shapes. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^3]: ADR-0002, simulated and aggregated state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^4]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^5]: Blockers register, BLK-006. `docs/BLOCKERS.md`
[^6]: ADR Registry, row 0015. `docs/adrs/REGISTRY.md`
[^7]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^8]: Recurring defect shapes, shape 3. `.claude/rules/recurring-defects.md`
