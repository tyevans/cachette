//! A probe that reports what the climate did to the map.
//!
//! The probe builds one world with no spin and one world with a spin, over the
//! same seed and the same extent. It then reports three things.
//!
//! The first is the share of the world that each kind holds, before and after.
//! The second is where the forest sits: the probe sorts the weather cells by
//! how wet the climate says they are, and reports the forest share of the
//! driest cells against the wettest. The third is the same reading against the
//! temperature, so that a reader sees whether cold ground carries less forest.
//!
//! **A climate that shapes nothing gives the same forest share in every band.**
//! That is the reading that says whether the work did anything.
//!
//! The probe drives the world constructor, not the climate module, so it tests
//! that the climate reaches the map rather than that the climate exists.[^1]
//!
//! Run the probe with `cargo run --release -p cachette-core --example
//! climate_map_probe`.
//!
//! # References
//!
//! [^1]: Testing rules, section 5. `.agents/rules/testing.md`

use cachette_core::climate::SPIN_TICKS;
use cachette_core::hex::Axial;
use cachette_core::terrain::{TileKind, KIND_COUNT};
use cachette_core::weather::WeatherScale;
use cachette_core::world::{World, WorldConfig};

/// The seed of the world that the probe builds.
const SEED: u64 = 0x5EED_C11A_7E00_0001;

/// The bands that the probe sorts the weather cells into.
const BANDS: usize = 4;

fn main() {
    let config = WorldConfig {
        width: 256,
        height: 256,
        seed: SEED,
        faction_count: 2,
        unit_capacity: 16,
    };

    let bare = World::new(config).expect("the world builds");
    let shaped = World::with_climate(config, WeatherScale::DEFAULT, SPIN_TICKS, 1)
        .expect("the shaped world builds");

    println!(
        "world {} by {} tiles, spin {SPIN_TICKS} ticks, {} weather cells",
        config.width,
        config.height,
        shaped.climate().cells().len(),
    );

    println!("\n-- the share of the world that each kind holds --");
    let before = kinds_of(&bare);
    let after = kinds_of(&shaped);
    let total: u64 = before.iter().sum();
    println!("kind      before   after   change");
    for kind in TileKind::ALL {
        let at = kind.to_u8() as usize;
        println!(
            "{kind:9?} {:5}.{:1}% {:5}.{:1}%  {:+6}",
            before[at] * 100 / total,
            before[at] * 1000 / total % 10,
            after[at] * 100 / total,
            after[at] * 1000 / total % 10,
            after[at] as i64 - before[at] as i64,
        );
    }

    println!("\n-- forest share by how wet the climate says the cell is --");
    report_bands(&shaped, &bare, Reading::Wetness);
    println!("\n-- forest share by how warm the climate says the cell is --");
    report_bands(&shaped, &bare, Reading::Warmth);
}

/// The climate reading that the probe sorts the cells by.
#[derive(Clone, Copy)]
enum Reading {
    /// The mean standing water of the cell.
    Wetness,
    /// The mean temperature of the cell.
    Warmth,
}

/// Returns the tile count of each kind in a world.
fn kinds_of(world: &World) -> [u64; KIND_COUNT] {
    let mut counts = [0u64; KIND_COUNT];
    for row in 0..world.grid().height() {
        for column in 0..world.grid().width() {
            let address = Axial::new(column as i32, row as i32);
            let Some(kind) = world.tile_kind(address) else {
                continue;
            };
            counts[kind.to_u8() as usize] += 1;
        }
    }
    counts
}

/// Reports the forest share of each band of cells, before and after.
fn report_bands(shaped: &World, bare: &World, reading: Reading) {
    let cells = shaped.climate().cells();
    let mut order: Vec<usize> = (0..cells.len()).collect();
    // The sort key is the reading and then the cell index, so two cells that
    // read the same still take one order.[^1]
    //
    // [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    order.sort_by_key(|at| {
        let cell = cells[*at];
        let key = match reading {
            Reading::Wetness => cell.mean_wetness(),
            Reading::Warmth => cell.mean_warmth(),
        };
        (key, *at)
    });

    // Each tile of the world reports the cell that covers it, so the probe
    // counts the tiles of a cell by walking the world once.
    let mut per_cell_before = vec![[0u64; KIND_COUNT]; cells.len()];
    let mut per_cell_after = vec![[0u64; KIND_COUNT]; cells.len()];
    for row in 0..shaped.grid().height() {
        for column in 0..shaped.grid().width() {
            let address = Axial::new(column as i32, row as i32);
            let Some(cell) = shaped.climate().cell_of(address) else {
                continue;
            };
            let at = cell as usize;
            if let Some(kind) = bare.tile_kind(address) {
                per_cell_before[at][kind.to_u8() as usize] += 1;
            }
            if let Some(kind) = shaped.tile_kind(address) {
                per_cell_after[at][kind.to_u8() as usize] += 1;
            }
        }
    }

    let step = order.len().div_ceil(BANDS).max(1);
    println!("band  reading    cells   forest before   forest after");
    for band in 0..BANDS {
        let low = band * step;
        if low >= order.len() {
            break;
        }
        let high = ((band + 1) * step).min(order.len());
        let mut before = 0u64;
        let mut after = 0u64;
        let mut tiles = 0u64;
        let mut key_total = 0i64;
        for at in &order[low..high] {
            before += per_cell_before[*at][TileKind::Forest.to_u8() as usize];
            after += per_cell_after[*at][TileKind::Forest.to_u8() as usize];
            tiles += per_cell_after[*at].iter().sum::<u64>();
            key_total += match reading {
                Reading::Wetness => cells[*at].mean_wetness(),
                Reading::Warmth => cells[*at].mean_warmth(),
            };
        }
        let tiles = tiles.max(1);
        let span = (high - low) as i64;
        println!(
            "{:4}  {:7}  {:7}   {:9}.{:1}%   {:8}.{:1}%",
            band,
            key_total / span.max(1),
            high - low,
            before * 100 / tiles,
            before * 1000 / tiles % 10,
            after * 100 / tiles,
            after * 1000 / tiles % 10,
        );
    }
}
