//! A probe that reports what the production pipeline reads over a run.
//!
//! This is a diagnostic, not a test. It exists so that a fixture can be built
//! against what the engine does rather than against what the author guessed.

use cachette_core::founding::{disc, SURVEY_RADIUS};
use cachette_core::resource::{RecoveryRules, ResourceKind};
use cachette_core::upgrade::UpgradeCategory;
use cachette_core::{CommodityId, Entity, World, WorldConfig};

fn main() {
    let mut world = World::new(WorldConfig {
        width: 192,
        height: 192,
        seed: 0x0cac_4e77_5104_0001,
        faction_count: 4,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the settings describe a world");
    let outcomes = world.found_run_for_every_faction(30);
    let site: Entity = outcomes
        .iter()
        .filter_map(|outcome| outcome.founding())
        .map(|founding| founding.settlement())
        .next()
        .expect("a group seats");
    let argument = std::env::args().nth(1).unwrap_or_default();
    if argument == "none" {
        world.set_recovery_rules(RecoveryRules::NONE);
    }
    let units: Vec<Entity> = world.soldiers().iter().collect();
    println!("units {}", units.len());
    let refused = world.order_gather_set(&units, ResourceKind::Food);
    println!("gather refused {refused}");
    let (refused, reason) = world.order_build_set_reporting(&units, UpgradeCategory::TERRACE);
    println!("build refused {refused} reason {reason:?}");
    let address = world.settlements().address(site).expect("live");
    let places = disc(world.grid(), address, SURVEY_RADIUS);
    for tick in 0..=2000u32 {
        if tick % 20 == 0 {
            let taken: u32 = places
                .iter()
                .filter_map(|place| world.taken_from(*place, ResourceKind::Food))
                .map(|amount| amount.0)
                .sum();
            let kinds: Vec<u8> = places
                .iter()
                .filter_map(|place| world.finished_upgrade(*place))
                .map(|category| category.0)
                .collect();
            let terraces = places
                .iter()
                .filter(|place| world.finished_upgrade(**place) == Some(UpgradeCategory::TERRACE))
                .count();
            let levels: usize = places
                .iter()
                .filter(|place| world.upgrade_level(**place) > 0)
                .count();
            let any = places
                .iter()
                .filter(|place| world.upgrade_at(**place).is_some())
                .count();
            println!(
                "tick {tick} taken {taken} terraces {terraces} levels {levels} entries {any} \
                 scale {:?} effective {:?} residents {:?} wet {:?} kinds {kinds:?}",
                world.production_scale(site),
                world.effective_production_rate(site, CommodityId(0)),
                world.site_residents(site),
                world.ground_is_wet(address),
            );
        }
        world.step(4).expect("the world steps");
    }
}
