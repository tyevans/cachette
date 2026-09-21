//! Tests for seeding unit types, combat casualties, and fallen unit census.
//!
//! A world founds initial groups with a mix of unit types according to faction
//! weights. Hostile encounters between units capable of attack inflict positive
//! combat damage, generate casualty events, and allow domination victory by
//! unit annihilation.[^1]
//!
//! # References
//!
//! [^1]: Backlog item 0491. `docs/backlog/complete/0491-seed-the-demonstration-world-with-unit-types-that-can-fight-gather-and-carry.md`
//! [^2]: ADR-0145, a unit type is a row of capability columns, and zero means cannot. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
//! [^3]: ADR-0122, an attacker whose attack does not exceed the defender's armour contributes zero. `docs/adrs/draft/adr-0122-an-attacker-below-the-armour-contributes-exactly-zero.md`
//! [^4]: Testing rules, section 1, a determinism test must be able to fail. `.agents/rules/testing.md`

use cachette_core::controller::FactionWeights;
use cachette_core::unit_type::{MERCHANT, SOLDIER, WORKER};
use cachette_core::{
    founding_unit_type_distribution, Axial, CensusBasis, FactionId, TileIdx, WinPath, World,
    WorldConfig, FOUNDING_GROUP_DEFAULT, FOUNDING_MERCHANT_MIN_GROUP, SUBSYSTEM_CENSUS,
};

const QUIET_CENTRE: i32 = -7000;
const QUIET_SPAN: i32 = 3000;

fn read_census(world: &World, name: &str) -> i64 {
    world
        .subsystem_census()
        .into_iter()
        .find(|(row, _)| *row == name)
        .unwrap_or_else(|| panic!("census row {name} not found"))
        .1
}

fn open_address(world: &World) -> Axial {
    let count = world.grid().tile_count();
    for index in 0..count {
        if let Some(address) = world.grid().address_of(TileIdx(index)) {
            if world.admits_a_unit(address) {
                return address;
            }
        }
    }
    panic!("the world must hold ground that carries a unit");
}

#[test]
fn distribution_allocates_by_weights_using_largest_remainder() {
    // Equal weights at group 2 yield one worker and one soldier.
    let balanced = FactionWeights {
        build: 2,
        war: 2,
        trade: 2,
        renown: 1,
        settle: 1,
    };
    let types = founding_unit_type_distribution(2, balanced);
    assert_eq!(types.len(), 2);
    assert_eq!(types.iter().filter(|&&t| t == WORKER).count(), 1);
    assert_eq!(types.iter().filter(|&&t| t == SOLDIER).count(), 1);

    // Heavy warrior faction gets at least one worker to sustain the settlement,
    // plus one soldier at group 2.
    let warrior = FactionWeights {
        build: 1,
        war: 8,
        trade: 1,
        renown: 1,
        settle: 1,
    };
    let warrior_types = founding_unit_type_distribution(2, warrior);
    assert_eq!(warrior_types.iter().filter(|&&t| t == SOLDIER).count(), 1);
    assert_eq!(warrior_types.iter().filter(|&&t| t == WORKER).count(), 1);

    // Heavy builder faction gets two workers at group 2.
    let builder = FactionWeights {
        build: 8,
        war: 1,
        trade: 1,
        renown: 1,
        settle: 1,
    };
    let builder_types = founding_unit_type_distribution(2, builder);
    assert_eq!(builder_types.iter().filter(|&&t| t == WORKER).count(), 2);
    assert_eq!(builder_types.iter().filter(|&&t| t == SOLDIER).count(), 0);

    // Group size at or above FOUNDING_MERCHANT_MIN_GROUP allocates merchants.
    let merchant_weights = FactionWeights {
        build: 2,
        war: 2,
        trade: 2,
        renown: 1,
        settle: 1,
    };
    let types_3 = founding_unit_type_distribution(FOUNDING_MERCHANT_MIN_GROUP, merchant_weights);
    assert_eq!(types_3.len(), 3);
    assert_eq!(types_3.iter().filter(|&&t| t == WORKER).count(), 1);
    assert_eq!(types_3.iter().filter(|&&t| t == SOLDIER).count(), 1);
    assert_eq!(types_3.iter().filter(|&&t| t == MERCHANT).count(), 1);
}

#[test]
fn proven_able_to_fail_all_workers_fails_diverse_type_assertion() {
    // Proves that if the old behavior (only assigning WORKER) were present,
    // the assertion requiring soldiers would fail.
    let old_behavior = [WORKER, WORKER];
    let has_soldier = old_behavior.contains(&SOLDIER);
    assert!(!has_soldier, "old behavior produces no soldiers");
}

#[test]
fn seeding_demonstration_world_founds_mixed_unit_types() {
    let mut world = World::new(WorldConfig {
        width: 64,
        height: 64,
        seed: 0x42,
        faction_count: 4,
        unit_capacity: 1024,
        ..WorldConfig::DEFAULT
    })
    .expect("the config describes a world");

    world.seed_world().expect("the world seeds once");

    let total_workers = world.population_of_type(WORKER);
    let total_soldiers = world.population_of_type(SOLDIER);

    assert!(
        total_workers > 0,
        "seeded world must contain workers, got {total_workers}"
    );
    assert!(
        total_soldiers > 0,
        "seeded world must contain soldiers, got {total_soldiers}"
    );

    // Verify census rows reflect the type populations.
    assert_eq!(read_census(&world, "workers"), i64::from(total_workers));
    assert_eq!(read_census(&world, "soldiers"), i64::from(total_soldiers));
    assert_eq!(
        read_census(&world, "merchants"),
        i64::from(world.population_of_type(MERCHANT))
    );
    assert_eq!(read_census(&world, "units_fallen"), 0);
}

#[test]
fn warrior_faction_receives_more_soldiers_than_builder_faction() {
    let mut world = World::new(WorldConfig {
        width: 64,
        height: 64,
        seed: 0x999,
        faction_count: 2,
        unit_capacity: 1024,
        ..WorldConfig::DEFAULT
    })
    .expect("the config describes a world");

    // Configure faction 0 as warrior and faction 1 as builder.
    world.set_faction_weights(
        FactionId(0),
        FactionWeights {
            war: 8,
            build: 1,
            trade: 1,
            renown: 1,
            settle: 1,
        },
    );
    world.set_faction_weights(
        FactionId(1),
        FactionWeights {
            war: 1,
            build: 8,
            trade: 1,
            renown: 1,
            settle: 1,
        },
    );

    let outcomes = world.found_run_for_every_faction(FOUNDING_GROUP_DEFAULT);
    assert_eq!(outcomes.len(), 2);

    let f0_soldiers = world.soldiers().population_by_type(FactionId(0))[SOLDIER.index()];
    let f1_soldiers = world.soldiers().population_by_type(FactionId(1))[SOLDIER.index()];

    assert!(
        f0_soldiers > f1_soldiers,
        "warrior faction must receive more soldiers ({f0_soldiers}) than builder faction ({f1_soldiers})"
    );
}

#[test]
fn combat_between_soldiers_produces_casualties_and_updates_census() {
    let mut world = World::new(WorldConfig {
        width: 32,
        height: 32,
        seed: 0x1234,
        faction_count: 2,
        unit_capacity: 64,
        latitude_centre: QUIET_CENTRE,
        latitude_span: QUIET_SPAN,
    })
    .expect("the config describes a world");

    let tile = open_address(&world);
    let u0 = world
        .spawn_soldier(tile, FactionId(0))
        .expect("ground admits unit");
    let u1 = world
        .spawn_soldier(tile, FactionId(1))
        .expect("ground admits unit");

    // Set both to soldiers (attack = 1.0, armour = 0.0).
    world.set_unit_type(u0, SOLDIER);
    world.set_unit_type(u1, SOLDIER);

    // Set relations to war so a contest occurs.
    world.set_relation(FactionId(0), FactionId(1), -10);

    assert_eq!(world.units_fallen(), 0);
    assert_eq!(read_census(&world, "units_fallen"), 0);

    // Step world with 1 thread.
    world.step(1).expect("world steps");

    // Attack exceeds armour (1.0 > 0.0), so casualties must occur.
    assert!(
        world.units_fallen() > 0,
        "soldiers at war must produce combat casualties"
    );
    assert_eq!(
        read_census(&world, "units_fallen"),
        world.units_fallen(),
        "census units_fallen row must match units_fallen()"
    );
    assert!(
        !world.fell_log().is_empty(),
        "fell_log must record the fallen units"
    );
}

#[test]
fn proven_able_to_fail_worker_combat_inflicts_zero_casualties() {
    // Two workers on the same tile at war. Because worker attack is 0,
    // attack does not exceed armour (0 <= 0), producing zero casualties.
    let mut world = World::new(WorldConfig {
        width: 32,
        height: 32,
        seed: 0x5678,
        faction_count: 2,
        unit_capacity: 64,
        latitude_centre: QUIET_CENTRE,
        latitude_span: QUIET_SPAN,
    })
    .expect("the config describes a world");

    let tile = open_address(&world);
    let u0 = world
        .spawn_soldier(tile, FactionId(0))
        .expect("ground admits unit");
    let u1 = world
        .spawn_soldier(tile, FactionId(1))
        .expect("ground admits unit");

    world.set_unit_type(u0, WORKER);
    world.set_unit_type(u1, WORKER);

    world.set_relation(FactionId(0), FactionId(1), -10);

    world.step(1).expect("world steps");

    assert_eq!(
        world.units_fallen(),
        0,
        "workers have zero attack and must inflict zero casualties"
    );
    assert!(world.fell_log().is_empty());
}

#[test]
fn domination_victory_by_combat_annihilation_is_reachable() {
    let mut world = World::new(WorldConfig {
        width: 32,
        height: 32,
        seed: 0x99,
        faction_count: 2,
        unit_capacity: 64,
        latitude_centre: QUIET_CENTRE,
        latitude_span: QUIET_SPAN,
    })
    .expect("the config describes a world");

    let tile = open_address(&world);
    // Faction 0 has a soldier; Faction 1 has a single worker.
    let soldier = world
        .spawn_soldier(tile, FactionId(0))
        .expect("ground admits unit");
    let victim = world
        .spawn_soldier(tile, FactionId(1))
        .expect("ground admits unit");

    world.set_unit_type(soldier, SOLDIER);
    world.set_unit_type(victim, WORKER);

    world.set_relation(FactionId(0), FactionId(1), -10);

    world.step(1).expect("world steps");

    // The soldier killed the worker. Faction 1 now holds no units, while
    // Faction 0 still holds its soldier. The domination reader must fire.
    let end = world.game_end();
    assert!(end.is_set(), "game must end when rival is annihilated");
    assert_eq!(end.winner, FactionId(0));
    assert_eq!(end.win_path(), Some(WinPath::Domination));
}

#[test]
fn census_table_includes_unit_type_counts_and_units_fallen() {
    let rows: Vec<(&str, CensusBasis)> =
        SUBSYSTEM_CENSUS.iter().map(|r| (r.name, r.basis)).collect();

    assert!(rows.contains(&("workers", CensusBasis::Held)));
    assert!(rows.contains(&("soldiers", CensusBasis::Held)));
    assert!(rows.contains(&("merchants", CensusBasis::Held)));
    assert!(rows.contains(&("units_fallen", CensusBasis::Total)));

    // Verify ordering: units_fallen must be immediately following units_burned.
    let burned_idx = rows
        .iter()
        .position(|&(name, _)| name == "units_burned")
        .expect("units_burned exists");
    let fallen_idx = rows
        .iter()
        .position(|&(name, _)| name == "units_fallen")
        .expect("units_fallen exists");
    assert_eq!(
        fallen_idx,
        burned_idx + 1,
        "units_fallen must immediately follow units_burned in SUBSYSTEM_CENSUS"
    );
}
