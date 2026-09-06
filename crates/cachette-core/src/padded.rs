//! A lattice with a ring of cells around it that no reader sees.
//!
//! A field solved over a bounded lattice has a defect at its border. The
//! border cell has no neighbour on the outside, so nothing arrives from that
//! side. A quantity that the solve advects therefore leaves through one edge
//! and never enters through the other, and the cells beside the edge stay
//! starved of it.
//!
//! **A padded lattice moves that defect to where no reader looks.** It holds
//! an inner lattice, which is the lattice a reader reads, and a ring of extra
//! cells around all four sides of it. The solve steps every cell of the whole
//! lattice. A reader asks for an inner cell and the lattice answers with the
//! index of that cell in the whole plane. The border defect is then at the
//! outer edge of the ring, which is a ring width away from the nearest cell
//! any reader can reach.
//!
//! # This type holds geometry only
//!
//! The type says nothing about what a plane holds or how a solve steps it. It
//! states where a cell sits, which inner cell a ring cell stands next to, and
//! which key a cell carries. A caller supplies the field.
//!
//! # The ring width is the caller's
//!
//! The type does not derive the ring width. The width follows from the
//! physics of the field, which this module does not know. A width of zero is
//! legal, and it gives the unpadded lattice back exactly: the whole lattice
//! is then the inner lattice, and the index of an inner cell is the inner
//! index itself.
//!
//! # Determinism
//!
//! Every value here is an integer, and every index map is a pure function of
//! the extent and the width.[^1] The type holds no state that a solve
//! writes.
//!
//! # References
//!
//! [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`

use crate::hex::{Axial, Grid, GridError};
use crate::types::TileIdx;

/// A lattice, and a ring of cells around it that no reader sees.
///
/// The inner lattice is the lattice a reader reads. The whole lattice is the
/// inner lattice widened by the ring width on each of the four sides, and it
/// is the lattice a solve steps.
///
/// **The whole extent and the ring width are derived from the inner extent
/// and never stored twice.** The type holds the whole grid because building
/// one costs a reciprocal, and [`PaddedLattice::new`] is the only site that
/// writes it.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaddedLattice {
    inner: Grid,
    whole: Grid,
    ring: u32,
}

impl PaddedLattice {
    /// Builds a padded lattice.
    ///
    /// A ring of zero gives a whole lattice equal to the inner lattice.
    ///
    /// # Errors
    ///
    /// Returns an error when the whole lattice does not describe a grid,
    /// which happens when the ring makes the cell count overflow the index
    /// type.
    pub fn new(inner: Grid, ring: u32) -> Result<Self, GridError> {
        let width = inner
            .width()
            .checked_add(ring.checked_mul(2).ok_or(GridError::TooManyTiles)?)
            .ok_or(GridError::TooManyTiles)?;
        let height = inner
            .height()
            .checked_add(ring.checked_mul(2).ok_or(GridError::TooManyTiles)?)
            .ok_or(GridError::TooManyTiles)?;
        Ok(Self {
            inner,
            whole: Grid::new(width, height)?,
            ring,
        })
    }

    /// Returns the lattice a reader reads.
    #[must_use]
    pub const fn inner(self) -> Grid {
        self.inner
    }

    /// Returns the lattice a solve steps.
    #[must_use]
    pub const fn whole(self) -> Grid {
        self.whole
    }

    /// Returns the cells of ring on each of the four sides.
    #[must_use]
    pub const fn ring(self) -> u32 {
        self.ring
    }

    /// Reports whether the lattice carries no ring.
    #[must_use]
    pub const fn is_bare(self) -> bool {
        self.ring == 0
    }

    /// Returns the whole-lattice index of an inner cell.
    ///
    /// Returns `None` when the index names no inner cell.
    ///
    /// **Every reader of a padded plane goes through this.** It is the one
    /// site that adds the ring offset, so a reader cannot read the plane at
    /// the wrong offset by forgetting the ring.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub fn whole_of_inner(self, inner: u32) -> Option<u32> {
        let address = self.inner.address_of(TileIdx(inner))?;
        let ring = self.ring as i32;
        self.whole
            .index_of(Axial::new(address.q + ring, address.r + ring))
            .map(|at| at.0)
    }

    /// Returns the inner index of a whole-lattice cell.
    ///
    /// Returns `None` when the cell stands in the ring, and when the index
    /// names no cell at all.
    #[must_use]
    pub fn inner_of_whole(self, whole: u32) -> Option<u32> {
        let address = self.whole.address_of(TileIdx(whole))?;
        let ring = self.ring as i32;
        self.inner
            .index_of(Axial::new(address.q - ring, address.r - ring))
            .map(|at| at.0)
    }

    /// Returns the inner cell that stands nearest to a whole-lattice cell.
    ///
    /// An inner cell answers with itself. A ring cell answers with the inner
    /// cell that its column and row clamp onto, which is the inner cell
    /// closest to it along each axis.
    ///
    /// **This is what a caller uses to continue a field into the ring.** A
    /// ring cell that copies the nearest inner cell carries ground of the
    /// same character as the edge it stands beside, rather than ground that
    /// nothing chose.
    ///
    /// Returns `None` when the index names no cell of the whole lattice.
    #[must_use]
    pub fn nearest_inner(self, whole: u32) -> Option<u32> {
        let address = self.whole.address_of(TileIdx(whole))?;
        let ring = self.ring as i32;
        let column = (address.q - ring).clamp(0, self.inner.width() as i32 - 1);
        let row = (address.r - ring).clamp(0, self.inner.height() as i32 - 1);
        self.inner.index_of(Axial::new(column, row)).map(|at| at.0)
    }

    /// Returns the column and the row of a whole-lattice cell, counted from
    /// the corner of the inner lattice.
    ///
    /// A ring cell answers with a negative number, or with a number at or
    /// above the inner extent along that axis. The reading is signed for that
    /// reason.
    ///
    /// Returns `None` when the index names no cell of the whole lattice.
    #[must_use]
    pub fn inner_address_of(self, whole: u32) -> Option<Axial> {
        let address = self.whole.address_of(TileIdx(whole))?;
        let ring = self.ring as i32;
        Some(Axial::new(address.q - ring, address.r - ring))
    }

    /// Returns the row of a whole-lattice cell inside the inner lattice,
    /// clamped to the inner rows.
    ///
    /// **A driver that reads a latitude reads this and not the whole-lattice
    /// row.** The inner lattice keeps the latitude band it had before the
    /// ring existed, so widening the ring does not move the poles of the
    /// world. A ring cell above the first inner row reads the first row, and
    /// a ring cell below the last reads the last.
    ///
    /// Returns zero when the index names no cell of the whole lattice.
    #[must_use]
    pub fn inner_row_of(self, whole: u32) -> u32 {
        let Some(address) = self.whole.address_of(TileIdx(whole)) else {
            return 0;
        };
        (address.r - self.ring as i32).clamp(0, self.inner.height() as i32 - 1) as u32
    }

    /// Returns the key that names a cell for a random draw.
    ///
    /// **An inner cell carries the same key at every ring width.** The key of
    /// an inner cell is its inner index, so a draw keyed on it gives one
    /// answer whether the lattice carries a ring or not, and a run at one
    /// ring width can be compared against a run at another.[^1]
    ///
    /// A ring cell carries a key above every inner key, so no ring cell can
    /// collide with an inner cell or with another ring cell.
    ///
    /// # References
    ///
    /// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
    #[must_use]
    pub fn draw_key(self, whole: u32) -> u32 {
        match self.inner_of_whole(whole) {
            Some(inner) => inner,
            None => self.inner.tile_count().saturating_add(whole),
        }
    }

    /// Returns the whole-lattice index of every inner cell, in inner index
    /// order.
    ///
    /// **A reader that must hand out a whole plane crops it with this.** The
    /// result is in inner index order, so a caller that indexes it by column
    /// and row reads the inner extent and never the whole one.
    #[must_use]
    pub fn inner_cells(self) -> Vec<u32> {
        (0..self.inner.tile_count())
            .filter_map(|inner| self.whole_of_inner(inner))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(width: u32, height: u32) -> Grid {
        Grid::new(width, height).expect("the grid builds")
    }

    #[test]
    fn a_bare_lattice_is_the_inner_lattice() {
        let lattice = PaddedLattice::new(grid(7, 5), 0).expect("the lattice builds");
        assert!(lattice.is_bare());
        assert_eq!(lattice.whole(), lattice.inner());
        for inner in 0..35 {
            assert_eq!(lattice.whole_of_inner(inner), Some(inner));
            assert_eq!(lattice.inner_of_whole(inner), Some(inner));
            assert_eq!(lattice.draw_key(inner), inner);
            assert_eq!(lattice.nearest_inner(inner), Some(inner));
        }
    }

    #[test]
    fn the_ring_widens_the_whole_lattice_on_four_sides() {
        let lattice = PaddedLattice::new(grid(7, 5), 3).expect("the lattice builds");
        assert_eq!(lattice.whole().width(), 13);
        assert_eq!(lattice.whole().height(), 11);
    }

    #[test]
    fn an_inner_cell_keeps_its_draw_key_at_every_ring_width() {
        let bare = PaddedLattice::new(grid(7, 5), 0).expect("the lattice builds");
        for ring in 1..4 {
            let padded = PaddedLattice::new(grid(7, 5), ring).expect("the lattice builds");
            for inner in 0..35 {
                let at = padded.whole_of_inner(inner).expect("the cell is inside");
                assert_eq!(padded.draw_key(at), bare.draw_key(inner));
            }
        }
    }

    #[test]
    fn no_ring_cell_takes_an_inner_key() {
        let lattice = PaddedLattice::new(grid(7, 5), 2).expect("the lattice builds");
        let mut seen = Vec::new();
        for whole in 0..lattice.whole().tile_count() {
            seen.push(lattice.draw_key(whole));
        }
        let mut sorted = seen.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), seen.len(), "every key is its own");
        for whole in 0..lattice.whole().tile_count() {
            if lattice.inner_of_whole(whole).is_none() {
                assert!(lattice.draw_key(whole) >= 35, "a ring key stands above");
            }
        }
    }

    #[test]
    fn a_ring_cell_clamps_onto_the_nearest_inner_cell() {
        let lattice = PaddedLattice::new(grid(4, 4), 2).expect("the lattice builds");
        // The whole lattice is eight wide. The cell at column zero, row zero
        // stands two cells out from the inner corner, so it clamps onto it.
        let corner = lattice.whole().index_of(Axial::new(0, 0)).expect("inside");
        assert_eq!(lattice.nearest_inner(corner.0), Some(0));
        // A cell beyond the far corner clamps onto the far inner cell.
        let far = lattice.whole().index_of(Axial::new(7, 7)).expect("inside");
        assert_eq!(lattice.nearest_inner(far.0), Some(15));
    }

    #[test]
    fn the_inner_row_holds_the_latitude_band_at_every_ring_width() {
        let bare = PaddedLattice::new(grid(4, 4), 0).expect("the lattice builds");
        let padded = PaddedLattice::new(grid(4, 4), 3).expect("the lattice builds");
        for inner in 0..16 {
            let at = padded.whole_of_inner(inner).expect("the cell is inside");
            assert_eq!(padded.inner_row_of(at), bare.inner_row_of(inner));
        }
    }

    #[test]
    fn the_inner_cells_come_back_in_inner_order() {
        let lattice = PaddedLattice::new(grid(3, 2), 1).expect("the lattice builds");
        let cells = lattice.inner_cells();
        assert_eq!(cells.len(), 6);
        for (inner, whole) in cells.iter().enumerate() {
            assert_eq!(lattice.inner_of_whole(*whole), Some(inner as u32));
        }
    }
}
