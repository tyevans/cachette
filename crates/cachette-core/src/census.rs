//! A census of one bounded window of the world.
//!
//! A caller outside this crate can read one tile at a time. It cannot ask
//! what a region of ground holds, because every total over a set of tiles is
//! a loop, and the control plane never runs one.[^1] This module runs that
//! loop inside the engine and answers once.
//!
//! The window is a rectangle of axial addresses. The world is a rhombus and
//! a tile index is raw axial, so a rectangle in `q` and `r` is a contiguous
//! region and needs no coordinate conversion.[^2]
//!
//! **The cost of a census is the window, never the world.** The radius has a
//! ceiling, and the call refuses a larger one. A caller that could name the
//! whole world in one call would have a pass over the world with no name on
//! it.[^3]
//!
//! Every total here is an `i64`. A `u32` accumulator over a large window
//! reaches its ceiling on a count that a window can hold.[^4]
//!
//! # What this is not
//!
//! The viewer counts the same two things while it paints, over the tiles it
//! painted.[^5] That count and this one answer different questions. The
//! drawing pass counts what a person saw, and it skips a block the camera
//! did not reach. This counts the addresses a caller named, whether or not
//! anything drew them. Neither derives from the other.
//!
//! # References
//!
//! [^1]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
//! [^2]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D1. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
//! [^3]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
//! [^4]: ADR-0024, every summary field is declared extensive or intensive, decision D3. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
//! [^5]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`

use core::fmt;

use crate::bridge::BridgeError;
use crate::hex::Axial;
use crate::terrain::KIND_COUNT;
use crate::world::World;

/// The largest radius a census accepts.
///
/// The window of the largest radius holds 129 by 129 addresses. The number
/// is a bound on one call and not a budget: it fixes that the cost of a
/// census follows the radius the caller named and never the extent of the
/// world.[^1]
///
/// # References
///
/// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
pub const RADIUS_CEILING: u32 = 64;

/// Why a census refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CensusError {
    /// The radius is above the ceiling.
    RadiusAboveCeiling {
        /// The radius the caller asked for.
        asked: u32,
        /// The largest radius a census accepts.
        ceiling: u32,
    },
    /// The window covers no address of the world.
    WindowOutsideWorld(Axial),
    /// The unit-to-tile bridge cannot answer, so no unit count exists.
    ///
    /// The bridge is derived, and it rebuilds at the barrier. A caller that
    /// changed the population and did not step reads a stale bridge, and the
    /// census refuses rather than report a count from before the change.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    Bridge(BridgeError),
}

impl fmt::Display for CensusError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RadiusAboveCeiling { asked, ceiling } => write!(
                out,
                "the radius {asked} is above the ceiling {ceiling}; \
                 a census reads a window, never the world"
            ),
            Self::WindowOutsideWorld(centre) => write!(
                out,
                "the window around ({}, {}) covers no address of the world",
                centre.q, centre.r
            ),
            Self::Bridge(error) => write!(
                out,
                "the unit-to-tile bridge holds no answer: {error}; \
                 step the world to rebuild it"
            ),
        }
    }
}

impl core::error::Error for CensusError {}

/// What one window of the world holds.
///
/// Every field is a count of the addresses the census read, which is the
/// window clipped to the world. The corners say which addresses those were,
/// so a reader can repeat the count one address at a time and compare.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Census {
    tiles: i64,
    by_kind: [i64; KIND_COUNT],
    open_tiles: i64,
    units: i64,
    crowd_worst: u32,
    crowded_most: Option<Axial>,
    tiles_at_capacity: i64,
    first: Axial,
    last: Axial,
}

impl Census {
    /// Returns the addresses the census read.
    #[must_use]
    pub const fn tiles(&self) -> i64 {
        self.tiles
    }

    /// Returns the count of each kind of ground, in the order of the kind
    /// numbering.
    #[must_use]
    pub const fn by_kind(&self) -> &[i64; KIND_COUNT] {
        &self.by_kind
    }

    /// Returns the addresses whose ground admits a unit.
    #[must_use]
    pub const fn open_tiles(&self) -> i64 {
        self.open_tiles
    }

    /// Returns the units standing on the addresses the census read.
    #[must_use]
    pub const fn units(&self) -> i64 {
        self.units
    }

    /// Returns the largest number of units on one address of the window.
    #[must_use]
    pub const fn crowd_worst(&self) -> u32 {
        self.crowd_worst
    }

    /// Returns the address that holds that number.
    ///
    /// Returns `None` when the window holds no unit. Two addresses that hold
    /// the same number give the one the scan reached first, and the scan
    /// runs in ascending row order and then ascending column order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub const fn crowded_most(&self) -> Option<Axial> {
        self.crowded_most
    }

    /// Returns the addresses that hold at least one unit and at least as
    /// many units as the ground and its upgrade admit.
    ///
    /// An empty address is never counted, however little it admits. This is
    /// the rule the drawing pass uses, and the two counts would otherwise
    /// disagree over every tile of open water.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    #[must_use]
    pub const fn tiles_at_capacity(&self) -> i64 {
        self.tiles_at_capacity
    }

    /// Returns the lowest address the census read.
    #[must_use]
    pub const fn first(&self) -> Axial {
        self.first
    }

    /// Returns the highest address the census read.
    #[must_use]
    pub const fn last(&self) -> Axial {
        self.last
    }
}

/// Counts what one window of the world holds.
///
/// The window is the square of the given radius around the centre, clipped
/// to the world. The scan runs in ascending row order, and inside a row in
/// ascending column order, so the answer never depends on anything but the
/// world and the window.[^1]
///
/// The call writes nothing.
///
/// # Errors
///
/// Returns an error when the radius is above [`RADIUS_CEILING`], when the
/// clipped window covers no address, or when the unit-to-tile bridge cannot
/// answer.
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
pub fn census(world: &World, centre: Axial, radius: u32) -> Result<Census, CensusError> {
    if radius > RADIUS_CEILING {
        return Err(CensusError::RadiusAboveCeiling {
            asked: radius,
            ceiling: RADIUS_CEILING,
        });
    }
    let grid = world.grid();
    let reach = i64::from(radius);
    let width = i64::from(grid.width());
    let height = i64::from(grid.height());
    let first_q = (i64::from(centre.q) - reach).max(0);
    let last_q = (i64::from(centre.q) + reach).min(width - 1);
    let first_r = (i64::from(centre.r) - reach).max(0);
    let last_r = (i64::from(centre.r) + reach).min(height - 1);
    if first_q > last_q || first_r > last_r {
        return Err(CensusError::WindowOutsideWorld(centre));
    }

    let mut found = Census {
        first: address_of(first_q, first_r),
        last: address_of(last_q, last_r),
        ..Census::default()
    };
    for r in first_r..=last_r {
        for q in first_q..=last_q {
            let address = address_of(q, r);
            // The clip put every address of this scan inside the world, so
            // the ground answers for each one. The count of addresses read
            // is a field of the answer, so a caller compares it against the
            // corners rather than trusting this branch.
            let Some(kind) = world.tile_kind(address) else {
                continue;
            };
            found.tiles += 1;
            found.by_kind[kind.to_u8() as usize] += 1;
            if kind.is_passable() {
                found.open_tiles += 1;
            }
            let standing = world
                .soldier_count_on(address)
                .map_err(CensusError::Bridge)?;
            let count = u32::try_from(standing).unwrap_or(u32::MAX);
            found.units += i64::from(count);
            if count > found.crowd_worst {
                found.crowd_worst = count;
                found.crowded_most = Some(address);
            }
            if count > 0
                && world
                    .tile_capacity(address)
                    .is_some_and(|room| count >= room)
            {
                found.tiles_at_capacity += 1;
            }
        }
    }
    Ok(found)
}

/// Builds an address from two values the clip put inside the world.
fn address_of(q: i64, r: i64) -> Axial {
    Axial::new(
        i32::try_from(q).unwrap_or(i32::MAX),
        i32::try_from(r).unwrap_or(i32::MAX),
    )
}
