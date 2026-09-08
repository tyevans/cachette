//! The conversion of a unit from one faction to another.
//!
//! A conversion moves a unit across a faction boundary. The verb, the pass
//! that applies the results and the log sit together, because a conversion is
//! not visible anywhere else.

use super::errors::{ConvertError, StepError};
use super::World;
use crate::conversion::{self, Convert, UnitConverted};
use crate::types::{Entity, FactionId};

impl World {
    /// Returns the units that changed faction in the last step.
    ///
    /// The log covers the last step alone. It holds one entry for each unit
    /// that changed hands, whether the field converted it or the control
    /// plane did. **This is what a god reads to see conversion happen.** A
    /// mechanic that a player cannot observe is a mechanic that a player
    /// cannot play.[^1]
    ///
    /// The entries lie in ascending arena slot order, which is a stable key
    /// and never a thread completion order.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0134, a god reads conversion as an event log and as the faction counts it already reads, decision D1. `docs/adrs/draft/adr-0134-a-god-reads-conversion-as-an-event-log.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn converted_log(&self) -> &[UnitConverted] {
        &self.converted_log
    }

    /// Returns the conversion log as bytes.
    ///
    /// The event type is plain data with an explicit layout, so the bytes are
    /// the events.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
    #[must_use]
    pub fn converted_log_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.converted_log)
    }

    /// Changes the faction of every unit that the identities name.
    ///
    /// **The set is all or nothing.** Every identity resolves and the faction
    /// is checked before anything changes. A partly applied set would leave
    /// the caller with no way to say what happened.[^1]
    ///
    /// A unit that already belongs to the faction is left alone, and it emits
    /// no event. The verb is therefore idempotent over one set.
    ///
    /// The engine holds no rule that decides when a control plane may call
    /// this. It is the deliberate route, beside the field that converts a
    /// unit where another faction leads.[^2]
    ///
    /// The call clears the orders of every unit it converts, and it changes
    /// the faction of the character that a converted unit carries. The
    /// reasoning is in the record.[^3]
    ///
    /// # Errors
    ///
    /// Returns [`ConvertError::NoSuchFaction`] when the number names no
    /// faction of this world, and [`ConvertError::DeadUnit`] when an identity
    /// names no live soldier.
    ///
    /// # References
    ///
    /// [^1]: ADR-0010, Python is a control plane, and it never touches an entity one at a time. `docs/adrs/REGISTRY.md`
    /// [^2]: ADR-0133, a unit converts to the faction that leads the influence field at its cell, decision D4. `docs/adrs/draft/adr-0133-a-unit-converts-to-the-faction-that-leads-the-field.md`
    /// [^3]: ADR-0132, conversion changes the faction of a unit and adds no second allegiance, decisions D2, D3 and D4. `docs/adrs/draft/adr-0132-conversion-changes-the-faction-of-a-unit.md`
    pub fn convert_units(
        &mut self,
        units: &[Entity],
        faction: FactionId,
    ) -> Result<(), ConvertError> {
        if faction.0 >= self.config.faction_count {
            return Err(ConvertError::NoSuchFaction(faction.0));
        }
        for unit in units {
            if self.soldiers.slot_of(*unit).is_none() {
                return Err(ConvertError::DeadUnit(*unit));
            }
        }
        let marks = conversion::marks_for_set(&self.soldiers, units, faction);
        self.apply_converts(&marks);
        // A faction change raises the arena revision, so the verb leaves the
        // world readable before it returns.[^2]
        //
        // [^2]: Findings register, FND-647. `docs/FINDINGS.md`
        self.leave_the_world_readable();
        Ok(())
    }

    /// Converts every unit that the influence field takes this frame.
    ///
    /// The pass marks in parallel and applies in one ascending scan of the
    /// slots, so the changes never follow a thread completion order.[^1]
    ///
    /// It runs a fixed amount of work for the world it is given. It holds no
    /// convergence test and no time budget.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D3. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    pub(super) fn convert(&mut self, threads: usize) -> Result<(), StepError> {
        self.converted_log.clear();
        // The structure the pass walks must describe the world it reads. The
        // reap of this frame has already passed its barrier, and nothing
        // between that barrier and here moves or removes a unit.
        self.bridge.describes(&self.soldiers)?;
        let mut marks = core::mem::take(&mut self.convert_marks);
        let outcome = conversion::resolve(
            conversion::DrawKey {
                seed: self.config.seed,
                tick: self.tick,
            },
            &self.relations,
            &self.soldiers,
            &self.bridge,
            &self.influence,
            self.pyramid.layout(),
            &mut marks,
            threads,
        );
        if outcome.is_ok() {
            self.apply_converts(&marks);
        }
        self.convert_marks = marks;
        outcome?;
        // A faction change raises the arena revision, so the derived unit
        // structure no longer describes the arena. The refresh here is what
        // keeps the whole-world invariant check strong on a frame that
        // converted somebody. It returns at once on every other frame.[^3]
        //
        // [^3]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
        self.refresh_bridge()?;
        Ok(())
    }

    /// Applies a list of marks in ascending slot order, and logs each one.
    ///
    /// **This is the one place that changes the faction of a unit.** The
    /// field pass and the control plane verb both come through here, so the
    /// two cannot disagree about what a conversion does.[^1]
    ///
    /// The arena moves its own per-faction count, and this call rebuilds the
    /// cohorts, which are the other total that follows the faction of a
    /// unit.[^2]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    /// [^2]: ADR-0132, conversion changes the faction of a unit and adds no second allegiance, decision D5. `docs/adrs/draft/adr-0132-conversion-changes-the-faction-of-a-unit.md`
    fn apply_converts(&mut self, marks: &[Convert]) {
        if marks.is_empty() {
            return;
        }
        let tick = self.tick;
        for mark in marks {
            let index = mark.slot as usize;
            let from = self.soldiers.faction_column()[index];
            let tile = self.soldiers.tile_column()[index];
            let generation = self.soldiers.generation_of(mark.slot);
            let Some(unit) = Entity::new(mark.slot, generation) else {
                continue;
            };
            if !self.soldiers.set_faction(unit, mark.faction) {
                continue;
            }
            // The orders end with the old faction. An order is a standing
            // instruction from the control plane that owned the unit, and a
            // unit that kept one would let a god steer the units of another
            // god.[^1]
            //
            // [^1]: ADR-0132, conversion changes the faction of a unit and adds no second allegiance, decision D3. `docs/adrs/draft/adr-0132-conversion-changes-the-faction-of-a-unit.md`
            self.soldiers.set_gather_order(unit, None);
            self.soldiers.set_build_order(unit, None);
            self.soldiers.set_sent(unit, None);
            // The person goes with the body. A body of one faction carrying a
            // person of another would be one allegiance in two places.[^2]
            //
            // [^2]: ADR-0132, conversion changes the faction of a unit and adds no second allegiance, decision D4. `docs/adrs/draft/adr-0132-conversion-changes-the-faction-of-a-unit.md`
            if let Some(Some(character)) = self.soldiers.character_of(unit) {
                self.characters.set_faction(character, mark.faction);
            }
            self.converted_log.push(UnitConverted::new(
                tick,
                unit.to_bits(),
                tile,
                from,
                mark.faction,
            ));
            // The faction that lost the unit lowers toward the faction that
            // took it. The marks are in slot order, so the writes are
            // too.[^3]
            //
            // [^3]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D3. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
            self.relations
                .on_units_converted(tick, from, mark.faction, 1);
        }
        // The cohorts are the units of one faction at one site, so a faction
        // change moves a unit from one row to another. A table left as it was
        // would hold a headcount that no unit answers to, and the invariant
        // check refuses that state.
        self.cohorts.rebuild(
            self.soldiers.home_column(),
            self.soldiers.faction_column(),
            self.soldiers.live_column(),
            self.settlements.slot_count(),
        );
    }
}
