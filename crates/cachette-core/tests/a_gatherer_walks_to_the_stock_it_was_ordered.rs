//! A unit ordered to gather walks to ground that holds the kind it wants.
//!
//! The exit field answers at the pitch of one level 1 cell, and the stock of a
//! tile is a level 0 property. A unit that stands on barren ground inside the
//! cell that holds the most food reads that its own cell is the best one. The
//! cell that no neighbour beats holds no direction at all, so the unit stood
//! there while the world still held food two tiles away.[^1]
//!
//! A field at the pitch of one tile answers the last cell. It is the same
//! instrument that steers a sent unit onto the tile it was sent to, and a
//! laden unit onto the tile of its own home, keyed on the resource kind
//! instead.[^2]
//!
//! Every test here drives the step. None calls the movement pass and none
//! calls the gather resolve.[^3]
//!
//! **The fixture is built for these tests.** It does not copy the world of the
//! demonstration binary, because that world is chosen to look right and not to
//! produce an extreme.[^4] The extreme this test needs is a unit on a bare
//! tile inside the richest cell of the world. That unit reads no coarse
//! direction, because no neighbouring cell beats the one it stands in, so the
//! coarse field alone leaves it where it is.
//!
//! # References
//!
//! [^1]: Findings register, FND-589. `docs/FINDINGS.md`
//! [^2]: Findings register, FND-315. `docs/FINDINGS.md`
//! [^3]: Testing rules, section 5. `.agents/rules/testing.md`
//! [^4]: Testing rules, section 2a. `.agents/rules/testing.md`

use cachette_core::choose::{self, SCORE_FLOOR};
use cachette_core::cohort::{NeedRule, NEED_FULL};
use cachette_core::pyramid::AT_SEED;
use cachette_core::resource::{Amount, ResourceKind};
use cachette_core::{Axial, Entity, FactionId, Fix32, World, WorldConfig};

/// The extent of every fixture world.
const EXTENT: u32 = 256;

/// The seed of every fixture world.
///
/// Each test asserts the property of the ground that it depends on, so a
/// change to the generator fails the fixture rather than the assertion.
const SEED: u64 = 7;

/// The option index of the row that scores the food of a cell.
const FORAGE: u8 = 2;

/// The number of frames that the walk gets.
///
/// The fixture asserts that the unit starts inside the block that holds its
/// food, so the walk is bounded by the block edge. The count is twice that
/// bound, which admits a detour of the same length again.
const FRAMES: usize = 64;

/// The thread counts that the equivalence test runs at.
const THREAD_COUNTS: [usize; 3] = [1, 2, 12];

/// Builds a world of many level 1 cells, with the choice on every tick.
fn world() -> World {
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
    world
}

/// Drains the need of every unit in one tick, and lets nobody die of it.
///
/// The `forage` row is driven by what a unit lacks. A fixture that left the
/// need alone would measure the drive and not the ground.
fn starve(world: &mut World) {
    world
        .set_economy_schedule(1, 0)
        .expect("the period is inside the range");
    let rule = NeedRule::new(
        NEED_FULL,
        NEED_FULL,
        Fix32(NEED_FULL.0 / 2),
        Fix32(NEED_FULL.0 / 16),
        Fix32::MAX,
    )
    .expect("every rate is at or above zero");
    world.set_need_rule(rule);
}

/// Puts every weight on one option and none on the others.
fn only(world: &mut World, option: u8, weight: Fix32) {
    for index in 0..choose::OPTION_COUNT as u8 {
        world
            .set_option_weight(index, Fix32::ZERO)
            .expect("the index is inside the set");
    }
    world
        .set_option_weight(option, weight)
        .expect("the index is inside the set");
}

/// Returns every address of one cell, in ascending row and then column.
fn addresses_of(world: &World, cell: u32) -> Vec<Axial> {
    let layout = world.pyramid().layout();
    let edge = layout.block_edge();
    let first_column = (cell % layout.blocks_wide()) * edge;
    let first_row = (cell / layout.blocks_wide()) * edge;
    let mut addresses = Vec::new();
    for row in first_row..first_row + edge {
        for column in first_column..first_column + edge {
            addresses.push(Axial::new(column as i32, row as i32));
        }
    }
    addresses
}

/// Returns the cell that holds the most food, by the mean of its tiles.
///
/// The scan is over the cells in ascending index and it compares strictly, so
/// the lowest index wins a tie.
fn richest_cell(world: &World) -> (u32, Fix32) {
    let mut best = (0u32, Fix32::ZERO);
    for cell in 0..world.pyramid().layout().grid().tile_count() {
        let Some(food) = world.pyramid().cell(cell).and_then(|it| it.mean_food()) else {
            continue;
        };
        if food.0 > best.1 .0 {
            best = (cell, food);
        }
    }
    best
}

/// Builds the fixture: a hungry unit on a bare tile of the richest cell.
///
/// Returns the world, the unit, and the address it starts on.
fn a_bare_tile_in_the_richest_cell() -> (World, Entity, Axial) {
    let mut world = world();
    starve(&mut world);
    let (cell, food) = richest_cell(&world);
    assert!(food.0 > 0, "the fixture holds no food anywhere");
    // The weight puts the score of the richest cell above the floor, so the
    // unit acts on the row rather than holding.
    let weight = Fix32(((i64::from(SCORE_FLOOR.0) << 16) / i64::from(food.0)) as i32 + 1);
    only(&mut world, FORAGE, weight);

    let addresses = addresses_of(&world, cell);
    let bare = addresses
        .iter()
        .copied()
        .find(|address| {
            world.admits_a_unit(*address)
                && world.tile_stock(*address, ResourceKind::Food) == Some(Amount::ZERO)
        })
        .expect("the richest cell holds no open tile that carries no food");
    let stocked = addresses
        .iter()
        .copied()
        .filter(|address| {
            world.admits_a_unit(*address)
                && world
                    .tile_stock(*address, ResourceKind::Food)
                    .is_some_and(|amount| amount.0 > 0)
        })
        .count();
    assert!(
        stocked > 0,
        "the richest cell holds no open tile that carries food"
    );
    // **No neighbouring cell beats the one the unit stands in**, so the
    // coarse field holds no direction here. This is the case the fine field
    // exists for, and a fixture that did not assert it would measure a unit
    // that the coarse field was already steering.
    assert_eq!(
        world.exit_direction(bare, FORAGE),
        Some(None),
        "the richest cell holds a coarse direction, so the fixture is not the case"
    );

    let unit = world
        .spawn_soldier(bare, FactionId(0))
        .expect("the open tile admits a unit");
    (world, unit, bare)
}

#[test]
fn a_unit_on_bare_ground_reaches_the_stock_of_its_own_cell() {
    let (mut world, unit, start) = a_bare_tile_in_the_richest_cell();
    assert_eq!(
        world.tile_stock(start, ResourceKind::Food),
        Some(Amount::ZERO),
        "the unit starts on food"
    );
    let mut stood_on_stock = None;
    for frame in 0..FRAMES {
        world.step(1).expect("the step must run");
        let Some(address) = world.soldiers().address(unit) else {
            panic!("the unit died in the fixture");
        };
        if world.stock_direction(address, ResourceKind::Food) == Some(AT_SEED) {
            stood_on_stock = Some(frame);
            break;
        }
    }
    let frame = stood_on_stock.expect("the unit never reached a tile that holds food");
    // The gather resolve runs after the movement of the same frame, so the
    // frame that carried the unit onto the stock also took from it. The load
    // is what proves that the walk reached the ground the order named, and
    // not only a tile the field liked.
    let taken: u32 = world
        .soldiers()
        .carry(unit)
        .map_or(0, |load| load.of(ResourceKind::Food).0);
    assert!(
        taken > 0,
        "the unit stood on food at frame {frame} and took none"
    );
}

#[test]
fn the_field_says_the_unit_stands_on_stock_only_where_stock_is() {
    // The field is the one statement of where the stock is, and the tile
    // column is the other. A test that read only the field would pass on a
    // field that answered the seed offset everywhere.
    let (mut world, unit, _) = a_bare_tile_in_the_richest_cell();
    for _ in 0..FRAMES {
        world.step(1).expect("the step must run");
        let Some(address) = world.soldiers().address(unit) else {
            panic!("the unit died in the fixture");
        };
        let says = world.stock_direction(address, ResourceKind::Food);
        let here = world
            .tile_stock(address, ResourceKind::Food)
            .map_or(0, |amount| amount.0);
        assert_eq!(
            says == Some(AT_SEED),
            here > 0,
            "the field and the tile disagree at {address:?}: {says:?} against {here}"
        );
    }
}

#[test]
fn the_walk_does_not_depend_on_the_thread_count() {
    let mut walks = Vec::new();
    for threads in THREAD_COUNTS {
        let (mut world, unit, _) = a_bare_tile_in_the_richest_cell();
        let mut walk = Vec::new();
        for _ in 0..FRAMES {
            world.step(threads).expect("the step must run");
            walk.push(world.soldiers().address(unit));
        }
        walks.push(walk);
    }
    assert_eq!(walks[0], walks[1], "one thread and two disagree");
    assert_eq!(walks[0], walks[2], "one thread and twelve disagree");
}
