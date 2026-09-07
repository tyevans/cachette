//! A probe that measures whether the air field holds clouds or filaments.
//!
//! This is a diagnostic, not a test. It answers three questions. How far
//! apart must two cells be before their air is uncorrelated? How many cells
//! sit exactly at the saturation mark? And how large is the directed share of
//! the transport against the isotropic share?
//!
//! Run it with `cargo run -p cachette-core --release --example
//! weather_cloud_probe`. The arguments are the extent, the seed, the ticks,
//! and the weather scale.
//!
//! Every figure here is an integer. A correlation is reported in thousandths
//! and a share is reported in hundredths, so no step needs a float.

use cachette_core::hex::{NEIGHBOURS, NEIGHBOUR_COUNT};
use cachette_core::weather::{Wind, AIR_SATURATION, CLOUD_SHARE_WHOLE, WIND_FINE};
use cachette_core::{WeatherScale, World, WorldConfig};

fn argument(position: usize, fallback: u64) -> u64 {
    std::env::args()
        .nth(position)
        .and_then(|text| text.parse::<u64>().ok())
        .unwrap_or(fallback)
}

/// Returns the whole part of the square root of a value that is not negative.
fn root(value: i128) -> i128 {
    if value <= 0 {
        return 0;
    }
    let mut guess = value;
    let mut next = (guess + 1) / 2;
    while next < guess {
        guess = next;
        next = (guess + value / guess) / 2;
    }
    guess
}

/// Returns the correlation of a plane against itself at a lag, in thousandths.
///
/// The lag is a step along the column axis of the lattice. A pair counts only
/// when both of its cells lie inside the lattice.
fn correlation_at(plane: &[i64], wide: u32, high: u32, lag: u32, down: bool) -> i64 {
    let mut pairs: Vec<(i64, i64)> = Vec::new();
    let (rows, columns) = if down {
        (high.saturating_sub(lag), wide)
    } else {
        (high, wide.saturating_sub(lag))
    };
    for row in 0..rows {
        for column in 0..columns {
            let here = (row * wide + column) as usize;
            let there = if down {
                ((row + lag) * wide + column) as usize
            } else {
                (row * wide + column + lag) as usize
            };
            let (Some(a), Some(b)) = (plane.get(here), plane.get(there)) else {
                continue;
            };
            pairs.push((*a, *b));
        }
    }
    if pairs.len() < 2 {
        return 0;
    }
    let count = pairs.len() as i128;
    let sum_a: i128 = pairs.iter().map(|pair| i128::from(pair.0)).sum();
    let sum_b: i128 = pairs.iter().map(|pair| i128::from(pair.1)).sum();
    let mean_a = sum_a / count;
    let mean_b = sum_b / count;
    let mut covariance = 0i128;
    let mut spread_a = 0i128;
    let mut spread_b = 0i128;
    for (a, b) in &pairs {
        let da = i128::from(*a) - mean_a;
        let db = i128::from(*b) - mean_b;
        covariance += da * db;
        spread_a += da * da;
        spread_b += db * db;
    }
    let below = root(spread_a) * root(spread_b);
    if below == 0 {
        return 0;
    }
    (covariance * 1000 / below) as i64
}

/// Returns the isotropic and the directed part of what one wind sends, in the
/// numerator unit of the transport.
fn shares(wind: Wind) -> (i64, i64) {
    let mut directed = 0i64;
    for neighbour in NEIGHBOURS {
        directed += i64::from(wind.along(neighbour).max(0));
    }
    (NEIGHBOUR_COUNT as i64 * i64::from(WIND_FINE), directed)
}

fn main() {
    let extent = argument(1, 256) as u32;
    let seed = argument(2, 0x2f);
    let ticks = argument(3, 400) as u32;
    let bits = argument(4, 3) as u32;
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

    let cells = world.weather().cells();
    let (wide, high) = (cells.width(), cells.height());
    println!("extent {extent} seed {seed:#x} ticks {ticks} scale bits {bits}");
    println!(
        "lattice {wide} by {high} cells, saturation mark {} drops, {} transport passes",
        AIR_SATURATION.0,
        scale.transport_passes()
    );

    for tick in 1..=ticks {
        world.step(1).expect("the step must run");
        if tick % (ticks / 2).max(1) != 0 {
            continue;
        }
        let field = world.weather();
        let air: Vec<i64> = field.air_plane().iter().map(|drops| drops.0).collect();
        let mut sorted = air.clone();
        sorted.sort_unstable();
        let count = sorted.len().max(1);
        // **The mark is the capacity of each cell, not one figure the whole
        // plane shares.** Warm air holds a lot of water and cold air holds
        // very little, so a count of drops says nothing about whether a sky
        // is full.
        let sky: Vec<i64> = (0..air.len())
            .map(|cell| field.cloud_share_at(cell as u32))
            .collect();
        let at_mark = sky
            .iter()
            .filter(|value| **value >= CLOUD_SHARE_WHOLE)
            .count();
        let near_mark = sky
            .iter()
            .filter(|value| **value * 10 >= CLOUD_SHARE_WHOLE * 9)
            .count();
        let total: i128 = air.iter().map(|value| i128::from(*value)).sum();
        println!("--- tick {tick}");
        println!(
            "  air  min {:>6}  median {:>6}  mean {:>6}  max {:>6}",
            sorted[0],
            sorted[count / 2],
            (total / count as i128) as i64,
            sorted[count - 1]
        );
        println!("  skies at their own mark {at_mark} of {count} cells, within a tenth of it {near_mark}");
        print!("  air deciles:");
        for step in 1..10 {
            print!(" {}", sorted[count * step / 10]);
        }
        println!();
        // What share of the saturation mark the cells hold, which is what the
        // overlay paints. A watcher sees an overcast sky where this is large
        // over a wide region.
        let overcast = sky
            .iter()
            .filter(|value| **value * 2 >= CLOUD_SHARE_WHOLE)
            .count();
        let thin = sky
            .iter()
            .filter(|value| **value * 20 < CLOUD_SHARE_WHOLE)
            .count();
        println!("  skies above half full {overcast}, below a twentieth {thin}");
        for (name, down) in [("across", false), ("down  ", true)] {
            print!("  air correlation {name} by lag, in thousandths:");
            let mut length = 0u32;
            let mut running = true;
            for lag in 1..=8u32 {
                let value = correlation_at(&air, wide, high, lag, down);
                print!(" {lag}:{value}");
                if running && value >= 500 {
                    length = lag;
                } else {
                    running = false;
                }
            }
            println!("  length {length}");
        }

        let mut isotropic = 0i64;
        let mut directed = 0i64;
        let mut still = 0usize;
        for wind in field.wind_plane() {
            let (base, along) = shares(*wind);
            isotropic += base;
            directed += along;
            if wind.is_still() {
                still += 1;
            }
        }
        println!(
            "  transport share directed {directed} against isotropic {isotropic}, ratio in hundredths {}",
            if isotropic == 0 { 0 } else { directed * 100 / isotropic }
        );
        println!("  still cells {still} of {count}");
    }
}
