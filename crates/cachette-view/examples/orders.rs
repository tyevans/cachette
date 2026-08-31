//! What the two orders cost, measured against each other.
//!
//! The engine holds two orders. The general order compares a key vector on
//! many threads. The bounded order runs a radix sort on one thread, and the
//! bridge rebuild uses it.
//!
//! This example runs both over one key set, in one process, one after the
//! other. A measurement of two runs on different days compares two machine
//! loads as much as it compares two algorithms. A measurement of both in one
//! process does not.
//!
//! It lives in the viewer crate because the viewer is allowed to read a
//! clock. The engine is not: a solver that stops on a time budget answers
//! differently on a loaded machine.[^1]
//!
//! Run it with `cargo run --release -p cachette-view --example orders`.
//!
//! Every figure it prints is a development machine, one run. That is a
//! smaller question than the blocker asks.[^2]
//!
//! # References
//!
//! [^1]: ADR-0005, a solver runs a fixed iteration count, never a convergence test, decision D2. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
//! [^2]: Blockers register, BLK-007. `docs/BLOCKERS.md`

// An example is its own crate, so the allowance at the viewer's crate root
// does not reach it. The reason is the same one: ADR-0067 D3 puts the float
// boundary at the viewer, and a mean of measured microseconds is a report,
// never a simulated value.
#![allow(clippy::disallowed_types)]

use cachette_core::sort::{self, BoundedKey, SortKey};
use cachette_view::Lap;

/// The number of keys in the set. It is near the unit count that the phase
/// report places on a world.
const KEYS: usize = 22_000;

/// The number of rounds that each figure averages over.
const ROUNDS: u32 = 40;

/// The ceiling of the ordering field. It is the key space of a world of
/// about a quarter of a million tiles.
const CEILING: u64 = 281_599;

/// A step that visits many tiles before it repeats.
const SPREAD: u64 = 9973;

/// Builds the bounded key set.
fn bounded() -> Vec<BoundedKey> {
    (0..KEYS as u64)
        .map(|index| BoundedKey::new(index.wrapping_mul(SPREAD) % (CEILING + 1), index))
        .collect()
}

/// Returns the same keys as a general key vector.
fn general(keys: &[BoundedKey]) -> Vec<SortKey<2>> {
    keys.iter()
        .map(|key| SortKey::new([key.order(), key.identifier()]))
        .collect()
}

/// Returns the mean cost of one call, in microseconds.
fn mean_micros(mut run: impl FnMut()) -> f64 {
    run();
    let at = Lap::start();
    for _ in 0..ROUNDS {
        run();
    }
    at.elapsed().as_secs_f64() * 1_000_000.0 / f64::from(ROUNDS)
}

fn main() {
    let threads = std::thread::available_parallelism()
        .map_or(1, std::num::NonZeroUsize::get)
        .min(12);

    let keys = bounded();
    let wide = general(&keys);

    // The two must agree. A cost figure for two orders that disagree means
    // nothing.
    let from_bounded = sort::order_bounded(&keys, CEILING).expect("the identifiers are unique");
    let from_general = sort::order_on(&wide, threads).expect("the identifiers are unique");
    assert_eq!(
        from_bounded, from_general,
        "the two orders must give one permutation"
    );

    println!("mean cost of one order over {KEYS} keys, {ROUNDS} rounds each");
    println!();
    println!("{:<44} {:>10}", "order", "us");

    for count in [1usize, 2, threads] {
        let us = mean_micros(|| {
            let _ = std::hint::black_box(sort::order_on(&wide, count));
        });
        println!("{:<44} {us:>10.0}", format!("general, {count} threads"));
    }
    let us = mean_micros(|| {
        let _ = std::hint::black_box(sort::order_bounded(&keys, CEILING));
    });
    println!("{:<44} {us:>10.0}", "bounded, one thread");

    println!();
    println!("The bounded order spawns no thread. The general order spawns");
    println!("one for each chunk, and the operating system charges for each.");
}
