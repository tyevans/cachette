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

/// Scales a per-unit rate by a whole headcount.
///
/// The product is exact in 64 bits and it saturates at the range limit. The
/// multiply that scales a fixed-point rate takes a whole number of at most
/// sixteen bits, and a headcount is wider than that, so a cohort demand
/// cannot go through it.[^1]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D3. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
#[must_use]
pub const fn scale_by_count(rate: Fix32, count: u32) -> Accum {
    Accum((rate.0 as i64).saturating_mul(count as i64))
}

/// Scales a whole amount by a fixed-point factor.
///
/// The product is exact in 64 bits and truncates towards zero, so a scale of
/// one returns the amount unchanged and a scale of zero returns zero. The
/// result saturates at the range of the amount. A negative scale has no
/// meaning for an amount, and it returns zero.
///
/// The gather pass uses this to scale the tile rate by the gather rate of a
/// unit type.[^1]
///
/// # References
///
/// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D1. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
#[must_use]
pub const fn scale_amount(amount: u32, scale: Fix32) -> u32 {
    if scale.0 <= 0 {
        return 0;
    }
    let wide = (amount as i64) * (scale.0 as i64);
    let shifted = wide >> FIX_FRACTIONAL_BITS;
    if shifted > u32::MAX as i64 {
        u32::MAX
    } else {
        shifted as u32
    }
}

/// Scales a whole quantity of work by a fixed-point factor.
///
/// The product is exact in 128 bits and truncates towards zero, then
/// saturates at the range of the accumulator. A scale of one returns the work
/// unchanged and a scale of zero returns zero. A negative scale has no meaning
/// for work, and it returns zero.
///
/// The build pass uses this to scale the builder rate by the build rate of a
/// unit type.[^1]
///
/// # References
///
/// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D1. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
#[must_use]
pub const fn scale_work(work: i64, scale: Fix32) -> i64 {
    if scale.0 <= 0 {
        return 0;
    }
    let wide = (work as i128) * (scale.0 as i128);
    saturate_i64(wide >> FIX_FRACTIONAL_BITS)
}

/// Returns the part of a total that one share of a whole earns.
///
/// The result truncates `total * part / whole` towards zero. The intermediate
/// product is 128 bits wide, so it is exact for every input that a 64-bit
/// accumulator holds. The floor is what makes a split of a total sum to at
/// most the total, so the caller hands out the remainder itself.
///
/// Returns `None` when the whole is zero. A division by zero is a caller
/// error, and this module does not panic on it.
#[must_use]
pub const fn share(total: Accum, part: Accum, whole: Accum) -> Option<Accum> {
    if whole.0 == 0 {
        return None;
    }
    let wide = (total.0 as i128) * (part.0 as i128) / (whole.0 as i128);
    Some(Accum(saturate_i64(wide)))
}

/// Returns what the share left behind.
///
/// The result is `total * part` less `whole` times the share of the same
/// three values. The intermediate product is 128 bits wide, so it is exact
/// for every input that a 64-bit accumulator holds. A caller that hands out
/// the remainder needs it exactly, because the floor of the share is what
/// makes a split sum to at most the total.
///
/// Returns `None` when the whole is zero. A division by zero is a caller
/// error, and this module does not panic on it.
#[must_use]
pub const fn share_remainder(total: Accum, part: Accum, whole: Accum) -> Option<Accum> {
    if whole.0 == 0 {
        return None;
    }
    let wide = (total.0 as i128) * (part.0 as i128) % (whole.0 as i128);
    Some(Accum(saturate_i64(wide)))
}

/// Divides an accumulator by a whole count.
///
/// The result truncates towards zero, narrowed into the fixed-point
/// range. The remainder is not returned, because the caller of this
/// operation spreads an intensive value over a headcount and an intensive
/// value is not conserved.[^1]
///
/// Returns `None` when the count is zero.
///
/// # References
///
/// [^1]: Research report 15, needs, consumption and the input-output economy, section 6.3. `docs/research/reports/15-needs-consumption-and-economy.md`
#[must_use]
pub const fn divide_by_count(total: Accum, count: u32) -> Option<Fix32> {
    if count == 0 {
        return None;
    }
    Some(Fix32(saturate_i32(total.0 / (count as i64))))
}

/// Clamps a 128-bit value into the accumulator range.
const fn saturate_i64(value: i128) -> i64 {
    if value > i64::MAX as i128 {
        i64::MAX
    } else if value < i64::MIN as i128 {
        i64::MIN
    } else {
        value as i64
    }
}
