//! A founded group feeds itself from the ground the survey measured.
//!
//! A founding seats a group, gives it a store, and sets what the site
//! produces. A founding that set no rate would seat a group that loses most
//! of itself.[^1]
//!
//! **The rate is no longer the only thing that fills the store.** A unit that
//! is ordered to gather now reaches ground that holds the kind it wants, and
//! it carries the load home. The ground feeds the few units that reach it,
//! and it does not feed a whole group.[^4]
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
//! [^4]: Findings register, FND-598. `docs/FINDINGS.md`

use cachette_core::cohort::NEED_FULL;
use cachette_core::{sim_math, CommodityId, Fix32, NeedCondition, World, WorldConfig};

/// The extent of the fixture.
const EXTENT: u32 = 192;

/// The number of factions the fixture holds.
const FACTIONS: u16 = 4;

/// The size of each founding group.
const GROUP: u32 = 30;

/// The margin on the derived span that each run takes.
///
/// A group with no rate reaches the bound at the derived span, and this
/// margin covers the ticks a unit spends on the food that the ground still
/// gives it. The run without the rate must lose somebody, so the margin makes
/// its claim easier and never weaker.
const STARVE_MARGIN: u32 = 2;

/// The number of threads each test steps at.
const THREADS: usize = 4;

/// Builds the fixture world.
fn world() -> World {
    World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: 0x0cac_4e77_e5ee_d001,
        faction_count: FACTIONS,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        ..WorldConfig::DEFAULT
    })
    .expect("the settings describe a world")
}

/// Returns the ticks a unit with no ration takes to reach the bound.
///
/// The need rule applies on every tick. A unit starts at a full need. The
/// need falls by the decay on each tick, and the deficit then rises by the
/// decay until it reaches the bound. The span is therefore the full need plus
/// the bound, divided by the decay, and one more tick for the end to reach
/// the world.
///
/// **The fixture derives the span and states no count of ticks.** The span
/// was a written number, and the sky of a world now decides what else happens
/// inside it. A number long enough to starve a group with no rate was also
/// long enough for a storm to take the food of a group that had one.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-745. `docs/FINDINGS.md`
fn span_that_starves(world: &World) -> u32 {
    let rule = world.need_rule();
    assert!(
        rule.decay().0 > 0,
        "the rule must take something off a need, or nothing ever starves"
    );
    let applications = (NEED_FULL.0 + rule.bound().0) / rule.decay().0;
    assert!(
        applications > 0,
        "the rule must take a unit to the bound in at least one application"
    );
    applications as u32 + 1
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

/// Seats a group for every faction, runs the derived span, and reports how
/// many of the people it seated are still alive.
///
/// The caller decides whether the rate the founding set stands. A caller that
/// takes the rate away puts the defect back, and the two answers must then
/// differ.
///
/// **The queue is off, because this fixture is about food and not about
/// building.** A site builds a typed unit from a queue, and a finished entry
/// spends one resident of the site. A faction founds with a small group, so
/// its first entry would take a person out of that group.[^1]
///
/// **No faction takes the built-in controller.** A founded faction has a
/// seat, so the controller would order its people to gather and the ground
/// would feed a group that has no rate. The flag says an external caller
/// controls the faction, and the controller then leaves it alone.
///
/// # References
///
/// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D4. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
fn people_alive_after_the_span(the_rate_stands: bool) -> (usize, usize) {
    let mut world = world();
    assert!(world.set_queue_bound(0), "zero is inside the block width");
    let outcomes = world.found_run_for_every_faction(GROUP);
    let people: Vec<_> = outcomes
        .iter()
        .filter_map(|outcome| outcome.founding())
        .flat_map(|founding| founding.people().to_vec())
        .collect();
    assert!(!people.is_empty(), "the run seated somebody");
    for outcome in &outcomes {
        let Some(founding) = outcome.founding() else {
            continue;
        };
        assert!(world.set_externally_controlled(outcome.faction(), true));
        if !the_rate_stands {
            world
                .set_production_rate(founding.settlement(), CommodityId(0), Fix32::ZERO)
                .expect("the rate is at or above zero");
        }
    }

    for _ in 0..span_that_starves(&world) * STARVE_MARGIN {
        world.step(THREADS).expect("the step runs");
    }

    let alive = people
        .iter()
        .filter(|person| world.unit_condition(**person).is_some())
        .count();
    let starved = people
        .iter()
        .filter(|person| world.unit_condition(**person) == Some(NeedCondition::Starved))
        .count();
    assert_eq!(
        starved, 0,
        "a person that reached the bound is still in the world, so the count of the living is wrong"
    );
    (alive, people.len())
}

/// A group that keeps the rate its founding set outlives the same group
/// without it.
///
/// **The test holds the sky still by comparison, and no longer by
/// construction.** The claim was that every seated person is alive after the
/// span that starves a group with no rate. A storm now takes the food that a
/// tile carries and ends a unit that stands in the open, so a group with a
/// rate loses people too, and no ground of any world stays dry.[^1] [^2] The
/// two runs meet the same sky from the same seed, so what differs between
/// them is the rate.
///
/// **The comparison is strict, and it is able to fail.** A run that took the
/// rate away from both worlds gives two equal counts, and the assertion
/// refuses them.
///
/// # References
///
/// [^1]: Findings register, FND-728. `docs/FINDINGS.md`
/// [^2]: Findings register, FND-745. `docs/FINDINGS.md`
#[test]
fn a_founded_group_outlives_the_same_group_with_no_rate() {
    let (with_the_rate, seated) = people_alive_after_the_span(true);
    let (without_the_rate, same_seated) = people_alive_after_the_span(false);
    assert_eq!(
        seated, same_seated,
        "the two runs must seat the same people, or they compare two fixtures"
    );
    assert!(
        without_the_rate < seated,
        "a group with no rate kept every person, so the rate decides nothing"
    );
    assert!(
        with_the_rate > without_the_rate,
        "the rate the founding set saved nobody: {with_the_rate} lived with it against {without_the_rate} without it"
    );
}
