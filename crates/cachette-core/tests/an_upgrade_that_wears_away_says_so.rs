//! The engine says when a level finishes and when an upgrade wears away.
//!
//! **The fixture is built for the extreme and is not a copy of the
//! demonstration world.** The extreme these tests need is an upgrade under
//! wear heavy enough to end it inside a test, because a collapse is the only
//! state the collapse log reports. The demonstration world wears an upgrade
//! by a very small amount for each tick, and a fixture that copied it would
//! measure the fixture.[^1]
//!
//! Both tests drive the engine and then read the log. Neither builds the log
//! by hand. A capability that nothing reaches passes its own test and ships
//! inert.[^2]
//!
//! # References
//!
//! [^1]: Testing Rules, section 2a. `.agents/rules/testing.md`
//! [^2]: Testing Rules, section 5. `.agents/rules/testing.md`

use cachette_core::cohort::NeedRule;
use cachette_core::event::{WEAR_CAUSE_ARMY, WEAR_CAUSE_BOTH, WEAR_CAUSE_WEATHER};
use cachette_core::holding::ReachRules;
use cachette_core::rates::RateSchedule;
use cachette_core::upgrade::{UpgradeCategory, CONDITION_FULL, LODGING_FIT, LODGING_LEVEL_1_WORK};
use cachette_core::{Axial, Entity, FactionId, Fix32, World, WorldConfig};

/// The extent of every world under test.
const WIDTH: u32 = 96;
/// The extent of every world under test.
const HEIGHT: u32 = 96;

/// The seed that these tests read.
const SEED: u64 = 0x0cac_4e77_10d6;

/// How many people the founding seats.
const GROUP: u32 = 2;

/// The interval between two choices, as a power of two. It is far above the
/// ticks a test takes, so no builder chooses to walk away.
const CHOICE_EXPONENT: u32 = 16;

/// The faction that builds.
const OWNER: FactionId = FactionId(0);

/// The faction that wears the upgrade away.
const ENEMY: FactionId = FactionId(1);

/// The store that one birth costs here.
const FOOD: Fix32 = Fix32::ONE;

/// The deficit at which a unit ends here. Nothing reaches it.
const BOUND: Fix32 = Fix32::from_int(4);

/// How many ticks a build is given before a test gives up.
const BUILD_PATIENCE: u64 = (LODGING_LEVEL_1_WORK as u64) * 4;

/// How many ticks a collapse is given before a test gives up.
///
/// The full condition divided by the wear one hostile unit takes in a tick,
/// with room for a fixture that seats fewer units than it asked for.
const WEAR_PATIENCE: u64 = (CONDITION_FULL as u64) / 2000 * 4;

/// A relation value deep inside the war band.
const AT_WAR: i32 = i32::MIN / 2;

/// Returns every address of the extent, in row-major order.
fn addresses() -> Vec<Axial> {
    let mut all = Vec::with_capacity((WIDTH * HEIGHT) as usize);
    for r in 0..HEIGHT {
        for q in 0..WIDTH {
            all.push(Axial::new(q as i32, r as i32));
        }
    }
    all
}

/// A site of the owning faction, and the tile beside it that carries a level.
struct Ground {
    /// The world under test.
    world: World,
    /// The tile the lodging stands on.
    beside: Axial,
    /// The unit that raised it. It still stands on the tile.
    builder: Entity,
}

/// Builds a world with one site and one finished lodging beside it.
fn ground_with_a_finished_lodging() -> Ground {
    let mut world = World::new(WorldConfig {
        width: WIDTH,
        height: HEIGHT,
        seed: SEED,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    world
        .set_choice_schedule(CHOICE_EXPONENT)
        .expect("the exponent is inside the range");
    let reach = WIDTH + HEIGHT;
    world.set_reach_rules(ReachRules::new(reach, 1, reach));
    world.set_growth_schedule(RateSchedule::new(1, 0).expect("one is inside the range"));
    world.set_food_per_birth([FOOD]);
    world.set_housing_per_person(1);
    world.set_need_rule(
        NeedRule::new(Fix32::ZERO, Fix32::ZERO, Fix32::ZERO, Fix32::ZERO, BOUND)
            .expect("no rate is below zero"),
    );
    world
        .set_economy_schedule(1, 0)
        .expect("one is inside the range");
    assert!(world.set_queue_bound(0), "zero is inside the block");
    for faction in [OWNER, ENEMY] {
        assert!(
            world.set_externally_controlled(faction, true),
            "the faction is in the world"
        );
    }
    let (seat, beside) = seat_with_a_neighbour(&world);
    world
        .found_group_at(seat, GROUP, OWNER)
        .expect("the ground admits the group");
    world.step(1).expect("the step must run");
    let builder = order_a_lodging(&mut world, beside);
    step_until_level(&mut world, builder, beside, 1);
    Ground {
        world,
        beside,
        builder,
    }
}

/// Finds a tile that admits a city and has a neighbour a lodging fits.
fn seat_with_a_neighbour(world: &World) -> (Axial, Axial) {
    for seat in addresses() {
        if !world.admits_a_unit(seat) {
            continue;
        }
        for side in world.grid().neighbours(seat).into_iter().flatten() {
            if world.admits_a_unit(side) && fits_a_lodging(world, side) {
                return (seat, side);
            }
        }
    }
    panic!("the world holds no seat with a neighbour that fits a lodging");
}

/// Reports whether the lodging row of the table fits the ground of one tile.
fn fits_a_lodging(world: &World, address: Axial) -> bool {
    world
        .tile_kind(address)
        .is_some_and(|kind| LODGING_FIT & (1u32 << kind.to_u8()) != 0)
}

/// Puts one builder on a tile and orders it to raise a lodging there.
fn order_a_lodging(world: &mut World, address: Axial) -> Entity {
    let unit = world
        .spawn_soldier(address, OWNER)
        .expect("the ground admits a unit");
    world
        .zone_project(OWNER, address, UpgradeCategory::LODGING)
        .expect("the faction holds the ground");
    world
        .order_build(unit, UpgradeCategory::LODGING)
        .expect("the ground fits a lodging");
    unit
}

/// Steps until the level on a tile reaches one value.
fn step_until_level(world: &mut World, unit: Entity, address: Axial, level: u8) {
    for _ in 1..=BUILD_PATIENCE {
        let here = world
            .soldiers_on(address)
            .map(|units| units.contains(&unit))
            .unwrap_or(false);
        if !here {
            world
                .place_soldier(unit, address)
                .expect("the ground admits the builder");
        }
        let _ = world.order_build(unit, UpgradeCategory::LODGING);
        world.step(1).expect("the step must run");
        if world.upgrade_level(address) >= level {
            return;
        }
    }
    panic!("the level {level} did not stand after {BUILD_PATIENCE} ticks");
}

// ---------------------------------------------------------------------------
// The finish
// ---------------------------------------------------------------------------

/// The engine writes one row when a level finishes, and the row names the
/// tile, the category and the level.
#[test]
fn a_finished_level_reaches_the_log() {
    let ground = ground_with_a_finished_lodging();
    let world = ground.world;
    let log = world.finished_log();
    assert_eq!(log.len(), 1, "one level finished on the last step: {log:?}");
    let row = log[0];
    assert_eq!(row.tick, world.tick(), "the row carries the tick");
    assert_eq!(
        row.tile,
        world
            .grid()
            .index_of(ground.beside)
            .expect("the tile is in the world"),
        "the row names the tile the level stands on"
    );
    assert_eq!(row.category, UpgradeCategory::LODGING.0);
    assert_eq!(row.level, 1);
    assert_eq!(
        row.holder.faction(),
        Some(OWNER),
        "the owner holds the ground the level stands on"
    );
}

/// The founding of a settlement reaches the log, with the faction and the
/// tile.
#[test]
fn a_founded_settlement_reaches_the_log() {
    let ground = ground_with_a_finished_lodging();
    let world = ground.world;
    // The founding happened before many steps ran, so the log of the last
    // step holds nothing. The rule is that a log holds what happened since
    // the last step began.
    assert!(
        world.founded_log().is_empty(),
        "a log holds only what happened since the last step began"
    );

    let mut fresh = World::new(WorldConfig {
        width: WIDTH,
        height: HEIGHT,
        seed: SEED,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    let (seat, _) = seat_with_a_neighbour(&fresh);
    fresh
        .found_group_at(seat, GROUP, OWNER)
        .expect("the ground admits the group");
    let log = fresh.founded_log();
    assert_eq!(log.len(), 1, "one settlement was founded: {log:?}");
    assert_eq!(log[0].faction, OWNER);
    assert_eq!(
        log[0].tile,
        fresh
            .grid()
            .index_of(seat)
            .expect("the seat is in the world")
    );
    // The next step clears it.
    fresh.step(1).expect("the step must run");
    assert!(
        fresh.founded_log().is_empty(),
        "the step clears the log before any system runs"
    );
}

// ---------------------------------------------------------------------------
// The collapse
// ---------------------------------------------------------------------------

/// An upgrade that an army wears away reaches the collapse log, and the row
/// says where it stood, what it was and what took it.
#[test]
fn an_upgrade_worn_away_reaches_the_log() {
    let mut ground = ground_with_a_finished_lodging();
    let beside = ground.beside;
    let builder = ground.builder;
    let world = &mut ground.world;
    // The builder leaves the tile, so no meeting resolves there and the test
    // measures the wear rather than a contest.
    assert!(
        world.despawn_soldier(builder),
        "the builder is alive at this point"
    );
    let tile = world
        .grid()
        .index_of(beside)
        .expect("the tile is in the world");
    assert_eq!(world.upgrade_level(beside), 1, "the level stands");
    assert!(
        world.set_relation(OWNER, ENEMY, AT_WAR),
        "both factions are in the world"
    );
    assert!(
        world.set_relation(ENEMY, OWNER, AT_WAR),
        "both factions are in the world"
    );
    let room = world.tile_capacity(beside).unwrap_or(1).max(1);
    let mut hostiles = Vec::new();
    for _ in 0..room {
        match world.spawn_soldier(beside, ENEMY) {
            Ok(unit) => hostiles.push(unit),
            Err(_) => break,
        }
    }
    assert!(!hostiles.is_empty(), "the ground admits no hostile unit");

    for _ in 1..=WEAR_PATIENCE {
        // The hostile units stay on the tile. A unit that walked off would
        // make the test measure the walk rather than the wear.
        for unit in &hostiles {
            let _ = world.place_soldier(*unit, beside);
        }
        world.step(1).expect("the step must run");
        if !world.collapsed_log().is_empty() {
            let log = world.collapsed_log();
            assert_eq!(log.len(), 1, "one upgrade collapsed: {log:?}");
            let row = log[0];
            assert_eq!(row.tick, world.tick(), "the row carries the tick");
            assert_eq!(row.tile, tile, "the row names the tile it stood on");
            assert_eq!(row.category, UpgradeCategory::LODGING.0);
            assert_eq!(row.level, 1, "a level stood there, so it is not zero");
            assert!(
                [WEAR_CAUSE_ARMY, WEAR_CAUSE_BOTH, WEAR_CAUSE_WEATHER].contains(&row.cause),
                "the row names a cause: {}",
                row.cause
            );
            assert_eq!(
                world.upgrade_level(beside),
                0,
                "the tile returned to the ground the generator made"
            );
            // The next step clears the log.
            world.step(1).expect("the step must run");
            assert!(
                world.collapsed_log().is_empty(),
                "the step clears the log before any system runs"
            );
            return;
        }
    }
    panic!("the upgrade still stands after {WEAR_PATIENCE} ticks of wear");
}
