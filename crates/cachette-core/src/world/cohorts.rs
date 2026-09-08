//! The cohort table, the need it reads, and the pass that reaps the dead.
//!
//! A cohort holds the units that share a condition. The need rule says what a
//! unit must receive. The reap pass removes the units the table marked dead.
//! The table and the pass that empties it sit together.

use super::errors::StepError;
use super::World;
use crate::cohort::{
    self, CohortTable, DrawLedger, NeedCondition, NeedRule, SiteRationed, UnitStarved,
};
use crate::types::Entity;

impl World {
    /// Returns the rule that says what a unit needs.
    #[must_use]
    pub const fn need_rule(&self) -> NeedRule {
        self.need_rule
    }

    /// Sets the rule that says what a unit needs.
    ///
    /// The rule carries the bound at which a shortage ends a unit, so the
    /// bound is a parameter of the world and never a constant of a
    /// kernel.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D3. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
    pub const fn set_need_rule(&mut self, rule: NeedRule) {
        self.need_rule = rule;
    }

    /// Returns the condition that a shortage has put a unit in.
    ///
    /// Returns `None` when the identity is dead.[^1] The condition is a
    /// name, and it is what a watcher reads. A watcher that read the
    /// accumulator would hold the bound of the rule a second time.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    /// [^2]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub fn unit_condition(&self, entity: Entity) -> Option<NeedCondition> {
        self.soldiers
            .deficit(entity)
            .map(|deficit| self.need_rule.condition(deficit))
    }

    /// Returns the units that a shortage ended at the last scan, in slot
    /// order.
    #[must_use]
    pub fn starved_log(&self) -> &[UnitStarved] {
        &self.starved_log
    }

    /// Returns the starved log as bytes.
    ///
    /// The thread-count equivalence test compares this slice byte for
    /// byte.[^1] The cast is safe because the event type is plain data.
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[must_use]
    pub fn starved_log_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.starved_log)
    }

    /// Returns the cohorts of the last consumption pass.
    ///
    /// The table is derived from the home column of the units, and the pass
    /// derives it again on every application.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D2. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
    #[must_use]
    pub const fn cohorts(&self) -> &CohortTable {
        &self.cohorts
    }

    /// Returns every draw that has run since the world was built.
    #[must_use]
    pub const fn draw_ledger(&self) -> DrawLedger {
        self.draw_ledger
    }

    /// Returns the sites that could not serve every cohort at the last
    /// draw, in slot order.
    #[must_use]
    pub fn rationed_log(&self) -> &[SiteRationed] {
        &self.rationed_log
    }

    /// Returns the rationed log as bytes.
    ///
    /// The thread-count equivalence test compares this slice byte for
    /// byte.[^1] The cast is safe because the event type is plain data.
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[must_use]
    pub fn rationed_log_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.rationed_log)
    }

    /// Ends every unit that the shortage marked, in ascending slot order.
    ///
    /// The mark pass writes one bit for each unit into a dense plane, and
    /// each thread owns disjoint words of it, so the plane is the same at
    /// any thread count. The scan is ordered all the same: the deaths apply
    /// in the order it finds them, and a free slot returns to the queue in
    /// that order.[^1]
    ///
    /// A death advances the generation of the slot, so the identity of the
    /// dead unit never resolves to the unit spawned next in that slot.[^2]
    /// The world removes the unit through its own despawn, which accounts
    /// for what the unit carried, so conservation still balances.[^3]
    ///
    /// The cohort table summarises the home column, and a death changes that
    /// column, so the table is derived again here. A summary left stale
    /// would state a headcount that no unit backs.[^4]
    ///
    /// # Errors
    ///
    /// Returns an error when the caller asks for zero threads.
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^2]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    /// [^3]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D5. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    /// [^4]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    pub(super) fn reap(&mut self, threads: usize) -> Result<(), StepError> {
        self.starved_log.clear();
        if !self.schedule.due(self.tick) {
            return Ok(());
        }
        cohort::mark_starved(
            self.need_rule,
            self.soldiers.deficit_column(),
            self.soldiers.live_column(),
            &mut self.death_plane,
            threads,
        )?;
        let order = cohort::starved_order(&self.death_plane, threads)?;
        if order.is_empty() {
            return Ok(());
        }
        let tick = self.tick;
        for slot in order {
            let index = slot as usize;
            let deficit = self.soldiers.deficit_column()[index];
            let generation = self.soldiers.generation_of(slot);
            let unit = Entity::new(slot, generation)
                .expect("a marked slot is live, so it holds a generation of one or more");
            self.starved_log
                .push(UnitStarved::new(tick, unit.to_bits(), deficit));
            let ended = self.despawn_soldier(unit);
            debug_assert!(ended, "a marked slot holds a live unit");
        }
        self.cohorts.rebuild(
            self.soldiers.home_column(),
            self.soldiers.faction_column(),
            self.soldiers.live_column(),
            self.settlements.slot_count(),
        );
        Ok(())
    }
}
