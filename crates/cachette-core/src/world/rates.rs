//! The production rate and the upkeep rate, and the passes that apply them.
//!
//! A rate says what a site makes or spends in one period. The table, the
//! ledger, the shortfall log and the two passes sit together, because a
//! shortfall is only readable against the rate that caused it.

use super::errors::StepError;
use super::World;
use crate::cohort;
use crate::rates::{RateError, RateLedger, RateSchedule, RateTable, SiteShortfall};
use crate::sim_math;
use crate::site::CommodityId;
use crate::types::{Accum, Entity, Fix32};

impl World {
    /// Returns the schedule that the site rates apply on.
    #[must_use]
    pub const fn economy_schedule(&self) -> RateSchedule {
        self.schedule
    }

    /// Sets the schedule that the site rates apply on.
    ///
    /// The interval is a parameter of the schedule. No kernel holds it as a
    /// constant, so a caller changes how often a store moves without
    /// touching the engine.[^1]
    ///
    /// A rate is what one tick earns, so raising the period does not raise
    /// what a site earns over a span of ticks.
    ///
    /// # Errors
    ///
    /// Returns an error when the period is zero, and when the period is
    /// above the range that the scaling multiply takes.
    ///
    /// # References
    ///
    /// [^1]: ADR-0062, production and upkeep are rates attached to a site, decision D4. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    pub fn set_economy_schedule(&mut self, period: u32, phase: u32) -> Result<(), RateError> {
        self.schedule =
            RateSchedule::new(period, phase).ok_or(RateError::PeriodOutsideRange(period))?;
        Ok(())
    }

    /// Returns the rate table of every site.
    #[must_use]
    pub const fn rates(&self) -> &RateTable {
        &self.rates
    }

    /// Returns every rate that has applied since the world was built.
    #[must_use]
    pub const fn rate_ledger(&self) -> RateLedger {
        self.rate_ledger
    }

    /// Returns the sites that could not pay at the last application.
    ///
    /// The log holds one event for each site and commodity that fell short,
    /// in slot order. It is empty on a tick that the schedule does not name.
    #[must_use]
    pub fn shortfall_log(&self) -> &[SiteShortfall] {
        &self.shortfall_log
    }

    /// Returns the shortfall log as bytes.
    ///
    /// The event type is plain data with declared padding, so the bytes are
    /// the same on every run and the determinism test compares them
    /// directly.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
    #[must_use]
    pub fn shortfall_log_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.shortfall_log)
    }

    /// Returns the production rate of a settlement, for one commodity.
    ///
    /// Returns `None` when the identity is dead, and `None` when the
    /// commodity is outside the set.
    #[must_use]
    pub fn production_rate(&self, entity: Entity, commodity: CommodityId) -> Option<Fix32> {
        let slot = self.settlements.slot_of(entity)?;
        self.rates.production(slot, commodity)
    }

    /// Returns the upkeep rate of a settlement, for one commodity.
    #[must_use]
    pub fn upkeep_rate(&self, entity: Entity, commodity: CommodityId) -> Option<Fix32> {
        let slot = self.settlements.slot_of(entity)?;
        self.rates.upkeep(slot, commodity)
    }

    /// Writes the production rate of a settlement, for one commodity.
    ///
    /// The rate is what one tick earns. The schedule scales it to the
    /// amount of one application, so raising the period does not raise what
    /// a site earns over a span of ticks.
    ///
    /// Returns `false` when the identity is dead.
    ///
    /// # Errors
    ///
    /// Returns an error when the rate is below zero, and when the commodity
    /// is outside the set.
    pub fn set_production_rate(
        &mut self,
        entity: Entity,
        commodity: CommodityId,
        rate: Fix32,
    ) -> Result<bool, RateError> {
        let Some(slot) = self.settlements.slot_of(entity) else {
            return Ok(false);
        };
        self.rates.open_to(self.settlements.slot_count());
        self.rates.set_production(slot, commodity, rate)?;
        Ok(true)
    }

    /// Writes the upkeep rate of a settlement, for one commodity.
    ///
    /// Upkeep is a rate above zero that subtracts. It is never a production
    /// rate below zero.[^1]
    ///
    /// Returns `false` when the identity is dead.
    ///
    /// # Errors
    ///
    /// Returns an error when the rate is below zero, and when the commodity
    /// is outside the set.
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-016. `docs/FINDINGS.md`
    pub fn set_upkeep_rate(
        &mut self,
        entity: Entity,
        commodity: CommodityId,
        rate: Fix32,
    ) -> Result<bool, RateError> {
        let Some(slot) = self.settlements.slot_of(entity) else {
            return Ok(false);
        };
        self.rates.open_to(self.settlements.slot_count());
        self.rates.set_upkeep(slot, commodity, rate)?;
        Ok(true)
    }

    /// Runs the rate pass of one frame.
    ///
    /// The stored table holds the base rate of each site. The pass derives an
    /// effective rate from it and from the world, and hands the derived table
    /// to the apply function. The stored table is not written.[^1]
    ///
    /// The derived table is scratch. It holds no fact of its own, so it stays
    /// out of the state hash, and its four inputs are stored and already
    /// enter it.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0062, production and upkeep are rates attached to a site, decisions D1 and D7. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    /// [^2]: ADR-0164, every stored value the step reads enters the state hash, decision D2. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    pub(super) fn apply_rates(&mut self, threads: usize) -> Result<(), StepError> {
        self.shortfall_log.clear();
        self.rates.open_to(self.settlements.slot_count());
        let schedule = self.schedule;
        let tick = self.tick;
        // The table is taken out of the world so that the fill may read the
        // world, and it is put back below. The take leaves the field empty
        // for the length of this call and nothing else reads it.
        let mut effective = core::mem::take(&mut self.effective_rates);
        self.fill_effective_rates(&mut effective);
        let outcome = crate::rates::apply(
            schedule,
            tick,
            &effective,
            self.settlements.store_update(),
            threads,
        );
        self.effective_rates = effective;
        let pass = outcome?;
        for (index, account) in self.store_account.iter_mut().enumerate() {
            let net = pass
                .ledger
                .net(CommodityId(index as u16))
                .expect("the index came from the commodity count");
            *account = sim_math::combine(*account, net);
        }
        self.rate_ledger = self.rate_ledger.combine(pass.ledger);
        self.shortfall_log = pass.shortfalls;
        Ok(())
    }

    /// Runs the consumption pass of one frame.
    ///
    /// The pass runs when the schedule is due, and it does nothing
    /// otherwise. It has four stages, and the order between them is the
    /// order of the rule: the need falls, the cohorts draw, the draw feeds
    /// the units, and the deficit follows the need.[^1]
    ///
    /// The cohort table is derived from the home column of the units. The
    /// pass derives it again here rather than carrying it between frames,
    /// so the table cannot disagree with the column it summarises.[^2]
    ///
    /// What leaves a store is what the cohorts received, so the account of
    /// the stores falls by the same amount. A pass that moved a quantity and
    /// forgot the account would fail the conservation check on every frame,
    /// at every thread count.[^3]
    ///
    /// # Errors
    ///
    /// Returns an error when the caller asks for zero threads, and when the
    /// columns disagree.
    ///
    /// # References
    ///
    /// [^1]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decisions D1, D2 and D4. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
    /// [^2]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    /// [^3]: Findings register, FND-065. `docs/FINDINGS.md`
    pub(super) fn consume(&mut self, threads: usize) -> Result<(), StepError> {
        self.rationed_log.clear();
        if !self.schedule.due(self.tick) {
            return Ok(());
        }
        let rule = self.need_rule;
        let schedule = self.schedule;
        let commodity = CommodityId(0);

        // The need falls first. The subtract saturates at zero.
        cohort::decay(
            schedule.per_application(rule.decay()),
            self.soldiers.need_update(),
            threads,
        )?;

        // The cohorts are derived from the unit columns, in slot order.
        let sites = self.settlements.slot_count();
        self.cohorts.rebuild(
            self.soldiers.home_column(),
            self.soldiers.faction_column(),
            self.soldiers.live_column(),
            sites,
        );

        let pass = cohort::draw(
            self.tick,
            schedule.per_application(rule.ration()),
            commodity,
            &self.cohorts,
            self.settlements.store_update(),
            threads,
        )?;

        // What the cohorts received left the stores.
        let index = commodity.0 as usize;
        self.store_account[index] = sim_math::combine(
            self.store_account[index],
            Accum(-pass.ledger.granted[index].0),
        );
        self.draw_ledger = self.draw_ledger.combine(pass.ledger);
        self.rationed_log = pass.rationed;

        cohort::satisfy(
            rule,
            schedule.per_application(rule.ration()),
            cohort::DrawKey {
                seed: self.config.seed,
                tick: self.tick,
            },
            &pass.shares,
            &self.cohorts,
            self.soldiers.need_update(),
            threads,
        )?;
        Ok(())
    }
}
