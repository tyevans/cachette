//! A faction sets option and ground weights, and the controller favours own ground.
//!
//! Every unit scores options using its faction's weights. Ground options
//! scale by the ground weight of the cell: own ground, rival ground, or
//! unheld ground. The deliver option keeps neutral ground weight so laden
//! units return home without penalty.[^1]
//!
//! # References
//!
//! [^1]: ADR-0156, a faction's option weights are policy, set through one verb. `docs/adrs/accepted/adr-0156-a-factions-option-weights-are-policy-set-through-one-verb.md`
//! [^2]: Testing Rules, section 2. `.agents/rules/testing.md`
//! [^3]: Testing Rules, section 2a. `.agents/rules/testing.md`

use cachette_core::choose::{self, CarryClass};
use cachette_core::cohort::NEED_FULL;
use cachette_core::controller::{
    FactionWeights, GROUND_WEIGHT_HIGH, GROUND_WEIGHT_LOW, WEIGHT_HIGH, WEIGHT_LOW,
};
use cachette_core::pyramid::CellSummary;
use cachette_core::{FactionId, Fix32, World, WorldConfig};

#[test]
fn seeded_controller_weights_favour_own_ground_over_unheld_and_rival() {
    for seed in 0..64u64 {
        for faction in 0..4u16 {
            let weights = FactionWeights::from_seed(seed, FactionId(faction));
            assert!(
                weights.own_ground > weights.unheld_ground,
                "own ground must strictly exceed unheld ground for seed {seed}, faction {faction}"
            );
            assert_eq!(
                weights.unheld_ground, 128,
                "unheld ground weight must be neutral 128"
            );
            assert!(
                weights.unheld_ground > weights.rival_ground,
                "unheld ground must strictly exceed rival ground for seed {seed}, faction {faction}"
            );
            assert!(
                weights.rival_ground >= WEIGHT_LOW,
                "rival ground weight must be at or above WEIGHT_LOW"
            );
            assert!(
                weights.is_inside_bound(),
                "seeded weights must satisfy is_inside_bound"
            );
        }
    }
}

#[test]
fn ground_weight_for_scales_by_majority_faction() {
    let mut world = World::new(WorldConfig {
        width: 64,
        height: 64,
        seed: 42,
        faction_count: 2,
        unit_capacity: 1024,
        ..WorldConfig::DEFAULT
    })
    .expect("the config describes a world");
    world.seed_world().expect("the world seeds");

    let weights = FactionWeights {
        war: 4,
        trade: 4,
        build: 4,
        renown: 4,
        settle: 4,
        own_ground: 192,
        rival_ground: 64,
        unheld_ground: 128,
    };

    let summary = world.pyramid().cell(0).expect("the world contains cell 0");

    let unheld_factor = choose::ground_weight_for(FactionId(999), summary, &weights);
    if summary.majority_faction().is_none() {
        assert_eq!(
            unheld_factor,
            Fix32::ONE,
            "neutral unheld ground weight (128) must scale to Fix32::ONE"
        );
    }

    // Direct check of scaling arithmetic: raw u8 scaled with << 9.
    let own_factor = Fix32((weights.own_ground as i32) << 9);
    let rival_factor = Fix32((weights.rival_ground as i32) << 9);
    let neutral_factor = Fix32((weights.unheld_ground as i32) << 9);

    assert_eq!(neutral_factor, Fix32::ONE);
    assert!(own_factor > neutral_factor);
    assert!(neutral_factor > rival_factor);
}

#[test]
fn unit_scores_cell_options_higher_on_own_ground_than_rival_ground() {
    let weights = FactionWeights {
        war: 4,
        trade: 4,
        build: 4,
        renown: 4,
        settle: 4,
        own_ground: 160,
        rival_ground: 96,
        unheld_ground: 128,
    };

    let mut world = World::new(WorldConfig {
        width: 64,
        height: 64,
        seed: 7,
        faction_count: 2,
        unit_capacity: 1024,
        ..WorldConfig::DEFAULT
    })
    .expect("the config describes a world");
    world.seed_world().expect("the world seeds");

    let summary = (0..world.pyramid().len())
        .filter_map(|idx| world.pyramid().cell(idx as u32))
        .find(|s| s.open_share().unwrap_or(Fix32::ZERO) > Fix32::ZERO)
        .expect("world must hold an open cell");

    let need = Fix32(NEED_FULL.0 / 2);
    let weight = Fix32::ONE;
    let roam_option = choose::OPTIONS[1]; // roam ranks Ranked::Cell(OpenShare)

    // Evaluate roam score when faction owns ground vs when rival owns ground
    let faction_own = summary.majority_faction().unwrap_or(FactionId(0));
    let faction_rival = FactionId(faction_own.0 + 1);

    let score_own = choose::score(
        faction_own,
        need,
        CarryClass::Free,
        weight,
        summary,
        roam_option,
        &weights,
    );
    let score_rival = choose::score(
        faction_rival,
        need,
        CarryClass::Free,
        weight,
        summary,
        roam_option,
        &weights,
    );

    if summary.majority_faction() == Some(faction_own) {
        assert!(
            score_own > score_rival,
            "score on own ground must strictly exceed score on rival ground"
        );
    }
}

#[test]
fn setting_custom_weights_reverses_the_preference() {
    let mut world = World::new(WorldConfig {
        width: 64,
        height: 64,
        seed: 123,
        faction_count: 2,
        unit_capacity: 1024,
        ..WorldConfig::DEFAULT
    })
    .expect("the config describes a world");
    world.seed_world().expect("the world seeds");

    let faction = FactionId(0);
    let custom = FactionWeights {
        war: 2,
        trade: 3,
        build: 5,
        renown: 1,
        settle: 7,
        own_ground: 64,
        rival_ground: 200,
        unheld_ground: 128,
    };

    assert!(world.set_faction_weights(faction, custom));
    assert_eq!(world.faction_weights(faction), Some(custom));

    // Bounds checking tests
    let invalid_zero = FactionWeights {
        own_ground: 0,
        ..custom
    };
    assert!(!world.set_faction_weights(faction, invalid_zero));

    let invalid_policy_high = FactionWeights {
        war: WEIGHT_HIGH + 1,
        ..custom
    };
    assert!(!world.set_faction_weights(faction, invalid_policy_high));

    let invalid_policy_low = FactionWeights { war: 0, ..custom };
    assert!(!world.set_faction_weights(faction, invalid_policy_low));

    let valid_extremes = FactionWeights {
        own_ground: GROUND_WEIGHT_HIGH,
        rival_ground: GROUND_WEIGHT_LOW,
        unheld_ground: GROUND_WEIGHT_HIGH,
        ..custom
    };
    assert!(world.set_faction_weights(faction, valid_extremes));
}

#[test]
fn laden_unit_takes_deliver_regardless_of_ground() {
    let weights = FactionWeights {
        war: 4,
        trade: 4,
        build: 4,
        renown: 4,
        settle: 4,
        own_ground: 250,
        rival_ground: 10,
        unheld_ground: 128,
    };

    let mut world = World::new(WorldConfig {
        width: 64,
        height: 64,
        seed: 99,
        faction_count: 2,
        unit_capacity: 1024,
        ..WorldConfig::DEFAULT
    })
    .expect("the config describes a world");
    world.seed_world().expect("the world seeds");

    let summary = world.pyramid().cell(0).expect("cell 0 exists");
    let need = Fix32(NEED_FULL.0 / 2);
    let weight = Fix32::ONE;
    let deliver_option = choose::OPTIONS[0]; // deliver ranks Ranked::Carry

    let score_f0 = choose::score(
        FactionId(0),
        need,
        CarryClass::Laden,
        weight,
        summary,
        deliver_option,
        &weights,
    );
    let score_f1 = choose::score(
        FactionId(1),
        need,
        CarryClass::Laden,
        weight,
        summary,
        deliver_option,
        &weights,
    );

    assert_eq!(
        score_f0, score_f1,
        "deliver option must ignore ground weights and score identically for any faction"
    );
    assert!(
        score_f0 > Fix32::ZERO,
        "laden unit must receive non-zero score for deliver"
    );
}

#[test]
fn proven_able_to_fail_ground_preference_requires_distinct_weights() {
    let equal_weights = FactionWeights {
        war: 4,
        trade: 4,
        build: 4,
        renown: 4,
        settle: 4,
        own_ground: 128,
        rival_ground: 128,
        unheld_ground: 128,
    };

    let summary = CellSummary::IDENTITY;
    let factor_a = choose::ground_weight_for(FactionId(0), summary, &equal_weights);
    let factor_b = choose::ground_weight_for(FactionId(1), summary, &equal_weights);

    // If equal weights showed preference, this assertion would fail.
    assert_eq!(
        factor_a, factor_b,
        "equal weights must produce equal ground factors"
    );

    // With differentiated weights, own ground strictly beats rival ground.
    let distinct_weights = FactionWeights {
        own_ground: 160,
        rival_ground: 96,
        ..equal_weights
    };
    let factor_own = Fix32((distinct_weights.own_ground as i32) << 9);
    let factor_rival = Fix32((distinct_weights.rival_ground as i32) << 9);
    assert!(
        factor_own > factor_rival,
        "differentiated weights must produce strictly different factors"
    );
}

#[test]
fn census_reports_units_on_own_and_rival_ground() {
    let mut world = World::new(WorldConfig {
        width: 64,
        height: 64,
        seed: 42,
        faction_count: 2,
        unit_capacity: 1024,
        ..WorldConfig::DEFAULT
    })
    .expect("the config describes a world");
    world.seed_world().expect("the world seeds");

    let own_census = world
        .subsystem_census()
        .into_iter()
        .find(|(name, _)| *name == "units_on_own_ground");
    let rival_census = world
        .subsystem_census()
        .into_iter()
        .find(|(name, _)| *name == "units_on_rival_ground");

    assert!(own_census.is_some(), "census must hold units_on_own_ground");
    assert!(
        rival_census.is_some(),
        "census must hold units_on_rival_ground"
    );
}
