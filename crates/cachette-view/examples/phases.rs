//! Where a step spends its time.
//!
//! The viewer's report says the whole step. This says which part, by running
//! worlds that differ in one thing at a time and comparing. It is the crude
//! form of a profile, and it is enough to tell a stub from a system.
//!
//! It lives in the viewer crate because the viewer is allowed to read a
//! clock. The engine is not: a solver that stops on a time budget answers
//! differently on a loaded machine.[^1]
//!
//! Run it with `cargo run --release -p cachette-view --example phases`.
//!
//! Every figure it prints is a development machine, one run. That is a
//! smaller question than the blocker asks.[^2]
//!
//! # References
//!
//! [^1]: ADR-0005, a solver runs a fixed iteration count, never a convergence test, decision D2. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
//! [^2]: Blockers register, BLK-007. `docs/BLOCKERS.md`

// An example is its own crate, so the allowance at the viewer's crate root
// does not reach it. The reason is the same one: ADR-0067 D3 puts the float
// boundary at the viewer, and a mean of measured microseconds is a report,
// never a simulated value.
#![allow(clippy::disallowed_types)]

use cachette_core::{Axial, FactionId, World, WorldConfig};
use cachette_view::Lap;

const TICKS: u32 = 40;
const SPREAD: u32 = 9973;

fn build(width: u32, height: u32, soldiers: u32) -> World {
    let mut world = World::new(WorldConfig {
        width,
        height,
        seed: 0x0cac_4e77_e5ee_d001,
        faction_count: 4,
    })
    .expect("the extent describes a world");
    // The ground refuses a soldier on water, so the measurement places its
    // soldiers on the open ground it finds.[^1]
    //
    // [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D4, a draft record. `docs/adrs/draft/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
    let open: Vec<Axial> = (0..world.grid().tile_count())
        .map(|index| Axial::new((index % width) as i32, (index / width) as i32))
        .filter(|address| world.admits_a_unit(*address))
        .collect();
    assert!(!open.is_empty(), "the world holds no open ground");
    for index in 0..soldiers {
        world
            .spawn_soldier(
                open[(index.wrapping_mul(SPREAD) as usize) % open.len()],
                FactionId((index % 4) as u16),
            )
            .expect("the address and the faction are valid");
    }
    world
}

fn mean_micros(width: u32, height: u32, soldiers: u32, threads: usize) -> f64 {
    let mut world = build(width, height, soldiers);
    // One step first, so the measurement excludes any first-touch cost.
    world.step(threads).expect("the step must run");
    let at = Lap::start();
    for _ in 0..TICKS {
        world.step(threads).expect("the step must run");
    }
    at.elapsed().as_secs_f64() * 1_000_000.0 / f64::from(TICKS)
}

/// Returns the mean cost of one bridge rebuild, in microseconds.
///
/// The rebuild is the derived structure the step builds at the barrier. It
/// is public, so it can be measured on its own, which separates it from the
/// movement that fills it.
fn rebuild_micros(width: u32, height: u32, soldiers: u32, threads: usize) -> f64 {
    let mut world = build(width, height, soldiers);
    world.rebuild_bridge(threads).expect("the rebuild must run");
    let at = Lap::start();
    for _ in 0..TICKS {
        world.rebuild_bridge(threads).expect("the rebuild must run");
    }
    at.elapsed().as_secs_f64() * 1_000_000.0 / f64::from(TICKS)
}

fn main() {
    let threads = std::thread::available_parallelism()
        .map_or(1, std::num::NonZeroUsize::get)
        .min(12);

    println!("mean step, microseconds, {threads} threads, {TICKS} ticks each");
    println!();
    println!("{:<34} {:>12}", "world", "step us");

    let cases: &[(&str, u32, u32, u32)] = &[
        ("640x440, no soldiers", 640, 440, 0),
        ("640x440, 22000 soldiers", 640, 440, 22_000),
        ("64x44, no soldiers", 64, 44, 0),
        ("64x44, 22000 soldiers", 64, 44, 22_000),
        ("640x440, 2200 soldiers", 640, 440, 2_200),
    ];
    for (name, width, height, soldiers) in cases {
        let us = mean_micros(*width, *height, *soldiers, threads);
        println!("{name:<34} {us:>12.0}");
    }

    println!();
    println!("{:<34} {:>12}", "rebuild alone", "us");
    for (name, width, height, soldiers) in cases {
        let us = rebuild_micros(*width, *height, *soldiers, threads);
        println!("{name:<34} {us:>12.0}");
    }

    println!();
    println!("The first pair separates the tile system from everything else.");
    println!("The second pair holds the soldiers and shrinks the tiles a");
    println!("hundredfold. The last shows what ten times fewer soldiers cost.");
}
