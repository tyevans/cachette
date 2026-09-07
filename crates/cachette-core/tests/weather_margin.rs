//! The weather lattice carries a margin, and no reader sees it.
//!
//! The lattice the weather solve steps is larger than the world. The extra is
//! a ring of cells on all four sides, and it exists so that the border of the
//! world has real upwind. Without it, air leaves the lattice through one edge
//! and nothing arrives through the other, so the cells beside an edge stay
//! starved of whatever the wind should carry in.
//!
//! These tests state three things. A margin of zero widens no lattice and
//! shifts no index, so nothing the margin adds reaches a reader. A reader that
//! names a tile reads the world at every margin width. And the margin gives
//! the border of the world more water than it holds without one.
//!
//! **No test here pins a hash against a stored number.** The state of a world
//! is pinned once, in the golden file, which is regenerated deliberately and
//! checked at more than one thread count. A second pin kept beside a subsystem
//! goes stale on every change to that subsystem and teaches its readers to
//! regenerate it without reading it.
//!
//! # References
//!
//! [^1]: ADR-0140, weather is a field over the level 1 cell lattice. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`

use cachette_core::bridge::BlockLayout;
use cachette_core::hex::{Axial, Grid, NEIGHBOUR_COUNT};
use cachette_core::padded::PaddedLattice;
use cachette_core::terrain::Terrain;
use cachette_core::types::Tick;
use cachette_core::types::TileIdx;
use cachette_core::weather::{cell_ground_of, ground_over_lattice, WeatherField, WeatherScale};
use cachette_core::world::{World, WorldConfig};

/// The world the tests run over.
///
/// The extent is small, so a run of many ticks is cheap, and it is large
/// enough at the per-tile pitch to hold a border that is not the whole map.
fn config(width: u32, height: u32) -> WorldConfig {
    WorldConfig {
        width,
        height,
        seed: 0x9e37_79b9_7f4a_7c15,
        faction_count: 2,
        unit_capacity: 64,
    }
}

/// Runs a world for a stated number of frames and returns its state hash.
fn hash_after(world: &mut World, frames: u32) -> u64 {
    for _ in 0..frames {
        world.step(4).expect("the step runs");
    }
    world.state_hash().finish()
}

/// A margin of zero widens no lattice, and nothing it adds reaches a reader.
///
/// The whole lattice is then the lattice of the world, the index of a cell is
/// its own index, the key of a draw is its own index, and the ground fold
/// gives the fold of the world alone.
///
/// **This test pinned a hash of the weather field, and it does not any more.**
/// The pin stated none of the four claims above. It stated that the field had
/// not moved, which is what the golden state hash states, and that file is
/// regenerated deliberately and checked at three thread counts. A second such
/// pin living in a weather suite is one fact in two places, and the copy with
/// no owner is the one that goes stale.[^3] It was regenerated twice in one
/// night, by two people passing through, and each time it recorded the weather
/// of that hour rather than a property of the margin.
///
/// **The four claims survive a change to the weather, and a hash cannot.**
/// None of them reads a temperature, a wind or a rainfall. They read the shape
/// of the lattice and the maps that the ring offsets. A weather change moves a
/// hash and moves none of these.
///
/// The equivalence the pin was written for is asserted elsewhere in this file,
/// against a hand-built bare field rather than against a stored number, so
/// removing the pin takes no coverage away.
///
/// # References
///
/// [^3]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
#[test]
fn a_margin_of_zero_widens_no_lattice_and_changes_no_index() {
    let scale = WeatherScale::LEVEL_1;
    let bare = World::with_weather_margin(config(48, 48), scale, 0).expect("it builds");
    let lattice = bare.weather().lattice();
    assert_eq!(lattice.ring(), 0, "the fixture asked for no ring");
    assert_eq!(
        lattice.whole(),
        lattice.inner(),
        "a margin of zero widens the lattice"
    );

    let count = lattice.inner().tile_count();
    assert!(count > 0, "the lattice of the fixture holds no cell");
    for cell in 0..count {
        assert_eq!(
            lattice.whole_of_inner(cell),
            Some(cell),
            "the whole index of inner cell {cell} is not its own index"
        );
        assert_eq!(
            lattice.inner_of_whole(cell),
            Some(cell),
            "the inner index of whole cell {cell} is not its own index"
        );
        assert_eq!(
            lattice.draw_key(cell),
            cell,
            "the draw key of cell {cell} is not its own index"
        );
    }
    assert_eq!(
        lattice.inner_cells(),
        (0..count).collect::<Vec<u32>>(),
        "the cropped list at no ring is not the lattice in order"
    );

    // The ground fold gives the fold of the world alone. The fold over a
    // lattice takes an early path when the lattice carries no ring, and this
    // is what states that the path gives the same answer as the fold that
    // knows nothing about a ring.
    let layout = bare.weather_layout();
    assert_eq!(
        ground_over_lattice(lattice, layout, bare.terrain()),
        cell_ground_of(layout, bare.terrain()),
        "the fold over a lattice of no ring parts from the fold of the world"
    );
}

/// The margin changes the field, so it is not inert.
///
/// A margin that changed nothing would pass every other test here and do
/// nothing at all.
#[test]
fn the_margin_changes_the_field() {
    for pitch in [0, 5] {
        let scale = WeatherScale::from_bits(pitch).expect("the scale is legal");
        let mut bare = World::with_weather_margin(config(48, 48), scale, 0).expect("it builds");
        let mut padded = World::with_weather_scale(config(48, 48), scale).expect("it builds");
        assert!(
            padded.weather().lattice().ring() > 0,
            "the derived margin is zero at pitch {pitch}"
        );
        assert_ne!(
            hash_after(&mut bare, 24),
            hash_after(&mut padded, 24),
            "the margin changed nothing at pitch {pitch}"
        );
    }
}

/// A bare world solved by hand agrees with a bare world the engine built.
///
/// The field is the whole of the weather, so a hand-built field at margin zero
/// and the field inside a world at margin zero must reach the same planes from
/// the same ground.
#[test]
fn a_bare_field_and_a_bare_world_reach_the_same_planes() {
    let scale = WeatherScale::LEVEL_1;
    let mut world = World::with_weather_margin(config(64, 64), scale, 0).expect("it builds");
    let layout = world.weather_layout();
    let lattice = PaddedLattice::new(world.weather().lattice().inner(), 0).expect("it builds");
    let ground = ground_over_lattice(lattice, layout, world.terrain());
    let mut field = WeatherField::new(lattice, scale, 1).expect("the field builds");
    for tick in 1..=16u64 {
        world.step(1).expect("the step runs");
        field
            .solve(Tick(tick), world.config().seed, &ground, 1)
            .expect("the solve runs");
    }
    assert_eq!(
        field.ground_plane(),
        world.weather().ground_plane(),
        "a hand-built bare field and a bare world part"
    );
}

/// Every reader that names a tile reads the world, at every margin width.
///
/// The margin shifts every cell index. A reader that did not add the offset
/// would read a cell of the margin, and the answer would be silently wrong
/// rather than absent.
#[test]
fn a_tile_reader_reads_the_world_at_every_margin() {
    let scale = WeatherScale::LEVEL_1;
    for margin in 0..4 {
        let world = World::with_weather_margin(config(64, 64), scale, margin).expect("it builds");
        let lattice = world.weather().lattice();
        let grid = world.grid();
        for row in 0..grid.height() {
            for column in 0..grid.width() {
                let address = Axial::new(column as i32, row as i32);
                let tile = grid.index_of(address).expect("the tile is inside");
                let cell = world.weather_cell_of(tile).expect("the tile has a cell");
                // The cell must map back to the cell of the world that the
                // block layout names. An offset that shifted the map by one
                // and stayed inside the lattice would pass a test that only
                // asked whether the cell was inside the world, so the test
                // asks for the exact cell.
                let layout = world.weather_layout();
                let block = layout.block_of_key(layout.key_of(tile).expect("the tile has a key"));
                assert_eq!(
                    lattice.inner_of_whole(cell),
                    Some(block),
                    "the tile ({column}, {row}) reads the wrong cell at margin {margin}"
                );
            }
        }
    }
}

/// The cropped readers are the size of the world, and the whole planes are the
/// size of the lattice.
#[test]
fn a_cropped_reader_is_the_size_of_the_world() {
    let scale = WeatherScale::LEVEL_1;
    let mut world = World::with_weather_scale(config(64, 64), scale).expect("it builds");
    for _ in 0..24 {
        world.step(4).expect("the step runs");
    }
    let field = world.weather();
    let world_cells = field.lattice().inner().tile_count() as usize;
    let whole_cells = field.lattice().whole().tile_count() as usize;
    assert!(whole_cells > world_cells, "the margin widened nothing");
    assert_eq!(field.warmth_over_world().len(), world_cells);
    assert_eq!(field.warmth_plane().len(), whole_cells);
    if !field.air_plane().is_empty() {
        assert_eq!(field.air_over_world().len(), world_cells);
        assert_eq!(field.ground_over_world().len(), world_cells);
        assert_eq!(field.air_plane().len(), whole_cells);
    }
    assert!(
        field.wet_cells() as usize <= world_cells,
        "the wet count reached the margin"
    );
}

/// The water account balances over the whole lattice.
///
/// The margin lifts water and holds water. The account that says a pass moves
/// water and never scales it is an account over everything the field steps, so
/// it must hold with the margin in it.
#[test]
fn the_account_balances_with_a_margin() {
    let mut world = World::with_weather_scale(config(64, 64), WeatherScale::LEVEL_1).expect("ok");
    for _ in 0..32 {
        world.step(4).expect("the step runs");
        let field = world.weather();
        assert_eq!(
            field.air_total().0 + field.ground_total().0 + field.evaporated(),
            field.raised(),
            "the account does not balance"
        );
    }
}

/// Every cell of the world has six neighbours in the lattice.
///
/// **This is the repair.** A cell on the edge of a bare lattice has fewer than
/// six neighbours, so it sends water to no outside and takes water from no
/// outside. It is therefore not a cell of the field: it holds what it lifts
/// and nothing carries anything to it. The margin gives every cell of the
/// world a full set of neighbours, so no cell a reader can see is one of
/// those.
///
/// The test fails at margin zero, which is what makes it a test.
#[test]
fn every_cell_of_the_world_has_six_neighbours() {
    let scale = WeatherScale::LEVEL_1;
    let world = World::with_weather_scale(config(64, 64), scale).expect("it builds");
    let lattice = world.weather().lattice();
    let whole = lattice.whole();
    for inner in 0..lattice.inner().tile_count() {
        let cell = lattice.whole_of_inner(inner).expect("the cell is inside");
        let address = whole
            .address_of(TileIdx(cell))
            .expect("the cell has a place");
        for direction in 0..NEIGHBOUR_COUNT {
            let neighbour = whole.neighbour(address, direction);
            assert!(
                neighbour.is_some(),
                "the cell {inner} of the world has no neighbour {direction}"
            );
        }
    }
    // The same test over a bare lattice must fail, or it proves nothing.
    let bare = World::with_weather_margin(config(64, 64), scale, 0).expect("it builds");
    let bare_lattice = bare.weather().lattice();
    let bare_whole = bare_lattice.whole();
    let corner = bare_whole
        .address_of(TileIdx(0))
        .expect("the corner is a cell");
    assert!(
        (0..NEIGHBOUR_COUNT).any(|direction| bare_whole.neighbour(corner, direction).is_none()),
        "a bare lattice gave its corner six neighbours, so the test cannot fail"
    );
}

/// A margin cell carries the ground of the cell of the world nearest to it.
///
/// **The margin continues the character of the edge rather than inventing
/// ground.** A coast stays a coast and a ridge stays a ridge just outside the
/// frame, so air that enters the world has crossed ground of the right kind.
#[test]
fn a_margin_cell_mirrors_the_cell_of_the_world_nearest_to_it() {
    let scale = WeatherScale::LEVEL_1;
    let grid = Grid::new(64, 64).expect("the extent is valid");
    let terrain = Terrain::new(0x9e37_79b9_7f4a_7c15, grid);
    let layout = BlockLayout::new(grid, scale.bits()).expect("the layout is valid");
    let cells = Grid::new(layout.blocks_wide(), layout.blocks_high()).expect("it builds");
    let lattice = PaddedLattice::new(cells, 3).expect("the padded lattice builds");
    let whole = ground_over_lattice(lattice, layout, terrain);
    let world = cell_ground_of(layout, terrain);
    let mut margin_cells = 0;
    for cell in 0..lattice.whole().tile_count() {
        let nearest = lattice
            .nearest_inner(cell)
            .expect("every cell has a nearest");
        assert_eq!(
            whole[cell as usize], world[nearest as usize],
            "the cell {cell} does not carry the ground of the world cell {nearest}"
        );
        if lattice.inner_of_whole(cell).is_none() {
            margin_cells += 1;
        }
    }
    assert!(margin_cells > 0, "the lattice carries no margin");
    // A margin cell is never empty. An empty cell holds no tile, so it is cold
    // and it lifts nothing, and a ring of those is the starved border the
    // margin exists to remove.
    for cell in 0..lattice.whole().tile_count() {
        assert!(
            whole[cell as usize].tiles() > 0,
            "the cell {cell} carries no tile"
        );
    }
}
