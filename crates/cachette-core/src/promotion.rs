//! Promotion of a soldier into the character tier.
//!
//! One million soldiers each carry an experience of their own. A story needs
//! somebody it is about, and this pass is where the world chooses one: a
//! soldier whose deeds reach a level becomes a person the world can name.
//!
//! # What a promotion is
//!
//! **A promotion creates. It never mutates.** An entity declares its tier when
//! it is created and never changes tier while it lives, so the soldier does
//! not become a character.[^1] The pass creates a character, links the soldier
//! to it, and leaves both rows where they were. The soldier keeps moving by
//! the pass that moves every soldier.
//!
//! **A promoted soldier gets no invented ancestry.** The character founds a
//! new line, holds a relation of zero to everybody, and cannot inherit by
//! blood. A title holder may appoint them.[^2]
//!
//! # The order
//!
//! The eligible set is collected in ascending slot order, ranked by a key
//! vector, and the identities are allocated after the budget cut and never
//! during the scan.[^3] [^4]
//!
//! # References
//!
//! [^1]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D4. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
//! [^2]: Blockers register, BLK-011. `docs/BLOCKERS.md`
//! [^3]: ADR-0007, content supplies a key vector, never a comparator, decisions D1 and D2. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
//! [^4]: ADR-0004, iteration order is explicit, decisions D1 and D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`

use bytemuck::{Pod, Zeroable};

use crate::character::{CharacterArena, CharacterError};
use crate::soldier::SoldierArena;
use crate::sort::{self, SortError, SortKey};
use crate::types::{Entity, FactionId, Tick};

/// The reason that a promotion pass refused to run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PromotionError {
    /// The caller asked for zero threads.
    ZeroThreads,
    /// The eligible set refused to sort.
    Ranking(SortError),
    /// The character arena refused to create a character.
    Arena(CharacterError),
}

impl core::fmt::Display for PromotionError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ZeroThreads => write!(formatter, "a pass needs at least one thread"),
            Self::Ranking(error) => write!(formatter, "the eligible set did not sort: {error}"),
            Self::Arena(error) => write!(formatter, "the character arena refused: {error}"),
        }
    }
}

impl std::error::Error for PromotionError {}

/// One promotion, as the log records it.
///
/// The layout is 8 + 8 + 8 + 8 + 2 + 6 bytes, which is 40 bytes at an
/// alignment of 8. The trailing array declares every padding byte.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct UnitPromoted {
    /// The tick at which the pass promoted the unit.
    pub tick: Tick,
    /// The unit that was promoted, as its identity in bits.
    pub unit: u64,
    /// The character that the promotion created, as its identity in bits.
    pub character: u64,
    /// The deeds that the unit carried. It is at or above the threshold.
    pub deeds: u64,
    /// The faction of the unit and of the character.
    pub faction: FactionId,
    /// The declared padding. Always zero.
    pub padding: [u8; 6],
}

impl UnitPromoted {
    /// Builds an event with zero padding.
    #[must_use]
    pub const fn new(
        tick: Tick,
        unit: u64,
        character: u64,
        deeds: u64,
        faction: FactionId,
    ) -> Self {
        Self {
            tick,
            unit,
            character,
            deeds,
            faction,
            padding: [0; 6],
        }
    }
}

/// Promotes the units that earned it, up to the budget, and returns the log.
///
/// # What it does
///
/// The pass scans the eligibility column, which the arena maintains whenever a
/// unit is given what it gathered. A unit enters the set when it is live, when
/// its byte says it reached the threshold, and when it carries no character
/// already. **A unit is promoted once.** The character link is what says so,
/// so the pass needs no second flag and cannot promote a unit twice.
///
/// The set is ranked by deeds, highest first, and the whole identity breaks
/// every tie.[^1] The pass then cuts the ranked set at the budget and creates
/// one character for each unit that survives the cut, in rank order.
///
/// **The identities are allocated after the cut.** A pass that minted an
/// identity during the scan would consume a character slot for a unit the
/// budget then rejected.[^2]
///
/// # The budget
///
/// The caller states how many characters may be created. The character arena
/// is built at the ceiling of its declared tier and refuses a create beyond
/// it, so the ceiling is enforced whatever the caller passes. The budget is
/// the cut at the rank, and it is not a second statement of the ceiling.[^3]
///
/// # Errors
///
/// Returns an error when the caller asks for zero threads, when the eligible
/// set refuses to sort, and when the character arena refuses a create for a
/// reason other than being full. **A full arena is not an error.** It is the
/// ceiling doing its work, and the pass stops promoting and returns what it
/// promoted.
///
/// # References
///
/// [^1]: ADR-0007, content supplies a key vector, never a comparator, decision D2. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
/// [^2]: ADR-0104, a soldier is promoted from a level that never falls, decision D4. `docs/adrs/draft/adr-0104-a-soldier-is-promoted-from-a-level-that-never-falls.md`
/// [^3]: Blockers register, BLK-004. `docs/BLOCKERS.md`
pub fn promote(
    units: &mut SoldierArena,
    characters: &mut CharacterArena,
    seed: u64,
    tick: Tick,
    budget: u32,
    threads: usize,
) -> Result<Vec<UnitPromoted>, PromotionError> {
    if threads == 0 {
        return Err(PromotionError::ZeroThreads);
    }
    if budget == 0 {
        return Ok(Vec::new());
    }

    // The scan reads the eligibility byte and never the deeds, which is what
    // the byte exists for. The slot order is fixed, so the input to the rank
    // below is the same on every run.
    let eligible = units.eligible_column();
    let links = units.character_column();
    let deeds = units.deed_column();
    let mut candidates: Vec<(u64, Entity)> = Vec::new();
    for slot in 0..units.slot_count() {
        let index = slot as usize;
        if eligible[index] != 1 || links[index] != 0 {
            continue;
        }
        let generation = units.generation_of(slot);
        let Some(unit) = Entity::new(slot, generation) else {
            continue;
        };
        candidates.push((deeds[index], unit));
    }
    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    // The rank puts the largest deeds first. The key vector orders ascending,
    // so the field is the complement of the deeds rather than the deeds. The
    // last field is the whole identity, which is unique across the set, so no
    // two candidates tie and the order is total.
    let keys: Vec<SortKey<2>> = candidates
        .iter()
        .map(|(earned, unit)| SortKey::new([u64::MAX - *earned, unit.to_bits()]))
        .collect();
    let order = sort::order_on(&keys, threads).map_err(PromotionError::Ranking)?;

    let mut log = Vec::new();
    for index in order.into_iter().take(budget as usize) {
        let (earned, unit) = candidates[index as usize];
        let Some(faction) = units.faction(unit) else {
            continue;
        };
        let character = match characters.create(seed, faction, tick) {
            Ok(character) => character,
            // The arena is full. The ceiling is the bound this pass exists to
            // respect, so reaching it ends the pass rather than failing it.
            Err(CharacterError::ArenaFull) => break,
            Err(error) => return Err(PromotionError::Arena(error)),
        };
        let linked = units.promote(unit, character);
        debug_assert!(linked, "the unit came from the live eligible set");
        log.push(UnitPromoted::new(
            tick,
            unit.to_bits(),
            character.to_bits(),
            earned,
            faction,
        ));
    }
    Ok(log)
}
