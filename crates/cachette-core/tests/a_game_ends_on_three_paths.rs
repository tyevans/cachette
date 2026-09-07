//! Four readers end a game, in one fixed order, once.
//!
//! Each reader fires on a fixture at its extreme. Domination fires for a
//! faction whose rivals hold no unit, and for a faction that holds every
//! seat while a rival still lives. The wonder fires when a finished wonder
//! stands on ground its faction holds, and not one unit of work short.
//! Renown fires at the renown target and not below. Two paths true on one
//! tick record the earlier path of the fixed order, and a path that becomes
//! true later changes nothing.[^1]
//!
//! **A stock total wins no game.** The wealth clause is gone, and the wonder
//! is a path of its own.[^2] Three tests hold that rule at the extremes the
//! old clause fired at: a stock total at the ceiling of one store, a stock
//! total that overflows a 32-bit accumulator, and the accumulator at the
//! target scale. Each of them steps the world and asserts that the record
//! stays empty, while the standing still reports the quantity.
//!
//! # References
//!
//! [^1]: ADR-0148, a game end is recorded once and stops the controllers, decisions D2 and D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
//! [^2]: ADR-0174, a wonder is a win path and a stock total is not, decisions D1 and D2. `docs/adrs/draft/adr-0174-a-wonder-is-a-win-path-and-a-stock-total-is-not.md`

use cachette_core::choose;
use cachette_core::sim_math::combine;
use cachette_core::site::CommodityId;
use cachette_core::upgrade::{UpgradeCategory, UpgradeRow};
use cachette_core::{
    Accum, Axial, Entity, FactionId, Fix32, WinPath, World, WorldConfig, RENOWN_TARGET,
    STOCK_CEILING_OF_ONE_SETTLEMENT,
};

const THREADS: usize = 2;

/// The work this file asks a wonder for.
///
/// **The fixture states its own bar and does not read the balance value.**
/// The project owner raised the wonder work on 5 September 2026, and a tile
/// of eight builders now dies before it finishes one.[^1] A fixture that
/// read the register would measure how long a unit lives and not the
/// reader. The value is the one the register held before the raise.
///
/// # References
///
/// [^1]: Balance register, the wonder work. `docs/reference/balance.md`
const FIXTURE_WONDER_WORK: u32 = 240;

/// The wonder row this file writes, so that the fixture states its own work.
///
/// Every other column is the column the default table holds for a wonder: it
/// stands on any land, it carries a victory claim, and its builder must
/// stand on ground its own faction holds.
fn fixture_wonder_row() -> UpgradeRow {
    UpgradeRow {
        ground_fit: cachette_core::upgrade::FITS_EVERY_LAND,
        work: FIXTURE_WONDER_WORK,
        victory_claim: cachette_core::upgrade::WONDER_VICTORY_CLAIM,
        own_ground_required: cachette_core::upgrade::OWN_GROUND_REQUIRED,
        ..UpgradeRow::NONE
    }
}

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

/// Founds one settlement of a faction for each amount, and fills its store
/// with that amount.
///
/// Returns the sum the fixture put in. **The sum is computed from the
/// amounts and is never read back from the reader under test**, so a reader
/// that lost a settlement fails the assertion rather than agreeing with
/// itself.
fn settle_and_fill(world: &mut World, faction: FactionId, amounts: &[i32]) -> (Vec<Entity>, i64) {
    let mut sites = Vec::new();
    let mut sum = 0i64;
    for amount in amounts {
        let site = settle(world, faction);
        world
            .set_settlement_store(site, CommodityId(0), Fix32(*amount))
            .expect("the commodity exists");
        sites.push(site);
        sum += i64::from(*amount);
    }
    (sites, sum)
}

/// Gives each faction a unit, so the domination reader stays quiet.
fn arm_both_factions(world: &mut World) {
    let address = open_address(world);
    world
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    world
        .spawn_soldier(address, FactionId(1))
        .expect("the ground admits a second unit");
}

#[test]
fn one_settlement_filled_to_the_ceiling_of_its_store_wins_nothing() {
    // A store is a `Fix32`, so one settlement of one commodity stops here
    // however long the world runs. No reader watches the total, so a full
    // settlement wins nothing whatever the bar is.
    let mut world = world(2, 36, 48);
    arm_both_factions(&mut world);
    let (_, put_in) = settle_and_fill(&mut world, FactionId(1), &[i32::MAX]);
    assert_eq!(
        put_in, STOCK_CEILING_OF_ONE_SETTLEMENT,
        "the fixture stands at the ceiling of one store"
    );
    for _ in 0..4 {
        step(&mut world);
        assert!(
            !world.game_end().is_set(),
            "a full settlement that waits wins nothing"
        );
    }
    assert_eq!(
        world
            .standing(FactionId(1))
            .expect("faction 1 exists")
            .store_total,
        STOCK_CEILING_OF_ONE_SETTLEMENT
    );
}

#[test]
fn two_settlements_at_the_top_of_the_range_end_no_game_and_the_total_does_not_wrap() {
    let mut world = world(2, 37, 48);
    arm_both_factions(&mut world);
    // Two stores at the top of the 32-bit range. Their sum wraps to a
    // negative number in 32 bits. The reported total is a 64-bit sum, so it
    // stands above the bar and above the range of one store.
    let (_, put_in) = settle_and_fill(&mut world, FactionId(0), &[i32::MAX, i32::MAX]);
    assert_eq!(put_in, 2 * i64::from(i32::MAX));
    assert!(
        put_in > i64::from(i32::MAX),
        "the fixture reaches the overflow"
    );
    assert_eq!(
        world
            .standing(FactionId(0))
            .expect("faction 0 exists")
            .store_total,
        put_in
    );
    for _ in 0..4 {
        step(&mut world);
        assert!(
            !world.game_end().is_set(),
            "however large, a stock total ends no game"
        );
    }
}

#[test]
fn the_stock_total_does_not_saturate_at_the_target_scale() {
    // The most the reader can ever total: one settlement for each tile of
    // the target world, each filled to the ceiling of its store. The
    // accumulator the reader uses must hold that exactly.
    //
    // **This test drives the accumulator and not the reader**, because no
    // fixture builds 16 million settlements. The test above drives the
    // reader over two settlements and fails when the reader narrows the
    // accumulator.
    const TARGET_TILE_COUNT: i64 = 16_777_216;
    let mut total = Accum(0);
    for _ in 0..TARGET_TILE_COUNT {
        total = combine(total, Accum(STOCK_CEILING_OF_ONE_SETTLEMENT));
    }
    assert_eq!(
        total.0,
        TARGET_TILE_COUNT * STOCK_CEILING_OF_ONE_SETTLEMENT,
        "the sum is exact, so nothing saturated"
    );
    assert!(total.0 < i64::MAX);
}

// ---------------------------------------------------------------------------
// Wonder
// ---------------------------------------------------------------------------

#[test]
fn a_wonder_finished_on_held_ground_ends_the_game_and_one_short_does_not() {
    let mut world = world(2, 102, 192);
    // The choice pass replaces the build order of a unit whose cell chooses
    // on that frame. The fixture keeps the choice away from the run.
    world
        .set_choice_schedule(choose::PERIOD_LOG2_CEILING)
        .expect("the exponent is inside the range");
    world
        .define_upgrade_row(UpgradeCategory::WONDER.to_u8(), 1, fixture_wonder_row())
        .expect("the category and the level are inside the table");
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
    // The work comes from the table the world holds, which this fixture
    // wrote, so the loop and the build read one declaration.
    let work = world.upgrade_table().work_above(UpgradeCategory::WONDER, 0);
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
        // [^2]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D3. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
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
    // **This is the rule this test states.** The wonder completes, on ground
    // its own faction holds, and the game ends on the wonder path.
    let end = world.game_end();
    assert!(end.is_set(), "a finished wonder ends the game");
    assert_eq!(end.win_path(), Some(WinPath::Wonder));
    assert_eq!(end.winner, FactionId(0));
    // The record is written once, so a later step leaves it where it is.
    let tick = end.tick;
    for _ in 0..4 {
        step(&mut world);
        assert_eq!(world.game_end().tick, tick, "the record is written once");
    }
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

    // Territory before renown. The limit is the first tick, both factions
    // hold a unit so domination stays quiet, and a character of faction 1 is
    // at the renown target. Territory names faction 0, because every held
    // count is zero and the lowest identifier wins the tie, so the winner
    // tells which reader wrote the record.
    let mut second = World::new(config(2, 38, 48)).expect("the extent describes a world");
    arm_both_factions(&mut second);
    let person = second
        .create_character(FactionId(1))
        .expect("the arena has room");
    assert!(second.set_character_renown(person, Fix32(RENOWN_TARGET)));
    second.set_tick_limit(1);
    step(&mut second);
    let end = second.game_end();
    assert_eq!(end.win_path(), Some(cachette_core::WinPath::Territory));
    assert_eq!(end.winner, FactionId(0));

    // Domination before territory. Only faction 1 holds a unit, and the
    // limit is the first tick, so both readers fire on that tick. Domination
    // names faction 1, and territory would name faction 0 on the tie the
    // case above resolves, so the winner tells which reader wrote it.
    let mut third = World::new(config(2, 39, 48)).expect("the extent describes a world");
    let address = open_address(&third);
    third
        .spawn_soldier(address, FactionId(1))
        .expect("the ground admits a unit");
    third.set_tick_limit(1);
    step(&mut third);
    let end = third.game_end();
    assert_eq!(end.win_path(), Some(cachette_core::WinPath::Domination));
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

    // The tick limit now falls behind the tick, so the territory reader
    // would fire and would name an earlier path with a different winner.
    // **The later fact must be one a reader watches.** The fixture used a
    // stock total, and the stock total has no reader, so it would have made
    // this test compare the record against nothing.
    world.set_tick_limit(world.tick().0);
    for _ in 0..3 {
        step(&mut world);
        assert_eq!(world.game_end(), end, "the record is written once");
    }
}

#[test]
fn the_standing_of_a_faction_names_every_running_value_and_refuses_a_stranger() {
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
