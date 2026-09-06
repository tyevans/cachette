//! Reads how long a settler lives away from its site, and how far it gets.
//!
//! The probe spawns one settler at the seat of a faction, sends it at a
//! distant tile, and reads the arena after every tick. It reports the tick
//! the unit died on, the deficit it carried, the tiles it walked, and whether
//! the starved log names it. A settler that vanishes with no entry in that
//! log died on some path other than the shortage.
//!
//! The probe calls the send verb a Python caller calls. It founds through the
//! one founding call. It drives no pass of its own.[^1]
//!
//! # References
//!
//! [^1]: Testing rules, section 5. `.agents/rules/testing.md`

use std::collections::{BTreeSet, VecDeque};

use cachette_core::site::CommodityId;
use cachette_core::terrain::TileKind;
use cachette_core::types::Fix32;
use cachette_core::unit_type::SETTLER;
use cachette_core::{Axial, FactionId, World, WorldConfig};

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

/// Returns the passable tiles a walker reaches from one address on foot.
fn land_reach(world: &World, from: Axial) -> BTreeSet<Axial> {
    let mut seen = BTreeSet::new();
    let mut queue = VecDeque::new();
    if !world.admits_a_unit(from) {
        return seen;
    }
    seen.insert(from);
    queue.push_back(from);
    while let Some(here) = queue.pop_front() {
        for direction in 0..6 {
            let Some(there) = world.grid().neighbour(here, direction) else {
                continue;
            };
            if !world.admits_a_unit(there) || seen.contains(&there) {
                continue;
            }
            seen.insert(there);
            queue.push_back(there);
        }
    }
    seen
}

/// The smallest landmass of a world that could hold a founding group.
///
/// The walk is a measurement of the fixture and not a pass of the engine, so
/// it may read the whole world.
fn smallest_island(world: &World) -> BTreeSet<Axial> {
    /// The smallest landmass the probe seats a group on.
    const FLOOR: usize = 64;
    let grid = world.grid();
    let mut seen: BTreeSet<Axial> = BTreeSet::new();
    let mut best: Option<BTreeSet<Axial>> = None;
    for q in 0..grid.width() as i32 {
        for r in 0..grid.height() as i32 {
            let at = Axial::new(q, r);
            if seen.contains(&at) || !world.admits_a_unit(at) {
                continue;
            }
            let mass = land_reach(world, at);
            for address in &mass {
                seen.insert(*address);
            }
            if mass.len() < FLOOR {
                continue;
            }
            if best.as_ref().is_none_or(|held| mass.len() < held.len()) {
                best = Some(mass);
            }
        }
    }
    best.unwrap_or_default()
}

fn main() {
    let seed = number("--seed", 1);
    let extent = number("--width", 256) as u32;
    let ticks = number("--ticks", 400) as u32;
    let threads = number("--threads", 1) as usize;
    let group = number("--group", 8) as u32;
    let span = number("--span", 40) as u32;
    let feed = number("--feed", 1) == 1;
    // The island mode seats the faction on the smallest landmass of the
    // world, so the walk the probe orders must cross water.
    let island_mode = number("--island", 0) == 1;

    let mut world = World::new(WorldConfig {
        width: extent,
        height: extent,
        seed,
        faction_count: 2,
        ..WorldConfig::default()
    })
    .expect("the extent must describe a world");
    world.set_founding_housing(256);
    let faction = FactionId(0);
    // The island mode seats the group on the smallest landmass that could
    // hold it, so the walk must cross water. The ordinary mode lets the
    // founding survey choose, which seats the group on the largest continent.
    let founding = if island_mode {
        let island = smallest_island(&world);
        let places: Vec<Axial> = island.iter().copied().collect();
        let survey = world
            .survey_places(&places, group, &[])
            .expect("the island holds places to survey");
        let seat = survey
            .chosen()
            .map_or(places[0], |candidate| candidate.address());
        world
            .found_group_at(seat, group, faction)
            .expect("the island must seat the group")
    } else {
        world
            .found_run(group, faction)
            .expect("the world must seat the faction")
    };
    let seat = founding.place();
    if feed {
        world
            .set_production_rate(founding.settlement(), CommodityId(0), Fix32::from_int(256))
            .expect("the rate is above zero");
    }
    // The controller of the faction stands down, so the walk the probe orders
    // is the walk it measures.
    world.set_externally_controlled(faction, true);

    // The target lies `span` tiles away, in ascending address order, so the
    // answer is a property of the world and not of a draw. In the island mode
    // the target is instead the nearest ground off the island, which is what
    // a settler that crosses water should aim at.
    let grid = world.grid();
    let home_land = land_reach(&world, seat);
    let mut target = None;
    let mut best_span = u32::MAX;
    for q in 0..grid.width() as i32 {
        for r in 0..grid.height() as i32 {
            let at = Axial::new(q, r);
            if !world.admits_a_unit(at) {
                continue;
            }
            if island_mode {
                if home_land.contains(&at) {
                    continue;
                }
                let reach = seat.distance(at);
                // A destination field is over level 1 cells, so a target
                // inside the cell that holds the settler steers nobody. The
                // span argument lets the probe put the target beyond that
                // cell and separate the two cases.
                if reach < span {
                    continue;
                }
                if reach < best_span {
                    best_span = reach;
                    target = Some(at);
                }
            } else if seat.distance(at) >= span && target.is_none() {
                target = Some(at);
            }
        }
    }
    let target = target.expect("the world holds ground that far away");

    let unit = world
        .spawn_soldier(seat, faction)
        .expect("the seat admits one unit");
    world.set_unit_type_set(&[unit], SETTLER);
    let plane = 4;
    world
        .send_units_to(&[unit], &[target], plane)
        .expect("the world holds the plane");

    println!(
        "seed {seed}: seat {seat:?}, target {target:?}, span {}, landmass {} tiles, fed {feed}",
        seat.distance(target),
        home_land.len()
    );

    let mut walked = 0u32;
    let mut last = seat;
    let mut water_ticks = 0u32;
    let mut furthest = 0u32;
    for tick in 0..ticks {
        world.step(threads).expect("the step runs");
        let Some(at) = world.soldiers().address(unit) else {
            let starved = world
                .starved_log()
                .iter()
                .any(|event| event.unit == unit.to_bits());
            println!(
                "  tick {tick}: the unit is gone. the starved log names it: {starved}. \
                 walked {walked} steps, furthest {furthest}, water ticks {water_ticks}"
            );
            return;
        };
        if at != last {
            walked += 1;
            last = at;
        }
        furthest = furthest.max(seat.distance(at));
        if world.tile_kind(at) == Some(TileKind::Water) {
            water_ticks += 1;
        }
        let deficit = world.soldiers().deficit(unit).unwrap_or(Fix32::ZERO);
        if tick % 10 == 0 || water_ticks > 0 {
            println!(
                "  tick {tick}: at {at:?} ({:?}) distance {} deficit {} home {:?}",
                world.tile_kind(at),
                seat.distance(at),
                deficit.0,
                world.soldiers().home(unit)
            );
        }
    }
    println!(
        "  the unit survived {ticks} ticks. walked {walked} steps, furthest {furthest}, \
         water ticks {water_ticks}"
    );
}
