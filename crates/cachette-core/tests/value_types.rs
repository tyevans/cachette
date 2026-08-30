//! The value types and the state hash.
//!
//! Every value type is a newtype, so the compiler rejects a substitution
//! between two integers of the same width.[^1] These tests cover the
//! conversions of each newtype, because a wrong shift in a conversion is
//! silent.
//!
//! # References
//!
//! [^1]: ADR-0002, Target platform and value types, decision D9. `docs/adrs/draft/adr-0002-target-platform-and-value-types.md`

use cachette_core::sim_math;
use cachette_core::types::{Accum, FIX_FRACTIONAL_BITS};
use cachette_core::{Entity, Fix32, StateHash};

#[test]
fn an_entity_carries_an_index_and_a_generation() {
    let entity = Entity::new(7, 3).expect("the handle is not zero");
    assert_eq!(entity.index(), 7);
    assert_eq!(entity.generation(), 3);
    assert_eq!(entity.to_bits(), (3u64 << 32) | 7);
}

#[test]
fn an_entity_of_index_zero_and_generation_zero_does_not_exist() {
    // ADR-0002 D9: the handle is never zero, so `Option<Entity>` stays 8
    // bytes wide.
    assert_eq!(Entity::new(0, 0), None);
    assert_eq!(size_of::<Option<Entity>>(), 8);
}

#[test]
fn an_entity_keeps_the_two_parts_apart() {
    let first = Entity::new(1, 0).expect("the handle is not zero");
    let second = Entity::new(0, 1).expect("the handle is not zero");
    assert_ne!(first, second);
    assert_eq!(first.index(), 1);
    assert_eq!(first.generation(), 0);
    assert_eq!(second.index(), 0);
    assert_eq!(second.generation(), 1);
}

#[test]
fn a_whole_number_converts_to_fixed_point_and_back() {
    assert_eq!(Fix32::from_int(0), Fix32::ZERO);
    assert_eq!(Fix32::from_int(1), Fix32::ONE);
    assert_eq!(Fix32::from_int(3).0, 3 << FIX_FRACTIONAL_BITS);
    assert_eq!(Fix32::from_int(-2).0, -2 << FIX_FRACTIONAL_BITS);
    for value in [-5i16, -1, 0, 1, 5, 300] {
        assert_eq!(Fix32::from_int(value).to_int_floor(), i32::from(value));
    }
}

#[test]
fn the_whole_part_rounds_towards_negative_infinity() {
    let half = Fix32(1 << (FIX_FRACTIONAL_BITS - 1));
    assert_eq!(half.to_int_floor(), 0);
    assert_eq!(sim_math::sub(Fix32::ZERO, half).to_int_floor(), -1);
    assert_eq!(Fix32(-1).to_int_floor(), -1);
}

#[test]
fn a_fixed_point_value_widens_into_an_accumulator() {
    assert_eq!(Fix32::ONE.to_accum(), Accum(i64::from(Fix32::ONE.0)));
    assert_eq!(Fix32(-7).to_accum(), Accum(-7));
}

#[test]
fn multiplication_saturates_at_both_limits() {
    let large = Fix32::from_int(1000);
    assert_eq!(sim_math::mul(large, large), Fix32::MAX);
    assert_eq!(sim_math::mul(large, Fix32::from_int(-1000)), Fix32::MIN);
    // A product just inside the limit does not saturate.
    let small = Fix32::from_int(2);
    assert_eq!(sim_math::mul(small, small), Fix32::from_int(4));
}

#[test]
fn division_saturates_at_both_limits() {
    let tiny = Fix32(1);
    assert_eq!(sim_math::div(Fix32::from_int(1000), tiny), Some(Fix32::MAX));
    assert_eq!(sim_math::div(Fix32::from_int(-1000), tiny), Some(Fix32::MIN));
}

#[test]
fn addition_saturates_at_both_limits() {
    assert_eq!(sim_math::add(Fix32::MAX, Fix32::ONE), Fix32::MAX);
    assert_eq!(sim_math::add(Fix32::MIN, Fix32(-1)), Fix32::MIN);
}

#[test]
fn the_accumulator_saturates_rather_than_wraps() {
    let limit = Accum(i64::MAX);
    assert_eq!(sim_math::combine(limit, Accum(1)), limit);
    assert_eq!(sim_math::accumulate(limit, Fix32::ONE), limit);
}

#[test]
fn the_hash_depends_on_every_byte_and_on_the_order() {
    let empty = StateHash::new().finish();
    assert_eq!(StateHash::default().finish(), empty);
    assert_ne!(StateHash::new().write(&[0]).finish(), empty);
    assert_ne!(
        StateHash::new().write(&[1, 2]).finish(),
        StateHash::new().write(&[2, 1]).finish()
    );
    assert_eq!(
        StateHash::new().write(&[1, 2]).finish(),
        StateHash::new().write(&[1]).write(&[2]).finish()
    );
}

#[test]
fn the_hash_writes_a_wide_integer_in_little_endian_order() {
    assert_eq!(
        StateHash::new().write_u64(0x0102_0304_0506_0708).finish(),
        StateHash::new()
            .write(&[0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01])
            .finish()
    );
}

#[test]
fn the_hash_prints_as_sixteen_hexadecimal_digits() {
    assert_eq!(format!("{}", StateHash::new()).len(), 16);
}
