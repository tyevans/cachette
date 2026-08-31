//! The soldier column set.
//!
//! The entity storage holds four fixed shapes, and each shape gets its own
//! set of columns.[^1] The soldier is the mobile shape. A soldier carries a
//! generational identity, a tile address, and a faction.
//!
//! The shapes do not vary at run time. A shape that is not one of the four
//! is a compile-time error here, because a column set is a Rust type and
//! not a row in a table.[^2]
//!
//! Every entity lives in the generational arena, and its identity pairs a
//! slot index with a generation.[^3] The arena mints every identity. No
//! caller builds one from parts.[^4]
//!
//! Every column holds an exact integer. No column holds a floating point
//! number.[^5]
//!
//! # References
//!
//! [^1]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
//! [^2]: ADR-0066, entity storage holds four fixed shapes, decision D3. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
//! [^3]: ADR-0012, tiles are dense columns and units are a generational arena, decision D3. `docs/adrs/accepted/adr-0012-tiles-are-dense-columns-and-units-are-a-generational-arena.md`
//! [^4]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
//! [^5]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`

use std::collections::VecDeque;

use crate::hash::StateHash;
use crate::hex::{Axial, Grid};
use crate::types::{Entity, FactionId, TileIdx, FACTION_CEILING};

/// The generation that means a slot carries no identity.
///
/// A generation starts at one, so no handle ever holds this value.[^1] The
/// arena writes it into a slot that it has never used, and into a slot that
/// it has retired.
///
/// # References
///
/// [^1]: ADR-0014, entity identity is an index plus a generation, decision D6. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
const NO_GENERATION: u32 = 0;

/// The first generation of a slot.
///
/// The record starts a generation at one, never at zero. Slot zero at
/// generation zero packs to the value zero, which the identity cannot
/// hold, and slot zero is the first slot the arena ever allocates.[^1]
///
/// # References
///
/// [^1]: ADR-0014, entity identity is an index plus a generation, decision D6. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
const FIRST_GENERATION: u32 = 1;

/// The largest generation that a slot can hold.
///
/// A slot that reaches this value cannot advance, so the arena retires
/// it.[^1] The value is the range of the generation field, which is a
/// property of the identity layout.
///
/// # References
///
/// [^1]: ADR-0014, entity identity is an index plus a generation, decision D5. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
const LAST_GENERATION: u32 = u32::MAX;

/// The number of slots that an arena opens when the caller states no limit.
///
/// The limit is the range of the slot index, which is a property of the
/// identity layout and not a budget.
const SLOT_INDEX_LIMIT: u32 = u32::MAX;

/// The reason that the arena refused a caller.
///
/// Each variant is a mistake that a caller can make. The arena returns the
/// variant. It never panics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoldierError {
    /// The arena holds no free slot and cannot open a new one.
    ArenaFull,
    /// The address is outside the world. The world does not wrap.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D2. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
    TileOutsideWorld(Axial),
    /// The faction identifier is at or above the ceiling.
    FactionAboveCeiling(FactionId),
}

impl core::fmt::Display for SoldierError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ArenaFull => write!(formatter, "the soldier arena holds no free slot"),
            Self::TileOutsideWorld(address) => write!(
                formatter,
                "the address ({}, {}) is outside the world",
                address.q, address.r
            ),
            Self::FactionAboveCeiling(faction) => write!(
                formatter,
                "the faction {} is at or above the ceiling {FACTION_CEILING}",
                faction.0
            ),
        }
    }
}

impl std::error::Error for SoldierError {}

/// The column set of the soldier shape.
///
/// The arena holds one entry for each slot it has ever opened, and it never
/// compacts the slot index space. Compaction would move a soldier to another
/// slot and invalidate every identity that names it.[^1]
///
/// The location column is a dense array indexed by the slot. The arena never
/// looks a soldier up in a hash map, because a hash map costs a hash on the
/// hot path and carries an iteration order that no key fixes.[^2]
///
/// # References
///
/// [^1]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
/// [^2]: ADR-0014, entity identity is an index plus a generation, decision D7. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
#[derive(Clone, Debug)]
pub struct SoldierArena {
    /// The shape of the world that holds the soldiers.
    grid: Grid,
    /// The largest number of slots that the arena opens.
    capacity: u32,
    /// The generation of each slot. Zero means the slot carries no identity.
    generations: Vec<u32>,
    /// One for a live slot, zero for a free slot or a retired slot.
    live: Vec<u8>,
    /// The tile of each slot. This is the location table.
    tiles: Vec<TileIdx>,
    /// The faction of each slot.
    factions: Vec<FactionId>,
    /// The free slots, oldest first.
    free: VecDeque<u32>,
    /// The number of live soldiers.
    live_count: u32,
    /// The number of retired slots.
    retired_count: u32,
    /// The number of structural changes that the arena has taken.
    ///
    /// A spawn, a despawn and a move each raise it by one. A derived
    /// structure records the value it was built from, and it refuses a read
    /// when the value has moved on. One fact in two places needs a check
    /// that fails when the copies disagree.[^1]
    ///
    /// The counter is bookkeeping. It decides no later frame, so it does not
    /// reach the state hash.
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, section 1. `.claude/rules/recurring-defects.md`
    revision: u64,
}

impl SoldierArena {
    /// Builds an arena over a world shape.
    ///
    /// The arena opens as many slots as the slot index holds. That limit is
    /// the range of the index, not a cost budget.
    #[must_use]
    pub fn new(grid: Grid) -> Self {
        Self::with_capacity(grid, SLOT_INDEX_LIMIT)
    }

    /// Builds an arena that opens at most `capacity` slots.
    ///
    /// A caller that asks for more soldiers than the capacity gets a typed
    /// refusal. The arena reserves no memory here, so a large capacity
    /// costs nothing until a caller spawns.
    #[must_use]
    pub fn with_capacity(grid: Grid, capacity: u32) -> Self {
        Self {
            grid,
            capacity,
            generations: Vec::new(),
            live: Vec::new(),
            tiles: Vec::new(),
            factions: Vec::new(),
            free: VecDeque::new(),
            live_count: 0,
            retired_count: 0,
            revision: 0,
        }
    }

    /// Returns the world shape that the arena places soldiers on.
    #[must_use]
    pub const fn grid(&self) -> Grid {
        self.grid
    }

    /// Returns the largest number of slots that the arena opens.
    #[must_use]
    pub const fn capacity(&self) -> u32 {
        self.capacity
    }

    /// Returns the number of live soldiers.
    #[must_use]
    pub const fn len(&self) -> u32 {
        self.live_count
    }

    /// Reports whether the arena holds no live soldier.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.live_count == 0
    }

    /// Returns the number of slots that the arena has opened.
    ///
    /// The count is the high water mark of the live population plus the
    /// retired slots. It never falls.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    #[must_use]
    pub fn slot_count(&self) -> u32 {
        self.generations.len() as u32
    }

    /// Returns the number of slots that the arena has retired.
    #[must_use]
    pub const fn retired_count(&self) -> u32 {
        self.retired_count
    }

    /// Returns the number of structural changes that the arena has taken.
    ///
    /// A spawn, a despawn and a move each raise the count by one. A derived
    /// structure reads it to find out whether it is still current.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Adds a soldier and returns its identity.
    ///
    /// The arena takes the oldest free slot. It never takes the newest,
    /// because last-in first-out reuse gives one slot every generation
    /// increment and wears it out early.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the arena holds no free slot, when the address
    /// is outside the world, or when the faction is at or above the
    /// ceiling.
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D4. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn spawn(&mut self, address: Axial, faction: FactionId) -> Result<Entity, SoldierError> {
        let tile = self
            .grid
            .index_of(address)
            .ok_or(SoldierError::TileOutsideWorld(address))?;
        if faction.0 >= FACTION_CEILING {
            return Err(SoldierError::FactionAboveCeiling(faction));
        }

        let slot = match self.free.pop_front() {
            Some(slot) => slot,
            None => self.open_slot()?,
        };
        let index = slot as usize;
        if self.generations[index] == NO_GENERATION {
            self.generations[index] = FIRST_GENERATION;
        }
        self.live[index] = 1;
        self.tiles[index] = tile;
        self.factions[index] = faction;
        self.live_count += 1;
        self.revision = self.revision.wrapping_add(1);
        Ok(Entity::new(slot, self.generations[index])
            .expect("a generation of one or more makes the identity non-zero"))
    }

    /// Opens one new slot and returns its index.
    fn open_slot(&mut self) -> Result<u32, SoldierError> {
        let slot = self.slot_count();
        if slot >= self.capacity {
            return Err(SoldierError::ArenaFull);
        }
        self.generations.push(NO_GENERATION);
        self.live.push(0);
        self.tiles.push(TileIdx(0));
        self.factions.push(FactionId(0));
        Ok(slot)
    }

    /// Removes a soldier and reports whether it removed one.
    ///
    /// A stale identity removes nothing and returns `false`. The arena
    /// reports no error for it, because the caller either handles the absent
    /// soldier or skips it.[^1]
    ///
    /// The generation advances here, at the free, and not at the next
    /// allocation. The identity of a dead soldier is therefore invalid at
    /// the moment the soldier dies.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    /// [^2]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn despawn(&mut self, entity: Entity) -> bool {
        let Some(slot) = self.slot_of(entity) else {
            return false;
        };
        let index = slot as usize;
        // The identity resolved, so the slot must be live. The check is local
        // because the argument that it holds runs across three functions, and
        // an underflow here wraps the count rather than failing.
        if self.live[index] != 1 {
            return false;
        }
        self.live[index] = 0;
        self.live_count -= 1;
        self.revision = self.revision.wrapping_add(1);
        if self.generations[index] == LAST_GENERATION {
            // The generation cannot advance, so the slot never returns. One
            // leaked slot beats two soldiers that share one identity.
            self.generations[index] = NO_GENERATION;
            self.retired_count += 1;
            return true;
        }
        self.generations[index] += 1;
        self.free.push_back(slot);
        true
    }

    /// Returns the slot that an identity names, or `None` when it is dead.
    ///
    /// Resolution compares the generation in the identity against the
    /// generation in the location table. A mismatch means the soldier is
    /// dead, and a dead identity resolves to nothing.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    #[must_use]
    pub fn slot_of(&self, entity: Entity) -> Option<u32> {
        let slot = entity.index();
        let stored = *self.generations.get(slot as usize)?;
        if stored == entity.generation() {
            Some(slot)
        } else {
            None
        }
    }

    /// Reports whether the identity names a live soldier.
    #[must_use]
    pub fn contains(&self, entity: Entity) -> bool {
        self.slot_of(entity).is_some()
    }

    /// Returns the tile of a soldier, or `None` when the identity is dead.
    #[must_use]
    pub fn tile(&self, entity: Entity) -> Option<TileIdx> {
        let slot = self.slot_of(entity)?;
        Some(self.tiles[slot as usize])
    }

    /// Returns the address of a soldier, or `None` when the identity is
    /// dead.
    #[must_use]
    pub fn address(&self, entity: Entity) -> Option<Axial> {
        self.grid.address_of(self.tile(entity)?)
    }

    /// Returns the faction of a soldier, or `None` when the identity is
    /// dead.
    #[must_use]
    pub fn faction(&self, entity: Entity) -> Option<FactionId> {
        let slot = self.slot_of(entity)?;
        Some(self.factions[slot as usize])
    }

    /// Moves a soldier to another tile.
    ///
    /// Returns `false` when the identity is dead. The caller handles the
    /// absent soldier or skips it.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the address is outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn place(&mut self, entity: Entity, address: Axial) -> Result<bool, SoldierError> {
        // Resolve the identity first. A dead handle moves nothing, whatever
        // address it names, so it gives `Ok(false)` and not an error about an
        // address that was never going to be used.[^1]
        let Some(slot) = self.slot_of(entity) else {
            return Ok(false);
        };
        let tile = self
            .grid
            .index_of(address)
            .ok_or(SoldierError::TileOutsideWorld(address))?;
        self.tiles[slot as usize] = tile;
        self.revision = self.revision.wrapping_add(1);
        Ok(true)
    }

    /// Returns the live soldiers in slot order.
    ///
    /// The order is the slot order, and it is the same on every run. It is
    /// never a thread completion order and never a hash order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.live
            .iter()
            .enumerate()
            .filter(|(_, live)| **live == 1)
            .map(|(index, _)| {
                Entity::new(index as u32, self.generations[index])
                    .expect("a live slot holds a generation of one or more")
            })
    }

    /// Returns the whole tile column.
    ///
    /// The column holds one entry for each slot, live or not. A caller that
    /// wants the live soldiers walks the identities instead.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    #[must_use]
    pub fn tile_column(&self) -> &[TileIdx] {
        &self.tiles
    }

    /// Returns the whole faction column.
    #[must_use]
    pub fn faction_column(&self) -> &[FactionId] {
        &self.factions
    }

    /// Absorbs the soldier columns into the state hash.
    ///
    /// The hash covers every byte that decides a later frame. It therefore
    /// covers the generation of each slot and the free queue, because both
    /// decide which slot the next spawn takes.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        let mut hash = hash
            .write_u64(u64::from(self.slot_count()))
            .write_u64(u64::from(self.live_count))
            .write_u64(u64::from(self.retired_count))
            .write(bytemuck::cast_slice(&self.tiles))
            .write(bytemuck::cast_slice(&self.factions))
            .write(&self.live);
        for generation in &self.generations {
            hash = hash.write(&generation.to_le_bytes());
        }
        for slot in &self.free {
            hash = hash.write(&slot.to_le_bytes());
        }
        hash
    }

    /// Reports whether the arena holds its invariants.
    ///
    /// The check compares the columns against each other. One value that
    /// lives in two places needs a check that fails when the copies
    /// disagree.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, section 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub fn check_invariants(&self) -> bool {
        let slots = self.generations.len();
        if self.live.len() != slots || self.tiles.len() != slots || self.factions.len() != slots {
            return false;
        }
        if self.live.iter().filter(|live| **live == 1).count() != self.live_count as usize {
            return false;
        }
        let tile_count = self.grid.tile_count();
        for slot in 0..slots {
            if self.live[slot] == 1 {
                if self.generations[slot] == NO_GENERATION {
                    return false;
                }
                if self.tiles[slot].0 >= tile_count {
                    return false;
                }
                if self.factions[slot].0 >= FACTION_CEILING {
                    return false;
                }
            }
        }
        // A free slot is never live, and it is never retired.
        if !self.free.iter().all(|slot| {
            self.live[*slot as usize] == 0 && self.generations[*slot as usize] != NO_GENERATION
        }) {
            return false;
        }
        // No slot appears in the free queue twice. A repeat hands one slot to
        // two callers, which is the worst failure this structure has, and it
        // is the one a caller can never detect from outside.
        let mut queued = vec![0u8; slots];
        for slot in &self.free {
            let index = *slot as usize;
            if index >= slots || queued[index] == 1 {
                return false;
            }
            queued[index] = 1;
        }
        // Every slot is live, queued, or retired. A slot that is none of the
        // three is lost, and nothing else would notice.
        (0..slots).all(|slot| {
            self.live[slot] == 1 || queued[slot] == 1 || self.generations[slot] == NO_GENERATION
        })
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for the cases that the public interface cannot reach.
    //!
    //! A slot retires when its generation reaches the end of its range.[^1]
    //! A test cannot reach that end through the public interface, because
    //! it would need four thousand million spawns. The test therefore sets
    //! the generation here.
    //!
    //! # References
    //!
    //! [^1]: ADR-0014, entity identity is an index plus a generation, decision D5. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`

    use super::*;

    /// Builds an arena over a small world.
    fn arena() -> SoldierArena {
        SoldierArena::new(Grid::new(4, 4).expect("a small extent describes a grid"))
    }

    #[test]
    fn a_slot_at_the_last_generation_retires_on_the_free() {
        let mut arena = arena();
        let first = arena
            .spawn(Axial::new(0, 0), FactionId(0))
            .expect("the spawn must succeed");
        arena.generations[0] = LAST_GENERATION;
        let aged = Entity::new(0, LAST_GENERATION).expect("the identity is not zero");
        assert!(!arena.contains(first));
        assert!(arena.despawn(aged));
        assert_eq!(arena.retired_count(), 1);
        assert!(arena.free.is_empty());
        assert!(arena.check_invariants());
    }

    #[test]
    fn a_despawn_of_a_slot_that_is_not_live_changes_nothing() {
        // The public interface cannot reach this state, because an identity
        // resolves only while its generation matches and the arena marks a
        // slot live before it hands out the identity. The guard exists
        // because the argument that it holds runs across three functions,
        // and the failure it prevents is a count that wraps rather than a
        // panic. The test constructs the state the interface cannot.
        let mut arena = arena();
        let entity = arena
            .spawn(Axial::new(0, 0), FactionId(0))
            .expect("the spawn must succeed");
        arena.live[0] = 0;
        assert!(!arena.despawn(entity));
        assert_eq!(arena.len(), 1, "the live count must not wrap");
    }

    #[test]
    fn a_free_queue_that_holds_one_slot_twice_fails_the_check() {
        // One slot in the queue twice hands one slot to two callers. It is
        // the worst failure this structure has, and a caller outside cannot
        // detect it.
        let mut arena = arena();
        let entity = arena
            .spawn(Axial::new(0, 0), FactionId(0))
            .expect("the spawn must succeed");
        assert!(arena.despawn(entity));
        assert!(arena.check_invariants());
        arena.free.push_back(0);
        assert!(!arena.check_invariants());
    }

    #[test]
    fn a_slot_that_is_neither_live_nor_queued_nor_retired_fails_the_check() {
        // A lost slot is invisible: it is not live, so nothing reads it, and
        // it is not queued, so nothing reuses it. The arena simply shrinks.
        let mut arena = arena();
        let entity = arena
            .spawn(Axial::new(0, 0), FactionId(0))
            .expect("the spawn must succeed");
        assert!(arena.despawn(entity));
        arena.free.clear();
        assert!(!arena.check_invariants());
    }

    #[test]
    fn a_retired_slot_never_returns_to_use() {
        let mut arena = arena();
        arena
            .spawn(Axial::new(0, 0), FactionId(0))
            .expect("the spawn must succeed");
        arena.generations[0] = LAST_GENERATION;
        let aged = Entity::new(0, LAST_GENERATION).expect("the identity is not zero");
        assert!(arena.despawn(aged));

        let next = arena
            .spawn(Axial::new(1, 0), FactionId(0))
            .expect("the spawn must succeed");
        assert_ne!(next.index(), 0, "a retired slot must never return");
        assert_eq!(arena.slot_count(), 2);
        assert!(arena.check_invariants());
    }

    #[test]
    fn a_retired_slot_resolves_no_identity() {
        let mut arena = arena();
        arena
            .spawn(Axial::new(0, 0), FactionId(0))
            .expect("the spawn must succeed");
        arena.generations[0] = LAST_GENERATION;
        let aged = Entity::new(0, LAST_GENERATION).expect("the identity is not zero");
        assert!(arena.despawn(aged));
        assert!(!arena.contains(aged));
        assert_eq!(arena.tile(aged), None);
    }

    #[test]
    fn a_short_column_fails_the_check() {
        let mut arena = arena();
        arena
            .spawn(Axial::new(0, 0), FactionId(0))
            .expect("the spawn must succeed");
        assert!(arena.check_invariants());
        arena.factions.pop();
        assert!(!arena.check_invariants());
    }

    #[test]
    fn a_live_count_that_disagrees_fails_the_check() {
        let mut arena = arena();
        arena
            .spawn(Axial::new(0, 0), FactionId(0))
            .expect("the spawn must succeed");
        arena.live_count = 7;
        assert!(!arena.check_invariants());
    }
}
