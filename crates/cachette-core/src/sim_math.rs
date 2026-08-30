//! The arithmetic boundary.
//!
//! All arithmetic on simulated state goes through this module. The record
//! requires the boundary and requires a lint to enforce it. The boundary
//! cannot be added later, because adding it later means auditing every
//! line.[^1]
//!
//! Every operation here is exact and total. No operation reads the wall
//! clock, the thread identity, or an allocation address.
//!
//! Integer addition and bitwise OR are exactly commutative and associative,
//! so a parallel reduction over them needs no declared order. Minimum,
//! maximum and first-wins do not have that property, and they need indexed
//! output slots.[^2]
//!
//! # References
//!
//! [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^2]: ADR-0004, iteration order is explicit, decisions D2 and D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`

use crate::types::{Accum, Fix32, FIX_FRACTIONAL_BITS};

/// Adds two fixed-point values. The result saturates at the range limit.
///
/// The operation saturates rather than wraps, because a wrap turns a large
/// value into a large negative value and hides the defect.
#[must_use]
pub const fn add(a: Fix32, b: Fix32) -> Fix32 {
    Fix32(a.0.saturating_add(b.0))
}

/// Subtracts one fixed-point value from another. The result saturates.
#[must_use]
pub const fn sub(a: Fix32, b: Fix32) -> Fix32 {
    Fix32(a.0.saturating_sub(b.0))
}

/// Multiplies two fixed-point values.
///
/// The product uses 64-bit intermediate arithmetic, which the target runs at
/// the same rate as 32-bit arithmetic.[^1] The result truncates towards
/// negative infinity, then saturates at the range limit.
///
/// # References
///
/// [^1]: ADR-0008, the primary target is aarch64, and NEON is a baseline rather than a dispatch. `docs/adrs/REGISTRY.md`
#[must_use]
pub const fn mul(a: Fix32, b: Fix32) -> Fix32 {
    let wide = (a.0 as i64) * (b.0 as i64);
    let shifted = wide >> FIX_FRACTIONAL_BITS;
    Fix32(saturate_i32(shifted))
}

/// Divides one fixed-point value by another.
///
/// Returns `None` when the divisor is zero. A division by zero is a caller
/// error, and this module does not panic on it.
#[must_use]
pub const fn div(a: Fix32, b: Fix32) -> Option<Fix32> {
    if b.0 == 0 {
        return None;
    }
    let wide = ((a.0 as i64) << FIX_FRACTIONAL_BITS) / (b.0 as i64);
    Some(Fix32(saturate_i32(wide)))
}

/// Adds a fixed-point value into an accumulator.
///
/// The accumulator is 64 bits wide, so the sum of a whole level of the
/// pyramid cannot overflow it.[^1]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
#[must_use]
pub const fn accumulate(total: Accum, value: Fix32) -> Accum {
    Accum(total.0.saturating_add(value.0 as i64))
}

/// Combines two accumulators.
///
/// This operation is commutative and associative, so a parallel reduction
/// over it gives one answer at any thread count.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decisions D2 and D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
#[must_use]
pub const fn combine(a: Accum, b: Accum) -> Accum {
    Accum(a.0.saturating_add(b.0))
}

/// Clamps a wide value into the 32-bit range.
const fn saturate_i32(value: i64) -> i32 {
    if value > i32::MAX as i64 {
        i32::MAX
    } else if value < i32::MIN as i64 {
        i32::MIN
    } else {
        value as i32
    }
}
