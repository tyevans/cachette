//! The character of a unit, its descent, its renown and its promotion.
//!
//! A character carries a name, parents and a record of deeds. Renown grows
//! from deeds, and a promotion spends it. The whole life of one character
//! sits in one module, because each part reads the others.

use super::errors::{IdentityError, StepError};
use super::World;
use crate::character::{CharacterArena, CharacterError};
use crate::descent::{DescentId, Parents};
use crate::promotion::{self, UnitPromoted};
use crate::rates::{RateError, RateSchedule};
use crate::sim_math;
use crate::types::{Accum, Entity, FactionId, Fix32};

impl World {
    /// Returns the promotions of the last frame that ran the pass.
    ///
    /// The slice holds one row for each unit the pass promoted. It is empty
    /// on a frame the schedule does not name, and empty on a frame that
    /// promoted nobody. A reader cannot tell the two apart from the slice,
    /// and nothing needs to.
    #[must_use]
    pub fn promoted_log(&self) -> &[UnitPromoted] {
        &self.promoted_log
    }

    /// Returns the promotion log as bytes, for a caller across the boundary.
    ///
    /// The event is plain data with declared padding, so the bytes are the
    /// same on every run.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
    #[must_use]
    pub fn promoted_log_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.promoted_log)
    }

    /// Returns the character that a unit was promoted into.
    ///
    /// The outer option reports whether the unit is live. The inner one
    /// reports whether the unit carries a character. The answer names a
    /// living character, because a character that the arena no longer holds
    /// resolves to nothing.
    #[must_use]
    pub fn unit_character(&self, unit: Entity) -> Option<Option<Entity>> {
        let named = self.soldiers.character_of(unit)?;
        Some(named.filter(|character| self.characters.contains(*character)))
    }

    /// Returns what a unit has ever gathered, summed over every kind.
    ///
    /// Returns `None` when the identity is dead. The value never falls while
    /// the unit lives.
    #[must_use]
    pub fn unit_deeds(&self, unit: Entity) -> Option<u64> {
        self.soldiers.deeds(unit)
    }

    /// Returns the deeds at which a unit becomes eligible for promotion.
    #[must_use]
    pub fn deed_threshold(&self) -> u64 {
        self.soldiers.deed_threshold()
    }

    /// Sets the deeds at which a unit becomes eligible for promotion.
    ///
    /// The threshold is a content parameter and not a budget. A caller raises
    /// it to make a person rarer and lowers it to make one common.
    pub fn set_deed_threshold(&mut self, threshold: u64) {
        self.soldiers.set_deed_threshold(threshold);
    }

    /// Returns the most characters that one promotion pass may create.
    #[must_use]
    pub const fn promotion_budget(&self) -> u32 {
        self.promotion_budget
    }

    /// Sets the most characters that one promotion pass may create.
    ///
    /// A budget of zero promotes nobody. The arena ceiling still binds above
    /// this, so a budget larger than the headroom takes the headroom.
    pub const fn set_promotion_budget(&mut self, budget: u32) {
        self.promotion_budget = budget;
    }

    /// Returns the schedule that the promotion pass runs on.
    #[must_use]
    pub const fn character_schedule(&self) -> RateSchedule {
        self.character_schedule
    }

    /// Sets the schedule that the promotion pass runs on.
    ///
    /// # Errors
    ///
    /// Returns an error when the period is zero or above the range.
    pub fn set_character_schedule(&mut self, period: u32, phase: u32) -> Result<(), RateError> {
        self.character_schedule =
            RateSchedule::new(period, phase).ok_or(RateError::PeriodOutsideRange(period))?;
        Ok(())
    }

    /// Returns the characters of the world.
    ///
    /// The living character is one of the four fixed entity shapes, and it
    /// has its own column set. It carries no tile position.[^1] The shape
    /// declares the character tier at the type, so a caller may walk the
    /// population.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
    /// [^2]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D1. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
    #[must_use]
    pub const fn characters(&self) -> &CharacterArena {
        &self.characters
    }

    /// Resolves the value of an identity back to the character it names.
    ///
    /// A caller outside this crate holds an identity as the value the engine
    /// gave. It cannot build one, and this is the only way back.[^1]
    ///
    /// The call compares the generation the value carries against the
    /// generation the arena holds for the slot. It refuses a mismatch, and it
    /// never returns the character who now holds the slot.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the value is not an identity, when the arena
    /// holds no such slot, or when the slot holds a later generation.
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decisions D2 and D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    /// [^2]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn resolve_character(&self, identity: u64) -> Result<Entity, IdentityError> {
        let entity = Entity::from_bits(identity).ok_or(IdentityError::NotAnIdentity)?;
        let slot = entity.index();
        if slot >= self.characters.slot_count() {
            return Err(IdentityError::NoSuchSlot { slot });
        }
        if self.characters.contains(entity) {
            return Ok(entity);
        }
        Err(IdentityError::Stale {
            slot,
            given: entity.generation(),
            held: self.characters.generation_of(slot),
        })
    }

    /// Creates a character in the world and returns their identity.
    ///
    /// The character is born on the current tick of the world.
    ///
    /// # Errors
    ///
    /// Returns an error when the arena holds no free slot, or when the
    /// faction is one the world does not have.
    pub fn create_character(&mut self, faction: FactionId) -> Result<Entity, CharacterError> {
        // The arena refuses a faction above the project ceiling. This world
        // holds a faction count of its own, which is at most that ceiling,
        // and a character of a faction the world does not have is a caller
        // mistake rather than a storage one.
        if faction.0 >= self.config.faction_count.max(1) {
            return Err(CharacterError::FactionAboveCeiling(faction));
        }
        self.characters.create(self.config.seed, faction, self.tick)
    }

    /// Bears a child of two characters and returns the identity of the
    /// child.
    ///
    /// The child is born on the current tick of the world. It takes the
    /// faction of its mother, and it records both parents. The record of
    /// descent keeps those edges after either parent is gone, so a watcher
    /// reads a dead parent through a living child.[^1]
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
    pub fn bear_character(
        &mut self,
        mother: Entity,
        father: Entity,
    ) -> Result<Entity, CharacterError> {
        self.characters
            .bear(self.config.seed, mother, father, self.tick)
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
    pub fn character_parents(&self, entity: Entity) -> Option<Parents> {
        self.characters.parents(entity)
    }

    /// Returns every ancestor of a living character, in ascending birth
    /// order.
    ///
    /// Returns an empty list when the identity is dead and when the
    /// character founds a line. The order is explicit and it is the same on
    /// every run.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn character_ancestors(&self, entity: Entity) -> Vec<DescentId> {
        let Some(id) = self.characters.descent_id(entity) else {
            return Vec::new();
        };
        self.characters.descent().ancestors(id)
    }

    /// Returns every descendant of a living character, in ascending birth
    /// order.
    ///
    /// Returns an empty list when the identity is dead and when the
    /// character has no child.
    #[must_use]
    pub fn character_descendants(&self, entity: Entity) -> Vec<DescentId> {
        let Some(id) = self.characters.descent_id(entity) else {
            return Vec::new();
        };
        self.characters.descent().descendants(id)
    }

    /// Returns the relation between two characters.
    ///
    /// The value is Wright's coefficient of relationship. A parent and a
    /// child give one half. Two characters with no ancestor in common give
    /// zero, and a character who founds a line therefore stands at zero to
    /// everybody.[^1]
    ///
    /// The value is a Q16.16 fixed-point number and it is exact. Every step
    /// of the recursion halves a value, so no step rounds.[^2]
    ///
    /// Returns zero when either identity is dead.
    ///
    /// # References
    ///
    /// [^1]: Blockers register, BLK-011. `docs/BLOCKERS.md`
    /// [^2]: The character graph and inheritance, section 3.6. `docs/research/reports/14-character-graph-and-inheritance.md`
    #[must_use]
    pub fn character_relation(&self, left: Entity, right: Entity) -> Fix32 {
        let (Some(left), Some(right)) = (
            self.characters.descent_id(left),
            self.characters.descent_id(right),
        ) else {
            return Fix32::ZERO;
        };
        self.characters.descent().relation(left, right)
    }

    /// Removes a character and reports whether it removed one.
    ///
    /// A stale identity removes nothing and returns `false`. The identity
    /// of a character who is gone never resolves again, so the character
    /// created next in that slot does not answer to it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn remove_character(&mut self, entity: Entity) -> bool {
        self.characters.remove(entity)
    }

    /// Writes the renown of a character and reports whether it wrote.
    ///
    /// Returns `false` when the identity is dead. A renown of zero is a
    /// real state, so a write of zero is a write.[^1]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-043. `docs/FINDINGS.md`
    pub fn set_character_renown(&mut self, entity: Entity, renown: Fix32) -> bool {
        self.characters.set_renown(entity, renown)
    }

    /// Promotes the units that earned it, on the character schedule.
    ///
    /// The budget is the headroom of the character arena: the ceiling of the
    /// declared tier less the characters alive now. The arena refuses a
    /// create beyond its capacity whatever this says, so the ceiling has one
    /// enforcement site and this is the cut at the rank.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0104, a soldier is promoted from a level that never falls, decision D4. `docs/adrs/draft/adr-0104-a-soldier-is-promoted-from-a-level-that-never-falls.md`
    pub(super) fn promote(&mut self, threads: usize) -> Result<(), StepError> {
        self.promoted_log.clear();
        if !self.character_schedule.due(self.tick) {
            return Ok(());
        }
        let headroom = CharacterArena::ceiling().saturating_sub(self.characters.len());
        let budget = self.promotion_budget.min(headroom);
        self.promoted_log = promotion::promote(
            &mut self.soldiers,
            &mut self.characters,
            self.config.seed,
            self.tick,
            budget,
            threads,
        )?;
        Ok(())
    }

    /// Gives the champion of each killer faction the renown its units earned.
    ///
    /// # Why this exists
    ///
    /// **This is the one source of renown in the engine.** A win path reads
    /// the renown column, and the standing reading reports it, and no pass
    /// wrote it. The path could therefore never fire and the reading never
    /// moved. A quantity that only a reader touches states a capability the
    /// engine does not have.[^1]
    ///
    /// # What it does
    ///
    /// The contest of this frame states, for each pair, which faction felled
    /// how many units of which other faction. The killer of each pair earns
    /// one share of renown for each unit it felled, and the share is a
    /// balance value.[^2]
    ///
    /// **The renown goes to one character and not to the faction.** The
    /// reader takes the highest renown among the live characters of a
    /// faction, so renown spread over every character would never reach the
    /// target and the source would stay inert. The champion of a faction is
    /// its live character with the highest renown, and a tie goes to the
    /// lowest identity.
    ///
    /// A faction with no live character earns nothing. Renown is a property
    /// of a person, and a faction that has promoted nobody has no person to
    /// carry it.
    ///
    /// # Determinism
    ///
    /// The scan visits the character arena in slot order and replaces the
    /// champion only on a strictly greater key, so the answer is a property
    /// of the arena and never of a thread.[^3] The gains are summed into
    /// 64-bit accumulators, which combine in any order, and every value is a
    /// fixed-point value or a whole number.[^4] The renown column already
    /// enters the state hash.
    ///
    /// # Cost
    ///
    /// The scan walks the character arena, whose ceiling the character tier
    /// declares, and never the unit population.[^5]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 3. `.agents/rules/recurring-defects.md`
    /// [^2]: Balance register, the renown target. `docs/reference/balance.md`
    /// [^3]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^4]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^5]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D3. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
    pub(super) fn award_renown(&mut self) {
        let factions = usize::from(self.config.faction_count.max(1));
        let mut earned = vec![Accum(0); factions];
        let mut any = false;
        for grievance in &self.grievances {
            let Some(slot) = earned.get_mut(usize::from(grievance.killer.0)) else {
                continue;
            };
            *slot = sim_math::combine(
                *slot,
                sim_math::scale_by_count(self.balance.renown_per_fell(), grievance.count),
            );
            any = true;
        }
        if !any {
            return;
        }
        // The champion of each faction: the live character with the highest
        // renown, and the lowest identity among equals.
        let mut champions: Vec<Option<(Fix32, u64, Entity)>> = vec![None; factions];
        for character in self.characters.iter() {
            let (Some(faction), Some(renown)) = (
                self.characters.faction(character),
                self.characters.renown(character),
            ) else {
                continue;
            };
            let Some(slot) = champions.get_mut(usize::from(faction.0)) else {
                continue;
            };
            let bits = character.to_bits();
            let better = match slot {
                Some((best, best_bits, _)) => (renown.0, *best_bits) > (best.0, bits),
                None => true,
            };
            if better {
                *slot = Some((renown, bits, character));
            }
        }
        for (index, gain) in earned.iter().copied().enumerate() {
            if gain.0 == 0 {
                continue;
            }
            let Some(Some((renown, _, champion))) = champions.get(index).copied() else {
                continue;
            };
            let raised = sim_math::add(renown, sim_math::narrow(gain));
            let wrote = self.characters.set_renown(champion, raised);
            debug_assert!(wrote, "the scan above found the character live");
        }
    }
}
