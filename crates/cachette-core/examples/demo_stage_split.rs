//! Prices every stage of a frame on the demonstration world.
//!
//! Every measurement of a frame so far was taken at the target extent. A cost
//! that does not follow the world would be invisible in all of them, and it
//! would dominate a world this small. This prints the split so the two shapes
//! can be compared.
use cachette_core::stage::{self, STAGES};
use cachette_core::{World, WorldConfig};

fn main() {
    let mut args = std::env::args().skip(1);
    let width: u32 = args.next().and_then(|a| a.parse().ok()).unwrap_or(256);
    let group: u32 = args.next().and_then(|a| a.parse().ok()).unwrap_or(64);
    let threads: usize = args.next().and_then(|a| a.parse().ok()).unwrap_or(4);
    let frames: u64 = 120;

    let mut world = World::new(WorldConfig {
        width,
        height: width,
        seed: 0x0123_4567_89ab_cdef,
        faction_count: 4,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("world");
    let _ = world.found_run_for_every_faction(group);
    // Warm up, so the split is of a running world and not of its first frames.
    for _ in 0..30 {
        world.step(threads).expect("step");
    }

    stage::reset();
    for _ in 0..frames {
        world.step(threads).expect("step");
    }
    let costs = stage::costs();

    let tiles = u64::from(width) * u64::from(width);
    let units = world.soldiers().iter().count();
    let total = costs.total_nanos();
    println!("world {width} by {width} = {tiles} tiles, {units} units, {threads} threads");
    println!("frame total {:.3} ms", total as f64 / 1e6 / frames as f64);
    // The hash is printed so that a sweep over thread counts is also a
    // determinism check. One binary must give one answer at any thread count.
    println!("state hash {}", world.state_hash());
    println!();
    println!(
        "{:<30} {:>11} {:>8} {:>9}",
        "stage", "ms a frame", "share", "entries"
    );
    let mut rows: Vec<_> = STAGES
        .iter()
        .map(|stage| (*stage, costs.cost(*stage)))
        .collect();
    rows.sort_by_key(|(_, cost)| std::cmp::Reverse(cost.nanos));
    for (stage, cost) in rows {
        if cost.nanos == 0 {
            continue;
        }
        let ms = cost.nanos as f64 / 1e6 / frames as f64;
        let share = cost.nanos as f64 * 100.0 / total as f64;
        let nested = if stage.is_nested() { "  nested" } else { "" };
        println!(
            "{:<30} {:>11.4} {:>7.1}% {:>9}{nested}",
            stage.name(),
            ms,
            share,
            cost.entries
        );
    }
}
