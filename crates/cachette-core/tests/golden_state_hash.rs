//! The golden state hash.
//!
//! The test hashes the whole simulation state each frame and compares the
//! sequence against a stored file.[^1] The padding rule stops the test from
//! failing falsely: an undeclared padding byte is uninitialised, and an
//! uninitialised byte enters the hash.[^2]
//!
//! To record a new golden file, set the environment variable
//! `CACHETTE_UPDATE_GOLDEN` to `1` and run the test. Read the difference
//! before you commit it. A changed golden file is a changed simulation.
//!
//! To add a scenario, add a row to `SCENARIOS` and record the file.
//!
//! # References
//!
//! [^1]: ADR-0001, Determinism as the primary constraint, decision D11. `docs/adrs/draft/adr-0001-determinism-outranks-every-other-constraint.md`
//! [^2]: ADR-0001, Determinism as the primary constraint, decision D9. `docs/adrs/draft/adr-0001-determinism-outranks-every-other-constraint.md`

use std::path::PathBuf;

use cachette_core::{World, WorldConfig};

/// The number of frames that each scenario runs.
const FRAMES: u64 = 32;

/// The thread count that records the golden file. The thread-count
/// equivalence test proves that the count does not change the result.
const THREADS: usize = 4;

/// A named scenario. The name is the golden file name.
const SCENARIOS: &[(&str, WorldConfig)] = &[
    (
        "small",
        WorldConfig {
            tile_count: 256,
            seed: 7,
            faction_count: 2,
        },
    ),
    (
        "default",
        WorldConfig {
            tile_count: 4_096,
            seed: 0x0123_4567_89ab_cdef,
            faction_count: 4,
        },
    ),
];

/// Returns the path of the golden file for one scenario.
fn golden_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join(format!("state-hash-{name}.txt"))
}

/// Runs a scenario and returns one hash line for each frame.
fn hash_sequence(config: WorldConfig) -> String {
    let mut world = World::new(config);
    let mut lines = String::new();
    lines.push_str(&format!("0 {}\n", world.state_hash()));
    for frame in 1..=FRAMES {
        world.step(THREADS).expect("the step must run");
        lines.push_str(&format!("{frame} {}\n", world.state_hash()));
    }
    lines
}

/// Reports whether the run must record the golden files.
fn recording() -> bool {
    std::env::var("CACHETTE_UPDATE_GOLDEN").as_deref() == Ok("1")
}

#[test]
fn the_state_hash_matches_the_golden_file() {
    for (name, config) in SCENARIOS {
        let produced = hash_sequence(*config);
        let path = golden_path(name);

        if recording() {
            std::fs::create_dir_all(path.parent().expect("the file has a parent"))
                .expect("the directory must exist");
            std::fs::write(&path, &produced).expect("the file must be written");
            continue;
        }

        let stored = std::fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!(
                "scenario {name}: cannot read {}: {error}. \
                 Set CACHETTE_UPDATE_GOLDEN=1 to record it.",
                path.display()
            )
        });

        assert_eq!(
            produced, stored,
            "scenario {name}: the state hash sequence changed. \
             A changed sequence is a changed simulation. Read the difference, \
             then set CACHETTE_UPDATE_GOLDEN=1 to record it."
        );
    }
}

#[test]
fn the_hash_changes_when_the_state_changes() {
    // A hash that never changes would pass the golden test and prove
    // nothing.
    let mut world = World::new(SCENARIOS[1].1);
    let before = world.state_hash().finish();
    world.step(THREADS).expect("the step must run");
    assert_ne!(before, world.state_hash().finish());
}
