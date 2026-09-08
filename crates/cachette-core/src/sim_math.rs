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

/// Narrows an accumulator into the fixed-point range, saturating.
///
/// A pass that sums a per-item quantity over a whole count holds the total in
/// an accumulator, because the accumulator is the width that a sum over the
/// world cannot overflow.[^1] A pass that then writes the total into a
/// fixed-point column needs this one narrowing, and it must saturate rather
/// than wrap, because a wrap turns a large value into a large negative
/// one.[^2]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
/// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D3. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
#[must_use]
pub const fn narrow(total: Accum) -> Fix32 {
    Fix32(saturate_i32(total.0))
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

/// Moves a whole relation value by a whole step, saturating at both ends.
///
/// The relation between two factions is a whole signed number and never a
/// fixed-point value, so this is the one addition the relation pass
/// makes.[^1] A step that would leave the range stops at the end of it, so no
/// cause wraps a war into an alliance.
///
/// # References
///
/// [^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D1. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
#[must_use]
pub const fn offset(value: i32, step: i32) -> i32 {
    value.saturating_add(step)
}

/// Multiplies a relation step by a whole count, saturating.
///
/// A cause that fires once for each unit of a group moves the relation by the
/// step for each unit, so the pass takes the product once rather than adding
/// the step in a loop.
#[must_use]
pub const fn offset_by_count(step: i32, count: u32) -> i32 {
    let product = (step as i64) * (count as i64);
    if product > i32::MAX as i64 {
        i32::MAX
    } else if product < i32::MIN as i64 {
        i32::MIN
    } else {
        product as i32
    }
}

/// The steps the sine table holds in one quarter turn.
///
/// The table holds one more entry than this, because the quarter ends on the
/// top of the wave and the interpolation reads the entry after the one it
/// starts from.
const SINE_QUARTER: i64 = 64;

/// The steps in one whole turn of the sine.
pub const SINE_STEPS: i64 = 4 * SINE_QUARTER;

/// A quarter turn of the sine, in Q16.16.
///
/// Entry `i` holds the sine of `i` steps of a turn of [`SINE_STEPS`] steps.
/// The first entry is zero and the last is one. The other three quarters of
/// the wave are this quarter reflected and negated, so the table states the
/// shape once.
///
/// **The table is data and not a polynomial**, because a reader can see the
/// shape without simulating it and because the same index always gives the
/// same value on every target.
const SINE_TABLE: [i32; 65] = [
    0, 1608, 3216, 4821, 6424, 8022, 9616, 11204, 12785, 14359, 15924, 17479, 19024, 20557, 22078,
    23586, 25080, 26558, 28020, 29466, 30893, 32303, 33692, 35062, 36410, 37736, 39040, 40320,
    41576, 42806, 44011, 45190, 46341, 47464, 48559, 49624, 50660, 51665, 52639, 53581, 54491,
    55368, 56212, 57022, 57798, 58538, 59244, 59914, 60547, 61145, 61705, 62228, 62714, 63162,
    63572, 63944, 64277, 64571, 64827, 65043, 65220, 65358, 65457, 65516, 65536,
];

/// Returns the sine at a whole step of the turn.
///
/// The step runs from zero to [`SINE_STEPS`]. The three quarters after the
/// first read the table backwards, or negated, or both.
const fn sine_at_step(step: i64) -> i32 {
    let step = step.rem_euclid(SINE_STEPS);
    if step <= SINE_QUARTER {
        SINE_TABLE[step as usize]
    } else if step <= 2 * SINE_QUARTER {
        SINE_TABLE[(2 * SINE_QUARTER - step) as usize]
    } else if step <= 3 * SINE_QUARTER {
        -SINE_TABLE[(step - 2 * SINE_QUARTER) as usize]
    } else {
        -SINE_TABLE[(4 * SINE_QUARTER - step) as usize]
    }
}

/// Returns the sine of a phase, in Q16.16.
///
/// The phase is a position in a turn and the period is the length of that
/// turn, both in whatever unit the caller counts in. So `sine(0, period)` is
/// zero, `sine(period / 4, period)` is one, and the answer repeats every
/// period. A phase outside one turn wraps, and a negative phase wraps the
/// other way.
///
/// **The answer comes from a table of a quarter turn, interpolated between
/// two entries.** It holds no floating point value, it reads no library, and
/// the same phase gives the same answer on every target.[^1]
///
/// Returns `None` when the period is not positive. A period of zero is a
/// caller error, and this module does not panic on it.
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decisions D1 and D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
#[must_use]
pub const fn sine(phase: i64, period: i64) -> Option<Fix32> {
    if period <= 0 {
        return None;
    }
    let turned = phase.rem_euclid(period);
    // The position in the turn, in table steps with a fixed-point tail. The
    // product is 128 bits wide, so a long period costs no accuracy.
    let fine = 1i64 << FIX_FRACTIONAL_BITS;
    let scaled =
        ((turned as i128) * (SINE_STEPS as i128) * (fine as i128) / (period as i128)) as i64;
    let step = scaled >> FIX_FRACTIONAL_BITS;
    let part = scaled - (step << FIX_FRACTIONAL_BITS);
    let low = sine_at_step(step) as i64;
    let high = sine_at_step(step + 1) as i64;
    Some(Fix32(saturate_i32(low + (high - low) * part / fine)))
}

/// Returns the angle whose cosine is the value, as a phase in table steps.
///
/// The answer is a position in a turn of [`SINE_STEPS`] steps, held in the
/// same fixed-point form as every other value here. So an answer of zero is
/// a cosine of one, and an answer of half a turn is a cosine of minus one.
/// The answer never leaves the first half turn, which is the range over
/// which the cosine falls once and takes each value once.
///
/// **The answer comes from the same table the sine reads.** The cosine of a
/// step is the sine of that step a quarter turn later, and that quarter turn
/// of the table falls from one to minus one over the half turn. So a search
/// over the table inverts the cosine without a second table and without a
/// polynomial, and the same value gives the same answer on every target.[^1]
///
/// A value outside the range from minus one to one saturates at the end of
/// the range it passes.
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decisions D1 and D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
#[must_use]
pub const fn arc_cosine_steps(value: Fix32) -> i64 {
    let one = 1i64 << FIX_FRACTIONAL_BITS;
    let target = value.0 as i64;
    if target >= one {
        return 0;
    }
    let half = 2 * SINE_QUARTER;
    if target <= -one {
        return half << FIX_FRACTIONAL_BITS;
    }
    // The cosine of a whole step, read as the sine a quarter turn later. It
    // falls over the half turn, so a bisection brackets the answer between
    // two neighbouring steps.
    let mut low = 0i64;
    let mut high = half;
    while high - low > 1 {
        let middle = (low + high) / 2;
        if (sine_at_step(middle + SINE_QUARTER) as i64) > target {
            low = middle;
        } else {
            high = middle;
        }
    }
    let above = sine_at_step(low + SINE_QUARTER) as i64;
    let below = sine_at_step(high + SINE_QUARTER) as i64;
    if above == below {
        return low << FIX_FRACTIONAL_BITS;
    }
    // The part of the way from the step above the target to the step below
    // it. Both ends come from the table, so the interpolation is the same
    // one the sine itself uses.
    let part = ((above - target) << FIX_FRACTIONAL_BITS) / (above - below);
    (low << FIX_FRACTIONAL_BITS) + part
}

/// Returns the sine of a phase held in table steps.
///
/// The phase is a position in a turn of [`SINE_STEPS`] steps, in the same
/// fixed-point form that [`arc_cosine_steps`] returns. So the two are
/// inverses of one another, up to the accuracy of the table.
#[must_use]
pub const fn sine_of_steps(phase: i64) -> Fix32 {
    let fine = 1i64 << FIX_FRACTIONAL_BITS;
    let step = phase >> FIX_FRACTIONAL_BITS;
    let part = phase - (step << FIX_FRACTIONAL_BITS);
    let low = sine_at_step(step) as i64;
    let high = sine_at_step(step + 1) as i64;
    Fix32(saturate_i32(low + (high - low) * part / fine))
}

/// The bit width that bounds a compressed magnitude.
///
/// A compressed magnitude divides the base-two logarithm of a quantity by
/// this width, so a quantity below two to this power maps inside the unit
/// range. The width is a structural cap and not a budget: it is the widest
/// quantity the observation of a faction can carry, which is a fixed-point
/// store total summed over the unit ceiling of the world.[^1]
///
/// # References
///
/// [^1]: Research report 42, what a policy should be able to see, section 4. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
pub const MAGNITUDE_CAP_BITS: u32 = 40;

/// Returns the part of a whole, bounded to the unit range.
///
/// The result lies between zero and one. A whole of zero or below reads as
/// one, and a part below zero reads as zero, so the function is total and it
/// never divides by zero. The division truncates toward zero.
///
/// **A learner reads this value, so it must be bounded whatever the world
/// size.** A raw count means one thing on a small world and another thing on
/// a large one, and a policy trained against one cannot read the other.[^1]
/// The caller names the denominator, and the denominator is a structural
/// property of the world or a total the engine already keeps.[^2]
///
/// # References
///
/// [^1]: Findings register, FND-670. `docs/FINDINGS.md`
/// [^2]: Research report 42, what a policy should be able to see, section 8.1. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
#[must_use]
pub const fn bounded_share(part: i64, whole: i64) -> Fix32 {
    let whole = if whole < 1 { 1 } else { whole };
    let part = if part < 0 { 0 } else { part };
    let wide = ((part as i128) << FIX_FRACTIONAL_BITS) / (whole as i128);
    let one = Fix32::ONE.0 as i128;
    Fix32(if wide > one {
        Fix32::ONE.0
    } else {
        wide as i32
    })
}

/// Returns the signed relation between two magnitudes.
///
/// The result lies between minus one and one. It is the difference of the two
/// values over the sum of their magnitudes, so it bounds itself and the
/// caller names no denominator. Two equal values give zero. A first value
/// with a second of zero gives one.
///
/// The division truncates toward zero. A sum of magnitudes of zero reads as
/// one, so the function is total.
#[must_use]
pub const fn signed_relation(a: i64, b: i64) -> Fix32 {
    let scale = (a.unsigned_abs() as i128) + (b.unsigned_abs() as i128);
    let scale = if scale < 1 { 1 } else { scale };
    let wide = (((a as i128) - (b as i128)) << FIX_FRACTIONAL_BITS) / scale;
    let one = Fix32::ONE.0 as i128;
    if wide > one {
        Fix32::ONE
    } else if wide < -one {
        Fix32(-Fix32::ONE.0)
    } else {
        Fix32(wide as i32)
    }
}

/// Returns a quantity compressed into the unit range, keeping its sign.
///
/// The result lies between minus one and one. It is the base-two logarithm
/// of one plus the magnitude of the value, over the cap width, with the sign
/// of the value restored.[^1] The map is monotone in the magnitude, so a
/// larger quantity always reads larger.
///
/// **This is the one form the observation publishes for a quantity with no
/// named denominator.** A running total declared with the whole integer range
/// as its bound spans twelve orders of magnitude, and a learner that takes a
/// fixed-size step cannot serve such an input.[^2]
///
/// # References
///
/// [^1]: Research report 42, what a policy should be able to see, section 4. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
/// [^2]: Findings register, FND-670. `docs/FINDINGS.md`
#[must_use]
pub const fn compressed_magnitude(value: i64) -> Fix32 {
    let logarithm = log2_fixed(1 + value.unsigned_abs()) as i64;
    let scaled = logarithm / (MAGNITUDE_CAP_BITS as i64);
    let one = Fix32::ONE.0 as i64;
    let bounded = if scaled > one {
        Fix32::ONE.0
    } else {
        scaled as i32
    };
    if value < 0 {
        Fix32(-bounded)
    } else {
        Fix32(bounded)
    }
}

/// Returns the base-two logarithm of a whole number, in Q16.16.
///
/// The answer is exact to the last fractional bit. The function reads the
/// leading zero count for the whole part, then squares the mantissa sixteen
/// times for the fractional bits. Every step is an integer operation, so two
/// runs on two machines give one answer.
///
/// A value of zero reads as one, because the logarithm of zero has no
/// integer answer and this module does not panic on a caller error.
#[must_use]
pub const fn log2_fixed(value: u64) -> u32 {
    let value = if value < 1 { 1 } else { value };
    let exponent = 63 - value.leading_zeros();
    let mut mantissa = (value as u128) << (63 - exponent);
    let mut out = exponent << FIX_FRACTIONAL_BITS;
    let mut bit = 1u32 << (FIX_FRACTIONAL_BITS - 1);
    while bit != 0 {
        mantissa = (mantissa * mantissa) >> 63;
        if mantissa >= 1u128 << 64 {
            mantissa >>= 1;
            out |= bit;
        }
        bit >>= 1;
    }
    out
}

/// Returns one quadrature of a cyclic phase, as a triangle wave.
///
/// The result lies between minus one and one. It rises across the first half
/// of the period and falls across the second half, so it is continuous at the
/// wrap point.
///
/// **A single phase value carries a step at the wrap point, and a learner
/// reads that step as a large change in the world.** A caller therefore takes
/// this value twice, once at the phase and once a quarter period after it,
/// and the pair removes the step.[^1]
///
/// A period of zero or below reads as one, so the function is total.
///
/// # References
///
/// [^1]: Research report 42, what a policy should be able to see, section 9.2. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
#[must_use]
pub const fn phase_triangle(phase: i64, period: i64) -> Fix32 {
    let period = if period < 1 { 1 } else { period };
    let turn = phase % period;
    let turn = if turn < 0 { turn + period } else { turn };
    let one = Fix32::ONE.0 as i128;
    let whole = (turn as i128) * 4 * one / (period as i128);
    let half = 2 * one;
    let risen = if whole < half { whole } else { 4 * one - whole };
    Fix32((risen - one) as i32)
}
/// Moves a decayed counter one step, and adds the arrivals of that step.
///
/// The recursion is `total - (total >> bits) + arrivals`. It holds one shift
/// and two additions, so it needs no division and no floating point
/// number.[^1]
///
/// **The fixed point of the recursion is the arrival rate shifted left by the
/// same bits.** Take a constant arrival of `a` on every step. The counter
/// grows while `total >> bits` stays below `a`, and it stops at the first
/// value whose shift equals `a`, which is `a` shifted left by `bits`. A
/// caller therefore reads the counter as an arrival rate at a stated scale,
/// and the bound of the counter is the largest arrival one step can hold at
/// the same scale.
///
/// A negative total is refused and reads as zero, because a counter of
/// arrivals never falls below zero. A shift at or above the width of the
/// value would be undefined, so the shift saturates at one below the width.
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decisions D1 and D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
#[must_use]
pub const fn decay(total: i64, arrivals: i64, bits: u32) -> i64 {
    if total < 0 {
        return if arrivals > 0 { arrivals } else { 0 };
    }
    let bits = if bits < i64::BITS {
        bits
    } else {
        i64::BITS - 1
    };
    let leaving = total >> bits;
    let held = total - leaving;
    if arrivals > 0 {
        held.saturating_add(arrivals)
    } else {
        held
    }
}
