//! A probe that reports the moisture the weather field holds over a run.
//!
//! This is a diagnostic, not a test. It exists so that a recovery curve is
//! fitted to what the weather does rather than to what the author guessed.

use cachette_core::{World, WorldConfig};

fn main() {
    let seed = std::env::args()
        .nth(1)
        .and_then(|text| u64::from_str_radix(text.trim_start_matches("0x"), 16).ok())
        .unwrap_or(0x0cac_4e77_5104_0001);
    let mut world = World::new(WorldConfig {
        width: 192,
        height: 192,
        seed,
        faction_count: 4,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the settings describe a world");
    world.found_run_for_every_faction(30);
    let mut histogram = [0u64; 20];
    let mut samples = 0u64;
    for tick in 0..=2000u32 {
        if tick % 50 == 0 {
            let plane = world.weather().ground_plane();
            let mut sorted: Vec<i64> = plane.iter().map(|drops| drops.0).collect();
            sorted.sort_unstable();
            for drops in &sorted {
                let band = (*drops / 32).clamp(0, 19) as usize;
                histogram[band] += 1;
                samples += 1;
            }
            let count = sorted.len();
            if count > 0 {
                let total: i64 = sorted.iter().sum();
                println!(
                    "tick {tick} cells {count} min {} p25 {} median {} p75 {} p95 {} max {} mean {}",
                    sorted[0],
                    sorted[count / 4],
                    sorted[count / 2],
                    sorted[count * 3 / 4],
                    sorted[count * 95 / 100],
                    sorted[count - 1],
                    total / count as i64,
                );
            }
        }
        world.step(4).expect("the world steps");
    }
    println!("--- ground water histogram, 32 drops for each band, {samples} samples ---");
    for (band, count) in histogram.iter().enumerate() {
        let share = (*count * 1000) / samples.max(1);
        println!(
            "band {:>3}..{:<4} {:>8}  {}.{}%",
            band * 32,
            band * 32 + 31,
            count,
            share / 10,
            share % 10
        );
    }
}
