//! The two summary fields that the ground fixes, and the two it derives.
//!
//! A level 1 cell is the exact sum of its level 0 tiles.[^1] These tests read
//! the tiles of a cell through the public tile readers and compare.
//!
//! The height square total and the deposit count are pure functions of the
//! seed and the address, so the world computes them once and the rebuild
//! never touches them.[^2] The water count is derived from the tile count and
//! the open count, so nothing stores it.[^3]
//!
//! # References
//!
//! [^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
//! [^2]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
//! [^3]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`

use cachette_core::{sim_math, Accum, Axial, ResourceKind, TileKind, World, WorldConfig};

/// Builds a world of a given extent and seed.
fn world(width: u32, height: u32, seed: u64) -> World {
    World::new(WorldConfig {
        width,
        height,
        seed,
        faction_count: 2,
        unit_capacity: 16,
    })
    .expect("the extent must describe a world")
}

/// Returns the four ground quantities that one tile contributes.
///
/// The reader builds them from the public tile readers of the world, so a
/// mismatch names a defect in the pyramid and never in this test.
fn ground_of(world: &World, address: Axial) -> (i64, i64, i64, i64) {
    let ground = world
        .tile_terrain(address)
        .expect("the address is inside the world");
    let square = i64::from(sim_math::mul(ground.height, ground.height).0);
    let deposit = ResourceKind::ALL.iter().any(|kind| {
        world
            .resources()
            .original(address, *kind)
            .is_some_and(|stock| stock.0 > 0)
    });
    let water = i64::from(ground.kind == TileKind::Water);
    (1, square, i64::from(deposit), water)
}

#[test]
fn a_cell_holds_the_exact_sum_of_the_ground_of_its_tiles() {
    let world = world(41, 37, 11);
    let mut by_cell: Vec<(i64, i64, i64, i64)> = vec![(0, 0, 0, 0); world.pyramid().len()];
    for row in 0..world.grid().height() {
        for column in 0..world.grid().width() {
            let here = Axial::new(column as i32, row as i32);
            let cell = world
                .cell_covering(here)
                .expect("the address is inside the world");
            let part = ground_of(&world, here);
            let slot = &mut by_cell[cell as usize];
            slot.0 += part.0;
            slot.1 += part.1;
            slot.2 += part.2;
            slot.3 += part.3;
        }
    }

    let mut deposits = 0i64;
    let mut water = 0i64;
    for (cell, expected) in by_cell.iter().enumerate() {
        let summary = world.pyramid().cell(cell as u32).expect("the cell exists");
        assert_eq!(summary.tiles(), expected.0, "cell {cell} tile count");
        assert_eq!(
            summary.height_square_total().0,
            expected.1,
            "cell {cell} height square total"
        );
        assert_eq!(summary.deposit_tiles(), expected.2, "cell {cell} deposits");
        assert_eq!(summary.water_tiles(), expected.3, "cell {cell} water");
        deposits += expected.2;
        water += expected.3;
    }
    assert!(
        by_cell.len() > 1,
        "the fixture must hold more than one cell, or the sum is the world"
    );
    assert!(
        deposits > 0,
        "the fixture must reach a tile that carries a deposit"
    );
    assert!(water > 0, "the fixture must reach a tile of open water");
}

#[test]
fn the_sum_of_every_cell_is_the_sum_of_every_tile() {
    let world = world(41, 37, 11);
    let mut square = 0i64;
    let mut deposits = 0i64;
    let mut water = 0i64;
    for row in 0..world.grid().height() {
        for column in 0..world.grid().width() {
            let part = ground_of(&world, Axial::new(column as i32, row as i32));
            square += part.1;
            deposits += part.2;
            water += part.3;
        }
    }
    assert!(deposits > 0, "the fixture must reach a deposit");
    assert!(water > 0, "the fixture must reach open water");
    let total = world.pyramid().total();
    assert_eq!(total.height_square_total().0, square);
    assert_eq!(total.deposit_tiles(), deposits);
    assert_eq!(total.water_tiles(), water);
}

#[test]
fn the_ground_fields_do_not_move_when_the_world_steps() {
    let mut world = world(24, 24, 7);
    let before: Vec<(Accum, i64)> = world
        .pyramid()
        .cells()
        .iter()
        .map(|cell| (cell.height_square_total(), cell.deposit_tiles()))
        .collect();
    world.step(4).expect("the step must run");
    let after: Vec<(Accum, i64)> = world
        .pyramid()
        .cells()
        .iter()
        .map(|cell| (cell.height_square_total(), cell.deposit_tiles()))
        .collect();
    assert_eq!(
        before, after,
        "a field the ground fixes must cost nothing on a tick and must not move"
    );
    assert!(
        before.iter().any(|(square, _)| square.0 > 0),
        "the fixture must reach a cell that holds height"
    );
}

#[test]
fn a_cell_reports_a_spread_and_a_flat_world_reports_none() {
    let world = world(41, 37, 11);
    let mut broken = 0usize;
    for cell in 0..world.pyramid().len() as u32 {
        let summary = world.pyramid().cell(cell).expect("the cell exists");
        let Some(spread) = summary.height_spread() else {
            assert_eq!(summary.tiles(), 0);
            continue;
        };
        assert!(spread.0 >= 0, "a spread is never below zero");
        if spread.0 > 0 {
            broken += 1;
        }
    }
    assert!(
        broken > 0,
        "the fixture must reach a cell whose ground is not flat"
    );
}
