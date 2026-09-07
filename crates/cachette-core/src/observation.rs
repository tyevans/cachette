//! What each faction observes, and what each faction remembers.
//!
//! A faction sees a tile when one of its own live units observes that tile.
//! A faction that never observed a tile reads nothing about it. A faction
//! that observed a tile and marched away reads what it last saw, and the
//! engine marks that reading as remembered.[^1]
//!
//! # Two layers, and why they differ
//!
//! **What a faction sees now is derived.** The units, their tiles and their
//! sight are all in the frame, so the layer is a pure function of the frame.
//! The step rebuilds it, and it enters no state hash.[^2] [^3]
//!
//! **What a faction saw once is remembered, and the step carries it
//! forward.** A faction that saw a valley and marched away holds a fact that
//! no present unit produces, and no derivation returns it. The step adds the
//! tiles seen now into the remembered layer and never removes one, so the
//! layer only grows. It is the one part of fog that is state, so it enters
//! the state hash.[^2] [^4]
//!
//! # The container
//!
//! Neither layer is a dense bitmap over the world. A bit for each tile for
//! each faction is a field of the world indexed by the faction, and a record
//! forbids that plane.[^5] The cost of such a plane follows the world, not
//! the ground a faction has walked.
//!
//! Each layer is instead an array of blocks over the block lattice that the
//! summary level aggregates over, so a fog layer and a summary share one
//! lattice.[^6] Each block holds one of four forms. A block holds no payload
//! when the faction sees no tile in it. A block holds a sorted array of
//! offsets when the faction sees few. A block holds a bitmap when the faction
//! sees many. A block holds no payload again when the faction sees every tile
//! in it. The threshold between the array and the bitmap is the population at
//! which the two cost the same bytes, and this module derives it from the
//! block edge rather than stating a value.[^2] [^7]
//!
//! A faction allocates nothing until it observes something. The block array
//! of a faction is built on its first observation and never on its
//! creation.[^2]
//!
//! # Sight
//!
//! The sight of a unit is a whole number of hex steps.[^8] The rules round
//! that number down to a multiple of a step and clamp it to a ceiling, so
//! units that stand on one tile share one answer.[^2]
//!
//! Ground blocks sight. The pass computes the tiles a unit observes by a
//! shadowcast over the six sextants of the hex grid, so ground that blocks
//! sight hides everything behind it.[^9]
//!
//! Sight comes from a live unit of the faction and from nothing else. A
//! settlement gives no sight, held ground gives no sight, and an upgrade
//! gives no sight.[^2]
//!
//! # Determinism
//!
//! The combine over one block is a union of sets. A union is associative,
//! commutative and idempotent, so a fold gives one answer whatever the
//! grouping.[^10]
//!
//! The pass does not rest on that alone. It sorts the stamps by block, then
//! by faction, then by observer tile, and it gives each worker a contiguous
//! run of blocks. Two workers therefore never write one block, and the update
//! needs no atomic operation.[^11] The partition comes from the block count
//! and the thread count, and never from the schedule. The join reads the
//! worker slots in slot order. **Nothing reads which worker finished first.**
//!
//! The pass draws no random number anywhere.
//!
//! # References
//!
//! [^1]: PRD-0001, a faction sees only what its own units observe. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
//! [^2]: ADR-0059, fog storage grows with observed area, not with world area, decisions D1 to D3. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
//! [^3]: ADR-0164, every stored value the step reads enters the state hash, decision D2. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
//! [^4]: ADR-0164, every stored value the step reads enters the state hash, decision D1. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
//! [^5]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D3. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
//! [^6]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
//! [^7]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
//! [^8]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^9]: Research report 08, fog of war representation. `docs/research/reports/08-fog-of-war-representation.md`
//! [^10]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
//! [^11]: ADR-0009, parallel stages write disjoint outputs, decisions D1, D2 and D3. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`

use crate::bridge::{BlockLayout, BridgeError};
use crate::hash::StateHash;
use crate::hex::{Axial, Grid, NEIGHBOURS, NEIGHBOUR_COUNT};
use crate::holding::FactionMask;
use crate::slots::Slots;
use crate::soldier::SoldierArena;
use crate::terrain::{Terrain, TileKind};
use crate::types::{FactionId, TileIdx, FACTION_CEILING};

/// How far a unit sees before the rules round the number, in hex steps.
///
/// The value is provisional. A sight of four steps gives a disc of 61 tiles,
/// which is the disc the base reach of a city covers, so a unit sees about as
/// far as one city holds. No measurement supports it.[^1]
///
/// # References
///
/// [^1]: Blockers register, BLK-050. `docs/BLOCKERS.md`
pub const SIGHT_RADIUS_DEFAULT: u32 = 4;

/// The step that the rules round a sight down to.
///
/// Rounding collapses the distinct discs the pass computes. A step of two
/// halves the number of distinct radii the pass can meet. The value is
/// provisional and no measurement supports it.[^1]
///
/// # References
///
/// [^1]: Blockers register, BLK-050. `docs/BLOCKERS.md`
pub const SIGHT_STEP_DEFAULT: u32 = 2;

/// The largest sight the engine admits, in hex steps.
///
/// The bound is structural rather than balance. The shadowcast scratch grows
/// with the square of the radius, and a bound keeps one observer inside a
/// fixed working set. A radius of sixteen gives a disc of 817 tiles.
pub const SIGHT_CEILING: u32 = 16;

/// The tile kinds that block sight, as a bit for each kind.
///
/// Trees and high ground hide what stands behind them. Water and level
/// ground do not.
pub const SIGHT_BLOCKERS_DEFAULT: u8 =
    (1 << TileKind::Forest as u8) | (1 << TileKind::Mountain as u8);

/// How far a unit sees, and what stops it seeing.
///
/// Every value here is a whole number of hex steps or a bit set over the tile
/// kinds. No value is a floating point number.[^1]
///
/// **The step reads these rules on every tick**, so they enter the state
/// hash. Two worlds that hold the same units and different rules must
/// diverge on the next rebuild, and a hash of the layers alone would report
/// the difference one tick late.[^2]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
/// [^2]: ADR-0164, every stored value the step reads enters the state hash, decisions D1 and D3. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SightRules {
    radius: u32,
    step: u32,
    ceiling: u32,
    blockers: u8,
}

impl Default for SightRules {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl SightRules {
    /// The provisional rules that a world starts with.
    pub const DEFAULT: Self = Self {
        radius: SIGHT_RADIUS_DEFAULT,
        step: SIGHT_STEP_DEFAULT,
        ceiling: SIGHT_CEILING,
        blockers: SIGHT_BLOCKERS_DEFAULT,
    };

    /// Builds a set of rules.
    ///
    /// The ceiling is clamped to the largest sight the engine admits, so no
    /// caller can widen the working set of one observer past the bound.
    #[must_use]
    pub const fn new(radius: u32, step: u32, ceiling: u32, blockers: u8) -> Self {
        let ceiling = if ceiling > SIGHT_CEILING {
            SIGHT_CEILING
        } else {
            ceiling
        };
        Self {
            radius,
            step,
            ceiling,
            blockers,
        }
    }

    /// Returns the sight before rounding, in hex steps.
    #[must_use]
    pub const fn radius(self) -> u32 {
        self.radius
    }

    /// Returns the step that the rules round a sight down to.
    #[must_use]
    pub const fn step(self) -> u32 {
        self.step
    }

    /// Returns the largest sight the rules admit, in hex steps.
    #[must_use]
    pub const fn ceiling(self) -> u32 {
        self.ceiling
    }

    /// Returns the tile kinds that block sight, as a bit for each kind.
    #[must_use]
    pub const fn blockers(self) -> u8 {
        self.blockers
    }

    /// Returns the sight the pass uses, in hex steps.
    ///
    /// The sight is clamped to the ceiling and then rounded down to a
    /// multiple of the step. A step of zero rounds to nothing, and the
    /// clamped radius stands.
    #[must_use]
    pub const fn rounded(self) -> u32 {
        let clamped = if self.radius > self.ceiling {
            self.ceiling
        } else {
            self.radius
        };
        match clamped.checked_div(self.step) {
            Some(steps) => steps * self.step,
            // A step of zero rounds to nothing, and the clamped radius
            // stands.
            None => clamped,
        }
    }

    /// Reports whether ground of this kind hides what stands behind it.
    #[must_use]
    pub const fn blocks(self, kind: TileKind) -> bool {
        self.blockers & (1 << kind.to_u8()) != 0
    }

    /// Absorbs the rules into the state hash.
    #[must_use]
    pub fn hash_into(self, hash: StateHash) -> StateHash {
        hash.write_u64(u64::from(self.radius))
            .write_u64(u64::from(self.step))
            .write_u64(u64::from(self.ceiling))
            .write_u64(u64::from(self.blockers))
    }
}

/// One block of one layer.
///
/// The four forms are the four the record names. Two of them hold no payload
/// at all: a block the faction sees nothing in, and a block the faction sees
/// whole.[^1]
///
/// # References
///
/// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D2. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum BlockForm {
    /// The faction sees no tile of this block. No payload.
    #[default]
    None,
    /// The faction sees few tiles. The offsets ascend and never repeat.
    Few(Vec<u32>),
    /// The faction sees many tiles. One bit for each tile of the block.
    Many(Vec<u64>),
    /// The faction sees every tile of this block. No payload.
    All,
}

impl BlockForm {
    /// Reports whether the block names the tile at one offset inside it.
    ///
    /// This is the one place that reads a payload, so a reader that walks a
    /// block never repeats the bit layout.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub fn holds(&self, offset: u32) -> bool {
        match self {
            Self::None => false,
            Self::All => true,
            Self::Few(offsets) => offsets.binary_search(&offset).is_ok(),
            Self::Many(words) => {
                let word = (offset / 64) as usize;
                words
                    .get(word)
                    .is_some_and(|bits| bits & (1u64 << (offset % 64)) != 0)
            }
        }
    }

    /// Returns the form as a small integer, for the state hash.
    const fn tag(&self) -> u64 {
        match self {
            Self::None => 0,
            Self::Few(_) => 1,
            Self::Many(_) => 2,
            Self::All => 3,
        }
    }
}

/// The tiles of one world that one faction sees, as an array of blocks.
///
/// The block order is the block number, and the offsets inside a block
/// ascend.[^1]
///
/// # References
///
/// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D2. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
#[derive(Clone, Debug)]
pub struct TileLayer {
    layout: BlockLayout,
    blocks: Vec<BlockForm>,
    /// How many tiles of each block the faction sees.
    populations: Vec<u32>,
    /// Which blocks hold a population above zero, in ascending order.
    populated: Vec<u32>,
    /// How many tiles the faction sees in the whole world.
    ///
    /// The total is widened, so no count over the layer depends on the
    /// margin of a narrow type.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D3. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    total: i64,
}

impl TileLayer {
    /// Builds a layer that names no tile.
    ///
    /// The block array is allocated here. A caller builds a layer on the
    /// first observation of a faction, never on its creation.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D2. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    #[must_use]
    pub fn new(layout: BlockLayout) -> Self {
        let count = layout.block_count() as usize;
        Self {
            layout,
            blocks: vec![BlockForm::None; count],
            populations: vec![0; count],
            populated: Vec::new(),
            total: 0,
        }
    }

    /// Returns the lattice that the layer divides the world over.
    #[must_use]
    pub const fn layout(&self) -> BlockLayout {
        self.layout
    }

    /// Returns how many tiles the faction sees in the whole world.
    #[must_use]
    pub const fn population(&self) -> i64 {
        self.total
    }

    /// Returns how many tiles the faction sees in one block.
    #[must_use]
    pub fn block_population(&self, block: u32) -> u32 {
        self.populations.get(block as usize).copied().unwrap_or(0)
    }

    /// Returns the form of one block.
    #[must_use]
    pub fn block(&self, block: u32) -> Option<&BlockForm> {
        self.blocks.get(block as usize)
    }

    /// Returns every block the faction sees a tile in, in ascending order.
    #[must_use]
    pub fn populated_blocks(&self) -> &[u32] {
        &self.populated
    }

    /// Reports whether the layer names one tile.
    ///
    /// The read is one block lookup and one search inside that block.
    #[must_use]
    pub fn contains(&self, tile: TileIdx) -> bool {
        let Some(key) = self.layout.key_of(tile) else {
            return false;
        };
        let block = self.layout.block_of_key(key) as usize;
        let offset = offset_of_key(self.layout, key);
        self.blocks
            .get(block)
            .is_some_and(|form| form.holds(offset))
    }

    /// Replaces every block of the layer with the blocks a rebuild produced.
    ///
    /// The rebuild names only the blocks that hold a tile, so the reset
    /// visits the blocks the previous frame populated and no others.
    fn replace(&mut self, produced: &mut Vec<(u32, u32, BlockForm)>) {
        for block in self.populated.drain(..) {
            let index = block as usize;
            if let Some(slot) = self.blocks.get_mut(index) {
                *slot = BlockForm::None;
            }
            if let Some(slot) = self.populations.get_mut(index) {
                *slot = 0;
            }
        }
        self.total = 0;
        // The produced list ascends by block, because the workers take
        // contiguous runs of blocks and the join reads the workers in slot
        // order.
        for (block, population, form) in produced.drain(..) {
            let index = block as usize;
            if population == 0 {
                continue;
            }
            let Some(slot) = self.blocks.get_mut(index) else {
                continue;
            };
            *slot = form;
            if let Some(count) = self.populations.get_mut(index) {
                *count = population;
            }
            self.populated.push(block);
            self.total += i64::from(population);
        }
    }

    /// Adds every tile of one block of another layer into this layer.
    ///
    /// The union never removes a tile, which is what makes the remembered
    /// layer monotonic.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D3. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    fn absorb(&mut self, block: u32, other: &BlockForm, tiles_in_block: u32) {
        // The scratch spans the whole block, because the offset of a tile
        // packs its row and its column against the block edge and not
        // against the tiles the world cuts the block down to.
        let index = block as usize;
        let Some(mine) = self.blocks.get(index) else {
            return;
        };
        if matches!(mine, BlockForm::All) || matches!(other, BlockForm::None) {
            return;
        }
        let words = offset_span(self.layout).div_ceil(64) as usize;
        let mut scratch = vec![0u64; words];
        write_form_into(mine, &mut scratch, self.layout, block);
        write_form_into(other, &mut scratch, self.layout, block);
        let population = scratch.iter().map(|word| word.count_ones()).sum::<u32>();
        let was = self.populations.get(index).copied().unwrap_or(0);
        if population == was {
            return;
        }
        if was == 0 {
            self.populated.push(block);
            self.populated.sort_unstable();
        }
        let form = choose_form(&scratch, population, tiles_in_block);
        if let Some(slot) = self.blocks.get_mut(index) {
            *slot = form;
        }
        if let Some(slot) = self.populations.get_mut(index) {
            *slot = population;
        }
        self.total += i64::from(population) - i64::from(was);
    }

    /// Reports whether the stored populations agree with the payloads.
    ///
    /// The population of a block is held beside the payload, so that a
    /// caller reads a cardinality without walking the payload. That is one
    /// fact in two places, and this check is what fails when the two
    /// disagree.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub fn populations_agree(&self) -> bool {
        let mut total: i64 = 0;
        for (block, form) in self.blocks.iter().enumerate() {
            let held = self.populations.get(block).copied().unwrap_or(0);
            let derived = match form {
                BlockForm::None => 0,
                BlockForm::All => tiles_in_block(self.layout, block as u32),
                BlockForm::Few(offsets) => offsets.len() as u32,
                BlockForm::Many(words) => words.iter().map(|word| word.count_ones()).sum(),
            };
            if held != derived {
                return false;
            }
            if (held > 0) != self.populated.binary_search(&(block as u32)).is_ok() {
                return false;
            }
            total += i64::from(held);
        }
        total == self.total
    }

    /// Absorbs the layer into the state hash.
    ///
    /// The walk takes the blocks in block number order and hashes the form
    /// tag, the population and the payload of each. A layer outside the hash
    /// would let two worlds that differ hash the same.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D5. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    #[must_use]
    pub fn hash_into(&self, mut hash: StateHash) -> StateHash {
        hash = hash.write_u64(self.total as u64);
        for block in &self.populated {
            let index = *block as usize;
            hash = hash.write_u64(u64::from(*block));
            hash = hash.write_u64(u64::from(self.populations.get(index).copied().unwrap_or(0)));
            let Some(form) = self.blocks.get(index) else {
                continue;
            };
            hash = hash.write_u64(form.tag());
            match form {
                BlockForm::None | BlockForm::All => {}
                BlockForm::Few(offsets) => {
                    for offset in offsets {
                        hash = hash.write(&offset.to_le_bytes());
                    }
                }
                BlockForm::Many(words) => {
                    for word in words {
                        hash = hash.write_u64(*word);
                    }
                }
            }
        }
        hash
    }
}

/// Returns the offset of a tile inside its block.
fn offset_of_key(layout: BlockLayout, key: u64) -> u32 {
    layout.offset_of_key(key)
}

/// Returns how many offsets one block of the lattice spans.
///
/// The offset of a tile inside a block is the row and the column of that tile
/// inside the block, packed into one number. The span is therefore the square
/// of the block edge, whatever the world cuts off at its own edge. A block at
/// the edge of the world holds fewer tiles than the span, and the offsets of
/// the tiles it does hold still reach the top of the span.
const fn offset_span(layout: BlockLayout) -> u32 {
    let edge = layout.block_edge();
    edge * edge
}

/// Sets one offset of a scratch bitmap.
fn set_offset(scratch: &mut [u64], offset: u32) {
    let word = (offset / 64) as usize;
    if let Some(slot) = scratch.get_mut(word) {
        *slot |= 1u64 << (offset % 64);
    }
}

/// Writes every tile of a form into a scratch bitmap.
fn write_form_into(form: &BlockForm, scratch: &mut [u64], layout: BlockLayout, block: u32) {
    match form {
        BlockForm::None => {}
        BlockForm::All => {
            // A block at the edge of the world holds a rectangle of tiles
            // that is narrower or shorter than the block edge, so the walk
            // takes the rows and the columns and never the whole span.
            let (wide, high) = block_extent(layout, block);
            for row in 0..high {
                for column in 0..wide {
                    set_offset(scratch, (row << layout.block_bits()) | column);
                }
            }
        }
        BlockForm::Few(offsets) => {
            for offset in offsets {
                set_offset(scratch, *offset);
            }
        }
        BlockForm::Many(words) => {
            for (slot, word) in scratch.iter_mut().zip(words.iter()) {
                *slot |= *word;
            }
        }
    }
}

/// Chooses the cheapest of the four forms for one block.
///
/// The threshold between the array and the bitmap is the population at which
/// the two cost the same bytes. An offset is four bytes and a tile is one
/// bit, so the bitmap wins above one thirty-second of the block. The
/// threshold is derived from the block edge here and stated nowhere as a
/// value.[^1]
///
/// # References
///
/// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D2. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
fn choose_form(scratch: &[u64], population: u32, tiles_in_block: u32) -> BlockForm {
    if population == 0 {
        return BlockForm::None;
    }
    if population == tiles_in_block {
        return BlockForm::All;
    }
    if population <= array_threshold(tiles_in_block) {
        let mut offsets = Vec::with_capacity(population as usize);
        for (index, word) in scratch.iter().enumerate() {
            let mut bits = *word;
            while bits != 0 {
                let low = bits.trailing_zeros();
                offsets.push((index as u32) * 64 + low);
                bits &= bits - 1;
            }
        }
        // The scan runs over ascending words and takes the low bit first, so
        // the offsets ascend without a sort.
        BlockForm::Few(offsets)
    } else {
        BlockForm::Many(scratch.to_vec())
    }
}

/// Returns the population at which a sorted array and a bitmap cost the same
/// bytes.
///
/// An offset is four bytes. A bitmap over the block is one bit for each tile.
/// The two are equal at the tile count divided by thirty-two.
#[must_use]
pub const fn array_threshold(tiles_in_block: u32) -> u32 {
    tiles_in_block / 32
}

/// The scratch that one shadowcast needs.
///
/// The array follows the square of the radius and never the world, so one
/// observer costs a fixed working set.
#[derive(Clone, Debug, Default)]
struct SightScratch {
    seen: Vec<u8>,
}

impl SightScratch {
    /// Returns the index of one cell of one sextant.
    const fn index(ring: u32, step: u32) -> usize {
        (ring * (ring + 1) / 2 + step) as usize
    }

    /// Clears the scratch and sizes it for one radius.
    fn reset(&mut self, radius: u32) {
        let cells = Self::index(radius + 1, 0);
        self.seen.clear();
        self.seen.resize(cells, 0);
    }
}

/// Marks every tile that one observer sees.
///
/// The pass is a shadowcast over the six sextants of the hex grid. A sextant
/// at ring `k` holds `k + 1` cells, and each cell has at most two parents in
/// the ring below it. A cell is seen when a parent is seen and that parent
/// does not block sight, so a blocker casts a shadow that widens with the
/// distance.[^1]
///
/// The tile the observer stands on never blocks its own sight, and it is
/// always seen. Ground outside the world blocks sight and is never marked.
///
/// # References
///
/// [^1]: Research report 08, fog of war representation. `docs/research/reports/08-fog-of-war-representation.md`
fn cast_sight(
    grid: Grid,
    terrain: Terrain,
    rules: SightRules,
    origin: Axial,
    scratch: &mut SightScratch,
    mark: &mut impl FnMut(TileIdx),
) {
    if let Some(index) = grid.index_of(origin) {
        mark(index);
    }
    let radius = rules.rounded();
    if radius == 0 {
        return;
    }
    let blocks_at = |address: Axial| -> bool {
        match grid.index_of(address).and_then(|_| terrain.kind(address)) {
            Some(kind) => rules.blocks(kind),
            // Ground outside the world hides everything behind it.
            None => true,
        }
    };
    for sextant in 0..NEIGHBOUR_COUNT {
        let first = NEIGHBOURS[sextant];
        let second = NEIGHBOURS[(sextant + 1) % NEIGHBOUR_COUNT];
        let cell = |ring: u32, step: u32| -> Axial {
            let along = (ring - step) as i32;
            let across = step as i32;
            Axial::new(
                origin.q + first.q * along + second.q * across,
                origin.r + first.r * along + second.r * across,
            )
        };
        scratch.reset(radius);
        for ring in 1..=radius {
            for step in 0..=ring {
                // The parent that lies one step back along the first
                // direction, and the parent one step back along the second.
                let mut seen = false;
                if step < ring {
                    seen |= parent_admits(scratch, &blocks_at, &cell, ring, step);
                }
                if step > 0 {
                    seen |= parent_admits(scratch, &blocks_at, &cell, ring, step - 1);
                }
                if !seen {
                    continue;
                }
                scratch.seen[SightScratch::index(ring, step)] = 1;
                let address = cell(ring, step);
                if let Some(index) = grid.index_of(address) {
                    mark(index);
                }
            }
        }
    }
}

/// Reports whether one parent cell passes sight on to the ring above it.
///
/// The tile the observer stands on is the parent of every cell of ring one,
/// and it never blocks its own sight.
fn parent_admits(
    scratch: &SightScratch,
    blocks_at: &impl Fn(Axial) -> bool,
    cell: &impl Fn(u32, u32) -> Axial,
    ring: u32,
    step: u32,
) -> bool {
    if ring == 1 {
        return true;
    }
    if scratch.seen[SightScratch::index(ring - 1, step)] == 0 {
        return false;
    }
    !blocks_at(cell(ring - 1, step))
}

/// What every faction sees now, and what every faction has ever seen.
///
/// The visible layer is derived and the step rebuilds it. The remembered
/// layer is state and the step carries it forward.[^1]
///
/// # References
///
/// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D3. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
#[derive(Clone, Debug)]
pub struct Observation {
    layout: BlockLayout,
    grid: Grid,
    rules: SightRules,
    /// The tiles each faction sees now. A faction that has never observed
    /// anything holds no layer at all.
    visible: Vec<Option<TileLayer>>,
    /// The tiles each faction has ever seen.
    remembered: Vec<Option<TileLayer>>,
    /// Which factions see each block now.
    seen_masks: Vec<FactionMask>,
    /// Which factions have ever seen each block.
    ever_masks: Vec<FactionMask>,
}

impl Observation {
    /// Builds an observation over a world in which nobody has seen anything.
    ///
    /// No layer is allocated here. A faction allocates on its first
    /// observation.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D2. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    #[must_use]
    pub fn new(layout: BlockLayout) -> Self {
        let blocks = layout.block_count() as usize;
        let factions = FACTION_CEILING as usize;
        Self {
            layout,
            grid: layout.grid(),
            rules: SightRules::DEFAULT,
            visible: vec![None; factions],
            remembered: vec![None; factions],
            seen_masks: vec![FactionMask::EMPTY; blocks],
            ever_masks: vec![FactionMask::EMPTY; blocks],
        }
    }

    /// Returns the lattice that the layers divide the world over.
    #[must_use]
    pub const fn layout(&self) -> BlockLayout {
        self.layout
    }

    /// Returns how far a unit sees, and what stops it seeing.
    #[must_use]
    pub const fn rules(&self) -> SightRules {
        self.rules
    }

    /// Sets how far a unit sees, and what stops it seeing.
    pub const fn set_rules(&mut self, rules: SightRules) {
        self.rules = rules;
    }

    /// Reports whether a faction sees a tile now.
    ///
    /// A faction that holds no layer sees nothing.
    #[must_use]
    pub fn sees_now(&self, faction: FactionId, tile: TileIdx) -> bool {
        self.visible
            .get(faction.0 as usize)
            .and_then(Option::as_ref)
            .is_some_and(|layer| layer.contains(tile))
    }

    /// Reports whether a faction has ever seen a tile.
    #[must_use]
    pub fn has_seen(&self, faction: FactionId, tile: TileIdx) -> bool {
        self.remembered
            .get(faction.0 as usize)
            .and_then(Option::as_ref)
            .is_some_and(|layer| layer.contains(tile))
    }

    /// Returns how many tiles a faction sees now.
    #[must_use]
    pub fn seen_now(&self, faction: FactionId) -> i64 {
        self.visible
            .get(faction.0 as usize)
            .and_then(Option::as_ref)
            .map_or(0, TileLayer::population)
    }

    /// Returns how many tiles a faction has ever seen.
    #[must_use]
    pub fn seen_ever(&self, faction: FactionId) -> i64 {
        self.remembered
            .get(faction.0 as usize)
            .and_then(Option::as_ref)
            .map_or(0, TileLayer::population)
    }

    /// Returns the layer of the tiles a faction sees now.
    #[must_use]
    pub fn visible_layer(&self, faction: FactionId) -> Option<&TileLayer> {
        self.visible
            .get(faction.0 as usize)
            .and_then(Option::as_ref)
    }

    /// Returns the layer of the tiles a faction has ever seen.
    #[must_use]
    pub fn remembered_layer(&self, faction: FactionId) -> Option<&TileLayer> {
        self.remembered
            .get(faction.0 as usize)
            .and_then(Option::as_ref)
    }

    /// Returns which factions see one block now.
    ///
    /// The mask is derived from the visible layers, and only the rebuild
    /// writes it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D3. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    #[must_use]
    pub fn block_seen_now(&self, block: u32) -> FactionMask {
        self.seen_masks
            .get(block as usize)
            .copied()
            .unwrap_or(FactionMask::EMPTY)
    }

    /// Returns which factions have ever seen one block.
    #[must_use]
    pub fn block_seen_ever(&self, block: u32) -> FactionMask {
        self.ever_masks
            .get(block as usize)
            .copied()
            .unwrap_or(FactionMask::EMPTY)
    }

    /// Returns how many tiles of one block lie inside the world.
    ///
    /// A block at the edge of the world is cut by the edge, so a faction that
    /// sees every tile of it sees fewer than a whole block.
    #[must_use]
    pub fn tiles_in_block(&self, block: u32) -> u32 {
        tiles_in_block(self.layout, block)
    }

    /// Rebuilds what every faction sees, and folds the result into what every
    /// faction remembers.
    ///
    /// The engine calls this at the end of a step, after the last change to a
    /// unit position and after the last change to a unit faction. A pass
    /// placed earlier would answer for a world the step had already left.
    ///
    /// # Errors
    ///
    /// Returns [`BridgeError::GridMismatch`] when the arena and the layout
    /// describe different worlds.
    pub fn rebuild(
        &mut self,
        arena: &SoldierArena,
        terrain: Terrain,
        threads: usize,
    ) -> Result<(), BridgeError> {
        if arena.grid() != self.grid {
            return Err(BridgeError::GridMismatch);
        }
        let stamps = self.collect_stamps(arena);
        let produced = self.rebuild_blocks(&stamps, terrain, threads);
        self.apply(produced);
        Ok(())
    }

    /// Returns one stamp for each block that each observer of each faction
    /// may reach, ordered by block, then by faction, then by observer tile.
    ///
    /// The order is total, because the triple is unique after the sort
    /// removes the repeats. Nothing here reads a thread.
    fn collect_stamps(&self, arena: &SoldierArena) -> Vec<(u32, u16, u32)> {
        let radius = self.rules.rounded();
        let live = arena.live_column();
        let tiles = arena.tile_column();
        let factions = arena.faction_column();
        let mut observers: Vec<(u16, u32)> = Vec::new();
        for (slot, mark) in live.iter().enumerate() {
            if *mark == 0 {
                continue;
            }
            let (Some(faction), Some(tile)) = (factions.get(slot), tiles.get(slot)) else {
                continue;
            };
            if faction.0 >= FACTION_CEILING {
                continue;
            }
            observers.push((faction.0, tile.0));
        }
        observers.sort_unstable();
        observers.dedup();

        let mut stamps: Vec<(u32, u16, u32)> = Vec::new();
        for (faction, tile) in observers {
            let Some(address) = self.grid.address_of(TileIdx(tile)) else {
                continue;
            };
            let edge = self.layout.block_edge();
            let low_q = address.q.saturating_sub(radius as i32).max(0) as u32;
            let low_r = address.r.saturating_sub(radius as i32).max(0) as u32;
            let high_q = (address.q.saturating_add(radius as i32).max(0) as u32)
                .min(self.grid.width().saturating_sub(1));
            let high_r = (address.r.saturating_add(radius as i32).max(0) as u32)
                .min(self.grid.height().saturating_sub(1));
            for block_row in (low_r / edge)..=(high_r / edge) {
                for block_column in (low_q / edge)..=(high_q / edge) {
                    let block = block_row * self.layout.blocks_wide() + block_column;
                    if block < self.layout.block_count() {
                        stamps.push((block, faction, tile));
                    }
                }
            }
        }
        stamps.sort_unstable();
        stamps.dedup();
        stamps
    }

    /// Rebuilds each block that a stamp names, from the observers that reach
    /// it now.
    ///
    /// Each worker takes a contiguous run of blocks, so two workers never
    /// write one block. Each worker writes its own slot, and the join reads
    /// the slots in slot order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0009, parallel stages write disjoint outputs, decisions D1, D2 and D3. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
    fn rebuild_blocks(
        &self,
        stamps: &[(u32, u16, u32)],
        terrain: Terrain,
        threads: usize,
    ) -> Vec<(u32, u16, u32, BlockForm)> {
        if stamps.is_empty() {
            return Vec::new();
        }
        let bounds = block_bounds(stamps, threads.max(1));
        let Ok(mut slots) = Slots::filled(bounds.len(), Vec::new()) else {
            return Vec::new();
        };
        let layout = self.layout;
        let grid = self.grid;
        let rules = self.rules;
        std::thread::scope(|scope| {
            let mut handles = Vec::new();
            for (range, slot) in bounds.iter().zip(slots.entries_mut()) {
                let run = &stamps[range.0..range.1];
                handles.push(scope.spawn(move || {
                    *slot = rebuild_run(run, layout, grid, terrain, rules);
                }));
            }
            for handle in handles {
                handle
                    .join()
                    .expect("an observation worker reads shared columns and writes its own slot");
            }
        });
        // The join reads the slots in slot order, and each slot holds one
        // contiguous ascending run of blocks, so the joined list ascends.
        slots.combine(
            Vec::new(),
            |mut joined, slot: &Vec<(u32, u16, u32, BlockForm)>| {
                joined.extend(slot.iter().cloned());
                joined
            },
        )
    }

    /// Writes the rebuilt blocks into the visible layers, folds them into the
    /// remembered layers, and derives the two block masks.
    fn apply(&mut self, produced: Vec<(u32, u16, u32, BlockForm)>) {
        let layout = self.layout;
        let factions = FACTION_CEILING as usize;
        let mut by_faction: Vec<Vec<(u32, u32, BlockForm)>> = vec![Vec::new(); factions];
        for (block, faction, population, form) in produced {
            if let Some(rows) = by_faction.get_mut(faction as usize) {
                rows.push((block, population, form));
            }
        }
        self.seen_masks.fill(FactionMask::EMPTY);
        for (index, mut rows) in by_faction.into_iter().enumerate() {
            let identity = FactionId(index as u16);
            let Some(visible_slot) = self.visible.get_mut(index) else {
                continue;
            };
            if rows.is_empty() && visible_slot.is_none() {
                continue;
            }
            let visible = visible_slot.get_or_insert_with(|| TileLayer::new(layout));
            visible.replace(&mut rows);
            // The forms are cloned out of the visible layer here, because the
            // remembered layer of the same faction is a second borrow.
            let seen: Vec<(u32, BlockForm)> = visible
                .populated_blocks()
                .iter()
                .filter_map(|block| visible.block(*block).map(|form| (*block, form.clone())))
                .collect();
            if seen.is_empty() {
                continue;
            }
            let Some(remembered_slot) = self.remembered.get_mut(index) else {
                continue;
            };
            let remembered = remembered_slot.get_or_insert_with(|| TileLayer::new(layout));
            for (block, form) in &seen {
                remembered.absorb(*block, form, tiles_in_block(layout, *block));
            }
            for (block, _) in &seen {
                if let Some(mask) = self.seen_masks.get_mut(*block as usize) {
                    *mask = mask.with(identity);
                }
                if let Some(mask) = self.ever_masks.get_mut(*block as usize) {
                    *mask = mask.with(identity);
                }
            }
        }
    }

    /// Absorbs the stored part of the observation into the state hash.
    ///
    /// The remembered layer is state that the next frame reads, so it
    /// enters. The visible layer is derived from the units and the sight
    /// rules, so it stays out and its inputs enter instead.[^1] [^2]
    ///
    /// The walk takes the factions in faction order and the blocks in block
    /// number order.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0164, every stored value the step reads enters the state hash, decisions D1 and D2. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    /// [^2]: ADR-0164, every stored value the step reads enters the state hash, decision D3. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    /// [^3]: ADR-0059, fog storage grows with observed area, not with world area, decision D5. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        let mut hash = self.rules.hash_into(hash);
        for (index, layer) in self.remembered.iter().enumerate() {
            let Some(layer) = layer else {
                continue;
            };
            hash = hash.write_u64(index as u64);
            hash = layer.hash_into(hash);
        }
        hash
    }
}

/// Returns how many columns and how many rows of one block lie inside the
/// world.
fn block_extent(layout: BlockLayout, block: u32) -> (u32, u32) {
    let edge = layout.block_edge();
    let grid = layout.grid();
    let column = (block % layout.blocks_wide()) * edge;
    let row = (block / layout.blocks_wide()) * edge;
    (
        edge.min(grid.width().saturating_sub(column)),
        edge.min(grid.height().saturating_sub(row)),
    )
}

/// Returns how many tiles of one block lie inside the world.
fn tiles_in_block(layout: BlockLayout, block: u32) -> u32 {
    let (wide, high) = block_extent(layout, block);
    wide * high
}

/// Splits the sorted stamps into contiguous runs that never cut a block.
///
/// The partition comes from the block count and the thread count, and never
/// from the schedule.[^1]
///
/// # References
///
/// [^1]: ADR-0009, parallel stages write disjoint outputs, decision D3. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
fn block_bounds(stamps: &[(u32, u16, u32)], threads: usize) -> Vec<(usize, usize)> {
    let mut starts: Vec<usize> = Vec::new();
    for (index, stamp) in stamps.iter().enumerate() {
        if index == 0 || stamps[index - 1].0 != stamp.0 {
            starts.push(index);
        }
    }
    let blocks = starts.len();
    let per_worker = blocks.div_ceil(threads).max(1);
    let mut bounds = Vec::new();
    let mut first = 0;
    while first < blocks {
        let last = (first + per_worker).min(blocks);
        let start = starts[first];
        let end = if last < blocks {
            starts[last]
        } else {
            stamps.len()
        };
        bounds.push((start, end));
        first = last;
    }
    bounds
}

/// Rebuilds every block of one contiguous run of stamps.
fn rebuild_run(
    run: &[(u32, u16, u32)],
    layout: BlockLayout,
    grid: Grid,
    terrain: Terrain,
    rules: SightRules,
) -> Vec<(u32, u16, u32, BlockForm)> {
    let mut produced = Vec::new();
    let mut scratch = SightScratch::default();
    let mut first = 0;
    while first < run.len() {
        let mut last = first;
        while last < run.len() && run[last].0 == run[first].0 && run[last].1 == run[first].1 {
            last += 1;
        }
        let block = run[first].0;
        let faction = run[first].1;
        let tiles = tiles_in_block(layout, block);
        // The scratch spans the whole block, not the tiles the world leaves
        // in it. The offset of a tile packs its row and its column against
        // the block edge, so a block that the world edge cuts still carries
        // offsets to the top of the span.
        let mut bits = vec![0u64; offset_span(layout).div_ceil(64) as usize];
        for (_, _, tile) in &run[first..last] {
            let Some(origin) = grid.address_of(TileIdx(*tile)) else {
                continue;
            };
            cast_sight(grid, terrain, rules, origin, &mut scratch, &mut |seen| {
                let Some(key) = layout.key_of(seen) else {
                    return;
                };
                if layout.block_of_key(key) != block {
                    return;
                }
                set_offset(&mut bits, offset_of_key(layout, key));
            });
        }
        let population = bits.iter().map(|word| word.count_ones()).sum::<u32>();
        if population > 0 {
            produced.push((
                block,
                faction,
                population,
                choose_form(&bits, population, tiles),
            ));
        }
        first = last;
    }
    produced
}
