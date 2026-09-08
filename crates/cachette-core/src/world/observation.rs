//! What a faction sees now, and what it has seen before.
//!
//! Sight is per faction and it is remembered. The rules, the live readers and
//! the memory readers sit together, because a caller that asks one asks the
//! other in the next line.

use super::World;
use crate::hex::Axial;
use crate::holding::FactionMask;
use crate::observation::{Observation, SightRules};
use crate::types::{FactionId, Tick};

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

    /// Returns the cell of the summary lattice that covers one tile.
    ///
    /// The fog layers and the summary level divide the world over one
    /// lattice, so this one number names the fog block and the summary cell
    /// together.[^1]
    ///
    /// Returns `None` when the address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
    #[must_use]
    pub fn cell_covering(&self, address: Axial) -> Option<u32> {
        let layout = self.observation.layout();
        let tile = self.grid.index_of(address)?;
        Some(layout.block_of_key(layout.key_of(tile)?))
    }

    /// Returns the tick on which one faction last saw one cell of the
    /// summary lattice.
    ///
    /// Returns `None` when the faction has never seen a tile of the cell. A
    /// caller tells a memory of tick zero from no memory at all.
    ///
    /// **The engine records a tick for each observed cell and not for each
    /// observed tile.** A tick for each tile for each faction is a field of
    /// the world indexed by the faction, and the record refuses one.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D2. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    #[must_use]
    pub fn faction_cell_last_seen(&self, faction: FactionId, cell: u32) -> Option<Tick> {
        self.observation.block_last_seen(faction, cell)
    }

    /// Returns the ticks that have passed since one faction last saw one cell
    /// of the summary lattice.
    ///
    /// A cell the faction sees now reports zero, because the observation pass
    /// stamped it on the tick that just ran. A cell it saw once and does not
    /// see now reports the ticks between then and now. A cell it has never
    /// seen reports `None`.
    ///
    /// **This is the reader that separates knowledge from memory.** A
    /// remembered cell from five hundred ticks ago is not current knowledge,
    /// and a caller that reads only the two fog layers cannot tell the
    /// difference.[^1]
    ///
    /// # References
    ///
    /// [^1]: Research report 42, what a policy should be able to see, section 5.3. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
    #[must_use]
    pub fn faction_cell_age(&self, faction: FactionId, cell: u32) -> Option<u64> {
        self.observation.block_age(faction, cell, self.tick)
    }

    /// Returns the ticks that have passed since one faction last saw the cell
    /// that covers one tile.
    ///
    /// The answer is the age of the cell, because the engine records no tick
    /// finer than a cell. Two tiles of one cell therefore report one age.
    ///
    /// Returns `None` when the address lies outside the world, and when the
    /// faction has never seen a tile of the cell that covers it.
    #[must_use]
    pub fn faction_tile_age(&self, faction: FactionId, address: Axial) -> Option<u64> {
        self.faction_cell_age(faction, self.cell_covering(address)?)
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
