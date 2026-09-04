//! The living character column set.
//!
//! The entity storage holds four fixed shapes, and each shape gets its own
//! set of columns.[^1] The living character is one of the four. It carries
//! no tile position and none of the soldier columns, because that absence
//! is the reason the shape is separate.[^1]
//!
//! The shapes do not vary at run time. A shape that is not one of the four
//! is a compile-time error here, because a column set is a Rust type and
//! not a row in a table.[^2]
//!
//! The shape declares the character tier, and it declares it at the type.
//! The tier decides the ceiling of the population.[^3] The arena checks
//! that ceiling once, when a caller builds it, and never on a later call.
//!
//! Every entity lives in the generational arena, and its identity pairs a
//! slot index with a generation.[^4] The arena mints every identity. No
//! caller builds one from parts.[^5] The generation advances when the arena
//! frees the slot, so a character who is gone never hands the identity to
//! the character created next in that slot.[^6]
//!
//! **The columns are struct-of-arrays: one array for each field.** The pass
//! that decides that is descent and succession, and every kernel of it reads
//! one or two columns for each row it visits.[^10] The layout follows the
//! column count of that pass and not the name of the tier the shape sits
//! in.[^11]
//!
//! Every column holds an exact integer or a Q16.16 fixed-point value. No
//! column holds a floating point number.[^7] A renown of zero is a real
//! state, and the renown type represents it.[^8]
//!
//! The arena holds no biography log. A biography log is the one character
//! structure that grows without bound, and the research states the rule
//! that must govern it before anything writes one.[^9]
//!
//! # References
//!
//! [^1]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
//! [^2]: ADR-0066, entity storage holds four fixed shapes, decision D3. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
//! [^3]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decisions D1 and D3. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
//! [^4]: ADR-0012, tiles are dense columns and units are a generational arena, decision D3. `docs/adrs/accepted/adr-0012-tiles-are-dense-columns-and-units-are-a-generational-arena.md`
//! [^5]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
//! [^6]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
//! [^7]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^8]: Findings register, FND-043. `docs/FINDINGS.md`
//! [^9]: The character graph and inheritance, section 2.5. `docs/research/reports/14-character-graph-and-inheritance.md`
//! [^10]: The character graph and inheritance, section 3.2. `docs/research/reports/14-character-graph-and-inheritance.md`
//! [^11]: ADR-0021, a layout claim names one structure and one pass, and never a tier, decisions D2 and D3. `docs/adrs/draft/adr-0021-layout-follows-the-access-pattern.md`

use std::collections::VecDeque;

use crate::descent::{Descent, DescentError, DescentId, HouseId, Parents, DESCENT_CEILING};
use crate::hash::StateHash;
use crate::rng;
use crate::tier::{EntityTier, Shape};
use crate::types::{Entity, FactionId, Fix32, Tick, FACTION_CEILING};

/// The generation that means a slot carries no identity.
///
/// A generation starts at one, so no handle ever holds this value.[^1]
///
/// # References
///
/// [^1]: ADR-0014, entity identity is an index plus a generation, decision D6. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
const NO_GENERATION: u32 = 0;

/// The first generation of a slot.
///
/// The record starts a generation at one, never at zero. Slot zero at
/// generation zero packs to the value zero, which the identity cannot
/// hold.[^1]
///
/// # References
///
/// [^1]: ADR-0014, entity identity is an index plus a generation, decision D6. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
const FIRST_GENERATION: u32 = 1;

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
pub enum CharacterError {
    /// The arena holds no free slot and cannot open a new one.
    ArenaFull,
    /// The faction identifier is at or above the ceiling.
    FactionAboveCeiling(FactionId),
    /// The asked-for capacity is above the ceiling of the declared tier.
    ///
    /// The arena reports this when a caller builds it, and never on a
    /// later call. A check on the current count would let the same script
    /// work on a small world and fail on a large one.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decisions D2 and D3. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
    CapacityAboveCeiling {
        /// The capacity that the caller asked for.
        asked: u32,
        /// The ceiling that the declared tier states.
        ceiling: u32,
        /// The tier that the shape declares.
        tier: EntityTier,
    },
    /// The identity of a parent no longer resolves.
    ///
    /// A birth reads the record of descent of each parent, so each parent
    /// must be alive when the child is born. A caller that holds the
    /// identity of a character who is gone receives this.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    ParentIsGone(Entity),
    /// The two parents of the birth are one character.
    ParentsAreOneCharacter(Entity),
    /// The record of descent refused the birth.
    Descent(DescentError),
}

/// The sex of a character.
///
/// The world draws the sex when it creates the character. The value is a
/// fact about the character and not a fact about its descent, so it lives
/// in the slot columns and a death releases it.[^1]
///
/// # References
///
/// [^1]: Decisions register, DEC-003. `docs/DECISIONS.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Sex {
    /// The character can be the mother of a child.
    Female,
    /// The character can be the father of a child.
    Male,
}

impl Sex {
    /// Returns the sex that a raw draw names.
    const fn from_draw(draw: u64) -> Self {
        if draw == 0 {
            Self::Female
        } else {
            Self::Male
        }
    }

    /// Returns the raw value that the column stores.
    ///
    /// A boundary that answers with a column of sexes reports this value. It
    /// reads the number from here rather than writing a second copy of the
    /// encoding, because two copies of one value have nothing that fails when
    /// they disagree.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub const fn to_column(self) -> u8 {
        match self {
            Self::Female => 0,
            Self::Male => 1,
        }
    }
}

impl core::fmt::Display for Sex {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let name = match self {
            Self::Female => "female",
            Self::Male => "male",
        };
        formatter.write_str(name)
    }
}

/// The draw index of the sex of a character who founds a line.
///
/// A character who founds a line already holds an identity when the world
/// draws, so the draw keys on that identity. A birth cannot do that,
/// because the child holds no identity when the draw happens.[^1]
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
const DRAW_FOUNDER_SEX: u32 = 0;

/// The number of values that the sex draw chooses between.
const SEX_OPTIONS: u64 = 2;

impl From<DescentError> for CharacterError {
    fn from(error: DescentError) -> Self {
        Self::Descent(error)
    }
}

impl core::fmt::Display for CharacterError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ArenaFull => write!(formatter, "the character arena holds no free slot"),
            Self::FactionAboveCeiling(faction) => write!(
                formatter,
                "the faction {} is at or above the ceiling {FACTION_CEILING}",
                faction.0
            ),
            Self::CapacityAboveCeiling {
                asked,
                ceiling,
                tier,
            } => write!(
                formatter,
                "a capacity of {asked} is above the {tier} tier ceiling of {ceiling}"
            ),
            Self::ParentIsGone(parent) => write!(
                formatter,
                "the parent {} is gone and cannot bear a child",
                parent.to_bits()
            ),
            Self::ParentsAreOneCharacter(parent) => write!(
                formatter,
                "the two parents are both the character {}",
                parent.to_bits()
            ),
            Self::Descent(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for CharacterError {}

/// The column set of the living character shape.
///
/// The arena holds one entry for each slot it has ever opened, and it never
/// compacts the slot index space. Compaction would move a character to
/// another slot and invalidate every identity that names them.[^1]
///
/// The columns are dense arrays indexed by the slot. The arena never looks
/// a character up in a hash map, because a hash map costs a hash on the hot
/// path and carries an iteration order that no key fixes.[^2]
///
/// **The arena holds no tile column.** A living character carries no tile
/// position, and that absence is the reason the shape is separate from the
/// soldier shape.[^3]
///
/// # References
///
/// [^1]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
/// [^2]: ADR-0014, entity identity is an index plus a generation, decision D7. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
/// [^3]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
#[derive(Clone, Debug)]
pub struct CharacterArena {
    /// The largest number of slots that the arena opens.
    capacity: u32,
    /// The generation of each slot. Zero means the slot carries no identity.
    generations: Vec<u32>,
    /// One for a live slot, zero for a free slot or a retired slot.
    live: Vec<u8>,
    /// The faction of each slot.
    factions: Vec<FactionId>,
    /// The tick that each character was created on.
    births: Vec<Tick>,
    /// The renown of each slot, as a Q16.16 value.
    renown: Vec<Fix32>,
    /// The sex of each slot. Zero is female and one is male.
    sexes: Vec<u8>,
    /// The row of the record of descent that each slot points at.
    ///
    /// The value names the character that holds the slot now. A creation
    /// overwrites it, so the character created next in a slot never reads
    /// the descent of the character who held it before.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0078, descent is a bounded record, and a relation is a bounded recursion, decision D1. `docs/adrs/draft/adr-0078-descent-is-a-bounded-record-and-a-relation-is-a-bounded-recursion.md`
    descent_of_slot: Vec<u32>,
    /// The record of descent.
    ///
    /// The record is append-only and it holds every character the arena has
    /// ever created. A death releases the slot columns, which the next
    /// character overwrites. It never releases this record.[^1]
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-003. `docs/DECISIONS.md`
    descent: Descent,
    /// The free slots, oldest first.
    free: VecDeque<u32>,
    /// The number of live characters.
    live_count: u32,
    /// The number of retired slots.
    retired_count: u32,
}

impl Default for CharacterArena {
    fn default() -> Self {
        Self::new()
    }
}

impl CharacterArena {
    /// Builds an arena at the ceiling of the declared tier.
    ///
    /// The shape declares the character tier, and the character tier states
    /// the ceiling. Neither the world size nor the current population takes
    /// part in that answer.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decisions D1 and D3. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(Self::ceiling()).expect("the ceiling is not above itself")
    }

    /// Returns the ceiling that the declared tier of this shape states.
    ///
    /// The value is a constant of the type. A caller reads it without a
    /// world and without a population.
    #[must_use]
    pub const fn ceiling() -> u32 {
        match <Self as Shape>::TIER.population_ceiling() {
            Some(ceiling) => ceiling,
            // The character tier states a ceiling, so this arm is
            // unreachable. A `match` states that fact where the compiler
            // can hold it, rather than a comment where it cannot.
            None => u32::MAX,
        }
    }

    /// Returns the tier that this shape declares.
    #[must_use]
    pub const fn tier() -> EntityTier {
        <Self as Shape>::TIER
    }

    /// Builds an arena that opens at most `capacity` slots.
    ///
    /// **The ceiling is checked here, when a caller builds the arena, and
    /// never on a later call.** A check on the current count would let a
    /// script that works against a small world fail against a large one,
    /// far from its cause.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the capacity is above the ceiling that the
    /// declared tier states.
    ///
    /// # References
    ///
    /// [^1]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decisions D2 and D3. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
    pub fn with_capacity(capacity: u32) -> Result<Self, CharacterError> {
        let ceiling = Self::ceiling();
        if capacity > ceiling {
            return Err(CharacterError::CapacityAboveCeiling {
                asked: capacity,
                ceiling,
                tier: Self::tier(),
            });
        }
        Ok(Self {
            capacity,
            generations: Vec::new(),
            live: Vec::new(),
            factions: Vec::new(),
            births: Vec::new(),
            renown: Vec::new(),
            sexes: Vec::new(),
            descent_of_slot: Vec::new(),
            descent: Descent::new(),
            free: VecDeque::new(),
            live_count: 0,
            retired_count: 0,
        })
    }

    /// Returns the largest number of slots that the arena opens.
    #[must_use]
    pub const fn capacity(&self) -> u32 {
        self.capacity
    }

    /// Returns the number of live characters.
    #[must_use]
    pub const fn len(&self) -> u32 {
        self.live_count
    }

    /// Reports whether the arena holds no live character.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.live_count == 0
    }

    /// Returns the number of slots that the arena has opened.
    ///
    /// The count never falls.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    #[must_use]
    pub fn slot_count(&self) -> u32 {
        self.generations.len() as u32
    }

    /// Returns the number of slots that the arena has retired.
    #[must_use]
    pub const fn retired_count(&self) -> u32 {
        self.retired_count
    }

    /// Creates a character and returns their identity.
    ///
    /// A new character holds a renown of zero. That is a real state and not
    /// an absent one.[^1]
    ///
    /// The arena takes the oldest free slot. It never takes the newest,
    /// because last-in first-out reuse gives one slot every generation
    /// increment and wears it out early.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the arena holds no free slot, or when the
    /// faction is at or above the ceiling.
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-043. `docs/FINDINGS.md`
    /// [^2]: ADR-0014, entity identity is an index plus a generation, decision D4. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn create(
        &mut self,
        seed: u64,
        faction: FactionId,
        birth: Tick,
    ) -> Result<Entity, CharacterError> {
        let entity = self.mint(faction, birth)?;
        let id = self.descent.record(entity, Parents::NONE)?;
        self.descent_of_slot[entity.index() as usize] = id.birth_order();
        // The character already holds an identity, so the draw keys on it. A
        // birth cannot key on the child, because the child holds no identity
        // when the draw happens.[^1]
        //
        // [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
        let draw = rng::draw_below(
            seed,
            rng::SYSTEM_CHARACTER,
            birth.0,
            entity.to_bits(),
            DRAW_FOUNDER_SEX,
            SEX_OPTIONS,
        );
        self.sexes[entity.index() as usize] = Sex::from_draw(draw).to_column();
        Ok(entity)
    }

    /// Bears a child of two characters and returns the identity of the
    /// child.
    ///
    /// The child takes the faction of its mother. It records both parents,
    /// and the record of descent keeps those edges after either parent is
    /// gone.[^1]
    ///
    /// Both parents must be alive. The record of descent outlives a
    /// character, so a caller reads a dead parent through a living child.
    /// It cannot name one as a parent of a new child.
    ///
    /// The sex draw keys on the mother and on the number of children she
    /// has borne on this tick, because the child holds no identity when the
    /// draw happens.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when either parent is gone, when the two parents
    /// are one character, when the arena holds no free slot, or when the
    /// record of descent is full.
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-003. `docs/DECISIONS.md`
    /// [^2]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
    pub fn bear(
        &mut self,
        seed: u64,
        mother: Entity,
        father: Entity,
        birth: Tick,
    ) -> Result<Entity, CharacterError> {
        if mother == father {
            return Err(CharacterError::ParentsAreOneCharacter(mother));
        }
        let mother_id = self
            .descent_id(mother)
            .ok_or(CharacterError::ParentIsGone(mother))?;
        let father_id = self
            .descent_id(father)
            .ok_or(CharacterError::ParentIsGone(father))?;
        let faction = self
            .faction(mother)
            .ok_or(CharacterError::ParentIsGone(mother))?;
        let sequence = self.descent.take_birth_sequence(mother_id, birth);
        let entity = self.mint(faction, birth)?;
        let id = self.descent.record(
            entity,
            Parents {
                mother: Some(mother_id),
                father: Some(father_id),
            },
        )?;
        self.descent_of_slot[entity.index() as usize] = id.birth_order();
        let draw = rng::draw_below(
            seed,
            rng::SYSTEM_CHARACTER,
            birth.0,
            mother.to_bits(),
            sequence,
            SEX_OPTIONS,
        );
        self.sexes[entity.index() as usize] = Sex::from_draw(draw).to_column();
        Ok(entity)
    }

    /// Takes a slot, writes the slot columns, and returns the identity.
    ///
    /// The record of descent is checked before the slot is taken, so a full
    /// record refuses the creation and leaves no slot half written.
    fn mint(&mut self, faction: FactionId, birth: Tick) -> Result<Entity, CharacterError> {
        if faction.0 >= FACTION_CEILING {
            return Err(CharacterError::FactionAboveCeiling(faction));
        }
        if self.descent.len() >= DESCENT_CEILING {
            return Err(CharacterError::Descent(DescentError::RecordFull));
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
        self.factions[index] = faction;
        self.births[index] = birth;
        self.renown[index] = Fix32::ZERO;
        self.live_count += 1;
        Ok(Entity::new(slot, self.generations[index])
            .expect("a generation of one or more makes the identity non-zero"))
    }

    /// Opens one new slot and returns its index.
    fn open_slot(&mut self) -> Result<u32, CharacterError> {
        let slot = self.slot_count();
        if slot >= self.capacity {
            return Err(CharacterError::ArenaFull);
        }
        self.generations.push(NO_GENERATION);
        self.live.push(0);
        self.factions.push(FactionId(0));
        self.births.push(Tick(0));
        self.renown.push(Fix32::ZERO);
        self.sexes.push(Sex::Female.to_column());
        self.descent_of_slot.push(0);
        Ok(slot)
    }

    /// Removes a character and reports whether it removed one.
    ///
    /// A stale identity removes nothing and returns `false`. The arena
    /// reports no error for it, because the caller either handles the
    /// absent character or skips them.[^1]
    ///
    /// The generation advances here, at the free, and not at the next
    /// creation. The identity of a character who is gone is therefore
    /// invalid at the moment they are lost, so the character created next
    /// in that slot never answers to it.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    /// [^2]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn remove(&mut self, entity: Entity) -> bool {
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
        self.live_count -= 1;
        if self.generations[index] == LAST_GENERATION {
            // The generation cannot advance, so the slot never returns. One
            // leaked slot beats two characters that share one identity.
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
    /// generation in the slot column. A mismatch means the character is
    /// gone, and a dead identity resolves to nothing.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    /// Returns the generation that one slot carries.
    ///
    /// A slot that carries no identity returns zero, and zero is never the
    /// generation of a live character.[^1] A boundary that refuses a stale
    /// identity reports this value, so a caller reads what the arena holds
    /// rather than guessing.
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

    /// Reports whether the identity names a live character.
    #[must_use]
    pub fn contains(&self, entity: Entity) -> bool {
        self.slot_of(entity).is_some()
    }

    /// Returns the faction of a character, or `None` when the identity is
    /// dead.
    #[must_use]
    pub fn faction(&self, entity: Entity) -> Option<FactionId> {
        let slot = self.slot_of(entity)?;
        Some(self.factions[slot as usize])
    }

    /// Returns the tick that a character was created on.
    ///
    /// Returns `None` when the identity is dead.
    #[must_use]
    pub fn birth(&self, entity: Entity) -> Option<Tick> {
        let slot = self.slot_of(entity)?;
        Some(self.births[slot as usize])
    }

    /// Returns the renown of a character, or `None` when the identity is
    /// dead.
    ///
    /// A renown of zero is a real state and not an absent one.[^1]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-043. `docs/FINDINGS.md`
    #[must_use]
    pub fn renown(&self, entity: Entity) -> Option<Fix32> {
        let slot = self.slot_of(entity)?;
        Some(self.renown[slot as usize])
    }

    /// Returns the sex of a character, or `None` when the identity is dead.
    ///
    /// The sex lives in the slot columns, so a death releases it.[^1]
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-003. `docs/DECISIONS.md`
    #[must_use]
    pub fn sex(&self, entity: Entity) -> Option<Sex> {
        let slot = self.slot_of(entity)?;
        Some(if self.sexes[slot as usize] == 0 {
            Sex::Female
        } else {
            Sex::Male
        })
    }

    /// Returns the record of descent.
    ///
    /// The record is append-only and it holds every character the arena has
    /// created. A caller reads a character that is gone through it.[^1]
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-003. `docs/DECISIONS.md`
    #[must_use]
    pub const fn descent(&self) -> &Descent {
        &self.descent
    }

    /// Returns the row of the record of descent that names a character.
    ///
    /// Returns `None` when the identity is dead. The resolution compares the
    /// generation in the identity against the generation in the slot, so an
    /// identity that names a character who is gone never reads the descent
    /// of the character created next in that slot.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    #[must_use]
    pub fn descent_id(&self, entity: Entity) -> Option<DescentId> {
        let slot = self.slot_of(entity)?;
        self.descent
            .id_at(self.descent_of_slot[slot as usize])
            .filter(|id| self.descent.born_as(*id) == Some(entity))
    }

    /// Returns the two parents of a living character.
    ///
    /// Returns `None` when the identity is dead. Returns a pair of absent
    /// parents when the character founds a line. The world invents no
    /// parent.[^1]
    ///
    /// # References
    ///
    /// [^1]: Blockers register, BLK-011. `docs/BLOCKERS.md`
    #[must_use]
    pub fn parents(&self, entity: Entity) -> Option<Parents> {
        self.descent.parents(self.descent_id(entity)?)
    }

    /// Returns the house that a character belongs to.
    ///
    /// A house is the group of characters that share a patrilineal founder,
    /// and its identifier is the descent identity of that founder. A
    /// character takes the house of its father at birth. A character with no
    /// father founds a house of its own.
    ///
    /// Returns `None` when the identity is dead. The house of a character
    /// who is gone stays in the record of descent, and a caller reads it
    /// through the descent identity rather than through the slot.
    #[must_use]
    pub fn house(&self, entity: Entity) -> Option<HouseId> {
        self.descent.house_of(self.descent_id(entity)?)
    }

    /// Rebuilds the Euler interval labels over the father forest.
    ///
    /// The arena runs this pass rather than labelling on every birth. A
    /// birth after the pass leaves the new character unlabelled, and a query
    /// about an unlabelled character answers nothing rather than answering
    /// from a stale label.[^1]
    ///
    /// # References
    ///
    /// [^1]: The character graph and inheritance, section 3.4. `docs/research/reports/14-character-graph-and-inheritance.md`
    pub fn relabel_lines(&mut self) {
        self.descent.relabel();
    }

    /// Reports whether one character is a patrilineal ancestor of another.
    ///
    /// The answer is two integer comparisons against the Euler labels, so it
    /// costs the same at any depth of line.
    ///
    /// Returns `None` when either identity is dead, or when either character
    /// was born after the last relabel.
    #[must_use]
    pub fn is_patrilineal_ancestor(&self, ancestor: Entity, of: Entity) -> Option<bool> {
        self.descent
            .is_patrilineal_ancestor(self.descent_id(ancestor)?, self.descent_id(of)?)
    }

    /// Makes a character the founder of a house and moves its line into it.
    ///
    /// This is the cadet split. The character and every patrilineal
    /// descendant of it leave the house they were in. The pass reads one
    /// contiguous span of the Euler order and writes the house column at the
    /// rows that span names.
    ///
    /// Returns the number of characters that the pass wrote, or `None` when
    /// the identity is dead or the character was born after the last
    /// relabel.
    pub fn found_house(&mut self, entity: Entity) -> Option<u32> {
        let id = self.descent_id(entity)?;
        self.descent.found_house_at(id)
    }

    /// Reports whether a line has ended.
    ///
    /// A line has ended when the character is gone and no descendant of the
    /// character is alive. A line that holds one living member has not
    /// ended.
    ///
    /// Returns `true` when the record holds no such row, because a line
    /// that never started holds nobody.
    #[must_use]
    pub fn line_ended(&self, id: DescentId) -> bool {
        if self.is_alive(id) {
            return false;
        }
        !self
            .descent
            .descendants(id)
            .iter()
            .any(|heir| self.is_alive(*heir))
    }

    /// Reports whether the character that a row names is alive.
    ///
    /// The row holds the identity that the arena minted, and that identity
    /// carries the generation of the slot at the birth. A slot that was
    /// reused holds a later generation, so the identity of a character who
    /// is gone never resolves to the character in the slot now.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    #[must_use]
    pub fn is_alive(&self, id: DescentId) -> bool {
        self.descent
            .born_as(id)
            .is_some_and(|entity| self.contains(entity))
    }

    /// Writes the renown of a character and reports whether it wrote.
    ///
    /// Returns `false` when the identity is dead. The caller handles the
    /// absent character or skips them.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn set_renown(&mut self, entity: Entity, renown: Fix32) -> bool {
        let Some(slot) = self.slot_of(entity) else {
            return false;
        };
        self.renown[slot as usize] = renown;
        true
    }

    /// Returns the live characters in slot order.
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

    /// Returns the whole faction column.
    ///
    /// The column holds one entry for each slot, live or not. A caller that
    /// wants the live characters walks the identities instead.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    #[must_use]
    pub fn faction_column(&self) -> &[FactionId] {
        &self.factions
    }

    /// Returns the whole birth column.
    #[must_use]
    pub fn birth_column(&self) -> &[Tick] {
        &self.births
    }

    /// Returns the whole renown column.
    #[must_use]
    pub fn renown_column(&self) -> &[Fix32] {
        &self.renown
    }

    /// Absorbs the character columns into the state hash.
    ///
    /// The hash covers every byte that decides a later frame. It therefore
    /// covers the generation of each slot and the free queue, because both
    /// decide which slot the next creation takes.[^1]
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
            .write(bytemuck::cast_slice(&self.factions))
            .write(bytemuck::cast_slice(&self.births))
            .write(bytemuck::cast_slice(&self.renown))
            .write(&self.sexes)
            .write(&self.live);
        for generation in &self.generations {
            hash = hash.write(&generation.to_le_bytes());
        }
        for slot in &self.free {
            hash = hash.write(&slot.to_le_bytes());
        }
        for row in &self.descent_of_slot {
            hash = hash.write(&row.to_le_bytes());
        }
        self.descent.hash_into(hash)
    }

    /// Reports whether the arena holds its invariants.
    ///
    /// The check compares the columns against each other. One value that
    /// lives in two places needs a check that fails when the copies
    /// disagree.[^1]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-040. `docs/FINDINGS.md`
    #[must_use]
    pub fn check_invariants(&self) -> bool {
        let slots = self.generations.len();
        if self.live.len() != slots
            || self.factions.len() != slots
            || self.births.len() != slots
            || self.renown.len() != slots
            || self.sexes.len() != slots
            || self.descent_of_slot.len() != slots
        {
            return false;
        }
        if !self.descent.check_invariants() {
            return false;
        }
        // The capacity is the second declaration of the tier ceiling. The
        // constructor refuses a larger one, and this check fails when the
        // two copies disagree.[^1]
        //
        // [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
        if self.capacity > Self::ceiling() {
            return false;
        }
        if slots > self.capacity as usize {
            return false;
        }
        if self.live.iter().filter(|live| **live == 1).count() != self.live_count as usize {
            return false;
        }
        for slot in 0..slots {
            if self.live[slot] == 1 {
                if self.generations[slot] == NO_GENERATION {
                    return false;
                }
                if self.factions[slot].0 >= FACTION_CEILING {
                    return false;
                }
                if self.sexes[slot] > 1 {
                    return false;
                }
                // The slot column and the record of descent hold the same
                // identity a second time. This check fails when the two
                // copies disagree, which is what a creation that reused the
                // descent row of the character before it would do.[^1]
                //
                // [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
                let Some(entity) = Entity::new(slot as u32, self.generations[slot]) else {
                    return false;
                };
                let Some(id) = self.descent.id_at(self.descent_of_slot[slot]) else {
                    return false;
                };
                if self.descent.born_as(id) != Some(entity) {
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
    //! it would need four thousand million creations. The test therefore
    //! sets the generation here.
    //!
    //! # References
    //!
    //! [^1]: ADR-0014, entity identity is an index plus a generation, decision D5. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`

    use super::*;

    /// The tier is a constant of the type, so it resolves in a constant
    /// context. A count could not. This item fails to compile if the tier
    /// ever becomes a run-time value.[^1]
    ///
    /// [^1]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D2. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
    const TIER_AT_COMPILE_TIME: EntityTier = <CharacterArena as Shape>::TIER;

    #[test]
    fn the_shape_declares_the_character_tier_at_compile_time() {
        assert_eq!(TIER_AT_COMPILE_TIME, EntityTier::Character);
        assert_eq!(CharacterArena::tier(), EntityTier::Character);
    }

    #[test]
    fn a_slot_at_the_last_generation_retires_on_the_loss() {
        let mut arena = CharacterArena::new();
        let first = arena
            .create(0, FactionId(0), Tick(0))
            .expect("the creation must succeed");
        arena.generations[0] = LAST_GENERATION;
        let aged = Entity::new(0, LAST_GENERATION).expect("the identity is not zero");
        assert!(!arena.contains(first));
        assert!(arena.remove(aged));
        assert_eq!(arena.retired_count(), 1);
        assert!(arena.free.is_empty());
        assert!(arena.check_invariants());
    }

    #[test]
    fn a_retired_slot_never_returns_to_use() {
        let mut arena = CharacterArena::new();
        arena
            .create(0, FactionId(0), Tick(0))
            .expect("the creation must succeed");
        arena.generations[0] = LAST_GENERATION;
        let aged = Entity::new(0, LAST_GENERATION).expect("the identity is not zero");
        assert!(arena.remove(aged));

        let next = arena
            .create(0, FactionId(0), Tick(0))
            .expect("the creation must succeed");
        assert_ne!(next.index(), 0, "a retired slot must never return");
        assert_eq!(arena.slot_count(), 2);
        assert!(arena.check_invariants());
    }

    #[test]
    fn a_removal_of_a_slot_that_is_not_live_changes_nothing() {
        // The public interface cannot reach this state, because an identity
        // resolves only while its generation matches and the arena marks a
        // slot live before it hands out the identity. The guard exists
        // because the argument that it holds runs across three functions,
        // and the failure it prevents is a count that wraps rather than a
        // panic.
        let mut arena = CharacterArena::new();
        let entity = arena
            .create(0, FactionId(0), Tick(0))
            .expect("the creation must succeed");
        arena.live[0] = 0;
        assert!(!arena.remove(entity));
        assert_eq!(arena.len(), 1, "the live count must not wrap");
    }

    #[test]
    fn a_free_queue_that_holds_one_slot_twice_fails_the_check() {
        let mut arena = CharacterArena::new();
        let entity = arena
            .create(0, FactionId(0), Tick(0))
            .expect("the creation must succeed");
        assert!(arena.remove(entity));
        assert!(arena.check_invariants());
        arena.free.push_back(0);
        assert!(!arena.check_invariants());
    }

    #[test]
    fn a_capacity_above_the_ceiling_fails_the_check() {
        // The constructor refuses this capacity, so the state is only
        // reachable from inside the module. The check exists because the
        // capacity is a second copy of the ceiling.
        let mut arena = CharacterArena::new();
        arena.capacity = CharacterArena::ceiling() + 1;
        assert!(!arena.check_invariants());
    }

    #[test]
    fn a_short_column_fails_the_check() {
        let mut arena = CharacterArena::new();
        arena
            .create(0, FactionId(0), Tick(0))
            .expect("the creation must succeed");
        assert!(arena.check_invariants());
        arena.renown.pop();
        assert!(!arena.check_invariants());
    }

    #[test]
    fn an_arena_that_holds_no_slot_refuses_a_creation() {
        let mut arena =
            CharacterArena::with_capacity(0).expect("a capacity of zero is below the ceiling");
        assert_eq!(
            arena.create(0, FactionId(0), Tick(0)),
            Err(CharacterError::ArenaFull)
        );
        assert!(arena.check_invariants());
    }
}
