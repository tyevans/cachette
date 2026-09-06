//! A probe that measures the weather as a function of distance from water.
//!
//! This is a diagnostic, not a test. It answers three questions. How far
//! inland does the air travel before it is gone? Where does the rain land?
//! And does the land hold a different temperature from the sea beside it?
//!
//! Run it with `cargo run -p cachette-core --release --example
//! weather_coast_probe`. The arguments are the extent, the seed, the ticks,
//! and the weather scale.

use cachette_core::hex::Axial;
use cachette_core::{TileIdx, WeatherScale, World, WorldConfig};

fn argument(position: usize, fallback: u64) -> u64 {
    std::env::args()
        .nth(position)
        .and_then(|text| text.parse::<u64>().ok())
        .unwrap_or(fallback)
}

/// Returns, for each weather cell, the count of water tiles it covers and the
/// sum of the heights of those water tiles.
///
/// The height of a water tile is its depth read the other way round: the
/// shoreline mark is the top of the range and zero is the deepest water.
fn water_of(world: &World, extent: u32, bits: u32, cells: usize) -> (Vec<i64>, Vec<i64>) {
    let mut count = vec![0i64; cells];
    let mut height = vec![0i64; cells];
    let grid = world.weather().cells();
    for row in 0..extent {
        for column in 0..extent {
            let address = Axial::new(column as i32, row as i32);
            let Some(tile) = world.tile_terrain(address) else {
                continue;
            };
            if tile.kind.is_passable() {
                continue;
            }
            let cell = Axial::new((column >> bits) as i32, (row >> bits) as i32);
            let Some(at) = grid.index_of(cell) else {
                continue;
            };
            count[at.0 as usize] += 1;
            height[at.0 as usize] += i64::from(tile.height.0);
        }
    }
    (count, height)
}

/// Returns the lattice distance from each cell to the nearest cell that holds
/// any open water. A cell that holds water is at distance zero.
fn distance_from_water(world: &World, water: &[i64]) -> Vec<i32> {
    let cells = world.weather().cells();
    let mut out = vec![-1i32; water.len()];
    let mut front: Vec<u32> = Vec::new();
    for (index, count) in water.iter().enumerate() {
        if *count > 0 {
            out[index] = 0;
            front.push(index as u32);
        }
    }
    let mut step = 0i32;
    while !front.is_empty() {
        step += 1;
        let mut next = Vec::new();
        for index in front {
            let Some(address) = cells.address_of(TileIdx(index)) else {
                continue;
            };
            for direction in 0..cachette_core::hex::NEIGHBOUR_COUNT {
                let Some(beside) = cells.neighbour(address, direction) else {
                    continue;
                };
                let Some(at) = cells.index_of(beside) else {
                    continue;
                };
                if out[at.0 as usize] >= 0 {
                    continue;
                }
                out[at.0 as usize] = step;
                next.push(at.0);
            }
        }
        front = next;
    }
    out
}

fn mean(values: &[i64]) -> i64 {
    if values.is_empty() {
        0
    } else {
        values.iter().sum::<i64>() / values.len() as i64
    }
}

fn median(values: &mut Vec<i64>) -> i64 {
    if values.is_empty() {
        return 0;
    }
    values.sort_unstable();
    values[values.len() / 2]
}

fn main() {
    let extent = argument(1, 256) as u32;
    let seed = argument(2, 0x2f);
    let ticks = argument(3, 400) as u32;
    let bits = argument(4, 5) as u32;
    let scale = WeatherScale::from_bits(bits).expect("the scale describes a lattice");
    let mut world = World::with_weather_scale(
        WorldConfig {
            width: extent,
            height: extent,
            seed,
            faction_count: 4,
            unit_capacity: 1024,
        },
        scale,
    )
    .expect("the settings describe a world");

    let count = world.weather().air_plane().len().max(
        world.weather().warmth_plane().len(),
    );
    let (water, water_height) = water_of(&world, extent, bits, count);
    let distance = distance_from_water(&world, &water);
    let deepest = distance.iter().copied().max().unwrap_or(0);
    let wet_cells = water.iter().filter(|count| **count > 0).count();
    println!("extent {extent} seed {seed:#x} ticks {ticks} scale bits {bits}");
    println!(
        "lattice {} cells, {wet_cells} hold water, deepest inland cell is {deepest} cells from any",
        count
    );

    for tick in 1..=ticks {
        world.step(1).expect("the step must run");
        if tick % (ticks / 2).max(1) != 0 {
            continue;
        }
        let field = world.weather();
        let air = field.air_plane().to_vec();
        let ground = field.ground_plane().to_vec();
        let warmth = field.warmth_plane().to_vec();
        let wind = field.wind_plane().to_vec();
        println!("--- tick {tick}");
        println!("  band   cells    air mean  air med  ground mean  warmth mean  wind mean");
        for band in 0..=deepest.min(24) {
            let slots: Vec<usize> = distance
                .iter()
                .enumerate()
                .filter(|(_, at)| **at == band)
                .map(|(index, _)| index)
                .collect();
            if slots.is_empty() {
                continue;
            }
            let mut airs: Vec<i64> = slots
                .iter()
                .map(|slot| air.get(*slot).map_or(0, |drops| drops.0))
                .collect();
            let grounds: Vec<i64> = slots
                .iter()
                .map(|slot| ground.get(*slot).map_or(0, |drops| drops.0))
                .collect();
            let warmths: Vec<i64> = slots
                .iter()
                .map(|slot| i64::from(warmth.get(*slot).copied().unwrap_or(0)))
                .collect();
            let winds: Vec<i64> = slots
                .iter()
                .map(|slot| {
                    i64::from(wind.get(*slot).copied().unwrap_or_default().speed())
                })
                .collect();
            println!(
                "  {band:>4} {:>7} {:>11} {:>8} {:>12} {:>12} {:>10}",
                slots.len(),
                mean(&airs),
                median(&mut airs),
                mean(&grounds),
                mean(&warmths),
                mean(&winds),
            );
        }
    }
}
