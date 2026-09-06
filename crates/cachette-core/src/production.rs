//! The production queue of a site, and the cost table it reads.
//!
//! # Why this exists
//!
//! A faction gets people in one way today: the seeding founds a group once,
//! and from that tick the population only falls. Every founded unit carries
//! the default row of the unit type table, so no faction can raise a cohort
//! that kills and no faction can send a settler to found a second city.[^1]
//!
//! This module holds the mechanism that answers both. A site holds a bounded,
//! ordered queue. Each entry names a unit type and the work done toward it.
//! One stage advances the front entry of each site, and the store pays as the
//! entry advances. A finished entry takes one resident of the site and the
//! goods the entry costs, and one unit of that type arrives at the site.[^2]
//!
//! # What it does not hold
//!
//! **It holds no policy about what to queue.** The engine holds the
//! mechanism, the bound and the refusals. No rule here names a unit type, and
//! no pass compares a type index against a constant.[^3]
//!
//! It holds no source of people. Organic growth is the only source, and a
//! separate record holds it.[^4]
//!
//! # Determinism
//!
//! The queue of one site is an array in position order. The position is the
//! order the entries were queued, and nothing here reorders them.[^5] The
//! advance visits the sites in slot order. No item in this module draws, and
//! no item uses a floating-point type.[^6]
//!
//! # The values
//!
//! Every value below is a placeholder. The balance register holds one row for
//! each of them, marks each unset, and records how the placeholder was
//! chosen.[^7]
//!
//! # References
//!
//! [^1]: Findings register, FND-486. `docs/FINDINGS.md`
//! [^2]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decisions D1, D3 and D4. `docs/adrs/draft/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
//! [^3]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D2. `docs/adrs/draft/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
//! [^4]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D5. `docs/adrs/draft/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
//! [^5]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
//! [^6]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^7]: Balance register, the production queue. `docs/reference/balance.md`

use bytemuck::{Pod, Zeroable};

use crate::hash::StateHash;
use crate::site::{CommodityId, COMMODITY_COUNT};
use crate::types::{Entity, FactionId, Fix32};
use crate::unit_type::{UnitTypeId, UNIT_TYPE_COUNT};

/// The entries that one site holds at once, at the most.
///
/// **This is the width of the stored block and not a budget.** It multiplies
/// the site count to give the cost of the advance stage, so it is small and
/// it is fixed. It never follows the population.[^1]
///
/// The bound a world enforces is a parameter of the world, and it is at most
/// this number. The value is a placeholder, and the balance register holds
/// the row.[^2]
///
/// # References
///
/// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decisions D1 and D5. `docs/adrs/draft/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
/// [^2]: Balance register, the production queue, the queue bound row. `docs/reference/balance.md`
pub const QUEUE_BOUND: usize = 4;

/// The work that one advance adds to the front entry.
///
/// The value is one, so the work column of the cost table counts the advances
/// a build takes. Two values would be two ways of saying one thing, and the
/// register would then hold a rate and a total that could disagree.[^1]
///
/// # References
///
/// [^1]: Balance register, the production queue, the work row. `docs/reference/balance.md`
pub const WORK_PER_ADVANCE: u32 = 1;

/// The placeholder work that one entry needs before it finishes.[^1]
///
/// # References
///
/// [^1]: Balance register, the production queue, the work row. `docs/reference/balance.md`
pub const PLACEHOLDER_BUILD_WORK: u32 = 8;

/// The placeholder number of residents that one finished entry takes.[^1]
///
/// # References
///
/// [^1]: Balance register, the production queue, the people row. `docs/reference/balance.md`
pub const PLACEHOLDER_BUILD_PEOPLE: u32 = 1;

/// The placeholder quantity of one good that one advance costs.[^1]
///
/// # References
///
/// [^1]: Balance register, the production queue, the charge row. `docs/reference/balance.md`
pub const PLACEHOLDER_ADVANCE_CHARGE: Fix32 = Fix32::ONE;

/// The placeholder quantity of one good that one finished entry costs.[^1]
///
/// # References
///
/// [^1]: Balance register, the production queue, the goods row. `docs/reference/balance.md`
pub const PLACEHOLDER_BUILD_GOODS: Fix32 = Fix32::from_int(8);

/// The placeholder period of the queue schedule.[^1]
///
/// # References
///
/// [^1]: Balance register, the production queue, the schedule row. `docs/reference/balance.md`
pub const QUEUE_PERIOD_DEFAULT: u32 = 10;

/// The placeholder phase of the queue schedule.[^1]
///
/// # References
///
/// [^1]: Balance register, the production queue, the schedule row. `docs/reference/balance.md`
pub const QUEUE_PHASE_DEFAULT: u32 = 0;

/// One entry of the queue of one site.
///
/// The entry is plain data with a declared layout and declared padding, so
/// the state hash reads its bytes and carries no uninitialised byte.[^1]
///
/// **It holds no identity of a unit**, because no unit exists until the entry
/// finishes.[^2]
///
/// The layout is 4 + 1 + 1 + 2 bytes, which is 8 bytes at an alignment of 4.
/// The trailing array declares every padding byte.
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
/// [^2]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D1. `docs/adrs/draft/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct QueueEntry {
    /// The work done toward the type. It never passes the work the row asks
    /// for.
    pub work: u32,
    /// The type the entry names.
    pub unit_type: UnitTypeId,
    /// One while the position holds an entry, zero while it is empty.
    pub filled: u8,
    /// Declared padding, always zero.
    pub padding: [u8; 2],
}

/// The size of one queue entry, in bytes.
pub const QUEUE_ENTRY_BYTES: usize = 8;

const _: () = assert!(core::mem::size_of::<QueueEntry>() == QUEUE_ENTRY_BYTES);

impl QueueEntry {
    /// The empty position.
    pub const EMPTY: Self = Self {
        work: 0,
        unit_type: UnitTypeId(0),
        filled: 0,
        padding: [0; 2],
    };

    /// Builds an entry that names a type and holds no work.
    #[must_use]
    pub const fn new(unit_type: UnitTypeId) -> Self {
        Self {
            work: 0,
            unit_type,
            filled: 1,
            padding: [0; 2],
        }
    }

    /// Reports whether the position holds an entry.
    #[must_use]
    pub const fn is_filled(&self) -> bool {
        self.filled != 0
    }
}

/// One row of the build cost table, for one unit type.
///
/// **The costs sit in their own table and not in the unit type row.** A unit
/// type row is a set of capability columns, and a zero in one means that the
/// type cannot do what the column names.[^1] A build work is not a capability:
/// a work of zero would mean a type that finishes at once, not a type that
/// cannot be built. Two meanings for one zero in one row is the shape this
/// project meets most often, so the costs are a second table indexed by the
/// same type.[^2]
///
/// The layout is 4 + 4 + 4 bytes for one commodity, which is 12 bytes at an
/// alignment of 4 and holds no padding. A test asserts the size.
///
/// # References
///
/// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decisions D1 and D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
/// [^2]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct BuildCostRow {
    /// The work an entry of this type needs before it finishes.
    pub work: u32,
    /// The residents a finished entry of this type takes.
    pub people: u32,
    /// The quantity of each good a finished entry of this type takes, in
    /// commodity order.
    pub goods: [Fix32; COMMODITY_COUNT],
}

impl BuildCostRow {
    /// The row that asks for nothing. A world built with it finishes an entry
    /// on the first advance and takes no resident and no good.
    pub const NONE: Self = Self {
        work: 0,
        people: 0,
        goods: [Fix32::ZERO; COMMODITY_COUNT],
    };

    /// The placeholder row that the default table holds for every type.
    pub const PLACEHOLDER: Self = Self {
        work: PLACEHOLDER_BUILD_WORK,
        people: PLACEHOLDER_BUILD_PEOPLE,
        goods: [PLACEHOLDER_BUILD_GOODS; COMMODITY_COUNT],
    };
}

/// The shared table that a unit type indexes for its build cost.
///
/// The table is data that the world is built with. It holds no code, and a
/// lookup in it is not a callback.[^1] It is dense and its length never
/// changes, in the way the upgrade table is.[^2]
///
/// # References
///
/// [^1]: ADR-0120, a unit carries a type that indexes a table, decision D1. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
/// [^2]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D1. `docs/adrs/draft/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuildCostTable {
    rows: [BuildCostRow; UNIT_TYPE_COUNT],
}

/// The default cost table that a world is built with.
///
/// **Every row holds the same placeholder.** A table that gave one type a
/// lower cost than another would state a balance judgement that nobody has
/// made, and the register marks every row unset.[^1]
///
/// # References
///
/// [^1]: Balance register, the production queue. `docs/reference/balance.md`
pub const DEFAULT_BUILD_COST_TABLE: BuildCostTable = BuildCostTable {
    rows: [BuildCostRow::PLACEHOLDER; UNIT_TYPE_COUNT],
};

impl Default for BuildCostTable {
    fn default() -> Self {
        DEFAULT_BUILD_COST_TABLE
    }
}

impl BuildCostTable {
    /// Builds a table in which every row asks for nothing.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            rows: [BuildCostRow::NONE; UNIT_TYPE_COUNT],
        }
    }

    /// Returns the row of one type.
    #[must_use]
    pub const fn row(&self, unit_type: UnitTypeId) -> BuildCostRow {
        self.rows[unit_type.index()]
    }

    /// Returns every row, in type order.
    #[must_use]
    pub const fn rows(&self) -> &[BuildCostRow; UNIT_TYPE_COUNT] {
        &self.rows
    }

    /// Writes one row of the table.
    ///
    /// # Errors
    ///
    /// Returns an error when the number names no row of the unit type table.
    pub const fn define(&mut self, unit_type: u8, row: BuildCostRow) -> Result<(), QueueError> {
        match UnitTypeId::from_u8(unit_type) {
            Some(unit_type) => {
                self.rows[unit_type.index()] = row;
                Ok(())
            }
            None => Err(QueueError::TypeAboveCeiling(unit_type)),
        }
    }

    /// Absorbs the table into the state hash.
    ///
    /// The table decides what a later frame does, so the whole-world hash
    /// covers it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        hash.write(bytemuck::cast_slice(&self.rows))
    }
}

/// Why the queue refused a caller.
///
/// Each variant is a mistake a caller can make. The verb returns the variant
/// and it never panics.[^1]
///
/// # References
///
/// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D6. `docs/adrs/draft/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueError {
    /// The number names no row of the unit type table.
    TypeAboveCeiling(u8),
    /// The identity names no site that stands.
    NoSuchSite(Entity),
    /// The site belongs to another faction.
    SiteBelongsToAnother {
        /// The faction that owns the site.
        owner: FactionId,
        /// The faction that asked.
        asked: FactionId,
    },
    /// The queue of the site already holds as many entries as it may.
    QueueFull {
        /// The bound the queue holds.
        bound: usize,
    },
    /// The position holds no entry.
    PositionEmpty(u8),
}

impl core::fmt::Display for QueueError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TypeAboveCeiling(value) => write!(
                formatter,
                "the unit type {value} is at or above the ceiling {UNIT_TYPE_COUNT}"
            ),
            Self::NoSuchSite(entity) => {
                write!(formatter, "the identity {} names no site", entity.to_bits())
            }
            Self::SiteBelongsToAnother { owner, asked } => write!(
                formatter,
                "the site belongs to the faction {} and the faction {} asked",
                owner.0, asked.0
            ),
            Self::QueueFull { bound } => {
                write!(formatter, "the queue already holds its bound of {bound}")
            }
            Self::PositionEmpty(position) => {
                write!(formatter, "the position {position} holds no entry")
            }
        }
    }
}

impl core::error::Error for QueueError {}

/// What one call of the queue verb asks for.
///
/// One verb takes both orders, so a caller, the built-in controller and a
/// learner reach the queue through one path and no verb exists for one of
/// them alone.[^1]
///
/// # References
///
/// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D2. `docs/adrs/draft/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueOrder {
    /// Put one entry of this type at the back of the queue.
    Push(UnitTypeId),
    /// Take the entry at this position out of the queue, and close the gap.
    ///
    /// **The work already charged is lost.** The store paid for it as the
    /// entry advanced, and nothing returns a charge.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D3. `docs/adrs/draft/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    Clear(u8),
}

/// The queue of every site, and what the last advance did.
///
/// The table holds one block of [`QUEUE_BOUND`] positions for each settlement
/// slot, in slot order. The entries of one block are packed at the front, so
/// the position of an entry is its place in the queue and the front entry is
/// position zero.[^1]
///
/// **Two sites never write one another's block**, so the disjointness of the
/// advance is a property of the partition and not a rule a reviewer must
/// check by reading.[^2]
///
/// # References
///
/// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D1. `docs/adrs/draft/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
/// [^2]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueueTable {
    /// One block of `QUEUE_BOUND` positions for each slot, in slot order.
    entries: Vec<QueueEntry>,
    /// The quantity of each good that one advance costs.
    charge: [Fix32; COMMODITY_COUNT],
    /// The entries one site may hold, which is a parameter of the world.
    ///
    /// **A bound of zero turns the queue off.** The verb then refuses every
    /// push, and no site ever holds an entry. A caller that wants the engine
    /// to leave its units alone sets it, in the way a plan bound of zero
    /// turns the plan off.
    bound: usize,
    /// The units the last advance produced.
    produced: u32,
    /// The finished entries the last advance refused, because the site held
    /// no resident to spend.
    refused_without_a_person: u32,
    /// The finished entries the last advance refused, because the store could
    /// not pay the goods.
    refused_without_goods: u32,
    /// The orders the verb refused since the last advance.
    refused_at_the_verb: u32,
}

impl Default for QueueTable {
    fn default() -> Self {
        Self::new()
    }
}

impl QueueTable {
    /// Builds a table that holds no block.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
            charge: [PLACEHOLDER_ADVANCE_CHARGE; COMMODITY_COUNT],
            bound: QUEUE_BOUND,
            produced: 0,
            refused_without_a_person: 0,
            refused_without_goods: 0,
            refused_at_the_verb: 0,
        }
    }

    /// Opens a block for every slot up to the given count.
    ///
    /// The table never shrinks, because the settlement arena never gives a
    /// slot index back.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn open_to(&mut self, slots: u32) {
        let wanted = slots as usize * QUEUE_BOUND;
        if self.entries.len() < wanted {
            self.entries.resize(wanted, QueueEntry::EMPTY);
        }
    }

    /// Returns how many slots the table holds a block for.
    #[must_use]
    pub fn slot_count(&self) -> u32 {
        (self.entries.len() / QUEUE_BOUND) as u32
    }

    /// Empties the block of one slot. A lost settlement takes its queue with
    /// it.
    pub fn clear_slot(&mut self, slot: u32) {
        if let Some(block) = self.block_mut(slot) {
            block.fill(QueueEntry::EMPTY);
        }
    }

    /// Returns the block of one slot, or an empty slice when the table holds
    /// no block for it.
    #[must_use]
    pub fn block(&self, slot: u32) -> &[QueueEntry] {
        let start = slot as usize * QUEUE_BOUND;
        self.entries
            .get(start..start + QUEUE_BOUND)
            .unwrap_or_default()
    }

    /// Returns the block of one slot for writing.
    fn block_mut(&mut self, slot: u32) -> Option<&mut [QueueEntry]> {
        let start = slot as usize * QUEUE_BOUND;
        self.entries.get_mut(start..start + QUEUE_BOUND)
    }

    /// Returns the entries of one slot, in queue position order.
    #[must_use]
    pub fn entries_of(&self, slot: u32) -> &[QueueEntry] {
        let block = self.block(slot);
        let filled = block.iter().take_while(|entry| entry.is_filled()).count();
        &block[..filled]
    }

    /// Returns how many entries the queue of one slot holds.
    #[must_use]
    pub fn len_of(&self, slot: u32) -> usize {
        self.entries_of(slot).len()
    }

    /// Returns the front entry of one slot, or `None` when the queue is
    /// empty.
    #[must_use]
    pub fn front(&self, slot: u32) -> Option<QueueEntry> {
        self.block(slot)
            .first()
            .copied()
            .filter(QueueEntry::is_filled)
    }

    /// Writes the front entry of one slot, and reports whether it wrote.
    pub fn set_front(&mut self, slot: u32, entry: QueueEntry) -> bool {
        match self.block_mut(slot).and_then(|block| block.first_mut()) {
            Some(front) if front.is_filled() => {
                *front = entry;
                true
            }
            _ => false,
        }
    }

    /// Puts one entry at the back of the queue of one slot.
    ///
    /// # Errors
    ///
    /// Returns an error when the queue already holds the bound.
    pub fn push(&mut self, slot: u32, unit_type: UnitTypeId) -> Result<(), QueueError> {
        let bound = self.bound;
        if self.len_of(slot) >= bound {
            return Err(QueueError::QueueFull { bound });
        }
        let Some(block) = self.block_mut(slot) else {
            return Err(QueueError::QueueFull { bound });
        };
        let Some(free) = block.iter_mut().find(|entry| !entry.is_filled()) else {
            return Err(QueueError::QueueFull { bound });
        };
        *free = QueueEntry::new(unit_type);
        Ok(())
    }

    /// Returns the entries one site may hold.
    #[must_use]
    pub const fn bound(&self) -> usize {
        self.bound
    }

    /// Sets the entries one site may hold, and reports whether it wrote.
    ///
    /// Returns `false` when the bound is above the width of the stored block.
    /// The call leaves every entry where it is, so a bound below the entries
    /// a site already holds stops a push and finishes what stands.
    pub const fn set_bound(&mut self, bound: usize) -> bool {
        if bound > QUEUE_BOUND {
            return false;
        }
        self.bound = bound;
        true
    }

    /// Takes the entry at one position out of the queue and closes the gap.
    ///
    /// The entries behind it keep their order, so the queue stays the order
    /// the entries were pushed in.
    ///
    /// # Errors
    ///
    /// Returns an error when the position holds no entry.
    pub fn remove(&mut self, slot: u32, position: u8) -> Result<QueueEntry, QueueError> {
        let index = position as usize;
        let Some(block) = self.block_mut(slot) else {
            return Err(QueueError::PositionEmpty(position));
        };
        if index >= QUEUE_BOUND || !block[index].is_filled() {
            return Err(QueueError::PositionEmpty(position));
        }
        let taken = block[index];
        for at in index..QUEUE_BOUND - 1 {
            block[at] = block[at + 1];
        }
        block[QUEUE_BOUND - 1] = QueueEntry::EMPTY;
        Ok(taken)
    }

    /// Returns the quantity of each good that one advance costs.
    #[must_use]
    pub const fn charge(&self) -> &[Fix32; COMMODITY_COUNT] {
        &self.charge
    }

    /// Sets the quantity of one good that one advance costs.
    ///
    /// Returns `false` when the commodity is outside the set.
    pub fn set_charge(&mut self, commodity: CommodityId, quantity: Fix32) -> bool {
        match self.charge.get_mut(commodity.0 as usize) {
            Some(slot) => {
                *slot = quantity;
                true
            }
            None => false,
        }
    }

    /// Empties the counts of the last advance and of the verb.
    pub const fn clear_counts(&mut self) {
        self.produced = 0;
        self.refused_without_a_person = 0;
        self.refused_without_goods = 0;
        self.refused_at_the_verb = 0;
    }

    /// Counts one unit the advance produced.
    pub const fn count_produced(&mut self) {
        self.produced = self.produced.saturating_add(1);
    }

    /// Counts one finished entry the advance refused for want of a resident.
    pub const fn count_without_a_person(&mut self) {
        self.refused_without_a_person = self.refused_without_a_person.saturating_add(1);
    }

    /// Counts one finished entry the advance refused for want of goods.
    pub const fn count_without_goods(&mut self) {
        self.refused_without_goods = self.refused_without_goods.saturating_add(1);
    }

    /// Counts one order the verb refused.
    pub const fn count_refused_at_the_verb(&mut self) {
        self.refused_at_the_verb = self.refused_at_the_verb.saturating_add(1);
    }

    /// Returns how many units the last advance produced.
    #[must_use]
    pub const fn produced(&self) -> u32 {
        self.produced
    }

    /// Returns how many finished entries the last advance refused, because
    /// the site held no resident to spend.
    #[must_use]
    pub const fn refused_without_a_person(&self) -> u32 {
        self.refused_without_a_person
    }

    /// Returns how many finished entries the last advance refused, because
    /// the store could not pay the goods.
    #[must_use]
    pub const fn refused_without_goods(&self) -> u32 {
        self.refused_without_goods
    }

    /// Returns how many orders the verb refused since the last advance.
    #[must_use]
    pub const fn refused_at_the_verb(&self) -> u32 {
        self.refused_at_the_verb
    }

    /// Absorbs the queues into the state hash.
    ///
    /// **The whole queue of every site enters the hash.** A queue the hash
    /// did not cover would let two different worlds hash the same and then
    /// diverge on the next tick.[^1]
    ///
    /// The counts do not enter. They are a census of one tick, and the next
    /// advance empties them, in the way the controller log does.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D1. `docs/adrs/draft/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        let mut hash = hash
            .write_u64(self.entries.len() as u64)
            .write(bytemuck::cast_slice(&self.entries));
        for quantity in &self.charge {
            hash = hash.write_u64(quantity.0 as u64);
        }
        hash.write_u64(self.bound as u64)
    }

    /// Reports whether the table holds its invariants.
    ///
    /// A block is packed at the front, every entry names a row of the unit
    /// type table, and every padding byte is zero. A block that held a gap
    /// would give one queue two orders, and nothing else would notice.
    #[must_use]
    pub fn check_invariants(&self, slots: u32) -> bool {
        if !self.entries.len().is_multiple_of(QUEUE_BOUND) {
            return false;
        }
        if self.slot_count() < slots {
            return false;
        }
        self.entries.chunks(QUEUE_BOUND).all(|block| {
            let filled = block.iter().take_while(|entry| entry.is_filled()).count();
            block.iter().skip(filled).all(|entry| !entry.is_filled())
                && block.iter().all(|entry| {
                    entry.padding == [0; 2]
                        && (!entry.is_filled() || UnitTypeId::from_u8(entry.unit_type.0).is_some())
                })
        })
    }
}
