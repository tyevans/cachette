//! What a faction sees now, and what it has seen before.
//!
//! Sight is per faction and it is remembered. The rules, the live readers and
//! the memory readers sit together, because a caller that asks one asks the
//! other in the next line.

use super::World;
use crate::hex::Axial;
use crate::holding::FactionMask;
use crate::observation::{Observation, SightRules};
use crate::types::FactionId;

impl World {
    /// Returns how far a unit sees, and what stops it seeing.
    #[must_use]
    pub const fn sight_rules(&self) -> SightRules {
        self.observation.rules()
    }

    /// Sets how far a unit sees, and what stops it seeing.
    ///
    /// The next step reads the new rules. The rules are a stored value that
    /// the step reads, so they enter the state hash and two worlds that
    /// differ in them diverge at once.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0164, every stored value the step reads enters the state hash, decisions D1 and D3. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    pub const fn set_sight_rules(&mut self, rules: SightRules) {
        self.observation.set_rules(rules);
    }

    /// Reports whether a faction sees a tile now.
    ///
    /// A faction sees a tile when one of its own live units observes it. A
    /// settlement gives no sight, held ground gives no sight, and an upgrade
    /// gives no sight.[^1]
    ///
    /// The answer is a reading of this frame. A faction that marched away
    /// stops seeing the tile at once.
    ///
    /// # References
    ///
    /// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D1. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    #[must_use]
    pub fn faction_sees_now(&self, faction: FactionId, address: Axial) -> bool {
        self.grid
            .index_of(address)
            .is_some_and(|tile| self.observation.sees_now(faction, tile))
    }

    /// Reports whether a faction has ever seen a tile.
    ///
    /// The remembered layer only grows. A faction that saw a tile once holds
    /// that fact for the life of the world, and the step never removes
    /// one.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D3. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    #[must_use]
    pub fn faction_has_seen(&self, faction: FactionId, address: Axial) -> bool {
        self.grid
            .index_of(address)
            .is_some_and(|tile| self.observation.has_seen(faction, tile))
    }

    /// Returns how many tiles a faction sees now.
    ///
    /// The count is widened, so it does not depend on the margin of a narrow
    /// type at the target extent.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D3. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    #[must_use]
    pub fn faction_seen_now(&self, faction: FactionId) -> i64 {
        self.observation.seen_now(faction)
    }

    /// Returns how many tiles a faction has ever seen.
    #[must_use]
    pub fn faction_seen_ever(&self, faction: FactionId) -> i64 {
        self.observation.seen_ever(faction)
    }

    /// Returns which factions see one level 1 cell now.
    ///
    /// The mask is derived from the layers of every faction, and only the
    /// observation pass writes it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D3. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    #[must_use]
    pub fn cell_seen_now(&self, cell: u32) -> FactionMask {
        self.observation.block_seen_now(cell)
    }

    /// Returns which factions have ever seen one level 1 cell.
    #[must_use]
    pub fn cell_seen_ever(&self, cell: u32) -> FactionMask {
        self.observation.block_seen_ever(cell)
    }

    /// Returns what each faction sees and what each faction remembers.
    ///
    /// A caller that walks the tiles of one faction reads the layers here
    /// rather than asking about each tile in turn.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D6. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    #[must_use]
    pub const fn observation(&self) -> &Observation {
        &self.observation
    }
}
