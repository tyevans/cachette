//! A founded group feeds itself from the ground the survey measured.
//!
//! A founding seats a group, gives it a store, and sets what the site
//! produces. Nothing else fills that store today, so a founding that set no
//! rate would seat a group that starves to the last unit.[^1]
//!
//! These tests go through the public interface of the crate.[^2]
//!
//! The world here is wider than the coarsest lattice spacing of the terrain
//! generator, so it holds more than one kind of ground.[^3]
//!
//! # References
//!
//! [^1]: Findings register, FND-124. `docs/FINDINGS.md`
//! [^2]: Testing rules, section 6. `.claude/rules/testing.md`
//! [^3]: Findings register, FND-054. `docs/FINDINGS.md`

use cachette_core::{sim_math, CommodityId, Fix32, NeedCondition, World, WorldConfig};

/// The extent of the fixture.
const EXTENT: u32 = 192;

/// The number of factions the fixture holds.
const FACTIONS: u16 = 4;

/// The size of each founding group.
const GROUP: u32 = 30;

/// The number of ticks each test runs.
///
/// A unit ends when its deficit reaches the bound. At the default rule that
/// takes fewer ticks than this, which the starvation test below shows by
/// running the same span with the rate removed.
const TICKS: u32 = 120;

/// The number of threads each test steps at.
const THREADS: usize = 4;

/// Builds the fixture world.
fn world() -> World {
    World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: 0x0cac_4e77_e5ee_d001,
        faction_count: FACTIONS,
    })
    .expect("the settings describe a world")
}

#[test]
fn a_founding_sets_the_rate_from_the_food_the_place_reaches() {
    let mut world = world();
    let ration = world.need_rule().ration();
    let outcomes = world.found_run_for_every_faction(GROUP);
    let mut seated = 0usize;
    for outcome in &outcomes {
        let Some(founding) = outcome.founding() else {
            continue;
        };
        seated += 1;
        let reached = founding
            .survey()
            .chosen()
            .expect("the founding chose a place")
            .provision()
            .food;
        let expected = sim_math::mul(ration, Fix32::from_int(reached.0 as i16));
        let rate = world
            .production_rate(founding.settlement(), CommodityId(0))
            .expect("the settlement is live");
        assert_eq!(
            rate, expected,
            "the rate follows the food the place reaches"
        );
    }
    assert!(seated > 0, "the run seated at least one faction");
}

#[test]
fn a_founded_group_is_alive_after_the_span_that_would_starve_it() {
    let mut world = world();
    let outcomes = world.found_run_for_every_faction(GROUP);
    let people: Vec<_> = outcomes
        .iter()
        .filter_map(|outcome| outcome.founding())
        .flat_map(|founding| founding.people().to_vec())
        .collect();
    assert!(!people.is_empty(), "the run seated somebody");

    for _ in 0..TICKS {
        world.step(THREADS).expect("the step runs");
    }

    for person in &people {
        assert_eq!(
            world.unit_condition(*person),
            Some(NeedCondition::Fed),
            "the person eats what the site produces"
        );
    }
}

#[test]
fn the_same_group_starves_when_the_rate_is_taken_away() {
    // The rate is what the founding set. Taking it away puts the defect back,
    // and the assertion above must then fail. A test that never sees this
    // case measures the fixture rather than the engine.[^1]
    //
    // [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
    let mut world = world();
    let outcomes = world.found_run_for_every_faction(GROUP);
    let people: Vec<_> = outcomes
        .iter()
        .filter_map(|outcome| outcome.founding())
        .flat_map(|founding| founding.people().to_vec())
        .collect();
    for outcome in &outcomes {
        let Some(founding) = outcome.founding() else {
            continue;
        };
        world
            .set_production_rate(founding.settlement(), CommodityId(0), Fix32::ZERO)
            .expect("the rate is at or above zero");
    }

    for _ in 0..TICKS {
        world.step(THREADS).expect("the step runs");
    }

    let alive = people
        .iter()
        .filter(|person| world.unit_condition(**person).is_some())
        .count();
    assert_eq!(alive, 0, "a group with no rate feeds nobody and ends");
}
