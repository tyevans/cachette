//! The ground states what it costs to cross, and the table is the only place
//! that states it.
//!
//! A crossing time is a function of three quantities. The capacity of the
//! tile and the dwell of a unit are two of them. The step multiplier of the
//! ground is the third, and an arithmetic check that omits it gives a
//! confident wrong answer.[^1]
//!
//! The multiplier is content. It sits in the terrain table beside the terrain
//! capacity, because the capacity and the multiplier describe the same
//! tile.[^2] Content states a validated range, and that range is the bound
//! that engine code would otherwise give.
//!
//! **Every figure here is derived, not measured.** No measurement exists on
//! the target platform, and one blocker holds every cost figure in this
//! project.[^3] The two crossing times are accepted values, and the
//! multiplier follows from their ratio.[^4]
//!
//! The tests see only the public crate API.[^5]
//!
//! # References
//!
//! [^1]: Findings register, FND-037. `docs/FINDINGS.md`
//! [^2]: Decisions register, DEC-017. `docs/DECISIONS.md`
//! [^3]: Blockers register, BLK-007. `docs/BLOCKERS.md`
//! [^4]: Decisions register, DEC-008. `docs/DECISIONS.md`
//! [^5]: Testing policy. `docs/TESTING.md`

use cachette_core::terrain::{
    multiplier_is_valid, TileKind, CROSSING_CAPACITY, KIND_COUNT, MOUNTAIN_MULTIPLIER,
    MULTIPLIER_CEILING, MULTIPLIER_FLOOR, ORDINARY_MULTIPLIER,
};
use cachette_core::Fix32;

/// The dwell of the baseline calibration, in ticks.
///
/// The scale constants table gives it, derived from the tile edge and the
/// march rate.[^1] The test states it here because it is an input to the
/// arithmetic below and not a value that this suite owns.
///
/// # References
///
/// [^1]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
const BASELINE_DWELL: u64 = 2;

/// The number of ticks the engine runs in one second.
const TICKS_IN_A_SECOND: u64 = 10;

/// The strength of the formation that the movement timing check crossed.
const FORMATION: u64 = 1_000;

/// The capacity of ordinary ground, in units.
const ORDINARY_CAPACITY: u64 = 8;

/// Returns the ticks that a formation takes to pass a chokepoint.
///
/// The closed-form throughput law is `strength * dwell / capacity`, and the
/// dwell of the exit tile carries the step multiplier of that tile. The
/// arithmetic runs in integers over the Q16.16 scale, so no floating point
/// enters it and no step rounds.[^1]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
fn steady_state_ticks(strength: u64, capacity: u64, multiplier: Fix32) -> u64 {
    let scale = 1u64 << 16;
    let raw = u64::try_from(multiplier.0).expect("a step multiplier is not negative");
    strength * BASELINE_DWELL * raw / (capacity * scale)
}

#[test]
fn every_kind_of_ground_answers_a_step_multiplier() {
    assert_eq!(TileKind::ALL.len(), KIND_COUNT);
    for kind in TileKind::ALL {
        assert!(
            kind.step_multiplier().0 > 0,
            "the kind {kind:?} answers no step multiplier"
        );
    }
}

#[test]
fn a_mountain_step_costs_twice_an_ordinary_step() {
    let ordinary = TileKind::Plain.step_multiplier();
    let mountain = TileKind::Mountain.step_multiplier();
    assert_eq!(ordinary, ORDINARY_MULTIPLIER);
    assert_eq!(mountain, MOUNTAIN_MULTIPLIER);
    assert_eq!(
        mountain.0,
        ordinary.0 * 2,
        "a mountain step no longer costs twice an ordinary step"
    );
}

#[test]
fn every_multiplier_sits_inside_the_validated_range() {
    for kind in TileKind::ALL {
        let multiplier = kind.step_multiplier();
        assert!(
            multiplier_is_valid(multiplier),
            "the kind {kind:?} states a multiplier outside the validated range"
        );
    }
}

#[test]
fn the_validated_range_refuses_a_value_outside_it() {
    assert!(multiplier_is_valid(MULTIPLIER_FLOOR));
    assert!(multiplier_is_valid(MULTIPLIER_CEILING));
    assert!(!multiplier_is_valid(Fix32(MULTIPLIER_FLOOR.0 - 1)));
    assert!(!multiplier_is_valid(Fix32(MULTIPLIER_CEILING.0 + 1)));
}

#[test]
fn the_two_accepted_crossing_times_give_the_mountain_multiplier() {
    // The ordinary crossing is 12.5 seconds, which is 125 ticks. It runs over
    // a crossing-terrain tile at the baseline dwell and the ordinary
    // multiplier.
    let ordinary = steady_state_ticks(
        FORMATION,
        u64::from(CROSSING_CAPACITY),
        TileKind::Plain.step_multiplier(),
    );
    assert_eq!(ordinary, 125);
    assert_eq!(ordinary * 10 / TICKS_IN_A_SECOND, 125);

    // The mountain crossing is 50 seconds, which is 500 ticks. It runs over
    // ordinary capacity at the baseline dwell and the mountain multiplier.
    // The multiplier is the only quantity that separates the two figures.
    let mountain = steady_state_ticks(
        FORMATION,
        ORDINARY_CAPACITY,
        TileKind::Mountain.step_multiplier(),
    );
    assert_eq!(mountain, 500);
    assert_eq!(mountain / TICKS_IN_A_SECOND, 50);
}

#[test]
fn the_ground_that_admits_no_unit_states_no_passability_rule_here() {
    // The capacity table is the one declaration of which ground admits a
    // unit. Water answers a multiplier like every other kind, and that answer
    // is not a second passability rule.[^1]
    //
    // [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    assert!(!TileKind::Water.is_passable());
    assert!(multiplier_is_valid(TileKind::Water.step_multiplier()));
}
