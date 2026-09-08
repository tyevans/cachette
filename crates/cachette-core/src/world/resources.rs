//! The resource column of each tile, and the recovery of a worked deposit.
//!
//! The readers answer what a tile holds, what it held at the start, and how
//! much of it a faction took. The recovery rules say how a worked deposit
//! returns. One subject, so one module.

use super::World;
use crate::hex::Axial;
use crate::resource::{Amount, DepletionLedger, RecoveryRules, ResourceField, ResourceKind};

impl World {
    /// Returns the resource field of the world.
    ///
    /// The field holds the ground, and nothing else. It costs the same at any
    /// tile count, because it stores no tile.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D1. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    #[must_use]
    pub const fn resources(&self) -> ResourceField {
        self.resources
    }

    /// Returns the stock that one tile started with.
    ///
    /// Returns `None` when the address lies outside the world.
    #[must_use]
    pub fn original_stock(&self, address: Axial, kind: ResourceKind) -> Option<Amount> {
        self.resources.original(address, kind)
    }

    /// Returns what has been taken from one tile.
    ///
    /// Returns `None` when the address lies outside the world.
    #[must_use]
    pub fn taken_from(&self, address: Axial, kind: ResourceKind) -> Option<Amount> {
        let tile = self.grid.index_of(address)?;
        Some(self.depletion.taken(tile, kind))
    }

    /// Returns the stock that one tile still holds.
    ///
    /// The answer is what the tile started with, less what has been taken.
    /// The engine stores the second term only, so a tile nobody touched costs
    /// nothing.[^1]
    ///
    /// Returns `None` when the address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    #[must_use]
    pub fn tile_stock(&self, address: Axial, kind: ResourceKind) -> Option<Amount> {
        let tile = self.grid.index_of(address)?;
        let original = self.resources.original(address, kind)?;
        Some(Amount(
            original
                .0
                .saturating_sub(self.depletion.taken(tile, kind).0),
        ))
    }

    /// Returns the depletion ledger.
    ///
    /// The ledger holds one entry for each tile and kind that somebody
    /// gathered from. A world in which nothing was gathered holds none.
    #[must_use]
    pub const fn depletion(&self) -> &DepletionLedger {
        &self.depletion
    }

    /// Returns how fast each kind of deposit recovers.
    #[must_use]
    pub const fn recovery_rules(&self) -> RecoveryRules {
        self.depletion.recovery()
    }

    /// Replaces the rules that say how fast each kind of deposit recovers.
    ///
    /// The caller replaces the whole rule set, so a period lives in one place
    /// and no two sites can hold a different value for one kind.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    pub fn set_recovery_rules(&mut self, rules: RecoveryRules) {
        self.depletion.set_recovery(rules);
    }

    /// Returns the ticks that one deposit here takes to regain one unit.
    ///
    /// **This is the answer the recovery pass acts on.** The reader calls the
    /// same rule the pass calls, over the same ground, so a caller that shows
    /// the number and a pass that uses it cannot disagree.[^1]
    ///
    /// The answer moves with the weather and with what stands on the tile. A
    /// caller that asks twice at different ticks gets two answers, and both
    /// are correct at the tick they were asked at.
    ///
    /// Returns `None` when the address lies outside the world, and when the
    /// kind does not recover at all.
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub fn recovery_period_at(&self, address: Axial, kind: ResourceKind) -> Option<u32> {
        let tile = self.grid.index_of(address)?;
        self.depletion
            .recovery()
            .period_for(kind, self.tile_ground(tile))
    }
}
