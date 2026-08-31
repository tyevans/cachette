//! Thread-count equivalence.
//!
//! The test runs the same tick at one thread, at two threads and at twelve
//! threads. It compares the event log byte for byte. The record calls this
//! the highest-value test in the project.[^1]
//!
//! The test sees only the public crate API. It does not reach into an
//! internal module.[^2]
//!
//! To add a scenario, add a row to `SCENARIOS`. The test then runs the new
//! row at every thread count.
//!
//! # References
//!
//! [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
//! [^2]: Testing policy. `docs/TESTING.md`

use cachette_core::{Axial, Entity, FactionId, World, WorldConfig};

/// The thread counts that every scenario runs at.
const THREAD_COUNTS: [usize; 3] = [1, 2, 12];

/// A named scenario. Add a row to cover a new case.
const SCENARIOS: &[(&str, WorldConfig, u64)] = &[
    (
        "one tile",
        WorldConfig {
            width: 1,
            height: 1,
            seed: 1,
            faction_count: 1,
        },
        1,
    ),
    (
        "fewer tiles than threads",
        WorldConfig {
            width: 7,
            height: 1,
            seed: 0xdead_beef,
            faction_count: 2,
        },
        4,
    ),
    (
        "an uneven split",
        WorldConfig {
            width: 17,
            height: 59,
            seed: 0x0123_4567_89ab_cdef,
            faction_count: 4,
        },
        8,
    ),
    (
        "many tiles",
        WorldConfig {
            width: 256,
            height: 256,
            seed: 42,
            faction_count: 16,
        },
        4,
    ),
];

/// Runs the frames and returns the log of the last frame and the state hash.
fn run(config: WorldConfig, frames: u64, threads: usize) -> (Vec<u8>, u64) {
    let mut world = World::new(config).expect("the extent must describe a world");
    for _ in 0..frames {
        world.step(threads).expect("the step must run");
    }
    (
        world.event_log_bytes().to_vec(),
        world.state_hash().finish(),
    )
}

#[test]
fn the_event_log_is_identical_at_every_thread_count() {
    for (name, config, frames) in SCENARIOS {
        let (expected_log, expected_hash) = run(*config, *frames, THREAD_COUNTS[0]);
        for threads in &THREAD_COUNTS[1..] {
            let (log, hash) = run(*config, *frames, *threads);
            assert_eq!(
                log, expected_log,
                "scenario {name}: the event log differs at {threads} threads"
            );
            assert_eq!(
                hash, expected_hash,
                "scenario {name}: the state hash differs at {threads} threads"
            );
        }
    }
}

#[test]
fn the_log_is_not_empty_for_a_large_scenario() {
    // A test that compares two empty logs proves nothing. This test fails
    // if the stub stops emitting events.
    let (log, _) = run(SCENARIOS[3].1, 1, 4);
    assert!(!log.is_empty(), "the scenario must emit events");
}

#[test]
fn a_step_at_zero_threads_returns_an_error() {
    let mut world = World::new(WorldConfig::default()).expect("the extent must describe a world");
    assert!(world.step(0).is_err());
}

/// Fills a world with soldiers and returns the identities it kept.
///
/// The population is a fixed pattern, so it is the same on every run and at
/// every thread count. The function despawns part of what it spawns, so the
/// arena carries a free queue and a set of stale identities as well as a
/// live set.
fn populate(world: &mut World) -> Vec<Entity> {
    let grid = world.grid();
    let mut kept = Vec::new();
    let mut freed = Vec::new();
    // The pattern walks the tiles in index order and passes over the ground
    // that admits no unit. Water refuses a spawn, and which tiles hold water
    // is a property of the seed.[^1]
    //
    // [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D4, a draft record. `docs/adrs/draft/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
    let open: Vec<Axial> = (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| world.admits_a_unit(*address))
        .take(97)
        .collect();
    for (index, address) in open.into_iter().enumerate() {
        let index = index as u32;
        // The faction must be one the world has. This line read `index % 5`
        // against a scenario of two factions, so the suite spawned soldiers
        // of factions the world did not hold and the invariant check passed.
        let ceiling = u32::from(world.config().faction_count.max(1));
        let faction = FactionId((index % ceiling) as u16);
        let soldier = world
            .spawn_soldier(address, faction)
            .expect("the address and the faction must be valid");
        if index % 3 == 2 {
            freed.push(soldier);
        } else {
            kept.push(soldier);
        }
    }
    for soldier in &freed {
        assert!(world.despawn_soldier(*soldier));
    }
    kept
}

/// Runs the frames over a world that holds soldiers.
fn run_with_soldiers(config: WorldConfig, frames: u64, threads: usize) -> (Vec<u8>, u64, usize) {
    let mut world = World::new(config).expect("the extent must describe a world");
    let kept = populate(&mut world);
    for _ in 0..frames {
        world.step(threads).expect("the step must run");
    }
    // Drive the step, then inspect the arena. A column set that no test
    // reaches through the engine is inert.[^1]
    //
    // [^1]: Findings register, FND-041. `docs/FINDINGS.md`
    assert!(world.check_invariants());
    assert!(kept
        .iter()
        .all(|soldier| world.soldiers().contains(*soldier)));
    (
        world.event_log_bytes().to_vec(),
        world.state_hash().finish(),
        world.soldiers().len() as usize,
    )
}

#[test]
fn a_world_that_holds_soldiers_is_identical_at_every_thread_count() {
    for (name, config, frames) in SCENARIOS {
        let expected = run_with_soldiers(*config, *frames, THREAD_COUNTS[0]);
        assert!(
            expected.2 > 0,
            "scenario {name}: the world must hold soldiers"
        );
        for threads in &THREAD_COUNTS[1..] {
            let produced = run_with_soldiers(*config, *frames, *threads);
            assert_eq!(
                produced.0, expected.0,
                "scenario {name}: the event log differs at {threads} threads"
            );
            assert_eq!(
                produced.1, expected.1,
                "scenario {name}: the state hash differs at {threads} threads"
            );
            assert_eq!(
                produced.2, expected.2,
                "scenario {name}: the live soldier count differs at {threads} threads"
            );
        }
    }
}

#[test]
fn the_soldier_columns_reach_the_state_hash() {
    // A hash that ignored the soldier columns would pass the golden test
    // while the arena changed underneath it.
    let config = SCENARIOS[2].1;
    let bare = World::new(config).expect("the extent must describe a world");
    let mut peopled = World::new(config).expect("the extent must describe a world");
    populate(&mut peopled);
    assert_ne!(bare.state_hash(), peopled.state_hash());

    let mut moved = World::new(config).expect("the extent must describe a world");
    let kept = populate(&mut moved);
    assert_eq!(moved.state_hash(), peopled.state_hash());
    let corner = Axial::new(
        (moved.grid().width() - 1) as i32,
        (moved.grid().height() - 1) as i32,
    );
    assert_ne!(moved.soldiers().address(kept[0]), Some(corner));
    assert_eq!(moved.place_soldier(kept[0], corner), Ok(true));
    assert_ne!(moved.state_hash(), peopled.state_hash());
}
