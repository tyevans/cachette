//! The readers and verbs of a settlement's economy and its people.
//!
//! This module holds the economy, housing and production reports of one site,
//! the verbs that spend at a set of sites, and the rules that govern growth,
//! birth, upkeep, stores and recovery.
//!
//! The grouping is by what the state belongs to. Every method here addresses a
//! settlement or a rule that a settlement pass reads.

use super::PyWorld;
use crate::errors::{VerbError, ViewError};
use crate::world::identity::{resolve_site, resolve_sites};
use cachette_core::rates::RateSchedule;
use cachette_core::resource::{RecoveryRules, RESOURCE_KIND_COUNT};
use cachette_core::{CommodityId, Fix32, ResourceKind};
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pymethods]
impl PyWorld {
    /// Returns what one site earns, holds and owes, as a `dict`.
    ///
    /// The site is one settlement identity, as a Python integer. The
    /// commodity is the number of the commodity to report on, and it
    /// defaults to zero.
    ///
    /// **A commodity is not a resource kind.** The numbers name different
    /// things. The world holds one commodity today, and its number is zero.
    /// Every other number raises `ViewError`, so the resource kinds one and
    /// two name no commodity.
    ///
    /// - `q` and `r`, integers. The address of the site.
    /// - `faction`, an integer. The faction that owns the site.
    /// - `commodity`, an integer. The commodity the call took.
    /// - `store`, an integer. What the site holds now. **A Q16.16 value as
    ///   its raw integer. Divide by 65536.**
    /// - `production`, an integer. What it adds each time the rate pass runs.
    ///   **Also Q16.16.**
    /// - `upkeep`, an integer. What it owes each time. **Also Q16.16.**
    /// - `store_capacity_raise`, an integer. **A raw Q16.16 quantity.** The
    ///   sum of the raise of every finished store upgrade on the tile of the
    ///   site or on one of its six neighbours. **Nothing in the engine reads
    ///   this.** The engine holds no store capacity, so the sum changes no
    ///   pass.
    /// - `rationed`, a `bool`. Whether the last draw could not serve every
    ///   cohort in full.
    /// - `demanded` and `granted`, integers or `None`. What the cohorts asked
    ///   for and what the store gave. **Both are Q16.16 when they are
    ///   integers.** Both are `None` when `rationed` is `False`.
    ///
    /// The engine holds no floating point number in simulated state, because
    /// float addition is not associative.[^1]
    ///
    /// **A settlement founded by `found_settlements` produces nothing**, and
    /// reports a production of zero. The rate comes from the survey, and only
    /// `found_group` and `found_run_for_every_faction` run one.[^2]
    ///
    /// The ration entries come from the log of the draw that just ran. The
    /// engine keeps that log for one tick, so a site that served every
    /// cohort in full reports no shortfall.[^3]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no live settlement, or
    /// when the world holds no such commodity.
    ///
    /// # References
    ///
    /// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^2]: ADR-0062, production and upkeep are rates attached to a site, decision D2. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    /// [^3]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D3. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
    #[pyo3(signature = (site, commodity = 0))]
    fn site_economy<'py>(
        &self,
        python: Python<'py>,
        site: u64,
        commodity: u16,
    ) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let entity = resolve_site(&world, site)?;
        let goods = CommodityId(commodity);
        let arena = world.settlements();
        let refuse = || ViewError::new_err(format!("{commodity} names no commodity of this world"));
        let store = arena
            .store(entity)
            .and_then(|held| held.quantity(goods))
            .ok_or_else(refuse)?;
        let production = world.production_rate(entity, goods).ok_or_else(refuse)?;
        let upkeep = world.upkeep_rate(entity, goods).ok_or_else(refuse)?;
        let address = arena
            .address(entity)
            .ok_or_else(|| ViewError::new_err(format!("the identity {site} names no live site")))?;
        let faction = arena
            .faction(entity)
            .ok_or_else(|| ViewError::new_err(format!("the identity {site} names no live site")))?;
        let report = PyDict::new(python);
        report.set_item("q", address.q)?;
        report.set_item("r", address.r)?;
        report.set_item("faction", faction.0)?;
        report.set_item("commodity", commodity)?;
        report.set_item("store", store.0)?;
        report.set_item("production", production.0)?;
        report.set_item("upkeep", upkeep.0)?;
        report.set_item(
            "store_capacity_raise",
            world
                .store_capacity_raise(entity)
                .expect("the identity resolved to a live site above"),
        )?;
        // The identity crosses whole. The comparison is against the value
        // the engine wrote into the log, and this code takes neither apart.
        let rationed = world
            .rationed_log()
            .iter()
            .find(|event| event.site == site && event.commodity == commodity);
        match rationed {
            Some(event) => {
                report.set_item("rationed", true)?;
                report.set_item("demanded", event.demanded.0)?;
                report.set_item("granted", event.granted.0)?;
            }
            None => {
                report.set_item("rationed", false)?;
                report.set_item("demanded", python.None())?;
                report.set_item("granted", python.None())?;
            }
        }
        Ok(report)
    }

    /// Returns the housing of one site and how full it is, as a `dict`.
    ///
    /// The site is one settlement identity, as a Python integer.
    ///
    /// **The housing is a quantity of housing and not a count of people.**
    /// It follows from what has been built at the site, and the ground never
    /// sets it. The people the housing holds is that quantity divided by the
    /// housing one person takes.
    ///
    /// The call answers about one site and it walks no population. The
    /// resident count is derived from the home column of the units, and
    /// nothing stores it.
    ///
    /// Three entries are Python integers.
    ///
    /// - `housing`. The housing that stands at the site.
    /// - `residents`. How many units live at the site, over every faction.
    /// - `free_places`. The people the housing holds, less the residents. A
    ///   site above its housing reads zero and never a value below zero.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no live settlement.
    fn site_housing<'py>(&self, python: Python<'py>, site: u64) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let entity = resolve_site(&world, site)?;
        let report = PyDict::new(python);
        report.set_item(
            "housing",
            world
                .site_housing(entity)
                .expect("the identity resolved to a live site above"),
        )?;
        report.set_item(
            "residents",
            world
                .site_residents(entity)
                .expect("the identity resolved to a live site above"),
        )?;
        report.set_item(
            "free_places",
            world
                .site_free_places(entity)
                .expect("the identity resolved to a live site above"),
        )?;
        Ok(report)
    }

    /// Returns what one settlement produces now, as a `dict`.
    ///
    /// The keys are:
    ///
    /// - `base`. The stored rate, as its raw Q16.16 integer. The founding
    ///   writes it once, from the food the survey measured, and nothing else
    ///   moves it.[^1]
    /// - `scale`. What the pipeline gives the site now, as its raw Q16.16
    ///   integer. One is 65536. The pipeline reads the ground the site
    ///   reaches, the moisture over it, the terraces on it, and the residents
    ///   in it.
    /// - `effective`. The base scaled, which is what the next application
    ///   earns. This is the value that varies over a run.
    ///
    /// **The scale and the effective rate are derived.** The engine stores
    /// neither, and neither enters the state hash. Both are read again from
    /// the world on every call.[^2]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no live settlement.
    ///
    /// # References
    ///
    /// [^1]: ADR-0062, production and upkeep are rates attached to a site, decision D1. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    /// [^2]: ADR-0164, every stored value the step reads enters the state hash, decision D2. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    #[pyo3(signature = (site, commodity = 0))]
    fn site_production<'py>(
        &self,
        python: Python<'py>,
        site: u64,
        commodity: u16,
    ) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let entity = resolve_site(&world, site)?;
        let goods = CommodityId(commodity);
        let slot = world
            .settlements()
            .slot_of(entity)
            .expect("the identity resolved to a live site above");
        let base = world.rates().production(slot, goods).ok_or_else(|| {
            VerbError::new_err(format!("{commodity} names no commodity of this world"))
        })?;
        let scale = world
            .production_scale(entity)
            .expect("the identity resolved to a live site above");
        let effective = world
            .effective_production_rate(entity, goods)
            .expect("the slot and the commodity both resolved above");
        let report = PyDict::new(python);
        report.set_item("base", base.0)?;
        report.set_item("scale", scale.0)?;
        report.set_item("effective", effective.0)?;
        Ok(report)
    }

    /// Writes the housing that stands at a set of settlements.
    ///
    /// Returns `None`.
    ///
    /// The sites are a sequence of settlement identities, or the NumPy array
    /// of `numpy.uint64` that `found_settlements` returned.
    ///
    /// **The housing is a quantity of housing and not a count of people.**
    /// Divide it by the housing one person takes to get the people it holds.
    ///
    /// **The write is absolute and not relative.** The site holds the value
    /// given, whatever it held before.
    ///
    /// **A site above its new housing keeps every resident.** A population
    /// above the housing that holds it is a state of the world and not a
    /// fault. The site grows nobody until the housing rises again.
    ///
    /// **The housing may be written at any time.** It is simulated state and
    /// it enters the state hash.[^1]
    ///
    /// **Read it back with `site_housing`.**
    ///
    /// **The set is all or nothing.** Every identity resolves before anything
    /// is written.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live settlement.
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    fn set_site_housing(&self, sites: Vec<u64>, housing: u32) -> PyResult<()> {
        let mut world = self.lock();
        let resolved = resolve_sites(&world, &sites)?;
        for site in resolved {
            world.set_site_housing(site, housing);
        }
        Ok(())
    }

    /// Returns how many people the growth stage added on the last tick.
    ///
    /// The count is a census of one tick. The growth stage clears it before
    /// it acts, so a zero says that the world grew nobody on that tick.
    ///
    /// **A zero beside a free place of zero says why.** A faction that never
    /// raises its housing stops growing, and the two numbers together state
    /// that reason.
    #[getter]
    fn births(&self) -> u32 {
        self.lock().births()
    }

    /// Returns the housing that one person takes.
    #[getter]
    fn housing_per_person(&self) -> u32 {
        self.lock().housing_per_person()
    }

    /// Sets the housing that one person takes.
    ///
    /// Returns `None`. A value of zero leaves no site with a free place, so
    /// no site grows.
    fn set_housing_per_person(&self, housing: u32) {
        self.lock().set_housing_per_person(housing);
    }

    /// Sets the chance that one proposal of one site becomes a birth.
    ///
    /// **The chance is a Q16.16 value as its raw integer.** Multiply the share
    /// you want by 65536. A chance at or above one whole unit makes every
    /// proposal a birth. A chance of zero makes none, so it turns growth off.
    /// Returns `None`.
    ///
    /// The chance is a balance row.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the population, the birth rate row. `docs/reference/balance.md`
    fn set_birth_chance(&self, chance: i32) {
        self.lock().set_birth_chance(Fix32(chance));
    }

    /// Sets how often the growth stage acts, and its offset in the period.
    ///
    /// The period is the ticks between two applications. The phase is the
    /// offset inside the period. Returns `None`.
    ///
    /// The schedule is a balance row.[^1]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the period is zero or above the range.
    ///
    /// # References
    ///
    /// [^1]: Balance register, the population, the growth schedule row. `docs/reference/balance.md`
    fn set_growth_schedule(&self, period: u32, phase: u32) -> PyResult<()> {
        let schedule = RateSchedule::new(period, phase).ok_or_else(|| {
            VerbError::new_err(format!("the period {period} is outside the range"))
        })?;
        self.lock().set_growth_schedule(schedule);
        Ok(())
    }

    /// Sets the store that one birth costs, for one commodity.
    ///
    /// The commodity is a commodity number. The quantity is a raw Q16.16
    /// integer. Returns `None`.
    ///
    /// **A birth is never free.** A site whose store cannot pay makes no
    /// person. The cost is a balance row.[^1]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no commodity of this world.
    ///
    /// # References
    ///
    /// [^1]: Balance register, the population, the food per birth row. `docs/reference/balance.md`
    #[pyo3(signature = (quantity, commodity = 0))]
    fn set_food_per_birth(&self, quantity: i32, commodity: u16) -> PyResult<()> {
        let mut world = self.lock();
        let mut cost = world.food_per_birth();
        let index = commodity as usize;
        if index >= cost.len() {
            return Err(VerbError::new_err(format!(
                "{commodity} names no commodity of this world"
            )));
        }
        cost[index] = Fix32(quantity);
        world.set_food_per_birth(cost);
        Ok(())
    }

    /// Gives every settlement the identities name one upkeep rate.
    ///
    /// Upkeep is the amount a site spends of a commodity. It is a rate at or
    /// above zero. It subtracts from the store, and it is never a production
    /// rate below zero.[^1]
    ///
    /// The sites are a sequence of settlement identities, or the NumPy array
    /// of `numpy.uint64` that `found_settlements` returned. Returns `None`.
    ///
    /// The `commodity` is the number of a commodity. A commodity is not a
    /// resource kind. The world holds one commodity today, and its number is
    /// zero. The argument has that number by default.
    ///
    /// **The rate is a Q16.16 value as its raw integer.** Multiply the amount
    /// you want by 65536. The rate is what one tick spends. The schedule
    /// scales it to one application. A longer period therefore does not
    /// change what a site spends over a span of steps. The engine holds no
    /// floating point number in simulated state, because float addition is
    /// not associative.[^2]
    ///
    /// **This is the one call that makes a shortfall possible.** A site that
    /// spends nothing can never fall short. `shortfall_log_columns` therefore
    /// answers with an empty log until a caller writes an upkeep rate. A
    /// finding records that the rate had no caller outside a test before this
    /// binding.[^3]
    ///
    /// **The set is all or nothing.** Every identity resolves, and the rate
    /// and the commodity are checked, before any site is written. One refusal
    /// leaves the world unchanged and raises.[^4]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live settlement. Raises
    /// `VerbError` when the rate is below zero, and when the number names no
    /// commodity of this world.
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-016. `docs/FINDINGS.md`
    /// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^3]: Findings register, FND-460. `docs/FINDINGS.md`
    /// [^4]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    #[pyo3(signature = (sites, rate, commodity = 0))]
    fn spend_at_sites(&self, sites: Vec<u64>, rate: i32, commodity: u16) -> PyResult<()> {
        let mut world = self.lock();
        // **The engine holds the one check of the rate, and this call adds
        // none.** A copy here would be a second statement of a rule the engine
        // already enforces, and nothing would fail when the two disagreed. The
        // engine checks the rate before it writes anything, and the rate is
        // one value for the whole set, so the first refusal leaves every site
        // untouched.
        let goods = CommodityId(commodity);
        let mut resolved = Vec::with_capacity(sites.len());
        for site in &sites {
            resolved.push(resolve_site(&world, *site)?);
        }
        // The commodity is checked against the store of each site before
        // anything is written. A write that refused halfway would leave one
        // part of the set changed and the rest untouched. The store holds one
        // quantity for each commodity, so it is what states the set.
        for entity in &resolved {
            world
                .settlements()
                .store(*entity)
                .and_then(|held| held.quantity(goods))
                .ok_or_else(|| {
                    VerbError::new_err(format!("{commodity} names no commodity of this world"))
                })?;
        }
        for entity in resolved {
            let wrote = world
                .set_upkeep_rate(entity, goods, Fix32(rate))
                .map_err(|error| VerbError::new_err(error.to_string()))?;
            assert!(
                wrote,
                "a resolved identity must name a settlement the world can write"
            );
        }
        Ok(())
    }

    /// Sets what a set of settlements earns of one commodity in one tick.
    ///
    /// Returns `None`.
    ///
    /// The sites are a sequence of settlement identities, or the NumPy array
    /// of `numpy.uint64` that `found_settlements` returned.
    ///
    /// **The rate is a Q16.16 value as its raw integer.** Multiply the amount
    /// you want by 65536. A rate of 65536 means one unit of the commodity in
    /// one tick. A rate of 0 means the site earns nothing.
    ///
    /// **The rate is what one tick earns, not what one application earns.**
    /// The engine multiplies it by the period of the economy schedule. A site
    /// earns the same amount over a span of ticks, whatever the period is.[^1]
    /// A caller may read the rate as the amount of one application. That
    /// caller writes a rate that is too large by the period.
    ///
    /// **The rate may be set at any time, and it takes effect at the next
    /// application.** It is not construction-time configuration. It is state
    /// that a later frame reads. It enters the state hash. Two worlds that
    /// hold different rates are two different worlds.[^2] No write can land
    /// inside a step. The engine releases the interpreter for the whole step.
    /// No Python line runs while the step runs.[^3]
    ///
    /// **Read the rate back with `site_economy`**, under the key
    /// `production`. This call publishes no reader of its own, because the
    /// value would then have two places to come from.[^4]
    ///
    /// The commodity defaults to zero, and the world holds one commodity.
    ///
    /// **The set is all or nothing.** Every identity resolves before anything
    /// is written. The engine refuses the rate and the commodity before it
    /// writes the first site. Neither refusal depends on the site.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live settlement. Raises
    /// `VerbError` when the rate is below zero, and when the number names no
    /// commodity of this world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0062, production and upkeep are rates attached to a site, decision D4. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    /// [^3]: ADR-0042, the interpreter is released for the whole step. `docs/adrs/REGISTRY.md`
    /// [^4]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[pyo3(signature = (sites, rate, commodity = 0))]
    fn set_production_rate(&self, sites: Vec<u64>, rate: i32, commodity: u16) -> PyResult<()> {
        let mut world = self.lock();
        let goods = CommodityId(commodity);
        let rate = Fix32(rate);
        let resolved = resolve_sites(&world, &sites)?;
        for site in resolved {
            world
                .set_production_rate(site, goods, rate)
                .map_err(|error| VerbError::new_err(error.to_string()))?;
        }
        Ok(())
    }

    /// Sets what a set of settlements owes of one commodity in one tick.
    ///
    /// Returns `None`.
    ///
    /// The sites are a sequence of settlement identities, or the NumPy array
    /// of `numpy.uint64` that `found_settlements` returned.
    ///
    /// **The rate is a Q16.16 value as its raw integer, and it is at or above
    /// zero.** Multiply the amount you want by 65536. Upkeep is a rate above
    /// zero that subtracts. It is never a production rate below zero, and the
    /// engine refuses one.[^1]
    ///
    /// **The rate is what one tick owes, not what one application owes.** The
    /// engine multiplies it by the period of the economy schedule, in the same
    /// way it does for production.[^2]
    ///
    /// Production runs before upkeep in one application, so a site pays this
    /// bill from the earnings of the same application. Upkeep that the store
    /// cannot pay is a shortfall: the store stops at zero rather than going
    /// below it.
    ///
    /// **The rate may be set at any time, and it takes effect at the next
    /// application.** It is state that a later frame reads, and it enters the
    /// state hash.[^3]
    ///
    /// **Read the rate back with `site_economy`**, under the key `upkeep`.
    ///
    /// **The set is all or nothing.** Every identity resolves before anything
    /// is written. The engine refuses the rate and the commodity before it
    /// writes the first site. Neither refusal depends on the site.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live settlement. Raises
    /// `VerbError` when the rate is below zero, and when the number names no
    /// commodity of this world.
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-016. `docs/FINDINGS.md`
    /// [^2]: ADR-0062, production and upkeep are rates attached to a site, decision D4. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    /// [^3]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[pyo3(signature = (sites, rate, commodity = 0))]
    fn set_upkeep_rate(&self, sites: Vec<u64>, rate: i32, commodity: u16) -> PyResult<()> {
        let mut world = self.lock();
        let goods = CommodityId(commodity);
        let rate = Fix32(rate);
        let resolved = resolve_sites(&world, &sites)?;
        for site in resolved {
            world
                .set_upkeep_rate(site, goods, rate)
                .map_err(|error| VerbError::new_err(error.to_string()))?;
        }
        Ok(())
    }

    /// Writes what a set of settlements holds of one commodity now.
    ///
    /// Returns `None`.
    ///
    /// The sites are a sequence of settlement identities, or the NumPy array
    /// of `numpy.uint64` that `found_settlements` returned.
    ///
    /// **The quantity is a Q16.16 value as its raw integer.** Multiply the
    /// amount you want by 65536. It is a quantity and not a rate. It says
    /// what the store holds at this tick. The next application changes it
    /// again.
    ///
    /// **The write is absolute and not relative.** The store holds the value
    /// given, whatever it held before. The engine moves its own account of the
    /// stores by the same amount, so a world-wide total stays exact.
    ///
    /// **Pass a quantity at or above zero.** The engine accepts one below zero
    /// and does not refuse it. The next application of upkeep then takes such
    /// a store to zero. It reports the upkeep, plus the amount the store sat
    /// below zero, less that application's production, as the shortfall. A
    /// store of zero is a real state and not an absent one.[^1]
    ///
    /// **The store may be written at any time.** It is simulated state and it
    /// enters the state hash.[^2]
    ///
    /// **Read the store back with `site_economy`**, under the key `store`.
    ///
    /// **The set is all or nothing.** Every identity resolves before anything
    /// is written. The engine refuses the commodity before it writes the
    /// first site, and that refusal does not depend on the site.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live settlement. Raises
    /// `VerbError` when the number names no commodity of this world.
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-043. `docs/FINDINGS.md`
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[pyo3(signature = (sites, quantity, commodity = 0))]
    fn set_settlement_store(&self, sites: Vec<u64>, quantity: i32, commodity: u16) -> PyResult<()> {
        let mut world = self.lock();
        let goods = CommodityId(commodity);
        let resolved = resolve_sites(&world, &sites)?;
        for site in resolved {
            world
                .set_settlement_store(site, goods, Fix32(quantity))
                .map_err(|error| VerbError::new_err(error.to_string()))?;
        }
        Ok(())
    }

    /// Sets the schedule that the site rates apply on.
    ///
    /// Returns `None`.
    ///
    /// The period is a count of ticks, and it must be at least one. A period
    /// of one applies on every step. The phase is the offset inside the
    /// period. A period of four and a phase of one apply on the ticks one,
    /// five and nine. A phase at or above the period wraps into it.
    ///
    /// The interval is a parameter of the schedule and not a constant of the
    /// engine.[^1] The schedule enters the state hash, because the next frame
    /// reads it.[^2]
    ///
    /// The world starts with a schedule already set. This call replaces it.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the period is zero, or above the range that the
    /// scaling multiply takes. The message names the limit it applied.
    ///
    /// # References
    ///
    /// [^1]: ADR-0062, production and upkeep are rates attached to a site, decision D4. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    fn set_economy_schedule(&self, period: u32, phase: u32) -> PyResult<()> {
        self.lock()
            .set_economy_schedule(period, phase)
            .map_err(|error| VerbError::new_err(error.to_string()))
    }

    /// Returns how fast a depleted deposit of each kind returns, as a list.
    ///
    /// The list holds one entry for each resource kind, in the order food,
    /// wood, stone. An entry is a count of ticks, or `None` for a kind that
    /// does not recover.
    ///
    /// **A period is the simulated time in which one depleted deposit regains
    /// one unit of stock.** It is a count of ticks and it is not fixed point.
    ///
    /// Write the rules with `set_recovery_rules`.
    fn recovery_rules(&self) -> Vec<Option<u32>> {
        let world = self.lock();
        let rules = world.recovery_rules();
        ResourceKind::ALL
            .iter()
            .map(|kind| rules.period_of(*kind))
            .collect()
    }

    /// Sets how fast a depleted deposit of each kind returns.
    ///
    /// Returns `None`.
    ///
    /// The periods are a sequence of three entries, one for each resource
    /// kind, in the order food, wood, stone. An entry is a count of ticks at
    /// or above one, or `None` for a kind that does not recover.
    ///
    /// **A period is the simulated time in which one depleted deposit regains
    /// one unit of stock.** It is a count of ticks and it is not fixed point.
    /// A smaller period returns a deposit faster.
    ///
    /// **The caller states every kind, and the engine takes the whole set.**
    /// No call changes one kind. A merge would put the period of a kind in
    /// two places while the call ran.[^1] A caller that wants to change one
    /// kind reads the three with `recovery_rules`, changes one, and writes the
    /// three back.
    ///
    /// **The rules may be set at any time.** They are world-wide, so this is
    /// one write for the world. The engine reads them on every tick that ages
    /// a depleted deposit.
    ///
    /// **The rules do not enter the state hash today, and that is a
    /// defect.**[^2] Two worlds that hold the same tiles and different rules
    /// hash the same and then diverge. The golden state test reports the
    /// effect of a change made here, and never the change itself. A backlog
    /// item holds the repair.[^3]
    ///
    /// # Errors
    ///
    /// Raises `ValueError` when the sequence does not hold exactly three
    /// entries. Raises `VerbError` when a period is zero. A period of zero
    /// returns the whole take in one tick. That is a second way to say that a
    /// deposit was never depleted.
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-270. `docs/DECISIONS.md`
    /// [^2]: Findings register, FND-480. `docs/FINDINGS.md`
    /// [^3]: Backlog item 0471, fold the recovery rules into the state hash. `docs/backlog/complete/0471-fold-the-recovery-rules-into-the-state-hash.md`
    fn set_recovery_rules(&self, periods: Vec<Option<u32>>) -> PyResult<()> {
        if periods.len() != RESOURCE_KIND_COUNT {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "the rules need one period for each of the {RESOURCE_KIND_COUNT} resource kinds, and {} were given",
                periods.len()
            )));
        }
        let mut taken = [None; RESOURCE_KIND_COUNT];
        taken.copy_from_slice(&periods);
        let rules = RecoveryRules::from_ticks(taken).ok_or_else(|| {
            VerbError::new_err(
                "a recovery period of zero is not a period; use None for a kind that does not recover",
            )
        })?;
        self.lock().set_recovery_rules(rules);
        Ok(())
    }
}
