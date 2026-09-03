//! Reading the stock of a tile from a ground the caller already holds.
//!
//! The stock a tile still holds is the stock it started with, less what
//! somebody took from it. The engine stores the second term only, and it
//! generates the first from the seed, the address and the ground.[^1]
//!
//! A caller that has already read the ground of an address holds that third
//! term. The reader here takes it, so the caller pays for one generation of
//! the ground and not two. The drawing is the caller this was written for: it
//! reads the ground of every visible tile to get a colour, and it read the
//! stock of the same tile through a path that generated the ground again.
//!
//! # References
//!
//! [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`

use crate::hex::Axial;
use crate::resource::{Amount, ResourceKind};
use crate::terrain::TileKind;
use crate::world::World;

impl World {
    /// Returns the stock that one tile still holds, from the ground the
    /// caller already holds.
    ///
    /// The answer is what the tile started with, less what has been taken. It
    /// equals the answer of the reader that starts from the address alone,
    /// for every caller that gives the ground of that address.[^1]
    ///
    /// **The answer follows the ground the caller gives, and the reader
    /// checks nothing.** Give the ground of this address, read from the
    /// terrain of this world. A caller that gives another ground gets the
    /// stock of a tile that does not exist.
    ///
    /// Returns `None` when the address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    #[must_use]
    pub fn tile_stock_of_ground(
        &self,
        address: Axial,
        ground: TileKind,
        kind: ResourceKind,
    ) -> Option<Amount> {
        let tile = self.grid().index_of(address)?;
        let original = self.resources().original_of_ground(address, ground, kind)?;
        Some(Amount(
            original
                .0
                .saturating_sub(self.depletion().taken(tile, kind).0),
        ))
    }
}
