//! Reads what a range of seeds makes of the demonstration world.
//!
//! **One seed measures one world.** A reading of the seeder taken at one seed
//! says nothing about the seed a watcher draws, and the demonstration draws a
//! new seed on every run. This sweep runs many seeds and prints one row for
//! each, so the reader sees the distribution and not an average.[^1]
//!
//! The sweep builds the world the demonstration builds and calls no verb of
//! its own. It seeds the world through the one seeding call, steps it, and
//! reads the census. A sweep that founded by hand would measure the sweep.[^2]
//!
//! The output is one header line and then one line for each seed, with the
//! fields separated by a tab. A reader loads it with any tool.
//!
//! # References
//!
//! [^1]: Findings register, FND-496. `docs/FINDINGS.md`
//! [^2]: Testing rules, section 5. `.agents/rules/testing.md`

use cachette_core::plan::PlanRules;
use cachette_core::{World, WorldConfig, SUBSYSTEM_CENSUS};

/// Returns one census row of a world.
fn census(world: &World, name: &str) -> i64 {
    SUBSYSTEM_CENSUS
        .iter()
        .find(|row| row.name == name)
        .map(|row| (row.read)(world))
        .expect("the census holds the row")
}

/// Returns the value of a named argument, or the fallback.
fn number(name: &str, fallback: u64) -> u64 {
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        if argument == name {
            return args
                .next()
                .and_then(|value| value.parse().ok())
                .unwrap_or(fallback);
        }
    }
    fallback
}

fn main() {
    let first = number("--first", 1);
    let count = number("--count", 200);
    let width = number("--width", 256) as u32;
    let height = number("--height", width as u64) as u32;
    let factions = number("--factions", 4) as u16;
    let ticks = number("--ticks", 800) as u32;
    let threads = number("--threads", 4) as usize;
    // The plan bound of the engine, unless the caller names another one. The
    // sweep states no default of its own, because a second declaration of the
    // bound could disagree with the engine and nothing would fail.
    let bound = number("--plan-bound", 0) as u32;

    println!(
        "# world {width} by {height}, {factions} factions, {ticks} ticks, \
         seeds {first} to {}",
        first + count - 1
    );
    println!("seed\tseated\tsettlements\tunits\tzoned\tfinished\tdropped\tended\tupgrades");
    for offset in 0..count {
        let seed = first + offset;
        let mut world = World::new(WorldConfig {
            width,
            height,
            seed,
            faction_count: factions,
            unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        })
        .expect("the extent must describe a world");
        if bound > 0 {
            world.set_plan_rules(PlanRules::DEFAULT.with_bound(bound));
        }
        let outcomes = world.seed_world().expect("the world seeds once");
        let seated = outcomes
            .iter()
            .filter(|outcome| outcome.founding().is_some())
            .count();
        for _ in 0..ticks {
            world.step(threads).expect("the step runs");
        }
        println!(
            "{seed}\t{seated}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            census(&world, "settlements"),
            census(&world, "units"),
            census(&world, "projects_zoned"),
            census(&world, "projects_finished"),
            census(&world, "projects_dropped"),
            census(&world, "game_ended"),
            census(&world, "upgrades_complete"),
        );
    }
}
