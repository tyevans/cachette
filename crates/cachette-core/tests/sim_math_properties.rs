//! Property tests for the arithmetic boundary.
//!
//! This is the worked example that proves the property harness. Copy its
//! shape. The properties that matter are the algebraic laws that determinism
//! depends on: exact commutativity and exact associativity.[^1]
//!
//! The test sees only the public crate API.[^2]
//!
//! # References
//!
//! [^1]: ADR-0001, Determinism as the primary constraint, decisions D2 and D7. `docs/adrs/draft/adr-0001-determinism.md`
//! [^2]: Testing policy. `docs/TESTING.md`

use cachette_core::sim_math;
use cachette_core::types::Accum;
use cachette_core::Fix32;
use proptest::prelude::*;

/// A strategy that produces a fixed-point value over the whole range.
fn any_fix32() -> impl Strategy<Value = Fix32> {
    any::<i32>().prop_map(Fix32)
}

/// A strategy that produces an accumulator over a range that cannot
/// saturate when three of them combine.
fn any_accum() -> impl Strategy<Value = Accum> {
    (i64::MIN / 4..i64::MAX / 4).prop_map(Accum)
}

proptest! {
    /// Accumulator combination is commutative. A parallel reduction over it
    /// therefore needs no declared order.
    #[test]
    fn combine_is_commutative(a in any_accum(), b in any_accum()) {
        prop_assert_eq!(sim_math::combine(a, b), sim_math::combine(b, a));
    }

    /// Accumulator combination is associative. The fold order therefore
    /// cannot change the answer.
    #[test]
    fn combine_is_associative(a in any_accum(), b in any_accum(), c in any_accum()) {
        let left = sim_math::combine(sim_math::combine(a, b), c);
        let right = sim_math::combine(a, sim_math::combine(b, c));
        prop_assert_eq!(left, right);
    }

    /// Addition is commutative, including at the saturation limit.
    #[test]
    fn add_is_commutative(a in any_fix32(), b in any_fix32()) {
        prop_assert_eq!(sim_math::add(a, b), sim_math::add(b, a));
    }

    /// Zero is the identity of addition.
    #[test]
    fn zero_is_the_identity_of_addition(a in any_fix32()) {
        prop_assert_eq!(sim_math::add(a, Fix32::ZERO), a);
    }

    /// One is the identity of multiplication.
    #[test]
    fn one_is_the_identity_of_multiplication(a in any_fix32()) {
        prop_assert_eq!(sim_math::mul(a, Fix32::ONE), a);
    }

    /// Multiplication is commutative.
    #[test]
    fn mul_is_commutative(a in any_fix32(), b in any_fix32()) {
        prop_assert_eq!(sim_math::mul(a, b), sim_math::mul(b, a));
    }

    /// Addition never wraps. A wrap turns a large value into a large
    /// negative value and hides the defect.
    #[test]
    fn add_saturates_and_never_wraps(a in any_fix32(), b in any_fix32()) {
        let sum = sim_math::add(a, b);
        if b.0 > 0 {
            prop_assert!(sum.0 >= a.0);
        } else {
            prop_assert!(sum.0 <= a.0);
        }
    }

    /// Division by zero returns nothing. The module does not panic.
    #[test]
    fn division_by_zero_returns_nothing(a in any_fix32()) {
        prop_assert_eq!(sim_math::div(a, Fix32::ZERO), None);
    }

    /// A value divided by itself is one, where the value is large enough to
    /// hold the quotient exactly.
    #[test]
    fn a_value_divided_by_itself_is_one(a in 1i32..=i32::MAX) {
        let value = Fix32(a);
        prop_assert_eq!(sim_math::div(value, value), Some(Fix32::ONE));
    }
}
