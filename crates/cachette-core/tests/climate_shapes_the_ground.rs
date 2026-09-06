//! Tests of the climate field, and of what it does to the ground.
//!
//! The climate is a small field that the engine derives once, by running the
//! weather forward over an empty world for a fixed tick count. The terrain
//! then reads it, so the ground of a world is a pure function of the seed, the
//! address and that field.
//!
//! These tests state four things. The spin repeats, at one thread count and at
//! several. The spin depends on the seed, so a field that ignored an input
//! would fail here rather than in a golden file.[^1] The loop between the
//! ground and the weather is broken, because a ground array folded from a
//! climate-shaped world equals one folded from the base world. The climate
//! reaches the map through the world, and not only through its own module.[^2]
//!
//! # References
//!
//! [^1]: Testing rules, section 2. `.agents/rules/testing.md`
//! [^2]: Testing rules, section 5. `.agents/rules/testing.md`

use cachette_core::bridge::BlockLayout;
use cachette_core::climate::{base_ground_of, CellClimate, Climate, ClimateField, SPIN_TICKS};
use cachette_core::hash::StateHash;
use cachette_core::hex::{Axial, Grid};
use cachette_core::terrain::{self, Terrain, TileKind};
use cachette_core::types::Fix32;
use cachette_core::weather::{CellGround, WeatherScale};
use cachette_core::world::{World, WorldConfig};

/// The extent of the worlds these tests build.
///
/// The extent is four weather cells a side at the default pitch, so the
/// lattice holds enough cells for a wet one and a dry one to stand apart.
const SIDE: u32 = 128;

/// The seed of the worlds these tests build.
const SEED: u64 = 0x0C11_A7E0_0BAD_F00D;

/// A shorter spin, for the tests that do not read a settled field.
///
/// The tests that compare one run against another need only that the two runs
/// agree, so they pay for the smallest spin that exercises every pass.
const SHORT_SPIN: u64 = 96;

/// Returns the world settings that these tests build from.
fn config() -> WorldConfig {
    WorldConfig {
        width: SIDE,
        height: SIDE,
        seed: SEED,
        faction_count: 2,
        unit_capacity: 8,
    }
}

/// Returns the terrain of the test world.
fn terrain() -> Terrain {
    Terrain::new(SEED, Grid::new(SIDE, SIDE).expect("the extent is valid"))
}

/// Returns a spin at a stated thread count.
fn spin(threads: usize, ticks: u64) -> ClimateField {
    ClimateField::spin(terrain(), WeatherScale::DEFAULT, ticks, threads).expect("the spin runs")
}

#[test]
fn the_spin_gives_one_answer_at_any_thread_count() {
    let one = spin(1, SHORT_SPIN);
    let two = spin(2, SHORT_SPIN);
    let twelve = spin(12, SHORT_SPIN);
    assert_eq!(
        one.cells(),
        two.cells(),
        "a spin at two threads gave a different climate from a spin at one"
    );
    assert_eq!(
        one.cells(),
        twelve.cells(),
        "a spin at twelve threads gave a different climate from a spin at one"
    );
    let hash = |field: &ClimateField| field.hash_into(StateHash::new()).finish();
    assert_eq!(hash(&one), hash(&twelve), "the two fields hash apart");
}

#[test]
fn the_spin_repeats() {
    assert_eq!(
        spin(1, SHORT_SPIN).cells(),
        spin(1, SHORT_SPIN).cells(),
        "two spins over one seed gave two climates"
    );
}

#[test]
fn the_seed_reaches_the_climate() {
    // A field that ignored the seed would repeat, would agree at every thread
    // count, and would pass both tests above. Only this one sees it.
    let here = spin(1, SHORT_SPIN);
    let grid = Grid::new(SIDE, SIDE).expect("the extent is valid");
    let elsewhere = ClimateField::spin(
        Terrain::new(SEED ^ 0xFFFF_FFFF, grid),
        WeatherScale::DEFAULT,
        SHORT_SPIN,
        1,
    )
    .expect("the spin runs");
    assert_ne!(
        here.cells(),
        elsewhere.cells(),
        "two seeds gave one climate, so the seed does not reach the field"
    );
}

#[test]
fn the_tick_count_reaches_the_climate() {
    assert_ne!(
        spin(1, SHORT_SPIN).cells(),
        spin(1, SHORT_SPIN + 32).cells(),
        "two spin lengths gave one climate, so the tick count does not reach \
         the field"
    );
}

#[test]
fn the_climate_leaves_the_ground_fold_alone() {
    // **This is the test that says the loop is broken.** The weather reads the
    // mean height of a cell and its open water share, and it reads nothing
    // else from the ground. If a climate moved either of them, a second spin
    // over the shaped world would read a different ground and would give a
    // different climate, and the world would depend on how many times the
    // engine ran the loop.
    let terrain = terrain();
    let layout = BlockLayout::new(terrain.grid(), WeatherScale::DEFAULT.bits())
        .expect("the layout is valid");
    let base = base_ground_of(layout, terrain);
    let field = spin(1, SHORT_SPIN);
    let reference = field.wetness_reference();

    let shaped = shaped_ground_of(layout, terrain, &field, reference);
    assert_eq!(
        base, shaped,
        "the ground under a climate-shaped world differs from the base \
         ground, so the weather would read a different world on a second spin"
    );

    // The fixture must reach the case. A climate that shaped nothing would
    // pass the assertion above and prove nothing, so the test states that the
    // climate did move at least one kind.
    let mut moved = 0;
    for row in 0..terrain.grid().height() {
        for column in 0..terrain.grid().width() {
            let address = Axial::new(column as i32, row as i32);
            let Some(base) = terrain.tile(address) else {
                continue;
            };
            let under = terrain::under(base, field.at_reference(address, reference));
            if under.kind != base.kind {
                moved += 1;
            }
        }
    }
    assert!(
        moved > 0,
        "the climate moved no tile at all, so the fold test measured a \
         climate that does nothing"
    );
}

/// Folds the ground of a world whose terrain the climate shaped.
///
/// **Nothing in the engine calls this.** It exists so that the test above can
/// build the array that the engine must never build, and compare it against
/// the array the engine does build.
fn shaped_ground_of(
    layout: BlockLayout,
    terrain: Terrain,
    field: &ClimateField,
    reference: i64,
) -> Vec<CellGround> {
    let grid = layout.grid();
    let count = (layout.blocks_wide() as usize).saturating_mul(layout.blocks_high() as usize);
    let mut ground = vec![CellGround::EMPTY; count];
    for row in 0..grid.height() {
        for column in 0..grid.width() {
            let address = Axial::new(column as i32, row as i32);
            let Some(base) = terrain.tile(address) else {
                continue;
            };
            let tile = terrain::under(base, field.at_reference(address, reference));
            let Some(key) = grid.index_of(address).and_then(|at| layout.key_of(at)) else {
                continue;
            };
            let Some(slot) = ground.get_mut(layout.block_of_key(key) as usize) else {
                continue;
            };
            *slot = slot.combine(CellGround {
                height_total: cachette_core::sim_math::accumulate(
                    cachette_core::types::Accum(0),
                    tile.height,
                )
                .0,
                tiles: 1,
                open_tiles: i32::from(tile.kind.is_passable()),
            });
        }
    }
    ground
}

#[test]
fn a_quiet_climate_builds_the_world_the_seed_gives() {
    let bare = World::new(config()).expect("the world builds");
    let asked = World::with_climate(config(), WeatherScale::DEFAULT, 0, 1)
        .expect("the world with no spin builds");
    for row in 0..bare.grid().height() {
        for column in 0..bare.grid().width() {
            let address = Axial::new(column as i32, row as i32);
            assert_eq!(
                bare.tile_terrain(address),
                asked.tile_terrain(address),
                "a spin of no ticks changed the ground at {address:?}"
            );
        }
    }
    assert!(
        asked.climate().is_quiet(),
        "a spin of no ticks said something"
    );
}

#[test]
fn the_climate_reaches_the_map_through_the_world() {
    // The test drives the world, not the climate module, because the engine is
    // obligated to invoke the climate and a test of the module alone would
    // pass over an inert field.
    let bare = World::new(config()).expect("the world builds");
    let shaped = World::with_climate(config(), WeatherScale::DEFAULT, SPIN_TICKS, 1)
        .expect("the shaped world builds");

    let mut moved = 0u32;
    let mut height_moved = 0u32;
    for row in 0..bare.grid().height() {
        for column in 0..bare.grid().width() {
            let address = Axial::new(column as i32, row as i32);
            let (Some(before), Some(after)) =
                (bare.tile_terrain(address), shaped.tile_terrain(address))
            else {
                continue;
            };
            if before.kind != after.kind {
                moved += 1;
            }
            if before.height != after.height {
                height_moved += 1;
            }
        }
    }
    assert!(
        moved > 0,
        "the climate reached no tile of the map, so the world does not read it"
    );
    assert_eq!(
        height_moved, 0,
        "the climate moved the height of a tile, which the loop break forbids"
    );
    assert_ne!(
        bare.state_hash(),
        shaped.state_hash(),
        "two worlds with different climates hash the same"
    );
}

#[test]
fn a_wetter_cell_carries_more_forest() {
    // **This is the reading that says the map makes more sense.** The forest
    // of a world with no climate follows a moisture field that correlates with
    // nothing. Under a climate it must follow the water that the weather left.
    let shaped = World::with_climate(config(), WeatherScale::DEFAULT, SPIN_TICKS, 1)
        .expect("the shaped world builds");
    let cells = shaped.climate().cells();
    let reference = shaped.climate().wetness_reference();

    let mut forest_dry = 0u64;
    let mut level_dry = 0u64;
    let mut forest_wet = 0u64;
    let mut level_wet = 0u64;
    for row in 0..shaped.grid().height() {
        for column in 0..shaped.grid().width() {
            let address = Axial::new(column as i32, row as i32);
            let Some(kind) = shaped.tile_kind(address) else {
                continue;
            };
            // Only level ground answers the question. A mountain carries no
            // forest whatever the climate says, so counting mountains would
            // measure the height field and call it a climate.
            if kind != TileKind::Forest && kind != TileKind::Plain {
                continue;
            }
            let Some(cell) = shaped
                .climate()
                .cell_of(address)
                .and_then(|at| cells.get(at as usize))
            else {
                continue;
            };
            let wet = cell.mean_wetness() >= reference;
            let forest = u64::from(kind == TileKind::Forest);
            if wet {
                forest_wet += forest;
                level_wet += 1;
            } else {
                forest_dry += forest;
                level_dry += 1;
            }
        }
    }
    assert!(
        level_wet > 0 && level_dry > 0,
        "the world holds no level ground on one side of the reference, so the \
         fixture supplies no contrast"
    );
    let wet_share = forest_wet * 1000 / level_wet;
    let dry_share = forest_dry * 1000 / level_dry;
    assert!(
        wet_share > dry_share,
        "the wet cells carry {wet_share} forest in a thousand and the dry \
         cells carry {dry_share}, so the forest does not follow the water"
    );
}

#[test]
fn a_dry_climate_takes_moisture_and_a_wet_one_adds_it() {
    let dry = Climate::of(CellClimate::EMPTY.observe(128, 0).observe(128, 0), 100);
    let wet = Climate::of(CellClimate::EMPTY.observe(128, 400).observe(128, 400), 100);
    assert!(dry.is_dry(), "a cell with no water reads as wet");
    assert!(
        !wet.is_dry(),
        "a cell four times the reference reads as dry"
    );
    assert!(
        wet.moisture_offset.0 > 0 && dry.moisture_offset.0 < 0,
        "the offsets do not straddle zero"
    );

    let base = cachette_core::terrain::TerrainTile {
        height: Fix32(45_000),
        moisture: Fix32(34_078),
        kind: TileKind::Hill,
    };
    // The height puts the tile on a hill, and no climate moves it off one.
    assert_eq!(terrain::under(base, wet).height, base.height);
    assert_eq!(terrain::under(base, dry).height, base.height);
    assert!(
        terrain::under(base, wet).moisture > terrain::under(base, dry).moisture,
        "a wet climate left less moisture than a dry one"
    );
}
