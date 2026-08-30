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

use cachette_core::{World, WorldConfig};

/// The thread counts that every scenario runs at.
const THREAD_COUNTS: [usize; 3] = [1, 2, 12];

/// A named scenario. Add a row to cover a new case.
const SCENARIOS: &[(&str, WorldConfig, u64)] = &[
    (
        "one tile",
        WorldConfig {
            tile_count: 1,
            seed: 1,
            faction_count: 1,
        },
        1,
    ),
    (
        "fewer tiles than threads",
        WorldConfig {
            tile_count: 7,
            seed: 0xdead_beef,
            faction_count: 2,
        },
        4,
    ),
    (
        "an uneven split",
        WorldConfig {
            tile_count: 1_003,
            seed: 0x0123_4567_89ab_cdef,
            faction_count: 4,
        },
        8,
    ),
    (
        "many tiles",
        WorldConfig {
            tile_count: 65_536,
            seed: 42,
            faction_count: 16,
        },
        4,
    ),
];

/// Runs the frames and returns the log of the last frame and the state hash.
fn run(config: WorldConfig, frames: u64, threads: usize) -> (Vec<u8>, u64) {
    let mut world = World::new(config);
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
    let mut world = World::new(WorldConfig::default());
    assert!(world.step(0).is_err());
}
