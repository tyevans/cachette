//! The unit arena, the tile bridge, and the readers that count a population.
//!
//! A unit lives in an arena and stands on a tile. The bridge maps the one to
//! the other. The readers here answer which units a tile carries and how many
//! people a faction holds.

use super::errors::{IdentityError, StepError};
use super::World;
use crate::bridge::{BridgeError, UnitTileBridge};
use crate::hex::Axial;
use crate::resource::{CarryLoad, ResourceKind};
use crate::soldier::{SoldierArena, SoldierError};
use crate::types::{Entity, FactionId, FACTION_CEILING};

impl World {
    /// Returns the faction of one soldier.
    ///
    /// Returns `None` when the identity names no live soldier.
    ///
    /// **This is a point read.** A caller that wants the units of a faction
    /// asks for the whole set instead, because the control plane never walks
    /// the population one unit at a time.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0010, Python is a control plane, and it never touches an entity one at a time. `docs/adrs/REGISTRY.md`
    #[must_use]
    pub fn soldier_faction(&self, entity: Entity) -> Option<FactionId> {
        self.soldiers.faction(entity)
    }

    /// Returns what one soldier carries.
    ///
    /// Returns `None` when the identity is dead.
    #[must_use]
    pub fn soldier_carry(&self, entity: Entity) -> Option<CarryLoad> {
        self.soldiers.carry(entity)
    }

    /// Returns the soldiers of the world.
    ///
    /// The soldier is one of the four fixed entity shapes, and it has its
    /// own column set.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
    #[must_use]
    pub const fn soldiers(&self) -> &SoldierArena {
        &self.soldiers
    }

    /// Resolves the value of an identity back to the soldier it names.
    ///
    /// A caller outside this crate holds an identity as the value that
    /// [`Entity::to_bits`] gave. It cannot build one, and this is the only
    /// way back.[^1]
    ///
    /// The call compares the generation the value carries against the
    /// generation the arena holds for the slot. It refuses a mismatch. It
    /// never returns the soldier that now occupies the slot, because that
    /// soldier is not the one the caller named.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the value is not an identity, when the arena
    /// holds no such slot, or when the slot holds a later generation.
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decisions D2 and D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    /// [^2]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn resolve_soldier(&self, identity: u64) -> Result<Entity, IdentityError> {
        let entity = Entity::from_bits(identity).ok_or(IdentityError::NotAnIdentity)?;
        let slot = entity.index();
        if slot >= self.soldiers.slot_count() {
            return Err(IdentityError::NoSuchSlot { slot });
        }
        if self.soldiers.contains(entity) {
            return Ok(entity);
        }
        Err(IdentityError::Stale {
            slot,
            given: entity.generation(),
            held: self.soldiers.generation_of(slot),
        })
    }

    /// Adds a soldier to the world and returns its identity.
    ///
    /// # Errors
    ///
    /// Returns an error when the arena holds no free slot, when the address
    /// is outside the world, when the ground at the address admits no
    /// unit, or when the faction is at or above the ceiling.
    pub fn spawn_soldier(
        &mut self,
        address: Axial,
        faction: FactionId,
    ) -> Result<Entity, SoldierError> {
        // The arena refuses a faction above the project ceiling. This world
        // holds a faction count of its own, which is at most that ceiling,
        // and a soldier of a faction the world does not have is a caller
        // mistake rather than a storage one.
        if faction.0 >= self.config.faction_count.max(1) {
            return Err(SoldierError::FactionAboveCeiling(faction));
        }
        self.refuse_impassable(address)?;
        self.soldiers.spawn(address, faction)
    }

    /// Removes a soldier and reports whether it removed one.
    ///
    /// A stale identity removes nothing and returns `false`.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn despawn_soldier(&mut self, entity: Entity) -> bool {
        // Read the load before the arena clears it. What the soldier carried
        // leaves the world, and conservation still has to balance, so the
        // world records where it went.[^2]
        //
        // [^2]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D5. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
        let load = self.soldiers.carry(entity);
        if !self.soldiers.despawn(entity) {
            return false;
        }
        if let Some(load) = load {
            for kind in ResourceKind::ALL {
                self.departed[kind.index()] += u64::from(load.of(kind).0);
            }
        }
        true
    }

    /// Moves a soldier to another tile.
    ///
    /// Returns `false` when the identity is dead.
    ///
    /// # Errors
    ///
    /// Returns an error when the address is outside the world, or when the
    /// ground at the address admits no unit.
    pub fn place_soldier(&mut self, entity: Entity, address: Axial) -> Result<bool, SoldierError> {
        self.refuse_impassable(address)?;
        self.soldiers.place(entity, address)
    }

    /// Returns the unit-to-tile bridge.
    ///
    /// The bridge is derived from the soldier columns, and it rebuilds at the
    /// frame barrier.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    #[must_use]
    pub const fn bridge(&self) -> &UnitTileBridge {
        &self.bridge
    }

    /// Returns the soldiers that stand on one tile.
    ///
    /// The call reads the block range, then searches inside it. It scans no
    /// population.[^1]
    ///
    /// The answer is the occupancy as it stood at the last barrier. A spawn,
    /// a despawn or a move since then makes the bridge stale, and the call
    /// then returns an error rather than a wrong answer.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the bridge is stale, or when the address is
    /// outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D4. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    /// [^2]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    pub fn soldiers_on(&self, address: Axial) -> Result<&[Entity], BridgeError> {
        self.bridge.on_tile(&self.soldiers, address)
    }

    /// Returns the number of soldiers that stand on one tile.
    ///
    /// # Errors
    ///
    /// Returns an error for the same reasons that [`Self::soldiers_on`] does.
    pub fn soldier_count_on(&self, address: Axial) -> Result<usize, BridgeError> {
        self.bridge.count_on_tile(&self.soldiers, address)
    }

    /// Returns the number of live soldiers of one faction.
    ///
    /// **This is one read, not a pass over the population.** The arena
    /// maintains the count where a slot becomes live and where it stops being
    /// live, so a caller that asks how many people a faction has left never
    /// reads a unit.[^1]
    ///
    /// A faction whose last unit ends reads zero here, and nothing else in
    /// the engine says so.
    ///
    /// # References
    ///
    /// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    #[must_use]
    pub fn population_of(&self, faction: FactionId) -> u32 {
        self.soldiers.population_of(faction)
    }

    /// Returns the live soldier count of every faction, by faction number.
    #[must_use]
    pub const fn population_by_faction(&self) -> &[u32; FACTION_CEILING as usize] {
        self.soldiers.population_by_faction()
    }

    /// Rebuilds the unit-to-tile bridge from the soldier columns.
    ///
    /// **This is the public form of the rule that a verb leaves the world
    /// readable.** A caller that spawns, removes or moves a unit outside a
    /// step calls it, and the world answers every reader again.[^1] The
    /// call compares two revisions and returns when the arena has not moved,
    /// so a caller pays for one rebuild and never for two.[^2]
    ///
    /// The step calls the same rule at each of its barriers.
    ///
    /// # Errors
    ///
    /// Returns an error when the caller asks for zero threads, or when the
    /// rebuild refuses.
    ///
    /// # References
    ///
    /// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decisions D3 and D4. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    /// [^2]: Findings register, FND-647. `docs/FINDINGS.md`
    pub fn rebuild_bridge(&mut self, threads: usize) -> Result<(), StepError> {
        if threads == 0 {
            return Err(StepError::ZeroThreads);
        }
        self.refresh_bridge()?;
        Ok(())
    }
}
