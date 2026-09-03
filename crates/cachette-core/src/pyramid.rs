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
//! **A resource total is split across the two parts of a cell.** The stock a
//! tile started with is a pure function of the seed and the address, so it
//! joins the ground contribution and is read once.[^14] What a unit took from
//! a tile is a fact of a frame, so the rebuild subtracts it from the
//! ledger.[^15] The stored total is therefore what the tiles still hold, which
//! is what a tile reader reports for one tile.
//!
//! **The geometry is the block layout, and this module declares none.** The
//! derived unit structure partitions the world by the same block that the
//! pyramid aggregates over, so neither subsystem chooses the block edge
//! alone.[^10]
//!
//! # References
//!
//! [^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
//! [^2]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
//! [^3]: ADR-0024, every summary field is declared extensive or intensive, decision D2. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
//! [^4]: ADR-0024, every summary field is declared extensive or intensive, decision D3. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
//! [^5]: ADR-0023, an aggregate combines exactly, in any order, decision D3. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
//! [^6]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
//! [^7]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
//! [^8]: ADR-0023, an aggregate combines exactly, in any order, decision D2. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
//! [^9]: ADR-0023, an aggregate combines exactly, in any order, decision D4. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
//! [^10]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D2. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
//! [^11]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
//! [^12]: PRD-0003, a developer sees a world worth looking at. `docs/product/accepted/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
//! [^13]: ADR-0068, terrain is generated from the seed and is never stored as a map, the consequences. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
//! [^14]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D1. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
//! [^15]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`

use crate::bridge::{BlockLayout, BridgeError, UnitTileBridge};
use crate::choose::{field_value, Ranked, OPTIONS, OPTION_COUNT};
use crate::hash::StateHash;
use crate::hex::{Axial, Grid, NEIGHBOUR_COUNT};
use crate::holding::Holder;
use crate::resource::{ledger_key, Amount, DepletionLedger, ResourceField, ResourceKind};
use crate::sim_math;
use crate::soldier::SoldierArena;
use crate::tile_value::TileValues;
use crate::types::{Accum, FactionId, Fix32, TileIdx};

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
/// [^1]: ADR-0024, every summary field is declared extensive or intensive, decision D2. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
/// [^2]: ADR-0024, every summary field is declared extensive or intensive, decision D3. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CellSummary {
    tiles: i64,
    open_tiles: i64,
    units: i64,
    held_tiles: i64,
    value_total: Accum,
    height_total: Accum,
    food_total: Accum,
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
    /// [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    pub const IDENTITY: Self = Self {
        tiles: 0,
        open_tiles: 0,
        units: 0,
        held_tiles: 0,
        value_total: Accum(0),
        height_total: Accum(0),
        food_total: Accum(0),
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
    /// [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    /// [^2]: ADR-0023, an aggregate combines exactly, in any order, decision D2. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    /// [^3]: ADR-0023, an aggregate combines exactly, in any order, decision D4. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    #[must_use]
    pub const fn combine(self, other: Self) -> Self {
        Self {
            tiles: self.tiles.saturating_add(other.tiles),
            open_tiles: self.open_tiles.saturating_add(other.open_tiles),
            units: self.units.saturating_add(other.units),
            held_tiles: self.held_tiles.saturating_add(other.held_tiles),
            value_total: sim_math::combine(self.value_total, other.value_total),
            height_total: sim_math::combine(self.height_total, other.height_total),
            food_total: sim_math::combine(self.food_total, other.food_total),
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
    /// [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D4. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    #[must_use]
    pub const fn remove(self, other: Self) -> Self {
        Self {
            tiles: self.tiles.saturating_sub(other.tiles),
            open_tiles: self.open_tiles.saturating_sub(other.open_tiles),
            units: self.units.saturating_sub(other.units),
            held_tiles: self.held_tiles.saturating_sub(other.held_tiles),
            value_total: Accum(self.value_total.0.saturating_sub(other.value_total.0)),
            height_total: Accum(self.height_total.0.saturating_sub(other.height_total.0)),
            food_total: Accum(self.food_total.0.saturating_sub(other.food_total.0)),
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

    /// Returns the tiles that a faction holds. Extensive.
    ///
    /// The field counts the tiles whose holder is a faction rather than
    /// nobody. It does not say which faction, because a field indexed by the
    /// faction would multiply the world by the faction count, and the record
    /// rejects that shape.[^1] The accumulator is 64 bits wide, because a
    /// one-byte count summed over the tile count of the target world
    /// overflows a 32-bit accumulator.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D3. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    /// [^2]: ADR-0023, an aggregate combines exactly, in any order, decision D3. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    #[must_use]
    pub const fn held_tiles(self) -> i64 {
        self.held_tiles
    }

    /// Returns the share of the ground that a faction holds. Intensive.
    ///
    /// The value is not stored. It is the held count divided by the tile
    /// count, both of which are stored, and the division happens here.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0024, every summary field is declared extensive or intensive, decision D3. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
    #[must_use]
    pub fn held_share(self) -> Option<Fix32> {
        ratio_of(self.held_tiles, self.tiles)
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

    /// Returns the food that the tiles of the cell still hold. Extensive.
    ///
    /// The total is the food the ground generated, less what the depletion
    /// ledger says was taken from it. That is what a tile reader reports for
    /// one tile, so the cell is the exact combination of its tiles.[^1] [^2]
    ///
    /// The accumulator holds whole units of stock, because a stock is a whole
    /// number and never a fraction.[^3] It is 64 bits wide, because a
    /// one-byte tile field summed over the tile count of the target world
    /// overflows a 32-bit accumulator.[^4]
    ///
    /// The field covers the food kind alone. A total for each kind would
    /// treble the width of the summary, and nothing reads a wood total or a
    /// stone total.[^5]
    ///
    /// # References
    ///
    /// [^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
    /// [^2]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    /// [^3]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^4]: ADR-0023, an aggregate combines exactly, in any order, decision D3. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    /// [^5]: Recurring defect shapes, shape 3. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub const fn food_total(self) -> Accum {
        self.food_total
    }

    /// Returns the food for each tile of the cell. Intensive.
    ///
    /// The denominator is the tile count, because the ground gives every tile
    /// a food stock and water holds a stock of zero. A field defined over a
    /// subset would divide by the count of that subset.[^1]
    ///
    /// The total holds whole units, so the division scales the total into the
    /// fixed-point range before it divides. A summary that covers no tile
    /// returns no value.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0024, every summary field is declared extensive or intensive, decision D4. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
    /// [^2]: ADR-0024, every summary field is declared extensive or intensive, decision D5. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
    #[must_use]
    pub fn mean_food(self) -> Option<Fix32> {
        ratio_of(self.food_total.0, self.tiles)
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
    /// [^1]: ADR-0024, every summary field is declared extensive or intensive, decision D3. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
    /// [^2]: ADR-0024, every summary field is declared extensive or intensive, decision D5. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
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
    /// [^1]: ADR-0024, every summary field is declared extensive or intensive, decision D4. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
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
    /// [^1]: ADR-0024, every summary field is declared extensive or intensive, decision D4. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
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
            .write_u64(self.held_tiles as u64)
            .write_u64(self.value_total.0 as u64)
            .write_u64(self.height_total.0 as u64)
            .write_u64(self.food_total.0 as u64)
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

/// Returns one whole-number total divided by another, as a fixed-point value.
///
/// The numerator scales into the fixed-point range before the division, so a
/// total of whole numbers gives a mean with a fraction. Every caller here
/// holds whole numbers in both terms: a count of tiles, a count of units, or a
/// count of units of stock.
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
/// [^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
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
    /// The contribution holds the food the tiles started with, as well as the
    /// tile count, the open ground and the height total. The field reads the
    /// ground to generate a stock, so this takes the resource field rather
    /// than the ground alone.[^3]
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
    /// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
    /// [^3]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D1. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    pub fn new(layout: BlockLayout, resources: ResourceField) -> Result<Self, BridgeError> {
        if layout.grid() != resources.grid() {
            return Err(BridgeError::GridMismatch);
        }
        let count = layout.block_count() as usize;
        let ground: Vec<CellSummary> = (0..count as u32)
            .map(|block| ground_of_block(layout, resources, block))
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
    /// [^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn total(&self) -> CellSummary {
        self.cells
            .iter()
            .fold(CellSummary::IDENTITY, |total, cell| total.combine(*cell))
    }

    /// Rebuilds every cell from level 0.
    ///
    /// The rebuild reads the ground, the tile values, the tile holders, the
    /// depletion ledger and the derived unit structure, and writes the
    /// summaries. It is the one mechanism that
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
    /// [^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D3. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
    /// [^2]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
    /// [^3]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^4]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    pub fn rebuild(
        &mut self,
        values: &TileValues,
        holders: &[Holder],
        arena: &SoldierArena,
        bridge: &UnitTileBridge,
        depletion: &DepletionLedger,
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
                let moving = moving_part(
                    layout,
                    values,
                    holders,
                    arena,
                    bridge,
                    depletion,
                    block as u32,
                )?;
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
                        let moving =
                            moving_part(layout, values, holders, arena, bridge, depletion, block)?;
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
fn ground_of_block(layout: BlockLayout, resources: ResourceField, block: u32) -> CellSummary {
    let terrain = resources.terrain();
    let mut summary = CellSummary::IDENTITY;
    for address in addresses_of_block(layout, block) {
        let Some(ground) = terrain.tile(address) else {
            continue;
        };
        // The food a tile started with is a pure function of the seed and the
        // address, in the same way the height is, so it is read here and never
        // again.[^1] What a unit took from it is a fact of a frame, and the
        // moving part below subtracts that.[^2]
        //
        // [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D1. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
        // [^2]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
        let food = resources
            .original(address, ResourceKind::Food)
            .unwrap_or(Amount::ZERO);
        summary = summary.combine(CellSummary {
            tiles: 1,
            open_tiles: i64::from(ground.kind.is_passable()),
            units: 0,
            held_tiles: 0,
            value_total: Accum(0),
            height_total: sim_math::accumulate(Accum(0), ground.height),
            food_total: food.to_accum(),
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
    values: &TileValues,
    holders: &[Holder],
    arena: &SoldierArena,
    bridge: &UnitTileBridge,
    depletion: &DepletionLedger,
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
    let mut held_tiles = 0i64;
    let mut food_taken = 0i64;
    for row in first_row..(first_row + edge).min(grid.height()) {
        let start = (row * grid.width() + first_column) as usize;
        let end = (row * grid.width() + (first_column + edge).min(grid.width())) as usize;
        if start >= end || end > values.tile_count() || end > holders.len() {
            continue;
        }
        // The field holds no array of values, so the run is read one tile at
        // a time. The tiles of the run are contiguous, so the read converts
        // no coordinate.
        for index in start..end {
            let Some(value) = values.at(TileIdx(index as u32)) else {
                continue;
            };
            value_total = sim_math::accumulate(value_total, value);
        }
        // The holder column is indexed the same way as the value column, so
        // one row of a block is one contiguous run of it too.
        for holder in &holders[start..end] {
            held_tiles += i64::from(!holder.is_nobody());
        }
        food_taken += food_taken_in_run(depletion, start as u32, end as u32);
    }

    Ok(CellSummary {
        tiles: 0,
        open_tiles: 0,
        units: bridge.in_block(arena, block)?.len() as i64,
        held_tiles,
        value_total,
        height_total: Accum(0),
        // The ground part holds the food the tiles started with, so the moving
        // part holds what was taken, as a negative amount. Nothing takes more
        // from a tile than the tile ever held, and the world invariant is what
        // checks that, so the sum of the two parts is never below zero.[^1]
        //
        // [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D5. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
        food_total: Accum(-food_taken),
    })
}

/// Returns the food taken from one contiguous run of tiles.
///
/// The ledger holds one entry for each tile and kind that somebody gathered
/// from, in ascending key order, and a world in which nothing was gathered
/// holds no entry.[^1] [^2] The key of a tile rises with the tile index, so
/// one run of tiles is one contiguous span of the ledger, and a search finds
/// the start of that span.
///
/// **A run whose tiles hold no stored take costs one search and no per-tile
/// read.** A search over an empty ledger returns at once, so a world that
/// gathered nothing pays one search for each row of each block and nothing
/// else. No cost figure here is measured, because no measurement exists on
/// the target platform.[^3]
///
/// The scan visits the entries in key order, which is fixed.[^4]
///
/// # References
///
/// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
/// [^2]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
/// [^3]: Blockers register, BLK-007. `docs/BLOCKERS.md`
/// [^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
fn food_taken_in_run(depletion: &DepletionLedger, first: u32, end: u32) -> i64 {
    let entries = depletion.entries();
    if entries.is_empty() || first >= end {
        return 0;
    }
    let low = ledger_key(TileIdx(first), ResourceKind::Food);
    let high = ledger_key(TileIdx(end), ResourceKind::Food);
    let start = entries.partition_point(|entry| entry.key < low);
    let mut taken = 0i64;
    for entry in &entries[start..] {
        if entry.key >= high {
            break;
        }
        if ResourceKind::from_u8((entry.key & 0b11) as u8) == Some(ResourceKind::Food) {
            taken += i64::from(entry.taken);
        }
    }
    taken
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

/// The exit direction that means a cell holds none.
///
/// The value sits at the top of the byte range, which no direction index
/// reaches, because the neighbour count is far below it. It is a property of
/// the array layout and not a budget.
pub const NO_EXIT: u8 = u8::MAX;

/// The direction that each cell holds, for each option.
///
/// **Movement takes its direction from this array, and never from a per-unit
/// search over the neighbouring cells.**[^1] A unit reads the entry of its
/// cell and its option, and it steps to the neighbouring tile in that
/// direction. The cost of the array follows the cell count and the option
/// count. It does not follow the population.
///
/// The array is a projection of level 0. The engine derives every entry again
/// at each rebuild of level 1, from the summaries that the rebuild produced,
/// and nothing accumulates between two derivations.[^2] Level 0 stays the only
/// source of truth, and this array states no fact of its own.[^3]
///
/// **The array is not a summary field.** A summary field is extensive, and two
/// summaries combine by adding their fields. Two directions do not add, so the
/// array sits beside the level and never inside it.[^4] [^5]
///
/// It holds no floating point value. A direction is a small unsigned
/// integer.[^6]
///
/// The array does not reach the state hash. It is an exact function of the
/// summaries, and those are hashed, so hashing it as well would state one fact
/// in two places with nothing to fail when the copies disagree.[^7]
///
/// **The array is indexed by the cell, and writing it once for each tile was
/// measured and refused.** A tile-indexed copy would let a unit read at its own
/// tile index and drop the arithmetic in front of the read. It loses twice.
/// The pass that fills it costs more than the whole frame budget, and the read
/// itself is slower than the read it replaces, because this array is small
/// enough to stay in cache for a whole pass and a tile-indexed one is not.[^8]
/// [^9]
///
/// # References
///
/// [^1]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
/// [^2]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D2. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
/// [^3]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
/// [^4]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D3. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
/// [^5]: ADR-0024, every summary field is declared extensive or intensive, decision D2. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
/// [^6]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
/// [^7]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
/// [^8]: Findings register, FND-281. `docs/FINDINGS.md`
/// [^9]: The exit locality benchmark. `crates/cachette-core/benches/exit_locality.rs`
#[derive(Clone, Debug)]
pub struct ExitField {
    cells: Grid,
    /// One entry for each cell and each option, at the cell index times the
    /// option count plus the option index.
    directions: Vec<u8>,
}

impl ExitField {
    /// Builds a field over a cell lattice, with no direction anywhere.
    ///
    /// The lattice is a hex grid at the pitch of one level 1 block, which is
    /// the lattice the influence field already solves over. This type declares
    /// no geometry of its own.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D2. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    #[must_use]
    pub fn new(cells: Grid) -> Self {
        let count = cells.tile_count() as usize * OPTION_COUNT;
        Self {
            cells,
            directions: vec![NO_EXIT; count],
        }
    }

    /// Returns the cell lattice the field covers.
    #[must_use]
    pub const fn cells(&self) -> Grid {
        self.cells
    }

    /// Returns the exit direction of one cell and one option.
    ///
    /// The outer option reports whether the cell and the option name an entry.
    /// The inner one reports whether the cell holds a direction at all. A cell
    /// that no neighbour beats keeps none, and a unit there falls back to the
    /// uniform draw.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D4. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
    #[must_use]
    pub fn exit(&self, cell: u32, option: u8) -> Option<Option<u8>> {
        if option as usize >= OPTION_COUNT {
            return None;
        }
        let at = (cell as usize).checked_mul(OPTION_COUNT)? + option as usize;
        let direction = *self.directions.get(at)?;
        Some(if direction == NO_EXIT {
            None
        } else {
            Some(direction)
        })
    }

    /// Derives every entry from a level 1.
    ///
    /// **A cell ranks its neighbours on the value that the option reads from a
    /// cell, and never on the score of the option.** A score multiplies that
    /// value by what one unit wants. A want of zero makes every score equal,
    /// and the multiplication saturates, so under either property the
    /// tie-break would decide the direction and the ground would not.[^1] [^2]
    ///
    /// The scan reads the six directions in ascending direction index and
    /// compares strictly, so the lowest direction index wins a tie. That is the
    /// order that every other walk over the neighbours of a hex uses, and it is
    /// the rule the choice pass already uses for a tie between two
    /// options.[^3] [^4]
    ///
    /// The running best starts at the value of the cell itself, so a neighbour
    /// must beat the ground the unit already stands on. A cell that no
    /// neighbour beats holds no direction.[^1]
    ///
    /// A neighbour outside the lattice is not a candidate. The world does not
    /// wrap.
    ///
    /// **The pass writes every entry and accumulates nothing.** Deriving twice
    /// from one level 1 gives one answer, so the field carries nothing between
    /// two frames.[^5]
    ///
    /// The pass runs on the calling thread. It reads the summaries and writes
    /// the directions, in ascending cell order and then in ascending option
    /// order, so the result names no thread and depends on no thread
    /// count.[^3] No figure appears here, because no measurement exists on the
    /// target platform.[^6]
    ///
    /// # References
    ///
    /// [^1]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D4. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
    /// [^2]: Findings register, FND-190. `docs/FINDINGS.md`
    /// [^3]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^4]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D5. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
    /// [^5]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D2. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
    /// [^6]: Blockers register, BLK-007. `docs/BLOCKERS.md`
    pub fn derive(&mut self, pyramid: &Pyramid) {
        let cells = self.cells;
        for cell in 0..cells.tile_count() {
            let (Some(here), Some(mine)) = (cells.address_of(TileIdx(cell)), pyramid.cell(cell))
            else {
                continue;
            };
            for (option, row) in OPTIONS.iter().enumerate() {
                // **A row that ranks no cell field holds no exit anywhere.**
                // The exit field ranks a neighbouring cell on a value the
                // cell carries, and a row that ranks the state of the unit
                // names no such value. A separate field steers that row.[^13]
                //
                // The entry stays in the array. Dropping it would make the
                // index of a row differ from the index of its entry, which is
                // one order declared twice.[^14]
                //
                // [^13]: ADR-0108, a unit returns by climbing a reach field seeded at every site of its faction, decision D1. `docs/adrs/draft/adr-0108-a-unit-returns-by-climbing-a-reach-field.md`
                // [^14]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
                let Ranked::Cell(field) = row.ranked else {
                    self.directions[cell as usize * OPTION_COUNT + option] = NO_EXIT;
                    continue;
                };
                let mut best = NO_EXIT;
                let mut best_value = field_value(mine, field);
                for direction in direction_order() {
                    let Some(there) = cells.neighbour(here, direction) else {
                        continue;
                    };
                    let Some(index) = cells.index_of(there) else {
                        continue;
                    };
                    let Some(summary) = pyramid.cell(index.0) else {
                        continue;
                    };
                    // **A cell that admits no unit is not a candidate.** The
                    // rank reads a field that says nothing about whether a
                    // unit may stand there, so open water outranks dry ground
                    // on any field that water scores well on, and a whole
                    // block is then sent at a coast it can never cross.[^10]
                    //
                    // The open tile count is the one statement of how much of
                    // a cell admits a unit, and it is the same count the open
                    // share reads. A second rule here would be that fact in
                    // two places.[^11] [^12]
                    //
                    // This refuses a cell that admits nobody at all. A cell
                    // that admits somebody stays a candidate, whatever the
                    // shape of the ground inside it, because the field holds
                    // one direction for the whole block and the tile a unit
                    // stands against is not a fact the block carries.[^10]
                    //
                    // [^10]: Findings register, FND-315. `docs/FINDINGS.md`
                    // [^11]: ADR-0024, every summary field is declared extensive or intensive, decision D4. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
                    // [^12]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
                    if summary.open_tiles() == 0 {
                        continue;
                    }
                    let value = field_value(summary, field);
                    if value > best_value {
                        best_value = value;
                        best = direction as u8;
                    }
                }
                self.directions[cell as usize * OPTION_COUNT + option] = best;
            }
        }
    }
}

/// Returns the order in which the derivation scans the six directions.
///
/// The order is ascending direction index. The comparison is strict, so the
/// lowest direction index wins a tie between two equal neighbours.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decisions D1 and D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
#[cfg(not(feature = "probe-nondeterminism"))]
const fn direction_order() -> [usize; NEIGHBOUR_COUNT] {
    [0, 1, 2, 3, 4, 5]
}

/// Returns the directions in descending index order, which is a defect.
///
/// This is the perturbed build. The scan reads the neighbours from the top, so
/// the strict comparison now gives a tie to the **highest** direction index.
/// The field is still deterministic and still gives one answer at any thread
/// count, so neither determinism test can see it. Only a test that builds two
/// equal neighbours and names the winner can.[^1]
///
/// The whole point is that it must fail. A determinism test with no proven
/// failure mode is decoration.[^2]
///
/// # References
///
/// [^1]: Testing rules, section 2. `.claude/rules/testing.md`
/// [^2]: Testing rules, section 1. `.claude/rules/testing.md`
#[cfg(feature = "probe-nondeterminism")]
const fn direction_order() -> [usize; NEIGHBOUR_COUNT] {
    [5, 4, 3, 2, 1, 0]
}

/// The reach that means a cell reached no site of a faction.
///
/// The value sits at the top of the byte range. No reach reaches it, because
/// the relaxation runs a fixed number of passes and each pass adds one.
pub const UNREACHED: u8 = u8::MAX;

/// The number of relaxation passes that one derivation of the return field
/// runs.
///
/// **The count is the reach, in cells.** Each pass carries the reach of a cell
/// one step outward, so a cell further than this from every site of its
/// faction holds no direction and a unit there keeps the behaviour it already
/// has. The register holds the reasoning that accepts a reach limit, and the
/// reasoning that several sites seed one field so the limit binds on the
/// spacing of sites rather than on the size of the world.[^1]
///
/// The solve reads no residual and tests no convergence. It runs this count
/// whatever the field holds.[^2]
///
/// The value matches the pass count of the influence solve, so one number
/// says how far a field reaches over this lattice. A reader who changes one
/// should read the other. **No cost figure supports either**, and one blocker
/// governs every cost figure this project holds.[^3] [^4]
///
/// # References
///
/// [^1]: Decisions register, DEC-095. `docs/DECISIONS.md`
/// [^2]: ADR-0005, a solver runs a fixed iteration count, decision D1. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
/// [^3]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
/// [^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
pub const RETURN_PASSES: u32 = 8;

/// The direction of the nearest site of a faction, for each level 1 cell.
///
/// **A unit that carries a load home reads one entry and steps.** It reads no
/// neighbouring cell, it scores no neighbour, and it computes nothing from its
/// own address toward its own site. The direction belongs to the cell and to
/// the faction, and every unit of that cell and that faction reads one
/// answer.[^1] [^2]
///
/// The field holds one plane for each faction. The faction is the major
/// index, so the plane of one faction is one contiguous run, which is the
/// layout the influence field over the same lattice already uses.[^3]
///
/// **A field indexed by the faction is refused at level 1 and admitted
/// here.** A summary field indexed by the faction would multiply the tile side
/// of the world by the faction count. This field is at the pitch of one level
/// 1 cell, where the influence field is already one plane for each
/// faction.[^4] [^5]
///
/// The field is derived again at every rebuild of level 1, from nothing. It
/// carries no value between two frames, so it states no fact of its own and
/// level 0 stays the only source of truth.[^6] [^7]
///
/// # References
///
/// [^1]: ADR-0095, a behavioural strategy arrives as a field over cells, never as a search from a unit, decision D1. `docs/adrs/draft/adr-0095-a-behavioural-strategy-arrives-as-a-field-over-cells.md`
/// [^2]: ADR-0108, a unit returns by climbing a reach field seeded at every site of its faction, decision D1. `docs/adrs/draft/adr-0108-a-unit-returns-by-climbing-a-reach-field.md`
/// [^3]: ADR-0060, an influence map is stored as a shared basis, decision D1. `docs/adrs/draft/adr-0060-an-influence-map-is-stored-as-a-shared-basis.md`
/// [^4]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D3. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
/// [^5]: ADR-0108, a unit returns by climbing a reach field seeded at every site of its faction, decision D3. `docs/adrs/draft/adr-0108-a-unit-returns-by-climbing-a-reach-field.md`
/// [^6]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
/// [^7]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D2. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
#[derive(Clone, Debug)]
pub struct ReturnField {
    cells: Grid,
    faction_count: u16,
    /// One direction for each faction and cell. The faction is the major
    /// index.
    directions: Vec<u8>,
    /// The steps from each cell to the nearest seed of the faction that the
    /// derivation is working on. It holds one plane, and the derivation
    /// reuses it for every faction.
    reach: Vec<u8>,
    /// The write half of one relaxation pass.
    scratch: Vec<u8>,
}

impl ReturnField {
    /// Builds a field over a cell lattice, with no direction anywhere.
    #[must_use]
    pub fn new(cells: Grid, faction_count: u16) -> Self {
        let count = cells.tile_count() as usize;
        Self {
            cells,
            faction_count,
            directions: vec![NO_EXIT; count * faction_count as usize],
            reach: vec![UNREACHED; count],
            scratch: vec![UNREACHED; count],
        }
    }

    /// Returns the cell lattice the field covers.
    #[must_use]
    pub const fn cells(&self) -> Grid {
        self.cells
    }

    /// Returns the number of factions the field holds a plane for.
    #[must_use]
    pub const fn faction_count(&self) -> u16 {
        self.faction_count
    }

    /// Returns the direction of one faction and one cell.
    ///
    /// The outer option reports whether the faction and the cell name an
    /// entry. The inner one reports whether the cell holds a direction at all.
    /// A cell that holds a site, and a cell that reached no site, both hold
    /// none, and a unit there falls back to the keyed draw.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D6. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
    #[must_use]
    pub fn direction(&self, faction: FactionId, cell: u32) -> Option<Option<u8>> {
        let at = self.slot(faction, cell)?;
        let direction = *self.directions.get(at)?;
        Some(if direction == NO_EXIT {
            None
        } else {
            Some(direction)
        })
    }

    /// Returns the slot of one faction and one cell.
    fn slot(&self, faction: FactionId, cell: u32) -> Option<usize> {
        if faction.0 >= self.faction_count {
            return None;
        }
        let count = self.cells.tile_count();
        if cell >= count {
            return None;
        }
        Some(faction.0 as usize * count as usize + cell as usize)
    }

    /// Derives every entry from a level 1 and a set of seeds.
    ///
    /// A seed is one faction and the cell that holds a site of it. **Several
    /// sites of one faction seed one plane at once**, so one derivation serves
    /// every unit of that faction and the field carries the direction of the
    /// nearest site to each cell.[^1]
    ///
    /// The reach of a seed cell is zero. Each pass gives a cell one more than
    /// the smallest reach of its neighbours, when that is smaller than the
    /// reach it holds. The pass reads one plane and writes another, so no cell
    /// of one pass reads a value that the same pass wrote and the answer does
    /// not depend on the order the cells were visited in.[^2]
    ///
    /// **A cell that admits no unit is not a candidate, and it conducts
    /// nothing.** A cell of open water would otherwise carry the reach across
    /// a lake and send a whole block at a coast it can never cross. The rule
    /// reads the open tile count, which is the same count the open share
    /// reads, so it states no second rule of its own.[^3] [^4]
    ///
    /// The direction of a cell is the first neighbour, in ascending direction
    /// index, whose reach is strictly smaller than the reach of the cell. The
    /// lowest direction index therefore wins a tie, which is the order every
    /// other walk over the neighbours of a hex uses.[^2]
    ///
    /// The pass runs on the calling thread, in ascending faction identifier
    /// and then in ascending cell index. It names no thread and depends on no
    /// thread count.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0095, a behavioural strategy arrives as a field over cells, never as a search from a unit, decision D3. `docs/adrs/draft/adr-0095-a-behavioural-strategy-arrives-as-a-field-over-cells.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^3]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D5. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
    /// [^4]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    pub fn derive(&mut self, pyramid: &Pyramid, seeds: &[(FactionId, u32)]) {
        let cells = self.cells;
        let count = cells.tile_count();
        for faction in 0..self.faction_count {
            self.reach.iter_mut().for_each(|cell| *cell = UNREACHED);
            for (seeded, cell) in seeds {
                if seeded.0 != faction || *cell >= count {
                    continue;
                }
                if !admits_a_unit(pyramid, *cell) {
                    continue;
                }
                self.reach[*cell as usize] = 0;
            }
            for _ in 0..RETURN_PASSES {
                self.relax(pyramid);
            }
            for cell in 0..count {
                let at = faction as usize * count as usize + cell as usize;
                self.directions[at] = self.step_down(cell);
            }
        }
    }

    /// Runs one relaxation pass over the reach plane.
    fn relax(&mut self, pyramid: &Pyramid) {
        let cells = self.cells;
        for cell in 0..cells.tile_count() {
            let index = cell as usize;
            let here = self.reach[index];
            if !admits_a_unit(pyramid, cell) {
                self.scratch[index] = UNREACHED;
                continue;
            }
            let mut nearest = here;
            if let Some(address) = cells.address_of(TileIdx(cell)) {
                for direction in direction_order() {
                    let Some(there) = cells.neighbour(address, direction) else {
                        continue;
                    };
                    let Some(at) = cells.index_of(there) else {
                        continue;
                    };
                    let reach = self.reach[at.0 as usize];
                    if reach < UNREACHED && reach.saturating_add(1) < nearest {
                        nearest = reach.saturating_add(1);
                    }
                }
            }
            self.scratch[index] = nearest;
        }
        self.reach.copy_from_slice(&self.scratch);
    }

    /// Returns the direction of the neighbour that is nearer to a seed.
    fn step_down(&self, cell: u32) -> u8 {
        let here = self.reach[cell as usize];
        if here == UNREACHED || here == 0 {
            return NO_EXIT;
        }
        let Some(address) = self.cells.address_of(TileIdx(cell)) else {
            return NO_EXIT;
        };
        for direction in direction_order() {
            let Some(there) = self.cells.neighbour(address, direction) else {
                continue;
            };
            let Some(at) = self.cells.index_of(there) else {
                continue;
            };
            if self.reach[at.0 as usize] < here {
                return direction as u8;
            }
        }
        NO_EXIT
    }
}

/// Reports whether one cell of a level 1 holds any ground that admits a unit.
///
/// A cell outside the level 1 admits nobody. The count is the one the open
/// share reads, so this states no second rule.[^1]
///
/// # References
///
/// [^1]: ADR-0024, every summary field is declared extensive or intensive, decision D4. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
fn admits_a_unit(pyramid: &Pyramid, cell: u32) -> bool {
    pyramid
        .cell(cell)
        .is_some_and(|summary| summary.open_tiles() > 0)
}
