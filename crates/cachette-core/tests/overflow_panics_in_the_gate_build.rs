//! The gate build turns an integer overflow into a panic.
//!
//! The gate compiles this crate in the development profile, and that profile
//! checks every integer operation for overflow. The check is a safety net,
//! and a faster profile takes it away. These tests fail when it goes.[^1]
//!
//! The net does not cover the fixed-point module. Every operation there
//! saturates, so none of them overflows and none of them panics.[^2] The net
//! covers the arithmetic outside that module: a count, an index, a capacity,
//! and the accumulator of a pyramid level. A tile field is one byte wide, and
//! an accumulator that sums one over a whole level must be wider than the
//! field.[^3]
//!
//! A test here drives an overflow on purpose. It reads the outcome through a
//! caught panic rather than through a compiler switch, because the switch
//! that names this profile is not stable on the pinned toolchain.
//!
//! # References
//!
//! [^1]: ADR-0083, the gate build checks every integer overflow, decision D1. `docs/adrs/draft/adr-0083-the-gate-build-checks-every-integer-overflow.md`
//! [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^3]: ADR-0002, simulated and aggregated state holds no floating point number, decision D3. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`

use std::hint::black_box;
use std::panic::{catch_unwind, set_hook, take_hook, AssertUnwindSafe};

/// The number of tiles the project targets.
const TILES: u32 = 16_700_000;

/// The largest value a two-byte tile field holds.
const WIDE_FIELD: u32 = u16::MAX as u32;

/// Runs a body and reports whether it panicked.
///
/// The hook is replaced for the call, so a deliberate panic prints nothing
/// and a reader of the test output sees only the result.
fn panics(body: impl FnOnce()) -> bool {
    let previous = take_hook();
    set_hook(Box::new(|_| {}));
    let outcome = catch_unwind(AssertUnwindSafe(body));
    set_hook(previous);
    outcome.is_err()
}

/// Adds one to the ceiling of a `u32`.
///
/// Both operands pass through `black_box`, so the compiler cannot fold the
/// sum and report the overflow at compile time instead.
fn add_one_past_the_ceiling() {
    let ceiling = black_box(u32::MAX);
    let one = black_box(1u32);
    black_box(ceiling + one);
}

/// Sums a two-byte tile field over the world into a `u32` accumulator.
///
/// The accumulator is too narrow for the sum. This is the shape the widening
/// rule exists to stop.
fn sum_a_wide_field_into_a_u32() {
    let mut total: u32 = 0;
    for _ in 0..black_box(TILES) {
        total += black_box(WIDE_FIELD);
    }
    black_box(total);
}

/// Sums the same field into the widened accumulator, and returns the total.
fn sum_a_wide_field_into_an_i64() -> i64 {
    let mut total: i64 = 0;
    for _ in 0..black_box(TILES) {
        total += i64::from(black_box(WIDE_FIELD));
    }
    total
}

/// The net is armed: the simplest overflow there is ends in a panic.
///
/// The test compiles only where the debug assertions are on. That is the
/// build the gate runs. The release run of the slow gate compiles it out,
/// because the release profile wraps by design.
#[test]
#[cfg(debug_assertions)]
fn the_gate_build_panics_when_an_addition_passes_the_ceiling() {
    assert!(
        panics(add_one_past_the_ceiling),
        "the gate build must turn an integer overflow into a panic. \
         Read ADR-0083 before you change the profile that produced this build."
    );
}

/// The net catches the defect it exists for: an accumulator that is too
/// narrow for the level it sums.
#[test]
#[cfg(debug_assertions)]
fn the_gate_build_panics_when_a_narrow_accumulator_sums_a_level() {
    assert!(
        panics(sum_a_wide_field_into_a_u32),
        "the gate build must catch an accumulator that is too narrow"
    );
}

/// The widened accumulator holds the sum exactly, in every profile.
#[test]
fn the_widened_accumulator_holds_the_sum_of_every_tile() {
    assert_eq!(
        sum_a_wide_field_into_an_i64(),
        i64::from(TILES) * i64::from(WIDE_FIELD),
        "the widened accumulator must hold the exact sum of the level"
    );
}
