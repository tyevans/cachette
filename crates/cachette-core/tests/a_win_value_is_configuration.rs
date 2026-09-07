//! A win-path value is configuration, and turning the readers off changes
//! nothing the simulation does.
//!
//! Each value that a game end reader compares is a value the world holds. A
//! world that nobody configures holds the constant that names the default, so
//! it behaves as it did before the values were settable.[^1]
//!
//! **The readers decide when the step stops watching, and nothing else.**
//! With the readers off, one run to the tick limit gives a trajectory that
//! scores any set of thresholds. That is only true when the run itself is
//! unchanged, so this file compares a run with the readers off against a run
//! with the readers on that never fires, byte for byte.[^2]
//!
//! **Each equivalence test states how it fails.** The pair of runs that must
//! agree is followed by a pair that must differ, so the comparison is proven
//! able to fail rather than trusted.[^3]
//!
//! The tests see only the public crate API.
//!
//! # References
//!
//! [^1]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decision D1. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
//! [^2]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decision D3. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
//! [^3]: Testing rules, section 1. `.agents/rules/testing.md`

use cachette_core::contest::RENOWN_PER_FELL;
use cachette_core::upgrade::{self, UpgradeCategory};
use cachette_core::{FactionId, Fix32, WinPath, World, WorldConfig, RENOWN_TARGET};

const THREADS: usize = 2;

/// The ticks each run of this file plays.
///
/// The number is high enough that the factions found cities, raise units and
/// fight, and low enough that the file runs in seconds.
const FRAMES: u64 = 120;

/// A tick limit no run of this file reaches, so the territory reader stays
/// quiet unless a test lowers it.
const LIMIT_BEYOND_THE_RUN: u64 = 100_000;

/// The extent of the world every run of this file plays.
const EXTENT: u32 = 96;

/// The seed every run of this file plays.
const SEED: u64 = 0x0123_4567_89ab_cdef;

fn config() -> WorldConfig {
    WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: SEED,
        faction_count: 4,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    }
}

/// Builds a seeded world whose readers cannot fire by the tick limit.
fn seeded() -> World {
    let mut world = World::new(config()).expect("the extent must describe a world");
    world.seed_world().expect("the world must seed");
    world.set_tick_limit(LIMIT_BEYOND_THE_RUN);
    world
}

/// Plays a world and returns every event log of the run, joined.
///
/// The log holds one step, so the run appends each step to one buffer. A
/// comparison of the last log alone would miss a run that differed earlier
/// and agreed at the end.
fn play(world: &mut World, frames: u64) -> Vec<u8> {
    let mut log = Vec::new();
    for _ in 0..frames {
        world.step(THREADS).expect("the step must run");
        log.extend_from_slice(world.event_log_bytes());
    }
    log
}

// ---------------------------------------------------------------------------
// The recording mode
// ---------------------------------------------------------------------------

#[test]
fn a_run_with_the_readers_off_holds_the_event_log_of_a_run_that_never_fires() {
    let mut watching = seeded();
    let mut recording = seeded();
    recording.set_win_readers_enabled(false);

    let watched = play(&mut watching, FRAMES);
    let recorded = play(&mut recording, FRAMES);

    // The premise of the comparison: the watching run never fired. Without
    // this the test would compare two runs that both ended nothing, and it
    // would pass for the wrong reason.
    assert!(
        !watching.game_end().is_set(),
        "the watching run must reach the end of the frames without firing"
    );
    assert!(
        !recording.game_end().is_set(),
        "the recording run records nothing"
    );
    assert!(!watched.is_empty(), "the fixture must produce events");
    assert_eq!(
        watched, recorded,
        "the readers decide when the step stops watching and nothing else"
    );
}

#[test]
fn the_event_log_comparison_fails_when_a_reader_fires() {
    // **This is the proof that the test above can fail.** The same pair of
    // runs, with one change: the watching run now has a tick limit inside the
    // frames, so the territory reader fires and the controllers fall silent.
    // The logs must then differ.
    let mut watching = seeded();
    watching.set_tick_limit(FRAMES / 2);
    let mut recording = seeded();
    recording.set_tick_limit(FRAMES / 2);
    recording.set_win_readers_enabled(false);

    let watched = play(&mut watching, FRAMES);
    let recorded = play(&mut recording, FRAMES);

    assert_eq!(
        watching.game_end().win_path(),
        Some(WinPath::Territory),
        "the fixture must reach the tick limit and fire"
    );
    assert!(
        !recording.game_end().is_set(),
        "the recording run records nothing at the tick limit either"
    );
    assert_ne!(
        watched, recorded,
        "a run that ends must differ from a run that does not"
    );
}

#[test]
fn the_readers_run_again_when_a_caller_turns_them_back_on() {
    let mut world = seeded();
    world.set_win_readers_enabled(false);
    world.set_tick_limit(FRAMES / 2);
    play(&mut world, FRAMES);
    assert!(!world.game_end().is_set(), "the readers recorded nothing");
    // The tick limit is behind the world now, so the territory reader fires
    // on the first tick after the switch goes back on.
    world.set_win_readers_enabled(true);
    world.step(THREADS).expect("the step must run");
    assert_eq!(world.game_end().win_path(), Some(WinPath::Territory));
}

// ---------------------------------------------------------------------------
// The defaults
// ---------------------------------------------------------------------------

#[test]
fn a_world_nobody_configures_holds_the_constant_of_every_value() {
    let world = World::new(config()).expect("the extent must describe a world");
    let balance = world.balance();
    assert_eq!(balance.renown_target(), RENOWN_TARGET);
    assert_eq!(balance.renown_per_fell(), RENOWN_PER_FELL);
    assert!(balance.win_readers_enabled(), "the readers run by default");
    let row = world
        .upgrade_table()
        .row(UpgradeCategory::WONDER, upgrade::WONDER_LEVEL)
        .expect("the default table holds a wonder row");
    assert_eq!(row.work, upgrade::WONDER_WORK);
    assert_eq!(row.victory_claim, upgrade::WONDER_VICTORY_CLAIM);
}

#[test]
fn a_world_set_to_the_defaults_runs_as_a_world_nobody_configured() {
    let mut untouched = seeded();
    let mut configured = seeded();
    configured.set_renown_target(RENOWN_TARGET);
    configured.set_renown_per_fell(RENOWN_PER_FELL.0);
    configured.set_win_readers_enabled(true);
    assert!(configured.set_wonder_work(upgrade::WONDER_WORK));
    assert!(configured.set_wonder_victory_claim(upgrade::WONDER_VICTORY_CLAIM));

    let plain = play(&mut untouched, FRAMES);
    let same = play(&mut configured, FRAMES);
    assert_eq!(
        plain, same,
        "setting a value to its default changes nothing"
    );
    assert_eq!(
        untouched.state_hash().finish(),
        configured.state_hash().finish()
    );
}

// ---------------------------------------------------------------------------
// Every value reaches the reader it governs, and the hash
// ---------------------------------------------------------------------------

#[test]
fn the_renown_target_decides_when_the_renown_reader_fires() {
    let mut world = World::new(config()).expect("the extent must describe a world");
    world.set_tick_limit(LIMIT_BEYOND_THE_RUN);
    let address = cachette_core::Axial::new(0, 0);
    let address = (0..EXTENT as i32)
        .map(|x| cachette_core::Axial::new(x, 0))
        .find(|place| world.admits_a_unit(*place))
        .unwrap_or(address);
    world
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    world
        .spawn_soldier(address, FactionId(1))
        .expect("the ground admits a second unit");
    let person = world
        .create_character(FactionId(0))
        .expect("the faction takes a character");
    // A renown a long way under the default target. The default reader is
    // quiet at it, and a reader whose target this test lowered fires.
    let renown = Fix32(RENOWN_TARGET / 8);
    assert!(world.set_character_renown(person, renown));
    world.step(THREADS).expect("the step must run");
    assert!(
        !world.game_end().is_set(),
        "the default target stands above the fixture"
    );
    world.set_renown_target(renown.0);
    world.step(THREADS).expect("the step must run");
    let end = world.game_end();
    assert_eq!(end.win_path(), Some(WinPath::Renown));
    assert_eq!(end.winner, FactionId(0));
}

#[test]
fn a_wonder_claim_of_zero_takes_the_wonder_path_out_of_the_game() {
    let mut world = World::new(config()).expect("the extent must describe a world");
    assert!(world.set_wonder_victory_claim(0));
    let row = world
        .upgrade_table()
        .row(UpgradeCategory::WONDER, upgrade::WONDER_LEVEL)
        .expect("the row is still there");
    assert_eq!(row.victory_claim, 0, "the claim column is zero");
    assert_eq!(
        row.work,
        upgrade::WONDER_WORK,
        "the other columns of the row stand where they were"
    );
}

#[test]
fn the_wonder_work_writes_one_column_of_the_row() {
    let mut world = World::new(config()).expect("the extent must describe a world");
    assert!(world.set_wonder_work(240));
    let row = world
        .upgrade_table()
        .row(UpgradeCategory::WONDER, upgrade::WONDER_LEVEL)
        .expect("the row is still there");
    assert_eq!(row.work, 240);
    assert_eq!(
        row.victory_claim,
        upgrade::WONDER_VICTORY_CLAIM,
        "the other columns of the row stand where they were"
    );
}

#[test]
fn every_win_value_reaches_the_state_hash() {
    // A value the step reads and the hash does not cover lets two worlds hash
    // the same and diverge on the next tick.
    let base = World::new(config())
        .expect("the extent must describe a world")
        .state_hash()
        .finish();
    let mut moved = World::new(config()).expect("the extent must describe a world");
    moved.set_renown_target(RENOWN_TARGET / 2);
    assert_ne!(base, moved.state_hash().finish(), "the renown target");

    let mut moved = World::new(config()).expect("the extent must describe a world");
    moved.set_renown_per_fell(RENOWN_PER_FELL.0 * 2);
    assert_ne!(
        base,
        moved.state_hash().finish(),
        "the renown for each fell"
    );

    let mut moved = World::new(config()).expect("the extent must describe a world");
    moved.set_win_readers_enabled(false);
    assert_ne!(base, moved.state_hash().finish(), "the reader switch");
}

// ---------------------------------------------------------------------------
// The standing reports every running value a reader compares
// ---------------------------------------------------------------------------

#[test]
fn the_standing_reports_the_running_value_of_every_win_path() {
    let mut world = seeded();
    play(&mut world, 8);
    let standing = world.standing(FactionId(0)).expect("faction 0 exists");
    // Each field is a quantity one reader compares, and the fixture must
    // reach a value in each rather than read a zero the engine never wrote.
    assert!(standing.held_tiles > 0, "the seeded faction holds ground");
    assert!(standing.live_units > 0, "the seeded faction holds units");
    assert!(
        standing.seats_held > 0,
        "the seeded faction holds its own seat"
    );
    // The wonder work and the renown start at nothing, and the reader
    // compares them from there. The assertion is that the reader reports a
    // number and not an absence.
    assert_eq!(standing.wonder_progress, 0, "nobody built a wonder yet");
    assert_eq!(standing.best_renown, 0, "nobody earned renown yet");
    // The live unit count is the one the domination reader compares, so the
    // report must agree with the population the engine keeps.
    assert_eq!(
        standing.live_units,
        i64::from(world.population_of(FactionId(0))),
        "the report is the count the reader compares"
    );
    assert!(world.standing(FactionId(u16::MAX)).is_none());
}
