//! Level 1 of the level of detail pyramid.
//!
//! Level 0 holds the tiles and the units, and it is the only source of
//! truth.[^1] A level 1 cell summarises one block of tiles, and it equals the
//! exact combination of the tiles it covers.[^2] Nothing here holds a fact of
//! its own: every value in this module is a pure function of level 0, and the
//! whole level can be thrown away and rebuilt.
//!
//! **Every stored field is extensive.** An extensive quantity scales with the
//! ground it covers, so combining two of them adds them.[^3] An intensive
//! quantity does not, and this module stores none: every intensive reading is
//! a division of two extensive fields, done when a caller asks for it.[^4]
//! That is what makes the weighting automatic. A cell that covers four
//! hundred tiles contributes four hundred to the denominator, so there is no
//! separate weight for a caller to get wrong.
//!
//! **Every accumulator is 64 bits wide.** A `u8` tile field summed over the
//! tile count of the target world overflows a `u32`, so a level 1 accumulator
//! is wider than the field it sums.[^5] [^6]
//!
//! **The combine operation is field-wise integer addition.** It is exactly
//! associative and commutative, so a fold over a set of cells gives one
//! answer whatever the grouping and whatever the order.[^7] [^8] It also has
//! an inverse, so a later cost decision may repair a cell by removing the old
//! contribution rather than by rereading its children.[^9]
//!
//! **The ground contribution of a cell is computed once.** The ground is a
//! pure function of the seed and the address, and it does not change for the
//! life of a world.[^11] [^12] Reading it costs arithmetic, and a sweep of the
//! whole world every frame is what the record calls a design mistake.[^13] The
//! tile count, the open ground and the height total are therefore computed
//! when the level is built and combined into every rebuild after that.
//!
//! **The geometry is the block layout, and this module declares none.** The
//! derived unit structure partitions the world by the same block that the
//! pyramid aggregates over, so neither subsystem chooses the block edge
//! alone.[^10]
//!
//! # References
//!
//! [^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1, a draft record. `docs/adrs/draft/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
//! [^2]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2, a draft record. `docs/adrs/draft/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
//! [^3]: ADR-0024, every summary field is declared extensive or intensive, decision D2, a draft record. `docs/adrs/draft/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
//! [^4]: ADR-0024, every summary field is declared extensive or intensive, decision D3, a draft record. `docs/adrs/draft/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
//! [^5]: ADR-0023, an aggregate combines exactly, in any order, decision D3, a draft record. `docs/adrs/draft/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
//! [^6]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
//! [^7]: ADR-0023, an aggregate combines exactly, in any order, decision D1, a draft record. `docs/adrs/draft/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
//! [^8]: ADR-0023, an aggregate combines exactly, in any order, decision D2, a draft record. `docs/adrs/draft/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
//! [^9]: ADR-0023, an aggregate combines exactly, in any order, decision D4, a draft record. `docs/adrs/draft/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
//! [^10]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D2. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
//! [^11]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1, a draft record. `docs/adrs/draft/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
//! [^12]: PRD-0003, a developer sees a world worth looking at. `docs/product/shaped/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
//! [^13]: ADR-0068, terrain is generated from the seed and is never stored as a map, the consequences, a draft record. `docs/adrs/draft/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`

use crate::bridge::{BlockLayout, BridgeError, UnitTileBridge};
use crate::hash::StateHash;
use crate::hex::Axial;
use crate::sim_math;
use crate::soldier::SoldierArena;
use crate::terrain::Terrain;
use crate::types::{Accum, Fix32};

/// The summary of one block of tiles.
///
/// Every field is extensive, so the combination of two summaries is the
/// field-wise sum.[^1] A reading that does not scale with the ground is a
/// division of two of these fields, and this type never stores one.[^2]
///
/// The type is `Copy` and holds no allocation, so a level is one contiguous
/// array of them.
///
/// # References
///
/// [^1]: ADR-0024, every summary field is declared extensive or intensive, decision D2, a draft record. `docs/adrs/draft/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
/// [^2]: ADR-0024, every summary field is declared extensive or intensive, decision D3, a draft record. `docs/adrs/draft/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CellSummary {
    tiles: i64,
    open_tiles: i64,
    units: i64,
    value_total: Accum,
    height_total: Accum,
}

impl CellSummary {
    /// The identity of the combine operation.
    ///
    /// Combining the identity with any summary gives that summary back. A
    /// cell that covers no tile is the identity, which is what a fold over an
    /// empty set must return.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D1, a draft record. `docs/adrs/draft/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    pub const IDENTITY: Self = Self {
        tiles: 0,
        open_tiles: 0,
        units: 0,
        value_total: Accum(0),
        height_total: Accum(0),
    };

    /// Combines two summaries.
    ///
    /// The operation is field-wise integer addition. It is exactly
    /// associative and commutative, and its identity is [`Self::IDENTITY`],
    /// so a fold over a set gives one answer whatever the grouping and
    /// whatever the order.[^1] [^2]
    ///
    /// It also has an inverse, which is what would permit repairing a cell by
    /// removing a contribution rather than by rereading its children.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D1, a draft record. `docs/adrs/draft/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    /// [^2]: ADR-0023, an aggregate combines exactly, in any order, decision D2, a draft record. `docs/adrs/draft/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    /// [^3]: ADR-0023, an aggregate combines exactly, in any order, decision D4, a draft record. `docs/adrs/draft/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    #[must_use]
    pub const fn combine(self, other: Self) -> Self {
        Self {
            tiles: self.tiles.saturating_add(other.tiles),
            open_tiles: self.open_tiles.saturating_add(other.open_tiles),
            units: self.units.saturating_add(other.units),
            value_total: sim_math::combine(self.value_total, other.value_total),
            height_total: sim_math::combine(self.height_total, other.height_total),
        }
    }

    /// Removes a summary that was combined into this one.
    ///
    /// The combine operation has an inverse, so this returns the summary that
    /// would combine with `other` to give `self`.[^1] Nothing calls it yet:
    /// the pyramid rebuilds rather than repairs, and which path a change takes
    /// is a cost decision that no record settles.
    ///
    /// # References
    ///
    /// [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D4, a draft record. `docs/adrs/draft/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    #[must_use]
    pub const fn remove(self, other: Self) -> Self {
        Self {
            tiles: self.tiles.saturating_sub(other.tiles),
            open_tiles: self.open_tiles.saturating_sub(other.open_tiles),
            units: self.units.saturating_sub(other.units),
            value_total: Accum(self.value_total.0.saturating_sub(other.value_total.0)),
            height_total: Accum(self.height_total.0.saturating_sub(other.height_total.0)),
        }
    }

    /// Returns the tiles the summary covers. Extensive.
    #[must_use]
    pub const fn tiles(self) -> i64 {
        self.tiles
    }

    /// Returns the tiles whose ground admits a unit. Extensive.
    #[must_use]
    pub const fn open_tiles(self) -> i64 {
        self.open_tiles
    }

    /// Returns the units that stand on the ground it covers. Extensive.
    #[must_use]
    pub const fn units(self) -> i64 {
        self.units
    }

    /// Returns the sum of the tile values. Extensive.
    #[must_use]
    pub const fn value_total(self) -> Accum {
        self.value_total
    }

    /// Returns the sum of the tile heights. Extensive.
    #[must_use]
    pub const fn height_total(self) -> Accum {
        self.height_total
    }

    /// Returns the mean tile value. Intensive.
    ///
    /// The value is not stored. It is the sum divided by the tile count, both
    /// of which are stored, and the division happens here.[^1] A summary that
    /// covers no tile returns no value: a mean over nothing is not zero, and
    /// reporting it as zero gives a caller an answer it cannot tell from a
    /// true one.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0024, every summary field is declared extensive or intensive, decision D3, a draft record. `docs/adrs/draft/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
    /// [^2]: ADR-0024, every summary field is declared extensive or intensive, decision D5, a draft record. `docs/adrs/draft/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
    #[must_use]
    pub fn mean_value(self) -> Option<Fix32> {
        mean_of(self.value_total, self.tiles)
    }

    /// Returns the mean tile height. Intensive.
    ///
    /// The denominator is the tile count, because the ground gives every tile
    /// a height. A field defined over a subset would divide by the count of
    /// that subset.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0024, every summary field is declared extensive or intensive, decision D4, a draft record. `docs/adrs/draft/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
    #[must_use]
    pub fn mean_height(self) -> Option<Fix32> {
        mean_of(self.height_total, self.tiles)
    }

    /// Returns the share of the ground that admits a unit. Intensive.
    #[must_use]
    pub fn open_share(self) -> Option<Fix32> {
        ratio_of(self.open_tiles, self.tiles)
    }

    /// Returns the units for each tile of open ground. Intensive.
    ///
    /// The denominator is the open ground and not the whole cell, because a
    /// unit cannot stand on water. A field that borrowed the tile count here
    /// would report a lower crowd than the ground carries.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0024, every summary field is declared extensive or intensive, decision D4, a draft record. `docs/adrs/draft/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
    #[must_use]
    pub fn units_for_each_open_tile(self) -> Option<Fix32> {
        ratio_of(self.units, self.open_tiles)
    }

    /// Folds the summary into a state hash, field by field.
    #[must_use]
    pub fn hash_into(self, hash: StateHash) -> StateHash {
        hash.write_u64(self.tiles as u64)
            .write_u64(self.open_tiles as u64)
            .write_u64(self.units as u64)
            .write_u64(self.value_total.0 as u64)
            .write_u64(self.height_total.0 as u64)
    }
}

/// Returns an accumulated total divided by a count, as a fixed-point value.
///
/// The arithmetic is exact and integer throughout. The accumulator holds the
/// raw fixed-point bits of a sum, so dividing it by the count gives the raw
/// bits of the mean.[^1]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
fn mean_of(total: Accum, count: i64) -> Option<Fix32> {
    if count == 0 {
        return None;
    }
    Some(Fix32(clamp_to_fix(total.0 / count)))
}

/// Returns one count divided by another, as a fixed-point fraction.
fn ratio_of(part: i64, whole: i64) -> Option<Fix32> {
    if whole == 0 {
        return None;
    }
    Some(Fix32(clamp_to_fix(
        (part << crate::types::FIX_FRACTIONAL_BITS) / whole,
    )))
}

/// Clamps a wide value into the fixed-point range.
const fn clamp_to_fix(value: i64) -> i32 {
    if value > i32::MAX as i64 {
        i32::MAX
    } else if value < i32::MIN as i64 {
        i32::MIN
    } else {
        value as i32
    }
}

/// Level 1 of the pyramid.
///
/// The level is one array of summaries, one for each block of the layout. It
/// is derived, so it may be dropped and rebuilt at any time, and a save file
/// need not hold it.[^1]
///
/// # References
///
/// [^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1, a draft record. `docs/adrs/draft/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
#[derive(Clone, Debug)]
pub struct Pyramid {
    layout: BlockLayout,
    /// The part of each cell that the ground fixes: the tile count, the open
    /// ground and the height total. It is computed once, because the ground
    /// does not change for the life of a world.
    ground: Vec<CellSummary>,
    cells: Vec<CellSummary>,
}

impl Pyramid {
    /// Builds a level over a layout and reads the ground into it.
    ///
    /// The ground contribution of each cell is computed here and never again,
    /// because the ground does not change for the life of a world. A rebuild
    /// then reads only what a frame can change.[^1]
    ///
    /// **This runs on the calling thread.** It reads the ground of every tile
    /// of the world, which is one whole-world sweep and the only one the
    /// pyramid performs. It is the most expensive thing in building a world.
    ///
    /// It runs on one thread because no caller states a thread count when it
    /// builds a world, and a parameter that every caller passes as one is a
    /// branch nothing takes. A caller that needs this faster asks for the
    /// parameter, and the item that measured the cost says what it buys.[^2]
    ///
    /// # References
    ///
    /// [^2]: Backlog item 0046. `docs/backlog/proposed/0046-read-the-ground-of-a-new-world-in-parallel.md`
    ///
    /// The cells start at their ground contribution, so a level that nothing
    /// has rebuilt describes a world with no units rather than a world with no
    /// ground.
    ///
    /// # Errors
    ///
    /// Returns an error when the ground describes another world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1, a draft record. `docs/adrs/draft/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
    pub fn new(layout: BlockLayout, terrain: Terrain) -> Result<Self, BridgeError> {
        if layout.grid() != terrain.grid() {
            return Err(BridgeError::GridMismatch);
        }
        let count = layout.block_count() as usize;
        let ground: Vec<CellSummary> = (0..count as u32)
            .map(|block| ground_of_block(layout, terrain, block))
            .collect();
        Ok(Self {
            layout,
            cells: ground.clone(),
            ground,
        })
    }

    /// Returns the layout the level aggregates over.
    #[must_use]
    pub const fn layout(&self) -> BlockLayout {
        self.layout
    }

    /// Returns the number of cells the level holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Reports whether the level holds no cell.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Returns the summary of one cell.
    #[must_use]
    pub fn cell(&self, block: u32) -> Option<CellSummary> {
        self.cells.get(block as usize).copied()
    }

    /// Returns the summary of the cell that covers one tile.
    #[must_use]
    pub fn cell_covering(&self, address: Axial) -> Option<CellSummary> {
        let tile = self.layout.grid().index_of(address)?;
        let key = self.layout.key_of(tile)?;
        self.cell(self.layout.block_of_key(key))
    }

    /// Returns every cell, in block order.
    #[must_use]
    pub fn cells(&self) -> &[CellSummary] {
        &self.cells
    }

    /// Returns the combination of every cell.
    ///
    /// This is the whole world as one summary. It is what level 2 would hold
    /// if the world had one region, and it is the cheapest statement of the
    /// property that matters: a combination of the level below.[^1]
    ///
    /// The fold runs over the cells in block order, which is fixed. The
    /// operation is commutative, so the order does not change the answer, and
    /// the order is stated anyway because a reader should not have to prove
    /// that to know what this returns.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2, a draft record. `docs/adrs/draft/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn total(&self) -> CellSummary {
        self.cells
            .iter()
            .fold(CellSummary::IDENTITY, |total, cell| total.combine(*cell))
    }

    /// Rebuilds every cell from level 0.
    ///
    /// The rebuild reads the ground, the tile values and the derived unit
    /// structure, and writes the summaries. It is the one mechanism that
    /// maintains this level: no simulation system writes here.[^1]
    ///
    /// Each thread fills its own run of cells, and a cell is named by its
    /// block rather than by the thread that filled it, so the result never
    /// depends on which thread finished first.[^2]
    ///
    /// **A level with fewer cells than threads is filled on one thread.**
    /// Starting a thread for one cell costs more than the cell does, measured
    /// on a development machine. The rule reads the cell count and the thread
    /// count, both of which the caller supplied, and holds no constant of its
    /// own.
    ///
    /// The cells are visited in block order and the tiles of a cell in index
    /// order. Both are fixed.[^3]
    ///
    /// The unit count comes from the derived structure, which the barrier
    /// rebuilds before this runs. A second count of where units stand would
    /// be one fact in two places.[^4]
    ///
    /// # Errors
    ///
    /// Returns an error when the derived structure does not describe the
    /// arena, or when it was built over another layout.
    ///
    /// # References
    ///
    /// [^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D3, a draft record. `docs/adrs/draft/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
    /// [^2]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak. `docs/adrs/REGISTRY.md`
    /// [^3]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^4]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    pub fn rebuild(
        &mut self,
        values: &[Fix32],
        arena: &SoldierArena,
        bridge: &UnitTileBridge,
        threads: usize,
    ) -> Result<(), BridgeError> {
        if self.layout.grid() != arena.grid() {
            return Err(BridgeError::GridMismatch);
        }
        bridge.describes(arena)?;

        let layout = self.layout;
        let threads = threads.max(1);
        let ground = &self.ground[..];

        // A level with fewer cells than the caller has threads is small
        // enough that starting a thread costs more than the cell it would
        // fill. The rule uses the two numbers the caller already supplied and
        // no constant of its own.
        if self.cells.len() <= threads {
            for (block, cell) in self.cells.iter_mut().enumerate() {
                let moving = moving_part(layout, values, arena, bridge, block as u32)?;
                *cell = ground[block].combine(moving);
            }
            return Ok(());
        }

        let chunk_len = self.cells.len().div_ceil(threads).max(1);
        let mut refusal: Option<BridgeError> = None;
        std::thread::scope(|scope| {
            let mut handles = Vec::new();
            let mut base = 0u32;
            for chunk in self.cells.chunks_mut(chunk_len) {
                let first = base;
                base += chunk.len() as u32;
                handles.push(scope.spawn(move || {
                    for (offset, cell) in chunk.iter_mut().enumerate() {
                        let block = first + offset as u32;
                        let moving = moving_part(layout, values, arena, bridge, block)?;
                        *cell = ground[block as usize].combine(moving);
                    }
                    Ok(())
                }));
            }
            for handle in handles {
                if let Ok(Err(error)) = handle.join() {
                    refusal.get_or_insert(error);
                }
            }
        });

        match refusal {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    /// Folds every cell into a state hash, in block order.
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        self.cells
            .iter()
            .fold(hash, |hash, cell| cell.hash_into(hash))
    }
}

/// Returns the part of a cell that the ground fixes.
///
/// The tiles of a block are visited in the row-major order of the block, which
/// is fixed. The combine operation is commutative, so the order does not
/// change the answer.[^1]
///
/// A block at the edge of the world reaches past it. Those tiles are not part
/// of the world and contribute nothing, not even to the tile count, or the
/// denominator of every intensive reading would count ground that does not
/// exist.
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
fn ground_of_block(layout: BlockLayout, terrain: Terrain, block: u32) -> CellSummary {
    let mut summary = CellSummary::IDENTITY;
    for address in addresses_of_block(layout, block) {
        let Some(ground) = terrain.tile(address) else {
            continue;
        };
        summary = summary.combine(CellSummary {
            tiles: 1,
            open_tiles: i64::from(ground.kind.is_passable()),
            units: 0,
            value_total: Accum(0),
            height_total: sim_math::accumulate(Accum(0), ground.height),
        });
    }
    summary
}

/// Returns the part of a cell that a frame can change.
///
/// The unit count comes from the derived structure, which holds one contiguous
/// run for each block. A second count of where units stand would be one fact
/// in two places.[^1]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
fn moving_part(
    layout: BlockLayout,
    values: &[Fix32],
    arena: &SoldierArena,
    bridge: &UnitTileBridge,
    block: u32,
) -> Result<CellSummary, BridgeError> {
    let grid = layout.grid();
    let edge = layout.block_edge();
    let first_column = (block % layout.blocks_wide()) * edge;
    let first_row = (block / layout.blocks_wide()) * edge;

    // The value column is indexed row by row, so one row of a block is one
    // contiguous run of it. Summing a run rather than a tile at a time is the
    // difference between a coordinate conversion for each tile and none.
    let mut value_total = Accum(0);
    for row in first_row..(first_row + edge).min(grid.height()) {
        let start = (row * grid.width() + first_column) as usize;
        let end = (row * grid.width() + (first_column + edge).min(grid.width())) as usize;
        if start >= end || end > values.len() {
            continue;
        }
        for value in &values[start..end] {
            value_total = sim_math::accumulate(value_total, *value);
        }
    }

    Ok(CellSummary {
        tiles: 0,
        open_tiles: 0,
        units: bridge.in_block(arena, block)?.len() as i64,
        value_total,
        height_total: Accum(0),
    })
}

/// Returns every address a block covers, in the row-major order of the block.
///
/// The order is fixed and does not depend on how a caller visited the world.
/// An address outside the extent is returned too, and each caller drops it,
/// because the block is a rectangle and the world need not fill one.
fn addresses_of_block(layout: BlockLayout, block: u32) -> impl Iterator<Item = Axial> {
    let edge = layout.block_edge();
    let first_column = (block % layout.blocks_wide()) * edge;
    let first_row = (block / layout.blocks_wide()) * edge;
    (first_row..first_row + edge).flat_map(move |row| {
        (first_column..first_column + edge).map(move |column| Axial::new(column as i32, row as i32))
    })
}
