//! The engine says when a level finishes and when an upgrade wears away.
//!
//! **The fixture is built for the extreme and is not a copy of the
//! demonstration world.** The extreme these tests need is an upgrade under
//! wear heavy enough to end it inside a test, because a collapse is the only
//! state the collapse log reports. The demonstration world wears an upgrade
//! by a very small amount for each tick, and a fixture that copied it would
//! measure the fixture.[^1]
//!
//! **The region stands where the only wear is the one under test.** The wear
//! pass sums four terms over a tile. It names the collapse after the heaviest
//! term of the tick that ended it. Rain and travelling storms pass over every
//! tile of a temperate region, so a collapse there carries the name of the
//! weather. It carries that name however long an army stood on the ground.
//!
//! The world these tests build therefore stands in a polar region, and it
//! spreads thirty degrees of latitude over its rows. Cold air carries little
//! water, so the ground stays dry. The wide span raises the front mark above
//! the temperature gradient of a polar region, so no storm forms. The army is
//! the only term left, and the cause column says so.[^1]
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
use cachette_core::event::WEAR_CAUSE_ARMY;
use cachette_core::holding::ReachRules;
use cachette_core::rates::RateSchedule;
use cachette_core::upgrade::{
    UpgradeCategory, ARMY_WEAR_FOR_EACH_UNIT, CONDITION_FULL, LODGING_FIT, LODGING_LEVEL_1_WORK,
};
use cachette_core::{Axial, Entity, FactionId, Fix32, World, WorldConfig};

/// The extent of every world under test.
const WIDTH: u32 = 96;
/// The extent of every world under test.
const HEIGHT: u32 = 96;

/// The seed that these tests read.
const SEED: u64 = 0x0cac_4e77_10d6;

/// The latitude of the middle row of every world under test, in hundredths of
/// a degree.
///
/// **Seventy degrees south is half of what makes the army the only wear
/// term.** Cold air carries little water. The air over a polar region
/// therefore holds almost none, and the ground under the site stays dry. The
/// span below is the other half.
const LATITUDE_CENTRE: i32 = -7000;

/// The latitude from the first row of every world under test to the last, in
/// hundredths of a degree.
///
/// **Thirty degrees is what keeps a storm off the site.** A front rises where
/// the temperature across one weather cell passes a mark. The engine takes
/// that mark from the latitude that one row of the world covers. A world that
/// spreads thirty degrees over its rows sets a high mark. The gradient of a
/// polar region never reaches it, so no front forms over the site.
///
/// **This is not the span of a region of the target world, and it is not
/// meant to be.** A fixture states the world that produces the case it
/// measures. It does not copy the world the demonstration binary shows.[^1]
///
/// The wear test asserts on every tick that the ground is dry and that no
/// hazard stands over the site. A world that stops holding either therefore
/// fails the test, rather than measuring the weather. The centre and the span
/// stand inside a wide range of settings that hold both, and not on an edge
/// of one. The commit that chose them holds the survey.[^2]
///
/// # References
///
/// [^1]: Testing Rules, section 2a. `.agents/rules/testing.md`
/// [^2]: Commit Message Rules. `.agents/rules/commits.md`
const LATITUDE_SPAN: i32 = 3000;

/// The chance that lightning starts a fire on one tick.
///
/// Ground that burns wears far faster than an army does, and the wear pass
/// names the fire first, so a fixture that measures an army must state that
/// nothing catches.
const LIGHTNING_CHANCE: u64 = 0;

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
/// with room for a fixture that seats fewer units than it asked for. The rate
/// is read from the module that states it, so this holds no second copy of
/// it.[^1]
///
/// The test proves the slack rather than trusting this derivation. It counts
/// the ticks the wear of the fixture it built actually needs, and it asserts
/// that this budget stands above that count with room to spare.
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
const WEAR_PATIENCE: u64 = (CONDITION_FULL as u64) / (ARMY_WEAR_FOR_EACH_UNIT as u64) * 4;

/// The room that the wear budget must hold above the wear it waits for.
const WEAR_SLACK: u64 = 2;

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

/// The settings that every world under test is built from.
///
/// **This is the one site that states them.** Both tests build a world. A
/// second literal would let one of them stand in a region the other does
/// not.[^1]
///
/// Every field carries a value, and none comes from the default settings. A
/// field that the settings gain therefore breaks this file, and the fixture
/// states what it wants rather than taking what a default gives it.
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
fn settings() -> WorldConfig {
    WorldConfig {
        width: WIDTH,
        height: HEIGHT,
        seed: SEED,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        latitude_centre: LATITUDE_CENTRE,
        latitude_span: LATITUDE_SPAN,
    }
}

/// Builds a world with one site and one finished lodging beside it.
fn ground_with_a_finished_lodging() -> Ground {
    let mut world = World::new(settings()).expect("the extent must describe a world");
    world.set_lightning_chance(LIGHTNING_CHANCE);
    world
        .set_choice_schedule(CHOICE_EXPONENT)
        .expect("the exponent is inside the range");
    let reach = WIDTH + HEIGHT;
    world.set_reach_rules(ReachRules::new(reach, 1, reach));
    world.set_growth_schedule(RateSchedule::new(1, 0).expect("one is inside the range"));
    world.set_food_per_birth([FOOD, Fix32::ZERO, Fix32::ZERO]);
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
///
/// **The loop states that the builder is alive on every tick.** A builder that
/// left the world takes no build order, and the loop would then run out its
/// budget and report a slow world. The polar region of this fixture forms no
/// storm, so a builder that goes names the storm log that did not name it.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-744. `docs/FINDINGS.md`
fn step_until_level(world: &mut World, unit: Entity, address: Axial, level: u8) {
    for tick in 1..=BUILD_PATIENCE {
        assert!(
            world.soldiers().contains(unit),
            "the world ended the builder on tick {tick}, and the storm log of that tick names \
             {:?}",
            world
                .units_lost_to_storms()
                .iter()
                .map(|lost| lost.unit)
                .collect::<Vec<u64>>()
        );
        let here = world
            .soldiers_on(address)
            .map(|units| units.contains(&unit))
            .unwrap_or(false);
        if !here {
            world
                .place_soldier(unit, address)
                .expect("the ground admits the builder");
        }
        world
            .order_build(unit, UpgradeCategory::LODGING)
            .expect("the builder stands on ground that fits a lodging");
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

    let mut fresh = World::new(settings()).expect("the extent must describe a world");
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

/// Returns the ticks that a count of hostile units needs to end one level.
///
/// The polar region leaves no other wear term over the tile, so the whole of
/// a tick is the army term. The count of ticks that a full level needs then
/// follows from the rate that the upgrade module states.
fn ticks_the_army_needs(hostiles: usize) -> u64 {
    let each_tick = (hostiles as u64) * (ARMY_WEAR_FOR_EACH_UNIT as u64);
    (CONDITION_FULL as u64).div_ceil(each_tick)
}

/// Asserts that the next step charges the tile for the army and for nothing
/// else.
///
/// The wear pass reads the weather and the fire that the previous step left.
/// What the tile carries now is therefore what the next step charges it. A
/// tile that carried rain, a storm or a fire would give the collapse row a
/// cause that this fixture did not arrange.
fn assert_the_army_is_the_only_wear(world: &World, address: Axial) {
    assert_eq!(
        world.ground_is_wet(address),
        Some(false),
        "the polar region must leave the ground dry, or the weather wears the site"
    );
    assert_eq!(
        world.tile_under_a_hazard(address),
        Some(false),
        "no storm and no fire may stand over the site, or one of them names the cause"
    );
}

/// An upgrade that an army wears away reaches the collapse log, and the row
/// says where it stood, what it was and that an army took it.
///
/// **The cause column is pinned to the army and not to a set of causes.** The
/// wear pass names the collapse after the heaviest term of the tick that
/// ended it, so a row that named the weather or a storm would say that the
/// fixture stopped reaching the case it was built for. The world stands in a
/// polar region for that reason, and the loop below asserts on every tick
/// that no other term stands over the tile.[^1]
///
/// # References
///
/// [^1]: Testing Rules, section 2a. `.agents/rules/testing.md`
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

    let needed = ticks_the_army_needs(hostiles.len());
    assert!(
        WEAR_PATIENCE >= needed.saturating_mul(WEAR_SLACK),
        "the budget of {WEAR_PATIENCE} ticks must hold {WEAR_SLACK} times the {needed} ticks \
         that {} hostile units need",
        hostiles.len()
    );

    for spent in 1..=WEAR_PATIENCE {
        assert_the_army_is_the_only_wear(world, beside);
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
            assert_eq!(
                row.cause, WEAR_CAUSE_ARMY,
                "an army wore this site away, so the row names the army"
            );
            assert_eq!(
                spent, needed,
                "the army is the only wear over the tile, so the collapse lands on the tick \
                 the army rate says"
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
