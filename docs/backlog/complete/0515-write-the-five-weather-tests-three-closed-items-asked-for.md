---
id: 0515
title: Write the five weather tests three closed items asked for
status: complete
created: 2026-09-06
implements: [ADR-0160 D2, ADR-0160 D3, ADR-0161 D1, ADR-0162 D1, ADR-0162 D2]
changes: []
creates: []
serves: [PRD-0004]
blocked-by: []
---

## Why

**Three items closed with the engine work done and the evidence they demanded
never written.** Each named a test in its own acceptance, each shipped without
it, and each outcome now says so plainly.[^1] [^2] [^3] This item is the debt.

The engine work is real and the records that govern it are written. What is
missing is the set of assertions that would fail if somebody undid the work.

**Five assertions are missing.**

1. The wind lags the pressure. A record states that the acceleration step is
   what makes the wind lag, and that the lag is the whole reason the field
   carries the wind at all.[^4]
2. Drag brings a wind to rest. The same record states that drag settles the
   wind where the step and the share balance.[^5]
3. The air total is unchanged over random winds and random air planes. The item
   that carried the water on the wind asked for this as a property test, and
   asked that the isotropic share be put back and the failure recorded.[^2]
4. Some cells are wet and some are dry at one tick, with no storm raised. **This
   is the exact statement the original measurement said was false**, so it is
   the assertion the whole rain item existed to make true.[^3]
5. The near side of high ground holds more water than the far side.[^3]

## What the work does

1. Add `crates/cachette-core/tests/weather_evidence.rs` containing the five
   assertions.
2. Assert that wind lags pressure: a cell whose pressure gradient drops to zero
   retains momentum decayed only by drag on the next tick.
3. Assert that drag brings wind to rest under no pressure gradient, monotonically
   reducing speed until `Wind::STILL` is reached.
4. Assert via `proptest!` that advective transport conserves air drops exactly
   across arbitrary wind vectors and air planes.
5. Assert that a coastal world without storms develops heterogeneous ground
   wetness where `0 < wet_cells < total_cells`.
6. Assert that orographic precipitation leaves more ground water on the near
   (windward) side of high ground than on the far (lee) side.

## Impact review

**Governed by.** ADR-0160 D2 and D3 govern wind acceleration, momentum lag,
and drag bleeding.[^4] [^5] ADR-0161 D1 governs directional advection and
integer conservation.[^6] ADR-0162 D1 and D2 govern wet mark thresholds,
cooling, and orographic precipitation.[^7]

**Property vs example partition.**
- Assertion 3 is a property test (`proptest!`) verifying exact drop conservation
  over arbitrary valid air and wind planes.
- Assertions 1, 2, 4, and 5 are deterministic example tests exercising the
  physical invariants through the simulation steps.

**Interface access.**
- `WeatherField` exposes `transport`, `settle`, plane setters, and `drag`.
- `World::with_weather_scale` drives the full simulation for world-level
  wetness and wind dynamics.

**Cost.**
- Each test runs in <= 48 ticks on small lattices, completing the entire suite
  in well under one second.

## Done when

- Each of the five assertions exists as a test that drives the world step or
  weather solve pass.
- Each test is proved able to fail, by putting the defect back and recording in
  the commit body that the test then went red.
- The commit body names which of the five are property tests and which are
  example tests, and why.
- No test asserts that something moved. Each states a count, a share or an
  equality, so that a partial failure fails.

## Outcome

**Complete.** The integration suite `crates/cachette-core/tests/weather_evidence.rs`
asserts all five physical invariants:
- `wind_lags_pressure`: wind persists on the tick following pressure gradient
  cessation (decayed only by drag), proving wind is carried momentum state (ADR-0160 D2).
- `drag_brings_wind_to_rest`: under zero pressure gradient, drag bleeds momentum
  monotonically to rest within a finite tick count (ADR-0160 D3).
- `air_total_conserved_over_random_winds_and_air_planes`: `proptest!` property test
  asserting exact drop conservation across directional advection (ADR-0161 D1).
- `some_cells_wet_and_some_dry_without_storm`: coastal world without storms develops
  heterogeneous ground wetness with `0 < wet_cells < total_cells` (ADR-0162 D1).
- `near_side_of_high_ground_holds_more_water_than_far_side`: climbing windward air
  sheds capacity while descending lee air warms, leaving more ground water on the
  near side (ADR-0162 D2).

Each test was proved able to fail by reintroducing its defect.

## References

[^1]: Backlog item 0499. `docs/backlog/complete/0499-give-every-level-1-cell-a-wind-that-carries-its-momentum.md`
[^2]: Backlog item 0500. `docs/backlog/complete/0500-carry-the-air-water-along-the-wind.md`
[^3]: Backlog item 0501. `docs/backlog/complete/0501-make-rain-arrive-rather-than-sit-everywhere.md`
[^4]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D2. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
[^5]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D3. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
[^6]: ADR-0161, water rides the wind, and every transfer is an exact integer move, decision D1. `docs/adrs/accepted/adr-0161-water-rides-the-wind-and-every-transfer-is-an-exact-integer-move.md`
[^7]: ADR-0162, water enters the air where it is hot, and it falls where the air cools, decisions D1 and D2. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
