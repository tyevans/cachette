//! Luxury resources, and the variety score over them.
//!
//! A luxury is a presence, not a quantity. The three gatherable kinds are a
//! fixed catalogue, and a unit takes an amount of each one into a store.[^1] A
//! luxury is the opposite: a tile either carries one or it does not, and no
//! unit gathers it. The two tiers are separate types, and this module holds
//! only the second one.
//!
//! **A luxury lives on a tile.** Level 0 is the only truth, and every level
//! above it is derived from the tiles.[^2] A luxury on a site would move when
//! the site moved, and a region would then report a variety that followed the
//! building rather than the ground. A site reads the luxuries of the tile it
//! stands on, so no second declaration site exists.[^3]
//!
//! **A luxury set is one 64-bit word.** The set costs the same whatever the
//! number of luxuries in it, in the same way a set of factions does.[^4] The
//! catalogue therefore holds 64 luxuries, and this module refuses a placement
//! above that ceiling rather than folding it onto another bit.
//!
//! **The variety of a set is its population count.** The count is an exact
//! whole number, so no floating-point arithmetic reaches it.[^5] The union of
//! two sets is associative, commutative and idempotent, so a fold over a
//! group of tiles gives one answer whatever the order and whatever the
//! grouping.[^6]
//!
//! **The control plane seeds the field once, and the field never changes
//! after that.** A placement is authored content and not a draw, so nothing
//! here reads the counter-based generator.[^7] The field is simulated state
//! all the same, because two worlds that carry different luxuries are
//! different worlds, so the field enters the state hash.[^8]
//!
//! **Nothing in the engine consumes the variety score.** This module answers
//! a read and it modifies no pass. The register holds that decision, and it
//! holds the open question of what should consume it.[^9] [^10]
//!
//! No item in this module uses a floating-point type.[^5]
//!
//! # References
//!
//! [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D3. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
//! [^2]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
//! [^3]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
//! [^4]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D3. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
//! [^5]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^6]: ADR-0023, an aggregate combines exactly, in any order, decisions D1 and D2. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
//! [^7]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
//! [^8]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
//! [^9]: Decisions register, DEC-200. `docs/DECISIONS.md`
//! [^10]: Blockers register, BLK-110. `docs/BLOCKERS.md`

use bytemuck::{Pod, Zeroable};

use crate::bridge::BlockLayout;
use crate::hash::StateHash;
use crate::hex::Grid;
use crate::sim_math;
use crate::types::{Accum, TileIdx};

/// The number of luxuries that the catalogue addresses.
///
/// A luxury set is one 64-bit word, so the catalogue holds 64 luxuries and no
/// more. The value is a property of the word width that this project chose.
/// It is not a budget, and no measurement moves it.[^1]
///
/// **A placement above the ceiling is refused, never folded.** A set of
/// factions puts an unaddressable faction on an overflow bit, because the
/// question it answers is whether anybody holds the ground.[^2] A luxury set
/// answers how many different luxuries stand on the ground, and two luxuries
/// on one bit answer that question with the wrong number. The seed therefore
/// returns [`LuxuryError::IdAboveCeiling`] instead.
///
/// # References
///
/// [^1]: Decision Record Scope, section 4.1. `.claude/rules/adr-scope.md`
/// [^2]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D3. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
pub const LUXURY_CEILING: u8 = 64;

/// The identifier of one luxury.
///
/// The identifier is an index into the catalogue, and it is data. It is not a
/// type, not a trait and not a verb, so a new luxury adds no code and
/// multiplies no verb.[^1]
///
/// # References
///
/// [^1]: ADR-0007, content supplies a key vector, never a comparator, decision D3. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct LuxuryId(pub u8);

impl LuxuryId {
    /// Reports whether the catalogue addresses this identifier.
    #[must_use]
    pub const fn is_addressable(self) -> bool {
        self.0 < LUXURY_CEILING
    }
}

/// The reason that this module refused a caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuxuryError {
    /// The caller named a luxury that the catalogue does not address.
    ///
    /// The catalogue holds 64 luxuries, because a set of them is one 64-bit
    /// word. The value is the identifier that the caller gave.
    IdAboveCeiling(u8),
    /// The caller named a tile that lies outside the world.
    ///
    /// The value is the tile index that the caller gave.
    NoSuchTile(u32),
    /// The world already holds a luxury field.
    ///
    /// The field is seeded once and it never changes after that. A second
    /// seed would make the field a fact of a frame, and every reader of it
    /// would then have to say which frame it read.
    AlreadySeeded,
}

impl core::fmt::Display for LuxuryError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::IdAboveCeiling(id) => write!(
                formatter,
                "the catalogue addresses {LUXURY_CEILING} luxuries and the caller named {id}"
            ),
            Self::NoSuchTile(tile) => {
                write!(formatter, "the world holds no tile {tile}")
            }
            Self::AlreadySeeded => {
                write!(formatter, "the world already holds a luxury field")
            }
        }
    }
}

impl std::error::Error for LuxuryError {}

/// A set of luxuries.
///
/// The set is one 64-bit word, so it costs the same whatever the number of
/// luxuries in it.[^1] Bit `n` stands for the luxury whose identifier is `n`.
///
/// # References
///
/// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D3. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct LuxurySet(u64);

impl LuxurySet {
    /// The set that holds no luxury.
    pub const EMPTY: Self = Self(0);

    /// Returns the set that holds one luxury.
    ///
    /// Returns `None` when the catalogue does not address the identifier. The
    /// set never folds an unaddressable luxury onto another bit, because that
    /// reports two luxuries as one.
    #[must_use]
    pub const fn of(luxury: LuxuryId) -> Option<Self> {
        if luxury.is_addressable() {
            Some(Self(1u64 << luxury.0))
        } else {
            None
        }
    }

    /// Returns the set with one luxury added.
    ///
    /// Returns `None` when the catalogue does not address the identifier.
    #[must_use]
    pub const fn with(self, luxury: LuxuryId) -> Option<Self> {
        match Self::of(luxury) {
            Some(one) => Some(Self(self.0 | one.0)),
            None => None,
        }
    }

    /// Returns the union of two sets.
    ///
    /// The operation is associative, commutative and exact, so a fold over a
    /// group of tiles gives one answer whatever the order.[^1] It is also
    /// idempotent, so the union of a set with itself is that set.
    ///
    /// **The union has no inverse.** Two tiles that carry one luxury combine
    /// to one bit, and nothing in the result says that two tiles carried it.
    /// A caller that must take a tile out of a region rebuilds the region.
    ///
    /// # References
    ///
    /// [^1]: ADR-0023, an aggregate combines exactly, in any order, decisions D1 and D2. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Reports whether the set holds a luxury.
    #[must_use]
    pub const fn contains(self, luxury: LuxuryId) -> bool {
        match Self::of(luxury) {
            Some(one) => self.0 & one.0 != 0,
            None => false,
        }
    }

    /// Reports whether the set holds no luxury.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns the variety of the set.
    ///
    /// The variety is the number of different luxuries that the set holds. It
    /// is the population count of the word, so it is an exact whole number
    /// and no floating-point arithmetic reaches it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    #[must_use]
    pub const fn variety(self) -> u32 {
        self.0.count_ones()
    }

    /// Returns the set as a raw word. The state hash reads it.
    #[must_use]
    pub const fn to_bits(self) -> u64 {
        self.0
    }

    /// Rebuilds a set from the word that [`Self::to_bits`] gave.
    #[must_use]
    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }
}

/// The luxuries of one tile.
///
/// The layout is an 8-byte set, a 4-byte tile index and 4 declared bytes of
/// padding, at an alignment of 8. The declaration states the padding, so no
/// uninitialised byte reaches the state hash.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct LuxuryTile {
    /// The set of luxuries that the tile carries. The set is never empty.
    pub set: LuxurySet,
    /// The tile that carries them.
    pub tile: TileIdx,
    /// The declared padding. Every byte is zero.
    pub padding: [u8; 4],
}

/// Every luxury in one world.
///
/// **The field holds one entry for each tile that carries a luxury, and it
/// holds nothing else.** A world in which nobody seeded a luxury holds no
/// entry, so the memory cost follows the seeding and not the size of the
/// world.[^1] One word for every tile would cost 134 megabytes at the tile
/// count this project targets, and almost every word would be zero.[^2]
///
/// The entries are held sorted by tile, so a lookup is a binary search, and
/// the order never depends on the order that the caller listed the placements
/// in.[^3]
///
/// **The field is seeded once and it never changes.** Nothing in the engine
/// writes it after construction.
///
/// # References
///
/// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
/// [^2]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
/// [^3]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LuxuryField {
    tiles: Vec<LuxuryTile>,
}

impl LuxuryField {
    /// Builds a field that holds no luxury.
    #[must_use]
    pub const fn new() -> Self {
        Self { tiles: Vec::new() }
    }

    /// Builds a field from a set of placements.
    ///
    /// Each placement names a tile and a luxury. The caller gives the whole
    /// set in one call, and it gives the placements in any order. Two
    /// placements that name one tile combine by union, and the union is
    /// commutative, so the order between them changes nothing.[^1]
    ///
    /// The result is sorted by tile, whatever order the caller used.[^2]
    ///
    /// # Errors
    ///
    /// Returns [`LuxuryError::IdAboveCeiling`] when a placement names a
    /// luxury that the catalogue does not address. Returns
    /// [`LuxuryError::NoSuchTile`] when a placement names a tile outside the
    /// world. A refusal builds nothing.
    ///
    /// # References
    ///
    /// [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D2. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub fn seed(grid: Grid, placements: &[(TileIdx, LuxuryId)]) -> Result<Self, LuxuryError> {
        let mut rows: Vec<LuxuryTile> = Vec::with_capacity(placements.len());
        for (tile, luxury) in placements {
            let Some(one) = LuxurySet::of(*luxury) else {
                return Err(LuxuryError::IdAboveCeiling(luxury.0));
            };
            if tile.0 >= grid.tile_count() {
                return Err(LuxuryError::NoSuchTile(tile.0));
            }
            rows.push(LuxuryTile {
                set: one,
                tile: *tile,
                padding: [0; 4],
            });
        }
        // The sort is by tile alone, and it is stable. Two rows of one tile
        // then coalesce by union, and the union is commutative, so the answer
        // does not depend on which of the two the sort put first.
        rows.sort_by_key(|row| row.tile.0);
        let mut coalesced: Vec<LuxuryTile> = Vec::with_capacity(rows.len());
        for row in rows {
            match coalesced.last_mut() {
                Some(last) if last.tile == row.tile => {
                    last.set = last.set.union(row.set);
                }
                _ => coalesced.push(row),
            }
        }
        Ok(Self { tiles: coalesced })
    }

    /// Returns the number of tiles that carry a luxury.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    /// Reports whether no tile carries a luxury.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }

    /// Returns every tile that carries a luxury, in ascending tile order.
    #[must_use]
    pub fn tiles(&self) -> &[LuxuryTile] {
        &self.tiles
    }

    /// Returns the luxuries that one tile carries.
    ///
    /// A tile that carries none gives the empty set. The field stores no
    /// entry for such a tile, and the empty set is what that means.
    #[must_use]
    pub fn at(&self, tile: TileIdx) -> LuxurySet {
        match self.tiles.binary_search_by_key(&tile.0, |row| row.tile.0) {
            Ok(at) => self.tiles[at].set,
            Err(_) => LuxurySet::EMPTY,
        }
    }

    /// Returns the variety of one tile.
    ///
    /// The variety of a tile is the number of different luxuries on it.
    #[must_use]
    pub fn variety_at(&self, tile: TileIdx) -> u32 {
        self.at(tile).variety()
    }

    /// Returns the union of every luxury in the world.
    ///
    /// The variety of this set is the variety of the whole world. The fold
    /// runs over the entries in ascending tile order, and the union is
    /// commutative, so the answer does not depend on the order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D2. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    #[must_use]
    pub fn set(&self) -> LuxurySet {
        self.tiles
            .iter()
            .fold(LuxurySet::EMPTY, |total, row| total.union(row.set))
    }

    /// Returns the number of deposits in the world.
    ///
    /// A deposit is one luxury on one tile. A tile that carries three
    /// luxuries holds three deposits. The total is the sum of the variety of
    /// every tile, and it is an extensive quantity: two regions combine by
    /// adding their totals.[^1]
    ///
    /// **The accumulator is 64 bits wide.** The world that this project
    /// targets holds 16,777,216 tiles, and one tile carries at most 64
    /// luxuries, so the total reaches 1,073,741,824. A 32-bit accumulator
    /// holds that, and it holds it only by a margin. An accumulator must not
    /// depend on such a margin.[^2] [^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0024, every summary field is declared extensive or intensive, decision D2. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
    /// [^2]: ADR-0023, an aggregate combines exactly, in any order, decision D3. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    /// [^3]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
    #[must_use]
    pub fn deposits(&self) -> Accum {
        self.tiles.iter().fold(Accum(0), |total, row| {
            sim_math::combine(total, Accum(i64::from(row.set.variety())))
        })
    }

    /// Absorbs the field into the state hash.
    ///
    /// A luxury is simulated state. Two worlds that carry different luxuries
    /// are different worlds, and the whole-world hash must say so.[^1] The
    /// entries enter in tile order, which the field holds them in.[^2]
    ///
    /// The count enters first. A length that no hash reads is a length that a
    /// defect changes in silence.
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        let mut running = hash.write_u64(self.tiles.len() as u64);
        for row in &self.tiles {
            running = running
                .write(&row.tile.0.to_le_bytes())
                .write(&row.set.to_bits().to_le_bytes());
        }
        running
    }

    /// Reports whether the field holds its invariants.
    ///
    /// The entries rise, and they name each tile once. A field that broke
    /// either would answer a lookup with the wrong tile, and nothing else
    /// would notice.
    ///
    /// No entry is empty, because a tile that carries nothing is a tile that
    /// the field does not store. Two ways to say one thing is the defect
    /// shape that this project keeps meeting.[^1]
    ///
    /// Every tile lies inside the world, and every padding byte is zero.
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub fn check_invariants(&self, tile_count: u32) -> bool {
        if !self
            .tiles
            .windows(2)
            .all(|pair| pair[0].tile.0 < pair[1].tile.0)
        {
            return false;
        }
        self.tiles
            .iter()
            .all(|row| row.tile.0 < tile_count && !row.set.is_empty() && row.padding == [0; 4])
    }
}

/// The luxuries of every level 1 cell.
///
/// **The level is derived, and level 0 is the truth.** The whole level can be
/// thrown away and rebuilt from the field.[^1] Nothing here holds a fact of
/// its own, so the level does not enter the state hash.
///
/// **The level sits beside the cell summaries and not inside them.** A cell
/// summary holds extensive fields only, and its combine has an inverse.[^2]
/// [^3] A union of luxuries is idempotent and it has no inverse, so it does
/// not belong in that type, in the same way that a direction does not.[^4]
///
/// The level holds two answers for each cell. The union says how many
/// different luxuries the cell holds. The deposit total says how many
/// luxuries stand on its tiles, and it counts a repeat each time. The first
/// is the variety score. The second is extensive, and it combines by
/// addition.
///
/// # References
///
/// [^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
/// [^2]: ADR-0024, every summary field is declared extensive or intensive, decision D2. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
/// [^3]: ADR-0023, an aggregate combines exactly, in any order, decision D4. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
/// [^4]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D3. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VarietyLevel {
    layout: BlockLayout,
    sets: Vec<LuxurySet>,
    deposits: Vec<Accum>,
}

impl VarietyLevel {
    /// Derives the level from the field.
    ///
    /// The pass walks the entries of the field in ascending tile order, and
    /// it visits no tile that carries no luxury. Its cost follows the number
    /// of placements, not the size of the world.
    ///
    /// The pass runs on one thread and it writes each cell once, so no answer
    /// here depends on a thread count.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn derive(layout: BlockLayout, field: &LuxuryField) -> Self {
        let count = layout.block_count() as usize;
        let mut sets = vec![LuxurySet::EMPTY; count];
        let mut deposits = vec![Accum(0); count];
        for row in field.tiles() {
            let Some(key) = layout.key_of(row.tile) else {
                continue;
            };
            let block = layout.block_of_key(key) as usize;
            let Some(set) = sets.get_mut(block) else {
                continue;
            };
            *set = set.union(row.set);
            deposits[block] =
                sim_math::combine(deposits[block], Accum(i64::from(row.set.variety())));
        }
        Self {
            layout,
            sets,
            deposits,
        }
    }

    /// Returns the partition that the level is built over.
    #[must_use]
    pub const fn layout(&self) -> BlockLayout {
        self.layout
    }

    /// Returns the number of cells that the level holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.sets.len()
    }

    /// Reports whether the level holds no cell.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sets.is_empty()
    }

    /// Returns the luxuries of one cell.
    ///
    /// Returns `None` when the level holds no such cell.
    #[must_use]
    pub fn cell(&self, block: u32) -> Option<LuxurySet> {
        self.sets.get(block as usize).copied()
    }

    /// Returns the variety of one cell.
    ///
    /// Returns `None` when the level holds no such cell. The answer is the
    /// number of different luxuries on the tiles of the cell.
    #[must_use]
    pub fn variety(&self, block: u32) -> Option<u32> {
        self.cell(block).map(LuxurySet::variety)
    }

    /// Returns the deposits of one cell.
    ///
    /// Returns `None` when the level holds no such cell. The answer counts a
    /// luxury once for each tile that carries it.
    #[must_use]
    pub fn deposits(&self, block: u32) -> Option<Accum> {
        self.deposits.get(block as usize).copied()
    }

    /// Returns the luxuries of every cell, in ascending cell order.
    #[must_use]
    pub fn cells(&self) -> &[LuxurySet] {
        &self.sets
    }

    /// Returns the union of every cell.
    ///
    /// The fold runs over the cells in ascending order. The union is
    /// commutative and associative, so the answer does not depend on the
    /// order or on the grouping.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0023, an aggregate combines exactly, in any order, decisions D1 and D2. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    #[must_use]
    pub fn total(&self) -> LuxurySet {
        self.sets
            .iter()
            .fold(LuxurySet::EMPTY, |total, set| total.union(*set))
    }

    /// Returns the deposits of every cell.
    ///
    /// The accumulator is 64 bits wide, and the addition is exactly
    /// associative, so the answer does not depend on the fold order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D3. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    #[must_use]
    pub fn deposit_total(&self) -> Accum {
        self.deposits
            .iter()
            .fold(Accum(0), |total, part| sim_math::combine(total, *part))
    }
}
