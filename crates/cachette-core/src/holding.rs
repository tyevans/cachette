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
/// Counts what the holding apply does on each frame.
///
/// **The switch exists because the apply has three parts and the stage table
/// names only the whole.** The apply writes the holder of each changed tile,
/// rebuilds the list of held tiles, and repairs the block mask of every block
/// a change touched. Those grow with different things, and a figure for the
/// stage says nothing about which one carries it.
///
/// The counters observe. Nothing reads them inside the engine, and no
/// simulated value depends on one, so they cannot reach a result.[^1]
///
/// The whole module compiles to nothing when the switch is off.
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
#[cfg(feature = "census-holding")]
pub mod census {
    use core::sync::atomic::{AtomicU64, Ordering};

    /// The tiles whose holder changed, since the last reset.
    static MOVED: AtomicU64 = AtomicU64::new(0);
    /// The blocks whose mask was read again, since the last reset.
    static DIRTY: AtomicU64 = AtomicU64::new(0);
    /// The entries the held list was rebuilt with, since the last reset.
    static REBUILT: AtomicU64 = AtomicU64::new(0);

    /// Records one apply.
    pub fn record(moved: u64, dirty: u64, rebuilt: u64) {
        MOVED.fetch_add(moved, Ordering::Relaxed);
        DIRTY.fetch_add(dirty, Ordering::Relaxed);
        REBUILT.fetch_add(rebuilt, Ordering::Relaxed);
    }

    /// Returns the moved tiles, the dirty blocks and the rebuilt entries.
    #[must_use]
    pub fn totals() -> (u64, u64, u64) {
        (
            MOVED.load(Ordering::Relaxed),
            DIRTY.load(Ordering::Relaxed),
            REBUILT.load(Ordering::Relaxed),
        )
    }

    /// Sets every count back to zero.
    pub fn reset() {
        MOVED.store(0, Ordering::Relaxed);
        DIRTY.store(0, Ordering::Relaxed);
        REBUILT.store(0, Ordering::Relaxed);
    }
}

use crate::hash::StateHash;
use crate::hex::{Axial, Grid, NEIGHBOUR_COUNT};
use crate::slots::Slots;
use crate::soldier::SoldierArena;
use crate::stage::{self, Stage};
use crate::terrain::{Terrain, TileKind};
use crate::types::{Entity, FactionId, TileIdx, FACTION_CEILING};

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

    /// Returns the set with one faction removed.
    ///
    /// Removing a faction the set does not hold gives the same set.
    #[must_use]
    pub const fn without(self, faction: FactionId) -> Self {
        if faction.0 as u32 >= MASK_BITS {
            return self;
        }
        Self(self.0 & !(1u64 << faction.0))
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
    ///
    /// The mask is derived from the counts below and is kept beside them, so
    /// that a reader of one block pays one read rather than one for each
    /// faction. The invariant check derives both from the holder column and
    /// fails when either disagrees.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    block_masks: Vec<FactionMask>,
    /// The number of tiles each faction holds in each block.
    ///
    /// The entry of a block and a faction sits at the block index times the
    /// mask width plus the faction bit.
    ///
    /// **This exists so that a mask never has to be read again from the
    /// tiles.** A block loses a faction bit exactly when that faction's count
    /// in the block reaches zero, and a count sees that in one step. Reading
    /// the block instead costs every tile of it, and a frame at the target
    /// scale dirties most blocks.[^1]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-301. `docs/FINDINGS.md`
    block_census: Vec<u32>,
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
            block_census: vec![0; layout.block_count() as usize * MASK_BITS as usize],
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
    /// stands on.[^1]
    ///
    /// **The holding is not small.** At one million units scattered over the
    /// target world it reaches 39 percent of the tiles, so a cost that grows
    /// with the holding grows with the world in practice.[^3] The pass that
    /// chooses the tiles therefore holds a set of the world rather than a
    /// list of what it touched, and it takes a thread count.
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
    /// [^3]: Findings register, FND-285. `docs/FINDINGS.md`
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
            self.candidates(arena, threads)
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
        let layout = self.layout;
        let decide_span = stage::open(Stage::HoldingDecide);
        std::thread::scope(|scope| {
            let mut handles = Vec::new();
            for (chunk, slot) in candidates.chunks(chunk_len).zip(slots.entries_mut()) {
                handles.push(scope.spawn(move || {
                    let mut changes = Vec::new();
                    let mut scratch = Scratch::new(layout.block_count() as usize);
                    for tile in chunk {
                        if let Some(holder) =
                            decide(layout, terrain, holders, arena, bridge, &mut scratch, *tile)
                        {
                            changes.push((*tile, holder));
                        }
                    }
                    *slot = changes;
                }));
            }
            for handle in handles {
                // A thread here reads shared memory and writes its own slot.
                // The freshness of the derived unit structure was established
                // once, before the walk started, so no thread can refuse.
                handle.join().expect("a decide thread cannot fail");
            }
        });
        drop(decide_span);

        // The join and the write are one stage. The join is what fixes the
        // order of the result, and the write is what the order is for, so a
        // reader who wants to know what applying a decision costs wants both.
        let _span = stage::open(Stage::HoldingApply);
        let changes = slots.combine(Vec::new(), |mut joined, slot| {
            joined.extend_from_slice(slot);
            joined
        });
        self.apply(&changes, threads);
        Ok(changes.len())
    }

    /// Returns the tiles that this tick can change, in ascending tile order.
    ///
    /// A tile inside a holding cannot change hands. Its holder draws support
    /// from all six neighbours and from holding the tile, and no challenger
    /// can raise more than that, so the list holds the edge of a holding and
    /// not its area.[^1]
    ///
    /// **The answer is a set, so the pass builds a set and not a list.** One
    /// bit for each tile of the world holds it. A tile that two sources reach
    /// sets the same bit twice, and the scan that reads the bits back visits
    /// the words in ascending order, so the result is in ascending tile order
    /// with no sort. The earlier pass pushed one index for every tile the
    /// sources touched and then ordered them, which cost a comparison sort
    /// over a list several times longer than the answer.[^2]
    ///
    /// The bit plane covers the world, so its size follows the lattice and
    /// not the population.[^3] The pass allocates it here rather than holding
    /// it, because it carries nothing between frames and the holding would
    /// then copy it on every clone.
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    /// [^2]: Findings register, FND-285. `docs/FINDINGS.md`
    /// [^3]: ADR-0096, cost follows the lattice, not the population. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
    fn candidates(&self, arena: &SoldierArena, threads: usize) -> Vec<TileIdx> {
        let grid = self.layout.grid();
        let tile_count = grid.tile_count();
        let words = (tile_count as usize).div_ceil(64);
        let holders = &self.holders[..];

        // The held list is divided into contiguous runs, one for each thread.
        // The division is a function of the list and of the thread count, and
        // of nothing else, so no thread claims the next piece.[^1]
        //
        // Each thread fills its own bit plane, and the join below reads the
        // planes in slot order. No two threads write one word.[^1] The plane
        // is the memory that this shape costs, and the record says so.[^1]
        //
        // A thread reports the words it touched. The held list is in
        // ascending tile order, so a run of it reaches one window of the
        // plane and the pages outside that window are never written and never
        // read. The whole join therefore costs one plane and not one for each
        // thread.
        //
        // [^1]: ADR-0009, parallel stages write disjoint outputs, decisions D1, D2 and D3. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
        // [^2]: Findings register, FND-286. `docs/FINDINGS.md`
        let threads = threads.max(1);
        let chunk_len = self.held.len().div_ceil(threads).max(1);
        let slot_count = self.held.len().div_ceil(chunk_len).max(1);
        // One allocation holds every plane, and a thread takes one chunk of
        // it. A plane for each thread in its own allocation costs one mapping
        // for each thread on every frame, and giving those mappings back
        // reaches every core.[^2]
        let mut planes: Vec<u64> = vec![0; slot_count * words];
        let mut slots: Slots<(usize, usize)> = Slots::filled(slot_count, (words, 0))
            .expect("a slot count of one or more names at least one slot");
        std::thread::scope(|scope| {
            let mut handles = Vec::new();
            for ((chunk, plane), slot) in self
                .held
                .chunks(chunk_len)
                .zip(planes.chunks_mut(words))
                .zip(slots.entries_mut())
            {
                handles.push(scope.spawn(move || {
                    let mut lowest = words;
                    let mut highest = 0usize;
                    for tile in chunk {
                        let index = tile.0;
                        if index >= tile_count {
                            continue;
                        }
                        let holder = holders[index as usize];
                        let (neighbours, found) = neighbour_indices(grid, index);
                        // A tile at the edge of the world has fewer than six
                        // neighbours, so it is never inside a holding. This is
                        // what the earlier pass said by treating an absent
                        // neighbour as a mismatch.
                        let inside = found == NEIGHBOUR_COUNT
                            && neighbours[..found]
                                .iter()
                                .all(|neighbour| holders[*neighbour as usize] == holder);
                        if inside {
                            continue;
                        }
                        mark(plane, index);
                        lowest = lowest.min((index / 64) as usize);
                        highest = highest.max((index / 64) as usize + 1);
                        for neighbour in &neighbours[..found] {
                            mark(plane, *neighbour);
                            lowest = lowest.min((*neighbour / 64) as usize);
                            highest = highest.max((*neighbour / 64) as usize + 1);
                        }
                    }
                    *slot = (lowest, highest);
                }));
            }
            for handle in handles {
                // A thread here reads shared memory and writes its own chunk,
                // so it has no failure of its own. A panic inside one is a
                // defect, and it travels rather than being swallowed.
                handle.join().expect("a candidate thread cannot fail");
            }
        });

        let mut marked: Vec<u64> = vec![0; words];
        for (plane, (lowest, highest)) in planes.chunks(words).zip(slots.entries()) {
            for position in *lowest..*highest {
                marked[position] |= plane[position];
            }
        }

        // The arena iterates in slot order, which is fixed. The bit plane
        // makes the answer independent of that anyway, because a set does not
        // record the order in which it was filled.
        let column = arena.tile_column();
        for soldier in arena.iter() {
            let tile = column[soldier.index() as usize];
            if tile.0 < tile_count {
                mark(&mut marked, tile.0);
            }
        }

        // The count is read first so that the list is allocated once. It
        // costs one pass over the words, which is small against the tiles the
        // scan below emits.
        let held_count: u32 = marked.iter().map(|word| word.count_ones()).sum();
        let mut candidates: Vec<TileIdx> = Vec::with_capacity(held_count as usize);
        for (position, word) in marked.iter().enumerate() {
            let mut rest = *word;
            let base = (position as u32) * 64;
            while rest != 0 {
                let bit = rest.trailing_zeros();
                candidates.push(TileIdx(base + bit));
                rest &= rest - 1;
            }
        }
        candidates
    }

    /// Writes the decided changes and repairs the three derived parts.
    ///
    /// The write is one scattered store for each change, and it runs on the
    /// calling thread because it reads the holder it is about to overwrite.
    ///
    /// **One repair follows the change count and the other follows the list
    /// it repairs.** The mask of a block is no longer derived by reading the
    /// block: the holding counts the tiles each faction holds in each block,
    /// so a mask gains a bit when a count leaves zero and loses one when a
    /// count reaches zero, and a moved tile touches two counters.[^1] The held
    /// list is still rebuilt by a merge that reads all of it, and that merge
    /// takes a thread count.[^2]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-301. `docs/FINDINGS.md`
    /// [^2]: ADR-0009, parallel stages write disjoint outputs, decisions D1, D2 and D3. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
    fn apply(&mut self, changes: &[(TileIdx, Holder)], threads: usize) {
        let threads = threads.max(1);
        let mut moved: Vec<TileIdx> = Vec::with_capacity(changes.len());
        for (tile, holder) in changes {
            let previous = self.holders[tile.0 as usize];
            if previous == *holder {
                continue;
            }
            let block = self
                .layout
                .key_of(*tile)
                .map(|key| self.layout.block_of_key(key) as usize);
            if let Some(faction) = previous.faction() {
                self.census[faction.0 as usize] -= 1;
                if let Some(block) = block {
                    self.leave_block(block, faction);
                }
            }
            if let Some(faction) = holder.faction() {
                self.census[faction.0 as usize] += 1;
                if let Some(block) = block {
                    self.enter_block(block, faction);
                }
            }
            self.holders[tile.0 as usize] = *holder;
            moved.push(*tile);
        }
        if moved.is_empty() {
            return;
        }
        moved.sort_unstable();
        moved.dedup();
        #[cfg(feature = "census-holding")]
        let moved_count = moved.len() as u64;

        self.rebuild_held(&moved, threads);

        // **No block is read again.** A block loses a faction bit exactly when
        // that faction's count in the block reaches zero, and the counts above
        // saw that as each tile moved. Rereading a block cost every tile of
        // it, and a frame at the target scale dirties most blocks.[^1]
        //
        // [^1]: Findings register, FND-301. `docs/FINDINGS.md`
        #[cfg(feature = "census-holding")]
        census::record(moved_count, 0, self.held.len() as u64);
    }

    /// Records that one faction took one more tile of one block.
    ///
    /// The mask gains the bit when the count leaves zero, and at no other
    /// time.
    fn enter_block(&mut self, block: usize, faction: FactionId) {
        let at = block * MASK_BITS as usize + faction.0 as usize;
        let Some(count) = self.block_census.get_mut(at) else {
            return;
        };
        *count += 1;
        if *count == 1 {
            if let Some(mask) = self.block_masks.get_mut(block) {
                *mask = mask.with(faction);
            }
        }
    }

    /// Records that one faction gave up one tile of one block.
    ///
    /// The mask loses the bit when the count reaches zero, and at no other
    /// time.
    fn leave_block(&mut self, block: usize, faction: FactionId) {
        let at = block * MASK_BITS as usize + faction.0 as usize;
        let Some(count) = self.block_census.get_mut(at) else {
            return;
        };
        *count = count.saturating_sub(1);
        if *count == 0 {
            if let Some(mask) = self.block_masks.get_mut(block) {
                *mask = mask.without(faction);
            }
        }
    }

    /// Rebuilds the held list from the old list and the tiles that changed.
    ///
    /// The held list stays in ascending tile order. A change either adds a
    /// tile to it or takes one out, so the merge reads the old list once and
    /// the changed tiles once.
    ///
    /// **The tile space is cut at values taken from the old list, and both
    /// lists are cut at the same values.** A thread therefore merges one band
    /// of tiles, writes its own buffer, and no two threads produce one tile.
    /// The join reads the buffers in band order, which is ascending tile
    /// order, and never in the order a thread finished.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0009, parallel stages write disjoint outputs, decisions D1, D2 and D3. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
    fn rebuild_held(&mut self, moved: &[TileIdx], threads: usize) {
        let held = std::mem::take(&mut self.held);
        let chunk_len = held.len().div_ceil(threads).max(1);

        // The cut points are tile values and not positions, so the two lists
        // are cut at the same places and every tile falls in exactly one band.
        let mut cuts: Vec<(usize, usize)> = vec![(0, 0)];
        let mut at = chunk_len;
        while at < held.len() {
            let pivot = held[at];
            cuts.push((at, moved.partition_point(|tile| *tile < pivot)));
            at += chunk_len;
        }
        cuts.push((held.len(), moved.len()));

        let holders = &self.holders[..];
        let bands = cuts.len() - 1;
        let mut slots: Slots<Vec<TileIdx>> = Slots::filled(bands, Vec::new())
            .expect("the cut list always holds a first and a last entry");
        std::thread::scope(|scope| {
            let mut handles = Vec::new();
            for (band, slot) in slots.entries_mut().iter_mut().enumerate() {
                let (held_from, moved_from) = cuts[band];
                let (held_to, moved_to) = cuts[band + 1];
                let old = &held[held_from..held_to];
                let changed = &moved[moved_from..moved_to];
                handles.push(scope.spawn(move || {
                    let mut joined: Vec<TileIdx> = Vec::with_capacity(old.len() + changed.len());
                    let mut cursor = 0usize;
                    for tile in changed {
                        while cursor < old.len() && old[cursor] < *tile {
                            joined.push(old[cursor]);
                            cursor += 1;
                        }
                        if cursor < old.len() && old[cursor] == *tile {
                            cursor += 1;
                        }
                        if !holders[tile.0 as usize].is_nobody() {
                            joined.push(*tile);
                        }
                    }
                    joined.extend_from_slice(&old[cursor..]);
                    *slot = joined;
                }));
            }
            for handle in handles {
                // A band reads shared memory and writes its own buffer, so it
                // has no failure of its own.
                handle.join().expect("a band of the held list cannot fail");
            }
        });

        let total: usize = slots.entries().iter().map(Vec::len).sum();
        let mut rebuilt: Vec<TileIdx> = Vec::with_capacity(total);
        for band in slots.entries() {
            rebuilt.extend_from_slice(band);
        }
        self.held = rebuilt;
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
        if self.block_census.len() != self.layout.block_count() as usize * MASK_BITS as usize {
            return false;
        }

        let mut census = [0i64; MASK_BITS as usize];
        let mut held: Vec<TileIdx> = Vec::new();
        let mut masks = vec![FactionMask::EMPTY; self.block_masks.len()];
        // The per-block count is a second declaration of what the holder
        // column says, so it is derived here and compared like the rest.[^1]
        let mut counts = vec![0u32; self.block_census.len()];
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
            let at = block * MASK_BITS as usize + faction.0 as usize;
            if at >= counts.len() {
                return false;
            }
            counts[at] += 1;
        }

        census == self.census
            && held == self.held
            && masks == self.block_masks
            && counts == self.block_census
    }
}

/// Sets the bit of one tile in a tile bit plane.
#[inline]
fn mark(marked: &mut [u64], tile: u32) {
    let word = (tile / 64) as usize;
    if let Some(entry) = marked.get_mut(word) {
        *entry |= 1u64 << (tile % 64);
    }
}

/// Returns the tile indices of the neighbours of one tile, and how many the
/// world holds.
///
/// The grid is axial and the index of an address is the row times the width
/// plus the column, so each of the six directions is one fixed offset from
/// the index.[^1] The address arithmetic is therefore one division for the
/// tile and a comparison for each direction, rather than a conversion to an
/// address and back for every neighbour.
///
/// The order is the direction order that the grid gives, and the entries
/// after `found` hold nothing. A caller that needs to know which direction is
/// absent asks the grid instead.
///
/// **This is a second way to say what the grid already says.** The test
/// `neighbour_indices_agree_with_the_grid` derives both answers for every
/// tile of a small world, edges included, and compares them.[^2]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
/// [^2]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
#[inline]
fn neighbour_indices(grid: Grid, tile: u32) -> ([u32; NEIGHBOUR_COUNT], usize) {
    let width = grid.width();
    let height = grid.height();
    let column = tile % width;
    let row = tile / width;
    let mut neighbours = [0u32; NEIGHBOUR_COUNT];
    let mut found = 0usize;
    let mut take = |index: u32| {
        neighbours[found] = index;
        found += 1;
    };
    let east = column + 1 < width;
    let west = column >= 1;
    let north = row >= 1;
    let south = row + 1 < height;
    if east {
        take(tile + 1);
    }
    if east && north {
        take(tile + 1 - width);
    }
    if north {
        take(tile - width);
    }
    if west {
        take(tile - 1);
    }
    if west && south {
        take(tile - 1 + width);
    }
    if south {
        take(tile + width);
    }
    (neighbours, found)
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
    /// How far the walk has reached inside each block of the derived unit
    /// structure. One entry for each block of the world.
    cursors: Vec<u32>,
}

impl Scratch {
    /// Builds working memory in which no faction has support and no block has
    /// been walked.
    fn new(blocks: usize) -> Self {
        Self {
            tally: [0; MASK_BITS as usize],
            supporters: Vec::with_capacity(MASK_BITS as usize),
            cursors: vec![0; blocks],
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
    layout: BlockLayout,
    terrain: Terrain,
    holders: &[Holder],
    arena: &SoldierArena,
    bridge: &UnitTileBridge,
    scratch: &mut Scratch,
    tile: TileIdx,
) -> Option<Holder> {
    let grid = layout.grid();
    scratch.clear();
    let address = grid.address_of(tile)?;

    // The neighbour index of a tile is one fixed offset from the index, so
    // this reads the holder column directly rather than converting to an
    // address and back for each of the six directions.
    let (neighbours, found) = neighbour_indices(grid, tile.0);
    for index in &neighbours[..found] {
        if let Some(faction) = holders[*index as usize].faction() {
            scratch.raise(faction, 1);
        }
    }
    for unit in units_on_tile(layout, bridge, &mut scratch.cursors, tile) {
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
        if support <= incumbent {
            continue;
        }
        if best.is_none_or(|(top, _)| support > top) {
            best = Some((support, *faction));
        }
    }
    let (support, faction) = best?;

    // **The ground is read last, because it can only refuse.** It says how
    // much support the tile asks for, and open water asks for more than any
    // claim can raise. A tile whose best challenger does not beat the holder
    // keeps its holder whatever the ground says, so the read is skipped for
    // it. Reading the ground first cost a generated value for every candidate
    // tile, and most candidates have no challenger.[^1]
    //
    // The threshold is one number for the tile, so the strongest challenger
    // is the one most likely to reach it. A challenger that the threshold
    // refuses is refused for every weaker challenger too.
    //
    // [^1]: Findings register, FND-293. `docs/FINDINGS.md`
    let threshold = claim_threshold(terrain.kind(address)?)?;
    if support < threshold {
        // The ground either admits no holder at all, or asks for more support
        // than the challenger raised. A tile of open water that somebody held
        // would be a defect, and the invariant check names it.
        return None;
    }

    Some(Holder::of(FactionId(faction)))
}

/// Returns the units that stand on one tile, by walking the block rather than
/// searching it.
///
/// **The caller must ask for the tiles of a block in ascending tile order.**
/// The low part of a bridge key is the row-major offset of the tile inside its
/// block, so ascending tile order gives ascending keys inside every block.[^1]
/// The cursor of a block therefore only ever moves forward, and the whole walk
/// over a candidate list costs the list plus the units, rather than a search
/// for each candidate.
///
/// The cursor is left at the first entry of the answer and not after it, so
/// asking twice for one tile gives the same answer twice.
///
/// The caller establishes that the structure describes the arena before it
/// starts to walk, and holds the borrow of the arena for the whole walk.[^2]
///
/// # References
///
/// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D2. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
/// [^2]: Findings register, FND-295. `docs/FINDINGS.md`
fn units_on_tile<'a>(
    layout: BlockLayout,
    bridge: &'a UnitTileBridge,
    cursors: &mut [u32],
    tile: TileIdx,
) -> &'a [Entity] {
    let Some(key) = layout.key_of(tile) else {
        return &[];
    };
    let block = layout.block_of_key(key) as usize;
    let (keys, units) = bridge.block_window(block as u32);
    if keys.is_empty() {
        return &[];
    }
    let mut at = cursors.get(block).map_or(0, |cursor| *cursor as usize);
    if at > keys.len() {
        at = 0;
    }
    while at < keys.len() && keys[at] < key {
        at += 1;
    }
    let first = at;
    while at < keys.len() && keys[at] == key {
        at += 1;
    }
    if let Some(cursor) = cursors.get_mut(block) {
        *cursor = first as u32;
    }
    &units[first..at]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The candidate pass derives a neighbour index from an offset. The grid
    /// derives it from an address. Two sites state one fact, so this test
    /// derives both for every tile of a small world and compares them.[^1]
    ///
    /// The world is deliberately small and not square, so that every edge,
    /// every corner and the wrap between two rows is covered.
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[test]
    fn neighbour_indices_agree_with_the_grid() {
        let grid = Grid::new(7, 5).expect("the extent must describe a grid");
        for tile in 0..grid.tile_count() {
            let address = grid
                .address_of(TileIdx(tile))
                .expect("the index is inside the world");
            let expected: Vec<u32> = grid
                .neighbours(address)
                .into_iter()
                .flatten()
                .filter_map(|neighbour| grid.index_of(neighbour))
                .map(|index| index.0)
                .collect();
            let (found, count) = neighbour_indices(grid, tile);
            assert_eq!(
                &found[..count],
                &expected[..],
                "the two derivations disagree at tile {tile}"
            );
        }
    }

    /// The cursor walk is a second way to answer what the derived unit
    /// structure already answers by searching. This drives both over every
    /// tile of a small world, in the ascending order the walk requires, and
    /// compares them tile by tile.[^1]
    ///
    /// The world puts more than one unit on one tile, puts two units of one
    /// block on different tiles, and leaves whole blocks empty, so the walk
    /// meets a run of length two, a forward step inside a block, and a block
    /// it must skip.
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[test]
    fn the_cursor_walk_agrees_with_the_search() {
        use crate::hex::Axial;
        let grid = Grid::new(16, 16).expect("a small extent describes a grid");
        let mut arena = SoldierArena::new(grid, 64);
        for address in [
            Axial::new(1, 1),
            Axial::new(1, 1),
            Axial::new(2, 1),
            Axial::new(3, 2),
            Axial::new(15, 15),
            Axial::new(9, 4),
            Axial::new(0, 0),
        ] {
            arena
                .spawn(address, FactionId(0))
                .expect("the spawn must succeed");
        }
        let layout = BlockLayout::new(grid, 2).expect("the exponent is inside the ceiling");
        let mut bridge = UnitTileBridge::new(layout);
        bridge.rebuild(&arena).expect("the rebuild must succeed");

        let mut cursors = vec![0u32; layout.block_count() as usize];
        for index in 0..grid.tile_count() {
            let tile = TileIdx(index);
            let address = grid
                .address_of(tile)
                .expect("the index is inside the world");
            let searched = bridge
                .on_tile(&arena, address)
                .expect("the bridge describes the arena");
            let walked = units_on_tile(layout, &bridge, &mut cursors, tile);
            assert_eq!(walked, searched, "the two answers disagree at tile {index}");
            // The cursor stays on the answer, so a second ask repeats it.
            let again = units_on_tile(layout, &bridge, &mut cursors, tile);
            assert_eq!(again, searched, "the second ask differs at tile {index}");
        }
        assert!(
            (0..grid.tile_count()).any(|index| {
                units_on_tile(
                    layout,
                    &bridge,
                    &mut vec![0; layout.block_count() as usize],
                    TileIdx(index),
                )
                .len()
                    > 1
            }),
            "the fixture must hold a tile with more than one unit"
        );
    }

    /// A bit plane holds a set, and the scan reads it back in ascending
    /// order. This proves the two halves of that against a small case.
    #[test]
    fn a_marked_bit_plane_reads_back_in_ascending_order() {
        let mut marked = vec![0u64; 3];
        for tile in [130u32, 0, 63, 64, 130] {
            mark(&mut marked, tile);
        }
        mark(&mut marked, 500);
        let mut read: Vec<u32> = Vec::new();
        for (position, word) in marked.iter().enumerate() {
            let mut rest = *word;
            while rest != 0 {
                read.push((position as u32) * 64 + rest.trailing_zeros());
                rest &= rest - 1;
            }
        }
        assert_eq!(read, vec![0, 63, 64, 130]);
    }
}
