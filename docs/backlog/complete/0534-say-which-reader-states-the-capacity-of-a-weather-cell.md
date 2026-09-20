---
id: 0534
title: Say which reader states the capacity of a weather cell
status: complete
created: 2026-09-09
implements: [ADR-0141 D1, ADR-0141 D2, ADR-0162 D2]
changes: []
creates: []
serves: [PRD-0004]
blocked-by: []
---

## Why

**Two places state what the air over a weather cell can hold, and they answer
different numbers.** The published reader answers the capacity of air at rest
over the cell, which follows the temperature of that cell alone. The settle pass
bounds the air at a travelling capacity, which takes the same temperature and
then applies the cooling the air met and the slope it went up. The slope term is
signed, because air that descends warms and holds what it carries, so the
travelling capacity can stand above the published one.

**A test asserts the wrong one of the two, and it is red today.** It reads the
whole plane after twenty steps and asks that no cell stand above its own
published capacity. One cell stood 23 drops above 1526, with no storm over it. A
finding holds the measurement.[^1]

**The test is right to fail and it must not be relaxed.** It asserts a bound
that nothing promises. A test changed to match a picture that fails its own rule
states something false with the authority of a test.

**Nothing is visibly wrong in the picture.** The reader that turns the pair into
a share of a sky clamps its answer, so a watcher never sees more than a whole
sky. The clamp is why this stood unread.

**This is one value with two declaration sites.** Nothing fails when the two
disagree, which is the shape this project keeps meeting.[^2]

## The architectural impact review

**The records that govern this work.** ADR-0141 D1 and D2 hold that a weather
pass moves water and never scales it, so any repair must keep the water account
exact.[^3] ADR-0162 D2 holds where water enters the air and where it falls, and
its second decision is the one that makes the slope term signed.[^4] PRD-0004
states the need for weather that a watcher can read.[^5]

**The records this work changes.** None. The work operates inside ADR-0141 and
ADR-0162 and respects their claims.

**The records this work creates.** None. The distinction between rest capacity
and travelling capacity operates inside the accepted weather architecture.

**The blockers that hold it.** None. Every figure the work needs is a property
of the model rather than a cost.

**Serves.** PRD-0004.

## What the work does

1. Clarifies that the phrase "capacity of a cell" at rest is stated by
   `capacity_at_cell(&self, cell: u32) -> Drops`, which derives from the warmth
   of that cell alone.
2. Adds `settle_capacity_at_cell(&self, cell: u32) -> Drops` and
   `settle_capacity_plane(&self) -> &[Drops]`, which state the travelling bound
   applied by the settle pass. That bound combines warmth, cooling, slope/climb,
   and cyclone depression.
3. Records `settle_capacity: Vec<Drops>` as derived state on `WeatherField`
   during `settle`. It enters no state hash.
4. Repairs the doc comment of `capacity_at_cell`, which said that air which
   cooled or climbed holds less than the figure it returns and did not mention
   descent.
5. Repairs `the_air_never_stands_above_the_capacity_of_its_own_cell()` in
   `crates/cachette-core/tests/weather.rs` to assert that air never stands above
   `settle_capacity_at_cell(cell)`.
6. Adds a companion test proving that descending air can stand above
   `capacity_at_cell(cell)` while respecting `settle_capacity_at_cell(cell)`.

## What good looks like

One reader states the bound the settle pass applies. A test asserts that bound
over the whole plane and passes. The doc comment of each reader says which of
the two figures it answers. The water account still balances and the state
hashes do not move, because the work changes what is published and not what the
solve computes.

## What it does not do

It does not change the settle pass, because the measurement says it is behaving
as its own record describes.

It does not change the cloud share reader. That reader clamps its answer, and
the clamp is correct whichever figure it divides by.

## Done when

- `settle_capacity_at_cell` and `settle_capacity_plane` exist on `WeatherField`.
- `capacity_at_cell` doc comment states that it answers rest capacity and notes
  that descending air warms and holds more.
- `the_air_never_stands_above_the_capacity_of_its_own_cell()` asserts against
  `settle_capacity_at_cell` and passes.
- A test proves that descending air can exceed `capacity_at_cell` while
  respecting `settle_capacity_at_cell`.
- All priority checks pass.

## Outcome

Complete. `settle_capacity_at_cell(&self, cell: u32) -> Drops` and
`settle_capacity_plane(&self) -> &[Drops]` publish the bound applied by the
settle pass as derived state. The doc comment on `capacity_at_cell` states that
it answers rest capacity and explicitly notes that descending air warms and holds
more. `the_air_never_stands_above_the_capacity_of_its_own_cell()` asserts
against `settle_capacity_at_cell` and passes, and
`descending_air_can_exceed_rest_capacity_within_settle_capacity()` proves the
failure mode by demonstrating that descending air legitimately exceeds rest
capacity while respecting settle capacity. State hashes and the water account are
unchanged.

## References

[^1]: Findings register, FND-730. `docs/FINDINGS.md`
[^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
[^3]: ADR-0141, a weather pass moves water and never scales it, decisions D1 and D2. `docs/adrs/draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md`
[^4]: ADR-0162, water enters the air where it is hot, and it falls where the air cools, decision D2. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
[^5]: PRD-0004, the world has weather that a watcher can read. `docs/product/accepted/prd-0004-the-world-has-weather-that-a-watcher-can-read.md`
