//! A throwaway probe that asks whether a storm shows in the published cover.
//!
//! This is a diagnostic and not a test. A previous report said that raising a
//! storm changed the published cloud cover not at all against an identical
//! control world. This probe answers that claim in numbers, and it separates
//! three explanations of it.
//!
//! It builds two worlds from one set of settings, settles both, and raises one
//! storm on the first. It then steps both and reports, for each tick, the
//! footprint of the storm, the cover over that footprint in each world, the
//! air and the ground under it, and the wind speed over it. A storm that rains
//! its own sky out shows as a falling cover and a rising ground. A storm that
//! never reaches the field shows as no difference anywhere.
//!
//! It also reports how many cyclones the control world raised on its own, so
//! that a reader can tell an imposed storm from a formed one.
//!
//! Run it with `cargo run -p cachette-core --release --example
//! storm_cover_probe`. The arguments are the extent, the seed, the weather
//! scale bits, the ticks to settle for, and the ticks to run for.
//!
//! Every figure is an integer.

use cachette_core::weather::ground_over_lattice;
use cachette_core::{Axial, CycloneSetting, TileIdx, WeatherScale, World, WorldConfig};

/// The threads that one step runs on.
const THREADS: usize = 1;

/// Reads one command line argument, in decimal or in hexadecimal.
fn argument(position: usize, fallback: u64) -> u64 {
    std::env::args()
        .nth(position)
        .and_then(|text| match text.strip_prefix("0x") {
            Some(rest) => u64::from_str_radix(rest, 16).ok(),
            None => text.parse::<u64>().ok(),
        })
        .unwrap_or(fallback)
}

/// Builds one world from the settings the probe runs at.
fn build(extent: u32, seed: u64, scale: WeatherScale) -> World {
    World::with_weather_scale(
        WorldConfig {
            width: extent,
            height: extent,
            seed,
            faction_count: 4,
            unit_capacity: 1024,
            ..WorldConfig::DEFAULT
        },
        scale,
    )
    .expect("the settings describe a world")
}

/// The mean of a set of readings, or zero when the set is empty.
fn mean(total: i64, count: i64) -> i64 {
    if count <= 0 {
        0
    } else {
        total / count
    }
}

/// What the probe reads over one set of cells at one tick.
struct Reading {
    cover: i64,
    air: i64,
    ground: i64,
    wind: i64,
}

/// Reads the cover, the air, the ground and the wind over a set of cells.
fn read(world: &World, cells: &[u32]) -> Reading {
    let field = world.weather();
    let count = cells.len() as i64;
    let mut cover = 0i64;
    let mut air = 0i64;
    let mut ground = 0i64;
    let mut wind = 0i64;
    for cell in cells.iter().copied() {
        cover += field.cloud_share_at(cell);
        air += field.air_at(cell).0;
        ground += field.ground_at(cell).0;
        wind += i64::from(field.wind_at(cell).speed());
    }
    Reading {
        cover: mean(cover, count),
        air: mean(air, count),
        ground: mean(ground, count),
        wind: mean(wind, count),
    }
}

/// Counts the tiles whose published cover differs between two worlds.
///
/// The published cover is the number that the tile reader answers, so this
/// counts exactly what a watcher at the Python boundary would see move.
fn cover_gap(stormed: &World, control: &World) -> (u32, i64) {
    let grid = stormed.grid();
    let mut moved = 0u32;
    let mut widest = 0i64;
    for index in 0..grid.tile_count() {
        let Some(address) = grid.address_of(TileIdx(index)) else {
            continue;
        };
        let one = stormed.cloud_share_at(address).unwrap_or(0);
        let other = control.cloud_share_at(address).unwrap_or(0);
        if one != other {
            moved += 1;
            widest = widest.max((one - other).abs());
        }
    }
    (moved, widest)
}

fn main() {
    let extent = argument(1, 96) as u32;
    let seed = argument(2, 0x2f);
    let bits = argument(3, 0) as u32;
    let settle = argument(4, 40);
    let run = argument(5, 10);
    let scale = WeatherScale::from_bits(bits).expect("the scale describes a lattice");

    let mut stormed = build(extent, seed, scale);
    let mut control = build(extent, seed, scale);
    println!("extent {extent} seed {seed:#x} bits {bits} settle {settle} run {run}");

    for _ in 0..settle {
        stormed.step(THREADS).expect("the step must run");
        control.step(THREADS).expect("the step must run");
    }
    println!(
        "after {settle} settling ticks the control raised {} storms on its own",
        control.weather().cyclones_raised()
    );

    let lattice = stormed.weather_lattice();
    let layout = stormed.weather_layout();
    let under = ground_over_lattice(lattice, layout, stormed.terrain());
    let warmth = stormed.weather().warmth_plane().to_vec();
    let mut best = None;
    let mut best_heat = i32::MIN;
    for (cell, ground) in under.iter().copied().enumerate() {
        if ground.tiles() <= 0 || ground.open_tiles() * 2 > ground.tiles() {
            continue;
        }
        let heat = warmth.get(cell).copied().unwrap_or(0);
        if heat > best_heat {
            best_heat = heat;
            best = Some(cell as u32);
        }
    }
    let cell = best.expect("the world holds a sea cell");
    let inner = lattice
        .inner_address_of(cell)
        .expect("the cell covers the world");
    let place = Axial::new(
        inner.q * layout.block_edge() as i32,
        inner.r * layout.block_edge() as i32,
    );
    let raised = stormed
        .raise_cyclone(place, CycloneSetting::TROPICAL)
        .expect("the verb must place a storm");
    println!(
        "raised storm {} at tile ({}, {}) depth {} radius {} life {}",
        raised.id, place.q, place.r, raised.depth, raised.radius, raised.life
    );

    // The reading before the first step says what the verb alone changed. The
    // verb writes a storm into the list and the solve writes the deficit
    // plane, so the deficit is expected to be flat here.
    let footprint_now: Vec<u32> = lattice
        .inner_cells()
        .into_iter()
        .filter(|cell| stormed.weather().depression_at(*cell) > 0)
        .collect();
    println!(
        "before the first step the deficit covers {} cells of the world",
        footprint_now.len()
    );

    println!(
        "tick depth footprint coverA coverB airA airB groundA groundB windA windB \
         tilesMoved widest"
    );
    for tick in 1..=run {
        stormed.step(THREADS).expect("the step must run");
        control.step(THREADS).expect("the step must run");
        let depth = stormed
            .weather()
            .cyclones()
            .iter()
            .find(|one| one.id == raised.id)
            .map_or(0, |one| one.depth);
        let footprint: Vec<u32> = lattice
            .inner_cells()
            .into_iter()
            .filter(|cell| stormed.weather().depression_at(*cell) > 0)
            .collect();
        let here = read(&stormed, &footprint);
        let there = read(&control, &footprint);
        let (moved, widest) = cover_gap(&stormed, &control);
        println!(
            "world air {} against {}, world ground {} against {}",
            stormed.weather().air_total().0,
            control.weather().air_total().0,
            stormed.weather().ground_total().0,
            control.weather().ground_total().0
        );
        println!(
            "{tick} {depth} {} {} {} {} {} {} {} {} {} {moved} {widest}",
            footprint.len(),
            here.cover,
            there.cover,
            here.air,
            there.air,
            here.ground,
            there.ground,
            here.wind,
            there.wind
        );
    }

    println!(
        "after {} ticks the control raised {} storms on its own",
        settle + run,
        control.weather().cyclones_raised()
    );
}
