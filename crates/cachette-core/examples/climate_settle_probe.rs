//! A probe that reports when the climate stops moving, and what it costs.
//!
//! The probe runs the weather forward over an empty world and reads the
//! temperature and the standing water of every weather cell at each tick. It
//! reports, for each tick, how far the running mean of each cell moved since
//! the tick before. When that movement falls and stays low, the climate has
//! settled and a spin may stop there.
//!
//! The probe reads no clock. A time budget makes a result depend on the load
//! of the machine, and the record bans the clock for that reason.[^1] A reader
//! who wants the wall time runs the probe under a timing command.
//!
//! Run the probe with `cargo run --release -p cachette-core --example
//! climate_settle_probe`.
//!
//! # References
//!
//! [^1]: ADR-0005, a solver runs a fixed iteration count, decision D1. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`

use cachette_core::bridge::BlockLayout;
use cachette_core::climate::{CellClimate, Climate, WARM_UP_TICKS};
use cachette_core::hex::Grid;
use cachette_core::padded::PaddedLattice;
use cachette_core::terrain::Terrain;
use cachette_core::types::Tick;
use cachette_core::weather::{ground_over_lattice, WeatherField, WeatherScale};

/// The ticks that the probe runs.
const TICKS: u64 = 512;

/// The seed of the world that the probe builds.
const SEED: u64 = 0x5EED_C11A_7E00_0001;

fn main() {
    for (width, height, label) in [(256u32, 256u32, "demonstration"), (1024, 1024, "large")] {
        run(width, height, label);
    }
}

/// Returns the mean standing water over every counted cell.
fn mean_wetness_of(cells: &[CellClimate]) -> i64 {
    let mut total = 0i64;
    let mut counted = 0i64;
    for cell in cells {
        if cell.counted <= 0 {
            continue;
        }
        total += cell.mean_wetness();
        counted += 1;
    }
    if counted <= 0 {
        return 0;
    }
    total / counted
}

/// Runs one world and reports what it measured.
fn run(width: u32, height: u32, label: &str) {
    let grid = Grid::new(width, height).expect("the extent is valid");
    let terrain = Terrain::new(SEED, grid);
    let scale = WeatherScale::DEFAULT;
    let layout = BlockLayout::new(grid, scale.bits()).expect("the layout is valid");
    let cell_lattice =
        Grid::new(layout.blocks_wide(), layout.blocks_high()).expect("the lattice is valid");
    // The probe runs over the padded lattice that the world runs over, so
    // that what it measures is what the world settles to.
    let lattice = PaddedLattice::new(cell_lattice, scale.margin_cells())
        .expect("the padded lattice is valid");

    let ground = ground_over_lattice(lattice, layout, terrain);
    let world_cells = lattice.inner_cells();

    let mut weather = WeatherField::new(lattice, scale, 1).expect("the weather builds");
    let mut cells = vec![CellClimate::EMPTY; world_cells.len()];
    let mut last_warmth = vec![0i64; world_cells.len()];
    let mut last_wetness = vec![0i64; world_cells.len()];
    let mut last_offsets: Vec<i32> = Vec::new();

    println!(
        "== {label}: {width} by {height} tiles, {} weather cells at pitch {} ==",
        world_cells.len(),
        scale.side()
    );
    println!(
        "tick  mean warmth  mean wetness  warmth move  wetness move  wet cells   offset move  flipped"
    );

    for tick in 1..=TICKS {
        weather
            .solve(Tick(tick), SEED, &ground, 1)
            .expect("the solve runs");
        if tick <= WARM_UP_TICKS {
            continue;
        }
        let mut warmth_move = 0i64;
        let mut wetness_move = 0i64;
        let mut warmth_sum = 0i64;
        let mut wetness_sum = 0i64;
        let mut wet = 0i64;
        for (cell, slot) in cells.iter_mut().enumerate() {
            let index = world_cells[cell];
            *slot = slot.observe(
                i64::from(weather.warmth_at(index)),
                weather.ground_at(index).0,
            );
            let warmth = slot.mean_warmth();
            let wetness = slot.mean_wetness();
            warmth_move += (warmth - last_warmth[cell]).abs();
            wetness_move += (wetness - last_wetness[cell]).abs();
            last_warmth[cell] = warmth;
            last_wetness[cell] = wetness;
            warmth_sum += warmth;
            wetness_sum += wetness;
            if wetness > 0 {
                wet += 1;
            }
        }
        let count = cells.len() as i64;
        if tick % 16 == 0 {
            // **The classification reads the offset, not the mean.** The mean
            // wetness of a world climbs for as long as the run lasts, because
            // the sea keeps lifting water into a lattice that dries slowly.
            // That climb moves every cell together and changes no kind. What
            // changes a kind is the offset of a cell from the world around
            // it, so the settle criterion is the offset and not the mean.
            let reference = mean_wetness_of(&cells);
            let mut offsets = Vec::with_capacity(cells.len());
            for cell in &cells {
                offsets.push(Climate::of(*cell, reference).moisture_offset.0);
            }
            let mut offset_move = 0i64;
            let mut flipped = 0i64;
            if last_offsets.len() == offsets.len() {
                for (at, offset) in offsets.iter().enumerate() {
                    offset_move += i64::from((offset - last_offsets[at]).abs());
                    if (*offset < 0) != (last_offsets[at] < 0) {
                        flipped += 1;
                    }
                }
            }
            last_offsets = offsets;
            println!(
                "{tick:4}  {:11}  {:12}  {:11}  {:12}  {wet:9}  {:12}  {flipped:7}",
                warmth_sum / count,
                wetness_sum / count,
                warmth_move,
                wetness_move,
                offset_move / count,
            );
        }
    }
    let spread: i64 = cells.iter().map(|cell| cell.season_spread()).sum();
    println!(
        "mean season spread {}\n",
        spread / cells.len().max(1) as i64
    );
}
