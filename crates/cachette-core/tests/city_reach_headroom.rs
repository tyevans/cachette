//! The finished upgrades of one city, and the reach it may still earn.
//!
//! The reach of a city is a base plus one step for each block of finished
//! upgrades on the ground it holds, capped at a bound.[^1] The reach alone
//! cannot say how far a city stands from its next step, because the reach
//! stops at the bound and the count does not.
//!
//! **The fixture reaches the extreme rather than the typical case.** It runs
//! a city through no upgrade, an unfinished upgrade, one finished upgrade,
//! and a second finished upgrade that the bound refuses. A fixture that
//! stopped below the bound would pass a headroom that never reached zero.[^2]
//!
//! The tests drive the world step and the public verbs.[^3]
//!
//! # References
//!
//! [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D2. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
//! [^2]: Testing rules, section 2a. `.agents/rules/testing.md`
//! [^3]: Testing rules, section 5. `.agents/rules/testing.md`

use cachette_core::holding::ReachRules;
use cachette_core::upgrade::UpgradeCategory;
use cachette_core::{Axial, Entity, FactionId, World, WorldConfig, DEFAULT_UPGRADE_TABLE};

/// The extent of the fixture world.
const EXTENT: u32 = 48;

/// Builds a world of the extent, with the reach rules the test names.
fn world(seed: u64, rules: ReachRules) -> World {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        ..WorldConfig::DEFAULT
    })
    .expect("the extent must describe a world");
    world.set_reach_rules(rules);
    world
}

/// Returns every address of a world, in tile index order.
fn addresses(world: &World) -> Vec<Axial> {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .collect()
}

/// Spawns a unit on one tile and orders it to build a road there.
fn build_a_road(field: &mut World, address: Axial, faction: FactionId) -> Entity {
    let unit = field
        .spawn_soldier(address, faction)
        .expect("the ground admits a unit");
    assert!(
        field
            .zone_project(faction, address, UpgradeCategory::ROAD)
            .is_ok(),
        "the plan refused a road project at {address:?}"
    );
    assert!(
        field.order_build(unit, UpgradeCategory::ROAD).is_ok(),
        "the verb refused a road at {address:?}"
    );
    unit
}

/// The margin on the ticks a road build is given, above the work it asks for.
///
/// **This bounds a loop that leaves early. It is not a count of ticks that a
/// test takes.** A build of the stated work finishes in the ticks the work
/// asks for and takes none of this margin. The margin covers the ticks a
/// builder spends repairing what the rain and the storms of a world take off
/// the level while it stands.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-745. `docs/FINDINGS.md`
const BUILD_MARGIN: i64 = 4;

/// Steps the world until every road of a set stands, and returns the ticks.
///
/// **The budget comes from the work table, and it carries a margin.** Rain
/// and a storm wear a level that stands, and a builder pays the repair before
/// it advances the level, so the ticks a build takes are a property of the
/// sky. A budget equal to the work asks the sky to stay quiet.[^1]
///
/// **The loop states that the builder of every road still going up is
/// alive.** A storm ends a unit that stands in the open, and a fixture that
/// waited on a dead builder would step a world that built nothing and would
/// then report a slow world. A road that already stands needs no builder, so
/// the loop reads none.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-745. `docs/FINDINGS.md`
fn hold_the_roads(field: &mut World, roads: &[(Axial, Entity)]) -> u64 {
    let work = DEFAULT_UPGRADE_TABLE.work_above(UpgradeCategory::ROAD, 0);
    assert!(work > 0, "a road level must ask a builder for work");
    let budget = (work * BUILD_MARGIN) as u64 * roads.len() as u64;
    for taken in 1..=budget {
        for (address, builder) in roads {
            if field.finished_upgrade(*address) == Some(UpgradeCategory::ROAD) {
                continue;
            }
            assert!(
                field.soldiers().contains(*builder),
                "the world ended the builder of {address:?} at tick {taken}, so the weather decided this test"
            );
        }
        field.step(1).expect("the step must run");
        if roads
            .iter()
            .all(|(address, _)| field.finished_upgrade(*address) == Some(UpgradeCategory::ROAD))
        {
            return taken;
        }
    }
    panic!("the roads did not all stand after {budget} ticks");
}

#[test]
fn a_city_reports_its_finished_upgrades_and_its_remaining_reach() {
    let rules = ReachRules::new(2, 1, 3);
    let mut field = world(11, rules);
    let seat = addresses(&field)
        .into_iter()
        .find(|address| {
            field.admits_a_unit(*address)
                && field.admits_a_unit(Axial::new(address.q + 1, address.r))
                && field.admits_a_unit(Axial::new(address.q + 2, address.r))
        })
        .expect("the world holds a seat with open ground beside it");
    let site = field
        .found_settlement(seat, FactionId(0))
        .expect("the ground admits a city");
    field.step(1).expect("the step must run");

    assert_eq!(field.city_finished_upgrades(site), Some(0));
    assert_eq!(
        field.city_reach_headroom(site),
        Some(rules.cap() - rules.base()),
        "a city with no upgrade may still earn every step up to the bound"
    );

    let first = Axial::new(seat.q + 1, seat.r);
    let first_builder = build_a_road(&mut field, first, FactionId(0));
    field.step(1).expect("the step must run");
    assert!(
        field
            .upgrade_at(first)
            .is_some_and(|upgrade| !upgrade.is_complete()),
        "the fixture finished the road at once, so the unfinished case is untested"
    );
    assert_eq!(
        field.city_finished_upgrades(site),
        Some(0),
        "an unfinished upgrade must not count"
    );

    hold_the_roads(&mut field, &[(first, first_builder)]);
    assert_eq!(field.finished_upgrade(first), Some(UpgradeCategory::ROAD));
    assert_eq!(
        field.city_finished_upgrades(site),
        Some(1),
        "one finished upgrade must count"
    );
    assert_eq!(field.city_reach(site), Some(3));
    assert_eq!(
        field.city_reach_headroom(site),
        Some(0),
        "one step of reach must leave no headroom under this bound"
    );

    let second = Axial::new(seat.q + 2, seat.r);
    let second_builder = build_a_road(&mut field, second, FactionId(0));
    hold_the_roads(
        &mut field,
        &[(first, first_builder), (second, second_builder)],
    );
    assert_eq!(field.finished_upgrade(second), Some(UpgradeCategory::ROAD));
    assert_eq!(field.city_reach(site), Some(rules.cap()));
    assert_eq!(field.city_reach_headroom(site), Some(0));
    assert_eq!(
        field.city_finished_upgrades(site),
        Some(2),
        "the count must pass the bound that the reach stops at"
    );
    assert!(field.check_invariants());
}

#[test]
fn an_identity_that_names_no_city_reports_nothing() {
    let mut field = world(11, ReachRules::DEFAULT);
    let seat = addresses(&field)
        .into_iter()
        .find(|address| field.admits_a_unit(*address))
        .expect("the world admits a unit somewhere");
    let site = field
        .found_settlement(seat, FactionId(0))
        .expect("the ground admits a city");
    field.step(1).expect("the step must run");
    assert!(field.city_finished_upgrades(site).is_some());
    assert!(field.destroy_settlement(site));
    field.step(1).expect("the step must run");
    assert_eq!(field.city_finished_upgrades(site), None);
    assert_eq!(field.city_reach_headroom(site), None);
}
