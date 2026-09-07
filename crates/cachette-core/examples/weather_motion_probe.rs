//! A probe that measures how much the weather field moves.
//!
//! This is a diagnostic, not a test. A field can grade every climate
//! correctly and still stand still, and a watcher sees the standing still
//! first. The probe reports what drives the motion and what the motion is, so
//! that a field which moves less can be told from a test that asks for too
//! much.
//!
//! Run it with `cargo run -p cachette-core --release --example
//! weather_motion_probe`. The arguments are the extent, the seed, the weather
//! scale, the ticks to settle for and the ticks to watch for.

use cachette_core::{TileIdx, World, WorldConfig};

/// The threads that one step runs on.
const THREADS: usize = 1;

fn argument(position: usize, fallback: u64) -> u64 {
    std::env::args()
        .nth(position)
        .and_then(|text| match text.strip_prefix("0x") {
            Some(rest) => u64::from_str_radix(rest, 16).ok(),
            None => text.parse::<u64>().ok(),
        })
        .unwrap_or(fallback)
}

fn main() {
    let extent = argument(1, 128) as u32;
    let seed = argument(2, 0x2f);
    let bits = argument(3, 0) as u32;
    let settle = argument(4, 400);
    let watch = argument(5, 1000);
    // **The world is built the way the test that guards the motion builds
    // it.** An earlier version of this probe named a weather scale and a
    // smaller faction count, which gave a different lattice and a different
    // answer, and it reported that the field moved when the test said it did
    // not. A probe that does not reproduce the thing it explains explains
    // nothing.
    let _ = bits;
    let mut world = World::new(WorldConfig {
        width: extent,
        height: extent,
        seed,
        faction_count: 4,
        unit_capacity: 1024,
    })
    .expect("the settings describe a world");

    for _ in 0..settle {
        world.step(THREADS).expect("the step must run");
    }

    let mut peaks: Vec<usize> = Vec::new();
    let mut speed_total = 0i64;
    let mut speed_counted = 0i64;
    let mut speed_top = 0i64;
    let mut warmth_step_total = 0i64;
    let mut warmth_step_counted = 0i64;
    for tick in 0..watch {
        world.step(THREADS).expect("the step must run");
        if tick % 20 != 0 {
            continue;
        }
        let field = world.weather();
        let plane = field.air_plane();
        let mut best = 0usize;
        for cell in 0..plane.len() {
            if plane[cell].0 > plane[best].0 {
                best = cell;
            }
        }
        peaks.push(best);
        // The wind is what carries the field. A field that stands still
        // because nothing pushes it is a different fault from a field that
        // stands still while the wind blows.
        for wind in field.wind_plane() {
            let speed = i64::from(wind.speed());
            speed_total += speed;
            speed_counted += 1;
            speed_top = speed_top.max(speed);
        }
        // The pressure gradient is what accelerates the wind, and it reads
        // the temperature difference between neighbours.
        let warmth = field.warmth_plane();
        let cells = field.cells();
        for index in 0..warmth.len() {
            let Some(address) = cells.address_of(TileIdx(index as u32)) else {
                continue;
            };
            let Some(neighbour) = cells.neighbour(address, 0) else {
                continue;
            };
            let Some(at) = cells.index_of(neighbour) else {
                continue;
            };
            let Some(other) = warmth.get(at.0 as usize) else {
                continue;
            };
            warmth_step_total += i64::from((warmth[index] - other).abs());
            warmth_step_counted += 1;
        }
    }
    let mut seen = peaks.clone();
    seen.sort_unstable();
    seen.dedup();
    println!("extent {extent} seed {seed:#x} scale bits {bits} settle {settle} watch {watch}");
    println!(
        "the air peak visited {} cells in {watch} ticks, sampled every 20",
        seen.len()
    );
    println!(
        "the mean wind speed is {} of a ceiling of {}, and the fastest cell reached {}",
        speed_total / speed_counted.max(1),
        cachette_core::weather::SPEED_CEILING,
        speed_top
    );
    println!(
        "the mean temperature step between two neighbours is {} hundredths of a warmth unit",
        warmth_step_total * 100 / warmth_step_counted.max(1)
    );
}
