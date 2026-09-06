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
//! Run it with `cargo run -p cachette-core --release --example
//! weather_travel_probe`.

use cachette_core::weather::heat_of;
use cachette_core::{World, WorldConfig};

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

fn main() {
    let mut world = World::new(WorldConfig {
        width: WIDTH,
        height: HEIGHT,
        seed: SEED,
        faction_count: FACTIONS,
        unit_capacity: 1024,
    })
    .expect("the settings describe a world");

    let cells = world.weather().cells();
    let wide = cells.width() as usize;
    let count = cells.tile_count() as usize;

    // How often each cell stood above the wet mark, and the argmax at each
    // stop, so that a reader can see both the travel and the separation.
    let mut wet_ticks = vec![0u32; count];
    let mut stops: Vec<(u32, u32)> = Vec::new();

    println!("tick   total  argmax cell  arg row  arg col  centroid  max     heat     wind");
    for tick in 1..=TICKS {
        world.step(1).expect("the step runs");
        let field = world.weather();
        for cell in 0..count {
            if field.cell_is_wet(cell as u32) {
                wet_ticks[cell] += 1;
            }
        }
        if tick % EVERY != 0 {
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
                .pyramid()
                .cells()
                .iter()
                .map(|summary| i64::from(heat_of(*summary))),
        );
        let wind = fingerprint(
            world
                .weather()
                .wind_plane()
                .iter()
                .flat_map(|wind| [i64::from(wind.q), i64::from(wind.r)]),
        );
        println!(
            "{tick:4}  {total:6}  {best:11}  {:7}  {:7}  {:5}.{:02}  {max:5}  {:08x} {:08x}",
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

    let always_wet = wet_ticks.iter().filter(|count| **count == TICKS).count();
    let never_wet = wet_ticks.iter().filter(|count| **count == 0).count();
    let mean: i64 = wet_ticks.iter().map(|count| i64::from(*count)).sum::<i64>() * 100
        / (count as i64 * i64::from(TICKS));
    let low = wet_ticks.iter().copied().min().unwrap_or(0) * 100 / TICKS;
    let high = wet_ticks.iter().copied().max().unwrap_or(0) * 100 / TICKS;
    println!("wet share mean {mean} percent, from {low} to {high}");
    println!("cells wet all run {always_wet}, cells dry all run {never_wet}, of {count}");
}
