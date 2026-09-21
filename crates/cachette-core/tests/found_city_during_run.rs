//! A faction founds a city during a run.
//!
//! A faction begins with its starting city and seats its initial units. When a
//! site holds the required store surplus and unheld eligible land exists within
//! survey reach, the controller queues a settler unit, sends it toward the
//! target, and executes the settle verb. Founding consumes the settler unit,
//! opens site rows for the new settlement, expands held ground, and leaves the
//! faction's primary seat tile immutable.[^1] [^2] [^3]
//!
//! # References
//!
//! [^1]: Backlog item 0514. `docs/backlog/complete/0514-let-a-faction-found-a-city-during-a-run.md`
//! [^2]: ADR-0150, held ground is the ground within reach of a city its faction owns, decisions D2 and D5. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
//! [^3]: ADR-0148, a game end is recorded once and stops the controllers, decision D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
//! [^4]: Testing rules, section 1, a determinism test must be able to fail. `.agents/rules/testing.md`

use cachette_core::hash::StateHash;
use cachette_core::site::CommodityId;
use cachette_core::types::{Entity, FactionId, Fix32};
use cachette_core::{World, WorldConfig};

/// The extent of the test worlds.
const EXTENT: u32 = 96;

/// Builds a test world with two factions.
fn world(seed: u64) -> World {
    World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        ..WorldConfig::DEFAULT
    })
    .expect("the extent must describe a valid world")
}

#[test]
fn a_faction_founds_a_second_city_during_a_run() {
    let mut field = world(101);
    let _ = field.found_run_for_every_faction(2);

    let faction = FactionId(0);
    let initial_sites = field.settlements().len();
    assert_eq!(
        initial_sites, 2,
        "initial seeding founds one site per faction"
    );

    let initial_seat = field
        .seat(faction)
        .expect("faction has a seat from first founding");

    // Provide the parent site with ample food and store goods to satisfy surplus.
    let site = field
        .settlements()
        .iter()
        .find(|s| field.settlements().faction(*s) == Some(faction))
        .expect("faction owns a site");

    field
        .set_settlement_store(site, CommodityId(0), Fix32::from_int(100))
        .expect("commodity 0 is valid");

    let initial_held = field.holding_of(faction);
    let mut secondary_founded = false;

    // Run the step until the controller queues, builds, walks, and settles a second city.
    for _tick in 0..250 {
        field.step(1).expect("step must execute");
        if field.settlements().len() > initial_sites {
            secondary_founded = true;
            break;
        }
    }

    assert!(
        secondary_founded,
        "controller must autonomously queue, walk, and settle a second city"
    );

    // Verify invariants:
    // 1. Primary seat remains immutable.
    assert_eq!(
        field.seat(faction),
        Some(initial_seat),
        "primary seat tile must remain unchanged on secondary founding"
    );

    // 2. New settlement opens site slots (positions, rates, queues).
    let new_site_count = field.settlements().len();
    assert_eq!(new_site_count, initial_sites + 1);

    let new_site = field
        .settlements()
        .iter()
        .find(|s| *s != site && field.settlements().faction(*s) == Some(faction))
        .expect("new site belongs to faction");

    let new_slot = field
        .settlements()
        .slot_of(new_site)
        .expect("new site has a slot");
    assert!(
        field.site_queue(new_site).is_some(),
        "new site has queue slots opened"
    );
    assert_eq!(new_slot, new_site_count - 1);

    // 3. Settler unit was consumed.
    let settler_count = field.settler_count(faction).unwrap_or(0);
    assert_eq!(settler_count, 0, "founding consumes the settler unit");

    // 4. Held tiles expand.
    field.step(1).expect("step updates holding rewrite");
    let after_held = field.holding_of(faction);
    assert!(
        after_held > initial_held,
        "secondary city reach must expand the faction's held ground"
    );

    assert!(field.check_invariants(), "all world invariants hold");
}

#[test]
fn secondary_city_founding_is_deterministic_across_threads() {
    let run = |threads: usize| -> (StateHash, Vec<Entity>) {
        let mut field = world(202);
        let _ = field.found_run_for_every_faction(2);

        let faction = FactionId(0);
        let site = field
            .settlements()
            .iter()
            .find(|s| field.settlements().faction(*s) == Some(faction))
            .expect("faction owns a site");

        field
            .set_settlement_store(site, CommodityId(0), Fix32::from_int(100))
            .expect("commodity 0 is valid");

        for _ in 0..80 {
            field.step(threads).expect("step must execute");
        }

        let sites: Vec<Entity> = field.settlements().iter().collect();
        (field.state_hash(), sites)
    };

    let (hash_1, sites_1) = run(1);
    let (hash_2, sites_2) = run(2);

    assert_eq!(
        hash_1, hash_2,
        "world state hash must match byte-for-byte between 1 thread and 2 threads"
    );
    assert_eq!(
        sites_1, sites_2,
        "settlement entities must match identically across thread counts"
    );
}

#[test]
fn the_determinism_test_can_fail() {
    // Testing rules section 1: a determinism test must be able to fail.
    let mut field_a = world(303);
    let _ = field_a.found_run_for_every_faction(2);
    field_a.step(1).expect("step must execute");

    let mut field_b = world(304);
    let _ = field_b.found_run_for_every_faction(2);
    field_b.step(1).expect("step must execute");

    assert_ne!(
        field_a.state_hash(),
        field_b.state_hash(),
        "different seeds must yield distinct state hashes"
    );
}
