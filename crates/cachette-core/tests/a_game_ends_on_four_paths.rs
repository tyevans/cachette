//! Four readers end a game, in one fixed order, once.
//!
//! Each reader fires on a fixture at its extreme. Domination fires for a
//! faction whose rivals hold no unit, and for a faction that holds every
//! seat while a rival still lives. Wealth fires at the stock target and not
//! one raw unit below it, and its total survives a sum that overflows a
//! 32-bit accumulator. The wonder fires on the tick the work completes and
//! not the tick before. Renown fires at the renown target and not below.
//! Two paths true on one tick record the earlier path of the fixed order,
//! and a path that becomes true later changes nothing.[^1]
//!
//! # References
//!
//! [^1]: ADR-0148, a game end is recorded once and stops the controllers, decisions D2 and D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`

use cachette_core::choose;
use cachette_core::site::CommodityId;
use cachette_core::upgrade::UpgradeCategory;
use cachette_core::{
    Axial, Entity, FactionId, Fix32, World, WorldConfig, RENOWN_TARGET, STOCK_TARGET,
};

const THREADS: usize = 2;

/// The people each founding settles. Small, so a step is cheap.
const GROUP: u32 = 8;

/// A limit no fixture here reaches, so the territory reader stays quiet.
const FAR_LIMIT: u64 = 100_000;

fn config(factions: u16, seed: u64, extent: u32) -> WorldConfig {
    WorldConfig {
        width: extent,
        height: extent,
        seed,
        faction_count: factions,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    }
}

/// Builds a world whose territory reader never fires inside a test.
fn world(factions: u16, seed: u64, extent: u32) -> World {
    let mut world =
        World::new(config(factions, seed, extent)).expect("the extent describes a world");
    world.set_tick_limit(FAR_LIMIT);
    world
}

/// Returns every address of a world, in row-major order.
fn addresses(world: &World) -> Vec<Axial> {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .collect()
}

/// Returns the first address that admits a unit.
fn open_address(world: &World) -> Axial {
    addresses(world)
        .into_iter()
        .find(|address| world.admits_a_unit(*address))
        .expect("the world holds open ground")
}

/// Returns an island: an open tile whose every neighbour refuses a unit, so
/// a unit on it never moves.
fn island(world: &World) -> Axial {
    addresses(world)
        .into_iter()
        .find(|address| {
            world.admits_a_unit(*address)
                && world
                    .grid()
                    .neighbours(*address)
                    .iter()
                    .all(|side| side.is_none_or(|next| !world.admits_a_unit(next)))
        })
        .expect("the world holds an island")
}

/// Founds one settlement for a faction at the first place the survey takes.
fn settle(world: &mut World, faction: FactionId) -> Entity {
    for address in addresses(world) {
        if let Ok(site) = world.found_settlement(address, faction) {
            return site;
        }
    }
    panic!("faction {} finds no place to settle", faction.0);
}

/// Founds one group for a faction, and returns the place, which is its seat.
fn seat(world: &mut World, faction: FactionId, apart_from: &[Axial]) -> Axial {
    for address in addresses(world) {
        if apart_from
            .iter()
            .any(|place| (place.q - address.q).abs() < 12 && (place.r - address.r).abs() < 12)
        {
            continue;
        }
        if world.found_group_at(address, GROUP, faction).is_ok() {
            return address;
        }
    }
    panic!("faction {} finds no place to found", faction.0);
}

fn step(world: &mut World) {
    world.step(THREADS).expect("the step runs");
}

// ---------------------------------------------------------------------------
// Domination
// ---------------------------------------------------------------------------

#[test]
fn a_faction_whose_three_rivals_hold_no_unit_wins_by_domination() {
    let mut world = world(4, 31, 32);
    let address = open_address(&world);
    world
        .spawn_soldier(address, FactionId(2))
        .expect("the ground admits a unit");
    step(&mut world);
    let end = world.game_end();
    assert!(end.is_set());
    assert_eq!(end.winner, FactionId(2));
    assert_eq!(end.win_path(), Some(cachette_core::WinPath::Domination));
    assert_eq!(end.tick.0, 1);
}

#[test]
fn an_empty_world_and_a_world_of_one_faction_do_not_end_by_domination() {
    // Every faction has zero units, so no faction has a unit to win with.
    let mut empty = world(3, 32, 32);
    for _ in 0..3 {
        step(&mut empty);
    }
    assert!(!empty.game_end().is_set());

    // One faction has no rival to have lost.
    let mut alone = world(1, 33, 32);
    let address = open_address(&alone);
    alone
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    for _ in 0..3 {
        step(&mut alone);
    }
    assert!(!alone.game_end().is_set());
}

#[test]
fn a_faction_that_holds_every_seat_wins_by_domination_while_a_rival_lives() {
    let mut world = world(2, 18, 48);
    let first = seat(&mut world, FactionId(0), &[]);
    let second = seat(&mut world, FactionId(1), &[first]);
    // Faction 1 keeps one unit far from its seat, so the unit clause cannot
    // fire and only the seat clause can end the game. The rest of faction 1
    // leaves, and faction 0 fills the seat of faction 1 with its own people.
    let refuge = addresses(&world)
        .into_iter()
        .find(|address| {
            world.admits_a_unit(*address)
                && [first, second].iter().all(|place| {
                    (place.q - address.q).abs() >= 12 || (place.r - address.r).abs() >= 12
                })
        })
        .expect("the world holds open ground away from both seats");
    let survivors: Vec<Entity> = world.soldiers().iter().collect();
    for unit in survivors {
        if world.soldiers().faction(unit) == Some(FactionId(1)) {
            assert!(world.despawn_soldier(unit));
        }
    }
    world
        .spawn_soldier(refuge, FactionId(1))
        .expect("the refuge admits a unit");
    let room = world
        .tile_capacity(second)
        .expect("the seat is inside the world");
    for _ in 0..room {
        world
            .spawn_soldier(second, FactionId(0))
            .expect("the seat admits a unit");
    }
    // A faction holds the ground its cities reach, and a unit standing on a
    // tile gives its faction no claim on it.[^2] Faction 0 therefore takes
    // the seat of faction 1 by founding a city on it, after the city of
    // faction 1 goes.
    //
    // [^2]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D1. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    let rival_city = world
        .settlements()
        .on_tile(second)
        .expect("the second founding left a city on the seat");
    assert!(world.destroy_settlement(rival_city));
    world
        .found_settlement(second, FactionId(0))
        .expect("the seat admits a city");
    let mut ended_on = None;
    for _ in 0..200 {
        step(&mut world);
        let standing = world.standing(FactionId(0)).expect("faction 0 exists");
        if world.game_end().is_set() {
            assert_eq!(standing.seats_held, 2, "the winner holds both seats");
            ended_on = Some(world.tick().0);
            break;
        }
        assert!(standing.seats_held < 2, "two seats held and no end");
    }
    assert!(ended_on.is_some(), "faction 0 never took the second seat");
    let end = world.game_end();
    assert_eq!(end.winner, FactionId(0));
    assert_eq!(end.win_path(), Some(cachette_core::WinPath::Domination));
    assert!(
        world.soldiers().population_of(FactionId(1)) > 0,
        "the rival still lives, so the seat clause and not the unit clause fired"
    );
}

// ---------------------------------------------------------------------------
// Wealth
// ---------------------------------------------------------------------------

#[test]
fn a_stock_total_at_the_target_ends_the_game_and_one_below_does_not() {
    let mut world = world(2, 34, 32);
    // Both factions keep a unit, so domination stays quiet.
    let address = open_address(&world);
    world
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    world
        .spawn_soldier(address, FactionId(1))
        .expect("the ground admits a second unit");
    let site = settle(&mut world, FactionId(1));
    let below = Fix32(i32::try_from(STOCK_TARGET - 1).expect("the target fits a store"));
    world
        .set_settlement_store(site, CommodityId(0), below)
        .expect("the commodity exists");
    step(&mut world);
    assert!(!world.game_end().is_set(), "one raw unit below the target");
    assert_eq!(
        world
            .standing(FactionId(1))
            .expect("faction 1 exists")
            .store_total,
        STOCK_TARGET - 1
    );
    let at = Fix32(i32::try_from(STOCK_TARGET).expect("the target fits a store"));
    world
        .set_settlement_store(site, CommodityId(0), at)
        .expect("the commodity exists");
    step(&mut world);
    let end = world.game_end();
    assert!(end.is_set());
    assert_eq!(end.winner, FactionId(1));
    assert_eq!(end.win_path(), Some(cachette_core::WinPath::WealthOrWonder));
}

#[test]
fn the_stock_total_survives_a_sum_that_overflows_a_32_bit_accumulator() {
    let mut world = world(2, 35, 48);
    let address = open_address(&world);
    world
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    world
        .spawn_soldier(address, FactionId(1))
        .expect("the ground admits a second unit");
    // Two stores at the top of the 32-bit range. Their sum wraps to a
    // negative number in 32 bits and never reaches the target there. In 64
    // bits it is above the target.
    let first = settle(&mut world, FactionId(0));
    let second = settle(&mut world, FactionId(0));
    for site in [first, second] {
        world
            .set_settlement_store(site, CommodityId(0), Fix32(i32::MAX))
            .expect("the commodity exists");
    }
    let total = world
        .standing(FactionId(0))
        .expect("faction 0 exists")
        .store_total;
    assert_eq!(total, 2 * i64::from(i32::MAX));
    assert!(
        total > i64::from(i32::MAX),
        "the fixture reaches the overflow"
    );
    step(&mut world);
    let end = world.game_end();
    assert!(end.is_set());
    assert_eq!(end.winner, FactionId(0));
    assert_eq!(end.win_path(), Some(cachette_core::WinPath::WealthOrWonder));
}

// ---------------------------------------------------------------------------
// Wonder
// ---------------------------------------------------------------------------

#[test]
fn a_wonder_ends_the_game_on_the_tick_it_completes_and_not_the_tick_before() {
    let mut world = world(2, 102, 192);
    // The choice pass replaces the build order of a unit whose cell chooses
    // on that frame. The fixture keeps the choice away from the run.
    world
        .set_choice_schedule(choose::PERIOD_LOG2_CEILING)
        .expect("the exponent is inside the range");
    let site = island(&world);
    // Every unit on the tile builds, so the work is done long before a unit
    // starves, and faction 0 holds the ground. Near the end all but one stop,
    // so the progress rises by one each tick and passes through one unit
    // short. A rival unit elsewhere keeps domination quiet.
    // A unit builds anything but a road only on ground its own faction
    // holds, and a faction holds the ground its cities reach.[^3] The city
    // on the island is what makes the wonder buildable there.
    //
    // [^3]: ADR-0150, held ground is the ground within reach of a city its faction owns, decisions D1 and D4. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    world
        .found_settlement(site, FactionId(0))
        .expect("the island admits a city");
    world.step(THREADS).expect("the step runs");
    let room = world
        .tile_capacity(site)
        .expect("the island is inside the world");
    let builders: Vec<Entity> = (0..room)
        .map(|_| {
            let unit = world
                .spawn_soldier(site, FactionId(0))
                .expect("the island admits a unit");
            assert!(world.order_build(unit, UpgradeCategory::WONDER).is_ok());
            unit
        })
        .collect();
    let elsewhere = addresses(&world)
        .into_iter()
        .find(|address| *address != site && world.admits_a_unit(*address))
        .expect("the world holds a second open tile");
    world
        .spawn_soldier(elsewhere, FactionId(1))
        .expect("the ground admits a unit");
    let work = cachette_core::DEFAULT_UPGRADE_TABLE.work_above(UpgradeCategory::WONDER, 0);
    let mut saw_one_short = false;
    let mut slowed = false;
    for _ in 0..(work as u64 + 8) {
        step(&mut world);
        let progress = world.upgrade_at(site).map_or(0, |built| built.progress.0);
        if !slowed && progress + i64::from(room) >= work {
            for unit in &builders[1..] {
                assert!(world.stop_build(*unit));
            }
            slowed = true;
        }
        if progress == work - 1 {
            saw_one_short = true;
            assert!(!world.game_end().is_set(), "one unit of work short");
            assert_eq!(
                world
                    .standing(FactionId(0))
                    .expect("faction 0 exists")
                    .wonder_progress,
                work - 1
            );
        }
        // The level rises in place and the work done returns to zero, so
        // the loop reads the level and not the work done.[^2]
        //
        // [^2]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D3. `docs/adrs/draft/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
        if world.finished_upgrade(site) == Some(UpgradeCategory::WONDER) {
            break;
        }
    }
    assert!(saw_one_short, "the fixture never passed one unit short");
    assert_eq!(world.finished_upgrade(site), Some(UpgradeCategory::WONDER));
    assert_eq!(
        world.tile_holder(site).and_then(|holder| holder.faction()),
        Some(FactionId(0)),
        "the builders hold the ground"
    );
    let end = world.game_end();
    assert!(
        end.is_set(),
        "the wonder completed and the game did not end"
    );
    assert_eq!(end.tick, world.tick());
    assert_eq!(end.winner, FactionId(0));
    assert_eq!(end.win_path(), Some(cachette_core::WinPath::WealthOrWonder));
    let census = world.subsystem_census();
    let count = |name: &str| {
        census
            .iter()
            .find(|(row, _)| *row == name)
            .map(|row| row.1)
            .expect("the census holds the row")
    };
    assert_eq!(count("wonders_complete"), 1);
    assert_eq!(count("stores_built"), 0);
}

// ---------------------------------------------------------------------------
// Renown
// ---------------------------------------------------------------------------

#[test]
fn a_character_at_the_renown_target_ends_the_game_and_one_below_does_not() {
    let mut world = world(2, 36, 32);
    let address = open_address(&world);
    world
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    world
        .spawn_soldier(address, FactionId(1))
        .expect("the ground admits a second unit");
    let person = world
        .create_character(FactionId(1))
        .expect("the arena has room");
    assert!(world.set_character_renown(person, Fix32(RENOWN_TARGET - 1)));
    step(&mut world);
    assert!(!world.game_end().is_set(), "one raw unit below the target");
    assert_eq!(
        world
            .standing(FactionId(1))
            .expect("faction 1 exists")
            .best_renown,
        i64::from(RENOWN_TARGET - 1)
    );
    assert!(world.set_character_renown(person, Fix32(RENOWN_TARGET)));
    step(&mut world);
    let end = world.game_end();
    assert!(end.is_set());
    assert_eq!(end.winner, FactionId(1));
    assert_eq!(end.win_path(), Some(cachette_core::WinPath::Renown));
}

// ---------------------------------------------------------------------------
// The order, and once
// ---------------------------------------------------------------------------

#[test]
fn two_paths_true_on_one_tick_record_the_earlier_path_of_the_fixed_order() {
    // Domination before renown. Faction 1 alone has a unit, and faction 0
    // has a character at the target. Domination is earlier in the order,
    // and its winner is not the renown winner, so the record tells which
    // reader wrote it.
    let mut first = world(2, 37, 32);
    let address = open_address(&first);
    first
        .spawn_soldier(address, FactionId(1))
        .expect("the ground admits a unit");
    let person = first
        .create_character(FactionId(0))
        .expect("the arena has room");
    assert!(first.set_character_renown(person, Fix32(RENOWN_TARGET)));
    step(&mut first);
    let end = first.game_end();
    assert_eq!(end.win_path(), Some(cachette_core::WinPath::Domination));
    assert_eq!(end.winner, FactionId(1));

    // Territory before wealth. The limit is the first tick and a store of
    // faction 1 is at the target. Territory names faction 0, because every
    // count is zero and the lowest identifier wins the tie.
    let mut second = World::new(config(2, 38, 32)).expect("the extent describes a world");
    let address = open_address(&second);
    second
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    second
        .spawn_soldier(address, FactionId(1))
        .expect("the ground admits a second unit");
    let site = settle(&mut second, FactionId(1));
    let at = Fix32(i32::try_from(STOCK_TARGET).expect("the target fits a store"));
    second
        .set_settlement_store(site, CommodityId(0), at)
        .expect("the commodity exists");
    second.set_tick_limit(1);
    step(&mut second);
    let end = second.game_end();
    assert_eq!(end.win_path(), Some(cachette_core::WinPath::Territory));

    // Wealth before renown. A store of faction 1 at the target and a
    // character of faction 0 at the renown target.
    let mut third = world(2, 39, 32);
    let address = open_address(&third);
    third
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    third
        .spawn_soldier(address, FactionId(1))
        .expect("the ground admits a second unit");
    let site = settle(&mut third, FactionId(1));
    third
        .set_settlement_store(site, CommodityId(0), at)
        .expect("the commodity exists");
    let person = third
        .create_character(FactionId(0))
        .expect("the arena has room");
    assert!(third.set_character_renown(person, Fix32(RENOWN_TARGET)));
    step(&mut third);
    let end = third.game_end();
    assert_eq!(end.win_path(), Some(cachette_core::WinPath::WealthOrWonder));
    assert_eq!(end.winner, FactionId(1));
}

#[test]
fn a_path_that_becomes_true_after_the_end_changes_nothing() {
    let mut world = world(2, 40, 48);
    let address = open_address(&world);
    world
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    world
        .spawn_soldier(address, FactionId(1))
        .expect("the ground admits a second unit");
    let person = world
        .create_character(FactionId(1))
        .expect("the arena has room");
    assert!(world.set_character_renown(person, Fix32(RENOWN_TARGET)));
    step(&mut world);
    let end = world.game_end();
    assert_eq!(end.win_path(), Some(cachette_core::WinPath::Renown));
    assert_eq!(end.winner, FactionId(1));

    // Faction 0 now reaches the stock target, and would win on an earlier
    // path if a reader ran. None runs.
    let site = settle(&mut world, FactionId(0));
    let at = Fix32(i32::try_from(STOCK_TARGET).expect("the target fits a store"));
    world
        .set_settlement_store(site, CommodityId(0), at)
        .expect("the commodity exists");
    for _ in 0..3 {
        step(&mut world);
        assert_eq!(world.game_end(), end, "the record is written once");
    }
}

#[test]
fn the_standing_of_a_faction_names_every_path_and_refuses_a_stranger() {
    let mut world = world(2, 41, 32);
    assert_eq!(world.standing(FactionId(2)), None);
    let standing = world.standing(FactionId(0)).expect("faction 0 exists");
    assert_eq!(standing, cachette_core::Standing::default());
    let site = settle(&mut world, FactionId(0));
    world
        .set_settlement_store(site, CommodityId(0), Fix32(7))
        .expect("the commodity exists");
    assert_eq!(
        world
            .standing(FactionId(0))
            .expect("faction 0 exists")
            .store_total,
        7
    );
    // The store raise of a site is the sum of the finished stores on or
    // beside it, and nothing stands beside a fresh site.
    assert_eq!(world.store_capacity_raise(site), Some(0));
}
