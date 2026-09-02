//! The public API of the core crate.
//!
//! This test goes through the front door. It sees only what a user of the
//! crate sees. It does not reach into an internal module.[^1]
//!
//! # References
//!
//! [^1]: Testing policy. `docs/TESTING.md`

use cachette_core::rng;
use cachette_core::{Axial, FactionId, SoldierError, World, WorldConfig};

#[test]
fn a_new_world_holds_its_invariants() {
    let world = World::new(WorldConfig::default()).expect("the extent must describe a world");
    assert!(world.check_invariants());
    assert_eq!(world.tick().0, 0);
    assert_eq!(world.tile_count(), world.grid().tile_count() as usize);
    assert!(world.event_log().is_empty());
}

#[test]
fn a_step_advances_the_tick_and_holds_the_invariants() {
    let mut world = World::new(WorldConfig::default()).expect("the extent must describe a world");
    world.step(2).expect("the step must run");
    assert_eq!(world.tick().0, 1);
    assert!(world.check_invariants());
}

#[test]
fn a_copy_of_the_tile_column_holds_one_value_for_each_tile() {
    // ADR-0044: what copies and what does not is declared at the call site.
    // The world holds no array of tile values, so the call that returns the
    // whole column is named for the copy it makes.
    let world = World::new(WorldConfig::default()).expect("the extent must describe a world");
    assert_eq!(world.copy_tile_values().len(), world.tile_count());
}

#[test]
fn two_worlds_with_one_seed_agree() {
    let config = WorldConfig::default();
    let mut first = World::new(config).expect("the extent must describe a world");
    let mut second = World::new(config).expect("the extent must describe a world");
    for _ in 0..8 {
        first.step(1).expect("the step must run");
        second.step(7).expect("the step must run");
    }
    assert_eq!(first.state_hash(), second.state_hash());
}

#[test]
fn two_worlds_with_two_seeds_differ() {
    let mut first = World::new(WorldConfig {
        seed: 1,
        ..WorldConfig::default()
    })
    .expect("the extent must describe a world");
    let mut second = World::new(WorldConfig {
        seed: 2,
        ..WorldConfig::default()
    })
    .expect("the extent must describe a world");
    first.step(1).expect("the step must run");
    second.step(1).expect("the step must run");
    assert_ne!(first.state_hash(), second.state_hash());
}

#[test]
fn the_generator_holds_no_state() {
    // ADR-0003 D1: the draw is a function of its key. Calling it twice with
    // one key gives one value.
    let first = rng::draw(7, rng::SYSTEM_TILE_STUB, 3, 11, 0);
    let second = rng::draw(7, rng::SYSTEM_TILE_STUB, 3, 11, 0);
    assert_eq!(first, second);
    assert_ne!(first, rng::draw(7, rng::SYSTEM_TILE_STUB, 3, 11, 1));
    assert_ne!(first, rng::draw(7, rng::SYSTEM_TILE_STUB, 4, 11, 0));
    assert_ne!(first, rng::draw(8, rng::SYSTEM_TILE_STUB, 3, 11, 0));
}

#[test]
fn the_generator_stays_inside_the_bound() {
    for entity in 0..512u64 {
        let value = rng::draw_below(1, rng::SYSTEM_TILE_STUB, 0, entity, 0, 6);
        assert!(value < 6);
    }
    assert_eq!(rng::draw_below(1, rng::SYSTEM_TILE_STUB, 0, 0, 0, 0), 0);
}

#[test]
fn the_generator_gives_the_known_answers() {
    // ADR-0001 D4 requires known-answer tests, because the project writes
    // the mixer instead of taking it from a dependency. Record a new value
    // only when the mixer changes on purpose.
    assert_eq!(rng::draw(0, 1, 0, 0, 0), 0x1957_a760_4e21_5178);
    assert_eq!(rng::draw(1, 1, 1, 1, 1), 0xda03_81b1_529a_cb69);
}

#[test]
fn the_event_log_and_its_bytes_agree() {
    let mut world = World::new(WorldConfig {
        width: 64,
        height: 64,
        ..WorldConfig::default()
    })
    .expect("the extent must describe a world");
    world.step(3).expect("the step must run");
    let events = world.event_log();
    assert!(!events.is_empty(), "the scenario must emit events");
    assert_eq!(world.event_log_bytes().len(), size_of_val(events));
    assert!(events.iter().all(|event| event.tick == world.tick()));
}

#[test]
fn the_tile_total_is_the_sum_of_the_column() {
    use cachette_core::sim_math;
    use cachette_core::types::Accum;

    let world = World::new(WorldConfig {
        width: 32,
        height: 16,
        ..WorldConfig::default()
    })
    .expect("the extent must describe a world");
    let expected = world
        .copy_tile_values()
        .iter()
        .fold(Accum(0), |total, value| sim_math::accumulate(total, *value));
    assert_eq!(world.tile_total(), expected);
    assert_ne!(world.tile_total(), Accum(0));
}

#[test]
fn the_change_kind_matches_the_direction_of_the_change() {
    use cachette_core::event::{CHANGE_KIND_LOWERED, CHANGE_KIND_RAISED};

    let mut world = World::new(WorldConfig {
        width: 64,
        height: 32,
        ..WorldConfig::default()
    })
    .expect("the extent must describe a world");
    let before = world.copy_tile_values();
    world.step(2).expect("the step must run");
    let after = world.copy_tile_values();

    let mut raised = 0;
    let mut lowered = 0;
    for event in world.event_log() {
        let index = event.tile.0 as usize;
        assert_eq!(event.value, after[index]);
        match event.kind {
            CHANGE_KIND_RAISED => {
                assert!(after[index].0 > before[index].0);
                raised += 1;
            }
            CHANGE_KIND_LOWERED => {
                assert!(after[index].0 < before[index].0);
                lowered += 1;
            }
            other => panic!("an event carries an unknown change kind: {other}"),
        }
    }
    assert!(raised > 0, "the scenario must raise a tile");
    assert!(lowered > 0, "the scenario must lower a tile");
}

#[test]
fn a_soldier_survives_a_step_and_the_world_holds_its_invariants() {
    // FND-041: a column set that no system reaches is inert. The test drives
    // the step and then inspects the arena.
    let mut world = World::new(WorldConfig::default()).expect("the extent must describe a world");
    let soldier = world
        .spawn_soldier(Axial::new(2, 3), FactionId(1))
        .expect("the address and the faction must be valid");
    assert_eq!(world.soldiers().len(), 1);
    world.step(4).expect("the step must run");
    assert!(world.check_invariants());
    assert!(world.soldiers().contains(soldier));
    // The step moves each soldier to a neighbour, or leaves it in place when
    // the chosen neighbour falls outside the world. The address therefore
    // stays within one tile of the spawn.[^1]
    //
    // [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
    let address = world
        .soldiers()
        .address(soldier)
        .expect("the soldier is alive");
    assert!(Axial::new(2, 3).distance(address) <= 1);

    assert!(world.despawn_soldier(soldier));
    assert!(!world.soldiers().contains(soldier));
    assert!(!world.despawn_soldier(soldier));
    assert!(world.check_invariants());
}

#[test]
fn the_world_refuses_a_soldier_that_no_tile_holds() {
    let mut world = World::new(WorldConfig::default()).expect("the extent must describe a world");
    let outside = Axial::new(-1, 0);
    assert_eq!(
        world.spawn_soldier(outside, FactionId(0)),
        Err(SoldierError::TileOutsideWorld(outside))
    );
    assert!(world.soldiers().is_empty());
}
