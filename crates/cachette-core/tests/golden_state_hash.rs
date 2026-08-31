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
//! [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
//! [^2]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`

use std::path::PathBuf;

use cachette_core::types::FactionId;
use cachette_core::{Axial, World, WorldConfig};

/// The number of frames that each scenario runs.
const FRAMES: u64 = 32;

/// The thread count that records the golden file. The thread-count
/// equivalence test proves that the count does not change the result.
const THREADS: usize = 4;

/// A named scenario. The name is the golden file name.
///
/// The third and fourth scenarios hold soldiers. The first two leave the
/// arena empty, so their hashes cover the tile columns and an empty arena
/// only. A golden file that never sees a populated arena cannot catch a
/// change to how the soldier state is represented, which is the whole purpose
/// of a golden file.[^1]
///
/// The fourth scenario is wider than the coarsest lattice spacing of the
/// generator, so its world holds water as well as open ground. The third is
/// narrower and holds no water at all. A suite of narrow worlds alone cannot
/// see a change to the rule that the ground refuses a unit, because no
/// soldier in it ever meets water.[^2] [^3]
///
/// # References
///
/// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
/// [^2]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D4, a draft record. `docs/adrs/draft/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
/// [^3]: Testing rules, section 2a. `.claude/rules/testing.md`
const SCENARIOS: &[(&str, WorldConfig, bool)] = &[
    (
        "small",
        WorldConfig {
            width: 16,
            height: 16,
            seed: 7,
            faction_count: 2,
        },
        false,
    ),
    (
        "default",
        WorldConfig {
            width: 64,
            height: 64,
            seed: 0x0123_4567_89ab_cdef,
            faction_count: 4,
        },
        false,
    ),
    (
        "soldiers",
        WorldConfig {
            width: 24,
            height: 24,
            seed: 0xfeed_face,
            faction_count: 3,
        },
        true,
    ),
    (
        "shoreline",
        WorldConfig {
            width: 96,
            height: 96,
            seed: 0x0cac_4e77_0068,
            faction_count: 3,
        },
        true,
    ),
];

/// Fills a world with soldiers, and frees some of them.
///
/// The frees matter. A run that only spawns never exercises the generation
/// advance, the free queue, or a reused slot, so the golden file would miss
/// a change to any of them.[^1]
///
/// # References
///
/// [^1]: ADR-0014, entity identity is an index plus a generation, decisions D3 and D4. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
fn populate(world: &mut World) {
    // The ground refuses a soldier on water, so the pattern walks the open
    // tiles in index order rather than the whole grid.[^2] The order is the
    // index order of the grid, which is fixed.[^3]
    let grid = world.grid();
    let open: Vec<Axial> = (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| world.admits_a_unit(*address))
        .collect();
    assert!(
        open.len() >= 64,
        "the scenario left only {} open tiles, too few to people",
        open.len()
    );
    let mut freed = Vec::new();
    for step in 0..64usize {
        let faction = FactionId((step % 3) as u16);
        let soldier = world
            .spawn_soldier(open[step * 7 % open.len()], faction)
            .expect("the spawn must succeed");
        if step % 3 == 0 {
            freed.push(soldier);
        }
    }
    for soldier in freed {
        assert!(world.despawn_soldier(soldier));
    }
    for step in 0..8usize {
        world
            .spawn_soldier(open[step * 11 % open.len()], FactionId(0))
            .expect("the respawn must reuse a freed slot");
    }
}

/// Returns the path of the golden file for one scenario.
fn golden_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join(format!("state-hash-{name}.txt"))
}

/// Runs a scenario and returns one hash line for each frame.
fn hash_sequence(config: WorldConfig, with_soldiers: bool) -> String {
    let mut world = World::new(config).expect("the extent must describe a world");
    if with_soldiers {
        populate(&mut world);
    }
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
    for (name, config, with_soldiers) in SCENARIOS {
        let produced = hash_sequence(*config, *with_soldiers);
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
    let mut world = World::new(SCENARIOS[1].1).expect("the extent must describe a world");
    let before = world.state_hash().finish();
    world.step(THREADS).expect("the step must run");
    assert_ne!(before, world.state_hash().finish());
}
