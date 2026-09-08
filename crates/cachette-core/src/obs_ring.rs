//! The egocentric ring frame that the spatial blocks of the observation share.
//!
//! A learner reads one array of fixed width, whatever the size of the world.
//! A field that holds one position for each cell of the world lattice cannot
//! do that, because the lattice grows with the world, and a policy trained on
//! one world then fits no other.[^1] This module states the frame that
//! replaces it.
//!
//! # What the frame is
//!
//! The frame has a centre, a set of rings at geometric radii, and a set of
//! angular sectors in each ring. Resolution falls with distance from the
//! centre, so the near ground is sampled finely and the far ground coarsely.
//! The cell count is a constant of this module, so the width of a field over
//! the frame follows nothing about the world.
//!
//! The ring index of a tile is the bit length of its hex distance from the
//! centre, capped. Ring `k` above zero therefore covers the distance band
//! from two to the power `k` less one, up to two to the power `k` minus one.
//! Each band is twice the width of the band before it.
//!
//! # Why the cap is thirteen
//!
//! The cap fixes the ring count, and it must be derived rather than chosen.
//! Ring `k` has a nominal area, which is the tile count of a full hex annulus
//! over its band. One channel of the ring stack divides the in-world tiles of
//! a cell by that nominal area, and it is the channel that lets one layout
//! serve every world size.
//!
//! A cap below the bit length of the widest distance in the world makes the
//! last ring absorb every distance above its band. The band of the last ring
//! is then unbounded, its nominal area is not a finite number, and the
//! channel that divides by it is not computable. The cap is therefore the bit
//! length of the greatest hex distance in the largest world the project
//! supports.
//!
//! The target scale is 16.7 million tiles, which is a grid of 4096 by
//! 4096.[^2] The greatest hex distance on that grid is 8190, and the bit
//! length of 8190 is 13. Ring 13 covers the band from 4096 to 8191, which the
//! target world reaches and cannot overflow. No ring is a catch-all at or
//! below the target scale, and every ring has a finite nominal area.
//!
//! The cap is a structural property of the largest supported world, so it is
//! a structural constant and not a budget.[^3]
//!
//! # The sector frame is anchored to the world axes
//!
//! Sector 0 opens on the positive `q` axial direction. The frame is
//! translation invariant and it is not rotation invariant. That is the
//! correct choice, because the action space names the six hex directions in
//! the world frame. A frame that turned with the faction would make the
//! meaning of a sector drift between decisions.
//!
//! # No trigonometry and no division
//!
//! A sextant is the half-open wedge that opens on one of the six hex
//! directions and closes before the next. Membership is two integer
//! orientation tests, and an orientation test is one multiplication and one
//! subtraction. The half split inside a sextant is a third test of the same
//! form against the bisector of the wedge. The whole of the sector arithmetic
//! is therefore exact integer comparison, which the physics of the project
//! requires.[^4]
//!
//! Every wedge is half-open, so the six sextants partition the plane with no
//! gap and no overlap. A point that lies exactly on an axis belongs to the
//! sextant that the axis opens.
//!
//! # References
//!
//! [^1]: Findings register, FND-670. `docs/FINDINGS.md`
//! [^2]: Project orientation, the target scale. `AGENTS.md`
//! [^3]: Decision Record Scope, section 4.1. `.agents/rules/adr-scope.md`
//! [^4]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`

use crate::hex::{Axial, Grid};

/// The highest ring index the frame holds.
///
/// The module documentation derives the number from the greatest hex distance
/// in the largest world the project supports.
pub const RING_CAP: u32 = 13;

/// The rings the frame holds, counting ring zero.
pub const RING_COUNT: u32 = RING_CAP + 1;

/// The sectors that ring 1 holds.
///
/// Ring 1 covers the six tiles at distance one, so a twelfth sector there
/// would hold no tile in any world. The ring drops the half split.
pub const NEAR_SECTORS: u32 = 6;

/// The sectors that every ring above ring 1 holds.
pub const FAR_SECTORS: u32 = 12;

/// The cells the frame holds.
///
/// Ring 0 holds one cell, because the centre tile has no direction. Ring 1
/// holds six. Every ring above holds twelve. **This is the only statement of
/// the cell count.** A field over the frame reads it here rather than holding
/// a second copy of the number.[^1]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
pub const RING_STACK_CELLS: u32 = 1 + NEAR_SECTORS + (RING_COUNT - 2) * FAR_SECTORS;

/// The channels that one cell of the ring stack holds.
pub const RING_STACK_CHANNELS: u32 = 25;

/// The cell index that ring 1 starts at.
const RING_ONE_START: u32 = 1;

/// The cell index that ring 2 starts at.
const RING_TWO_START: u32 = RING_ONE_START + NEAR_SECTORS;

/// The highest hex distance that the near pass covers.
///
/// The near pass reads level 0 tiles, and it covers rings 0 to 3. Those rings
/// together cover every distance below 8, which is 169 tiles. The count is
/// fixed, so the near pass costs the same on every world.
pub const NEAR_DISTANCE: u32 = 7;

/// The lowest ring that the far pass fills.
pub const FIRST_FAR_RING: u32 = 4;

/// The six hex directions, in the rotational order the sectors follow.
///
/// Direction zero is the positive `q` axial direction, and sector 0 opens on
/// it.
const AXES: [Axial; 6] = [
    Axial::new(1, 0),
    Axial::new(1, -1),
    Axial::new(0, -1),
    Axial::new(-1, 0),
    Axial::new(-1, 1),
    Axial::new(0, 1),
];

/// Returns the orientation of one axial vector against another.
///
/// The sign of the result says which side of `first` the point `second` lies
/// on. The axial basis maps to the plane by a linear map of fixed
/// determinant, so the sign is a valid orientation test on the hex plane even
/// though the two basis vectors are not perpendicular.
///
/// The product is widened to 64 bits, because the axial coordinates of the
/// target world reach 4095 and a product of two of them must not overflow the
/// accumulator.[^1]
///
/// # References
///
/// [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D3. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
const fn orientation(first: Axial, second: Axial) -> i64 {
    (first.q as i64) * (second.r as i64) - (first.r as i64) * (second.q as i64)
}

/// Returns the ring index of a hex distance.
///
/// The index is the bit length of the distance, capped at [`RING_CAP`]. A
/// distance of zero has a bit length of zero and gives ring 0.
#[must_use]
pub const fn ring_of_distance(distance: u32) -> u32 {
    let bits = u32::BITS - distance.leading_zeros();
    if bits > RING_CAP {
        RING_CAP
    } else {
        bits
    }
}

/// Returns the lowest and the highest hex distance that a ring covers.
///
/// The band of ring 0 is the single distance zero. The band of ring `k` above
/// zero runs from two to the power `k` less one to two to the power `k` minus
/// one.
#[must_use]
pub const fn ring_band(ring: u32) -> (u32, u32) {
    if ring == 0 {
        return (0, 0);
    }
    let ring = if ring > RING_CAP { RING_CAP } else { ring };
    (1 << (ring - 1), (1 << ring) - 1)
}

/// Returns the tiles that a full hex annulus over the band of a ring holds.
///
/// A hex ring at distance `d` above zero holds `6 * d` tiles, so the annulus
/// over a band is the sum of that over the band. The value is a property of
/// the geometry and of nothing else, so it is exact and it needs no
/// measurement.
#[must_use]
pub const fn ring_nominal_tiles(ring: u32) -> i64 {
    if ring == 0 {
        return 1;
    }
    let (low, high) = ring_band(ring);
    let high = high as i64;
    let low = low as i64;
    3 * (high * (high + 1) - (low - 1) * low)
}

/// Returns the sectors that a ring holds.
#[must_use]
pub const fn ring_sectors(ring: u32) -> u32 {
    if ring == 0 {
        1
    } else if ring == 1 {
        NEAR_SECTORS
    } else {
        FAR_SECTORS
    }
}

/// Returns the tiles that one cell of a ring nominally covers.
///
/// The sectors of a ring divide its annulus equally, so the nominal area of a
/// cell is the annulus divided by the sector count. The division truncates,
/// which is the rounding rule of the whole observation.
#[must_use]
pub const fn cell_nominal_tiles(ring: u32) -> i64 {
    ring_nominal_tiles(ring) / ring_sectors(ring) as i64
}

/// Returns the sextant that an axial delta lies in, or `None` at the centre.
///
/// A sextant is the half-open wedge that opens on one hex direction and
/// closes before the next. Sextant 0 opens on the positive `q` direction. A
/// delta that lies exactly on an axis belongs to the sextant that the axis
/// opens, so the six wedges partition the plane with no gap and no overlap.
///
/// The centre has no direction, so it answers `None`.
#[must_use]
pub fn sextant_of(delta: Axial) -> Option<u32> {
    if delta.q == 0 && delta.r == 0 {
        return None;
    }
    (0..6u32).find(|wedge| {
        let opening = AXES[*wedge as usize];
        let closing = AXES[(*wedge as usize + 1) % 6];
        orientation(opening, delta) <= 0 && orientation(closing, delta) > 0
    })
}

/// Returns the twelfth sector that an axial delta lies in, or `None` at the
/// centre.
///
/// The sector is the sextant, split in two by the bisector of its wedge. The
/// half that holds the axis the sextant opens on is the lower of the two. The
/// bisector is the sum of the two axes of the wedge, and the split is one
/// further orientation test.
#[must_use]
pub fn twelfth_of(delta: Axial) -> Option<u32> {
    let wedge = sextant_of(delta)?;
    let opening = AXES[wedge as usize];
    let closing = AXES[(wedge as usize + 1) % 6];
    let bisector = opening.add(closing);
    let half = u32::from(orientation(bisector, delta) <= 0);
    Some(wedge * 2 + half)
}

/// Returns the sector of a delta on the twelve-sector axis that every block
/// of the observation shares.
///
/// The centre has no direction, and it answers sector 0. A block that
/// publishes one position for each of the twelve sectors reads this, so the
/// sector axis of the ring stack and the sector axis of the frontier block
/// name the same directions.
#[must_use]
pub fn shared_sector_of(delta: Axial) -> u32 {
    twelfth_of(delta).unwrap_or(0)
}

/// Returns the hex distance of an axial delta from the centre of the frame.
#[must_use]
pub fn hex_distance_from_origin(delta: Axial) -> u32 {
    Axial::new(0, 0).distance(delta)
}

/// Returns the cell of the frame that an axial delta falls in.
///
/// The delta is the address of a tile less the centre of the frame. The cell
/// index runs ring by ring, and inside a ring it runs sector by sector, so a
/// field over the frame holds its positions in ascending cell order and needs
/// no stride table.
#[must_use]
pub fn cell_of_delta(delta: Axial) -> u32 {
    let ring = ring_of_distance(hex_distance_from_origin(delta));
    cell_of_ring_and_delta(ring, delta)
}

/// Returns the cell of a stated ring that an axial delta falls in.
///
/// The far pass reads the ring from the summary cell it walks and the
/// direction from the same delta, so it names both rather than deriving the
/// ring twice.
#[must_use]
pub fn cell_of_ring_and_delta(ring: u32, delta: Axial) -> u32 {
    if ring == 0 {
        return 0;
    }
    if ring == 1 {
        return RING_ONE_START + sextant_of(delta).unwrap_or(0);
    }
    let ring = if ring > RING_CAP { RING_CAP } else { ring };
    RING_TWO_START + (ring - 2) * FAR_SECTORS + shared_sector_of(delta)
}

/// Returns the ring that a cell of the frame belongs to.
#[must_use]
pub const fn ring_of_cell(cell: u32) -> u32 {
    if cell < RING_ONE_START {
        0
    } else if cell < RING_TWO_START {
        1
    } else {
        2 + (cell - RING_TWO_START) / FAR_SECTORS
    }
}

/// Returns the sector that a cell of the frame belongs to.
#[must_use]
pub const fn sector_of_cell(cell: u32) -> u32 {
    if cell < RING_ONE_START {
        0
    } else if cell < RING_TWO_START {
        cell - RING_ONE_START
    } else {
        (cell - RING_TWO_START) % FAR_SECTORS
    }
}

/// The distances that the area sample visits inside one ring band.
///
/// A near band holds few distances, so the sample visits every one of them
/// and the answer is exact. A far band doubles in width with every ring, so
/// the sample visits a fixed count of distances instead. The cost of the
/// sample is therefore a constant of this module, and it follows neither the
/// world size nor the observed area.
const BAND_SAMPLES: u32 = 6;

/// The positions that the area sample visits around one ring.
///
/// A hex ring at distance `d` holds `6 * d` positions, and the sample visits
/// a fixed count of them. Twenty-four positions give two samples in each of
/// the twelve sectors of a far ring.
const ROUND_SAMPLES: u32 = 24;

/// How much of each cell of the frame lies inside the world.
///
/// The two counts are a sample and not a tile walk. A far ring band covers
/// more tiles than the whole of a small world, so counting its in-world tiles
/// exactly would cost the world area, and the cost of this pass must follow
/// the observed area instead.[^1]
///
/// A near ring band holds few enough distances that the sample visits every
/// tile of it, so the counts of rings 0 to 3 are exact.
///
/// # References
///
/// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D1. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
#[derive(Clone, Debug)]
pub struct CellExtent {
    inside: Vec<i64>,
    sampled: Vec<i64>,
}

impl CellExtent {
    /// Samples how much of each cell of the frame lies inside the world.
    ///
    /// The pass walks a fixed set of addresses around the centre, in
    /// ascending distance and then in ascending position around the ring. It
    /// reads only whether the grid holds the address, so it touches no tile
    /// and it reads nothing about any faction.
    ///
    /// The cost is a constant. It does not follow the world size, and it does
    /// not follow the observed area.
    #[must_use]
    pub fn sample(grid: Grid, centre: Axial) -> Self {
        let mut extent = Self {
            inside: vec![0; RING_STACK_CELLS as usize],
            sampled: vec![0; RING_STACK_CELLS as usize],
        };
        for ring in 0..RING_COUNT {
            let (low, high) = ring_band(ring);
            for distance in sampled_distances(low, high) {
                extent.sample_round(grid, centre, ring, distance);
            }
        }
        extent
    }

    /// Adds the samples of one hex ring at one distance.
    fn sample_round(&mut self, grid: Grid, centre: Axial, ring: u32, distance: u32) {
        if distance == 0 {
            self.record(grid, centre, 0, centre);
            return;
        }
        for step in sampled_steps(distance) {
            let delta = ring_position(distance, step);
            self.record(grid, centre, ring, centre.add(delta));
        }
    }

    /// Records one sampled address against the cell it falls in.
    fn record(&mut self, grid: Grid, centre: Axial, ring: u32, address: Axial) {
        let delta = Axial::new(address.q - centre.q, address.r - centre.r);
        let cell = cell_of_ring_and_delta(ring, delta) as usize;
        let Some(sampled) = self.sampled.get_mut(cell) else {
            return;
        };
        *sampled += 1;
        if grid.contains(address) {
            if let Some(inside) = self.inside.get_mut(cell) {
                *inside += 1;
            }
        }
    }

    /// Returns the samples of one cell that fell inside the world.
    #[must_use]
    pub fn inside(&self, cell: u32) -> i64 {
        self.inside.get(cell as usize).copied().unwrap_or(0)
    }

    /// Returns the samples that one cell received.
    #[must_use]
    pub fn sampled(&self, cell: u32) -> i64 {
        self.sampled.get(cell as usize).copied().unwrap_or(0)
    }

    /// Returns how many tiles of the world one cell covers.
    ///
    /// The answer is the nominal area of the cell, scaled by the share of the
    /// sample that fell inside the world. It is the denominator of every
    /// channel that divides by the in-world tiles of a cell.
    #[must_use]
    pub fn in_world_tiles(&self, cell: u32) -> i64 {
        let sampled = self.sampled(cell);
        if sampled == 0 {
            return 0;
        }
        cell_nominal_tiles(ring_of_cell(cell)) * self.inside(cell) / sampled
    }
}

/// Returns the distances of a band that the area sample visits.
///
/// The sample visits every distance of a narrow band and a fixed count of
/// distances of a wide one, so the cost of a band does not grow with its
/// width.
fn sampled_distances(low: u32, high: u32) -> Vec<u32> {
    let width = high - low + 1;
    if width <= BAND_SAMPLES {
        return (low..=high).collect();
    }
    let stride = width / BAND_SAMPLES;
    (0..BAND_SAMPLES)
        .map(|index| low + index * stride)
        .collect()
}

/// Returns the steps around a hex ring that the area sample visits.
///
/// A hex ring at distance `d` holds `6 * d` positions. The sample visits
/// every one of them on a small ring and a fixed count of them on a large
/// one.
fn sampled_steps(distance: u32) -> Vec<u32> {
    let positions = 6 * distance;
    if positions <= ROUND_SAMPLES {
        return (0..positions).collect();
    }
    let stride = positions / ROUND_SAMPLES;
    (0..ROUND_SAMPLES)
        .map(|index| index * stride)
        .filter(|step| *step < positions)
        .collect()
}

/// Returns the axial delta of one position of a hex ring.
///
/// The positions of the ring at distance `d` run from the positive `q` axis,
/// six times `d` of them, in the rotational order the sectors follow. Step
/// `w * d + i` is the point `i` steps along the wedge that axis `w` opens.
#[must_use]
pub fn ring_position(distance: u32, step: u32) -> Axial {
    if distance == 0 {
        return Axial::new(0, 0);
    }
    let positions = 6 * distance;
    let step = step % positions;
    let wedge = (step / distance) as usize;
    let along = (step % distance) as i32;
    let remaining = distance as i32 - along;
    let opening = AXES[wedge];
    let closing = AXES[(wedge + 1) % 6];
    Axial::new(
        opening.q * remaining + closing.q * along,
        opening.r * remaining + closing.r * along,
    )
}
