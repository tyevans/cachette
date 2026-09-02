//! Properties of the slot reduction.
//!
//! A minimum, a maximum and a first-wins are not order-free. Each depends on
//! the order when two values tie. Each therefore writes into a slot indexed by
//! a stable key, and the combine step reads the slots in index order.[^1]
//!
//! Every property below runs the same reduction at one thread, at two threads
//! and at twelve threads. The generated ranks come from a small range, so the
//! input holds many ties. A tie is where the order shows.[^2]
//!
//! The last property drives the world step, which is the engine caller of the
//! mechanism. A test that only builds the mechanism proves that the mechanism
//! works, not that anything reaches it.[^3]
//!
//! The test sees only the public crate API.[^4]
//!
//! # References
//!
//! [^1]: ADR-0004, iteration order is explicit, decision D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
//! [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
//! [^3]: Testing policy, drive the real caller. `docs/TESTING.md`
//! [^4]: Testing policy. `docs/TESTING.md`

use cachette_core::slots::{Candidate, SlotError, Slots};
use cachette_core::{World, WorldConfig};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

/// The thread counts that every property runs at.
const THREAD_COUNTS: [usize; 3] = [1, 2, 12];

/// Splits the ranks over the threads and reduces to the lowest rank.
///
/// Each thread reduces its own chunk into its own slot. The combine step then
/// reads the slots. The payload is the position of the rank in the input, so a
/// tie is visible in the answer.
fn parallel_minimum(ranks: &[i64], threads: usize) -> Option<Candidate<u32>> {
    let mut slots: Slots<Option<Candidate<u32>>> =
        Slots::filled(threads, None).expect("the thread count is not zero");
    let chunk_len = ranks.len().div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        let mut base = 0u32;
        for (chunk, slot) in ranks.chunks(chunk_len).zip(slots.entries_mut()) {
            let start = base;
            base += chunk.len() as u32;
            scope.spawn(move || {
                let mut best: Option<Candidate<u32>> = None;
                for (offset, rank) in chunk.iter().enumerate() {
                    let candidate = Candidate::new(*rank, start + offset as u32);
                    let wins = match best {
                        None => true,
                        Some(current) => candidate.rank < current.rank,
                    };
                    if wins {
                        best = Some(candidate);
                    }
                }
                *slot = best;
            });
        }
    });
    slots.minimum()
}

/// Splits the ranks over the threads and reduces to the highest rank.
fn parallel_maximum(ranks: &[i64], threads: usize) -> Option<Candidate<u32>> {
    let mut slots: Slots<Option<Candidate<u32>>> =
        Slots::filled(threads, None).expect("the thread count is not zero");
    let chunk_len = ranks.len().div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        let mut base = 0u32;
        for (chunk, slot) in ranks.chunks(chunk_len).zip(slots.entries_mut()) {
            let start = base;
            base += chunk.len() as u32;
            scope.spawn(move || {
                let mut best: Option<Candidate<u32>> = None;
                for (offset, rank) in chunk.iter().enumerate() {
                    let candidate = Candidate::new(*rank, start + offset as u32);
                    let wins = match best {
                        None => true,
                        Some(current) => candidate.rank > current.rank,
                    };
                    if wins {
                        best = Some(candidate);
                    }
                }
                *slot = best;
            });
        }
    });
    slots.maximum()
}

/// Splits the ranks over the threads and takes the first rank above a bound.
///
/// This is the first-wins reduction. It is the case where every candidate is
/// equally good, so only the order decides.
fn parallel_first_above(ranks: &[i64], bound: i64, threads: usize) -> Option<u32> {
    let mut slots: Slots<Option<u32>> =
        Slots::filled(threads, None).expect("the thread count is not zero");
    let chunk_len = ranks.len().div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        let mut base = 0u32;
        for (chunk, slot) in ranks.chunks(chunk_len).zip(slots.entries_mut()) {
            let start = base;
            base += chunk.len() as u32;
            scope.spawn(move || {
                *slot = chunk
                    .iter()
                    .position(|rank| *rank > bound)
                    .map(|offset| start + offset as u32);
            });
        }
    });
    slots.first_wins()
}

/// A rank vector that holds many ties.
///
/// The range is narrow against the length, so the input ties in almost every
/// run. A vector of distinct ranks would hide the ordering defect.
fn tied_ranks() -> impl Strategy<Value = Vec<i64>> {
    prop::collection::vec(-3i64..4, 1..200)
}

proptest! {
    #![proptest_config(ProptestConfig {
        // An integration test has no lib.rs or main.rs above it, so the
        // default source-parallel persistence finds no root and silently
        // disables itself. A failing seed is then never written and never
        // replayed. Name the file, so that a seed which caught a defect runs
        // first on every later run.[^1]
        //
        // [^1]: Findings register, FND-044. `docs/FINDINGS.md`
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/slot_reduction.proptest-regressions"),
        ))),
        ..ProptestConfig::default()
    })]
    /// The minimum is identical at every thread count.
    #[test]
    fn the_minimum_does_not_depend_on_the_thread_count(ranks in tied_ranks()) {
        let expected = parallel_minimum(&ranks, THREAD_COUNTS[0]);
        for threads in &THREAD_COUNTS[1..] {
            prop_assert_eq!(parallel_minimum(&ranks, *threads), expected);
        }
    }

    /// The minimum is the lowest rank, and the lowest position wins a tie.
    ///
    /// This pins the answer to a value that the test computes without the
    /// mechanism. A property that only compares the mechanism against itself
    /// passes when the mechanism reads the slots backwards.
    #[test]
    fn the_lowest_position_wins_a_tie_for_the_minimum(ranks in tied_ranks()) {
        let lowest = *ranks.iter().min().expect("the vector is not empty");
        let position = ranks
            .iter()
            .position(|rank| *rank == lowest)
            .expect("the lowest rank is in the vector");
        for threads in THREAD_COUNTS {
            let found = parallel_minimum(&ranks, threads).expect("the vector is not empty");
            prop_assert_eq!(found.rank, lowest);
            prop_assert_eq!(found.payload, position as u32);
        }
    }

    /// The maximum is the highest rank, and the lowest position wins a tie.
    #[test]
    fn the_lowest_position_wins_a_tie_for_the_maximum(ranks in tied_ranks()) {
        let highest = *ranks.iter().max().expect("the vector is not empty");
        let position = ranks
            .iter()
            .position(|rank| *rank == highest)
            .expect("the highest rank is in the vector");
        for threads in THREAD_COUNTS {
            let found = parallel_maximum(&ranks, threads).expect("the vector is not empty");
            prop_assert_eq!(found.rank, highest);
            prop_assert_eq!(found.payload, position as u32);
        }
    }

    /// First-wins takes the lowest position, at every thread count.
    #[test]
    fn first_wins_takes_the_lowest_position(ranks in tied_ranks(), bound in -4i64..4) {
        let expected = ranks
            .iter()
            .position(|rank| *rank > bound)
            .map(|position| position as u32);
        for threads in THREAD_COUNTS {
            prop_assert_eq!(parallel_first_above(&ranks, bound, threads), expected);
        }
    }
}

#[test]
fn a_reduction_over_zero_slots_is_an_error() {
    let built: Result<Slots<Option<u32>>, SlotError> = Slots::filled(0, None);
    assert_eq!(built, Err(SlotError::ZeroSlots));
}

#[test]
fn an_empty_reduction_holds_no_candidate() {
    let slots: Slots<Option<Candidate<u32>>> =
        Slots::filled(4, None).expect("the slot count is not zero");
    assert_eq!(slots.count(), 4);
    assert_eq!(slots.minimum(), None);
    assert_eq!(slots.maximum(), None);
    assert_eq!(slots.first_wins(), None);
}

#[test]
fn the_combine_step_reads_the_slots_in_index_order() {
    let mut slots: Slots<u32> = Slots::filled(4, 0).expect("the slot count is not zero");
    for (index, slot) in slots.entries_mut().iter_mut().enumerate() {
        *slot = index as u32;
    }
    let joined = slots.combine(Vec::new(), |mut carried, slot| {
        carried.push(*slot);
        carried
    });
    assert_eq!(joined, vec![0, 1, 2, 3]);
    assert_eq!(slots.entries(), &[0, 1, 2, 3]);
}

#[test]
fn the_world_step_joins_its_slots_through_the_reduction() {
    // The engine is obligated to invoke the mechanism, so this test starts at
    // the engine. The step is the caller.
    let config = WorldConfig {
        width: 64,
        height: 64,
        seed: 0x0123_4567_89ab_cdef,
        faction_count: 4,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    };
    let mut expected: Option<Vec<u8>> = None;
    for threads in THREAD_COUNTS {
        let mut world = World::new(config).expect("the extent must describe a world");
        for _ in 0..4 {
            world.step(threads).expect("the step must run");
        }
        let log = world.event_log_bytes().to_vec();
        assert!(!log.is_empty(), "the scenario must emit events");
        match &expected {
            None => expected = Some(log),
            Some(first) => assert_eq!(&log, first, "the log differs at {threads} threads"),
        }
    }
}
