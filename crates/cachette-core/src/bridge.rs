//! The unit-to-tile bridge.
//!
//! A soldier holds the tile it stands on, so the map from a unit to a tile is
//! direct. The reverse map is not stored. This module derives it.[^1]
//!
//! The bridge holds a key array, a unit array, a block range array, and a
//! block occupancy bitplane.[^1] It owns no unit. It holds no fact that the
//! soldier columns do not already hold, and destroying it loses nothing.
//!
//! The bridge never sorts the arena. The slot index is half of the identity,
//! so a slot never moves.[^2]
//!
//! The bridge key is a block-major ordering of the tile address, and the
//! engine derives it by shifts and masks. The key lives on no unit and on no
//! tile.[^3] [`BlockLayout::key_of`] is the only place that derives it. A
//! second derivation would be one value declared twice, with nothing to fail
//! when the copies disagree.[^4]
//!
//! The bridge rebuilds once for each frame, at the barrier, by a sort on the
//! key.[^5] The sort takes a key vector of exact integer fields. It takes no
//! comparison function.[^6] The last field is the whole identity, taken as
//! one integer, so no two keys tie.[^7]
//!
//! # The stale read
//!
//! A caller that moves a soldier and then reads the bridge before the rebuild
//! gets an answer that looks correct and is not. This module makes that read
//! impossible to perform by accident, in two ways.
//!
//! A read takes the arena. The arena counts its own structural changes, the
//! bridge records the count it was built from, and a read against a different
//! count returns [`BridgeError::Stale`]. A comment that says "rebuild first"
//! would be the defect shape this project records.[^4]
//!
//! A read borrows the arena for as long as the caller holds the result. A
//! spawn, a despawn and a move all need the arena by mutable reference, so
//! the compiler refuses a structural change while a range is alive.
//!
//! # References
//!
//! [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D1. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
//! [^2]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
//! [^3]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D2. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
//! [^4]: Recurring defect shapes, section 1. `.claude/rules/recurring-defects.md`
//! [^5]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
//! [^6]: ADR-0007, content supplies a key vector, never a comparator, decision D1. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
//! [^7]: ADR-0007, content supplies a key vector, never a comparator, decision D2. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`

use crate::hex::{Axial, Grid};
use crate::soldier::SoldierArena;
use crate::sort::{self, SortError, SortKey};
use crate::types::{Entity, TileIdx};

/// The number of key fields. The first is the block-major tile key. The
/// second is the whole identity, which breaks every tie.[^1]
///
/// # References
///
/// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
const KEY_FIELDS: usize = 2;

/// The largest block edge exponent that a layout accepts.
///
/// A block edge of two to the fifteenth covers any world that a tile index
/// can name. The limit is the range of the index, not a budget.
pub const BLOCK_BITS_CEILING: u32 = 15;

/// The block edge exponent that a world uses when the caller states none.
///
/// The bridge partitions the world by the same block that the level of
/// detail pyramid aggregates over, so neither subsystem may choose the value
/// alone.[^1] The record that fixes the tile storage order is not written, so
/// the layout takes the exponent as a parameter and this value is only the
/// default. The research report recommends a block edge of thirty-two
/// tiles.[^2]
///
/// # References
///
/// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D2. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
/// [^2]: Report 01, the entity component system core and the memory layout, section 3. `docs/research/reports/01-ecs-and-memory-layout.md`
pub const BLOCK_BITS_DEFAULT: u32 = 5;

/// The reason that the bridge refused a caller.
///
/// Each variant is a mistake that a caller can make. The bridge returns the
/// variant. It never panics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BridgeError {
    /// The block edge exponent is above the ceiling.
    BlockBitsAboveCeiling(u32),
    /// The arena has changed since the rebuild, so every answer is stale.
    Stale {
        /// The arena revision that the bridge was built from.
        built: u64,
        /// The arena revision now.
        current: u64,
    },
    /// The bridge was never built, so it holds no answer.
    NeverBuilt,
    /// The arena describes another world than the bridge does.
    GridMismatch,
    /// The address is outside the world. The world does not wrap.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D2. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
    AddressOutsideWorld(Axial),
    /// The sort refused the key vector.
    Sort(SortError),
}

impl core::fmt::Display for BridgeError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BlockBitsAboveCeiling(bits) => write!(
                formatter,
                "the block edge exponent {bits} is above the ceiling {BLOCK_BITS_CEILING}"
            ),
            Self::Stale { built, current } => write!(
                formatter,
                "the bridge holds revision {built} and the arena holds revision {current}"
            ),
            Self::NeverBuilt => write!(formatter, "the bridge was never built"),
            Self::GridMismatch => write!(formatter, "the arena describes another world"),
            Self::AddressOutsideWorld(address) => write!(
                formatter,
                "the address ({}, {}) is outside the world",
                address.q, address.r
            ),
            Self::Sort(error) => write!(formatter, "the sort refused the keys: {error}"),
        }
    }
}

impl std::error::Error for BridgeError {}

impl From<SortError> for BridgeError {
    fn from(error: SortError) -> Self {
        Self::Sort(error)
    }
}

/// The partition of the world into blocks, and the key it derives.
///
/// The tiles of one block occupy one contiguous run of the key space. That
/// run is what makes a block range a start and a length rather than a list of
/// runs.[^1]
///
/// # References
///
/// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D2. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockLayout {
    grid: Grid,
    block_bits: u32,
    blocks_wide: u32,
    blocks_high: u32,
}

impl BlockLayout {
    /// Builds a layout over a world.
    ///
    /// The block edge is two raised to `block_bits`. An exponent of zero
    /// gives one tile for each block, which is legal and makes every search
    /// inside a block trivial.
    ///
    /// # Errors
    ///
    /// Returns an error when the exponent is above the ceiling.
    pub fn new(grid: Grid, block_bits: u32) -> Result<Self, BridgeError> {
        if block_bits > BLOCK_BITS_CEILING {
            return Err(BridgeError::BlockBitsAboveCeiling(block_bits));
        }
        let edge = 1u32 << block_bits;
        Ok(Self {
            grid,
            block_bits,
            blocks_wide: grid.width().div_ceil(edge),
            blocks_high: grid.height().div_ceil(edge),
        })
    }

    /// Returns the world that the layout partitions.
    #[must_use]
    pub const fn grid(self) -> Grid {
        self.grid
    }

    /// Returns the block edge exponent.
    #[must_use]
    pub const fn block_bits(self) -> u32 {
        self.block_bits
    }

    /// Returns the block edge in tiles.
    #[must_use]
    pub const fn block_edge(self) -> u32 {
        1 << self.block_bits
    }

    /// Returns the number of block columns.
    #[must_use]
    pub const fn blocks_wide(self) -> u32 {
        self.blocks_wide
    }

    /// Returns the number of block rows.
    #[must_use]
    pub const fn blocks_high(self) -> u32 {
        self.blocks_high
    }

    /// Returns the number of blocks that cover the world.
    #[must_use]
    pub const fn block_count(self) -> u32 {
        self.blocks_wide * self.blocks_high
    }

    /// Returns the bridge key of a tile, or `None` when the tile is outside
    /// the world.
    ///
    /// This is the one place that derives the key. The engine takes the tile
    /// address apart with shifts and masks, and it stores the result
    /// nowhere.[^1]
    ///
    /// The high part is the block, and the low part is the offset inside the
    /// block. The key of every tile of one block therefore lies in one
    /// contiguous run.
    ///
    /// # References
    ///
    /// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D2. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    #[must_use]
    pub fn key_of(self, tile: TileIdx) -> Option<u64> {
        let address = self.grid.address_of(tile)?;
        let column = address.q as u32;
        let row = address.r as u32;
        let mask = self.block_edge() - 1;
        let block = (row >> self.block_bits) * self.blocks_wide + (column >> self.block_bits);
        let inside = ((row & mask) << self.block_bits) | (column & mask);
        Some((u64::from(block) << (2 * self.block_bits)) | u64::from(inside))
    }

    /// Returns the block that a key names.
    #[must_use]
    pub const fn block_of_key(self, key: u64) -> u32 {
        (key >> (2 * self.block_bits)) as u32
    }
}

/// The start and the length of one block inside the sorted arrays.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlockRange {
    /// The index of the first unit of the block.
    pub start: u32,
    /// The number of units in the block.
    pub length: u32,
}

/// The map from a tile to the units that stand on it.
///
/// The bridge is wholly derived from the soldier columns.[^1] Two rebuilds
/// from the same columns give the same arrays, at any thread count.
///
/// # References
///
/// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D1. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
#[derive(Clone, Debug)]
pub struct UnitTileBridge {
    layout: BlockLayout,
    /// The arena revision that the last rebuild read.
    built: Option<u64>,
    /// The bridge key of each occupying unit, in key order.
    keys: Vec<u64>,
    /// The identity of each occupying unit, in the same order as the keys.
    units: Vec<Entity>,
    /// The start and the length of each block.
    ranges: Vec<BlockRange>,
    /// One bit for each block. The bit is set when the block holds a unit.
    occupancy: Vec<u64>,
}

impl UnitTileBridge {
    /// Builds an empty bridge over a layout.
    ///
    /// The bridge holds no answer until the first rebuild. A read before the
    /// first rebuild returns [`BridgeError::NeverBuilt`].
    #[must_use]
    pub fn new(layout: BlockLayout) -> Self {
        let blocks = layout.block_count() as usize;
        Self {
            layout,
            built: None,
            keys: Vec::new(),
            units: Vec::new(),
            ranges: vec![BlockRange::default(); blocks],
            occupancy: vec![0u64; blocks.div_ceil(64)],
        }
    }

    /// Returns the partition that the bridge orders by.
    #[must_use]
    pub const fn layout(&self) -> BlockLayout {
        self.layout
    }

    /// Returns the number of units that the bridge holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.units.len()
    }

    /// Reports whether the bridge holds no unit.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.units.is_empty()
    }

    /// Rebuilds the whole bridge from the soldier columns.
    ///
    /// The engine calls this once for each frame, at the barrier, after the
    /// structural apply. Every identity in the bridge is then live for the
    /// whole frame.[^1]
    ///
    /// The engine never updates the bridge while systems run. An incremental
    /// update would need a write from every system that moves a unit, and the
    /// merge order of those writes is the nondeterminism that this project
    /// cannot carry.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the arena describes another world, or when the
    /// sort refuses the key vector.
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    /// [^2]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    pub fn rebuild(&mut self, arena: &SoldierArena, threads: usize) -> Result<(), BridgeError> {
        if arena.grid() != self.layout.grid() {
            return Err(BridgeError::GridMismatch);
        }

        // The arena is read in slot order, which is explicit and stable.[^1]
        // The arena is not sorted: the slot index is half of the identity.[^2]
        //
        // [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
        // [^2]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
        let column = arena.tile_column();
        let mut units: Vec<Entity> = Vec::with_capacity(arena.len() as usize);
        let mut keys: Vec<SortKey<KEY_FIELDS>> = Vec::with_capacity(arena.len() as usize);
        for unit in arena.iter() {
            let tile = column[unit.index() as usize];
            let Some(key) = self.layout.key_of(tile) else {
                // The arena invariant keeps every live tile inside the world,
                // so a live soldier outside it is a broken arena and not a
                // caller mistake.
                return Err(BridgeError::GridMismatch);
            };
            units.push(unit);
            keys.push(SortKey::new([key, unit.to_bits()]));
        }

        let order = sort::order_on(&keys, threads)?;
        self.keys.clear();
        self.units.clear();
        for index in &order {
            let item = *index as usize;
            self.keys.push(keys[item].fields()[0]);
            self.units.push(units[item]);
        }

        self.rebuild_ranges();
        self.built = Some(arena.revision());
        Ok(())
    }

    /// Rebuilds the block range array and the occupancy bitplane.
    ///
    /// The keys are in block-major order, so the units of one block occupy
    /// one contiguous run.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D2. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    fn rebuild_ranges(&mut self) {
        for range in &mut self.ranges {
            *range = BlockRange::default();
        }
        for word in &mut self.occupancy {
            *word = 0;
        }
        let mut position = 0usize;
        while position < self.keys.len() {
            let block = self.layout.block_of_key(self.keys[position]);
            let mut end = position + 1;
            while end < self.keys.len() && self.layout.block_of_key(self.keys[end]) == block {
                end += 1;
            }
            let slot = block as usize;
            if slot < self.ranges.len() {
                self.ranges[slot] = BlockRange {
                    start: position as u32,
                    length: (end - position) as u32,
                };
                self.occupancy[slot / 64] |= 1u64 << (slot % 64);
            }
            position = end;
        }
    }

    /// Reports whether a block holds at least one unit.
    ///
    /// A query that descends the level of detail pyramid tests this and skips
    /// an empty block without reading its range.[^1]
    ///
    /// Returns `false` for a block that the world does not hold.
    ///
    /// # References
    ///
    /// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D5. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    #[must_use]
    pub fn block_is_occupied(&self, block: u32) -> bool {
        let slot = block as usize;
        if slot >= self.ranges.len() {
            return false;
        }
        self.occupancy[slot / 64] & (1u64 << (slot % 64)) != 0
    }

    /// Returns the range of a block, or `None` when the world has no such
    /// block.
    #[must_use]
    pub fn block_range(&self, block: u32) -> Option<BlockRange> {
        self.ranges.get(block as usize).copied()
    }

    /// Returns the units that stand on one tile.
    ///
    /// The call reads the range for the block that holds the tile, then
    /// searches that range for the tile key. The search is bounded by the
    /// block size and not by the unit count.[^1]
    ///
    /// The result borrows the arena, so the compiler refuses a spawn, a
    /// despawn or a move while the caller holds the range.
    ///
    /// # Errors
    ///
    /// Returns an error when the bridge was never built, when the arena has
    /// changed since the rebuild, when the arena describes another world, or
    /// when the address is outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D4. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    pub fn on_tile<'a>(
        &'a self,
        arena: &'a SoldierArena,
        address: Axial,
    ) -> Result<&'a [Entity], BridgeError> {
        self.check_fresh(arena)?;
        let tile = self
            .layout
            .grid()
            .index_of(address)
            .ok_or(BridgeError::AddressOutsideWorld(address))?;
        let key = self
            .layout
            .key_of(tile)
            .ok_or(BridgeError::AddressOutsideWorld(address))?;
        let block = self.layout.block_of_key(key);
        if !self.block_is_occupied(block) {
            return Ok(&[]);
        }
        let range = self.ranges[block as usize];
        let start = range.start as usize;
        let end = start + range.length as usize;
        let window = &self.keys[start..end];
        let low = start + window.partition_point(|held| *held < key);
        let high = start + window.partition_point(|held| *held <= key);
        Ok(&self.units[low..high])
    }

    /// Returns the units that stand inside one block.
    ///
    /// A system that needs many per-tile answers within one block reads the
    /// block once and works over the result.[^1]
    ///
    /// Returns an empty slice for a block that the world does not hold.
    ///
    /// # Errors
    ///
    /// Returns an error when the bridge was never built, when the arena has
    /// changed since the rebuild, or when the arena describes another world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D4. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    pub fn in_block<'a>(
        &'a self,
        arena: &'a SoldierArena,
        block: u32,
    ) -> Result<&'a [Entity], BridgeError> {
        self.check_fresh(arena)?;
        let Some(range) = self.block_range(block) else {
            return Ok(&[]);
        };
        let start = range.start as usize;
        Ok(&self.units[start..start + range.length as usize])
    }

    /// Returns the number of units that stand on one tile.
    ///
    /// # Errors
    ///
    /// Returns an error for the same reasons that [`Self::on_tile`] does.
    pub fn count_on_tile(
        &self,
        arena: &SoldierArena,
        address: Axial,
    ) -> Result<usize, BridgeError> {
        Ok(self.on_tile(arena, address)?.len())
    }

    /// Fails when the arena has changed since the rebuild.
    fn check_fresh(&self, arena: &SoldierArena) -> Result<(), BridgeError> {
        if arena.grid() != self.layout.grid() {
            return Err(BridgeError::GridMismatch);
        }
        let built = self.built.ok_or(BridgeError::NeverBuilt)?;
        if built == arena.revision() {
            Ok(())
        } else {
            Err(BridgeError::Stale {
                built,
                current: arena.revision(),
            })
        }
    }

    /// Reports whether the arrays agree with each other.
    ///
    /// The check reads no arena, so a caller may run it on a stale bridge.
    /// It proves that the arrays are parallel, that the keys are in order,
    /// and that each block range and each occupancy bit match the keys.
    #[must_use]
    pub fn check_structure(&self) -> bool {
        if self.keys.len() != self.units.len() {
            return false;
        }
        if self.ranges.len() != self.layout.block_count() as usize {
            return false;
        }
        if self.occupancy.len() != self.ranges.len().div_ceil(64) {
            return false;
        }
        // The order is total, so every neighbouring pair rises strictly on
        // the key and the identity together.[^1]
        //
        // [^1]: ADR-0007, content supplies a key vector, never a comparator, decision D2. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
        for index in 1..self.keys.len() {
            let before = (self.keys[index - 1], self.units[index - 1].to_bits());
            let after = (self.keys[index], self.units[index].to_bits());
            if before >= after {
                return false;
            }
        }
        let mut covered = 0usize;
        for (block, range) in self.ranges.iter().enumerate() {
            let occupied = self.block_is_occupied(block as u32);
            if occupied != (range.length > 0) {
                return false;
            }
            if range.length == 0 {
                continue;
            }
            let start = range.start as usize;
            let end = start + range.length as usize;
            if end > self.keys.len() {
                return false;
            }
            for key in &self.keys[start..end] {
                if self.layout.block_of_key(*key) as usize != block {
                    return false;
                }
            }
            covered += range.length as usize;
        }
        covered == self.keys.len()
    }

    /// Reports whether the bridge agrees with the soldier tile column.
    ///
    /// The bridge is a second declaration of where a soldier stands, and the
    /// tile column is the first. One fact in two places rots when nothing
    /// fails on disagreement, so this check is where the failure goes.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the bridge was never built, when the arena has
    /// changed since the rebuild, or when the arena describes another world.
    /// A stale bridge cannot be compared, because the columns it was derived
    /// from no longer exist.
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-040. `docs/FINDINGS.md`
    pub fn check_invariants(&self, arena: &SoldierArena) -> Result<bool, BridgeError> {
        self.check_fresh(arena)?;
        if !self.check_structure() {
            return Ok(false);
        }
        if self.units.len() != arena.len() as usize {
            return Ok(false);
        }
        let column = arena.tile_column();
        for (index, unit) in self.units.iter().enumerate() {
            if !arena.contains(*unit) {
                return Ok(false);
            }
            let tile = column[unit.index() as usize];
            if self.layout.key_of(tile) != Some(self.keys[index]) {
                return Ok(false);
            }
        }
        // Every unit is live and no two repeat, because the order above
        // rises strictly on the identity. The counts agree, so the bridge
        // holds exactly the live population.
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for the cases that the public interface cannot reach.
    //!
    //! The bridge is derived, so no public call can make it disagree with the
    //! tile column. A test of the disagreement must therefore write the key
    //! array here. The arena tests take the same route for the same
    //! reason.[^1]
    //!
    //! # References
    //!
    //! [^1]: Testing policy, section 2. `docs/TESTING.md`

    use super::*;
    use crate::types::FactionId;

    /// Builds an arena and a fresh bridge over a small world.
    fn built() -> (SoldierArena, UnitTileBridge) {
        let grid = Grid::new(8, 8).expect("a small extent describes a grid");
        let mut arena = SoldierArena::new(grid);
        arena
            .spawn(Axial::new(1, 1), FactionId(0))
            .expect("the spawn must succeed");
        arena
            .spawn(Axial::new(6, 5), FactionId(0))
            .expect("the spawn must succeed");
        let layout = BlockLayout::new(grid, 2).expect("the exponent is inside the ceiling");
        let mut bridge = UnitTileBridge::new(layout);
        bridge.rebuild(&arena, 1).expect("the rebuild must succeed");
        (arena, bridge)
    }

    #[test]
    fn a_sound_bridge_holds_its_invariants() {
        let (arena, bridge) = built();
        assert_eq!(bridge.check_invariants(&arena), Ok(true));
    }

    #[test]
    fn a_key_that_disagrees_with_the_tile_column_fails_the_check() {
        let (arena, mut bridge) = built();
        // Move the first key to another tile of the same block. The order
        // still holds, and the block range still holds, so only the
        // comparison against the tile column can catch it.
        bridge.keys[0] += 1;
        assert!(bridge.check_structure());
        assert_eq!(bridge.check_invariants(&arena), Ok(false));
    }

    #[test]
    fn a_short_unit_array_fails_the_structure_check() {
        let (_, mut bridge) = built();
        bridge.units.pop();
        assert!(!bridge.check_structure());
    }

    #[test]
    fn a_range_that_names_the_wrong_block_fails_the_structure_check() {
        let (_, mut bridge) = built();
        bridge.ranges[0] = BlockRange {
            start: 0,
            length: 2,
        };
        assert!(!bridge.check_structure());
    }

    #[test]
    fn a_bridge_that_lost_a_unit_fails_the_check() {
        let (arena, mut bridge) = built();
        bridge.keys.pop();
        bridge.units.pop();
        bridge.rebuild_ranges();
        assert!(bridge.check_structure());
        assert_eq!(bridge.check_invariants(&arena), Ok(false));
    }
}
