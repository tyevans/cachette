//! The hex world geometry.
//!
//! The world is a rhombus, so a tile address is a raw axial pair and the
//! index is the row multiplied by the row length, plus the column. No tile
//! access converts a coordinate.[^1]
//!
//! The six neighbours are six constant offsets, which is the property that
//! an offset index does not have.[^2] A neighbour outside the world is
//! absent: the world does not wrap.[^2]
//!
//! Every value here is an exact integer.[^3] A screen position is not, and
//! it does not belong in this crate: the engine stores the shape and the
//! viewer draws it.[^4]
//!
//! # References
//!
//! [^1]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D1. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
//! [^2]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D2. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
//! [^3]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D3. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
//! [^4]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D4. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`

use bytemuck::{Pod, Zeroable};

use crate::types::TileIdx;

/// The number of neighbours that a hex tile has.
pub const NEIGHBOUR_COUNT: usize = 6;

/// The six neighbour offsets, in a fixed order.
///
/// The order is the direction order, and it never changes. A system that
/// iterates the neighbours of a tile therefore has an explicit stable
/// order.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
pub const NEIGHBOURS: [Axial; NEIGHBOUR_COUNT] = [
    Axial { q: 1, r: 0 },
    Axial { q: 1, r: -1 },
    Axial { q: 0, r: -1 },
    Axial { q: -1, r: 0 },
    Axial { q: -1, r: 1 },
    Axial { q: 0, r: 1 },
];

/// An axial tile address.
///
/// The pair is the address. It is not a position: a position on a screen is
/// the viewer's business.
///
/// The type declares its layout, because a tile address reaches the state
/// hash. Two `i32` fields fill eight bytes exactly, so the type needs no
/// padding field.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct Axial {
    /// The column.
    pub q: i32,
    /// The row.
    pub r: i32,
}

impl Axial {
    /// Builds an address.
    #[must_use]
    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// Returns the sum of two addresses.
    ///
    /// Axial addresses add, which is why the neighbour offsets are constant.
    /// The addition saturates rather than wrapping, so a sum outside the
    /// range stays outside the world instead of appearing inside it.
    #[must_use]
    pub const fn add(self, other: Self) -> Self {
        Self {
            q: self.q.saturating_add(other.q),
            r: self.r.saturating_add(other.r),
        }
    }

    /// Returns the third cube component.
    ///
    /// A cube address is redundant, so the engine stores two components and
    /// derives the third when it needs one.
    #[must_use]
    pub const fn s(self) -> i32 {
        (-self.q).saturating_sub(self.r)
    }

    /// Returns the number of steps from this address to another.
    ///
    /// The result is the hex distance, which is the cube distance halved.
    #[must_use]
    pub const fn distance(self, other: Self) -> u32 {
        let dq = (self.q as i64) - (other.q as i64);
        let dr = (self.r as i64) - (other.r as i64);
        let ds = (self.s() as i64) - (other.s() as i64);
        let sum = dq.unsigned_abs() + dr.unsigned_abs() + ds.unsigned_abs();
        (sum / 2) as u32
    }
}

/// The shape of the world.
///
/// The world is a rhombus of `width` columns by `height` rows. Every address
/// inside those bounds is a tile, and the index space has no hole.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grid {
    width: u32,
    height: u32,
    /// The reciprocal that replaces the division by the width.
    ///
    /// The value is derived from the width, and [`Grid::new`] is the only
    /// site that writes it. The fields are private and the type has no
    /// setter, so the two cannot disagree.[^1]
    ///
    /// A width of one needs no division, and its reciprocal does not fit a
    /// `u64`, so a width of one stores zero here and the quotient is the
    /// index itself.
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    width_reciprocal: u64,
}

/// The reason that a grid refused to build.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridError {
    /// A side was zero. A world with no tile is not a world.
    EmptySide,
    /// The tile count does not fit in the index type.
    TooManyTiles,
}

impl core::fmt::Display for GridError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptySide => write!(formatter, "a grid side must hold at least one tile"),
            Self::TooManyTiles => write!(formatter, "the tile count does not fit a tile index"),
        }
    }
}

impl std::error::Error for GridError {}

/// Returns the reciprocal that replaces a division by `width`.
///
/// The value is `floor(2^64 / width) + 1`. A width of one needs no division
/// and its reciprocal does not fit a `u64`, so this returns zero for it and
/// the caller reads that as "the quotient is the index".
const fn reciprocal_of(width: u32) -> u64 {
    if width <= 1 {
        return 0;
    }
    (((1u128 << 64) / width as u128) + 1) as u64
}

impl Grid {
    /// Builds a grid.
    ///
    /// # Errors
    ///
    /// Returns an error when a side is zero, or when the tile count does not
    /// fit the index type.
    pub const fn new(width: u32, height: u32) -> Result<Self, GridError> {
        if width == 0 || height == 0 {
            return Err(GridError::EmptySide);
        }
        match (width as u64).checked_mul(height as u64) {
            // **The ceiling on the tile count is what makes the reciprocal
            // exact.** A caller that widens the tile index must revisit
            // `address_of`, which states the bound it depends on.
            Some(count) if count <= u32::MAX as u64 => Ok(Self {
                width,
                height,
                width_reciprocal: reciprocal_of(width),
            }),
            _ => Err(GridError::TooManyTiles),
        }
    }

    /// Returns the number of columns.
    #[must_use]
    pub const fn width(self) -> u32 {
        self.width
    }

    /// Returns the number of rows.
    #[must_use]
    pub const fn height(self) -> u32 {
        self.height
    }

    /// Returns the number of tiles.
    #[must_use]
    pub const fn tile_count(self) -> u32 {
        self.width * self.height
    }

    /// Reports whether the address is inside the world.
    #[must_use]
    pub const fn contains(self, address: Axial) -> bool {
        address.q >= 0
            && address.r >= 0
            && (address.q as u32) < self.width
            && (address.r as u32) < self.height
    }

    /// Returns the index of an address, or `None` when it is outside the
    /// world.
    ///
    /// The conversion is one multiply and one add. There is no offset step.
    #[must_use]
    pub const fn index_of(self, address: Axial) -> Option<TileIdx> {
        if !self.contains(address) {
            return None;
        }
        Some(TileIdx(
            (address.r as u32) * self.width + (address.q as u32),
        ))
    }

    /// Returns the address of an index, or `None` when it is outside the
    /// world.
    ///
    /// The viewer needs this direction, so it is part of the interface
    /// rather than a test helper.
    ///
    /// **The conversion multiplies by a reciprocal. It does not divide.** The
    /// width is fixed when the grid is built and is not known when the crate
    /// is compiled, so a division here is a hardware division, and this
    /// function has call sites in most modules of the crate. A measurement
    /// found the division to be the largest single part of turning a tile
    /// into a level 1 cell.[^1]
    ///
    /// **The multiply is exact, and it is exact by construction.** For a
    /// width `d`, the grid stores `m = floor(2^64 / d) + 1`. Write
    /// `e = m * d - 2^64`, which satisfies `1 <= e <= d`. Then
    /// `floor(n * m / 2^64) = floor(n / d)` holds for every `n` with
    /// `e * n < 2^64`.
    ///
    /// **This grid meets that bound because [`Grid::new`] refuses a world
    /// whose tile count leaves a `u32`.** So `e <= d <= u32::MAX` and
    /// `n < 2^32`, and the product stays below `2^64` for every world this
    /// crate can build. A change that widens the tile index breaks that
    /// argument, and it must widen this reasoning with it.
    ///
    /// The arithmetic is integer throughout, so the result is bit-identical
    /// to the division it replaces and no state depends on a rounding
    /// rule.[^2]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-282. `docs/FINDINGS.md`
    /// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    #[must_use]
    pub const fn address_of(self, index: TileIdx) -> Option<Axial> {
        if index.0 >= self.tile_count() {
            return None;
        }
        let row = self.row_of(index.0);
        // The remainder follows from the quotient, so the second division
        // goes with the first.
        let column = index.0 - row * self.width;
        Some(Axial {
            q: column as i32,
            r: row as i32,
        })
    }

    /// Returns the row that holds an index.
    ///
    /// The caller has already checked that the index is inside the world.
    const fn row_of(self, index: u32) -> u32 {
        if self.width_reciprocal == 0 {
            // The width is one, so every index is its own row.
            return index;
        }
        ((index as u128 * self.width_reciprocal as u128) >> 64) as u32
    }

    /// Returns the neighbour of an address in one direction.
    ///
    /// Returns `None` when the neighbour falls outside the world. The world
    /// does not wrap.
    #[must_use]
    pub const fn neighbour(self, address: Axial, direction: usize) -> Option<Axial> {
        if direction >= NEIGHBOUR_COUNT {
            return None;
        }
        let candidate = address.add(NEIGHBOURS[direction]);
        if self.contains(candidate) {
            Some(candidate)
        } else {
            None
        }
    }

    /// Returns the six neighbours of an address, in direction order.
    ///
    /// A neighbour outside the world is `None`. The array is always six
    /// long, so a caller reads a direction by its index and the order does
    /// not depend on how many neighbours exist.
    #[must_use]
    pub fn neighbours(self, address: Axial) -> [Option<Axial>; NEIGHBOUR_COUNT] {
        core::array::from_fn(|direction| self.neighbour(address, direction))
    }
}
