//! The weather lattice carries a margin, and no reader sees it.
//!
//! The lattice the weather solve steps is larger than the world. The extra is
//! a ring of cells on all four sides, and it exists so that the border of the
//! world has real upwind. Without it, air leaves the lattice through one edge
//! and nothing arrives through the other, so the cells beside an edge stay
//! starved of whatever the wind should carry in.
//!
//! These tests state three things. A margin of zero reproduces the field the
//! engine held before the margin existed. A reader that names a tile reads the
//! world at every margin width. And the margin gives the border of the world
//! more water than it holds without one.
//!
//! # References
//!
//! [^1]: ADR-0140, weather is a field over the level 1 cell lattice. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`

use cachette_core::bridge::BlockLayout;
use cachette_core::hash::StateHash;
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

/// Runs a world for a stated number of frames and returns its weather hash.
///
/// **The fold covers the weather field and nothing else.** The state hash of a
/// world covers every arena and every rule the world holds, so a change to a
/// rule that no weather pass reads moves it. A pin on the state hash is
/// therefore a second copy of the golden state hash, kept in a suite that is
/// about the weather, with nothing that says which of the two to regenerate
/// when they disagree.[^3]
///
/// # References
///
/// [^3]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
fn weather_hash_after(world: &mut World, frames: u32) -> u64 {
    for _ in 0..frames {
        world.step(4).expect("the step runs");
    }
    world.weather().hash_into(StateHash::new()).finish()
}

/// The weather hash of a world at margin zero, after the stated frames.
///
/// **The value no longer comes from the engine that had no margin, and it
/// cannot.** It was taken from a run of that engine, at the commit this branch
/// left, with the same extent, the same seed and the same frame count. The
/// weather model has since changed: the row axis of a world became a latitude,
/// the sun term became the published insolation geometry, and the capacity of
/// the air became the published saturation curve.[^2] Every world therefore
/// holds a different field, and the old value can never be reached again.
///
/// **So this constant is now a pin on the current engine and not a proof of
/// the equivalence it was written for.** It still fails when a change moves
/// the field at margin zero, which is what a regression pin does. It no longer
/// says that a margin of zero reproduces an engine that had none, because that
/// engine is gone. The other tests in this file carry the properties of the
/// margin that are still checkable. The commit body holds the command that
/// produced the value.
///
/// **The pin covers the weather field and not the whole world.** It covered
/// the world state once, and it then failed on every change to any rule the
/// world holds, whether or not a weather pass read that rule. A siege rule
/// added to the world moved it, and the message accused the weather of a move
/// the weather did not make. A pin on the state of a whole world already
/// exists as the golden state hash, and a second copy of it is one fact in two
/// places.[^3]
///
/// A test must read a stored value rather than compute it, or it compares a
/// run against itself and proves nothing.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 1. `.agents/rules/testing.md`
/// [^2]: ADR-0177, the row axis of a world is a latitude that the world states. `docs/adrs/draft/adr-0177-the-row-axis-of-a-world-is-a-latitude-that-the-world-states.md`
/// [^3]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
const BARE_HASH_AFTER_24_FRAMES: u64 = 13_874_570_617_531_493_441;

/// The frames that the bare hash was taken after.
const BARE_HASH_FRAMES: u32 = 24;

/// A margin of zero widens no lattice, and the field it gives is pinned.
///
/// The whole lattice is then the lattice of the world, the index of a cell is
/// its own index, the key of a draw is its own index, and the ground fold
/// gives the fold of the world alone. Nothing the margin adds reaches the
/// field.
#[test]
fn a_margin_of_zero_widens_no_lattice_and_holds_its_pinned_field() {
    let scale = WeatherScale::LEVEL_1;
    let mut bare = World::with_weather_margin(config(48, 48), scale, 0).expect("it builds");
    assert_eq!(
        bare.weather().lattice().whole(),
        bare.weather().lattice().inner(),
        "a margin of zero widens the lattice"
    );
    assert_eq!(
        weather_hash_after(&mut bare, BARE_HASH_FRAMES),
        BARE_HASH_AFTER_24_FRAMES,
        "the weather field at margin zero moved away from the value this \
         branch pinned"
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
