//! The stock reader that takes a ground answers from that ground.
//!
//! The stock a tile started with is a function of the seed, the address and
//! the ground of the tile.[^1] A caller that has already read the ground
//! holds the third term. The readers under test take it, so the caller does
//! not pay for the ground a second time.
//!
//! **The saving is the whole point, and a reader that quietly regenerated the
//! ground would still give the right answer.** No test that supplies the true
//! ground can see the difference. Each test below therefore supplies a ground
//! that the address does not carry, and asserts that the answer follows the
//! argument. A reader that regenerated would answer from the address instead,
//! and the assertion would fail.[^2]
//!
//! A second family asserts that the two readers agree over a whole world.
//! One fact must not have two answers, and nothing else in the tree would
//! fail if they disagreed.[^3]
//!
//! The tests go through the public crate interface.[^4]
//!
//! # References
//!
//! [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D1. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
//! [^2]: Testing rules, section 2. `.claude/rules/testing.md`
//! [^3]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
//! [^4]: Testing rules, section 6. `.claude/rules/testing.md`

use cachette_core::resource::{Amount, ResourceKind};
use cachette_core::terrain::TileKind;
use cachette_core::{Axial, FactionId, TileIdx, World, WorldConfig};

/// The width of the world under test.
///
/// The extent is wider than the coarsest lattice spacing of the ground
/// generator, so the world holds every kind of ground. A world smaller than
/// that spacing holds one terrain, and a test over it measures the
/// fixture.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-054. `docs/FINDINGS.md`
const WIDTH: u32 = 192;
/// The number of rows of that extent.
const HEIGHT: u32 = 192;
/// The seed of the world under test.
const SEED: u64 = 0x0123_4567_89ab_cdef;
/// The number of gatherers that the fixture seats.
const UNITS: u32 = 32;
/// The number of frames that the fixture runs before the sweep.
const FRAMES: u32 = 8;

/// Builds the world under test.
fn world() -> World {
    World::new(WorldConfig {
        width: WIDTH,
        height: HEIGHT,
        seed: SEED,
        faction_count: 4,
        unit_capacity: 256,
    })
    .expect("the extent must describe a world")
}

/// Finds a tile whose ground is not water and which started with food.
///
/// Open water carries nothing of any kind, so it is the ground that makes the
/// difference visible.[^1] The search states what it wants rather than naming
/// an address, because an address is a figure that the generator can move.
///
/// # References
///
/// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D2. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
fn a_tile_with_food(world: &World) -> Axial {
    let grid = world.grid();
    let mut index = 0;
    while index < grid.tile_count() {
        let address = grid
            .address_of(TileIdx(index))
            .expect("the index is inside the extent");
        let ground = world
            .tile_kind(address)
            .expect("the address is inside the extent");
        let stock = world
            .original_stock(address, ResourceKind::Food)
            .expect("the address is inside the extent");
        if ground != TileKind::Water && stock.0 > 0 {
            return address;
        }
        index += 1;
    }
    panic!("the world holds no tile of dry ground that started with food");
}

#[test]
fn the_field_reader_answers_from_the_ground_it_is_given() {
    let world = world();
    let field = world.resources();
    let address = a_tile_with_food(&world);

    let from_the_address = world
        .original_stock(address, ResourceKind::Food)
        .expect("the address is inside the extent");
    let from_the_water = field
        .original_of_ground(address, TileKind::Water, ResourceKind::Food)
        .expect("the address is inside the extent");

    assert!(
        from_the_address.0 > 0,
        "the fixture must give a tile that started with food, and it gave {from_the_address:?}"
    );
    assert_eq!(
        from_the_water,
        Amount::ZERO,
        "the reader must answer from the ground it was given. Water carries nothing, so a \
         reader that regenerated the ground of the address would answer {from_the_address:?}"
    );
}

#[test]
fn the_world_reader_answers_from_the_ground_it_is_given() {
    let world = world();
    let address = a_tile_with_food(&world);

    let from_the_address = world
        .tile_stock(address, ResourceKind::Food)
        .expect("the address is inside the extent");
    let from_the_water = world
        .tile_stock_of_ground(address, TileKind::Water, ResourceKind::Food)
        .expect("the address is inside the extent");

    assert!(
        from_the_address.0 > 0,
        "the fixture must give a tile that holds food, and it gave {from_the_address:?}"
    );
    assert_eq!(
        from_the_water,
        Amount::ZERO,
        "the reader must answer from the ground it was given. Water carries nothing, so a \
         reader that regenerated the ground of the address would answer {from_the_address:?}"
    );
}

/// Puts entries into the depletion ledger.
///
/// The remaining stock of a tile is the stock it started with, less what
/// somebody took from it.[^1] A world in which nobody gathered holds an empty
/// ledger, so the second term is zero at every tile, and a reader that dropped
/// it would still agree with the reader beside it. The fixture therefore
/// gathers before the sweep, and the sweep asserts that the ledger is not
/// empty.[^2]
///
/// The engine does the gathering. A test that wrote the ledger itself would
/// prove that the ledger works and not that anything reaches it.[^3]
///
/// # References
///
/// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
/// [^2]: Testing rules, section 2a. `.claude/rules/testing.md`
/// [^3]: Testing rules, section 5. `.claude/rules/testing.md`
fn gather_somewhere(world: &mut World) {
    let grid = world.grid();
    let mut placed = 0;
    let mut index = 0;
    while index < grid.tile_count() && placed < UNITS {
        let address = grid
            .address_of(TileIdx(index))
            .expect("the index is inside the extent");
        let ground = world
            .tile_kind(address)
            .expect("the address is inside the extent");
        let food = world
            .original_stock(address, ResourceKind::Food)
            .expect("the address is inside the extent");
        if ground.capacity() > 0 && food.0 > 0 {
            let unit = world
                .spawn_soldier(address, FactionId(0))
                .expect("the open tile admits a unit");
            world.order_gather(unit, ResourceKind::Food);
            placed += 1;
        }
        index += 1;
    }
    assert_eq!(placed, UNITS, "the fixture must seat every gatherer");
    for _ in 0..FRAMES {
        world.step(1).expect("the step must run");
    }
}

#[test]
fn the_two_readers_agree_on_every_tile_and_every_kind() {
    let mut world = world();
    gather_somewhere(&mut world);
    assert!(
        !world.depletion().is_empty(),
        "the fixture gathered nothing, so the term that one reader could drop is zero everywhere"
    );
    let field = world.resources();
    let grid = world.grid();

    let mut index = 0;
    while index < grid.tile_count() {
        let address = grid
            .address_of(TileIdx(index))
            .expect("the index is inside the extent");
        let ground = world
            .tile_kind(address)
            .expect("the address is inside the extent");
        for kind in ResourceKind::ALL {
            assert_eq!(
                field.original_of_ground(address, ground, kind),
                world.original_stock(address, kind),
                "the two readers of the starting stock disagree at {address:?} for {kind:?}"
            );
            assert_eq!(
                world.tile_stock_of_ground(address, ground, kind),
                world.tile_stock(address, kind),
                "the two readers of the remaining stock disagree at {address:?} for {kind:?}"
            );
        }
        index += 1;
    }
}

#[test]
fn an_address_outside_the_world_names_no_tile() {
    let world = world();
    let outside = Axial::new(WIDTH as i32, HEIGHT as i32);

    assert_eq!(
        world
            .resources()
            .original_of_ground(outside, TileKind::Plain, ResourceKind::Food),
        None,
        "an address outside the extent names no tile, whatever ground the caller gives"
    );
    assert_eq!(
        world.tile_stock_of_ground(outside, TileKind::Plain, ResourceKind::Food),
        None,
        "an address outside the extent names no tile, whatever ground the caller gives"
    );
}
