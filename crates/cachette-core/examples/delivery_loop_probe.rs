//! Reads why a unit of the demonstration world does or does not deliver.
//!
//! The probe drives the step and reports one row for each sample tick. It
//! calls no pass of its own.[^1]
//!
//! Each row holds the population, how many units hold a home, how many are
//! fed, how many are laden, what each option row holds, what the stores hold,
//! and what the engine delivered.
//!
//! Each row also holds what the ground under the units still carries, what the
//! whole world still carries, how many tiles the population stands on, how far
//! a unit stands from its home, and how many units changed tile since the
//! previous tick. Those columns separate a unit that cannot gather from a
//! world that holds nothing to gather.
//!
//! # References
//!
//! [^1]: Testing rules, section 5. `.agents/rules/testing.md`

use cachette_core::choose::{CarryClass, NO_INTENT, OPTIONS, OPTION_COUNT};
use cachette_core::cohort::NEED_FULL;
use cachette_core::resource::ResourceKind;
use cachette_core::CommodityId;
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

fn main() {
    let mut last: std::collections::BTreeMap<u64, (i32, i32)> = std::collections::BTreeMap::new();
    let ticks = number("--ticks", 300);
    let every = number("--every", 20).max(1);
    let mut world = World::new(WorldConfig {
        width: 256,
        height: 256,
        seed: 0x0123_4567_89ab_cdef,
        faction_count: 4,
        ..Default::default()
    })
    .expect("the world builds");
    let outcomes = world.found_run_for_every_faction(64);
    let founded = outcomes.iter().filter(|it| it.founding().is_some()).count();
    let rule = world.need_rule();
    println!(
        "{founded} factions founded, carry mark {}, ration {}, decay {}, need full {}",
        world.carry_mark().0,
        rule.ration().0,
        rule.decay().0,
        NEED_FULL.0
    );
    let names: Vec<&str> = OPTIONS.iter().map(|row| row.name).collect();
    println!(
        "tick units homed fed meanneed laden {} hold sent store atHome delivered carrySum top load1 loadHalf gathering grants granted ordFood ordWood ordStone stocked stockSum foodHere worldFood foodTiles worldTaken tiles free freeOnFood meanDist stepped noExit",
        names.join(" ")
    );

    for tick in 0..=ticks {
        if tick % every == 0 {
            report(&world, tick, &last);
        }
        last.clear();
        for unit in world.soldiers().iter() {
            if let Some(address) = world.soldiers().address(unit) {
                last.insert(unit.to_bits(), (address.q, address.r));
            }
        }
        if tick < ticks {
            world.step(2).expect("the step runs");
        }
    }
}

/// Prints one sample row.
fn report(world: &World, tick: u64, last: &std::collections::BTreeMap<u64, (i32, i32)>) {
    let mut stepped = 0usize;
    let mut no_exit = 0usize;
    let mut units = 0usize;
    let mut homed = 0usize;
    let mut fed = 0usize;
    let mut need_total = 0i64;
    let mut laden = 0usize;
    let mut intents = [0usize; OPTION_COUNT];
    let mut hold = 0usize;
    let mut sent = 0usize;
    let mut at_home_with_load = 0usize;
    let mut carry_total = 0u64;
    let mut top = 0u32;
    let mut some_load = 0usize;
    let mut half_load = 0usize;
    let mut gathering = 0usize;
    let mut ordered = [0usize; 3];
    let mut stocked = 0usize;
    let mut stock_sum = 0u64;
    let mut food_here = 0u64;
    let mut tiles = std::collections::BTreeSet::new();
    let mut free_on_food = 0usize;
    let mut free_units = 0usize;
    let mut distance_sum = 0i64;
    let mut moved_units = 0usize;
    for unit in world.soldiers().iter() {
        units += 1;
        let home = world.soldiers().home(unit).flatten();
        if home.is_some() {
            homed += 1;
        }
        let need = world
            .soldiers()
            .need(unit)
            .unwrap_or(cachette_core::types::Fix32(0));
        need_total += i64::from(need.0);
        if need.0 > 0 {
            fed += 1;
        }
        if world.carry_class(unit) == Some(CarryClass::Laden) {
            laden += 1;
        }
        match world.soldier_intent(unit) {
            Some(Some(NO_INTENT)) | Some(None) => hold += 1,
            Some(Some(option)) if (option as usize) < OPTION_COUNT => {
                intents[option as usize] += 1;
            }
            _ => {}
        }
        if let Some(address) = world.soldiers().address(unit) {
            if last
                .get(&unit.to_bits())
                .is_some_and(|was| *was != (address.q, address.r))
            {
                stepped += 1;
            }
            if let Some(Some(option)) = world.soldier_intent(unit) {
                if world.exit_direction(address, option) == Some(None) {
                    no_exit += 1;
                }
            }
        }
        let is_sent = matches!(world.sent_to(unit), Some(Some(_)));
        if is_sent {
            sent += 1;
        }
        if let Some(address) = world.soldiers().address(unit) {
            tiles.insert((address.q, address.r));
            if !is_sent {
                free_units += 1;
                if world
                    .tile_stock(address, ResourceKind::Food)
                    .is_some_and(|it| it.0 > 0)
                {
                    free_on_food += 1;
                }
            }
            if let Some(home) = home {
                if let Some(site_tile) = world.settlements().tile_column().get(home as usize) {
                    if let Some(site) = world.grid().address_of(*site_tile) {
                        distance_sum += i64::from(address.distance(site));
                        moved_units += 1;
                    }
                }
            }
        }
        let held: u32 = ResourceKind::ALL
            .iter()
            .map(|kind| {
                world
                    .soldiers()
                    .carry(unit)
                    .map_or(0, |load| load.of(*kind).0)
            })
            .sum();
        carry_total += u64::from(held);
        top = top.max(held);
        if held > 0 {
            some_load += 1;
        }
        if held * 2 >= world.carry_mark().0 {
            half_load += 1;
        }
        if let Some(Some(kind)) = world.soldiers().gather_order(unit) {
            gathering += 1;
            ordered[kind.index()] += 1;
            if let Some(address) = world.soldiers().address(unit) {
                let left = world.tile_stock(address, kind).map_or(0, |it| it.0);
                if left > 0 {
                    stocked += 1;
                }
                stock_sum += u64::from(left);
                food_here += u64::from(
                    world
                        .tile_stock(address, ResourceKind::Food)
                        .map_or(0, |it| it.0),
                );
            }
        }
        if held > 0 {
            if let (Some(home), Some(tile)) = (home, world.soldiers().tile(unit)) {
                if world
                    .settlements()
                    .tile_column()
                    .get(home as usize)
                    .is_some_and(|site| *site == tile)
                {
                    at_home_with_load += 1;
                }
            }
        }
    }
    let mut world_food = 0u64;
    let mut world_taken = 0u64;
    let mut food_tiles = 0usize;
    for row in 0..world.grid().height() {
        for column in 0..world.grid().width() {
            let address = cachette_core::Axial::new(column as i32, row as i32);
            let left = world
                .tile_stock(address, ResourceKind::Food)
                .map_or(0, |it| it.0);
            world_food += u64::from(left);
            if left > 0 {
                food_tiles += 1;
            }
            world_taken += u64::from(
                world
                    .taken_from(address, ResourceKind::Food)
                    .map_or(0, |it| it.0),
            );
        }
    }
    let store: i64 = world
        .settlements()
        .iter()
        .map(|site| {
            world
                .settlements()
                .store(site)
                .and_then(|store| store.quantity(CommodityId(0)))
                .map_or(0, |value| i64::from(value.0))
        })
        .sum();
    let delivered: u64 = world.delivered_carry().iter().sum();
    let grants = world.gather_log().len();
    let granted: u64 = world
        .gather_log()
        .iter()
        .map(|taken| u64::from(taken.amount))
        .sum();
    let mean_need = if units == 0 {
        0
    } else {
        need_total / units as i64
    };
    let rows = intents
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(" ");
    let tile_count = tiles.len();
    let mean_distance = if moved_units == 0 {
        0
    } else {
        distance_sum / moved_units as i64
    };
    let order_food = ordered[0];
    let order_wood = ordered[1];
    let order_stone = ordered[2];
    println!(
        "{tick} {units} {homed} {fed} {mean_need} {laden} {rows} {hold} {sent} {store} \
         {at_home_with_load} {delivered} {carry_total} {top} {some_load} {half_load} \
         {gathering} {grants} {granted} {order_food} {order_wood} {order_stone} {stocked} \
         {stock_sum} {food_here} {world_food} {food_tiles} {world_taken} {tile_count} \
         {free_units} {free_on_food} {mean_distance} {stepped} {no_exit}"
    );
}
