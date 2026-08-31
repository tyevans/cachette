//! Properties of the key vector sort.
//!
//! Content supplies an ordered vector of exact integer key fields. Content
//! never supplies a comparison function.[^1] The last field of every key is a
//! stable identifier, so no two items tie and the sort has exactly one correct
//! output.[^2] The engine never calls content code from inside a sort.[^3]
//!
//! Every generated key below ties in every field except the last. A key vector
//! that ties nowhere would hide the defect that the stable identifier
//! prevents.
//!
//! Proptest prints the failing input and writes it to the regressions file
//! beside this test, so a reader can repeat a failure.[^4]
//!
//! The test sees only the public crate API.[^4]
//!
//! # References
//!
//! [^1]: ADR-0007, content supplies a key vector, never a comparator, decision D1. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
//! [^2]: ADR-0007, content supplies a key vector, never a comparator, decision D2. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
//! [^3]: ADR-0007, content supplies a key vector, never a comparator, decision D3. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
//! [^4]: Testing policy. `docs/TESTING.md`

use cachette_core::sort::{self, SortError, SortKey};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

/// The number of fields in the generated key.
const FIELDS: usize = 3;

/// The thread counts that every property runs at.
const THREAD_COUNTS: [usize; 3] = [1, 2, 12];

/// A key set that ties in every field except the stable identifier.
///
/// The first two fields come from a narrow range, so the set ties in them
/// almost always. The last field is the position, which is unique.
fn tied_keys() -> impl Strategy<Value = Vec<SortKey<FIELDS>>> {
    prop::collection::vec((0u64..3, 0u64..3), 1..200).prop_map(|rows| {
        rows.into_iter()
            .enumerate()
            .map(|(position, (first, second))| SortKey::new([first, second, position as u64]))
            .collect()
    })
}

/// Reports whether the order is one exact permutation of the indices.
fn is_a_permutation(order: &[u32], count: usize) -> bool {
    let mut seen = vec![false; count];
    for index in order {
        let slot = *index as usize;
        if slot >= count || seen[slot] {
            return false;
        }
        seen[slot] = true;
    }
    order.len() == count
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
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/key_vector_sort.proptest-regressions"),
        ))),
        ..ProptestConfig::default()
    })]
    /// The order is identical at every thread count.
    #[test]
    fn the_order_does_not_depend_on_the_thread_count(keys in tied_keys()) {
        let expected = sort::order_on(&keys, THREAD_COUNTS[0]).expect("the identifiers are unique");
        for threads in &THREAD_COUNTS[1..] {
            let found = sort::order_on(&keys, *threads).expect("the identifiers are unique");
            prop_assert_eq!(found, expected.clone());
        }
    }

    /// The output is one exact permutation, and it rises in key order.
    #[test]
    fn the_output_is_one_rising_permutation(keys in tied_keys()) {
        for threads in THREAD_COUNTS {
            let order = sort::order_on(&keys, threads).expect("the identifiers are unique");
            prop_assert!(is_a_permutation(&order, keys.len()));
            for pair in order.windows(2) {
                let left = keys[pair[0] as usize];
                let right = keys[pair[1] as usize];
                prop_assert!(left < right, "the order does not rise: {:?} then {:?}", left, right);
            }
        }
    }

    /// The order does not depend on the input order.
    ///
    /// The property shuffles the key set, sorts both, and compares the keys
    /// that come out. The identifiers are unique, so exactly one output is
    /// correct.
    #[test]
    fn the_order_does_not_depend_on_the_input_order(keys in tied_keys(), rotation in 0usize..200) {
        let mut rotated = keys.clone();
        rotated.rotate_left(rotation % keys.len());

        let straight = sort::order(&keys).expect("the identifiers are unique");
        let shuffled = sort::order(&rotated).expect("the identifiers are unique");

        let straight_keys: Vec<SortKey<FIELDS>> =
            straight.iter().map(|index| keys[*index as usize]).collect();
        let shuffled_keys: Vec<SortKey<FIELDS>> =
            shuffled.iter().map(|index| rotated[*index as usize]).collect();
        prop_assert_eq!(straight_keys, shuffled_keys);
    }

    /// Sorting the items agrees with sorting the keys.
    #[test]
    fn the_items_follow_their_keys(keys in tied_keys()) {
        let items: Vec<u64> = keys.iter().map(SortKey::identifier).collect();
        for threads in THREAD_COUNTS {
            let sorted = sort::sorted(&items, &keys, threads).expect("the identifiers are unique");
            let order = sort::order_on(&keys, threads).expect("the identifiers are unique");
            let expected: Vec<u64> = order.iter().map(|index| items[*index as usize]).collect();
            prop_assert_eq!(sorted, expected);
        }
    }
}

#[test]
fn a_repeated_identifier_is_an_error() {
    // ADR-0007 D2: the last field is a stable identifier, so it is unique.
    let keys = [
        SortKey::new([1, 7]),
        SortKey::new([2, 7]),
        SortKey::new([0, 9]),
    ];
    assert_eq!(sort::order(&keys), Err(SortError::RepeatedIdentifier(7)));
}

#[test]
fn a_unique_identifier_is_accepted() {
    let keys = [SortKey::new([1, 8]), SortKey::new([1, 7])];
    assert_eq!(sort::order(&keys), Ok(vec![1, 0]));
}

#[test]
fn a_sort_at_zero_threads_is_an_error() {
    let keys = [SortKey::new([1, 1])];
    assert_eq!(sort::order_on(&keys, 0), Err(SortError::ZeroThreads));
}

#[test]
fn a_length_mismatch_is_an_error() {
    let keys = [SortKey::new([1, 1]), SortKey::new([1, 2])];
    let items = [9u8];
    assert_eq!(
        sort::sorted(&items, &keys, 1),
        Err(SortError::LengthMismatch { items: 1, keys: 2 })
    );
}

#[test]
fn an_empty_set_sorts_to_an_empty_order() {
    let keys: [SortKey<2>; 0] = [];
    assert_eq!(sort::order_on(&keys, 4), Ok(Vec::new()));
}

#[test]
fn a_descending_field_orders_from_high_to_low() {
    // The order is always ascending. Content inverts the field instead.
    let keys = [
        SortKey::new([sort::descending(1), 10]),
        SortKey::new([sort::descending(3), 11]),
        SortKey::new([sort::descending(2), 12]),
    ];
    let order = sort::order(&keys).expect("the identifiers are unique");
    let ranks: Vec<u64> = order
        .iter()
        .map(|index| keys[*index as usize].identifier())
        .collect();
    assert_eq!(ranks, vec![11, 12, 10]);
}

#[test]
fn a_signed_field_orders_by_its_signed_value() {
    let keys = [
        SortKey::new([sort::from_signed(0), 10]),
        SortKey::new([sort::from_signed(-5), 11]),
        SortKey::new([sort::from_signed(4), 12]),
    ];
    let order = sort::order(&keys).expect("the identifiers are unique");
    let ranks: Vec<u64> = order
        .iter()
        .map(|index| keys[*index as usize].identifier())
        .collect();
    assert_eq!(ranks, vec![11, 10, 12]);
}

#[test]
fn a_key_reports_its_fields_and_its_identifier() {
    let key = SortKey::new([4, 5, 6]);
    assert_eq!(key.fields(), &[4, 5, 6]);
    assert_eq!(key.identifier(), 6);
}
