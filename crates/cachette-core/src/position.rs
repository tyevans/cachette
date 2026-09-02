//! Ranked positions at a site.
//!
//! A site holds a small fixed number of positions. A position names a kind
//! of work and a rank, and it holds one unit or nobody. This module holds
//! the structure. It does not choose who fills a position: the rule that
//! assigns a unit to a position is separate work, and this module gives it
//! the thing it writes into.
//!
//! **A position that holds nobody is a state, not an absence.** The entry
//! exists, it names its kind and its rank, and its holder field is zero.
//!
//! **A position names a unit by its generational identity.** The identity
//! pairs a slot index with a generation, and the arena mints it.[^1] A
//! position that held a bare slot index would follow a dead unit into the
//! unit that the arena puts in that slot next, and nothing would fail. Every
//! read of a holder resolves the identity against the arena, which refuses a
//! generation that has moved on.[^2]
//!
//! **The storage is per site, and it is never per tile.** One site holds one
//! fixed-width row. Nothing here is sized to the tile count.
//!
//! **The number of positions of a site comes from the capacity of the ground
//! it stands on.** A site cannot hold more workers than can stand in it. The
//! capacity is a property of the terrain kind, and the terrain table is the
//! one place that states it.[^3] The width of a row is the largest capacity
//! any ground admits, folded from that same table, so raising a capacity
//! raises the row width and neither number is a copy of the other.[^4]
//!
//! **A kind of work is the resource kind that the work gathers.** The engine
//! already enumerates the resource kinds, and a second enumeration of the
//! same set would be one fact in two places.[^4]
//!
//! Every value here is an integer or a Q16.16 fixed-point value, and every
//! operation goes through the arithmetic module.[^5] [^6]
//!
//! # References
//!
//! [^1]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
//! [^2]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
//! [^3]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
//! [^4]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
//! [^5]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^6]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`

use bytemuck::{Pod, Zeroable};

use crate::hash::StateHash;
use crate::resource::{ResourceKind, RESOURCE_KIND_COUNT};
use crate::sim_math;
use crate::site::{CommodityId, Store, COMMODITY_COUNT};
use crate::soldier::SoldierArena;
use crate::terrain::{self, Terrain};
use crate::types::{Accum, Entity, Fix32, TileIdx};

/// The number of positions that one site holds.
///
/// The value is the largest number of units that any ground admits, folded
/// from the terrain capacity table.[^1] A site stands on one tile, and no
/// site holds more positions than the ground under it admits, so the widest
/// row a site can need is the widest capacity that exists.
///
/// The constant is derived and it is not a copy. A capacity that rises in
/// the terrain table raises this number without anybody sweeping the tree
/// for it.[^2]
///
/// # References
///
/// [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
/// [^2]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
pub const POSITIONS_PER_SITE: usize = terrain::largest_capacity() as usize;

/// The kind value of an entry that is no position.
///
/// A row is fixed width, and a site holds at most that many positions. An
/// entry beyond what the site holds is not a vacant position. It is not a
/// position at all, and this value says so.
///
/// The value sits at the top of the byte range, which no resource kind
/// reaches. It is a property of the column layout and not a budget.
pub const NO_WORK: u8 = u8::MAX;

/// The commodity that work of each kind fills.
///
/// The table maps a kind of work onto the commodity that a site holds more
/// of when that work is done. It is content. It is declared here until a
/// content pipeline exists, and the register holds the open choice of
/// it.[^1]
///
/// The engine reads this table. It never calls into content to get a
/// value.[^2]
///
/// # References
///
/// [^1]: Decisions register, DEC-073. `docs/DECISIONS.md`
/// [^2]: ADR-0007, content supplies a key vector, never a comparator, decision D3. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
pub const WORK_COMMODITY: [CommodityId; RESOURCE_KIND_COUNT] =
    [CommodityId(0), CommodityId(0), CommodityId(0)];

/// The reason that this module refused a caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PositionError {
    /// The caller asked for zero threads.
    ZeroThreads,
    /// The table holds no such site slot.
    NoSuchSlot(u32),
    /// The site holds no entry at that index.
    NoSuchIndex(usize),
    /// The entry at that index is no position, so nobody can hold it.
    NotAPosition(usize),
    /// The unit already holds another position at the same site.
    UnitHoldsAnother(usize),
    /// A preference target is below zero. A site wants nothing less than
    /// nothing.
    TargetBelowZero(Fix32),
    /// The columns that the pass reads hold different numbers of rows.
    ///
    /// The table states a site count that the settlement arena already
    /// holds. A check must fail when the two copies disagree.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    ColumnsDisagree,
}

impl core::fmt::Display for PositionError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ZeroThreads => write!(formatter, "a pass needs at least one thread"),
            Self::NoSuchSlot(slot) => write!(formatter, "the table holds no site slot {slot}"),
            Self::NoSuchIndex(index) => write!(formatter, "a site holds no entry at {index}"),
            Self::NotAPosition(index) => write!(formatter, "the entry at {index} is no position"),
            Self::UnitHoldsAnother(index) => write!(
                formatter,
                "the unit already holds the position at {index} of the same site"
            ),
            Self::TargetBelowZero(target) => {
                write!(formatter, "the target {} is below zero", target.0)
            }
            Self::ColumnsDisagree => write!(formatter, "the columns hold different lengths"),
        }
    }
}

impl std::error::Error for PositionError {}

/// One entry of the row of a site.
///
/// The entry is a position when its kind names a kind of work. It is no
/// position when its kind is the value that says so.
///
/// The layout is 8 + 1 + 1 + 6 bytes, which is 16 bytes at an alignment of
/// 8. The trailing array declares every padding byte.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct Position {
    /// The identity of the unit that holds the position, as its bits. Zero
    /// means nobody.
    ///
    /// The field is not a slot index. It carries the generation as well, so
    /// a holder that died does not resolve to the unit that took its
    /// slot.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    holder: u64,
    /// The kind of work, as the number of the resource kind it gathers, or
    /// the value that says the entry is no position.
    kind: u8,
    /// The rank of the position inside its kind. Zero is the first.
    rank: u8,
    /// The declared padding. Always zero.
    padding: [u8; 6],
}

impl Position {
    /// The entry that is no position.
    pub const EMPTY: Self = Self {
        holder: 0,
        kind: NO_WORK,
        rank: 0,
        padding: [0; 6],
    };

    /// Builds a vacant position of one kind and one rank.
    #[must_use]
    pub const fn vacant(kind: ResourceKind, rank: u8) -> Self {
        Self {
            holder: 0,
            kind: kind.to_u8(),
            rank,
            padding: [0; 6],
        }
    }

    /// Reports whether the entry is a position at all.
    #[must_use]
    pub const fn exists(self) -> bool {
        self.kind != NO_WORK
    }

    /// Returns the kind of work, or `None` when the entry is no position.
    #[must_use]
    pub const fn kind(self) -> Option<ResourceKind> {
        ResourceKind::from_u8(self.kind)
    }

    /// Returns the kind of work as the number that a caller outside the
    /// crate reads.
    #[must_use]
    pub const fn kind_number(self) -> u8 {
        self.kind
    }

    /// Returns the rank of the position inside its kind.
    #[must_use]
    pub const fn rank(self) -> u8 {
        self.rank
    }

    /// Returns the identity that the position names, as its bits.
    ///
    /// Zero means nobody. A value above zero names a unit that may already
    /// have died, so resolve it against the arena before reading anything
    /// of it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    #[must_use]
    pub const fn holder_bits(self) -> u64 {
        self.holder
    }

    /// Returns the identity that the position names.
    ///
    /// The answer says who the position was given to. It does not say that
    /// the unit still lives. Ask the arena for that.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    #[must_use]
    pub(crate) const fn holder(self) -> Option<Entity> {
        Entity::from_bits(self.holder)
    }
}

/// What one site wants of each kind of work.
///
/// The target is what the site means to hold of the commodity that the kind
/// fills. What the site lacks is the target less what it holds, and that
/// shortfall is what decides how many positions of the kind the site opens.
///
/// The layout is three Q16.16 values, which is 12 bytes at an alignment of
/// 4. There is no padding to declare.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct SitePreference {
    target: [Fix32; RESOURCE_KIND_COUNT],
}

impl SitePreference {
    /// The preference that wants nothing. A site that wants nothing opens no
    /// position.
    pub const NONE: Self = Self {
        target: [Fix32::ZERO; RESOURCE_KIND_COUNT],
    };

    /// The preference that a site starts with.
    ///
    /// The values are content. They are declared here until content exists,
    /// and the register holds the open choice of them.[^1] A caller replaces
    /// them without touching a kernel.
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-073. `docs/DECISIONS.md`
    pub const DEFAULT: Self = Self {
        target: [Fix32::ONE; RESOURCE_KIND_COUNT],
    };

    /// Returns what the site wants of one kind.
    #[must_use]
    pub const fn target(self, kind: ResourceKind) -> Fix32 {
        self.target[kind.index()]
    }

    /// Writes what the site wants of one kind.
    ///
    /// # Errors
    ///
    /// Returns an error when the target is below zero.
    pub fn set_target(&mut self, kind: ResourceKind, target: Fix32) -> Result<(), PositionError> {
        if target.0 < 0 {
            return Err(PositionError::TargetBelowZero(target));
        }
        self.target[kind.index()] = target;
        Ok(())
    }
}

/// The positions of every site, and what every site wants.
///
/// The table is indexed by the slot of the site, so a span of sites is a
/// contiguous span of rows and a pass over it needs no sort.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PositionTable {
    /// One fixed-width row for each site slot, in slot order.
    rows: Vec<Position>,
    /// What each site slot wants, in slot order.
    preferences: Vec<SitePreference>,
}

impl Default for PositionTable {
    fn default() -> Self {
        Self::new()
    }
}

impl PositionTable {
    /// Builds an empty table.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            rows: Vec::new(),
            preferences: Vec::new(),
        }
    }

    /// Returns the number of site slots that the table holds.
    #[must_use]
    pub fn slot_count(&self) -> u32 {
        self.preferences.len() as u32
    }

    /// Reports whether the table holds no slot.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.preferences.is_empty()
    }

    /// Opens rows until the table holds this many site slots.
    ///
    /// A new slot holds no position and the preference that a site starts
    /// with. The table never shrinks, because the slot index space never
    /// shrinks.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn open_to(&mut self, slots: u32) {
        if (slots as usize) > self.preferences.len() {
            self.preferences
                .resize(slots as usize, SitePreference::DEFAULT);
            self.rows
                .resize(slots as usize * POSITIONS_PER_SITE, Position::EMPTY);
        }
    }

    /// Clears the positions and the preference of one slot.
    ///
    /// The world does this when a settlement is lost. A position that
    /// outlived the site that opened it would offer work at a place that no
    /// longer stands, and the settlement founded next in that slot would
    /// inherit a staff it never hired.
    pub fn clear_slot(&mut self, slot: u32) {
        if let Some(preference) = self.preferences.get_mut(slot as usize) {
            *preference = SitePreference::DEFAULT;
        }
        if let Some(row) = self.row_mut(slot) {
            row.fill(Position::EMPTY);
        }
    }

    /// Returns the whole row of one site slot.
    #[must_use]
    pub fn row(&self, slot: u32) -> Option<&[Position]> {
        let start = (slot as usize).checked_mul(POSITIONS_PER_SITE)?;
        self.rows.get(start..start + POSITIONS_PER_SITE)
    }

    /// Returns the whole row of one site slot, for writing.
    fn row_mut(&mut self, slot: u32) -> Option<&mut [Position]> {
        let start = (slot as usize).checked_mul(POSITIONS_PER_SITE)?;
        self.rows.get_mut(start..start + POSITIONS_PER_SITE)
    }

    /// Returns the whole table, in slot order.
    #[must_use]
    pub fn rows(&self) -> &[Position] {
        &self.rows
    }

    /// Returns the number of positions that one site holds.
    ///
    /// The answer counts the entries of the row that are positions. Nothing
    /// stores the count, so no stored count can disagree with the row.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub fn count(&self, slot: u32) -> usize {
        self.row(slot)
            .map_or(0, |row| row.iter().filter(|entry| entry.exists()).count())
    }

    /// Returns the number of positions of one kind that one site holds.
    #[must_use]
    pub fn count_of_kind(&self, slot: u32, kind: ResourceKind) -> usize {
        self.row(slot).map_or(0, |row| {
            row.iter()
                .filter(|entry| entry.kind_number() == kind.to_u8())
                .count()
        })
    }

    /// Returns what one site wants.
    #[must_use]
    pub fn preference(&self, slot: u32) -> Option<SitePreference> {
        self.preferences.get(slot as usize).copied()
    }

    /// Writes what one site wants of one kind.
    ///
    /// # Errors
    ///
    /// Returns an error when the slot is outside the table, or when the
    /// target is below zero.
    pub fn set_target(
        &mut self,
        slot: u32,
        kind: ResourceKind,
        target: Fix32,
    ) -> Result<(), PositionError> {
        let preference = self
            .preferences
            .get_mut(slot as usize)
            .ok_or(PositionError::NoSuchSlot(slot))?;
        preference.set_target(kind, target)
    }

    /// Gives one position of one site to one unit.
    ///
    /// This is the setter that an assignment rule writes through. It states
    /// no rule of its own: it does not choose the unit, and it does not
    /// choose the position.
    ///
    /// The caller passes an identity that the arena minted, and the table
    /// stores the whole identity.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the slot is outside the table, when the index
    /// is outside the row, when the entry at that index is no position, and
    /// when the unit already holds another position at the same site.
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn seat(&mut self, slot: u32, index: usize, unit: Entity) -> Result<(), PositionError> {
        if index >= POSITIONS_PER_SITE {
            return Err(PositionError::NoSuchIndex(index));
        }
        let row = self.row_mut(slot).ok_or(PositionError::NoSuchSlot(slot))?;
        if !row[index].exists() {
            return Err(PositionError::NotAPosition(index));
        }
        // One unit holds at most one position at one site. A unit in two
        // jobs at one place is two claims on one worker, and the row is
        // short enough that the check costs nothing.
        for (other, entry) in row.iter().enumerate() {
            if other != index && entry.holder_bits() == unit.to_bits() {
                return Err(PositionError::UnitHoldsAnother(other));
            }
        }
        row[index].holder = unit.to_bits();
        Ok(())
    }

    /// Takes the holder out of one position of one site.
    ///
    /// # Errors
    ///
    /// Returns an error when the slot is outside the table, or when the
    /// index is outside the row.
    pub fn vacate(&mut self, slot: u32, index: usize) -> Result<(), PositionError> {
        if index >= POSITIONS_PER_SITE {
            return Err(PositionError::NoSuchIndex(index));
        }
        let row = self.row_mut(slot).ok_or(PositionError::NoSuchSlot(slot))?;
        row[index].holder = 0;
        Ok(())
    }

    /// Returns the unit that holds one position, when that unit still
    /// lives.
    ///
    /// The call resolves the stored identity against the arena. A holder
    /// that died gives `None`, and the unit that took its slot is never the
    /// answer.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    #[must_use]
    pub fn occupant(&self, slot: u32, index: usize, units: &SoldierArena) -> Option<Entity> {
        let entity = self.row(slot)?.get(index)?.holder()?;
        units.contains(entity).then_some(entity)
    }

    /// Absorbs the table into the state hash.
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        hash.write_u64(self.preferences.len() as u64)
            .write(bytemuck::cast_slice(&self.rows))
            .write(bytemuck::cast_slice(&self.preferences))
    }

    /// Reports whether the table holds its own invariants.
    ///
    /// The row width and the slot count are two statements of the same
    /// length, so the check compares them.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub fn check_invariants(&self) -> bool {
        if self.rows.len() != self.preferences.len() * POSITIONS_PER_SITE {
            return false;
        }
        if self.preferences.iter().any(|preference| {
            ResourceKind::ALL
                .iter()
                .any(|kind| preference.target(*kind).0 < 0)
        }) {
            return false;
        }
        for slot in 0..self.slot_count() {
            let Some(row) = self.row(slot) else {
                return false;
            };
            // The positions of a site sit at the front of the row, and an
            // entry that is no position holds nothing. A holder past the
            // end of what the site holds would be a unit assigned to a job
            // the site does not have.
            let mut ended = false;
            for entry in row {
                if entry.padding != [0; 6] {
                    return false;
                }
                if entry.exists() {
                    if ended {
                        return false;
                    }
                    if entry.kind().is_none() {
                        return false;
                    }
                    if (entry.rank() as usize) >= POSITIONS_PER_SITE {
                        return false;
                    }
                } else {
                    ended = true;
                    if entry.holder_bits() != 0 || entry.rank() != 0 {
                        return false;
                    }
                }
            }
            // A kind holds ranks zero upward with nothing missing and
            // nothing repeated, so the row states one order and not two.
            for kind in ResourceKind::ALL {
                let count = self.count_of_kind(slot, kind);
                for rank in 0..count {
                    if !row
                        .iter()
                        .any(|e| e.kind_number() == kind.to_u8() && e.rank() as usize == rank)
                    {
                        return false;
                    }
                }
            }
            // One unit holds at most one position at one site.
            for (index, entry) in row.iter().enumerate() {
                if entry.holder_bits() == 0 {
                    continue;
                }
                if row
                    .iter()
                    .skip(index + 1)
                    .any(|other| other.holder_bits() == entry.holder_bits())
                {
                    return false;
                }
            }
        }
        true
    }

    /// Reports whether every position names a unit that still exists.
    ///
    /// A position that names a unit the arena no longer holds is a stale
    /// identity in stored state. It is the defect that the generation
    /// exists to catch, and this is the check that fails on it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    #[must_use]
    pub fn check_holders(&self, units: &SoldierArena) -> bool {
        self.rows.iter().all(|entry| match entry.holder() {
            None => entry.holder_bits() == 0,
            Some(unit) => units.contains(unit),
        })
    }

    /// Reports whether no site holds more positions than its ground admits.
    ///
    /// The two bounds come from one table. This check is what fails when a
    /// pass writes a row wider than the ground under the site.[^1] [^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the columns disagree on how many sites there
    /// are.
    ///
    /// # References
    ///
    /// [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
    /// [^2]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    pub fn check_capacity(
        &self,
        tiles: &[TileIdx],
        live: &[u8],
        terrain: Terrain,
    ) -> Result<bool, PositionError> {
        if tiles.len() != self.preferences.len() || live.len() != self.preferences.len() {
            return Err(PositionError::ColumnsDisagree);
        }
        for slot in 0..self.slot_count() {
            if live[slot as usize] != 1 {
                continue;
            }
            let capacity = capacity_at(terrain, tiles[slot as usize]);
            if self.count(slot) > capacity {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

/// Returns the number of units that the ground under one tile admits.
///
/// The answer comes from the terrain capacity table and from nowhere
/// else.[^1] A tile outside the world admits nobody.
///
/// # References
///
/// [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
fn capacity_at(terrain: Terrain, tile: TileIdx) -> usize {
    let capacity = terrain
        .tile_at(tile)
        .map_or(0, |ground| ground.kind.capacity()) as usize;
    // The row is as wide as the widest capacity the table states, so this
    // takes no effect today. It is the guard that keeps the two bounds in
    // one direction if a capacity ever rises above the width.
    capacity.min(POSITIONS_PER_SITE)
}

/// Releases every position whose holder no longer exists.
///
/// The pass runs on every frame, and not on the interval that the rebalance
/// runs on. A unit dies inside a frame, and a position that named it would
/// hold a stale identity until the next rebalance. The invariant check
/// refuses that state, so the release must be as frequent as the deaths.
///
/// Each thread takes a contiguous span of site rows and writes only inside
/// it, so the threads share nothing and the result does not depend on which
/// one finished first.[^1]
///
/// # Errors
///
/// Returns an error when the caller asks for zero threads.
///
/// # References
///
/// [^1]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
pub fn release_the_dead(
    table: &mut PositionTable,
    units: &SoldierArena,
    threads: usize,
) -> Result<(), PositionError> {
    if threads == 0 {
        return Err(PositionError::ZeroThreads);
    }
    let sites = table.preferences.len();
    if sites == 0 {
        return Ok(());
    }
    let chunk_len = sites.div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        for span in table.rows.chunks_mut(chunk_len * POSITIONS_PER_SITE) {
            scope.spawn(move || {
                for entry in span {
                    let Some(held) = entry.holder() else {
                        continue;
                    };
                    if !units.contains(held) {
                        entry.holder = 0;
                    }
                }
            });
        }
    });
    Ok(())
}

/// Sets the number of positions of each kind that every site holds.
///
/// A site opens as many positions as the ground under it admits, and it
/// splits them between the kinds of work in proportion to what it lacks of
/// each. What it lacks is what it wants less what it holds. A site that
/// lacks nothing opens no position.
///
/// The split is exact. Each kind takes the truncated proportion, and the
/// remainder goes one position at a time to the kinds in ascending kind
/// order, so the parts sum to the whole and the tie needs no draw.[^1]
///
/// **A unit keeps its position when the position survives the rebalance.**
/// The new row is built in ascending kind order and ascending rank, and
/// each new position takes the holder of the old position of the same kind
/// and the same rank. A position that the rebalance removed releases its
/// holder.
///
/// Each thread takes a contiguous span of site rows and writes only inside
/// it.[^2]
///
/// # Errors
///
/// Returns an error when the caller asks for zero threads, and when the
/// columns disagree on how many sites there are.
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decisions D1 and D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
/// [^2]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
pub fn rebalance(
    table: &mut PositionTable,
    live: &[u8],
    tiles: &[TileIdx],
    stores: &[Store],
    terrain: Terrain,
    threads: usize,
) -> Result<(), PositionError> {
    if threads == 0 {
        return Err(PositionError::ZeroThreads);
    }
    let sites = table.preferences.len();
    if live.len() != sites || tiles.len() != sites || stores.len() != sites {
        return Err(PositionError::ColumnsDisagree);
    }
    if sites == 0 {
        return Ok(());
    }
    let chunk_len = sites.div_ceil(threads).max(1);
    let preferences = &table.preferences;
    std::thread::scope(|scope| {
        let mut base = 0usize;
        for span in table.rows.chunks_mut(chunk_len * POSITIONS_PER_SITE) {
            let start = base;
            base += span.len() / POSITIONS_PER_SITE;
            let live_span = &live[start..base];
            let tile_span = &tiles[start..base];
            let store_span = &stores[start..base];
            let preference_span = &preferences[start..base];
            scope.spawn(move || {
                rebalance_span(
                    span,
                    live_span,
                    tile_span,
                    store_span,
                    preference_span,
                    terrain,
                );
            });
        }
    });
    Ok(())
}

/// Rebalances one contiguous span of sites.
///
/// The span belongs to one thread. The function reads and writes nothing
/// outside it.
fn rebalance_span(
    rows: &mut [Position],
    live: &[u8],
    tiles: &[TileIdx],
    stores: &[Store],
    preferences: &[SitePreference],
    terrain: Terrain,
) {
    for offset in 0..live.len() {
        if live[offset] != 1 {
            continue;
        }
        let row = &mut rows[offset * POSITIONS_PER_SITE..(offset + 1) * POSITIONS_PER_SITE];
        let counts = shares(
            capacity_at(terrain, tiles[offset]),
            preferences[offset],
            stores[offset],
        );
        let mut next = [Position::EMPTY; POSITIONS_PER_SITE];
        let mut index = 0;
        for kind in ResourceKind::ALL {
            for rank in 0..counts[kind.index()] {
                let mut entry = Position::vacant(kind, rank as u8);
                // The holder follows its own position across the
                // rebalance. A position that survives keeps the unit that
                // held it, and a position that goes releases it.
                if let Some(old) = row
                    .iter()
                    .find(|e| e.kind_number() == kind.to_u8() && e.rank() as usize == rank)
                {
                    entry.holder = old.holder;
                }
                next[index] = entry;
                index += 1;
            }
        }
        row.copy_from_slice(&next);
    }
}

/// Returns how many positions of each kind a site opens.
///
/// The parts sum to the capacity when the site lacks anything, and to zero
/// when it lacks nothing.
fn shares(
    capacity: usize,
    preference: SitePreference,
    store: Store,
) -> [usize; RESOURCE_KIND_COUNT] {
    let mut counts = [0usize; RESOURCE_KIND_COUNT];
    if capacity == 0 {
        return counts;
    }
    let mut lacked = [Accum(0); RESOURCE_KIND_COUNT];
    let mut whole = Accum(0);
    for kind in ResourceKind::ALL {
        let held = store
            .quantity(WORK_COMMODITY[kind.index()])
            .unwrap_or(Fix32::ZERO);
        let short = sim_math::sub(preference.target(kind), held);
        let value = if short.0 > 0 {
            short.to_accum()
        } else {
            Accum(0)
        };
        lacked[kind.index()] = value;
        whole = sim_math::combine(whole, value);
    }
    if whole.0 == 0 {
        return counts;
    }
    let total = Accum(capacity as i64);
    let mut handed: i64 = 0;
    for kind in ResourceKind::ALL {
        let part = sim_math::share(total, lacked[kind.index()], whole)
            .expect("what the site lacks in total is above zero");
        counts[kind.index()] = part.0 as usize;
        handed = handed.saturating_add(part.0);
    }
    // The remainder goes one position at a time, in ascending kind order,
    // to the kinds that lack anything. The order is the kind numbering, so
    // it does not depend on the thread that ran the site.
    let mut remainder = capacity as i64 - handed;
    while remainder > 0 {
        let mut given = false;
        for kind in ResourceKind::ALL {
            if remainder == 0 {
                break;
            }
            if lacked[kind.index()].0 > 0 {
                counts[kind.index()] += 1;
                remainder -= 1;
                given = true;
            }
        }
        if !given {
            break;
        }
    }
    counts
}

// The work-to-commodity table cannot name a commodity that no store holds,
// and it holds one entry for each kind of work. Both are checked when the
// crate compiles, so neither can drift as the other changes.
const _: () = assert!(WORK_COMMODITY.len() == RESOURCE_KIND_COUNT);
const _: () = {
    let mut index = 0;
    while index < RESOURCE_KIND_COUNT {
        assert!((WORK_COMMODITY[index].0 as usize) < COMMODITY_COUNT);
        index += 1;
    }
};
