//! Reads how many pairs of a faction's settlements a finished road joins.
//!
//! **The probe asks the ground and never the planner.** It walks the finished
//! joining upgrades that stand, together with the settlement tiles of one
//! faction, and it reports which pairs of settlements that walk connects. A
//! probe that asked the plan whether it thought a pair was joined would
//! measure the plan.[^1]
//!
//! The probe builds the world the demonstration builds and calls no verb of
//! its own. It seeds the world through the one seeding call, then steps it
//! and reads the arenas after every tick.
//!
//! The output holds one line for each seed. It gives the settlements each
//! faction ends with, the pairs a road joins at the end, the tick of the
//! first join, and the road tiles that stand.
//!
//! # References
//!
//! [^1]: Testing rules, section 5. `.agents/rules/testing.md`

use std::collections::{HashSet, VecDeque};

use cachette_core::types::FactionId;
use cachette_core::{Axial, World, WorldConfig};

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

/// Returns the addresses of the settlements of one faction.
fn sites_of(world: &World, faction: FactionId) -> Vec<Axial> {
    let mut places: Vec<Axial> = world
        .settlements()
        .iter()
        .filter(|site| world.settlements().faction(*site) == Some(faction))
        .filter_map(|site| world.settlements().address(site))
        .collect();
    places.sort_unstable_by_key(|address| (address.r, address.q));
    places
}

/// Reports whether a finished road stands on one address.
fn road_stands(world: &World, address: Axial) -> bool {
    world.upgrade_at(address).is_some_and(|site| {
        site.is_complete() && site.category == cachette_core::UpgradeCategory::ROAD
    })
}

/// Returns how many pairs of settlements of one faction a road walk connects.
///
/// The walk starts at each settlement in turn and crosses a tile when a
/// finished road stands on it, or when a settlement of the same faction
/// stands on it. Two settlements in one walk are one joined pair.
fn joined_pairs(world: &World, _faction: FactionId, sites: &[Axial]) -> usize {
    if sites.len() < 2 {
        return 0;
    }
    let places: HashSet<Axial> = sites.iter().copied().collect();
    let mut pairs = 0;
    for (index, start) in sites.iter().enumerate() {
        let mut seen: HashSet<Axial> = HashSet::new();
        let mut queue = VecDeque::new();
        seen.insert(*start);
        queue.push_back(*start);
        let mut reached: HashSet<Axial> = HashSet::new();
        while let Some(here) = queue.pop_front() {
            for neighbour in world.grid().neighbours(here).into_iter().flatten() {
                if seen.contains(&neighbour) {
                    continue;
                }
                let is_site = places.contains(&neighbour);
                if !is_site && !road_stands(world, neighbour) {
                    continue;
                }
                seen.insert(neighbour);
                if is_site {
                    reached.insert(neighbour);
                    continue;
                }
                queue.push_back(neighbour);
            }
        }
        for other in sites.iter().skip(index + 1) {
            if reached.contains(other) {
                pairs += 1;
            }
        }
    }
    pairs
}

/// Returns the road projects one faction plans, the roads that stand
/// anywhere in the world, and the roads that are part built.
///
/// **A finished project leaves the plan**, so the plan cannot say how many
/// roads stand. The count walks the addresses of the world instead, which is
/// a pass the engine never makes and a probe may.
fn roads_of(world: &World, faction: FactionId, width: u32, height: u32) -> (usize, usize, usize) {
    let planned = world
        .plan_of(faction)
        .iter()
        .filter(|project| project.category == cachette_core::UpgradeCategory::ROAD)
        .count();
    let mut standing = 0;
    let mut started = 0;
    for r in 0..height {
        for q in 0..width {
            let Some(site) = world.upgrade_at(Axial::new(q as i32, r as i32)) else {
                continue;
            };
            if site.category != cachette_core::UpgradeCategory::ROAD {
                continue;
            }
            if site.is_complete() {
                standing += 1;
            } else {
                started += 1;
            }
        }
    }
    (planned, standing, started)
}

fn main() {
    let first = number("--first", 1);
    let count = number("--count", 6);
    let width = number("--width", 256) as u32;
    let height = number("--height", u64::from(width)) as u32;
    let factions = number("--factions", 4) as u16;
    let ticks = number("--ticks", 400) as u32;
    let threads = number("--threads", 4) as usize;
    // The demonstration controller draws a category and orders the whole
    // faction to build it where it stands. That order competes with the
    // project order for the same units. The flag removes the competing draw,
    // so a reader can see what the plan alone finishes.
    let evaluations = number("--evaluations", 1) as u32;

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
        world.set_controller_evaluations(evaluations);
        world.seed_world().expect("the world seeds once");
        let mut first_join: Vec<Option<u32>> = vec![None; usize::from(factions)];
        for tick in 0..ticks {
            world.step(threads).expect("the step runs");
            for index in 0..factions {
                if first_join[usize::from(index)].is_some() {
                    continue;
                }
                let faction = FactionId(index);
                let sites = sites_of(&world, faction);
                if joined_pairs(&world, faction, &sites) > 0 {
                    first_join[usize::from(index)] = Some(tick);
                }
            }
        }
        let mut report = Vec::new();
        for index in 0..factions {
            let faction = FactionId(index);
            let sites = sites_of(&world, faction);
            if sites.len() < 2 {
                continue;
            }
            let pairs = joined_pairs(&world, faction, &sites);
            let (planned, standing, started) = roads_of(&world, faction, width, height);
            report.push(format!(
                "f{index}: {} sites, {pairs} joined pairs of {}, {planned} road projects, \
                 {standing} roads stand and {started} are part built, first join {}",
                sites.len(),
                sites.len() * (sites.len() - 1) / 2,
                first_join[usize::from(index)]
                    .map_or_else(|| "never".to_string(), |tick| format!("t{tick}"))
            ));
        }
        // The gap between the two nearest settlements of each faction, and
        // how much of the ground between them carries a road. This says
        // whether the ways stop short or whether nobody finishes them.
        for index in 0..factions {
            let faction = FactionId(index);
            let sites = sites_of(&world, faction);
            if sites.len() < 2 {
                continue;
            }
            let planned: Vec<Axial> = world
                .plan_of(faction)
                .iter()
                .filter(|project| project.category == cachette_core::UpgradeCategory::ROAD)
                .filter_map(|project| world.grid().address_of(project.tile))
                .collect();
            let near_first: usize = planned
                .iter()
                .filter(|address| sites.iter().any(|site| site.distance(**address) <= 2))
                .count();
            let gap = sites
                .iter()
                .enumerate()
                .flat_map(|(i, a)| sites.iter().skip(i + 1).map(move |b| a.distance(*b)))
                .min()
                .unwrap_or(0);
            println!(
                "    f{index} debug: nearest gap {gap}, {} planned roads, {near_first} of them \
                 within two of a settlement",
                planned.len()
            );
        }
        println!("seed {seed}");
        if report.is_empty() {
            println!("  no faction holds two settlements");
        } else {
            for line in report {
                println!("  {line}");
            }
        }
    }
}
