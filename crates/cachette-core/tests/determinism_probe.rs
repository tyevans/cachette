//! The proof that the determinism tests can fail.
//!
//! A determinism test compares one run against another run. A test that
//! compares a run against itself always passes and proves nothing. This
//! file perturbs the engine behind a test-only feature and asserts that the
//! comparison then reports a difference.
//!
//! Run it with the feature on:
//!
//! ```text
//! cargo test --package cachette-core --features probe-nondeterminism \
//!     --test determinism_probe
//! ```
//!
//! The whole file compiles to nothing when the feature is off.
//!
//! The feature makes the step join its output slots in reverse order. At
//! one thread there is one slot, so the order does not change. At more than
//! one thread the order changes, and the event log changes with it. That is
//! exactly the defect that ADR-0004 D1 forbids.[^1]
//!
//! # References
//!
//! [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`, and ADR-0001, one binary gives one answer at any thread count, decision D5. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
#![cfg(feature = "probe-nondeterminism")]

use cachette_core::slots::{Candidate, Slots};
use cachette_core::{World, WorldConfig};

/// The scenario. It must hold more tiles than threads, so that a run at
/// twelve threads fills more than one output slot.
const CONFIG: WorldConfig = WorldConfig {
    width: 32,
    height: 32,
    seed: 0x0123_4567_89ab_cdef,
    faction_count: 4,
};

/// Runs one frame and returns the event log as bytes.
fn run(threads: usize) -> Vec<u8> {
    let mut world = World::new(CONFIG).expect("the extent must describe a world");
    world.step(threads).expect("the step must run");
    world.event_log_bytes().to_vec()
}

#[test]
fn the_thread_count_test_fails_when_the_order_rule_breaks() {
    let at_one = run(1);
    let at_twelve = run(12);
    assert!(!at_one.is_empty(), "the scenario must emit events");
    assert_ne!(
        at_one, at_twelve,
        "the probe did not perturb the event log, so the determinism test \
         has no proven failure mode"
    );
}

#[test]
fn the_perturbed_log_holds_the_same_events_in_a_different_order() {
    // The probe changes the order and nothing else. A probe that also
    // changed the content would prove less.
    let mut at_one = run(1);
    let mut at_twelve = run(12);
    assert_eq!(at_one.len(), at_twelve.len());
    at_one.sort_unstable();
    at_twelve.sort_unstable();
    assert_eq!(at_one, at_twelve);
}

/// Reduces the ranks to the lowest one, over the given number of threads.
///
/// Every rank is equal, so only the order decides which position wins. This
/// is the case that the slot rule exists for.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
fn tied_minimum(threads: usize) -> Option<Candidate<u32>> {
    let mut slots: Slots<Option<Candidate<u32>>> =
        Slots::filled(threads, None).expect("the thread count is not zero");
    for (index, slot) in slots.entries_mut().iter_mut().enumerate() {
        *slot = Some(Candidate::new(0, index as u32));
    }
    slots.minimum()
}

#[test]
fn the_slot_reduction_test_fails_when_the_order_rule_breaks() {
    // The probe reverses the combine order, so the highest slot now wins the
    // tie. The property test asserts the lowest slot wins, so it fails.
    assert_eq!(tied_minimum(1), Some(Candidate::new(0, 0)));
    assert_eq!(tied_minimum(12), Some(Candidate::new(0, 11)));
}
