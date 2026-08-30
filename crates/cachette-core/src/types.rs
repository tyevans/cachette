//! The value types.
//!
//! A raw integer carries no meaning, and two raw integers of the same width
//! substitute for each other in silence. Every value below is a newtype, so
//! the compiler rejects the substitution. A newtype costs nothing at run
//! time.[^1]
//!
//! This module holds a subset of the table in the record. It holds the types
//! that the current stubs need. Add the rest when a subsystem needs them.
//!
//! # References
//!
//! [^1]: ADR-0002, Target platform and value types, decision D9. `docs/adrs/draft/adr-0002-target-platform-and-value-types.md`

use bytemuck::{Pod, Zeroable};

/// The index of a tile in the world grid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct TileIdx(pub u32);

/// The identifier of a faction.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct FactionId(pub u16);

/// Simulation time, counted in ticks.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct Tick(pub u64);

/// A summary accumulator.
///
/// Every accumulator is 64 bits wide, and the widening happens at the first
/// summary level. A one-byte tile field summed over 2^24 tiles reaches 2^32
/// exactly, which a 32-bit accumulator cannot hold.[^1]
///
/// # References
///
/// [^1]: ADR-0001, Determinism as the primary constraint, decision D4. `docs/adrs/draft/adr-0001-determinism.md`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct Accum(pub i64);

/// A generational entity handle.
///
/// The handle is never a bare index. The generation field is what makes a
/// stale reference detectable instead of silently wrong. The value is never
/// zero, so `Option<Entity>` stays 8 bytes wide.[^1]
///
/// # References
///
/// [^1]: ADR-0002, Target platform and value types, decision D9. `docs/adrs/draft/adr-0002-target-platform-and-value-types.md`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entity(core::num::NonZeroU64);

impl Entity {
    /// Builds a handle from an index and a generation.
    ///
    /// Returns `None` when both parts are zero, because the handle may not
    /// hold the value zero.
    #[must_use]
    pub const fn new(index: u32, generation: u32) -> Option<Self> {
        let raw = ((generation as u64) << 32) | (index as u64);
        match core::num::NonZeroU64::new(raw) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    /// Returns the index part of the handle.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0.get() as u32
    }

    /// Returns the generation part of the handle.
    #[must_use]
    pub const fn generation(self) -> u32 {
        (self.0.get() >> 32) as u32
    }

    /// Returns the whole handle as one integer. The sort key uses this.
    #[must_use]
    pub const fn to_bits(self) -> u64 {
        self.0.get()
    }
}

/// The number of fractional bits in the project-wide fixed-point scale.
pub const FIX_FRACTIONAL_BITS: u32 = 16;

/// A fixed-point scalar in Q16.16.
///
/// The project uses one scale everywhere. The record rejects a second
/// scale, because the only argument for it was to keep a multiply inside 32
/// bits, and the target runs 64-bit integer arithmetic at full rate.[^1]
///
/// Use the operations in the arithmetic module. Do not do arithmetic on the
/// raw field.[^2]
///
/// # References
///
/// [^1]: ADR-0001, Determinism as the primary constraint, decision D4. `docs/adrs/draft/adr-0001-determinism.md`
/// [^2]: ADR-0001, Determinism as the primary constraint, decision D3. `docs/adrs/draft/adr-0001-determinism.md`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct Fix32(pub i32);

impl Fix32 {
    /// The value zero.
    pub const ZERO: Self = Self(0);
    /// The value one.
    pub const ONE: Self = Self(1 << FIX_FRACTIONAL_BITS);
    /// The largest value that the type holds.
    pub const MAX: Self = Self(i32::MAX);
    /// The smallest value that the type holds.
    pub const MIN: Self = Self(i32::MIN);

    /// Builds a value from a whole number.
    ///
    /// The result saturates when the whole number is outside the range.
    #[must_use]
    pub const fn from_int(value: i16) -> Self {
        Self((value as i32) << FIX_FRACTIONAL_BITS)
    }

    /// Returns the whole part of the value. The result rounds towards
    /// negative infinity.
    #[must_use]
    pub const fn to_int_floor(self) -> i32 {
        self.0 >> FIX_FRACTIONAL_BITS
    }

    /// Widens the value into an accumulator.
    #[must_use]
    pub const fn to_accum(self) -> Accum {
        Accum(self.0 as i64)
    }
}
