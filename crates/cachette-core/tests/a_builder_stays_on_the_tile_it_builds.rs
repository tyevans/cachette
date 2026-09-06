//! A unit under a build order stays on the tile it builds until the work ends.
//!
//! The build pass adds the work of a unit to the site on the tile the unit
//! stands on.[^1] One level of one upgrade asks for more work than one unit
//! adds in one tick, so a site finishes only when the same unit stands on the
//! same tile for several ticks. The movement pass therefore asks whether the
//! unit stands on the work its order names, and it leaves that unit alone.[^2]
//!
//! **The hold is derived on every tick and nothing stores it.**[^2] Each test
//! below drives a condition false and asserts that the unit moves again, so
//! together they state that no hold outlives the reason for it.
//!
//! The tests drive the engine through the public verbs. A test that called the
//! movement pass directly would prove that the pass works and not that
//! anything reaches it.[^3]
//!
//! **The fixture is built for the case these tests need, and it is not a copy
//! of the demonstration world.**[^4] The tile is chosen because a unit that
//! carries no order leaves it, and the fixture asserts that before it asserts
//! anything else. A tile that a unit never leaves would pass every test below
//! whatever the movement pass did.
//!
//! # References
//!
//! [^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decisions D1 and D2. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
//! [^2]: ADR-0165, a build order holds a unit on its tile, and the hold is derived and never stored, decisions D1, D2 and D3. `docs/adrs/draft/adr-0165-a-build-order-holds-a-unit-on-its-tile.md`
//! [^3]: Testing rules, section 5. `.agents/rules/testing.md`
//! [^4]: Testing rules, section 2a. `.agents/rules/testing.md`

use cachette_core::unit_type;
use cachette_core::upgrade::{UpgradeCategory, ROAD_LEVEL_1_WORK, ROAD_LEVEL_2_WORK};
use cachette_core::{Axial, Entity, FactionId, TileIdx, World, WorldConfig};

/// The extent of every fixture world.
///
/// The extent covers several level 1 blocks in each direction, so the choice
/// pass and the exit field both have a lattice to work on.
const EXTENT: u32 = 64;

/// The seed of every fixture world.
///
/// The seed is the one the other movement fixtures of this project use.
const SEED: u64 = 7;

/// The number of frames a free unit gets to leave the tile it started on.
///
/// A unit whose cell exit the ground refuses falls back to a keyed draw, and
/// a tile may hold closed ground on several neighbours, so one frame is not
/// enough.[^1] This count is a bound on a random walk and not a budget.[^2]
///
/// # References
///
/// [^1]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D6. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
/// [^2]: Decision Record Scope, section 4.1. `.agents/rules/adr-scope.md`
const FRAMES: u64 = 32;

/// The work that finishes both levels of a road.
///
/// A worker adds one work in one tick, so this is also the number of ticks a
/// worker takes to reach the top level of a road on its own.[^1]
///
/// # References
///
/// [^1]: Balance register, the road work by level. `docs/reference/balance.md`
const ROAD_WORK: u64 = ROAD_LEVEL_1_WORK as u64 + ROAD_LEVEL_2_WORK as u64;

/// Builds a fixture world with the choice on every tick.
///
/// A unit that holds no intent does not move, so a world whose choice pass
/// never ran would freeze every unit and every test below would pass.[^1]
///
/// # References
///
/// [^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D4. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
fn fixture() -> World {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: SEED,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    world
        .set_choice_schedule(0)
        .expect("the exponent is inside the range");
    world.rebuild_pyramid(1).expect("the rebuild must run");
    world
}

/// Returns the tile that every test below builds on.
///
/// **The tile is the fixture, and it is chosen by the case and not by how it
/// looks.**[^1] The scan takes the first tile in ascending order that admits a
/// unit, that a road fits, and that a unit carrying no order leaves inside the
/// frame bound. The last clause is what makes the assertions able to fail.
///
/// The scan order is ascending tile order, so the answer is fixed and does not
/// depend on how a caller walked the world.[^2]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
/// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
fn a_tile_a_free_unit_leaves() -> Axial {
    let probe = fixture();
    let grid = probe.grid();
    for tile in 0..grid.tile_count() {
        let here = grid
            .address_of(TileIdx(tile))
            .expect("the tile is inside the world");
        if !probe.admits_a_unit(here) {
            continue;
        }
        let mut world = fixture();
        if world.zone_project(FactionId(0), here, ROAD).is_err() {
            continue;
        }
        let unit = world
            .spawn_soldier(here, FactionId(0))
            .expect("the ground admits the unit");
        if leaves(&mut world, unit, here) {
            return here;
        }
    }
    panic!("no tile of the fixture world admits a road and lets a free unit leave");
}

/// The category that every test below builds.
///
/// A road asks for no held ground, so the plan of the faction is the only
/// thing that permits it. That makes the plan the one condition a test drives
/// to release the unit.[^1]
///
/// # References
///
/// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decisions D3 and D4. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
const ROAD: UpgradeCategory = UpgradeCategory::ROAD;

/// Steps the world and reports whether the unit left the tile.
fn leaves(world: &mut World, unit: Entity, from: Axial) -> bool {
    for _ in 0..FRAMES {
        world.step(1).expect("the step must run");
        if world.soldiers().address(unit) != Some(from) {
            return true;
        }
    }
    false
}

/// Builds a world that holds one zoned road and one worker standing on it.
fn a_worker_on_a_zoned_road(tile: Axial) -> (World, Entity) {
    let mut world = fixture();
    world
        .zone_project(FactionId(0), tile, ROAD)
        .expect("the ground fits a road");
    let unit = world
        .spawn_soldier(tile, FactionId(0))
        .expect("the ground admits the unit");
    world
        .order_build(unit, ROAD)
        .expect("the plan of the faction zones this tile");
    (world, unit)
}

#[test]
fn a_unit_with_no_build_order_leaves_the_tile_it_stands_on() {
    // This is the control of every test below. A unit that carries no order
    // reads the exit of its cell and steps, and nothing in the movement pass
    // changed for it.
    let tile = a_tile_a_free_unit_leaves();
    let mut world = fixture();
    world
        .zone_project(FactionId(0), tile, ROAD)
        .expect("the ground fits a road");
    let unit = world
        .spawn_soldier(tile, FactionId(0))
        .expect("the ground admits the unit");

    assert!(
        leaves(&mut world, unit, tile),
        "a unit with no build order held tile {tile:?} for {FRAMES} frames"
    );
}

#[test]
fn a_unit_under_a_build_order_does_not_move_while_work_remains() {
    let tile = a_tile_a_free_unit_leaves();
    let (mut world, unit) = a_worker_on_a_zoned_road(tile);

    assert!(
        !leaves(&mut world, unit, tile),
        "the builder left tile {tile:?} while the road was unfinished"
    );
    // The unit stood still and it also worked. A unit that stood still and
    // built nothing would pass the line above and would be a different
    // defect.
    assert!(
        world.upgrade_level(tile) > 0,
        "the builder held the tile and finished no level of the road"
    );
}

#[test]
fn the_hold_clears_when_the_work_finishes() {
    let tile = a_tile_a_free_unit_leaves();
    let (mut world, unit) = a_worker_on_a_zoned_road(tile);

    // A worker adds one work in one tick, so the road reaches its top level
    // after the work of both levels. The extra frames give the unit room to
    // step once the road holds no row above it.
    for _ in 0..ROAD_WORK {
        world.step(1).expect("the step must run");
    }
    assert_eq!(
        world.upgrade_level(tile),
        2,
        "the road did not reach its top level, so this test cannot say \
         whether a finished build releases the unit"
    );
    assert_eq!(
        world.soldiers().address(unit),
        Some(tile),
        "the builder left before the road was finished"
    );

    assert!(
        leaves(&mut world, unit, tile),
        "the builder held tile {tile:?} after the road reached its top level"
    );
}

#[test]
fn the_hold_clears_when_the_plan_drops_the_project() {
    let tile = a_tile_a_free_unit_leaves();
    let (mut world, unit) = a_worker_on_a_zoned_road(tile);

    world.step(1).expect("the step must run");
    assert_eq!(
        world.soldiers().address(unit),
        Some(tile),
        "the builder left before the plan dropped the project"
    );

    assert!(
        world.clear_project(FactionId(0), tile),
        "the plan held the project this test drops"
    );
    assert!(
        leaves(&mut world, unit, tile),
        "the builder held tile {tile:?} after the plan dropped the project"
    );
}

#[test]
fn a_unit_that_died_while_held_leaves_no_hold_for_the_next_unit() {
    let tile = a_tile_a_free_unit_leaves();
    let (mut world, unit) = a_worker_on_a_zoned_road(tile);

    world.step(1).expect("the step must run");
    assert_eq!(
        world.soldiers().address(unit),
        Some(tile),
        "the builder left before this test removed it"
    );
    assert!(world.despawn_soldier(unit), "the builder was live");

    // The next spawn takes the slot the dead builder freed, so a hold that
    // lived in that slot would move to a unit that nobody ordered.
    let next = world
        .spawn_soldier(tile, FactionId(0))
        .expect("the ground admits the unit");
    assert_eq!(
        world.build_order(next),
        Some(None),
        "the new unit inherited a build order from the slot"
    );
    assert!(
        leaves(&mut world, next, tile),
        "a unit that inherited a dead builder's slot held tile {tile:?}"
    );
}

#[test]
fn a_unit_that_adds_no_work_is_never_held() {
    // The soldier row builds at zero, and the built-in controller orders
    // every unit of its faction that stands on a zoned tile, whatever its
    // type.[^1] A hold that ignored the rate would freeze such a unit for the
    // rest of the run, because no work would ever finish the site under
    // it.[^2]
    //
    // [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D5. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    // [^2]: ADR-0165, a build order holds a unit on its tile, and the hold is derived and never stored, decision D3. `docs/adrs/draft/adr-0165-a-build-order-holds-a-unit-on-its-tile.md`
    let tile = a_tile_a_free_unit_leaves();
    let (mut world, unit) = a_worker_on_a_zoned_road(tile);
    assert!(
        world.set_unit_type(unit, unit_type::SOLDIER),
        "the unit is live"
    );
    assert_eq!(
        world
            .unit_type_row(unit)
            .expect("the unit is live")
            .build_rate,
        cachette_core::types::Fix32::ZERO,
        "the soldier row builds above zero, so this test asserts nothing"
    );

    assert!(
        leaves(&mut world, unit, tile),
        "a unit that adds no work held tile {tile:?} under a build order"
    );
    assert_eq!(
        world.upgrade_level(tile),
        0,
        "a unit that builds at zero raised the road"
    );
}
