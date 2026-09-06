//! Reads how many settlements each faction founds during a run.
//!
//! **A fixed set of cities makes a frozen world.** A tile belongs to the
//! faction of the nearest city within reach, so a run that founds nothing
//! after the seeding holds the map it started with. This probe measures
//! whether that is true, over a range of seeds.
//!
//! The probe builds the world the demonstration builds and calls no verb of
//! its own. It seeds the world through the one seeding call, then steps it
//! and reads the arenas after every tick. A probe that founded by hand would
//! measure the probe.[^1]
//!
//! The output holds one line for each seed. It gives the settlements each
//! faction holds at the start and at the end, the tiles each faction holds at
//! the start and at the end, and the tick of each founding after the seeding.
//!
//! # References
//!
//! [^1]: Testing rules, section 5. `.agents/rules/testing.md`

use cachette_core::types::FactionId;
use cachette_core::unit_type::SETTLER;
use cachette_core::{World, WorldConfig};

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

/// Returns the settlements each faction holds.
fn settlements_of(world: &World, factions: u16) -> Vec<usize> {
    let mut counts = vec![0usize; usize::from(factions)];
    for site in world.settlements().iter() {
        if let Some(faction) = world.settlements().faction(site) {
            if usize::from(faction.0) < counts.len() {
                counts[usize::from(faction.0)] += 1;
            }
        }
    }
    counts
}

/// Returns how many settlers each faction holds.
fn settlers_of(world: &World, factions: u16) -> Vec<usize> {
    let mut counts = vec![0usize; usize::from(factions)];
    for unit in world.soldiers().iter() {
        if world.soldiers().unit_type(unit) != Some(SETTLER) {
            continue;
        }
        if let Some(faction) = world.soldiers().faction(unit) {
            if usize::from(faction.0) < counts.len() {
                counts[usize::from(faction.0)] += 1;
            }
        }
    }
    counts
}

/// Returns the tiles each faction holds.
fn held_of(world: &World, factions: u16) -> Vec<i64> {
    (0..factions)
        .map(|index| world.holding_of(FactionId(index)))
        .collect()
}

fn main() {
    let first = number("--first", 1);
    let count = number("--count", 6);
    let width = number("--width", 256) as u32;
    let height = number("--height", u64::from(width)) as u32;
    let factions = number("--factions", 4) as u16;
    let ticks = number("--ticks", 400) as u32;
    let threads = number("--threads", 4) as usize;

    println!(
        "# world {width} by {height}, {factions} factions, {ticks} ticks, \
         seeds {first} to {}",
        first + count - 1
    );
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
        world.seed_world().expect("the world seeds once");
        let start_sites = settlements_of(&world, factions);
        let start_held = held_of(&world, factions);
        let mut last = start_sites.clone();
        let mut foundings: Vec<(u32, u16)> = Vec::new();
        let mut settler_ticks = 0usize;
        let mut settler_peak = 0usize;
        for tick in 0..ticks {
            world.step(threads).expect("the step runs");
            let now = settlements_of(&world, factions);
            for index in 0..now.len() {
                if now[index] > last[index] {
                    for _ in 0..(now[index] - last[index]) {
                        foundings.push((tick, index as u16));
                    }
                }
            }
            last = now;
            let alive: usize = settlers_of(&world, factions).iter().sum();
            if alive > 0 {
                settler_ticks += 1;
            }
            settler_peak = settler_peak.max(alive);
        }
        let end_held = held_of(&world, factions);
        println!(
            "seed {seed}\tsites {start_sites:?} -> {last:?}\theld {start_held:?} -> {end_held:?}\
             \tsettler ticks {settler_ticks} peak {settler_peak}"
        );
        if foundings.is_empty() {
            println!("  foundings: none");
        } else {
            let text: Vec<String> = foundings
                .iter()
                .map(|(tick, faction)| format!("t{tick}:f{faction}"))
                .collect();
            println!("  foundings: {}", text.join(" "));
        }
    }
}
