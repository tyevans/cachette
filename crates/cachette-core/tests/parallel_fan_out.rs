//! What the parallel fan-out creates, and what the engine asks it to create.
//!
//! Every parallel stage in the core divides its work into chunks and runs one
//! closure for each chunk. The chunk count never passes the thread count, so a
//! step at one thread gives every stage a single chunk. The fan-out runs the
//! last chunk on the calling thread, so a stage of one chunk creates no
//! operating system thread at all.
//!
//! # Why a counter and not an assertion on a result
//!
//! No assertion on a simulation result can see a thread creation. A stage that
//! spawns one thread and joins it at once gives the same answer as a stage
//! that runs inline. The saving is therefore invisible to every other test,
//! and a later change could put the spawn back without turning anything red.
//!
//! The fan-out counts its own thread creations behind a feature, and these
//! tests read that counter. The feature compiles to nothing when it is off, so
//! a normal build holds no atomic in the path of a stage.
//!
//! # Why the engine drives these tests
//!
//! A test that calls the fan-out directly proves that the fan-out works. It
//! does not prove that the engine reaches it. The engine is the party that is
//! obligated to avoid the spawn, so two tests below step a world and then read
//! the counter.[^1]
//!
//! # References
//!
//! [^1]: Testing Rules, drive the real caller. `.agents/rules/testing.md`

#![cfg(feature = "probe-spawn-count")]

use cachette_core::parallel::{fan_out, fan_out_each, reset_spawn_count, spawn_count};
use cachette_core::{FactionId, World, WorldConfig};

/// Serialises the tests that read the thread creation count.
///
/// The counter is one static for the whole process, and the test harness runs
/// the tests of one binary on several threads. Each test below takes this lock
/// before it resets the counter.
///
/// A test that fails poisons the lock. The tests below take the guard out of a
/// poisoned lock, so one failure reports one failure rather than five.
static COUNTER: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Builds the world the trainer builds, and seeds it.
fn trainer_world() -> World {
    let mut world = World::new(WorldConfig {
        width: 48,
        height: 48,
        seed: 0,
        faction_count: 3,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the shape must describe a world");
    world.seed_world().expect("the world must seed");
    world.set_win_readers_enabled(true);
    world.set_tick_limit(2500);
    world.set_externally_controlled(FactionId(0), true);
    world
}

#[test]
fn one_unit_of_work_creates_no_thread() {
    let guard = COUNTER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_spawn_count();
    let mut slot = 0u32;
    fan_out_each([|| slot = 7]);
    assert_eq!(slot, 7, "the one unit of work must have run");
    assert_eq!(
        spawn_count(),
        0,
        "one unit of work runs on the calling thread and creates no thread"
    );
    drop(guard);
}

#[test]
fn no_unit_of_work_creates_no_thread() {
    let guard = COUNTER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_spawn_count();
    let results: Vec<u32> = fan_out(Vec::<fn() -> u32>::new());
    assert!(results.is_empty(), "no unit of work gives no result");
    assert_eq!(spawn_count(), 0, "no unit of work creates no thread");
    drop(guard);
}

#[test]
fn many_units_of_work_create_one_thread_for_each_but_the_last() {
    let guard = COUNTER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_spawn_count();
    let results: Vec<usize> = fan_out((0..4usize).map(|index| move || index * 10));
    assert_eq!(
        results,
        vec![0, 10, 20, 30],
        "the results come back in unit order"
    );
    assert_eq!(
        spawn_count(),
        3,
        "four units of work spawn three threads and keep one for the caller"
    );
    drop(guard);
}

#[test]
fn a_step_at_one_thread_creates_no_thread() {
    let guard = COUNTER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut world = trainer_world();
    for _ in 0..40 {
        world.step(1).expect("the step must run");
    }
    reset_spawn_count();
    for _ in 0..40 {
        world.step(1).expect("the step must run");
    }
    assert_eq!(
        spawn_count(),
        0,
        "every stage of a step at one thread holds a single chunk"
    );
    drop(guard);
}

#[test]
fn a_step_at_twelve_threads_still_divides_the_work() {
    let guard = COUNTER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut world = trainer_world();
    for _ in 0..40 {
        world.step(12).expect("the step must run");
    }
    reset_spawn_count();
    world.step(12).expect("the step must run");
    let spawned = spawn_count();
    assert!(
        spawned > 0,
        "a step at twelve threads must still divide the work, and it spawned {spawned} threads"
    );
    drop(guard);
}
