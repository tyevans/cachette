//! Two factions meet when units pass within line of sight of each other,
//! or within line of sight of a city.
//!
//! # Why this exists
//!
//! Factions cannot interact, trade, or declare war until they have met in the
//! world. A unit or city of one faction must be observed within line of sight
//! by a unit of the other faction before diplomacy opens between them.[^1]
//!
//! # What this tests
//!
//! 1. Factions start with no contact with each other.
//! 2. Moving relations before meeting is refused.
//! 3. Units passing within line of sight of each other record contact.
//! 4. A unit passing within line of sight of a settlement records contact.
//! 5. The controller does not pick an unmet faction as a rival or prey.
//!
//! # References
//!
//! [^1]: ADR-0205, two factions meet when a unit has line of sight to another unit or a city. `docs/adrs/draft/adr-0205-two-factions-meet-when-a-unit-has-line-of-sight-to-another-unit-or-a-city.md`
//! [^2]: Testing rules, section 2. `.agents/rules/testing.md`

use cachette_core::relation::RelationError;
use cachette_core::unit_type::{LEADER, WORKER};
use cachette_core::{Axial, FactionId, MoveRelationError, World, WorldConfig};

const A: FactionId = FactionId(0);
const B: FactionId = FactionId(1);

fn test_world(width: u32, height: u32) -> World {
    World::new(WorldConfig {
        width,
        height,
        seed: 1,
        faction_count: 2,
        unit_capacity: 64,
        latitude_centre: -7000,
        latitude_span: 3000,
    })
    .expect("a world configuration is valid")
}

#[test]
fn unmet_factions_cannot_move_relations() {
    let mut world = test_world(20, 20);
    let leader_a = world
        .spawn_soldier(Axial::new(0, 0), A)
        .expect("spawns on valid ground");
    assert!(world.set_unit_type(leader_a, LEADER));

    assert!(!world.has_met(A, B));
    assert!(!world.has_met(B, A));
    assert!(world.has_met(A, A));

    let err = world.move_relation(leader_a, B, -1);
    assert_eq!(
        err,
        Err(MoveRelationError::Relation(RelationError::UnmetFaction(1))),
        "unmet factions cannot move relations"
    );
}

#[test]
fn units_passing_within_line_of_sight_meet() {
    let mut world = test_world(30, 30);
    let leader_a = world
        .spawn_soldier(Axial::new(0, 0), A)
        .expect("spawns on valid ground");
    assert!(world.set_unit_type(leader_a, LEADER));

    // Unit B is spawned far away (distance 20 hexes, well outside sight radius 4).
    let worker_b = world
        .spawn_soldier(Axial::new(20, 0), B)
        .expect("spawns on valid ground");
    assert!(world.set_unit_type(worker_b, WORKER));

    // Step once to compute observations.
    world.step(1).expect("step runs");

    assert!(
        !world.has_met(A, B),
        "factions are out of sight and remain unmet"
    );

    // Spawn a scout of A within line of sight of worker B (distance 2 hexes).
    let scout_a = world
        .spawn_soldier(Axial::new(18, 0), A)
        .expect("spawns on valid ground");
    assert!(world.set_unit_type(scout_a, WORKER));

    // Step once so the observation pass sees worker B.
    world.step(1).expect("step runs");

    assert!(world.has_met(A, B), "scout of A saw worker of B");
    assert!(world.has_met(B, A), "contact is symmetric");

    // Now leader A can move relations with B.
    assert!(world.move_relation(leader_a, B, -1).is_ok());
}

#[test]
fn a_unit_passing_within_line_of_sight_of_a_settlement_meets_the_holder() {
    let mut world = test_world(30, 30);
    let leader_a = world
        .spawn_soldier(Axial::new(0, 0), A)
        .expect("spawns on valid ground");
    assert!(world.set_unit_type(leader_a, LEADER));

    // Found a settlement for B at (20, 0).
    let city_b = world
        .found_settlement(Axial::new(20, 0), B)
        .expect("founds settlement");
    assert!(world.settlements().address(city_b).is_some());

    world.step(1).expect("step runs");
    assert!(!world.has_met(A, B), "settlement is far out of sight");

    // Unit of A arrives within line of sight of the city (distance 2 hexes).
    let scout_a = world
        .spawn_soldier(Axial::new(18, 0), A)
        .expect("spawns on valid ground");
    assert!(world.set_unit_type(scout_a, WORKER));

    world.step(1).expect("step runs");
    assert!(
        world.has_met(A, B),
        "unit of A saw the city of B and established contact"
    );
    assert!(world.has_met(B, A));
}

#[test]
fn unmet_factions_do_not_interact_via_controller() {
    let mut world = test_world(40, 40);
    let leader_a = world
        .spawn_soldier(Axial::new(0, 0), A)
        .expect("spawns on valid ground");
    assert!(world.set_unit_type(leader_a, LEADER));

    let leader_b = world
        .spawn_soldier(Axial::new(35, 35), B)
        .expect("spawns on valid ground");
    assert!(world.set_unit_type(leader_b, LEADER));

    for _ in 0..10 {
        world.step(1).expect("step runs");
    }

    assert!(!world.has_met(A, B));
    assert_eq!(world.relation(A, B), Some(0));
    assert_eq!(world.relation(B, A), Some(0));
}
