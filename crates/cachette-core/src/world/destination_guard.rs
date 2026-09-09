//! The guarded destination field equals the field a full derivation gives.
//!
//! # What this holds
//!
//! The engine steers a sent unit from a coarse field over level 1 cells and a
//! fine field over the tiles of a block. Both come from the seed set of each
//! destination plane, the crossing of each plane, and the ground. The step
//! derives both only when a send changed a seed set or a crossing.
//!
//! **A skipped derivation is an optimisation and it must give the answer a
//! full derivation gives.** The record that permits an incremental update
//! states that condition, and this test is what checks it.[^1]
//!
//! A stale field is wrong behaviour that repeats. The thread count test and
//! the stored state hash both pass on it, because the run still repeats.[^2]
//!
//! # How a reader proves that this test can fail
//!
//! One test-only switch removes the mark that the send verb writes. The step
//! then derives the field once and never again, so the field describes the
//! seed set of the first frame for the whole run.
//!
//! ```text
//! ! cargo test --package cachette-core --features probe-frozen-destinations \
//!     --lib destination_guard
//! ```
//!
//! # The fixture
//!
//! The world is large enough that a send crosses a level 1 cell, and it runs
//! its own controller. The controller sends on its own schedule, so the run
//! holds frames that change a seed set and frames that change none. The test
//! counts both, and it fails when it met only one kind.[^3]
//!
//! # References
//!
//! [^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
//! [^2]: Testing rules, section 2. `.agents/rules/testing.md`
//! [^3]: Testing rules, section 2a. `.agents/rules/testing.md`

use super::config::WorldConfig;
use super::World;

/// How many tiles the world holds on a side.
const EXTENT: u32 = 192;

/// How many factions the world holds.
const FACTIONS: u16 = 3;

/// How many frames the test drives.
const FRAMES: u64 = 250;

/// Builds a seeded world whose factions the controller drives.
fn world() -> World {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: 0,
        faction_count: FACTIONS,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the shape must describe a world");
    world.seed_world().expect("the world must seed");
    world.set_win_readers_enabled(true);
    world.set_tick_limit(2500);
    world
}

/// The field the step leaves equals the field a full derivation gives, on
/// every frame of a long run.
///
/// The test clones the world after each step and derives the clone from the
/// seed set it holds. The clone reaches the unguarded derivation, so the two
/// fields come from two different paths.
#[test]
fn the_guarded_field_equals_a_full_derivation_on_every_frame() {
    let mut world = world();
    let mut kept = 0usize;
    let mut derived = 0usize;
    let mut previous = world.destination_seeds.clone();
    let mut crossings = world.destination_crossings.clone();
    for frame in 0..FRAMES {
        world.step(1).expect("the step must run");
        let mut fresh = world.clone();
        fresh.derive_destination_fields();
        assert!(
            world.destinations == fresh.destinations,
            "frame {frame} left a coarse field that a full derivation changes"
        );
        assert!(
            world.approaches == fresh.approaches,
            "frame {frame} left a fine field that a full derivation changes"
        );
        if world.destination_seeds == previous && world.destination_crossings == crossings {
            kept += 1;
        } else {
            derived += 1;
        }
        previous = world.destination_seeds.clone();
        crossings = world.destination_crossings.clone();
    }
    // **The fixture must reach both branches.** A run whose seed sets never
    // changed would pass the assertions above without the guard ever holding
    // a field that a send had to replace.
    assert!(
        derived > 0,
        "no seed set changed in {FRAMES} frames, so this test measured its fixture"
    );
    assert!(
        kept > 0,
        "every frame of {FRAMES} changed a seed set, so the guard skipped nothing"
    );
}
