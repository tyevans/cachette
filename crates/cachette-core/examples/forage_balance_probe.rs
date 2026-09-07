//! A probe that reports where a faction's food comes from over a run.
//!
//! This is a diagnostic, not a test. It exists so that the recovery rate is
//! chosen against what the engine does rather than against a derivation.
//!
//! The probe splits every gather event by whether the tile it came from
//! carried a finished upgrade at that tick. It reports the split, the food
//! the sites hold, the population, and what the moisture was doing over the
//! tiles that produced the food.

use cachette_core::founding::{disc, SURVEY_RADIUS};
use cachette_core::resource::{ResourceKind, MOISTURE_BAND_CEILING};
use cachette_core::upgrade::UpgradeCategory;
use cachette_core::{Entity, FactionId, World, WorldConfig};

/// The ticks that the run walks.
const TICKS: u32 = 2000;

/// The ticks between two reports.
const REPORT: u32 = 400;

/// The threads that the step runs on.
const THREADS: usize = 4;

/// The seeds that the probe walks.
const SEEDS: [u64; 3] = [
    0x0cac_4e77_5104_0001,
    0x0cac_4e77_5104_0003,
    0x0cac_4e77_5104_0007,
];

fn main() {
    for seed in SEEDS {
        run(seed);
    }
}

fn run(seed: u64) {
    let mut world = World::new(WorldConfig {
        width: 192,
        height: 192,
        seed,
        faction_count: 4,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the settings describe a world");
    let outcomes = world.found_run_for_every_faction(30);
    let sites: Vec<Entity> = outcomes
        .iter()
        .filter_map(|outcome| outcome.founding())
        .map(|founding| founding.settlement())
        .collect();
    println!("=== seed {seed:#x}, {} sites ===", sites.len());

    // Order every unit to forage, so the probe measures the worst case for
    // the ground rather than the best case for the sites.
    let units: Vec<Entity> = world.soldiers().iter().collect();
    world.order_gather_set(&units, ResourceKind::Food);
    world.order_build_set(&units, UpgradeCategory::TERRACE);

    let mut bare_food = 0i64;
    let mut improved_food = 0i64;
    let mut bare_wood = 0i64;
    let mut improved_wood = 0i64;
    let mut bands = [0i64; 7];

    for tick in 0..=TICKS {
        for event in world.gather_log() {
            let Some(address) = world.grid().address_of(event.tile) else {
                continue;
            };
            let improved = world.finished_upgrade(address).is_some();
            let amount = i64::from(event.amount);
            match (ResourceKind::from_u8(event.kind), improved) {
                (Some(ResourceKind::Food), true) => improved_food += amount,
                (Some(ResourceKind::Food), false) => bare_food += amount,
                (Some(ResourceKind::Wood), true) => improved_wood += amount,
                (Some(ResourceKind::Wood), false) => bare_wood += amount,
                _ => {}
            }
            let drops = world.ground_water_at(address).unwrap_or(0);
            bands[band_of(drops)] += amount;
        }
        if tick % REPORT == 0 {
            report(&world, tick, bare_food, improved_food, &sites);
        }
        world.step(THREADS).expect("the world steps");
        // A unit that finishes a task takes no new one, so the probe places
        // the orders again.
        if tick % 32 == 0 {
            let live: Vec<Entity> = world.soldiers().iter().collect();
            world.order_gather_set(&live, ResourceKind::Food);
            world.order_build_set(&live, UpgradeCategory::TERRACE);
        }
    }
    println!(
        "food gathered: bare {bare_food}, improved {improved_food}; \
         wood gathered: bare {bare_wood}, improved {improved_wood}"
    );
    println!("food and wood by the moisture band of the tile: {bands:?}");
    let terraces: usize = sites
        .iter()
        .filter_map(|site| world.settlements().address(*site))
        .map(|address| {
            disc(world.grid(), address, SURVEY_RADIUS)
                .into_iter()
                .filter(|place| world.finished_upgrade(*place) == Some(UpgradeCategory::TERRACE))
                .count()
        })
        .sum();
    println!("terraces standing inside the site discs: {terraces}");
}

fn report(world: &World, tick: u32, bare: i64, improved: i64, sites: &[Entity]) {
    let mut stored = 0i64;
    let mut people = 0u32;
    for faction in 0..4u16 {
        stored += world.faction_stores(FactionId(faction))[ResourceKind::Food.index()];
        people += world.population_of(FactionId(faction));
    }
    let residents: u32 = sites
        .iter()
        .filter_map(|site| world.site_residents(*site))
        .sum();
    println!(
        "tick {tick:>5} food stored {stored:>8} units {people:>5} residents {residents:>5} \
         gathered bare {bare:>7} improved {improved:>7}"
    );
}

fn band_of(drops: i64) -> usize {
    let mut band = 0;
    while band < MOISTURE_BAND_CEILING.len() {
        if drops < MOISTURE_BAND_CEILING[band] {
            return band;
        }
        band += 1;
    }
    MOISTURE_BAND_CEILING.len()
}
