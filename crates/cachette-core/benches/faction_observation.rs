//! What one observation array costs, against one step.
//!
//! A learner reads one array on every decision, so the cost of building that
//! array sits beside the cost of the step it follows. This benchmark measures
//! the two together, so a reader compares them on one machine and in one run.
//!
//! # What it measures
//!
//! One row builds the whole array for one faction. That is the reader the
//! engine offers.
//!
//! One row builds the map part of the same array by the shape this work
//! rejected: one call to the per-cell reader for each cell of the lattice.
//! The two rows say what the whole-set read buys.
//!
//! One row runs one step of the same world. That is the comparator.
//!
//! Each row runs for a faction that camps in one corner. A faction that has
//! walked more of the map pays more, because the reader walks the cells the
//! faction observed and no other cell.
//!
//! # What it does not measure
//!
//! The measurement runs on the machine that runs it. The project targets
//! another platform, and a local figure misleads on the cache line.[^1] A run
//! on the target platform is what the cost register takes.[^2]
//!
//! # References
//!
//! [^1]: Testing rules, section 3. `.agents/rules/testing.md`
//! [^2]: Blockers register, BLK-007. `docs/BLOCKERS.md`

use std::hint::black_box;
use std::time::Instant;

use cachette_core::faction_observation::observation_schema;
use cachette_core::faction_view::Admit;
use cachette_core::{Axial, FactionId, SightRules, World, WorldConfig};

/// The seed that every world in this benchmark takes.
const SEED: u64 = 0x0123_4567_89ab_cdef;

/// How many times each row builds an array.
const READS: u32 = 2_000;

/// The faction that watches.
const WATCHER: FactionId = FactionId(0);

/// The exponent that keeps a unit still.
const KEEP_STILL: u32 = 12;

/// Reads the clock.
///
/// One lint forbids the clock across this workspace, because a simulation
/// that reads a clock gives an answer that depends on the load of the
/// machine.[^1] A benchmark is the one caller that must read it: it produces
/// no simulated state, it enters no state hash, and it asserts nothing.[^2]
///
/// # References
///
/// [^1]: ADR-0005, decision D1. `docs/adrs/REGISTRY.md`
/// [^2]: Testing rules, section 3. `.agents/rules/testing.md`
#[allow(clippy::disallowed_methods)]
fn now() -> Instant {
    Instant::now()
}

/// Builds a world in which one faction camps in one corner.
///
/// The camp is a square of the given side at the origin. A faction that camps
/// in a corner of a wide world observes few of its cells, which is the shape
/// a learner meets at the start of a run.
fn a_world(extent: u32, camp: i32, radius: u32) -> World {
    let config = WorldConfig {
        width: extent,
        height: extent,
        seed: SEED,
        faction_count: 4,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    };
    let mut world = World::new(config).expect("the configuration describes a world");
    world
        .set_choice_schedule(KEEP_STILL)
        .expect("the exponent is inside the range");
    world.set_sight_rules(SightRules::new(radius, 1, 16, 0));
    for row in 0..camp {
        for column in 0..camp {
            let address = Axial::new(column, row);
            if world.admits_a_unit(address) {
                world
                    .spawn_soldier(address, WATCHER)
                    .expect("the ground admits a unit");
            }
        }
    }
    world.step(1).expect("the step runs");
    world
}

/// Reports one row.
fn row(name: &str, runs: u32, nanoseconds: u128) {
    let each = nanoseconds / u128::from(runs);
    println!("{name:<38} {each:>9} ns for each run");
}

fn main() {
    measure(64, 32, 16);
    println!();
    measure(256, 32, 6);
}

/// Reports the three rows for one world.
fn measure(extent: u32, camp: i32, radius: u32) {
    let world = a_world(extent, camp, radius);
    let layout = world.observation().layout();
    let cells = layout.block_count();
    let length = observation_schema().length();
    println!(
        "A world of {} tiles, {} cells of {} tiles, and an array of {} positions",
        world.grid().tile_count(),
        cells,
        layout.block_edge() * layout.block_edge(),
        length
    );
    let standing = world.standing(WATCHER).expect("the faction stands");
    let observed = (0..cells)
        .filter(|block| {
            world
                .observation()
                .remembered_layer(WATCHER)
                .is_some_and(|layer| layer.block_population(*block) > 0)
        })
        .count();
    println!(
        "The watcher holds {} live units and has observed {observed} of the {cells} cells",
        standing.live_units
    );
    println!("Each reader row runs {READS} times");
    println!();

    let mut sink = 0i64;

    // The array holds what the faction observes and what it holds. This row
    // separates the two, because the second half walks no cell.
    let start = now();
    for _ in 0..READS {
        sink += black_box(&world)
            .standing(black_box(WATCHER))
            .expect("the faction stands")
            .held_tiles;
    }
    row(
        "the standing alone, no cell walked",
        READS,
        start.elapsed().as_nanos(),
    );

    let start = now();
    for _ in 0..READS {
        sink += black_box(&world)
            .faction_observation(black_box(WATCHER))
            .expect("the faction names a faction of this world")
            .len() as i64;
    }
    row(
        "one observation, the whole-set read",
        READS,
        start.elapsed().as_nanos(),
    );

    // The rejected shape. It calls the per-cell reader once for each cell,
    // so it looks the layer up for every cell and it walks the tiles of a
    // cell the faction has never seen.
    let addresses: Vec<Axial> = (0..cells)
        .map(|block| {
            let edge = layout.block_edge();
            let column = (block % layout.blocks_wide()) * edge;
            let row = (block / layout.blocks_wide()) * edge;
            Axial::new(column as i32, row as i32)
        })
        .collect();
    let start = now();
    for _ in 0..READS {
        for address in &addresses {
            sink += world
                .faction_summary_covering(WATCHER, *address, Admit::SeenEver)
                .expect("the cell covers the address")
                .admitted();
        }
    }
    row(
        "the map alone, one call for each cell",
        READS,
        start.elapsed().as_nanos(),
    );

    let mut stepping = a_world(extent, camp, radius);
    let steps = 50u32;
    let start = now();
    for _ in 0..steps {
        stepping.step(1).expect("the step runs");
    }
    row(
        "one step of the same world",
        steps,
        start.elapsed().as_nanos(),
    );

    println!();
    println!("The sum of every reading is {sink}. Nothing reads it.");
}
