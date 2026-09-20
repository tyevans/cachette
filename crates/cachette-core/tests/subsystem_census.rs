//! Tests for the subsystem census rows that had no direct assertion.
//!
//! Seven rows of the subsystem census table previously lacked tests exercising
//! their simulated subsystems: `seats_filled`, `characters`,
//! `upgrades_complete`, `contracts`, `controller_refused`, `contracts_bound`,
//! and `wars_declared`.[^1]
//!
//! Each test here exercises the simulated subsystem through the public
//! interface and asserts that the corresponding census row records the state
//! or cumulative act.[^2]
//!
//! # References
//!
//! [^1]: Findings register, FND-548. `docs/FINDINGS.md`
//! [^2]: Testing rules, section 2. `.agents/rules/testing.md`
//! [^3]: ADR-0002, simulated and aggregated state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^4]: ADR-0146, a faction relation is one signed integer per ordered pair. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
//! [^5]: ADR-0151, an upgrade is a category with a ground fit and a level. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
//! [^6]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`

use cachette_core::unit_type::SOLDIER;
use cachette_core::upgrade::UpgradeCategory;
use cachette_core::{
    Axial, CensusBasis, CommodityId, Entity, FactionId, Fix32, Verb, World, WorldConfig,
    SUBSYSTEM_CENSUS, TRADE_BOUND, WORK_COMMODITY,
};

const SEED: u64 = 0x0123_4567_89ab_cdef;
const ZERO: FactionId = FactionId(0);
const ONE: FactionId = FactionId(1);
const GOODS: CommodityId = WORK_COMMODITY[0];
const MARK: u32 = 8;
const GROUP: u32 = 8;

fn small_world(seed: u64, factions: u16) -> World {
    World::new(WorldConfig {
        width: 16,
        height: 16,
        seed,
        faction_count: factions,
        unit_capacity: 1024,
        ..WorldConfig::DEFAULT
    })
    .expect("the config describes a world")
}

fn census(world: &World, name: &str) -> i64 {
    world
        .subsystem_census()
        .into_iter()
        .find(|(row, _)| *row == name)
        .unwrap_or_else(|| panic!("census row {name} not found"))
        .1
}

fn basis_of(name: &str) -> CensusBasis {
    SUBSYSTEM_CENSUS
        .iter()
        .find(|row| row.name == name)
        .unwrap_or_else(|| panic!("row {name} not found in SUBSYSTEM_CENSUS"))
        .basis
}

#[test]
fn seats_filled_census_row_reflects_occupied_positions() {
    assert_eq!(basis_of("seats_filled"), CensusBasis::Held);

    let mut world = small_world(SEED, 2);
    world
        .set_position_schedule(1, 0)
        .expect("period inside range");
    let place = Axial::new(0, 0);
    let site = world
        .found_settlement(place, ZERO)
        .expect("the tile is inside the world");

    // Open positions during the first step.
    world.step(1).expect("the step must run");
    assert_eq!(
        census(&world, "seats_filled"),
        0,
        "initially no seats are filled"
    );

    // Spawn a resident and seat them at the site.
    let unit = world
        .spawn_soldier(place, ZERO)
        .expect("tile must admit unit");
    assert!(world.set_home_site(unit, Some(site)));

    world.step(1).expect("the step must run");
    assert_eq!(
        census(&world, "seats_filled"),
        1,
        "the resident fills one seat"
    );

    // Despawn the resident. The seat becomes vacant again.
    assert!(world.despawn_soldier(unit));
    world.step(1).expect("the step must run");
    assert_eq!(
        census(&world, "seats_filled"),
        0,
        "held seat count falls when resident despawns"
    );
}

#[test]
fn characters_census_row_reflects_live_characters() {
    assert_eq!(basis_of("characters"), CensusBasis::Held);

    let mut world = small_world(SEED, 2);
    assert_eq!(census(&world, "characters"), 0);

    let char1 = world.create_character(ZERO).expect("creation must succeed");
    assert!(world.characters().contains(char1));
    assert_eq!(census(&world, "characters"), 1);

    let char2 = world.create_character(ONE).expect("creation must succeed");
    assert!(world.characters().contains(char2));
    assert_eq!(census(&world, "characters"), 2);
}

#[test]
fn upgrades_complete_census_row_reflects_finished_upgrades() {
    assert_eq!(basis_of("upgrades_complete"), CensusBasis::Held);

    let mut world = small_world(SEED, 2);
    assert_eq!(census(&world, "upgrades_complete"), 0);

    // Find an open tile that admits units.
    let address = Axial::new(4, 4);
    assert!(world.admits_a_unit(address));

    world
        .zone_project(ZERO, address, UpgradeCategory::ROAD)
        .expect("plan zones road");

    // Spawn a crowd of builders on the tile so the road finishes in one tick.
    let crowd: Vec<Entity> = (0..24)
        .map(|_| {
            world
                .spawn_soldier(address, ZERO)
                .expect("tile admits soldier")
        })
        .collect();

    for unit in &crowd {
        world
            .order_build(*unit, UpgradeCategory::ROAD)
            .expect("tile is zoned");
    }

    // Step once: the road is under construction, so it is not yet complete.
    world.step(1).expect("the step runs");
    assert_eq!(
        census(&world, "upgrades_complete"),
        0,
        "upgrade under construction is not complete"
    );

    // Step until the road finishes.
    for _ in 0..4 {
        if world.finished_upgrade(address).is_some() {
            break;
        }
        world.step(1).expect("the step runs");
    }
    assert_eq!(
        world.finished_upgrade(address),
        Some(UpgradeCategory::ROAD),
        "the road must finish"
    );
    assert_eq!(
        census(&world, "upgrades_complete"),
        1,
        "completed upgrade appears in census"
    );
}

#[test]
fn controller_refused_census_row_counts_refused_actions() {
    assert_eq!(basis_of("controller_refused"), CensusBasis::Total);

    let mut world = small_world(SEED, 2);
    assert_eq!(census(&world, "controller_refused"), 0);

    let schema = world.action_schema();
    let settle = schema
        .encode(Verb::Settle, &[])
        .expect("settle verb encodes");

    // The faction has no settlers, so the settle action must be refused.
    let applied = world.act(ZERO, settle);
    assert!(!applied, "action without settlers must be refused");
    assert_eq!(
        census(&world, "controller_refused"),
        1,
        "first refusal recorded"
    );

    // Stepping folds the refusal into the census total.
    world.step(1).expect("step runs");
    assert_eq!(
        census(&world, "controller_refused"),
        1,
        "refusal preserved across step barrier"
    );

    // A second refusal increments the count.
    let applied2 = world.act(ZERO, settle);
    assert!(!applied2, "second action without settlers must be refused");
    assert_eq!(
        census(&world, "controller_refused"),
        2,
        "second refusal recorded"
    );
}

#[test]
fn wars_declared_census_row_counts_crossings_into_war_band() {
    assert_eq!(basis_of("wars_declared"), CensusBasis::Total);

    let mut world = small_world(SEED, 2);
    assert_eq!(census(&world, "wars_declared"), 0);

    let war_edge = world.relation_rules().war_edge;

    // Cross into the war band: logs a war declaration.
    assert!(world.set_relation(ZERO, ONE, war_edge - 1));
    assert_eq!(
        census(&world, "wars_declared"),
        1,
        "war crossing recorded immediately"
    );

    // Stepping folds the relations log into the census total.
    world.step(1).expect("step runs");
    assert_eq!(
        census(&world, "wars_declared"),
        1,
        "war declaration preserved across step"
    );

    // Move back to peace, then cross into the war band a second time.
    assert!(world.set_relation(ZERO, ONE, 0));
    assert!(world.set_relation(ZERO, ONE, war_edge - 1));
    assert_eq!(
        census(&world, "wars_declared"),
        2,
        "second war crossing recorded"
    );
}

// ---------------------------------------------------------------------------
// Helpers for trading tests
// ---------------------------------------------------------------------------

fn seat_close(world: &mut World) {
    let grid = world.grid();
    let mut first = None;
    for index in 0..grid.tile_count() {
        let address = Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32);
        if !world.admits_a_unit(address) {
            continue;
        }
        match first {
            None => {
                if world.found_group_at(address, GROUP, ZERO).is_ok() {
                    first = Some(address);
                }
            }
            Some(place) => {
                if (address.q - place.q).abs() + (address.r - place.r).abs() > 4 {
                    continue;
                }
                if world.found_group_at(address, GROUP, ONE).is_ok() {
                    return;
                }
            }
        }
    }
    panic!("the fixture found no pair of places close together");
}

fn a_trading_world(seed: u64) -> World {
    let mut world = World::new(WorldConfig {
        width: 48,
        height: 48,
        seed,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        ..WorldConfig::DEFAULT
    })
    .expect("the config describes a world");
    seat_close(&mut world);
    world.set_surplus_mark(MARK);
    world.set_overmatch_ratio(0);
    world
        .set_advertisement_schedule(1, 0)
        .expect("the period is inside the range");
    world.step(1).expect("the step runs");
    for faction in [ZERO, ONE] {
        let mut units: Vec<Entity> = world.soldiers().iter_faction(faction).collect();
        units.sort_unstable_by_key(|unit| unit.to_bits());
        units.truncate(2);
        world.set_unit_type_set(&units, SOLDIER);
    }
    world
}

fn set_store(world: &mut World, faction: FactionId, quantity: Fix32) {
    let site = world
        .trading_site_of(faction)
        .expect("the faction holds a site");
    world
        .set_settlement_store(site, GOODS, quantity)
        .expect("commodity is inside set");
}

fn renew_presence(world: &mut World, one: FactionId, other: FactionId) {
    garrison(world);
    for (speaker, listener) in [(one, other), (other, one)] {
        if world.stands_in_territory_of(speaker, listener) {
            continue;
        }
        let grid = world.grid();
        let seats = world.standing_places();
        let mut place = None;
        for index in 0..grid.tile_count() {
            let address = Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32);
            if world.tile_holder(address) == Some(cachette_core::holding::Holder::of(listener))
                && world.admits_a_unit(address)
                && !seats.contains(&address)
            {
                place = Some(address);
                break;
            }
        }
        let place = place.expect("listener holds tile outside site");
        let _ = world.spawn_soldier(place, speaker);
    }
}

fn garrison(world: &mut World) {
    let sites: Vec<(Axial, FactionId)> = world
        .settlements()
        .iter()
        .filter_map(|site| {
            Some((
                world.settlements().address(site)?,
                world.settlements().faction(site)?,
            ))
        })
        .collect();
    for (address, owner) in sites {
        let held = world
            .soldiers()
            .iter_faction(owner)
            .any(|unit| world.soldiers().address(unit) == Some(address));
        if held {
            continue;
        }
        world
            .spawn_soldier(address, owner)
            .expect("site tile admits unit");
    }
}

fn drive_to_a_contract(world: &mut World, bound: u64) -> Option<u64> {
    for tick in 1..=bound {
        set_store(
            world,
            ZERO,
            Fix32::from_int(i16::try_from(MARK).unwrap_or(0) + 2),
        );
        set_store(world, ONE, Fix32::ZERO);
        renew_presence(world, ZERO, ONE);
        world.step(1).expect("the step runs");
        if world
            .trade_book()
            .iter()
            .any(|row| row.status == TRADE_BOUND)
        {
            return Some(tick);
        }
    }
    None
}

#[test]
fn contracts_and_contracts_bound_census_rows_reflect_trade() {
    assert_eq!(basis_of("contracts"), CensusBasis::Held);
    assert_eq!(basis_of("contracts_bound"), CensusBasis::Total);

    let mut world = a_trading_world(29);
    assert_eq!(census(&world, "contracts"), 0);
    assert_eq!(census(&world, "contracts_bound"), 0);

    let bound_tick = drive_to_a_contract(&mut world, 400);
    assert!(
        bound_tick.is_some(),
        "controllers must negotiate and bind a contract"
    );

    assert_eq!(census(&world, "contracts"), 1, "one bound contract is held");
    assert!(
        census(&world, "contracts_bound") >= 1,
        "at least one contract bound over the run"
    );
}

#[test]
fn every_unasserted_census_row_is_in_the_subsystem_table() {
    let unasserted = [
        "seats_filled",
        "characters",
        "upgrades_complete",
        "contracts",
        "controller_refused",
        "contracts_bound",
        "wars_declared",
    ];
    let names: Vec<&'static str> = SUBSYSTEM_CENSUS.iter().map(|row| row.name).collect();
    for name in unasserted {
        assert!(
            names.contains(&name),
            "census table must declare row {name}"
        );
    }
}
