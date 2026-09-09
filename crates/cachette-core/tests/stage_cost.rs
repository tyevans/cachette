//! Every stage the step declares is a stage the step opens.
//!
//! The stage cost table is a second listing of the frame. The enumeration
//! names the passes, and the step opens a span over each one. Two listings
//! that nothing compares are the defect shape this project meets most
//! often, and the two would disagree the first time somebody added a pass or
//! renamed one.[^1]
//!
//! This test derives one listing from the tree and compares. It drives one
//! frame through the public interface, then reads the table. A stage that
//! lost its span reports zero entries and the test fails by name.
//!
//! # This test can fail
//!
//! A determinism test with no proven failure mode is decoration, and the same
//! holds here.[^2] The proof is direct: delete one `stage::open` line from the
//! step and this test names that stage. That was checked by doing it.
//!
//! The two tests that name the holding stages carry their own proof. Put both
//! gathering positions of the holding rewrite under one label, then declare
//! two entries for that label. The general check stays green, because the
//! declaration and the step still agree. Both named tests fail. That was
//! checked by doing it.
//!
//! # It says nothing about time
//!
//! No assertion here reads a duration. The rule forbids it, and a duration
//! would be flaky on a loaded machine.[^3] The test asserts on entry counts,
//! which are integers the step controls.
//!
//! # References
//!
//! [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
//! [^2]: Testing rules, section 1. `.claude/rules/testing.md`
//! [^3]: Testing rules, section 3. `.claude/rules/testing.md`
#![cfg(feature = "stage-cost")]

use std::sync::{Mutex, MutexGuard};

use cachette_core::{stage, Axial, FactionId, Stage, World, WorldConfig, STAGES};

/// Serialises the tests that read the table.
///
/// The table is one static for the process, so two tests that step a world at
/// the same time add to the same counters and both read a total neither
/// produced. The test harness runs tests on several threads, so this is not a
/// hypothesis: the first version of this file failed that way, and the
/// counts it reported were double.
///
/// This lock is a fact about the instrument and not about the engine. Nothing
/// in the simulation reads the table.
static TABLE: Mutex<()> = Mutex::new(());

/// Takes the lock, and takes it even when another test left it poisoned.
fn alone() -> MutexGuard<'static, ()> {
    TABLE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The seed that every world in this file takes.
const SEED: u64 = 0x0123_4567_89ab_cdef;

/// Builds a small world and puts a few units in it.
fn world_with_units(units: u32) -> World {
    let config = WorldConfig {
        width: 64,
        height: 64,
        seed: SEED,
        faction_count: 4,
        unit_capacity: 1024,
    };
    let mut world = World::new(config).expect("the extent must describe a world");
    let mut placed = 0u32;
    let mut index = 0u32;
    while placed < units && index < world.grid().tile_count() {
        let address = Axial::new((index % 64) as i32, (index / 64) as i32);
        index += 1;
        if !world.admits_a_unit(address) {
            continue;
        }
        if world.spawn_soldier(address, FactionId(0)).is_ok() {
            placed += 1;
        }
    }
    world
}

#[test]
fn one_frame_opens_every_declared_stage_exactly_as_often_as_it_declares() {
    let _alone = alone();
    let mut world = world_with_units(32);
    // The first frame gives the spawns their barrier. The table is reset
    // after it, so the counts below belong to one settled frame.
    world.step(1).expect("the step must run");

    stage::reset();
    world.step(1).expect("the step must run");
    let costs = stage::costs();

    let mut wrong: Vec<String> = Vec::new();
    for stage in STAGES {
        let seen = costs.cost(*stage).entries;
        let declared = stage.entries_for_each_frame();
        if seen != declared {
            wrong.push(format!(
                "{}: declared {declared}, opened {seen}",
                stage.name()
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "the step and the stage list disagree: {}",
        wrong.join(", ")
    );
}

#[test]
fn the_table_accumulates_across_frames_and_a_reset_clears_it() {
    let _alone = alone();
    let mut world = world_with_units(32);
    world.step(1).expect("the step must run");

    stage::reset();
    for _ in 0..4 {
        world.step(1).expect("the step must run");
    }
    let after_four = stage::costs();
    assert_eq!(
        after_four.cost(Stage::TileScan).entries,
        4,
        "four frames must open the tile scan four times"
    );

    stage::reset();
    let after_reset = stage::costs();
    assert_eq!(
        after_reset.cost(Stage::TileScan).entries,
        0,
        "a reset must clear the entry count"
    );
    assert_eq!(
        after_reset.total_nanos(),
        0,
        "a reset must clear the time as well"
    );
}

#[test]
fn every_stage_has_a_name_of_its_own() {
    // The enumeration, the names and the threading answer all come from one
    // macro list, so a duplicate name is the one way two stages can still
    // collide. A register that reported two rows under one name would be
    // read as one row.
    let mut names: Vec<&str> = STAGES.iter().map(|stage| stage.name()).collect();
    let count = names.len();
    names.sort_unstable();
    names.dedup();
    assert_eq!(count, names.len(), "two stages share a name");
}

#[test]
fn a_build_without_the_feature_says_so() {
    // This test file only compiles under the feature, so the answer here is
    // fixed. It exists so that a reader of the table knows the question is
    // asked somewhere: a table of zeros means one of two things, and this
    // function is how a caller tells them apart.
    assert!(
        stage::is_recording(),
        "a build with the feature must record"
    );
}

/// The holding rewrite gathers the cities, then it gathers the candidate
/// tiles from those cities. Each position is its own stage.
///
/// One label over both walks would make the cost for each entry of that
/// label an average of two unrelated walks. The one before it reads the
/// settlement arena and the upgrade sites. The one after it reads the grid
/// around each city. The general check above compares every declared count
/// at once, and it would pass again if somebody merged the two positions
/// under one label and declared two entries. This test names the two
/// positions, so that merge fails here.
#[test]
fn the_rewrite_opens_one_stage_for_the_cities_and_one_for_the_candidates() {
    let _alone = alone();
    let mut world = world_with_units(32);
    world.step(1).expect("the step must run");

    stage::reset();
    world.step(1).expect("the step must run");
    let costs = stage::costs();

    assert_eq!(
        costs.cost(Stage::HoldingCities).entries,
        1,
        "one frame must gather the cities once"
    );
    assert_eq!(
        costs.cost(Stage::HoldingCandidates).entries,
        1,
        "one frame must gather the candidate tiles once"
    );
}

/// The two gathering stages open before the early return of the rewrite, and
/// the two working stages open after it.
///
/// The rewrite stops as soon as the candidate list is empty. A world with no
/// unit holds no city, so the list is empty and the rewrite decides nothing
/// and applies nothing. The gathering stages still open once each, because
/// the rewrite must gather the list to find that it is empty.
///
/// This test fixes which side of the early return each stage sits on. A
/// world that models the typical case supplies units, and it therefore
/// reaches neither position of the return.[^1] The counts below are what
/// tells a missing span apart from a pass that did no work.
///
/// # References
///
/// [^1]: Testing rules, a fixture supplies the input. `.claude/rules/testing.md`
#[test]
fn a_world_with_no_unit_gathers_the_candidates_and_decides_nothing() {
    let _alone = alone();
    let mut world = world_with_units(0);
    world.step(1).expect("the step must run");

    stage::reset();
    world.step(1).expect("the step must run");
    let costs = stage::costs();

    assert_eq!(
        costs.cost(Stage::HoldingCities).entries,
        1,
        "the rewrite must gather the cities before it can stop"
    );
    assert_eq!(
        costs.cost(Stage::HoldingCandidates).entries,
        1,
        "the rewrite must gather the candidate tiles before it can stop"
    );
    assert_eq!(
        costs.cost(Stage::HoldingDecide).entries,
        0,
        "an empty candidate list decides nothing"
    );
    assert_eq!(
        costs.cost(Stage::HoldingApply).entries,
        0,
        "an empty candidate list applies nothing"
    );
}

#[test]
fn the_observation_pass_divides_into_three_nested_stages() {
    // The observation pass held the whole cost of a frame in one row, and a
    // row for the whole pass cannot say which of the three passes spends it.
    // This test drives the engine, so a span that nothing reaches reports
    // zero entries rather than passing on a mechanism the step never opens.
    let _alone = alone();
    let mut world = world_with_units(32);
    world.step(1).expect("the step must run");

    stage::reset();
    world.step(1).expect("the step must run");
    let costs = stage::costs();

    for stage in [
        Stage::ObserveStamps,
        Stage::ObserveBlocks,
        Stage::ObserveApply,
    ] {
        assert_eq!(
            costs.cost(stage).entries,
            1,
            "one frame must open {} once",
            stage.name()
        );
        assert!(
            stage.is_nested(),
            "{} divides the observation pass, so the frame total must skip it",
            stage.name()
        );
    }
    assert!(
        !Stage::Observe.is_nested(),
        "the observation pass adds to the frame, so it is not nested"
    );
}

#[test]
fn a_nested_stage_stays_out_of_the_frame_total() {
    // The three nested rows sit inside the observation pass, so a total that
    // added them would count the same time twice. The check is a count and
    // not a duration: the total walks the stages that are not nested, and
    // this test states which stages that walk must leave out.
    let counted: Vec<&str> = STAGES
        .iter()
        .filter(|stage| !stage.is_nested())
        .map(|stage| stage.name())
        .collect();
    for name in ["observe_stamps", "observe_blocks", "observe_apply"] {
        assert!(
            !counted.contains(&name),
            "{name} divides the observation pass and must not be in the total"
        );
    }
    assert!(
        counted.contains(&"observe"),
        "the observation pass must be in the frame total"
    );
}
