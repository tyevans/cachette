//! The luxury deposits of the world, and the variety they give a faction.
//!
//! A luxury sits on a tile, and a faction that holds several kinds reaches a
//! variety level. The seeding call and the readers sit together, because the
//! readers report what the seeding placed.

use super::World;
use crate::luxury::{LuxuryError, LuxuryField, LuxuryId, LuxurySet, VarietyLevel};
use crate::types::{FactionId, TileIdx};

/// The number of luxury deposits the seeding layer places.
///
/// **This is a provisional value and not a measured one.** The balance
/// register holds the row, marks it unset, and records how this value was
/// chosen.[^1]
///
/// # References
///
/// [^1]: Balance register, the luxury deposits. `docs/reference/balance.md`
pub const LUXURY_DEPOSITS_DEFAULT: u32 = 8;

impl World {
    /// Seeds the luxuries of the world.
    ///
    /// Each placement names a tile and a luxury. The caller gives the whole
    /// set in one call, so the control plane crosses the boundary once and
    /// never loops over tiles.[^1] The caller states the placements in any
    /// order, and the engine sorts them.
    ///
    /// **The world takes a seed once.** The field is not a fact of a frame,
    /// and a reader of it never has to ask which frame it read. A second call
    /// is refused, whether or not the first one placed anything.
    ///
    /// The call derives level 1 from the field, so nothing derives it again
    /// on a later frame.
    ///
    /// # Errors
    ///
    /// Returns [`LuxuryError::AlreadySeeded`] when the world already took a
    /// seed. Returns [`LuxuryError::IdAboveCeiling`] when a placement names a
    /// luxury above the catalogue, and [`LuxuryError::NoSuchTile`] when a
    /// placement names a tile outside the world. A refusal changes nothing.
    ///
    /// # References
    ///
    /// [^1]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
    pub fn seed_luxuries(&mut self, placements: &[(TileIdx, LuxuryId)]) -> Result<(), LuxuryError> {
        if self.luxuries_seeded {
            return Err(LuxuryError::AlreadySeeded);
        }
        let field = LuxuryField::seed(self.grid, placements)?;
        self.variety = VarietyLevel::derive(self.bridge.layout(), &field);
        self.luxuries = field;
        self.luxuries_seeded = true;
        Ok(())
    }

    /// Reports whether the world has taken a luxury seed.
    #[must_use]
    pub const fn luxuries_seeded(&self) -> bool {
        self.luxuries_seeded
    }

    /// Returns the luxuries of the world.
    #[must_use]
    pub const fn luxuries(&self) -> &LuxuryField {
        &self.luxuries
    }

    /// Returns the luxuries of every level 1 cell.
    #[must_use]
    pub const fn variety_level(&self) -> &VarietyLevel {
        &self.variety
    }

    /// Returns the luxuries that one tile carries.
    ///
    /// A tile that carries none gives the empty set. A tile outside the world
    /// gives the empty set as well, because it carries nothing.
    #[must_use]
    pub fn luxuries_at(&self, tile: TileIdx) -> LuxurySet {
        self.luxuries.at(tile)
    }

    /// Returns the variety of the whole world.
    ///
    /// The variety is the number of different luxuries that stand anywhere in
    /// the world. **Nothing in the engine reads this. It is a score for the
    /// control plane, and no pass consumes it.**[^1]
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-200. `docs/DECISIONS.md`
    #[must_use]
    pub fn world_variety(&self) -> u32 {
        self.luxuries.set().variety()
    }

    /// Returns the variety of the ground that one faction holds.
    ///
    /// The answer is the number of different luxuries on the tiles of that
    /// faction. The fold runs over the luxury entries in ascending tile
    /// order, and it asks the holder column for each one, so its cost follows
    /// the number of placements and not the size of the world.[^1]
    ///
    /// **Nothing in the engine reads this.** It is a score for the control
    /// plane, and no pass consumes it.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^2]: Decisions register, DEC-200. `docs/DECISIONS.md`
    #[must_use]
    pub fn faction_variety(&self, faction: FactionId) -> u32 {
        let mut total = LuxurySet::EMPTY;
        for row in self.luxuries.tiles() {
            let Some(address) = self.grid.address_of(row.tile) else {
                continue;
            };
            let Some(holder) = self.holding.holder(address) else {
                continue;
            };
            if holder.faction() == Some(faction) {
                total = total.union(row.set);
            }
        }
        total.variety()
    }
}
