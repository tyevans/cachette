//! A probe that reports whether the weather travels across the lattice.
//!
//! This is a diagnostic, not a test. It steps one world and reports, at
//! intervals, the total water in the air, the cell that holds the most, the
//! centroid of the air plane along the column axis, and the largest cell.
//! A weather field that travels moves the argmax and keeps moving it.
//!
//! It also reports the spread of the wet share over the tiles, so that a
//! reader can see whether the map is still separated into wet places and dry
//! places rather than wetted everywhere alike.
//!
//! **The weather resolution is a parameter of the world**, and the probe
//! takes it as its fifth argument. The argument is the base-two logarithm of
//! the cell side in tiles, so zero gives one weather cell to each tile and
//! five gives one to each level 1 block. A finer lattice that stops
//! travelling is worse than a coarse one that moves, so the probe reports the
//! same table at every resolution.
//!
//! Run it with `cargo run -p cachette-core --release --example
//! weather_travel_probe`. The arguments are the extent, the seed, the ticks,
//! the interval between stops, and the weather scale.

use cachette_core::{WeatherScale, World, WorldConfig};

/// A fingerprint of a plane of whole numbers, so that a reader can see at a
/// glance whether the plane changed between two stops.
fn fingerprint(values: impl Iterator<Item = i64>) -> u64 {
    let mut running = 0xcbf2_9ce4_8422_2325u64;
    for value in values {
        running ^= value as u64;
        running = running.wrapping_mul(0x0100_0000_01b3);
    }
    running
}

const WIDTH: u32 = 256;
const HEIGHT: u32 = 256;
const SEED: u64 = 0x2f;
const FACTIONS: u16 = 4;
const TICKS: u32 = 400;
const EVERY: u32 = 20;

/// The weather scale the probe takes when the caller names none.
const DEFAULT_BITS: u32 = 5;

/// Returns one command line argument as a whole number, or a default.
fn argument(position: usize, fallback: u64) -> u64 {
    std::env::args()
        .nth(position)
        .and_then(|text| {
            text.strip_prefix("0x").map_or_else(
                || text.parse::<u64>().ok(),
                |hex| u64::from_str_radix(hex, 16).ok(),
            )
        })
        .unwrap_or(fallback)
}

fn main() {
    // The extent, the seed and the tick count may be named on the command
    // line, so that one probe can read the world a test builds as well as the
    // world the project owner measured.
    let extent = argument(1, u64::from(WIDTH)) as u32;
    let seed = argument(2, SEED);
    // The fifth argument is the weather scale: the base-two logarithm of the
    // cell side in tiles. Zero gives one weather cell to each tile, and five
    // gives one to each level 1 block.
    let bits = argument(5, u64::from(DEFAULT_BITS)) as u32;
    let scale = WeatherScale::from_bits(bits).expect("the scale describes a lattice");
    let mut world = World::with_weather_scale(
        WorldConfig {
            width: extent,
            height: if extent == WIDTH { HEIGHT } else { extent },
            seed,
            faction_count: FACTIONS,
            unit_capacity: 1024,
        },
        scale,
    )
    .expect("the settings describe a world");
    println!(
        "scale {bits} bits, {} tiles a side, {} transport passes",
        scale.side(),
        scale.transport_passes()
    );

    let cells = world.weather().cells();
    let wide = cells.width() as usize;
    let count = cells.tile_count() as usize;

    // How often each cell stood above the wet mark, and the argmax at each
    // stop, so that a reader can see both the travel and the separation.
    let mut wet_ticks = vec![0u32; count];
    let mut stops: Vec<(u32, u32)> = Vec::new();

    let ticks = argument(3, u64::from(TICKS)) as u32;
    let every = argument(4, u64::from(EVERY)) as u32;
    println!("tick   total  ground   wet  argmax cell  arg row  arg col  centroid  max      warmth     wind");
    for tick in 1..=ticks {
        world.step(1).expect("the step runs");
        let field = world.weather();
        for cell in 0..count {
            if field.cell_is_wet(cell as u32) {
                wet_ticks[cell] += 1;
            }
        }
        if tick % every != 0 {
            continue;
        }
        let air = field.air_plane();
        let total: i64 = air.iter().map(|drops| drops.0).sum();
        let mut best = 0usize;
        for cell in 0..air.len() {
            if air[cell].0 > air[best].0 {
                best = cell;
            }
        }
        let max = air.get(best).map_or(0, |drops| drops.0);
        // The centroid along the column axis, in cells, times one hundred.
        let weighted: i64 = air
            .iter()
            .enumerate()
            .map(|(cell, drops)| drops.0 * (cell % wide) as i64)
            .sum();
        let centroid = if total > 0 { weighted * 100 / total } else { 0 };
        stops.push((tick, best as u32));
        // The heat and the wind are what carry the water. A field whose
        // fingerprints hold still cannot move the water it carries.
        let heat = fingerprint(
            world
                .weather()
                .warmth_plane()
                .iter()
                .map(|degrees| i64::from(*degrees)),
        );
        let wind = fingerprint(
            world
                .weather()
                .wind_plane()
                .iter()
                .flat_map(|wind| [i64::from(wind.q), i64::from(wind.r)]),
        );
        let ground: i64 = world.weather().ground_total().0;
        let wet = world.weather().wet_cells();
        println!(
            "{tick:4}  {total:6}  {ground:6}  {wet:4}  {best:11}  {:7}  {:7}  {:5}.{:02}  {max:5}  {:08x} {:08x}",
            best / wide,
            best % wide,
            centroid / 100,
            centroid % 100,
            heat as u32,
            wind as u32,
        );
    }

    // How far the argmax moved between one stop and the next, so that a
    // reader can see whether it keeps moving rather than moving once.
    let mut moves = 0usize;
    for pair in stops.windows(2) {
        if pair[0].1 != pair[1].1 {
            moves += 1;
        }
    }
    println!("argmax moved at {moves} of {} stops", stops.len() - 1);

    let always_wet = wet_ticks.iter().filter(|count| **count == ticks).count();
    let never_wet = wet_ticks.iter().filter(|count| **count == 0).count();
    let mean: i64 = wet_ticks.iter().map(|count| i64::from(*count)).sum::<i64>() * 100
        / (count as i64 * i64::from(ticks));
    let low = wet_ticks.iter().copied().min().unwrap_or(0) * 100 / ticks;
    let high = wet_ticks.iter().copied().max().unwrap_or(0) * 100 / ticks;
    println!("wet share mean {mean} percent, from {low} to {high}");
    println!("cells wet all run {always_wet}, cells dry all run {never_wet}, of {count}");
}
