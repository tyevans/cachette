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

use cachette_core::sort::{self, BoundedKey, SortError, SortKey};
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

/// The ceiling of every generated bounded key set.
///
/// The value is small, so a generated set ties in the ordering field often.
/// A set that never ties would hide the step that removes the input order.
const BOUNDED_CEILING: u64 = 300_000;

/// A bounded key set that ties in the ordering field often.
///
/// The ordering field comes from a range far narrower than the ceiling, so
/// two keys share it often. The identifier counts down from the set size, so
/// it is unique and it runs against the input order. An identifier that rose
/// with the input order would hide a sort that keeps the input order.
fn tied_bounded_keys() -> impl Strategy<Value = Vec<BoundedKey>> {
    prop::collection::vec(0u64..8, 1..200).prop_map(|rows| {
        let count = rows.len() as u64;
        rows.into_iter()
            .enumerate()
            .map(|(position, order)| BoundedKey::new(order, count - position as u64))
            .collect()
    })
}

/// A bounded key set that spreads over the whole ceiling.
///
/// The wide set exercises every radix pass. The narrow set above exercises
/// only the lowest.
fn wide_bounded_keys() -> impl Strategy<Value = Vec<BoundedKey>> {
    prop::collection::vec(0u64..=BOUNDED_CEILING, 1..200).prop_map(|rows| {
        let count = rows.len() as u64;
        rows.into_iter()
            .enumerate()
            .map(|(position, order)| BoundedKey::new(order, count - position as u64))
            .collect()
    })
}

/// Returns the same keys as a general key vector.
fn as_general(keys: &[BoundedKey]) -> Vec<SortKey<2>> {
    keys.iter()
        .map(|key| SortKey::new([key.order(), key.identifier()]))
        .collect()
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

    /// The bounded order gives what the general order gives.
    ///
    /// The two are separate algorithms over one definition of the order. If
    /// they ever disagree, one is wrong and neither may be trusted.
    #[test]
    fn the_bounded_order_agrees_with_the_general_order(keys in tied_bounded_keys()) {
        let general = sort::order(&as_general(&keys)).expect("the identifiers are unique");
        let bounded = sort::order_bounded(&keys, BOUNDED_CEILING)
            .expect("the identifiers are unique and the keys are inside the ceiling");
        prop_assert_eq!(bounded, general);
    }

    /// The bounded order agrees with the general order over the whole range.
    #[test]
    fn the_bounded_order_agrees_over_the_whole_range(keys in wide_bounded_keys()) {
        let general = sort::order(&as_general(&keys)).expect("the identifiers are unique");
        let bounded = sort::order_bounded(&keys, BOUNDED_CEILING)
            .expect("the identifiers are unique and the keys are inside the ceiling");
        prop_assert_eq!(bounded, general);
    }

    /// The bounded order is one exact permutation of the indices.
    #[test]
    fn the_bounded_order_is_one_exact_permutation(keys in tied_bounded_keys()) {
        let order = sort::order_bounded(&keys, BOUNDED_CEILING)
            .expect("the identifiers are unique and the keys are inside the ceiling");
        prop_assert!(is_a_permutation(&order, keys.len()));
    }

    /// The bounded order does not depend on the input order.
    ///
    /// A radix pass is stable, so it keeps the input order for keys that tie.
    /// The sort replaces that order with the order of the identifiers. Delete
    /// that step and this property fails, because the rotated set then keeps
    /// the rotated order.
    #[test]
    fn the_bounded_order_does_not_depend_on_the_input_order(
        keys in tied_bounded_keys(),
        rotation in 0usize..200,
    ) {
        let mut rotated = keys.clone();
        rotated.rotate_left(rotation % keys.len());

        let straight = sort::order_bounded(&keys, BOUNDED_CEILING)
            .expect("the identifiers are unique and the keys are inside the ceiling");
        let shuffled = sort::order_bounded(&rotated, BOUNDED_CEILING)
            .expect("the identifiers are unique and the keys are inside the ceiling");

        let straight_keys: Vec<BoundedKey> =
            straight.iter().map(|index| keys[*index as usize]).collect();
        let shuffled_keys: Vec<BoundedKey> =
            shuffled.iter().map(|index| rotated[*index as usize]).collect();
        prop_assert_eq!(straight_keys, shuffled_keys);
    }

    /// A wider ceiling gives the same answer as a tight one.
    ///
    /// The ceiling sets the number of radix passes. A pass over a digit that
    /// every key holds at zero must change nothing.
    #[test]
    fn a_wider_ceiling_gives_the_same_order(keys in tied_bounded_keys()) {
        let tight = sort::order_bounded(&keys, 7)
            .expect("the identifiers are unique and the keys are inside the ceiling");
        let wide = sort::order_bounded(&keys, u64::MAX)
            .expect("the identifiers are unique and the keys are inside the ceiling");
        prop_assert_eq!(tight, wide);
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
fn a_repeated_key_is_an_error() {
    // ADR-0105 D1: two keys that agree in every field, the identifier
    // included, tie with nothing left to separate them.
    let keys = [
        SortKey::new([1, 7]),
        SortKey::new([1, 7]),
        SortKey::new([0, 9]),
    ];
    assert_eq!(sort::order(&keys), Err(SortError::RepeatedKey(7)));
}

/// Two keys that share an identifier and differ in an ordering field are
/// separated by the field they differ in, so the order is total without the
/// identifier deciding anything.
///
/// **The sort refused this until ADR-0105.** The check it refused it with cost
/// a comparison sort of the whole set on every call, and this is the case that
/// pays for it and gains nothing.[^1]
///
/// # References
///
/// [^1]: ADR-0105 D2, a total order needs no repeated identifier, only no repeated key. `docs/adrs/draft/adr-0105-a-total-order-needs-no-repeated-key.md`
#[test]
fn a_repeated_identifier_that_ties_nothing_is_accepted() {
    let keys = [
        SortKey::new([1, 7]),
        SortKey::new([2, 7]),
        SortKey::new([0, 9]),
    ];
    assert_eq!(sort::order(&keys), Ok(vec![2, 0, 1]));
}

/// The refusal is the same at every thread count, because it is a property of
/// the keys and not of how they were divided.[^1]
///
/// # References
///
/// [^1]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
#[test]
fn a_repeated_key_is_an_error_at_every_thread_count() {
    let keys: Vec<SortKey<2>> = (0..64u64)
        .map(|value| SortKey::new([value / 2, value / 2]))
        .collect();
    for threads in [1usize, 2, 3, 8, 12] {
        assert_eq!(
            sort::order_on(&keys, threads),
            Err(SortError::RepeatedKey(0)),
            "the refusal differs at {threads} threads"
        );
    }
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

#[test]
fn a_key_above_the_ceiling_is_an_error() {
    // ADR-0071 D1: the caller states the ceiling, and the sort refuses a key
    // above it rather than widening itself to fit.
    let keys = [BoundedKey::new(3, 10), BoundedKey::new(9, 11)];
    assert_eq!(
        sort::order_bounded(&keys, 4),
        Err(SortError::KeyAboveCeiling { key: 9, ceiling: 4 })
    );
}

#[test]
fn a_key_at_the_ceiling_is_accepted() {
    // The ceiling is the highest legal key, not the first illegal one. A
    // world whose extent divides by the block edge reaches it exactly.
    let keys = [BoundedKey::new(4, 10), BoundedKey::new(0, 11)];
    assert_eq!(sort::order_bounded(&keys, 4), Ok(vec![1, 0]));
}

#[test]
fn a_repeated_key_is_an_error_in_the_bounded_order() {
    // ADR-0105 D1: two keys that agree in both fields tie with nothing left
    // to separate them.
    let keys = [
        BoundedKey::new(1, 7),
        BoundedKey::new(1, 7),
        BoundedKey::new(0, 9),
    ];
    assert_eq!(
        sort::order_bounded(&keys, 4),
        Err(SortError::RepeatedKey(7))
    );
}

/// The bounded order accepts a repeated identifier that ties nothing, in the
/// same way the general order does.[^1]
///
/// # References
///
/// [^1]: ADR-0105 D2, a total order needs no repeated identifier, only no repeated key. `docs/adrs/draft/adr-0105-a-total-order-needs-no-repeated-key.md`
#[test]
fn a_repeated_identifier_that_ties_nothing_is_accepted_in_the_bounded_order() {
    let keys = [
        BoundedKey::new(1, 7),
        BoundedKey::new(2, 7),
        BoundedKey::new(0, 9),
    ];
    assert_eq!(sort::order_bounded(&keys, 4), Ok(vec![2, 0, 1]));
}

#[test]
fn an_empty_set_gives_an_empty_bounded_order() {
    let keys: [BoundedKey; 0] = [];
    assert_eq!(sort::order_bounded(&keys, 0), Ok(Vec::new()));
}

#[test]
fn a_ceiling_of_zero_orders_a_set_of_one_key_value() {
    // Every key is zero, so only the identifier orders the set.
    let keys = [
        BoundedKey::new(0, 30),
        BoundedKey::new(0, 10),
        BoundedKey::new(0, 20),
    ];
    let order = sort::order_bounded(&keys, 0).expect("the identifiers are unique");
    assert_eq!(order, vec![1, 2, 0]);
}

#[test]
fn the_identifier_breaks_a_tie_and_the_input_order_does_not() {
    // The radix pass is stable, so it would leave these two in the order the
    // caller gave. Delete the step that orders a tied run and this test fails
    // in one of its two halves.
    let forward = [BoundedKey::new(5, 90), BoundedKey::new(5, 20)];
    let backward = [BoundedKey::new(5, 20), BoundedKey::new(5, 90)];
    assert_eq!(sort::order_bounded(&forward, 8), Ok(vec![1, 0]));
    assert_eq!(sort::order_bounded(&backward, 8), Ok(vec![0, 1]));
}
