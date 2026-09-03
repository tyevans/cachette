//! A unit arena whose slot order has drifted away from the tile order.
//!
//! The movement pass walks every live unit in cell order, which the bridge
//! holds, and not in the slot order of the arena.[^1] The two orders hold the
//! same units. They agree only while the arena still carries the units in the
//! order it was filled in, and a slot never moves, so the arena drifts away
//! from the tile order as units die and slots return.[^2]
//!
//! **A fixture that spawns in tile order does not reach this case.** Slot
//! order is then cell order, the two walks agree, and a test over such a
//! world measures its fixture rather than the engine.[^3] Every world below
//! spawns in a permuted order, so the two walks disagree, and one test
//! asserts that they do.
//!
//! The test sees only the public crate interface.[^4]
//!
//! # References
//!
//! [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D1. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
//! [^2]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
//! [^3]: Testing rules, section 2a. `.claude/rules/testing.md`
//! [^4]: Testing rules, section 6. `.claude/rules/testing.md`

use cachette_core::{Axial, Entity, FactionId, World, WorldConfig};

/// The thread counts that every scenario runs at.
const THREAD_COUNTS: [usize; 3] = [1, 2, 12];

/// The world that every test below builds.
const CONFIG: WorldConfig = WorldConfig {
    width: 192,
    height: 192,
    seed: 0x5eed_0266_0266_5eed,
    faction_count: 4,
    unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
};

/// The number of frames that a scenario runs.
const FRAMES: u64 = 6;

/// Permutes a list in place from a fixed seed.
///
/// The draw is a linear congruential step. It seeds no simulated state and
/// reaches no state hash, so it is a fixture and not a simulation draw.[^1]
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
fn shuffle<T>(items: &mut [T]) {
    let mut state: u64 = 0x9e37_79b9_7f4a_7c15;
    for index in (1..items.len()).rev() {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let pick = (state >> 33) as usize % (index + 1);
        items.swap(index, pick);
    }
}

/// Returns the open addresses of the world, in a permuted order.
fn drifted_addresses(world: &World) -> Vec<Axial> {
    let grid = world.grid();
    let mut open: Vec<Axial> = (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| world.admits_a_unit(*address))
        .step_by(7)
        .collect();
    shuffle(&mut open);
    open
}

/// Builds a world whose arena order disagrees with its tile order.
fn drifted_world() -> World {
    let mut world = World::new(CONFIG).expect("the extent must describe a world");
    let addresses = drifted_addresses(&world);
    let ceiling = u32::from(CONFIG.faction_count);
    for (ordinal, address) in addresses.iter().enumerate() {
        let faction = FactionId((ordinal as u32 % ceiling) as u16);
        world
            .spawn_soldier(*address, faction)
            .expect("the open address admits a unit");
    }
    world
}

#[test]
fn the_fixture_reaches_the_drifted_case() {
    // Put the case the test exists for in front of the assertion. A world
    // whose slot order already is its cell order would pass every test below
    // without the engine reading the bridge order at all.
    let mut world = drifted_world();
    world.step(1).expect("the step must run");

    let soldiers = world.soldiers();
    let in_slot_order: Vec<Entity> = soldiers.iter().collect();
    let in_cell_order: Vec<Entity> = world
        .bridge()
        .units(soldiers)
        .expect("the bridge describes the arena after a step")
        .to_vec();

    assert!(
        in_slot_order.len() > 100,
        "the fixture must hold enough units to order"
    );
    assert_eq!(
        in_slot_order.len(),
        in_cell_order.len(),
        "the two orders hold the same units"
    );
    let mut sorted_slots = in_slot_order.clone();
    let mut sorted_cells = in_cell_order.clone();
    sorted_slots.sort_unstable_by_key(|unit: &Entity| unit.to_bits());
    sorted_cells.sort_unstable_by_key(|unit: &Entity| unit.to_bits());
    assert_eq!(
        sorted_slots, sorted_cells,
        "the two orders hold the same set"
    );
    assert_ne!(
        in_slot_order, in_cell_order,
        "the fixture must decorrelate the arena from the tile order"
    );
}

#[test]
fn the_bridge_order_is_the_cell_order() {
    let mut world = drifted_world();
    world.step(1).expect("the step must run");

    let soldiers = world.soldiers();
    let layout = world.bridge().layout();
    let units = world
        .bridge()
        .units(soldiers)
        .expect("the bridge describes the arena after a step");

    let mut previous: Option<u64> = None;
    for unit in units {
        let tile = soldiers.tile(*unit).expect("a listed unit is live");
        let key = layout.key_of(tile).expect("a tile of this world has a key");
        if let Some(before) = previous {
            assert!(
                before <= key,
                "the bridge holds the units in nondecreasing key order"
            );
        }
        previous = Some(key);
    }
}

#[test]
fn a_stale_bridge_refuses_the_unit_order() {
    // The order names units, so it is a guarded read. A structural change
    // that has not passed a barrier must make it refuse rather than answer
    // from the last rebuild.
    let mut world = drifted_world();
    world.step(1).expect("the step must run");
    let address = drifted_addresses(&world)
        .into_iter()
        .find(|address| world.admits_a_unit(*address))
        .expect("the world holds open ground");
    world
        .spawn_soldier(address, FactionId(0))
        .expect("the open address admits a unit");
    assert!(
        world.bridge().units(world.soldiers()).is_err(),
        "a bridge that no longer describes the arena refuses to name its units"
    );
}

#[test]
fn a_drifted_world_is_identical_at_every_thread_count() {
    let expected = run(THREAD_COUNTS[0]);
    for threads in &THREAD_COUNTS[1..] {
        let taken = run(*threads);
        assert_eq!(
            taken.0, expected.0,
            "the event log differs at {threads} threads"
        );
        assert_eq!(
            taken.1, expected.1,
            "the state hash differs at {threads} threads"
        );
    }
}

/// Runs the frames and returns the log of the last frame and the state hash.
fn run(threads: usize) -> (Vec<u8>, u64) {
    let mut world = drifted_world();
    for _ in 0..FRAMES {
        world.step(threads).expect("the step must run");
    }
    assert!(world.check_invariants());
    (
        world.event_log_bytes().to_vec(),
        world.state_hash().finish(),
    )
}
