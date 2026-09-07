//! Reads the population of one settlement against the tick, from the
//! founding to the bound it settles at.
//!
//! **The probe drives the engine and founds nothing of its own.** It seeds
//! the world through the one seeding call, then steps it and reads the
//! arenas. A probe that founded by hand would measure the probe.[^1]
//!
//! The output holds one line for each sample. It gives the tick, the
//! residents of the first site of faction zero, the housing of that site, the
//! lodging levels that stand on the seat and beside it, the store of the
//! site, and the people of the whole faction.
//!
//! The caller may state the founding housing and the housing one lodging
//! level gives, so a sweep measures a candidate without a rebuild.
//!
//! # References
//!
//! [^1]: Testing rules, section 5. `.agents/rules/testing.md`

use cachette_core::site::CommodityId;
use cachette_core::types::FactionId;
use cachette_core::upgrade::{UpgradeCategory, UpgradeRow};
use cachette_core::{Entity, World, WorldConfig};

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

/// Returns the first site of one faction, in slot order.
fn first_site(world: &World, faction: FactionId) -> Option<Entity> {
    world
        .settlements()
        .iter()
        .find(|site| world.settlements().faction(*site) == Some(faction))
}

/// Returns the people of one faction, over every site it holds.
fn people_of(world: &World, faction: FactionId) -> u32 {
    world
        .settlements()
        .iter()
        .filter(|site| world.settlements().faction(*site) == Some(faction))
        .filter_map(|site| world.site_residents(site))
        .sum()
}

/// Returns the sites of one faction.
fn sites_of(world: &World, faction: FactionId) -> usize {
    world
        .settlements()
        .iter()
        .filter(|site| world.settlements().faction(*site) == Some(faction))
        .count()
}

/// Returns the lodging levels that stand on the seat of a site and beside it.
fn lodging_levels(world: &World, site: Entity) -> u32 {
    let Some(address) = world.settlements().address(site) else {
        return 0;
    };
    let mut total = 0u32;
    for place in core::iter::once(Some(address)).chain(world.grid().neighbours(address)) {
        let Some(near) = place else {
            continue;
        };
        if world.upgrade_at(near).map(|site| site.category) == Some(UpgradeCategory::LODGING) {
            total += u32::from(world.upgrade_level(near));
        }
    }
    total
}

fn main() {
    let seed = number("--seed", 1);
    let width = number("--width", 256) as u32;
    let height = number("--height", u64::from(width)) as u32;
    let factions = number("--factions", 4) as u16;
    let ticks = number("--ticks", 6000) as u32;
    let threads = number("--threads", 4) as usize;
    let sample = number("--sample", 200) as u32;
    let founding = number("--founding-housing", 0) as u32;
    let step = number("--lodging-housing", 0) as u32;

    let mut world = World::new(WorldConfig {
        width,
        height,
        seed,
        faction_count: factions,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    if founding > 0 {
        world.set_founding_housing(founding);
    }
    if step > 0 {
        for level in 1..=2u8 {
            let row = world
                .upgrade_table()
                .row(UpgradeCategory::LODGING, level)
                .expect("the table holds the level");
            world
                .define_upgrade_row(
                    UpgradeCategory::LODGING.to_u8(),
                    level,
                    UpgradeRow {
                        housing_change: step,
                        ..row
                    },
                )
                .expect("the category and the level are in the table");
        }
    }
    world.seed_world().expect("the world seeds once");
    let owner = FactionId(0);
    let site = first_site(&world, owner).expect("the seeding founds a site");
    let ration = world.need_rule().ration();
    let rate = world
        .production_rate(site, CommodityId(0))
        .expect("the founding sets a rate");
    // The founding sets the rate to the ration times the food the survey
    // read, so the division gives that food back.
    let food = if ration.0 == 0 { 0 } else { rate.0 / ration.0 };
    println!(
        "# seed {seed}, world {width} by {height}, {factions} factions, \
         founding housing {}, lodging step {}",
        world.founding_housing(),
        world
            .upgrade_table()
            .row(UpgradeCategory::LODGING, 1)
            .map_or(0, |row| row.housing_change)
    );
    println!("# the survey of the seat reached {food} food, which feeds {food} people");
    println!("tick\tpeople\thousing\tlodging\tstore\tfaction\tsites");
    for tick in 1..=ticks {
        world.step(threads).expect("the step runs");
        if tick % sample != 0 && tick != 1 {
            continue;
        }
        if world.settlements().address(site).is_none() {
            println!("{tick}\tthe first site is gone");
            break;
        }
        let store = world
            .settlements()
            .store(site)
            .and_then(|held| held.quantity(CommodityId(0)))
            .map_or(0, |held| held.0 / 65536);
        println!(
            "{tick}\t{}\t{}\t{}\t{store}\t{}\t{}",
            world.site_residents(site).unwrap_or(0),
            world.site_housing(site).unwrap_or(0),
            lodging_levels(&world, site),
            people_of(&world, owner),
            sites_of(&world, owner)
        );
    }
}
