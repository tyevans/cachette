//! The settlement column set.
//!
//! The entity storage holds four fixed shapes, and each shape gets its own
//! set of columns.[^1] The settlement is the fixed shape. A settlement
//! carries a generational identity, the tile it stands on, a faction, and a
//! pooled store.
//!
//! The shapes do not vary at run time. A shape that is not one of the four
//! is a compile-time error here, because a column set is a Rust type and
//! not a row in a table.[^2]
//!
//! Every entity lives in the generational arena, and its identity pairs a
//! slot index with a generation.[^3] The arena mints every identity. No
//! caller builds one from parts.[^4] The generation advances when the arena
//! frees the slot, so a destroyed settlement never hands its identity to
//! the settlement founded next in that slot.[^5]
//!
//! The arena holds the tile of each settlement twice. The slot column says
//! which tile a settlement stands on. The tile column says which settlement
//! stands on a tile. One fact in two places rots when nothing fails on
//! disagreement, so the invariant check compares them.[^6]
//!
//! Every column holds an exact integer or a Q16.16 fixed-point value. No
//! column holds a floating point number.[^7] A store of zero is a real
//! state, and the store type represents it.[^8]
//!
//! # References
//!
//! [^1]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
//! [^2]: ADR-0066, entity storage holds four fixed shapes, decision D3. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
//! [^3]: ADR-0012, tiles are dense columns and units are a generational arena, decision D3. `docs/adrs/accepted/adr-0012-tiles-are-dense-columns-and-units-are-a-generational-arena.md`
//! [^4]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
//! [^5]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
//! [^6]: Findings register, FND-040. `docs/FINDINGS.md`
//! [^7]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^8]: Findings register, FND-043. `docs/FINDINGS.md`

use std::collections::VecDeque;

use crate::hash::StateHash;
use crate::hex::{Axial, Grid};
use crate::types::{Entity, FactionId, Fix32, TileIdx, FACTION_CEILING};

/// The generation that means a slot carries no identity.
///
/// A generation starts at one, so no handle ever holds this value.[^1]
///
/// # References
///
/// [^1]: ADR-0014, entity identity is an index plus a generation, decision D6. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
const NO_GENERATION: u32 = 0;

/// The first generation of a slot.
///
/// The record starts a generation at one, never at zero. Slot zero at
/// generation zero packs to the value zero, which the identity cannot
/// hold.[^1]
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

/// The number of commodities that a settlement store holds.
///
/// The set holds one commodity. A second commodity raises this number and
/// changes no code outside the store, because every read and every write
/// names a commodity by its identifier.
pub const COMMODITY_COUNT: usize = 1;

/// The identifier of a commodity in the store.
///
/// The identifier is an index into the store of a settlement. It is a
/// newtype, so a raw integer of the same width does not substitute for
/// it.[^1]
///
/// # References
///
/// [^1]: ADR-0011, every value type is a newtype with a declared size and alignment. `docs/adrs/REGISTRY.md`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommodityId(pub u16);

/// The pooled store of one settlement.
///
/// The store holds one quantity for each commodity. A quantity is a Q16.16
/// fixed-point value, so it holds a part of a unit exactly and it holds no
/// floating point number.[^1]
///
/// Zero is a real state. A settlement that holds nothing of a commodity is
/// not a settlement that holds no store, and the type represents the
/// difference.[^2]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
/// [^2]: Findings register, FND-043. `docs/FINDINGS.md`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Store {
    /// One quantity for each commodity, in commodity order.
    quantities: [Fix32; COMMODITY_COUNT],
}

// SAFETY: `Store` is `repr(transparent)` over an array of `Fix32`, which is
// itself `Pod` and `Zeroable`. A transparent wrapper adds no padding and no
// invalid bit pattern, so the state hash may read the bytes directly.
unsafe impl bytemuck::Zeroable for Store {}
// SAFETY: see the note on the `Zeroable` implementation above.
unsafe impl bytemuck::Pod for Store {}

impl Store {
    /// The store that holds nothing of any commodity.
    pub const EMPTY: Self = Self {
        quantities: [Fix32::ZERO; COMMODITY_COUNT],
    };

    /// Returns the quantity of one commodity.
    ///
    /// Returns `None` when the commodity is outside the set.
    #[must_use]
    pub fn quantity(&self, commodity: CommodityId) -> Option<Fix32> {
        self.quantities.get(commodity.0 as usize).copied()
    }

    /// Writes the quantity of one commodity and reports whether it wrote.
    ///
    /// Returns `false` when the commodity is outside the set.
    pub fn set_quantity(&mut self, commodity: CommodityId, quantity: Fix32) -> bool {
        match self.quantities.get_mut(commodity.0 as usize) {
            Some(slot) => {
                *slot = quantity;
                true
            }
            None => false,
        }
    }
}

/// The columns that a pass over the stores reads and writes.
///
/// The store column is mutable and the other two are not. A pass changes
/// what a settlement holds. It never changes which settlements exist,
/// because that is a structural change and it belongs to the arena.[^1]
///
/// # References
///
/// [^1]: ADR-0066, entity storage holds four fixed shapes, decision D2. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
#[derive(Debug)]
pub struct StoreUpdate<'a> {
    /// The pooled store of each slot.
    pub stores: &'a mut [Store],
    /// One for a live slot, zero otherwise.
    pub live: &'a [u8],
    /// The generation of each slot.
    pub generations: &'a [u32],
}

/// The reason that the arena refused a caller.
///
/// Each variant is a mistake that a caller can make. The arena returns the
/// variant. It never panics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettlementError {
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
    /// Another settlement already stands on the tile.
    ///
    /// A settlement is fixed to a tile and holds pooled stores.[^1] Two
    /// settlements on one tile would give one tile two pools, and every
    /// later question about the tile would then have two answers.
    ///
    /// # References
    ///
    /// [^1]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
    TileAlreadyHeld(Axial),
    /// The commodity is outside the commodity set.
    CommodityOutsideSet(CommodityId),
}

impl core::fmt::Display for SettlementError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ArenaFull => write!(formatter, "the settlement arena holds no free slot"),
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
            Self::TileAlreadyHeld(address) => write!(
                formatter,
                "a settlement already stands at ({}, {})",
                address.q, address.r
            ),
            Self::CommodityOutsideSet(commodity) => write!(
                formatter,
                "the commodity {} is outside the set of {COMMODITY_COUNT}",
                commodity.0
            ),
        }
    }
}

impl std::error::Error for SettlementError {}

/// The column set of the settlement shape.
///
/// The arena holds one entry for each slot it has ever opened, and it never
/// compacts the slot index space. Compaction would move a settlement to
/// another slot and invalidate every identity that names it.[^1]
///
/// The tile column is a dense array indexed by the slot. The arena never
/// looks a settlement up in a hash map, because a hash map costs a hash on
/// the hot path and carries an iteration order that no key fixes.[^2]
///
/// # References
///
/// [^1]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
/// [^2]: ADR-0014, entity identity is an index plus a generation, decision D7. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
#[derive(Clone, Debug)]
pub struct SettlementArena {
    /// The shape of the world that holds the settlements.
    grid: Grid,
    /// The largest number of slots that the arena opens.
    capacity: u32,
    /// The generation of each slot. Zero means the slot carries no identity.
    generations: Vec<u32>,
    /// One for a live slot, zero for a free slot or a retired slot.
    live: Vec<u8>,
    /// The tile of each slot.
    tiles: Vec<TileIdx>,
    /// The faction of each slot.
    factions: Vec<FactionId>,
    /// The pooled store of each slot.
    stores: Vec<Store>,
    /// The settlement that stands on each tile, in tile order.
    ///
    /// This column is the tile side of the fact that the tile column of the
    /// slots already holds. The arena keeps both, because the founding
    /// needs the tile side to refuse a second settlement in one read. The
    /// invariant check fails when the two sides disagree.[^1]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-040. `docs/FINDINGS.md`
    holders: Vec<Option<Entity>>,
    /// The free slots, oldest first.
    free: VecDeque<u32>,
    /// The number of live settlements.
    live_count: u32,
    /// The number of retired slots.
    retired_count: u32,
}

impl SettlementArena {
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
    /// A caller that asks for more settlements than the capacity gets a
    /// typed refusal.
    #[must_use]
    pub fn with_capacity(grid: Grid, capacity: u32) -> Self {
        Self {
            grid,
            capacity,
            generations: Vec::new(),
            live: Vec::new(),
            tiles: Vec::new(),
            factions: Vec::new(),
            stores: Vec::new(),
            holders: vec![None; grid.tile_count() as usize],
            free: VecDeque::new(),
            live_count: 0,
            retired_count: 0,
        }
    }

    /// Returns the world shape that the arena places settlements on.
    #[must_use]
    pub const fn grid(&self) -> Grid {
        self.grid
    }

    /// Returns the largest number of slots that the arena opens.
    #[must_use]
    pub const fn capacity(&self) -> u32 {
        self.capacity
    }

    /// Returns the number of live settlements.
    #[must_use]
    pub const fn len(&self) -> u32 {
        self.live_count
    }

    /// Reports whether the arena holds no live settlement.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.live_count == 0
    }

    /// Returns the number of slots that the arena has opened.
    ///
    /// The count never falls.[^1]
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

    /// Founds a settlement and returns its identity.
    ///
    /// The store of a new settlement holds nothing of any commodity. That
    /// is a real state and not an absent one.[^1]
    ///
    /// The arena takes the oldest free slot. It never takes the newest,
    /// because last-in first-out reuse gives one slot every generation
    /// increment and wears it out early.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the arena holds no free slot, when the address
    /// is outside the world, when the faction is at or above the ceiling,
    /// or when another settlement already stands on the tile.
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-043. `docs/FINDINGS.md`
    /// [^2]: ADR-0014, entity identity is an index plus a generation, decision D4. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn found(&mut self, address: Axial, faction: FactionId) -> Result<Entity, SettlementError> {
        let tile = self
            .grid
            .index_of(address)
            .ok_or(SettlementError::TileOutsideWorld(address))?;
        if faction.0 >= FACTION_CEILING {
            return Err(SettlementError::FactionAboveCeiling(faction));
        }
        if self.holders[tile.0 as usize].is_some() {
            return Err(SettlementError::TileAlreadyHeld(address));
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
        self.stores[index] = Store::EMPTY;
        self.live_count += 1;
        let entity = Entity::new(slot, self.generations[index])
            .expect("a generation of one or more makes the identity non-zero");
        self.holders[tile.0 as usize] = Some(entity);
        Ok(entity)
    }

    /// Opens one new slot and returns its index.
    fn open_slot(&mut self) -> Result<u32, SettlementError> {
        let slot = self.slot_count();
        if slot >= self.capacity {
            return Err(SettlementError::ArenaFull);
        }
        self.generations.push(NO_GENERATION);
        self.live.push(0);
        self.tiles.push(TileIdx(0));
        self.factions.push(FactionId(0));
        self.stores.push(Store::EMPTY);
        Ok(slot)
    }

    /// Destroys a settlement and reports whether it destroyed one.
    ///
    /// A stale identity destroys nothing and returns `false`. The arena
    /// reports no error for it, because the caller either handles the
    /// absent settlement or skips it.[^1]
    ///
    /// The generation advances here, at the free, and not at the next
    /// founding. The identity of a destroyed settlement is therefore
    /// invalid at the moment the settlement is lost, so the settlement
    /// founded next in that slot never answers to it.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    /// [^2]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn destroy(&mut self, entity: Entity) -> bool {
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
        self.holders[self.tiles[index].0 as usize] = None;
        if self.generations[index] == LAST_GENERATION {
            // The generation cannot advance, so the slot never returns. One
            // leaked slot beats two settlements that share one identity.
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
    /// generation in the slot column. A mismatch means the settlement is
    /// gone, and a dead identity resolves to nothing.[^1]
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

    /// Reports whether the identity names a live settlement.
    #[must_use]
    pub fn contains(&self, entity: Entity) -> bool {
        self.slot_of(entity).is_some()
    }

    /// Returns the tile of a settlement, or `None` when the identity is
    /// dead.
    #[must_use]
    pub fn tile(&self, entity: Entity) -> Option<TileIdx> {
        let slot = self.slot_of(entity)?;
        Some(self.tiles[slot as usize])
    }

    /// Returns the address of a settlement, or `None` when the identity is
    /// dead.
    #[must_use]
    pub fn address(&self, entity: Entity) -> Option<Axial> {
        self.grid.address_of(self.tile(entity)?)
    }

    /// Returns the faction of a settlement, or `None` when the identity is
    /// dead.
    #[must_use]
    pub fn faction(&self, entity: Entity) -> Option<FactionId> {
        let slot = self.slot_of(entity)?;
        Some(self.factions[slot as usize])
    }

    /// Returns the settlement that stands on an address.
    ///
    /// Returns `None` when the address is outside the world, and `None`
    /// when no settlement stands there. The call is one subscript. It scans
    /// no population.
    #[must_use]
    pub fn on_tile(&self, address: Axial) -> Option<Entity> {
        let tile = self.grid.index_of(address)?;
        self.holders[tile.0 as usize]
    }

    /// Returns the pooled store of a settlement.
    ///
    /// Returns `None` when the identity is dead.
    #[must_use]
    pub fn store(&self, entity: Entity) -> Option<Store> {
        let slot = self.slot_of(entity)?;
        Some(self.stores[slot as usize])
    }

    /// Writes the quantity of one commodity into the store of a settlement.
    ///
    /// Returns `false` when the identity is dead. The caller handles the
    /// absent settlement or skips it.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the commodity is outside the commodity set.
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn set_store(
        &mut self,
        entity: Entity,
        commodity: CommodityId,
        quantity: Fix32,
    ) -> Result<bool, SettlementError> {
        // Resolve the identity first. A dead handle writes nothing, whatever
        // commodity it names, so it gives `Ok(false)` and not an error about
        // a commodity that was never going to be read.
        let Some(slot) = self.slot_of(entity) else {
            return Ok(false);
        };
        if !self.stores[slot as usize].set_quantity(commodity, quantity) {
            return Err(SettlementError::CommodityOutsideSet(commodity));
        }
        Ok(true)
    }

    /// Returns the live settlements in slot order.
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
    /// wants the live settlements walks the identities instead.[^1]
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

    /// Returns the whole live column.
    ///
    /// The column holds one for a live slot and zero otherwise. A pass that
    /// reads a column beside it needs this one to know which entries mean
    /// anything.
    #[must_use]
    pub fn live_column(&self) -> &[u8] {
        &self.live
    }

    /// Returns the whole store column.
    #[must_use]
    pub fn store_column(&self) -> &[Store] {
        &self.stores
    }

    /// Returns the columns that a pass over the stores needs.
    ///
    /// A pass that maps over the stores must also know which slots are live
    /// and what generation each slot carries. One call hands over all three,
    /// because three separate calls would borrow the arena three times and
    /// one of those borrows is mutable.
    ///
    /// The arena hands out the columns and keeps the invariant. A caller
    /// writes a store and never a length, so the columns stay the same
    /// length as each other.
    pub fn store_update(&mut self) -> StoreUpdate<'_> {
        StoreUpdate {
            stores: &mut self.stores,
            live: &self.live,
            generations: &self.generations,
        }
    }

    /// Absorbs the settlement columns into the state hash.
    ///
    /// The hash covers every byte that decides a later frame. It therefore
    /// covers the generation of each slot and the free queue, because both
    /// decide which slot the next founding takes.[^1]
    ///
    /// The hash reads the slot columns and not the tile column of holders.
    /// The holders hold the same fact a second time, and the invariant
    /// check is what proves the two copies agree.
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
            .write(bytemuck::cast_slice(&self.stores))
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
    /// [^1]: Findings register, FND-040. `docs/FINDINGS.md`
    #[must_use]
    pub fn check_invariants(&self) -> bool {
        let slots = self.generations.len();
        if self.live.len() != slots
            || self.tiles.len() != slots
            || self.factions.len() != slots
            || self.stores.len() != slots
        {
            return false;
        }
        if self.holders.len() != self.grid.tile_count() as usize {
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
        // The tile column of holders states a second time where a settlement
        // stands, and the slot column states it first. A holder must name a
        // live settlement whose own tile is the tile that holds it.
        for (tile, holder) in self.holders.iter().enumerate() {
            let Some(entity) = holder else {
                continue;
            };
            let Some(slot) = self.slot_of(*entity) else {
                return false;
            };
            if self.live[slot as usize] != 1 || self.tiles[slot as usize].0 as usize != tile {
                return false;
            }
        }
        // Every live settlement appears once in the tile column. The loop
        // above proves that no holder is wrong. This count proves that no
        // live settlement is missing from the column.
        if self
            .holders
            .iter()
            .filter(|holder| holder.is_some())
            .count()
            != self.live_count as usize
        {
            return false;
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
    //! it would need four thousand million foundings. The test therefore
    //! sets the generation here.
    //!
    //! # References
    //!
    //! [^1]: ADR-0014, entity identity is an index plus a generation, decision D5. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`

    use super::*;

    /// Builds an arena over a small world.
    fn arena() -> SettlementArena {
        SettlementArena::new(Grid::new(4, 4).expect("a small extent describes a grid"))
    }

    #[test]
    fn a_slot_at_the_last_generation_retires_on_the_loss() {
        let mut arena = arena();
        let first = arena
            .found(Axial::new(0, 0), FactionId(0))
            .expect("the founding must succeed");
        arena.generations[0] = LAST_GENERATION;
        let aged = Entity::new(0, LAST_GENERATION).expect("the identity is not zero");
        arena.holders[0] = Some(aged);
        assert!(!arena.contains(first));
        assert!(arena.destroy(aged));
        assert_eq!(arena.retired_count(), 1);
        assert!(arena.free.is_empty());
        assert!(arena.check_invariants());
    }

    #[test]
    fn a_retired_slot_never_returns_to_use() {
        let mut arena = arena();
        arena
            .found(Axial::new(0, 0), FactionId(0))
            .expect("the founding must succeed");
        arena.generations[0] = LAST_GENERATION;
        let aged = Entity::new(0, LAST_GENERATION).expect("the identity is not zero");
        arena.holders[0] = Some(aged);
        assert!(arena.destroy(aged));

        let next = arena
            .found(Axial::new(1, 0), FactionId(0))
            .expect("the founding must succeed");
        assert_ne!(next.index(), 0, "a retired slot must never return");
        assert_eq!(arena.slot_count(), 2);
        assert!(arena.check_invariants());
    }

    #[test]
    fn a_loss_of_a_slot_that_is_not_live_changes_nothing() {
        // The public interface cannot reach this state, because an identity
        // resolves only while its generation matches and the arena marks a
        // slot live before it hands out the identity. The guard exists
        // because the argument that it holds runs across three functions,
        // and the failure it prevents is a count that wraps rather than a
        // panic.
        let mut arena = arena();
        let entity = arena
            .found(Axial::new(0, 0), FactionId(0))
            .expect("the founding must succeed");
        arena.live[0] = 0;
        assert!(!arena.destroy(entity));
        assert_eq!(arena.len(), 1, "the live count must not wrap");
    }

    #[test]
    fn a_free_queue_that_holds_one_slot_twice_fails_the_check() {
        let mut arena = arena();
        let entity = arena
            .found(Axial::new(0, 0), FactionId(0))
            .expect("the founding must succeed");
        assert!(arena.destroy(entity));
        assert!(arena.check_invariants());
        arena.free.push_back(0);
        assert!(!arena.check_invariants());
    }

    #[test]
    fn a_slot_that_is_neither_live_nor_queued_nor_retired_fails_the_check() {
        let mut arena = arena();
        let entity = arena
            .found(Axial::new(0, 0), FactionId(0))
            .expect("the founding must succeed");
        assert!(arena.destroy(entity));
        arena.free.clear();
        assert!(!arena.check_invariants());
    }

    #[test]
    fn a_short_column_fails_the_check() {
        let mut arena = arena();
        arena
            .found(Axial::new(0, 0), FactionId(0))
            .expect("the founding must succeed");
        assert!(arena.check_invariants());
        arena.stores.pop();
        assert!(!arena.check_invariants());
    }

    #[test]
    fn a_live_count_that_disagrees_fails_the_check() {
        let mut arena = arena();
        arena
            .found(Axial::new(0, 0), FactionId(0))
            .expect("the founding must succeed");
        arena.live_count = 7;
        assert!(!arena.check_invariants());
    }

    #[test]
    fn a_holder_on_the_wrong_tile_fails_the_check() {
        // The tile of a settlement lives in two places. Nothing else fails
        // when the two disagree.[^1]
        //
        // [^1]: Findings register, FND-040. `docs/FINDINGS.md`
        let mut arena = arena();
        let entity = arena
            .found(Axial::new(0, 0), FactionId(0))
            .expect("the founding must succeed");
        assert!(arena.check_invariants());
        arena.holders[0] = None;
        arena.holders[3] = Some(entity);
        assert!(!arena.check_invariants());
    }

    #[test]
    fn a_holder_that_names_a_dead_settlement_fails_the_check() {
        let mut arena = arena();
        let entity = arena
            .found(Axial::new(0, 0), FactionId(0))
            .expect("the founding must succeed");
        assert!(arena.destroy(entity));
        assert!(arena.check_invariants());
        arena.holders[0] = Some(entity);
        assert!(!arena.check_invariants());
    }

    #[test]
    fn a_live_settlement_missing_from_the_tile_column_fails_the_check() {
        let mut arena = arena();
        arena
            .found(Axial::new(0, 0), FactionId(0))
            .expect("the founding must succeed");
        arena.holders[0] = None;
        assert!(!arena.check_invariants());
    }
}
