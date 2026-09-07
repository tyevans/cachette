//! What one legality answer costs, against one step.
//!
//! A learner asks which actions it may take on every decision, so the cost
//! of that answer sits beside the cost of the step it follows. This
//! benchmark measures the two together, so a reader compares them on one
//! machine and in one run.
//!
//! # What it measures
//!
//! One row builds the whole legality answer for one faction. That is the
//! reader the engine offers.
//!
//! One row runs one step of the same world. That is the comparator.
//!
//! One row builds the observation array of the same faction, because a
//! learner reads both on one decision.
//!
//! # Why the answer stays small
//!
//! The table holds one row for each verb that declares no argument
//! position, and one row for each value of each verb that declares one. No
//! verb of the enumeration carries a tile or a unit, so no bound follows the
//! population and the length of the answer is a small function of the
//! faction count alone. The benchmark prints that length, so a reader sees
//! it rather than takes it from prose.
//!
//! # What it does not measure
//!
//! The measurement runs on the machine that runs it. The project targets
//! another platform, and a local figure misleads on the cache line.[^1] A
//! run on the target platform is what the cost register takes.[^2]
//!
//! # References
//!
//! [^1]: Testing rules, section 3. `.agents/rules/testing.md`
//! [^2]: Blockers register, BLK-007. `docs/BLOCKERS.md`

use std::hint::black_box;
use std::time::Instant;

use cachette_core::{Axial, FactionId, SightRules, World, WorldConfig};

/// The seed that every world in this benchmark takes.
const SEED: u64 = 0x0123_4567_89ab_cdef;

/// How many times each reader row runs.
const READS: u32 = 2_000;

/// How many times the step row runs.
const STEPS: u32 = 100;

/// The faction that asks.
const ASKER: FactionId = FactionId(0);

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

/// Builds a world in which one faction holds a camp of units.
fn a_world(extent: u32, factions: u16, camp: i32, radius: u32) -> World {
    let config = WorldConfig {
        width: extent,
        height: extent,
        seed: SEED,
        faction_count: factions,
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
                    .spawn_soldier(address, ASKER)
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
    measure(64, 4, 32, 16);
    println!();
    measure(256, 16, 32, 6);
}

/// Reports the three rows for one world.
fn measure(extent: u32, factions: u16, camp: i32, radius: u32) {
    let mut world = a_world(extent, factions, camp, radius);
    let schema = world.action_schema();
    println!(
        "A world of {} tiles and {factions} factions. The action table holds {} rows",
        world.grid().tile_count(),
        schema.length()
    );
    let standing = world.standing(ASKER).expect("the faction stands");
    println!(
        "The asker holds {} live units. The observation array holds {} positions",
        standing.live_units,
        world.observation_schema().length()
    );
    println!("Each reader row runs {READS} times, and the step row runs {STEPS} times");
    println!();

    let mut sink = 0u64;

    let start = now();
    for _ in 0..READS {
        let answer = world
            .legal_actions(black_box(ASKER))
            .expect("the faction is of this world");
        sink = sink.wrapping_add(answer.iter().map(|byte| u64::from(*byte)).sum::<u64>());
    }
    row("one legality answer", READS, start.elapsed().as_nanos());

    let start = now();
    for _ in 0..READS {
        let values = world
            .faction_observation(black_box(ASKER))
            .expect("the world describes its own units");
        sink = sink.wrapping_add(values.len() as u64);
    }
    row("one observation array", READS, start.elapsed().as_nanos());

    let start = now();
    for _ in 0..STEPS {
        world.step(1).expect("the step runs");
    }
    row("one step at one thread", STEPS, start.elapsed().as_nanos());

    println!("(the sink reads {sink})");
}
