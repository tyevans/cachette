//! A probe that measures what the weather stage costs at each resolution.
//!
//! This is a diagnostic, not a test. It builds one world at each weather
//! scale, steps it, and reports the cells of the lattice, the bytes the field
//! holds, and the time the weather stage takes for each tick.
//!
//! **The time is measured and the target figure is derived.** The probe
//! reports the time for each cell, and a reader multiplies that by the cell
//! count of a larger world to derive its cost. A derived figure is not a
//! measured one, and the report says which is which.[^1]
//!
//! The stage table is behind a feature, so the probe needs it. Run the probe
//! with `cargo run -p cachette-core --release --features stage-cost --example
//! weather_scale_cost`. The arguments are the extent, the seed, the ticks,
//! and the thread count.
//!
//! # References
//!
//! [^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`

use cachette_core::stage::{self, Stage};
use cachette_core::{WeatherScale, World, WorldConfig};

/// The extent of the demonstration world.
const EXTENT: u32 = 256;

/// The seed of the demonstration world.
const SEED: u64 = 0x2f;

/// The factions the probe builds.
const FACTIONS: u16 = 4;

/// The ticks the probe steps at each scale.
const TICKS: u32 = 200;

/// The ticks the probe steps before it starts measuring.
///
/// **A cold field allocates no plane and runs no transport pass.** A run that
/// measured from the first tick would report the cost of a dry world.
const WARMUP: u32 = 40;

/// The tiles of the target world the project aims at.
const TARGET_TILES: u64 = 16_777_216;

/// Returns one command line argument as a whole number, or a default.
fn argument(position: usize, fallback: u64) -> u64 {
    std::env::args()
        .nth(position)
        .and_then(|text| text.parse::<u64>().ok())
        .unwrap_or(fallback)
}

/// The bytes one cell of the field holds.
///
/// The field holds three planes of water, two planes of wind, two planes of
/// temperature, and one array of ground. The count is derived from the widths
/// the types declare, and the probe asserts it against the types.
const BYTES_FOR_EACH_CELL: u64 = 3 * 8 + 2 * 8 + 2 * 4 + 16;

fn main() {
    let extent = argument(1, u64::from(EXTENT)) as u32;
    let seed = argument(2, SEED);
    let ticks = argument(3, u64::from(TICKS)) as u32;
    let threads = argument(4, 1) as usize;

    let width = core::mem::size_of::<cachette_core::Drops>() as u64 * 3
        + core::mem::size_of::<cachette_core::Wind>() as u64 * 2
        + 4 * 2
        + core::mem::size_of::<cachette_core::CellGround>() as u64;
    assert_eq!(
        width, BYTES_FOR_EACH_CELL,
        "the declared width and the type widths disagree"
    );

    println!(
        "extent {extent}, seed {seed}, {ticks} ticks, {threads} thread(s), \
         {} tiles",
        u64::from(extent) * u64::from(extent)
    );
    println!("all times are measured; every target figure is derived from them");
    println!();
    println!(
        "bits  side   cells  passes    bytes  ns/tick   ns/cell/tick  \
         target cells  target ms/tick"
    );

    for bits in 0..=8u32 {
        let scale = WeatherScale::from_bits(bits).expect("the scale describes a lattice");
        let mut world = World::with_weather_scale(
            WorldConfig {
                width: extent,
                height: extent,
                seed,
                faction_count: FACTIONS,
                unit_capacity: 1024,
            },
            scale,
        )
        .expect("the settings describe a world");

        for _ in 0..WARMUP {
            world.step(threads).expect("the step runs");
        }
        stage::reset();
        for _ in 0..ticks {
            world.step(threads).expect("the step runs");
        }
        let cost = stage::costs().cost(Stage::WeatherSolve);
        let cells = u64::from(world.weather().cells().tile_count());
        let bytes = cells * BYTES_FOR_EACH_CELL;
        let nanos = cost.nanos / u64::from(ticks).max(1);
        // The time for each cell is what a reader carries to another world
        // size. It is a division of two measured numbers and it stays
        // measured. The two columns that follow it are derived from it.
        let for_each_cell = (nanos * 1000).checked_div(cells).unwrap_or(0);
        let target_cells = TARGET_TILES / (u64::from(scale.side()) * u64::from(scale.side()));
        let target_millis = for_each_cell * target_cells / 1_000_000 / 1000;
        println!(
            "{bits:4}  {:4}  {cells:6}  {:6}  {:7}  {nanos:7}  {}.{:03}          \
             {target_cells:12}  {target_millis:14}",
            scale.side(),
            scale.transport_passes(),
            bytes,
            for_each_cell / 1000,
            for_each_cell % 1000,
        );
    }

    println!();
    println!(
        "the field holds {BYTES_FOR_EACH_CELL} bytes for each cell: \
         three water planes, two wind planes, two temperature planes, \
         and the ground array"
    );
}
