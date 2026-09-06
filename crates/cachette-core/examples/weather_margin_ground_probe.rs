//! What ground a margin cell should carry.
//!
//! The weather lattice is larger than the world. The extra is a ring of cells
//! on all four sides that the solve steps and no reader sees. A margin cell
//! stands outside the world, so no tile backs it and it has no ground of its
//! own. Something must choose what ground it carries.
//!
//! The probe runs the same lattice three times over one terrain, and it
//! changes only the ground under the margin.
//!
//! The mirror gives each margin cell a copy of the cell of the world nearest
//! to it. The ocean gives every margin cell open water. The empty margin gives
//! every margin cell no tile at all, which is what a cell outside a bare
//! lattice reads as.
//!
//! The reading is the water in the air over the outermost ring of cells of the
//! world, and the water over the middle quarter of it. It also reports how far
//! the two differ, because a margin that drowns the border is as wrong as one
//! that starves it.
//!
//! The probe runs at the per-tile pitch, so one weather cell covers one tile
//! and an ocean margin cell is exactly one tile of open water.

use cachette_core::bridge::BlockLayout;
use cachette_core::hex::Grid;
use cachette_core::padded::PaddedLattice;
use cachette_core::terrain::Terrain;
use cachette_core::types::Tick;
use cachette_core::weather::{
    cell_ground_of, ground_over_lattice, CellGround, Drops, WeatherField, WeatherScale,
};

/// The frames each run steps before the probe reads it.
const FRAMES: u64 = 128;

/// The world extent the probe runs over.
const EXTENT: u32 = 128;

/// The seed the probe runs on.
const SEED: u64 = 0x9e37_79b9_7f4a_7c15;

fn main() {
    let scale = WeatherScale::PER_TILE;
    let grid = Grid::new(EXTENT, EXTENT).expect("the extent is valid");
    let terrain = Terrain::new(SEED, grid);
    let layout = BlockLayout::new(grid, scale.bits()).expect("the layout is valid");
    let cells = Grid::new(layout.blocks_wide(), layout.blocks_high()).expect("the lattice builds");
    let margin = scale.margin_cells();
    let lattice = PaddedLattice::new(cells, margin).expect("the padded lattice builds");
    let world_ground = cell_ground_of(layout, terrain);

    println!("== {EXTENT} by {EXTENT} tiles, per-tile weather pitch, margin {margin} cells ==");
    println!(
        "the world holds {} cells of water out of {}",
        world_ground
            .iter()
            .filter(|cell| cell.tiles() > 0 && cell.open_tiles() == 0)
            .count(),
        world_ground.len()
    );
    println!(
        "{:>8}  {:>10}  {:>10}  {:>6}",
        "margin", "border air", "middle air", "share"
    );

    let mirror = ground_over_lattice(lattice, layout, terrain);
    report("mirror", &mirror, lattice, scale);

    let sea = sea_cell(&world_ground);
    let ocean = fill_margin(lattice, &world_ground, sea);
    report("ocean", &ocean, lattice, scale);

    let empty = fill_margin(lattice, &world_ground, CellGround::EMPTY);
    report("empty", &empty, lattice, scale);
}

/// Returns the ground of a cell of open water, taken from the world itself.
///
/// The probe takes a real cell rather than an invented one, so the ocean
/// margin carries water of the same depth as the water the world holds.
fn sea_cell(world: &[CellGround]) -> CellGround {
    world
        .iter()
        .copied()
        .find(|cell| cell.tiles() > 0 && cell.open_tiles() == 0)
        .unwrap_or(CellGround::EMPTY)
}

/// Returns a whole-lattice ground that holds the world, and one stated entry
/// under every margin cell.
fn fill_margin(
    lattice: PaddedLattice,
    world: &[CellGround],
    outside: CellGround,
) -> Vec<CellGround> {
    (0..lattice.whole().tile_count())
        .map(|cell| match lattice.inner_of_whole(cell) {
            Some(inner) => world.get(inner as usize).copied().unwrap_or(outside),
            None => outside,
        })
        .collect()
}

/// Runs one field over one ground and reports what its border holds.
fn report(label: &str, ground: &[CellGround], lattice: PaddedLattice, scale: WeatherScale) {
    let mut field = WeatherField::new(lattice, scale, 1).expect("the field builds");
    for tick in 1..=FRAMES {
        field
            .solve(Tick(tick), SEED, ground, 4)
            .expect("the solve runs");
    }
    let inner = lattice.inner();
    let plane: Vec<Drops> = lattice
        .inner_cells()
        .into_iter()
        .map(|cell| {
            field
                .air_plane()
                .get(cell as usize)
                .copied()
                .unwrap_or(Drops::ZERO)
        })
        .collect();
    let (mut border, mut border_count) = (0i64, 0i64);
    let (mut middle, mut middle_count) = (0i64, 0i64);
    let quarter_across = inner.width() / 4;
    let quarter_down = inner.height() / 4;
    for row in 0..inner.height() {
        for column in 0..inner.width() {
            let held = plane[(row * inner.width() + column) as usize].0;
            if column == 0 || row == 0 || column + 1 == inner.width() || row + 1 == inner.height() {
                border += held;
                border_count += 1;
            }
            if column >= quarter_across
                && row >= quarter_down
                && column < inner.width() - quarter_across
                && row < inner.height() - quarter_down
            {
                middle += held;
                middle_count += 1;
            }
        }
    }
    let border = if border_count == 0 {
        0
    } else {
        border / border_count
    };
    let middle = if middle_count == 0 {
        0
    } else {
        middle / middle_count
    };
    let share = if middle == 0 {
        0
    } else {
        border * 100 / middle
    };
    println!("{label:>8}  {border:>10}  {middle:>10}  {share:>5}%");
}
