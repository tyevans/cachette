//! Who holds a tile.
//!
//! A tile carries one holder, and the holder is a faction or nobody. The
//! holder is one dense column over the tiles, so no field of the world is
//! indexed by the faction.[^1] [^2] Exclusivity is therefore a property of
//! the storage: one tile holds one value, so no tile can name two factions
//! and no rule has to keep two factions apart.
//!
//! A faction is one bit in a 64-bit mask. A set of factions is one mask, and
//! the world stores one mask for each block of tiles. A query that asks where
//! a faction holds reads the masks, passes over every block that does not
//! name the faction, and walks only the blocks that do.[^1]
//!
//! **The count of what a faction holds is a running total.** The rule that
//! changes a holder adds one to the total of the faction that gained and
//! takes one from the total of the faction that lost. The answer therefore
//! costs nothing at read time, and maintaining it costs the tiles that
//! changed rather than the tiles that exist.[^3]
//!
//! **The spread rule reads the terrain.** A claim on a tile needs support
//! from the neighbours of the tile, and the ground says how much support the
//! tile asks for. Open water asks for more than any claim can raise, so no
//! faction ever holds water. High ground asks for more than level ground.
//!
//! **The rule reads one buffer and writes another.** Every candidate tile is
//! decided against the holders of the previous tick, so the answer does not
//! depend on the order in which the candidates were visited, and it does not
//! depend on how many threads visited them.[^4]
//!
//! **A contested tile resolves by a stable key.** The key is the support of
//! the claim, in descending order, then the faction identifier, in ascending
//! order. Nothing reads a thread completion order.[^4]
//!
//! # References
//!
//! [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
//! [^2]: ADR-0012, tiles are dense columns and units are a generational arena, decision D2. `docs/adrs/accepted/adr-0012-tiles-are-dense-columns-and-units-are-a-generational-arena.md`
//! [^3]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
//! [^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`

use bytemuck::{Pod, Zeroable};

use crate::bridge::{BlockLayout, BridgeError, UnitTileBridge};
use crate::hash::StateHash;
use crate::hex::{Axial, Grid, NEIGHBOUR_COUNT};
use crate::slots::Slots;
use crate::soldier::SoldierArena;
use crate::stage::{self, Stage};
use crate::terrain::{Terrain, TileKind};
use crate::types::{FactionId, TileIdx, FACTION_CEILING};

/// The number of bits in a faction mask.
///
/// A faction is one bit of a 64-bit word. The addressable set stops one below
/// this, because the top bit names every faction outside the addressable
/// set.[^1]
///
/// # References
///
/// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D1. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
pub const MASK_BITS: u32 = 64;

/// The bit that names a faction outside the addressable set.
///
/// Nothing sets it yet, because a world refuses a faction at or above the
/// ceiling. It is reserved so that a later minor faction does not have to
/// take an addressable slot, and so that a disjunctive query keeps working
/// when one arrives.[^1]
///
/// # References
///
/// [^1]: Research report 08, fog of war representation, section 6.4. `docs/research/reports/08-fog-of-war-representation.md`
pub const OVERFLOW_BIT: u32 = 63;

/// The holder of one tile.
///
/// The value names a faction, or nobody. It is one field, so a tile cannot
/// name two factions.[^1]
///
/// # References
///
/// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct Holder(u16);

impl Holder {
    /// The value that names no faction.
    pub const NOBODY: Self = Self(u16::MAX);

    /// Returns the holder that names one faction.
    #[must_use]
    pub const fn of(faction: FactionId) -> Self {
        Self(faction.0)
    }

    /// Returns the faction the holder names, or `None` for nobody.
    #[must_use]
    pub const fn faction(self) -> Option<FactionId> {
        if self.0 == u16::MAX {
            None
        } else {
            Some(FactionId(self.0))
        }
    }

    /// Reports whether the holder names no faction.
    #[must_use]
    pub const fn is_nobody(self) -> bool {
        self.0 == u16::MAX
    }

    /// Returns the holder as a raw number. The state hash reads it.
    #[must_use]
    pub const fn to_bits(self) -> u16 {
        self.0
    }
}

impl Default for Holder {
    fn default() -> Self {
        Self::NOBODY
    }
}

/// A set of factions.
///
/// The set is one 64-bit word, so it costs the same whatever the number of
/// factions in it. A field of the world holds one of these. A field of the
/// world never holds one value for each faction.[^1]
///
/// # References
///
/// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D3. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct FactionMask(u64);

impl FactionMask {
    /// The set that holds no faction.
    pub const EMPTY: Self = Self(0);

    /// Returns the set that holds one faction.
    ///
    /// A faction outside the addressable set takes the overflow bit, so a
    /// query that asks whether anybody holds the ground keeps working.
    #[must_use]
    pub const fn of(faction: FactionId) -> Self {
        if (faction.0 as u32) < OVERFLOW_BIT {
            Self(1u64 << faction.0)
        } else {
            Self(1u64 << OVERFLOW_BIT)
        }
    }

    /// Adds a faction to the set.
    #[must_use]
    pub const fn with(self, faction: FactionId) -> Self {
        Self(self.0 | Self::of(faction).0)
    }

    /// Returns the union of two sets.
    ///
    /// The operation is associative, commutative and exact, so a fold over a
    /// group of masks gives one answer whatever the order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D2. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Reports whether the set holds a faction.
    #[must_use]
    pub const fn contains(self, faction: FactionId) -> bool {
        self.0 & Self::of(faction).0 != 0
    }

    /// Reports whether the set holds no faction.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns the number of factions in the set.
    #[must_use]
    pub const fn len(self) -> u32 {
        self.0.count_ones()
    }

    /// Returns the set as a raw word.
    #[must_use]
    pub const fn to_bits(self) -> u64 {
        self.0
    }
}

/// The support that a unit standing on a tile gives to a claim on it.
///
/// A unit outweighs the six neighbours together, so presence takes a tile
/// from a neighbour that only surrounds it. That is what makes a holding
/// start. A world in which nobody holds anything has no neighbour to give
/// support, so without presence nothing would ever be claimed.
const PRESENCE_SUPPORT: u32 = NEIGHBOUR_COUNT as u32 + 1;

/// Returns the support that a claim on this ground must raise.
///
/// The ground decides. Open water returns `None`, and no faction ever holds
/// it. Level ground asks for one supporter, and each step upward asks for one
/// more, so a holding spreads over a plain and stops against a range of
/// mountains.[^1]
///
/// The numbers are a property of the rule and not a measurement. They are
/// ordered, and the order is what a reader and a test both need: level ground
/// is easier to hold than high ground.
///
/// # References
///
/// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D5. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
#[must_use]
pub const fn claim_threshold(kind: TileKind) -> Option<u32> {
    match kind {
        TileKind::Water => None,
        TileKind::Plain => Some(1),
        TileKind::Forest => Some(2),
        TileKind::Hill => Some(3),
        TileKind::Mountain => Some(4),
    }
}

/// The holding of a world.
///
/// It holds the holder of each tile, the list of tiles that somebody holds,
/// the count for each faction, and one faction mask for each block. The
/// holder column is the truth. The other three are derived from it, and the
/// invariant check derives each of them again and compares.[^1]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
#[derive(Clone, Debug)]
pub struct Holding {
    layout: BlockLayout,
    holders: Vec<Holder>,
    /// The tiles that somebody holds, in ascending tile order.
    held: Vec<TileIdx>,
    /// The number of tiles each faction holds, indexed by the faction bit.
    census: [i64; MASK_BITS as usize],
    /// The factions that hold ground in each block.
    block_masks: Vec<FactionMask>,
}

impl Holding {
    /// Builds a holding in which nobody holds anything.
    #[must_use]
    pub fn new(layout: BlockLayout) -> Self {
        let tiles = layout.grid().tile_count() as usize;
        Self {
            layout,
            holders: vec![Holder::NOBODY; tiles],
            held: Vec::new(),
            census: [0; MASK_BITS as usize],
            block_masks: vec![FactionMask::EMPTY; layout.block_count() as usize],
        }
    }

    /// Returns the block partition the holding indexes by.
    #[must_use]
    pub const fn layout(&self) -> BlockLayout {
        self.layout
    }

    /// Returns the grid the holding covers.
    #[must_use]
    pub const fn grid(&self) -> Grid {
        self.layout.grid()
    }

    /// Returns the holder column.
    #[must_use]
    pub fn holders(&self) -> &[Holder] {
        &self.holders
    }

    /// Returns the holder of one tile.
    ///
    /// Returns `None` when the address lies outside the world.
    #[must_use]
    pub fn holder(&self, address: Axial) -> Option<Holder> {
        let tile = self.layout.grid().index_of(address)?;
        self.holders.get(tile.0 as usize).copied()
    }

    /// Returns the number of tiles one faction holds.
    ///
    /// The call reads a running total. It walks no tile.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    #[must_use]
    pub fn holding_of(&self, faction: FactionId) -> i64 {
        self.census
            .get(faction.0 as usize)
            .copied()
            .unwrap_or_default()
    }

    /// Returns the number of tiles that somebody holds.
    #[must_use]
    pub fn held_tiles(&self) -> i64 {
        self.held.len() as i64
    }

    /// Returns the tiles that somebody holds, in ascending tile order.
    #[must_use]
    pub fn held(&self) -> &[TileIdx] {
        &self.held
    }

    /// Returns the factions that hold ground in one block.
    #[must_use]
    pub fn block_mask(&self, block: u32) -> Option<FactionMask> {
        self.block_masks.get(block as usize).copied()
    }

    /// Returns every block in which one faction holds ground.
    ///
    /// The call reads one mask for each block and returns the blocks whose
    /// mask names the faction. It reads no tile.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D3. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    pub fn blocks_held_by(&self, faction: FactionId) -> impl Iterator<Item = u32> + '_ {
        self.block_masks
            .iter()
            .enumerate()
            .filter(move |(_, mask)| mask.contains(faction))
            .map(|(block, _)| block as u32)
    }

    /// Returns every tile that one faction holds, in ascending tile order.
    ///
    /// The call walks the list of held tiles, which grows with the holding
    /// and not with the world.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    pub fn tiles_held_by(&self, faction: FactionId) -> impl Iterator<Item = TileIdx> + '_ {
        let holder = Holder::of(faction);
        self.held
            .iter()
            .copied()
            .filter(move |tile| self.holders[tile.0 as usize] == holder)
    }

    /// Folds the holding into a state hash, in tile order.
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        let hash = hash.write(bytemuck::cast_slice(&self.holders));
        self.census
            .iter()
            .fold(hash, |hash, count| hash.write_u64(*count as u64))
    }

    /// Runs the spread rule for one tick and returns the number of tiles that
    /// changed hands.
    ///
    /// The rule visits the tiles that a change can reach: the tiles somebody
    /// already holds, the neighbours of those tiles, and the tiles a unit
    /// stands on. Its cost therefore grows with the holding and with the
    /// population, and not with the world.[^1]
    ///
    /// Every candidate is decided against the holders of the previous tick,
    /// so the result does not depend on the visiting order and it does not
    /// depend on the thread count.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the derived unit structure does not describe the
    /// arena, or when it was built over another world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub fn advance(
        &mut self,
        terrain: Terrain,
        arena: &SoldierArena,
        bridge: &UnitTileBridge,
        threads: usize,
    ) -> Result<usize, BridgeError> {
        let grid = self.layout.grid();
        if grid != terrain.grid() || grid != arena.grid() {
            return Err(BridgeError::GridMismatch);
        }
        bridge.describes(arena)?;
        let threads = threads.max(1);

        let candidates = {
            let _span = stage::open(Stage::HoldingCandidates);
            self.candidates(arena)
        };
        if candidates.is_empty() {
            return Ok(0);
        }

        // Each thread fills its own slot, and the join reads the slots in
        // slot order. The chunks are contiguous runs of the candidate list,
        // so the joined result is in candidate order at every thread
        // count.[^1] Nothing reads which thread finished first.
        //
        // [^1]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
        let chunk_len = candidates.len().div_ceil(threads).max(1);
        let slot_count = candidates.len().div_ceil(chunk_len);
        let mut slots: Slots<Vec<(TileIdx, Holder)>> = Slots::filled(slot_count, Vec::new())
            .expect("the candidate list is not empty, so it needs at least one slot");
        let holders = &self.holders[..];
        let mut refusal: Option<BridgeError> = None;
        let decide_span = stage::open(Stage::HoldingDecide);
        std::thread::scope(|scope| {
            let mut handles = Vec::new();
            for (chunk, slot) in candidates.chunks(chunk_len).zip(slots.entries_mut()) {
                handles.push(scope.spawn(move || {
                    let mut changes = Vec::new();
                    let mut scratch = Scratch::new();
                    for tile in chunk {
                        let decided =
                            decide(grid, terrain, holders, arena, bridge, &mut scratch, *tile)?;
                        if let Some(holder) = decided {
                            changes.push((*tile, holder));
                        }
                    }
                    *slot = changes;
                    Ok(())
                }));
            }
            for handle in handles {
                if let Ok(Err(error)) = handle.join() {
                    refusal.get_or_insert(error);
                }
            }
        });
        drop(decide_span);
        if let Some(error) = refusal {
            return Err(error);
        }

        // The join and the write are one stage. The join is what fixes the
        // order of the result, and the write is what the order is for, so a
        // reader who wants to know what applying a decision costs wants both.
        let _span = stage::open(Stage::HoldingApply);
        let changes = slots.combine(Vec::new(), |mut joined, slot| {
            joined.extend_from_slice(slot);
            joined
        });
        self.apply(&changes);
        Ok(changes.len())
    }

    /// Returns the tiles that this tick can change, in ascending tile order.
    ///
    /// A tile inside a holding cannot change hands. Its holder draws support
    /// from all six neighbours and from holding the tile, and no challenger
    /// can raise more than that, so the list holds the edge of a holding and
    /// not its area.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    fn candidates(&self, arena: &SoldierArena) -> Vec<TileIdx> {
        let grid = self.layout.grid();
        let mut candidates: Vec<u32> = Vec::with_capacity(arena.len() as usize);
        for tile in &self.held {
            let Some(address) = grid.address_of(*tile) else {
                continue;
            };
            let holder = self.holders[tile.0 as usize];
            let neighbours = grid.neighbours(address);
            let inside = neighbours.iter().all(|neighbour| {
                neighbour
                    .and_then(|address| grid.index_of(address))
                    .is_some_and(|index| self.holders[index.0 as usize] == holder)
            });
            if inside {
                continue;
            }
            candidates.push(tile.0);
            for neighbour in neighbours.into_iter().flatten() {
                if let Some(index) = grid.index_of(neighbour) {
                    candidates.push(index.0);
                }
            }
        }
        // The arena iterates in slot order, which is fixed. The sort below
        // makes the candidate order independent of that anyway.
        for soldier in arena.iter() {
            if let Some(tile) = arena.tile(soldier) {
                candidates.push(tile.0);
            }
        }
        candidates.sort_unstable();
        candidates.dedup();
        candidates.into_iter().map(TileIdx).collect()
    }

    /// Writes the decided changes and repairs the three derived parts.
    fn apply(&mut self, changes: &[(TileIdx, Holder)]) {
        let mut dirty: Vec<u32> = Vec::with_capacity(changes.len());
        let mut moved: Vec<TileIdx> = Vec::with_capacity(changes.len());
        for (tile, holder) in changes {
            let previous = self.holders[tile.0 as usize];
            if previous == *holder {
                continue;
            }
            if let Some(faction) = previous.faction() {
                self.census[faction.0 as usize] -= 1;
            }
            if let Some(faction) = holder.faction() {
                self.census[faction.0 as usize] += 1;
            }
            self.holders[tile.0 as usize] = *holder;
            moved.push(*tile);
            if let Some(key) = self.layout.key_of(*tile) {
                dirty.push(self.layout.block_of_key(key));
            }
        }
        if moved.is_empty() {
            return;
        }

        // The held list stays in ascending tile order. A change either adds a
        // tile to it or takes one out, so the merge below reads the old list
        // once and the changed tiles once.
        moved.sort_unstable();
        moved.dedup();
        let mut held: Vec<TileIdx> = Vec::with_capacity(self.held.len() + moved.len());
        let mut cursor = 0usize;
        for tile in moved {
            while cursor < self.held.len() && self.held[cursor] < tile {
                held.push(self.held[cursor]);
                cursor += 1;
            }
            if cursor < self.held.len() && self.held[cursor] == tile {
                cursor += 1;
            }
            if !self.holders[tile.0 as usize].is_nobody() {
                held.push(tile);
            }
        }
        held.extend_from_slice(&self.held[cursor..]);
        self.held = held;

        // A block loses a bit only when the last tile of that faction leaves
        // it, and no running count can see that without reading the block. A
        // block that changed is therefore read once, and a block that did not
        // change is not read at all.
        dirty.sort_unstable();
        dirty.dedup();
        for block in dirty {
            self.block_masks[block as usize] = self.mask_of_block(block);
        }
    }

    /// Returns the factions that hold ground in one block, read from the
    /// holder column.
    fn mask_of_block(&self, block: u32) -> FactionMask {
        let grid = self.layout.grid();
        let edge = self.layout.block_edge();
        let first_column = (block % self.layout.blocks_wide()) * edge;
        let first_row = (block / self.layout.blocks_wide()) * edge;
        let mut mask = FactionMask::EMPTY;
        for row in first_row..(first_row + edge).min(grid.height()) {
            let start = (row * grid.width() + first_column) as usize;
            let end = (row * grid.width() + (first_column + edge).min(grid.width())) as usize;
            if start >= end || end > self.holders.len() {
                continue;
            }
            for holder in &self.holders[start..end] {
                if let Some(faction) = holder.faction() {
                    mask = mask.with(faction);
                }
            }
        }
        mask
    }

    /// Reports whether the holding holds its invariants.
    ///
    /// The holder column is the truth. The held list, the census and the
    /// block masks are three further declarations of the same fact, so this
    /// check derives each of them from the column and compares.[^1] It also
    /// proves the two properties the record states: no tile names a faction
    /// the world does not have, and no faction holds open water.[^2]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    /// [^2]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D5. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    #[must_use]
    pub fn check_invariants(&self, terrain: Terrain, faction_ceiling: u16) -> bool {
        let grid = self.layout.grid();
        if grid != terrain.grid() {
            return false;
        }
        if self.holders.len() != grid.tile_count() as usize {
            return false;
        }
        if self.block_masks.len() != self.layout.block_count() as usize {
            return false;
        }

        let mut census = [0i64; MASK_BITS as usize];
        let mut held: Vec<TileIdx> = Vec::new();
        let mut masks = vec![FactionMask::EMPTY; self.block_masks.len()];
        for (index, holder) in self.holders.iter().enumerate() {
            let tile = TileIdx(index as u32);
            let Some(faction) = holder.faction() else {
                continue;
            };
            if faction.0 >= faction_ceiling || faction.0 >= FACTION_CEILING {
                return false;
            }
            // No faction holds ground that admits no unit. The rule refuses
            // such a tile, and this is what fails when a later path forgets
            // to.
            let Some(address) = grid.address_of(tile) else {
                return false;
            };
            if !terrain.kind(address).is_some_and(TileKind::is_passable) {
                return false;
            }
            census[faction.0 as usize] += 1;
            held.push(tile);
            let Some(key) = self.layout.key_of(tile) else {
                return false;
            };
            let block = self.layout.block_of_key(key) as usize;
            masks[block] = masks[block].with(faction);
        }

        census == self.census && held == self.held && masks == self.block_masks
    }
}

/// The working memory of one thread that decides candidate tiles.
///
/// The tally is indexed by the faction bit, and the supporter list names the
/// entries that a tile wrote. A tile clears only what it wrote, so the cost of
/// one tile does not grow with the width of the mask.
#[derive(Clone, Debug)]
struct Scratch {
    tally: [u32; MASK_BITS as usize],
    supporters: Vec<u16>,
}

impl Scratch {
    /// Builds working memory in which no faction has support.
    fn new() -> Self {
        Self {
            tally: [0; MASK_BITS as usize],
            supporters: Vec::with_capacity(MASK_BITS as usize),
        }
    }

    /// Adds support for one faction.
    fn raise(&mut self, faction: FactionId, amount: u32) {
        let bit = faction.0 as usize;
        if bit >= self.tally.len() {
            return;
        }
        if self.tally[bit] == 0 {
            self.supporters.push(faction.0);
        }
        self.tally[bit] += amount;
    }

    /// Returns the support one faction raised.
    fn support(&self, faction: FactionId) -> u32 {
        self.tally
            .get(faction.0 as usize)
            .copied()
            .unwrap_or_default()
    }

    /// Clears what the last tile wrote.
    fn clear(&mut self) {
        for faction in self.supporters.drain(..) {
            self.tally[faction as usize] = 0;
        }
    }
}

/// Decides the holder of one candidate tile, or `None` when it does not
/// change hands.
///
/// The decision reads the holders of the previous tick and never a holder
/// that this tick wrote, so it is a pure function of the tile and of the
/// world before it.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
fn decide(
    grid: Grid,
    terrain: Terrain,
    holders: &[Holder],
    arena: &SoldierArena,
    bridge: &UnitTileBridge,
    scratch: &mut Scratch,
    tile: TileIdx,
) -> Result<Option<Holder>, BridgeError> {
    scratch.clear();
    let Some(address) = grid.address_of(tile) else {
        return Ok(None);
    };
    let Some(kind) = terrain.kind(address) else {
        return Ok(None);
    };
    let Some(threshold) = claim_threshold(kind) else {
        // The ground admits no holder. A tile of open water that somebody
        // held would be a defect, and the invariant check names it.
        return Ok(None);
    };

    for neighbour in grid.neighbours(address).into_iter().flatten() {
        let Some(index) = grid.index_of(neighbour) else {
            continue;
        };
        if let Some(faction) = holders[index.0 as usize].faction() {
            scratch.raise(faction, 1);
        }
    }
    for unit in bridge.on_tile(arena, address)? {
        if let Some(faction) = arena.faction(*unit) {
            scratch.raise(faction, PRESENCE_SUPPORT);
        }
    }

    let current = holders[tile.0 as usize];
    // A challenger must beat the holder rather than match it. The strict
    // comparison below is the whole of that rule, and it is stated once. A
    // second constant that added to the support of the holder would be the
    // same rule in two places.[^1]
    //
    // [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    let incumbent = match current.faction() {
        Some(faction) => scratch.support(faction),
        None => 0,
    };

    // The stable key is the support in descending order, then the faction
    // identifier in ascending order. The supporters are visited in ascending
    // identifier order and a later one must beat the leader strictly, which
    // gives that key without a second sort.[^1]
    //
    // [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    scratch.supporters.sort_unstable();
    let mut best: Option<(u32, u16)> = None;
    for faction in &scratch.supporters {
        if current.faction() == Some(FactionId(*faction)) {
            continue;
        }
        let support = scratch.tally[*faction as usize];
        if support < threshold || support <= incumbent {
            continue;
        }
        if best.is_none_or(|(top, _)| support > top) {
            best = Some((support, *faction));
        }
    }

    Ok(best.map(|(_, faction)| Holder::of(FactionId(faction))))
}
