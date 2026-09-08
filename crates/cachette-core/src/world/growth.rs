//! The people of a site: the housing that holds them and the births that add.
//!
//! Housing bounds a population, and food buys a birth. The housing readers,
//! the birth settings and the growth pass sit together, because a birth is
//! refused by the housing the readers report.

use super::World;
use crate::growth;
use crate::rates::RateSchedule;
use crate::sim_math;
use crate::site::{CommodityId, COMMODITY_COUNT};
use crate::types::{Accum, Entity, Fix32};

impl World {
    /// Returns the housing that stands at a site.
    ///
    /// The answer is a quantity of housing and not a count of people. It
    /// follows from what has been built at the site, and the ground never
    /// sets it.[^1]
    ///
    /// Returns `None` when the identity names no live site.
    ///
    /// # References
    ///
    /// [^1]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D1. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
    #[must_use]
    pub fn site_housing(&self, site: Entity) -> Option<u32> {
        self.settlements.housing(site)
    }

    /// Writes the housing that stands at a site.
    ///
    /// Returns `false` when the identity names no live site.
    pub fn set_site_housing(&mut self, site: Entity, housing: u32) -> bool {
        self.settlements.set_housing(site, housing)
    }

    /// Returns how many units live at a site.
    ///
    /// **Nothing stores this count.** The answer is the sum of the cohort
    /// rows of the site, and the cohort table derives every one of those
    /// rows from the home column of the unit arena.[^1]
    ///
    /// The call costs the faction ceiling, which is a structural constant of
    /// the project. It never walks the population.
    ///
    /// Returns `None` when the identity names no live site.
    ///
    /// # References
    ///
    /// [^1]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D2. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
    #[must_use]
    pub fn site_residents(&self, site: Entity) -> Option<u32> {
        let slot = self.settlements.slot_of(site)?;
        // A slot the derived table has not covered holds no counted
        // resident. That is the answer the table gives, and it is the answer
        // the growth stage reads. A caller that spawned a unit and did not
        // step reads it before the frame settled, in the way every derived
        // structure of this engine behaves, and one public check says so.[^1]
        //
        // [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
        Some(self.cohorts.residents(slot).unwrap_or(0))
    }

    /// Returns the free places of a site.
    ///
    /// The free places are the people the housing holds, less the residents
    /// the site has. A site above its housing has no free place, and the
    /// answer is zero rather than a value below zero.[^1]
    ///
    /// Returns `None` when the identity names no live site.
    ///
    /// # References
    ///
    /// [^1]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D2. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
    #[must_use]
    pub fn site_free_places(&self, site: Entity) -> Option<u32> {
        let slot = self.settlements.slot_of(site)?;
        Some(self.free_places_of(slot))
    }

    /// Returns the free places of one settlement slot.
    pub(super) fn free_places_of(&self, slot: u32) -> u32 {
        let housing = self
            .settlements
            .housing_column()
            .get(slot as usize)
            .copied()
            .unwrap_or(0);
        let residents = self.cohorts.residents(slot).unwrap_or(0);
        growth::free_places(housing, self.housing_per_person, residents)
    }

    /// Returns the housing that one person takes.
    #[must_use]
    pub const fn housing_per_person(&self) -> u32 {
        self.housing_per_person
    }

    /// Sets the housing that one person takes.
    ///
    /// A value of zero leaves no site with a free place, so no site grows.
    pub const fn set_housing_per_person(&mut self, housing: u32) {
        self.housing_per_person = housing;
    }

    /// Returns the housing that a founded site starts with.
    #[must_use]
    pub const fn founding_housing(&self) -> u32 {
        self.founding_housing
    }

    /// Sets the housing that a founded site starts with.
    ///
    /// The value reaches the sites founded after the call. It changes no site
    /// that already stands.
    pub const fn set_founding_housing(&mut self, housing: u32) {
        self.founding_housing = housing;
    }

    /// Returns the store that one birth costs.
    #[must_use]
    pub const fn food_per_birth(&self) -> [Fix32; COMMODITY_COUNT] {
        self.food_per_birth
    }

    /// Sets the store that one birth costs.
    pub const fn set_food_per_birth(&mut self, cost: [Fix32; COMMODITY_COUNT]) {
        self.food_per_birth = cost;
    }

    /// Returns the chance that one proposal becomes a birth.
    #[must_use]
    pub const fn birth_chance(&self) -> Fix32 {
        self.birth_chance
    }

    /// Sets the chance that one proposal becomes a birth.
    ///
    /// A chance at or above one makes every proposal a birth. A chance at or
    /// below zero makes none.
    pub const fn set_birth_chance(&mut self, chance: Fix32) {
        self.birth_chance = chance;
    }

    /// Returns when the growth stage acts.
    #[must_use]
    pub const fn growth_schedule(&self) -> RateSchedule {
        self.growth_schedule
    }

    /// Sets when the growth stage acts.
    pub const fn set_growth_schedule(&mut self, schedule: RateSchedule) {
        self.growth_schedule = schedule;
    }

    /// Returns what growth took out of the stores, for each commodity.
    ///
    /// The total is cumulative over the life of the world. A caller that
    /// states the conservation of the stores reads it, because growth is a
    /// sink that neither the rate ledger nor the draw ledger holds.
    #[must_use]
    pub const fn growth_ledger(&self) -> [Accum; COMMODITY_COUNT] {
        self.growth_ledger
    }

    /// Returns how many people the growth stage added on the last tick.
    ///
    /// The count is a census of one tick. The growth stage clears it before
    /// it acts, so a zero says that the world grew nobody and not that the
    /// world never grew.
    #[must_use]
    pub const fn births(&self) -> u32 {
        self.births
    }

    /// Grows the population of every site, and returns nothing.
    ///
    /// # What it does
    ///
    /// The store of a site sets a rate, and the rate proposes a birth. The
    /// free places of the site admit the proposals, in ordinal order, until
    /// no place is free. A refused proposal is discarded and it is not
    /// carried to the next application.[^1] [^2]
    ///
    /// **The housing is a bound and not a factor.** A site at its housing
    /// grows nobody, however much food it holds. A site with free places
    /// grows at the rate its store sets, and the free places never scale that
    /// rate.[^2]
    ///
    /// An admitted proposal costs the store, spawns one unit at the address
    /// of the site, and writes the residence of that unit. The unit carries
    /// the type the spawn path gives, which is the worker row. This stage
    /// names no unit type.[^3]
    ///
    /// # Where it runs, and why
    ///
    /// It runs after the shortage scan and before the queue advance. The
    /// step states both positions.
    ///
    /// # What it costs
    ///
    /// The stage visits the settlements in slot order and reads a fixed
    /// number of proposals at each one. It walks no unit and it searches
    /// nothing, so its cost follows the settlements and the faction ceiling
    /// and never the population.[^4]
    ///
    /// # Determinism
    ///
    /// The sites apply in slot order, and the proposals of one site apply in
    /// ordinal order. Every draw is keyed on the system, the tick, the site
    /// and the ordinal of the proposal.[^5] No thread takes part.
    ///
    /// # References
    ///
    /// [^1]: ADR-0082, the store sets the rate of a birth and the housing admits it, decision D1. `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md`
    /// [^2]: ADR-0082, the store sets the rate of a birth and the housing admits it, decision D2. `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md`
    /// [^3]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D5. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
    /// [^4]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D4. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
    /// [^5]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
    pub(super) fn grow(&mut self) {
        // The count is a census of one tick, and this stage is where the
        // tick starts for it. A world that cannot grow and a world that
        // chose not to both read zero here, and the free places say which.
        //
        // The run total is folded here, immediately before the per-tick
        // count is emptied, because this is the one site that empties
        // it.[^7]
        //
        // [^7]: Findings register, FND-498. `docs/FINDINGS.md`
        self.census.births += i64::from(self.births);
        self.births = 0;
        if !self.growth_schedule.due(self.tick) {
            return;
        }
        let tick = self.tick.0;
        let seed = self.config.seed;
        let cost = self.food_per_birth;
        let chance = self.birth_chance;
        let mut grew = false;
        for slot in 0..self.settlements.slot_count() {
            let Some(site) = self.settlements.entity_at(slot) else {
                continue;
            };
            // **The housing admits, and it never scales the rate.** The free
            // places are read once for the site, and each birth takes one of
            // them. A site with no free place is done here, whatever its
            // store holds.
            let mut free = self.free_places_of(slot);
            if free == 0 {
                continue;
            }
            let Some(store) = self.settlements.store(site) else {
                continue;
            };
            let mut held = [Fix32::ZERO; COMMODITY_COUNT];
            for (index, quantity) in held.iter_mut().enumerate() {
                *quantity = store
                    .quantity(CommodityId(index as u16))
                    .expect("the index is inside the commodity set");
            }
            let proposals = growth::proposals(&held, &cost);
            let (Some(address), Some(faction)) = (
                self.settlements.address(site),
                self.settlements.faction(site),
            ) else {
                continue;
            };
            for index in 0..proposals {
                if free == 0 {
                    break;
                }
                if !growth::proposal_takes(seed, tick, slot, index, chance) {
                    continue;
                }
                // The rate counted what the store could pay when the stage
                // read it. Each birth spends, so the check runs again for
                // each one rather than once for the site.
                if !self.store_holds(slot, &cost) {
                    break;
                }
                let Ok(person) = self.spawn_soldier(address, faction) else {
                    break;
                };
                self.set_home_site(person, Some(site));
                self.take_from_store(slot, &cost);
                for (index, quantity) in cost.iter().enumerate() {
                    self.growth_ledger[index] =
                        sim_math::accumulate(self.growth_ledger[index], *quantity);
                }
                free -= 1;
                self.births += 1;
                grew = true;
            }
        }
        if !grew {
            return;
        }
        // The cohort table summarises the home column, and this stage changed
        // that column. A table left stale would state a headcount that no
        // unit backs, and the invariant check refuses that state.[^6]
        //
        // [^6]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
        self.cohorts.rebuild(
            self.soldiers.home_column(),
            self.soldiers.faction_column(),
            self.soldiers.live_column(),
            self.settlements.slot_count(),
        );
    }
}
