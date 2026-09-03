//! The soldier column set.
//!
//! The entity storage holds four fixed shapes, and each shape gets its own
//! set of columns.[^1] The soldier is the mobile shape. A soldier carries a
//! generational identity, a tile address, and a faction.
//!
//! The shapes do not vary at run time. A shape that is not one of the four
//! is a compile-time error here, because a column set is a Rust type and
//! not a row in a table.[^2]
//!
//! Every entity lives in the generational arena, and its identity pairs a
//! slot index with a generation.[^3] The arena mints every identity. No
//! caller builds one from parts.[^4]
//!
//! Every column holds an exact integer. No column holds a floating point
//! number.[^5]
//!
//! # References
//!
//! [^1]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
//! [^2]: ADR-0066, entity storage holds four fixed shapes, decision D3. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
//! [^3]: ADR-0012, tiles are dense columns and units are a generational arena, decision D3. `docs/adrs/accepted/adr-0012-tiles-are-dense-columns-and-units-are-a-generational-arena.md`
//! [^4]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
//! [^5]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`

use std::collections::VecDeque;

use crate::choose::NO_INTENT;
use crate::cohort::NEED_FULL;
use crate::hash::StateHash;
use crate::hex::{Axial, Grid};
use crate::resource::{Amount, CarryLoad, ResourceKind};
use crate::types::{Entity, FactionId, Fix32, TileIdx, FACTION_CEILING};
use crate::upgrade::UpgradeKind;

/// The generation that means a slot carries no identity.
///
/// A generation starts at one, so no handle ever holds this value.[^1] The
/// arena writes it into a slot that it has never used, and into a slot that
/// it has retired.
///
/// # References
///
/// [^1]: ADR-0014, entity identity is an index plus a generation, decision D6. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
/// Hands out one identifier for each arena that the process builds.
///
/// A derived structure must know which arena it was built from, and not only
/// how many changes that arena has taken. Two arenas of one extent, each
/// holding one soldier on a different tile, both sit at revision one: a
/// counter alone lets a bridge built from the first answer questions about
/// the second, and every check passes.[^1]
///
/// The counter is process-wide and never enters simulated state, so it
/// reaches no state hash and no event log.
///
/// # References
///
/// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
fn next_arena_identity() -> u64 {
    use core::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

const NO_GENERATION: u32 = 0;

/// The first generation of a slot.
///
/// The record starts a generation at one, never at zero. Slot zero at
/// generation zero packs to the value zero, which the identity cannot
/// hold, and slot zero is the first slot the arena ever allocates.[^1]
///
/// # References
///
/// [^1]: ADR-0014, entity identity is an index plus a generation, decision D6. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
const FIRST_GENERATION: u32 = 1;

/// The gather order that means the soldier gathers nothing.
///
/// Any other value is the resource kind number plus one. One value therefore
/// says both whether the soldier gathers and what it gathers, so no second
/// column can disagree with this one.[^1]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
const NO_ORDER: u8 = 0;

/// The home value that means a unit belongs to no site.
///
/// The value is the top of the slot index range, which no arena reaches,
/// because a slot index below it always exists first. It is a property of
/// the index layout and not a budget.
pub const NO_HOME: u32 = u32::MAX;

/// The columns that a pass over the needs reads and writes.
///
/// The need column and the deficit column are mutable and the other two are
/// not. A pass changes what a unit needs. It never changes which units
/// exist, because that is a structural change and it belongs to the
/// arena.[^1]
///
/// # References
///
/// [^1]: ADR-0066, entity storage holds four fixed shapes, decision D2. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
#[derive(Debug)]
pub struct NeedUpdate<'a> {
    /// The need of each slot.
    pub needs: &'a mut [Fix32],
    /// The deficit accumulator of each slot.
    pub deficits: &'a mut [Fix32],
    /// One for a live slot, zero otherwise.
    pub live: &'a [u8],
    /// The site that each slot draws from, or `NO_HOME`.
    pub homes: &'a [u32],
    /// The faction of each slot.
    pub factions: &'a [FactionId],
}

/// Returns the gather order that a column value names.
const fn order_of(value: u8) -> Option<ResourceKind> {
    if value == NO_ORDER {
        return None;
    }
    ResourceKind::from_u8(value - 1)
}

/// The build order value that means a soldier builds nothing.
///
/// The column holds the kind number plus one, in the same way the gather
/// order column does, so one value says both whether the soldier builds and
/// what it builds.[^1]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
const NO_BUILD: u8 = 0;

/// The deeds at which a unit becomes eligible for promotion, before a caller
/// says otherwise.
///
/// **This is a fixture-facing parameter and not a budget.** It says how much
/// a unit must have gathered before the world will consider making a person
/// of it, and a caller changes it without touching the engine. No record
/// holds the value, because no record may hold a number a measurement or a
/// content choice can move.[^1]
///
/// # References
///
/// [^1]: Decision Record Scope, section 4.1. `.claude/rules/adr-scope.md`
pub const DEFAULT_DEED_THRESHOLD: u64 = 24;

/// Returns the build order that a column value names.
const fn build_of(value: u8) -> Option<UpgradeKind> {
    if value == NO_BUILD {
        return None;
    }
    UpgradeKind::from_u8(value - 1)
}

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

/// The reason that the arena refused a caller.
///
/// Each variant is a mistake that a caller can make. The arena returns the
/// variant. It never panics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoldierError {
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
    /// The ground of the tile admits no unit.[^1]
    ///
    /// The address is inside the world. The refusal comes from the kind of
    /// the ground, not from the extent, and the two never share a variant.
    ///
    /// # References
    ///
    /// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D4. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
    TileImpassable(Axial),
}

impl core::fmt::Display for SoldierError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ArenaFull => write!(formatter, "the soldier arena holds no free slot"),
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
            Self::TileImpassable(address) => write!(
                formatter,
                "the ground at ({}, {}) admits no unit",
                address.q, address.r
            ),
        }
    }
}

impl std::error::Error for SoldierError {}

/// The column set of the soldier shape.
///
/// The arena holds one entry for each slot it has ever opened, and it never
/// compacts the slot index space. Compaction would move a soldier to another
/// slot and invalidate every identity that names it.[^1]
///
/// The location column is a dense array indexed by the slot. The arena never
/// looks a soldier up in a hash map, because a hash map costs a hash on the
/// hot path and carries an iteration order that no key fixes.[^2]
///
/// # References
///
/// [^1]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
/// [^2]: ADR-0014, entity identity is an index plus a generation, decision D7. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
#[derive(Debug)]
pub struct SoldierArena {
    /// The shape of the world that holds the soldiers.
    grid: Grid,
    /// The largest number of slots that the arena opens.
    capacity: u32,
    /// The generation of each slot. Zero means the slot carries no identity.
    generations: Vec<u32>,
    /// One for a live slot, zero for a free slot or a retired slot.
    live: Vec<u8>,
    /// The tile of each slot. This is the location table.
    tiles: Vec<TileIdx>,
    /// The faction of each slot.
    factions: Vec<FactionId>,
    /// What the soldier in each slot carries.
    ///
    /// The load is part of the soldier shape, so it is a column of this arena
    /// and not a side table keyed on the identity.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D4. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
    carries: Vec<CarryLoad>,
    /// The gather order of each slot.
    ///
    /// The value zero means the soldier gathers nothing. Any other value is
    /// the kind number plus one. The column holds a small integer and not an
    /// option, because it is plain data that reaches the state hash.
    orders: Vec<u8>,
    /// The build order of each slot.
    ///
    /// The value zero means the soldier builds nothing. Any other value is
    /// the upgrade kind number plus one. The column holds a small integer and
    /// not an option, because it is plain data that reaches the state hash.
    ///
    /// The order is a column of this arena, because a unit carries what it
    /// was told to do. What it builds is a property of the tile, and that
    /// lives in the sparse upgrade map.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    builds: Vec<u8>,
    /// The need of each slot.
    ///
    /// A need runs from zero to full. It falls at an interval by a
    /// saturating subtract, and a draw against the store of a site raises
    /// it again.[^1]
    ///
    /// The need is a column of this arena, because a unit carries its own
    /// need. The draw that fills it is pooled, and that is a property of the
    /// draw and not of the need.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decisions D1 and D2. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
    needs: Vec<Fix32>,
    /// The deficit accumulator of each slot.
    ///
    /// The accumulator rises while the need is below the threshold and
    /// falls while it is at or above it. It is the input that a later rule
    /// reads to end a unit.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D4. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
    deficits: Vec<Fix32>,
    /// The site that each slot draws from, or `NO_HOME`.
    ///
    /// The column holds a slot of the settlement arena and not an identity,
    /// because the cohort array is indexed by that slot. A unit that names
    /// no site draws from nothing.
    homes: Vec<u32>,
    /// The option that each slot last chose, or `NO_INTENT`.
    ///
    /// The column holds the choice and never the score. A score is
    /// transient: the choice pass compares it and discards it, so no score
    /// reaches this arena and no score reaches the state hash.[^1]
    ///
    /// The intent is sticky. A unit keeps it between two choices, because a
    /// unit that re-decides on every tick oscillates between two options of
    /// nearly equal score and arrives nowhere.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decisions D1 and D2. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
    intents: Vec<u8>,
    /// What each slot has ever gathered, summed over every kind.
    ///
    /// **The value never falls.** It is a running total of what a unit took
    /// from the ground, and nothing subtracts from it. A carried load leaves
    /// a unit when the unit dies or delivers, and this column does not
    /// follow the load. It records the deed, not the goods.
    ///
    /// The eligibility scan reads a level and not an edge, so the scan is
    /// correct only while the value rises. A rule that lowered it would break
    /// the scan in silence, and the invariant check states that.[^1]
    ///
    /// The column is an integer count of an integer amount. No part of it is
    /// fixed-point, so no part of it rounds.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0104, a soldier is promoted from a level that never falls, decision D2. `docs/adrs/draft/adr-0104-a-soldier-is-promoted-from-a-level-that-never-falls.md`
    /// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    deeds: Vec<u64>,
    /// One for a slot whose deeds reach the threshold, zero otherwise.
    ///
    /// **This is a second statement of `deeds >= deed_threshold`.** It exists
    /// so that the promotion scan reads one byte for each unit rather than
    /// eight, and the arena writes it in the same call that raises the deeds.
    /// The invariant check fails when the two disagree, because a value in
    /// two places drifts and nothing else would notice.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    eligible: Vec<u8>,
    /// The character that each slot was promoted into, as identity bits, or
    /// zero for a slot that holds no character.
    ///
    /// **The link runs one way, from the unit to the character.** A character
    /// holds no unit column. One direction cannot disagree with the other,
    /// and a character outlives the unit that carried it, so a link from the
    /// character would name a dead unit for the rest of that character's
    /// life.[^1]
    ///
    /// The value is a whole identity and never a bare slot index, so a
    /// character that was removed does not resolve to the character created
    /// next in its slot.[^2]
    ///
    /// A slot with a character above zero is already promoted, so this column
    /// is also what stops a unit being promoted twice. That is one value
    /// answering one question, and not a second flag.
    ///
    /// # References
    ///
    /// [^1]: ADR-0104, a soldier is promoted from a level that never falls, decision D3. `docs/adrs/draft/adr-0104-a-soldier-is-promoted-from-a-level-that-never-falls.md`
    /// [^2]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    characters: Vec<u64>,
    /// The deeds at which a slot becomes eligible for promotion.
    ///
    /// **This is the one storage site of the threshold.** The world exposes a
    /// setter that forwards here and keeps no copy, because the arena is what
    /// maintains the eligibility column and a second copy would let the
    /// column disagree with the rule that wrote it.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    deed_threshold: u64,
    /// The free slots, oldest first.
    free: VecDeque<u32>,
    /// The number of live soldiers.
    live_count: u32,
    /// The number of retired slots.
    retired_count: u32,
    /// The number of structural changes that the arena has taken.
    ///
    /// A spawn, a despawn and a move each raise it by one. A derived
    /// structure records the value it was built from, and it refuses a read
    /// when the value has moved on. One fact in two places needs a check
    /// that fails when the copies disagree.[^1]
    ///
    /// The counter is bookkeeping. It decides no later frame, so it does not
    /// reach the state hash.
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, section 1. `.claude/rules/recurring-defects.md`
    revision: u64,
    identity: u64,
}

/// Copies a column and keeps the reservation of the original.
///
/// A derived copy of a column allocates for what the column holds, not for
/// what it reserved. A copied arena would therefore grow where the original
/// does not, and nothing would report it.[^1]
///
/// # References
///
/// [^1]: ADR-0084, the world reserves the unit columns at construction, decision D1. `docs/adrs/draft/adr-0084-the-world-reserves-the-unit-columns-at-construction.md`
fn copy_column<T: Clone>(column: &[T], capacity: u32) -> Vec<T> {
    let mut copy = Vec::with_capacity(capacity as usize);
    copy.extend_from_slice(column);
    copy
}

impl Clone for SoldierArena {
    fn clone(&self) -> Self {
        let capacity = self.capacity;
        let mut free = VecDeque::with_capacity(capacity as usize);
        free.extend(self.free.iter().copied());
        Self {
            grid: self.grid,
            capacity,
            generations: copy_column(&self.generations, capacity),
            live: copy_column(&self.live, capacity),
            tiles: copy_column(&self.tiles, capacity),
            factions: copy_column(&self.factions, capacity),
            carries: copy_column(&self.carries, capacity),
            orders: copy_column(&self.orders, capacity),
            builds: copy_column(&self.builds, capacity),
            needs: copy_column(&self.needs, capacity),
            deficits: copy_column(&self.deficits, capacity),
            homes: copy_column(&self.homes, capacity),
            intents: copy_column(&self.intents, capacity),
            deeds: copy_column(&self.deeds, capacity),
            eligible: copy_column(&self.eligible, capacity),
            characters: copy_column(&self.characters, capacity),
            deed_threshold: self.deed_threshold,
            free,
            live_count: self.live_count,
            retired_count: self.retired_count,
            revision: self.revision,
            identity: self.identity,
        }
    }
}

impl SoldierArena {
    /// Builds an arena that reserves `capacity` slots and opens no more.
    ///
    /// The arena reserves every column here, and it reserves the free queue,
    /// so no later spawn and no later death grows either one.[^1] A caller
    /// that asks for a soldier past the reservation gets a typed refusal.
    ///
    /// This is the only constructor, and it names no capacity of its own.
    /// The settings of the world state the reservation, in one place, so no
    /// default here can disagree with them.[^2]
    ///
    /// The reservation is paid once, when the caller builds the arena. No
    /// later call pays it.
    ///
    /// # References
    ///
    /// [^1]: ADR-0084, the world reserves the unit columns at construction, decision D1. `docs/adrs/draft/adr-0084-the-world-reserves-the-unit-columns-at-construction.md`
    /// [^2]: ADR-0084, the world reserves the unit columns at construction, decision D2. `docs/adrs/draft/adr-0084-the-world-reserves-the-unit-columns-at-construction.md`
    #[must_use]
    pub fn new(grid: Grid, capacity: u32) -> Self {
        let slots = capacity as usize;
        Self {
            grid,
            capacity,
            generations: Vec::with_capacity(slots),
            live: Vec::with_capacity(slots),
            tiles: Vec::with_capacity(slots),
            factions: Vec::with_capacity(slots),
            carries: Vec::with_capacity(slots),
            orders: Vec::with_capacity(slots),
            builds: Vec::with_capacity(slots),
            needs: Vec::with_capacity(slots),
            deficits: Vec::with_capacity(slots),
            homes: Vec::with_capacity(slots),
            intents: Vec::with_capacity(slots),
            deeds: Vec::with_capacity(slots),
            eligible: Vec::with_capacity(slots),
            characters: Vec::with_capacity(slots),
            deed_threshold: DEFAULT_DEED_THRESHOLD,
            // Every opened slot can be free at once, because every soldier
            // can die on one tick. A queue that grew on that tick would
            // reallocate inside the step.
            free: VecDeque::with_capacity(slots),
            live_count: 0,
            retired_count: 0,
            revision: 0,
            identity: next_arena_identity(),
        }
    }

    /// Returns the world shape that the arena places soldiers on.
    #[must_use]
    pub const fn grid(&self) -> Grid {
        self.grid
    }

    /// Returns the largest number of slots that the arena opens.
    #[must_use]
    pub const fn capacity(&self) -> u32 {
        self.capacity
    }

    /// Returns the number of live soldiers.
    #[must_use]
    pub const fn len(&self) -> u32 {
        self.live_count
    }

    /// Reports whether the arena holds no live soldier.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.live_count == 0
    }

    /// Returns the number of slots that the arena has opened.
    ///
    /// The count is the high water mark of the live population plus the
    /// retired slots. It never falls.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    #[must_use]
    pub fn slot_count(&self) -> u32 {
        self.generations.len() as u32
    }

    /// Returns the generation that one slot carries.
    ///
    /// A slot that carries no identity returns zero, and zero is never a
    /// generation of a live unit.[^1] A caller that holds a slot index
    /// rebuilds the identity from this value.
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D6. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    #[must_use]
    pub fn generation_of(&self, slot: u32) -> u32 {
        self.generations
            .get(slot as usize)
            .copied()
            .unwrap_or(NO_GENERATION)
    }

    /// Returns the number of slots that the arena has retired.
    #[must_use]
    pub const fn retired_count(&self) -> u32 {
        self.retired_count
    }

    /// Returns the number of structural changes that the arena has taken.
    ///
    /// A spawn, a despawn and a move each raise the count by one. A derived
    /// structure reads it to find out whether it is still current.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    /// Returns the identifier of this arena.
    ///
    /// The identifier names the arena. The revision counts its changes. A
    /// derived structure needs both, because a matching count from a
    /// different arena is not a match.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    #[must_use]
    pub const fn identity(&self) -> u64 {
        self.identity
    }

    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Adds a soldier and returns its identity.
    ///
    /// The arena takes the oldest free slot. It never takes the newest,
    /// because last-in first-out reuse gives one slot every generation
    /// increment and wears it out early.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the arena holds no free slot, when the address
    /// is outside the world, or when the faction is at or above the
    /// ceiling.
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D4. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn spawn(&mut self, address: Axial, faction: FactionId) -> Result<Entity, SoldierError> {
        let tile = self
            .grid
            .index_of(address)
            .ok_or(SoldierError::TileOutsideWorld(address))?;
        if faction.0 >= FACTION_CEILING {
            return Err(SoldierError::FactionAboveCeiling(faction));
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
        // A reused slot starts empty because the despawn emptied it, and the
        // arena invariant fails when a dead slot carries anything. A second
        // reset here would be one fact in two places, and it would read back
        // correctly while the copy that matters was wrong.[^3]
        //
        // [^3]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
        debug_assert_eq!(self.carries[index], CarryLoad::EMPTY);
        debug_assert_eq!(self.orders[index], NO_ORDER);
        debug_assert_eq!(self.builds[index], NO_BUILD);
        // A unit arrives fed and out of deficit, and it belongs to no site
        // until something gives it one.
        self.needs[index] = NEED_FULL;
        self.deficits[index] = Fix32::ZERO;
        self.homes[index] = NO_HOME;
        // A unit arrives holding nothing. It takes an intent at the first
        // choice its cell schedules, and it does not move before then.
        self.intents[index] = NO_INTENT;
        // A unit arrives having done nothing, eligible for nothing, and
        // carrying no character. The despawn cleared all three, and the
        // arena invariant fails when a dead slot carries any of them, so a
        // reset here would be one fact in two places. The assert states the
        // dependency instead, in the way the columns above it do.[^3]
        debug_assert_eq!(self.deeds[index], 0);
        debug_assert_eq!(self.eligible[index], 0);
        debug_assert_eq!(self.characters[index], 0);
        self.live_count += 1;
        self.revision = self.revision.wrapping_add(1);
        Ok(Entity::new(slot, self.generations[index])
            .expect("a generation of one or more makes the identity non-zero"))
    }

    /// Opens one new slot and returns its index.
    fn open_slot(&mut self) -> Result<u32, SoldierError> {
        let slot = self.slot_count();
        if slot >= self.capacity {
            return Err(SoldierError::ArenaFull);
        }
        self.generations.push(NO_GENERATION);
        self.live.push(0);
        self.tiles.push(TileIdx(0));
        self.factions.push(FactionId(0));
        self.carries.push(CarryLoad::EMPTY);
        self.orders.push(NO_ORDER);
        self.builds.push(NO_BUILD);
        self.needs.push(NEED_FULL);
        self.deficits.push(Fix32::ZERO);
        self.homes.push(NO_HOME);
        self.intents.push(NO_INTENT);
        self.deeds.push(0);
        self.eligible.push(0);
        self.characters.push(0);
        Ok(slot)
    }

    /// Removes a soldier and reports whether it removed one.
    ///
    /// A stale identity removes nothing and returns `false`. The arena
    /// reports no error for it, because the caller either handles the absent
    /// soldier or skips it.[^1]
    ///
    /// The generation advances here, at the free, and not at the next
    /// allocation. The identity of a dead soldier is therefore invalid at
    /// the moment the soldier dies.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    /// [^2]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn despawn(&mut self, entity: Entity) -> bool {
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
        // The load leaves with the soldier. The caller reads it before the
        // despawn and records where it went, because what leaves a tile must
        // arrive somewhere exactly.[^3]
        //
        // [^3]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D5. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
        self.carries[index] = CarryLoad::EMPTY;
        self.orders[index] = NO_ORDER;
        self.builds[index] = NO_BUILD;
        self.needs[index] = Fix32::ZERO;
        self.deficits[index] = Fix32::ZERO;
        self.homes[index] = NO_HOME;
        self.intents[index] = NO_INTENT;
        // The deeds end with the unit. The character does not: it was created
        // as its own entity and it outlives the body that earned it, so the
        // despawn clears the link and never removes the character.[^4]
        //
        // [^4]: ADR-0104, a soldier is promoted from a level that never falls, decision D3. `docs/adrs/draft/adr-0104-a-soldier-is-promoted-from-a-level-that-never-falls.md`
        self.deeds[index] = 0;
        self.eligible[index] = 0;
        self.characters[index] = 0;
        self.live_count -= 1;
        self.revision = self.revision.wrapping_add(1);
        if self.generations[index] == LAST_GENERATION {
            // The generation cannot advance, so the slot never returns. One
            // leaked slot beats two soldiers that share one identity.
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
    /// generation in the location table. A mismatch means the soldier is
    /// dead, and a dead identity resolves to nothing.[^1]
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

    /// Reports whether the identity names a live soldier.
    #[must_use]
    pub fn contains(&self, entity: Entity) -> bool {
        self.slot_of(entity).is_some()
    }

    /// Returns the tile of a soldier, or `None` when the identity is dead.
    #[must_use]
    pub fn tile(&self, entity: Entity) -> Option<TileIdx> {
        let slot = self.slot_of(entity)?;
        Some(self.tiles[slot as usize])
    }

    /// Returns the address of a soldier, or `None` when the identity is
    /// dead.
    #[must_use]
    pub fn address(&self, entity: Entity) -> Option<Axial> {
        self.grid.address_of(self.tile(entity)?)
    }

    /// Returns the faction of a soldier, or `None` when the identity is
    /// dead.
    #[must_use]
    pub fn faction(&self, entity: Entity) -> Option<FactionId> {
        let slot = self.slot_of(entity)?;
        Some(self.factions[slot as usize])
    }

    /// Moves a soldier to another tile.
    ///
    /// Returns `false` when the identity is dead. The caller handles the
    /// absent soldier or skips it.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the address is outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn place(&mut self, entity: Entity, address: Axial) -> Result<bool, SoldierError> {
        // Resolve the identity first. A dead handle moves nothing, whatever
        // address it names, so it gives `Ok(false)` and not an error about an
        // address that was never going to be used.[^1]
        let Some(slot) = self.slot_of(entity) else {
            return Ok(false);
        };
        let tile = self
            .grid
            .index_of(address)
            .ok_or(SoldierError::TileOutsideWorld(address))?;
        self.tiles[slot as usize] = tile;
        self.revision = self.revision.wrapping_add(1);
        Ok(true)
    }

    /// Returns the live soldiers in slot order.
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
    /// wants the live soldiers walks the identities instead.[^1]
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

    /// Returns the whole carry column.
    #[must_use]
    pub fn carry_column(&self) -> &[CarryLoad] {
        &self.carries
    }

    /// Returns the whole gather order column.
    #[must_use]
    pub fn order_column(&self) -> &[u8] {
        &self.orders
    }

    /// Returns what one soldier carries.
    ///
    /// Returns `None` when the identity is dead.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    #[must_use]
    pub fn carry(&self, entity: Entity) -> Option<CarryLoad> {
        let slot = self.slot_of(entity)?;
        Some(self.carries[slot as usize])
    }

    /// Adds an amount to what one soldier carries.
    ///
    /// Returns `false` when the identity is dead. The gather resolve is the
    /// only caller, so the visibility stays inside the crate: a load that a
    /// caller could raise on its own would break conservation in silence.
    ///
    /// A load is not a structural fact, so it does not raise the revision. The
    /// derived unit structure maps a tile to the units on it, and a load moves
    /// no unit.
    pub(crate) fn add_carry(&mut self, entity: Entity, kind: ResourceKind, amount: Amount) -> bool {
        let Some(slot) = self.slot_of(entity) else {
            return false;
        };
        let index = slot as usize;
        self.carries[index] = self.carries[index].with(kind, amount);
        // **This is the one place a deed is recorded.** A unit receives what
        // it gathered here and nowhere else, so the running total rises here
        // and nowhere else, and it cannot fall because nothing else writes
        // it. The eligibility byte is written in the same statement pair, so
        // the two cannot be left disagreeing by a path that raises one.[^4]
        //
        // The add saturates. A total that wrapped would fall, and a level
        // that falls breaks the scan that reads it.[^4]
        //
        // [^4]: ADR-0104, a soldier is promoted from a level that never falls, decisions D1 and D2. `docs/adrs/draft/adr-0104-a-soldier-is-promoted-from-a-level-that-never-falls.md`
        self.deeds[index] = self.deeds[index].saturating_add(u64::from(amount.0));
        self.eligible[index] = u8::from(self.deeds[index] >= self.deed_threshold);
        true
    }

    /// Takes an amount of one kind out of the load of a soldier.
    ///
    /// Returns `false` when the identity is dead.
    ///
    /// **The deed column does not follow the load.** A deed records what a
    /// unit took from the ground, and a delivery does not undo that. The
    /// column never falls, and the eligibility scan reads a level rather than
    /// an edge, so a rule that lowered it would break the scan in
    /// silence.[^1]
    ///
    /// A load is not a structural fact, so a delivery does not raise the
    /// revision, for the same reason that a gather does not.
    ///
    /// # References
    ///
    /// [^1]: ADR-0104, a soldier is promoted from a level that never falls, decision D2. `docs/adrs/draft/adr-0104-a-soldier-is-promoted-from-a-level-that-never-falls.md`
    pub(crate) fn take_carry(
        &mut self,
        entity: Entity,
        kind: ResourceKind,
        amount: Amount,
    ) -> bool {
        let Some(slot) = self.slot_of(entity) else {
            return false;
        };
        let index = slot as usize;
        self.carries[index] = self.carries[index].less(kind, amount);
        true
    }

    /// Returns what a soldier has ever gathered, summed over every kind.
    ///
    /// Returns `None` when the identity is dead.
    #[must_use]
    pub fn deeds(&self, entity: Entity) -> Option<u64> {
        let slot = self.slot_of(entity)?;
        Some(self.deeds[slot as usize])
    }

    /// Returns the whole deed column.
    #[must_use]
    pub fn deed_column(&self) -> &[u64] {
        &self.deeds
    }

    /// Returns the whole eligibility column. One means eligible.
    #[must_use]
    pub fn eligible_column(&self) -> &[u8] {
        &self.eligible
    }

    /// Returns the whole character link column, as identity bits.
    #[must_use]
    pub fn character_column(&self) -> &[u64] {
        &self.characters
    }

    /// Returns the deeds at which a unit becomes eligible for promotion.
    #[must_use]
    pub const fn deed_threshold(&self) -> u64 {
        self.deed_threshold
    }

    /// Sets the deeds at which a unit becomes eligible for promotion.
    ///
    /// The eligibility column is a second statement of the comparison, so
    /// changing the threshold rewrites it. A pass that left the column as the
    /// old threshold wrote it would promote against a rule nobody set.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    pub fn set_deed_threshold(&mut self, threshold: u64) {
        self.deed_threshold = threshold;
        for index in 0..self.deeds.len() {
            self.eligible[index] =
                u8::from(self.live[index] == 1 && self.deeds[index] >= threshold);
        }
    }

    /// Returns the character that a soldier was promoted into.
    ///
    /// The outer option reports whether the identity is live. The inner one
    /// reports whether the soldier carries a character. The answer names a
    /// character that may already have been removed, so resolve it against
    /// the character arena before reading anything of it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    #[must_use]
    pub fn character_of(&self, entity: Entity) -> Option<Option<Entity>> {
        let slot = self.slot_of(entity)?;
        Some(Entity::from_bits(self.characters[slot as usize]))
    }

    /// Links a soldier to the character it was promoted into.
    ///
    /// Returns `false` when the identity is dead, and `false` when the
    /// soldier already carries a character. A second promotion of one unit
    /// would put two people in one body.
    pub(crate) fn promote(&mut self, entity: Entity, character: Entity) -> bool {
        let Some(slot) = self.slot_of(entity) else {
            return false;
        };
        let index = slot as usize;
        if self.characters[index] != 0 {
            return false;
        }
        self.characters[index] = character.to_bits();
        true
    }

    /// Returns the gather order of one soldier.
    ///
    /// The outer option reports whether the identity is live. The inner one
    /// reports whether the soldier gathers.
    #[must_use]
    pub fn gather_order(&self, entity: Entity) -> Option<Option<ResourceKind>> {
        let slot = self.slot_of(entity)?;
        Some(order_of(self.orders[slot as usize]))
    }

    /// Sets the gather order of one soldier.
    ///
    /// Returns `false` when the identity is dead. An order of `None` stops the
    /// soldier gathering.
    pub fn set_gather_order(&mut self, entity: Entity, kind: Option<ResourceKind>) -> bool {
        let Some(slot) = self.slot_of(entity) else {
            return false;
        };
        self.orders[slot as usize] = match kind {
            Some(kind) => kind.to_u8() + 1,
            None => NO_ORDER,
        };
        true
    }

    /// Returns the whole build order column.
    #[must_use]
    pub fn build_column(&self) -> &[u8] {
        &self.builds
    }

    /// Returns the build order of one soldier.
    ///
    /// The outer option reports whether the identity is live. The inner one
    /// reports whether the soldier builds.
    #[must_use]
    pub fn build_order(&self, entity: Entity) -> Option<Option<UpgradeKind>> {
        let slot = self.slot_of(entity)?;
        Some(build_of(self.builds[slot as usize]))
    }

    /// Sets the build order of one soldier.
    ///
    /// Returns `false` when the identity is dead. An order of `None` stops
    /// the soldier building.
    ///
    /// An order is not a structural fact, so it does not raise the revision.
    /// The derived unit structure maps a tile to the units on it, and an
    /// order moves no unit.
    pub fn set_build_order(&mut self, entity: Entity, kind: Option<UpgradeKind>) -> bool {
        let Some(slot) = self.slot_of(entity) else {
            return false;
        };
        self.builds[slot as usize] = match kind {
            Some(kind) => kind.to_u8() + 1,
            None => NO_BUILD,
        };
        true
    }

    /// Returns the need of a soldier, or `None` when the identity is dead.
    #[must_use]
    pub fn need(&self, entity: Entity) -> Option<Fix32> {
        let slot = self.slot_of(entity)?;
        Some(self.needs[slot as usize])
    }

    /// Returns the deficit of a soldier, or `None` when the identity is
    /// dead.
    #[must_use]
    pub fn deficit(&self, entity: Entity) -> Option<Fix32> {
        let slot = self.slot_of(entity)?;
        Some(self.deficits[slot as usize])
    }

    /// Returns the site that a soldier draws from.
    ///
    /// Returns `None` when the identity is dead, and `Some(None)` when the
    /// soldier belongs to no site.
    #[must_use]
    pub fn home(&self, entity: Entity) -> Option<Option<u32>> {
        let slot = self.slot_of(entity)?;
        let home = self.homes[slot as usize];
        Some(if home == NO_HOME { None } else { Some(home) })
    }

    /// Writes the site that a soldier draws from, and reports whether it
    /// wrote.
    ///
    /// Returns `false` when the identity is dead. The caller handles the
    /// absent soldier or skips it.[^1]
    ///
    /// The arena takes a slot of the settlement arena and does not check it,
    /// because this arena holds no settlement column. The world owns that
    /// check, and its invariant check states it.
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn set_home(&mut self, entity: Entity, home: Option<u32>) -> bool {
        let Some(slot) = self.slot_of(entity) else {
            return false;
        };
        self.homes[slot as usize] = home.unwrap_or(NO_HOME);
        true
    }

    /// Returns the intent of a soldier.
    ///
    /// The outer option reports whether the identity is live. The inner one
    /// reports whether the soldier holds an intent at all. A soldier that
    /// holds none has found nothing above the floor, and it does not move.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D3. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
    #[must_use]
    pub fn intent(&self, entity: Entity) -> Option<Option<u8>> {
        let slot = self.slot_of(entity)?;
        let intent = self.intents[slot as usize];
        Some(if intent == NO_INTENT {
            None
        } else {
            Some(intent)
        })
    }

    /// Returns the whole intent column.
    #[must_use]
    pub fn intent_column(&self) -> &[u8] {
        &self.intents
    }

    /// Writes the intent of one slot.
    ///
    /// The choice pass is the only caller, so the visibility stays inside
    /// the crate. An intent that a caller could write on its own would let
    /// the control plane drive one unit at a time, which the project
    /// forbids.[^1]
    ///
    /// An intent is not a structural fact, so it does not raise the
    /// revision. The derived unit structure maps a tile to the units on it,
    /// and an intent moves no unit.
    ///
    /// # References
    ///
    /// [^1]: ADR-0010, Python is a control plane, and it never touches an entity one at a time. `docs/adrs/REGISTRY.md`
    pub(crate) fn set_intent_at(&mut self, slot: u32, intent: u8) {
        self.intents[slot as usize] = intent;
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

    /// Returns the whole need column.
    #[must_use]
    pub fn need_column(&self) -> &[Fix32] {
        &self.needs
    }

    /// Returns the whole deficit column.
    #[must_use]
    pub fn deficit_column(&self) -> &[Fix32] {
        &self.deficits
    }

    /// Returns the whole home column.
    #[must_use]
    pub fn home_column(&self) -> &[u32] {
        &self.homes
    }

    /// Returns the columns that a pass over the needs needs.
    ///
    /// One call hands over all four, because four separate calls would
    /// borrow the arena four times and two of those borrows are mutable.
    pub fn need_update(&mut self) -> NeedUpdate<'_> {
        NeedUpdate {
            needs: &mut self.needs,
            deficits: &mut self.deficits,
            live: &self.live,
            homes: &self.homes,
            factions: &self.factions,
        }
    }

    /// Absorbs the soldier columns into the state hash.
    ///
    /// The hash covers every byte that decides a later frame. It therefore
    /// covers the generation of each slot and the free queue, because both
    /// decide which slot the next spawn takes.[^1]
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
            .write(bytemuck::cast_slice(&self.carries))
            .write(&self.orders)
            .write(&self.builds)
            .write(bytemuck::cast_slice(&self.needs))
            .write(bytemuck::cast_slice(&self.deficits))
            .write(bytemuck::cast_slice(&self.homes))
            .write(&self.intents)
            .write(bytemuck::cast_slice(&self.deeds))
            .write(&self.eligible)
            .write(bytemuck::cast_slice(&self.characters))
            .write_u64(self.deed_threshold)
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
    /// [^1]: Recurring defect shapes, section 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub fn check_invariants(&self) -> bool {
        let slots = self.generations.len();
        if self.live.len() != slots || self.tiles.len() != slots || self.factions.len() != slots {
            return false;
        }
        if self.needs.len() != slots || self.deficits.len() != slots || self.homes.len() != slots {
            return false;
        }
        if self.intents.len() != slots {
            return false;
        }
        if self.deeds.len() != slots
            || self.eligible.len() != slots
            || self.characters.len() != slots
        {
            return false;
        }
        // The eligibility byte states `deeds >= deed_threshold`, and the
        // deeds state it too. This is the check that fails when the two
        // copies disagree, because nothing else would.[^4]
        //
        // A dead slot is eligible for nothing. It holds no deeds either, so
        // the comparison would answer for it, but a threshold of zero makes
        // every dead slot read as eligible and the promotion scan would find
        // them.
        //
        // [^4]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
        for index in 0..slots {
            let earned = self.live[index] == 1 && self.deeds[index] >= self.deed_threshold;
            if self.eligible[index] != u8::from(earned) {
                return false;
            }
        }
        // A dead slot carries no deeds and no character. A stale link there
        // would name a character for a unit that no longer exists.
        for index in 0..slots {
            if self.live[index] == 1 {
                continue;
            }
            if self.deeds[index] != 0 || self.characters[index] != 0 {
                return false;
            }
        }
        // An intent names an option the set holds, or it names nothing. A
        // number outside the set would index past the option table.
        if self
            .intents
            .iter()
            .any(|intent| *intent != NO_INTENT && (*intent as usize) >= crate::choose::OPTION_COUNT)
        {
            return false;
        }
        // A dead slot holds no intent. A stale intent there would reach the
        // state hash and would move the next unit in the slot before it had
        // read anything.
        if (0..slots).any(|slot| self.live[slot] == 0 && self.intents[slot] != NO_INTENT) {
            return false;
        }
        // A slot that is not live holds no need, no deficit and no site. A
        // stale value there would reach the state hash and would feed the
        // next unit in the slot.
        if (0..slots).any(|slot| {
            self.live[slot] == 0
                && (self.needs[slot] != Fix32::ZERO
                    || self.deficits[slot] != Fix32::ZERO
                    || self.homes[slot] != NO_HOME)
        }) {
            return false;
        }
        // A need never leaves its range, and a deficit never falls below
        // zero. Both are the floors and ceilings that the saturating
        // arithmetic states.
        if (0..slots).any(|slot| self.needs[slot] < Fix32::ZERO || self.needs[slot] > NEED_FULL) {
            return false;
        }
        if (0..slots).any(|slot| self.deficits[slot] < Fix32::ZERO) {
            return false;
        }
        if self.carries.len() != slots || self.orders.len() != slots {
            return false;
        }
        if self.builds.len() != slots {
            return false;
        }
        // A build order names a kind the catalogue holds, or it names
        // nothing. A number outside the catalogue would read as a kind that
        // the advance cannot build, and the soldier would then build nothing
        // without any caller asking for that.
        if self
            .builds
            .iter()
            .any(|order| *order != NO_BUILD && UpgradeKind::from_u8(*order - 1).is_none())
        {
            return false;
        }
        // A dead slot holds no build order. A stale order there would reach
        // the state hash and would set the next unit in the slot building.
        if (0..slots).any(|slot| self.live[slot] == 0 && self.builds[slot] != NO_BUILD) {
            return false;
        }
        // An order names a kind the catalogue holds, or it names nothing. A
        // number outside the catalogue would read as a kind that the resolve
        // cannot serve, and the soldier would then gather nothing without any
        // caller asking for that.
        if self
            .orders
            .iter()
            .any(|order| *order != NO_ORDER && ResourceKind::from_u8(*order - 1).is_none())
        {
            return false;
        }
        // A dead slot carries nothing. The world returns a dead soldier's load
        // to the register that records what left the world, so a load left in
        // a dead slot would be counted twice.
        if (0..slots).any(|slot| self.live[slot] == 0 && self.carries[slot] != CarryLoad::EMPTY) {
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
    //! it would need four thousand million spawns. The test therefore sets
    //! the generation here.
    //!
    //! # References
    //!
    //! [^1]: ADR-0014, entity identity is an index plus a generation, decision D5. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`

    use super::*;

    /// Builds an arena over a small world.
    fn arena() -> SoldierArena {
        SoldierArena::new(
            Grid::new(4, 4).expect("a small extent describes a grid"),
            16,
        )
    }

    #[test]
    fn a_slot_at_the_last_generation_retires_on_the_free() {
        let mut arena = arena();
        let first = arena
            .spawn(Axial::new(0, 0), FactionId(0))
            .expect("the spawn must succeed");
        arena.generations[0] = LAST_GENERATION;
        let aged = Entity::new(0, LAST_GENERATION).expect("the identity is not zero");
        assert!(!arena.contains(first));
        assert!(arena.despawn(aged));
        assert_eq!(arena.retired_count(), 1);
        assert!(arena.free.is_empty());
        assert!(arena.check_invariants());
    }

    #[test]
    fn a_despawn_of_a_slot_that_is_not_live_changes_nothing() {
        // The public interface cannot reach this state, because an identity
        // resolves only while its generation matches and the arena marks a
        // slot live before it hands out the identity. The guard exists
        // because the argument that it holds runs across three functions,
        // and the failure it prevents is a count that wraps rather than a
        // panic. The test constructs the state the interface cannot.
        let mut arena = arena();
        let entity = arena
            .spawn(Axial::new(0, 0), FactionId(0))
            .expect("the spawn must succeed");
        arena.live[0] = 0;
        assert!(!arena.despawn(entity));
        assert_eq!(arena.len(), 1, "the live count must not wrap");
    }

    #[test]
    fn a_free_queue_that_holds_one_slot_twice_fails_the_check() {
        // One slot in the queue twice hands one slot to two callers. It is
        // the worst failure this structure has, and a caller outside cannot
        // detect it.
        let mut arena = arena();
        let entity = arena
            .spawn(Axial::new(0, 0), FactionId(0))
            .expect("the spawn must succeed");
        assert!(arena.despawn(entity));
        assert!(arena.check_invariants());
        arena.free.push_back(0);
        assert!(!arena.check_invariants());
    }

    #[test]
    fn a_slot_that_is_neither_live_nor_queued_nor_retired_fails_the_check() {
        // A lost slot is invisible: it is not live, so nothing reads it, and
        // it is not queued, so nothing reuses it. The arena simply shrinks.
        let mut arena = arena();
        let entity = arena
            .spawn(Axial::new(0, 0), FactionId(0))
            .expect("the spawn must succeed");
        assert!(arena.despawn(entity));
        arena.free.clear();
        assert!(!arena.check_invariants());
    }

    #[test]
    fn a_retired_slot_never_returns_to_use() {
        let mut arena = arena();
        arena
            .spawn(Axial::new(0, 0), FactionId(0))
            .expect("the spawn must succeed");
        arena.generations[0] = LAST_GENERATION;
        let aged = Entity::new(0, LAST_GENERATION).expect("the identity is not zero");
        assert!(arena.despawn(aged));

        let next = arena
            .spawn(Axial::new(1, 0), FactionId(0))
            .expect("the spawn must succeed");
        assert_ne!(next.index(), 0, "a retired slot must never return");
        assert_eq!(arena.slot_count(), 2);
        assert!(arena.check_invariants());
    }

    #[test]
    fn a_retired_slot_resolves_no_identity() {
        let mut arena = arena();
        arena
            .spawn(Axial::new(0, 0), FactionId(0))
            .expect("the spawn must succeed");
        arena.generations[0] = LAST_GENERATION;
        let aged = Entity::new(0, LAST_GENERATION).expect("the identity is not zero");
        assert!(arena.despawn(aged));
        assert!(!arena.contains(aged));
        assert_eq!(arena.tile(aged), None);
    }

    #[test]
    fn a_short_column_fails_the_check() {
        let mut arena = arena();
        arena
            .spawn(Axial::new(0, 0), FactionId(0))
            .expect("the spawn must succeed");
        assert!(arena.check_invariants());
        arena.factions.pop();
        assert!(!arena.check_invariants());
    }

    #[test]
    fn a_live_count_that_disagrees_fails_the_check() {
        let mut arena = arena();
        arena
            .spawn(Axial::new(0, 0), FactionId(0))
            .expect("the spawn must succeed");
        arena.live_count = 7;
        assert!(!arena.check_invariants());
    }
}
