//! The effective production rate of a site, derived from the world each
//! application.
//!
//! # Why this exists
//!
//! A site sets its production rate once. The founding path reads the food the
//! survey measured and writes the rate, and nothing reads the ground
//! again.[^1] A site founded on good land therefore produces the same amount
//! on the last tick of a run as on the first, whatever happened to the land,
//! the weather, the upgrades or the people. A source with no sink can only
//! climb, and the store of every faction climbs to its clamp.[^2]
//!
//! This module turns the stored rate into an **effective rate** each time the
//! rate pass applies. The stored rate stays the base rate and stays on the
//! site, so the founding rule is unchanged.[^3]
//!
//! # The pipeline
//!
//! Four terms compose. Each is a share of one. The pipeline adds two terms to
//! a base of one and takes two away, in the order below, clamps the sum, and
//! multiplies the base rate by it once.
//!
//! 1. **The ground.** What the disc of the site still holds of food, against
//!    what it held untouched. This is the sink. Gatherers draw the ground
//!    down and the recovery brings it back, so the term rises and falls over
//!    a period of about one simulated day.[^4]
//! 2. **The moisture.** Wet ground yields more. The term takes the same
//!    reader that the gather resolve takes, so the project holds one moisture
//!    reader and not two, and it stays discontinuous for the reason that
//!    record gives.[^5]
//! 3. **The upgrades.** A standing terrace is worked ground, and worked
//!    ground yields more. The term counts the terraces of the disc and stops
//!    at a ceiling.
//! 4. **The people.** A site that has lost its residents works less of its
//!    ground. The term takes away and never adds.
//!
//! # Why the terms add and do not multiply
//!
//! Addition in Q16.16 is exact and saturating, so four added terms carry no
//! truncation at all. Four multiplied factors truncate four times, and they
//! compound: four factors that each move by a quarter reach two and a half
//! times their span together, which no reader can hold in mind. The pipeline
//! therefore multiplies once, at the end, and both operands of that multiply
//! are above zero, which is the direction where the truncation bias is
//! benign.[^6]
//!
//! # Determinism
//!
//! The pipeline visits the disc of a site in the fixed disc order, and it
//! composes the four terms in the stated order.[^7] It draws nothing. Every
//! operation goes through the arithmetic module and no value here is a
//! floating point number.[^8]
//!
//! # The state hash
//!
//! **The effective rate is derived and it stays out of the state hash.** It
//! is computed again from the ground, the weather, the upgrades and the
//! residents each time the pass runs, and all four of those are stored and
//! are already in the hash.[^9] The weights below are compile-time constants.
//! Nothing stores them and no verb writes them, so they add no hash line.
//!
//! # The values
//!
//! Every weight below is provisional. The balance register holds one row for
//! each of them with the derivation.[^10]
//!
//! # What this does not hold
//!
//! **No temperature term.** The worker who owns the weather field states that
//! it is not built and that its range is not measured. A weight written
//! against a range nobody has taken would be an invented value. A separate
//! item holds the term.[^11]
//!
//! # References
//!
//! [^1]: Backlog item 0136, provision a founded site from the ground it reaches. `docs/backlog/complete/0136-provision-a-founded-site-from-the-ground-it-reaches.md`
//! [^2]: Findings register, FND-543. `docs/FINDINGS.md`
//! [^3]: ADR-0062, production and upkeep are rates attached to a site, decisions D1 and D7. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
//! [^4]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
//! [^5]: ADR-0143, wet ground yields more to a gatherer, decisions D1 and D2. `docs/adrs/draft/adr-0143-wet-ground-yields-more-to-a-gatherer.md`
//! [^6]: Findings register, FND-012. `docs/FINDINGS.md`
//! [^7]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
//! [^8]: ADR-0002, simulated and aggregated state holds no floating point number, decisions D1 and D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^9]: ADR-0164, every stored value the step reads enters the state hash, decision D2. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
//! [^10]: Balance register, the production pipeline. `docs/reference/balance.md`
//! [^11]: Backlog item 0512, add the temperature term to the production pipeline. `docs/backlog/proposed/0512-add-the-temperature-term-to-the-production-pipeline.md`

use crate::founding::{disc, SURVEY_RADIUS};
use crate::hex::Axial;
use crate::rates::RateTable;
use crate::resource::{Amount, ResourceKind};
use crate::sim_math;
use crate::site::{CommodityId, COMMODITY_COUNT};
use crate::types::{Accum, Entity, Fix32};
use crate::upgrade::UpgradeCategory;
use crate::world::World;

/// The most the drawn-down ground takes away, as a share of one.
///
/// This is the largest of the four weights, because the ground is the only
/// term that both falls and recovers on its own. Half is chosen so that a
/// site whose disc has been stripped bare still produces, and so that the
/// ground alone cannot reach the clamp floor.
pub const GROUND_WEIGHT: Fix32 = Fix32(1 << 15);

/// What wet ground adds, as a share of one.
///
/// Half of the ground weight. Weather helps a site, and it cannot outrun what
/// the ground itself holds.
pub const WET_WEIGHT: Fix32 = Fix32(1 << 14);

/// What one standing terrace on the disc adds, as a share of one.
pub const TERRACE_WEIGHT: Fix32 = Fix32(1 << 12);

/// The most the terraces of one disc add together, as a share of one.
///
/// Eight terraces reach it. Without a ceiling the upgrades would be the
/// unbounded source that this module exists to remove.
pub const TERRACE_CEILING: Fix32 = Fix32(1 << 15);

/// The most an emptied site loses, as a share of one.
///
/// The smallest of the four weights. The population is bounded by the
/// housing, so it settles and stops moving, and a term whose input settles
/// must be small.
pub const PEOPLE_WEIGHT: Fix32 = Fix32(1 << 14);

/// The lowest scale the pipeline returns.
pub const SCALE_FLOOR: Fix32 = Fix32(1 << 14);

/// The highest scale the pipeline returns.
pub const SCALE_CEILING: Fix32 = Fix32(2 << 16);

impl World {
    /// Fills a table with the effective production rate of every site.
    ///
    /// The table is opened to the slot count of the settlement arena, and
    /// every slot is written. A slot with no live site takes the idle rate,
    /// because the apply pass skips it and a stale row would be a second
    /// declaration of a rate.
    ///
    /// The caller owns the table. It is scratch, it holds no fact of its own,
    /// and it does not enter the state hash.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0164, every stored value the step reads enters the state hash, decision D2. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    pub(crate) fn fill_effective_rates(&self, table: &mut RateTable) {
        let slots = self.settlements().slot_count();
        table.open_to(slots);
        for slot in 0..slots {
            let seat = self
                .settlements()
                .entity_at(slot)
                .and_then(|site| self.settlements().address(site).map(|place| (site, place)));
            let Some((site, address)) = seat else {
                table.clear_slot(slot);
                continue;
            };
            let scale = self.production_scale_at(address, site);
            for index in 0..COMMODITY_COUNT {
                let commodity = CommodityId(index as u16);
                let base = self
                    .rates()
                    .production(slot, commodity)
                    .unwrap_or(Fix32::ZERO);
                let upkeep = self.rates().upkeep(slot, commodity).unwrap_or(Fix32::ZERO);
                // One multiply, at the end of the pipeline. Both operands are
                // at or above zero, so the truncation runs towards zero.
                let effective = sim_math::mul(base, scale);
                table
                    .set_production(slot, commodity, effective)
                    .expect("the scale is above zero and the slot came from the arena");
                table
                    .set_upkeep(slot, commodity, upkeep)
                    .expect("the upkeep came from a table that refuses a rate below zero");
            }
        }
    }

    /// Returns the scale that the pipeline gives one site.
    ///
    /// Returns `None` when the identity names no live site.
    ///
    /// The answer is derived. It is read again from the world on every call,
    /// and it is the same value that the last application used only when the
    /// four inputs have not moved since.
    #[must_use]
    pub fn production_scale(&self, site: Entity) -> Option<Fix32> {
        let address = self.settlements().address(site)?;
        Some(self.production_scale_at(address, site))
    }

    /// Returns the production rate that one site would earn now.
    ///
    /// The answer is the stored base rate scaled by the pipeline. It is
    /// derived, so it is not stored anywhere and it does not enter the state
    /// hash.[^1]
    ///
    /// Returns `None` when the identity names no live site, or when the
    /// commodity is outside the set.
    ///
    /// # References
    ///
    /// [^1]: ADR-0164, every stored value the step reads enters the state hash, decision D2. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
    #[must_use]
    pub fn effective_production_rate(&self, site: Entity, commodity: CommodityId) -> Option<Fix32> {
        let slot = self.settlements().slot_of(site)?;
        let base = self.rates().production(slot, commodity)?;
        let scale = self.production_scale(site)?;
        Some(sim_math::mul(base, scale))
    }

    /// Returns the scale that the pipeline gives one site, as a Q16.16 value.
    ///
    /// The four terms are added to a base of one in the order the module
    /// header states, and the sum is clamped. The result is at or above the
    /// floor, so the rate it scales is never below zero.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0062, production and upkeep are rates attached to a site, decision D2. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    #[must_use]
    pub fn production_scale_at(&self, address: Axial, site: Entity) -> Fix32 {
        let mut scale = Fix32::ONE;
        scale = sim_math::sub(scale, self.ground_term(address));
        scale = sim_math::add(scale, self.moisture_term(address));
        scale = sim_math::add(scale, self.upgrade_term(address));
        scale = sim_math::sub(scale, self.people_term(site));
        clamp(scale, SCALE_FLOOR, SCALE_CEILING)
    }

    /// Returns what the drawn-down ground takes away.
    ///
    /// The disc is the disc the founding survey read, so the term answers for
    /// exactly the ground that set the base rate.[^1] The term is the ground
    /// weight times the share of the food of that disc that somebody has
    /// taken. Untouched ground gives zero.
    ///
    /// A disc that never held food gives zero. Such a site has a base rate of
    /// zero, so the term changes nothing whatever it returns.
    ///
    /// # References
    ///
    /// [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D5. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    #[must_use]
    fn ground_term(&self, address: Axial) -> Fix32 {
        let grid = self.grid();
        let mut original = 0i64;
        let mut taken = 0i64;
        for place in disc(grid, address, SURVEY_RADIUS) {
            let Some(tile) = grid.index_of(place) else {
                continue;
            };
            let held = self
                .resources()
                .original(place, ResourceKind::Food)
                .unwrap_or(Amount::ZERO);
            let gone = self.depletion().taken(tile, ResourceKind::Food);
            original += i64::from(held.0);
            taken += i64::from(gone.0.min(held.0));
        }
        let Some(part) = sim_math::share(
            Accum(i64::from(Fix32::ONE.0)),
            Accum(taken),
            Accum(original),
        ) else {
            return Fix32::ZERO;
        };
        sim_math::mul(GROUND_WEIGHT, narrow(part))
    }

    /// Returns what the moisture over the site adds.
    ///
    /// The weather field answers for a level 1 cell, so every tile of one
    /// cell answers the same. The term therefore reads the tile of the site
    /// itself and not the whole disc.
    ///
    /// The read is discontinuous, because the reader is. Ground is wet or it
    /// is not, and the record that owns the reader says why it stays that
    /// way.[^1]
    ///
    /// The weather solves after the rate pass in the frame, so the term reads
    /// the field that the previous frame settled on. That is the same lag the
    /// gather resolve carries.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0143, wet ground yields more to a gatherer, decisions D1 and D2. `docs/adrs/draft/adr-0143-wet-ground-yields-more-to-a-gatherer.md`
    #[must_use]
    fn moisture_term(&self, address: Axial) -> Fix32 {
        if self.ground_is_wet(address) == Some(true) {
            WET_WEIGHT
        } else {
            Fix32::ZERO
        }
    }

    /// Returns what the standing terraces of the disc add.
    ///
    /// A terrace is worked ground, and the upgrade table says a unit takes
    /// more from a terraced tile. A site whose people terraced its ground
    /// produces more from it. No other category counts: a store is capacity
    /// and a wall is defence, and giving either an output would be a rule
    /// nobody has stated.
    ///
    /// The count stops at the ceiling, so a disc covered in terraces cannot
    /// make the upgrades an unbounded source.
    #[must_use]
    fn upgrade_term(&self, address: Axial) -> Fix32 {
        let mut total = Fix32::ZERO;
        for place in disc(self.grid(), address, SURVEY_RADIUS) {
            if self.finished_upgrade(place) == Some(UpgradeCategory::TERRACE) {
                total = sim_math::add(total, TERRACE_WEIGHT);
                if total >= TERRACE_CEILING {
                    return TERRACE_CEILING;
                }
            }
        }
        total
    }

    /// Returns what a site short of residents loses.
    ///
    /// The term is the people weight times the share of the housing of the
    /// site that stands empty. A full site loses nothing, so the founding
    /// rule is unchanged for a site whose people are all at home.
    ///
    /// A site with no housing loses nothing. Housing of zero is the state of
    /// a site that the founding has not yet provisioned, and a term that
    /// punished it would punish the tick of its own founding.
    #[must_use]
    fn people_term(&self, site: Entity) -> Fix32 {
        let housing = self.site_housing(site).unwrap_or(0);
        if housing == 0 {
            return Fix32::ZERO;
        }
        let residents = self.site_residents(site).unwrap_or(0).min(housing);
        let empty = i64::from(housing - residents);
        let Some(part) = sim_math::share(
            Accum(i64::from(Fix32::ONE.0)),
            Accum(empty),
            Accum(i64::from(housing)),
        ) else {
            return Fix32::ZERO;
        };
        sim_math::mul(PEOPLE_WEIGHT, narrow(part))
    }
}

/// Narrows an accumulator that holds a share of one into the fixed-point
/// range, and clamps it to the closed range from zero to one.
///
/// Every caller builds the value from a part and a whole that it has already
/// bounded, so the clamp is a guard and not the rule.
const fn narrow(value: Accum) -> Fix32 {
    if value.0 <= 0 {
        Fix32::ZERO
    } else if value.0 >= Fix32::ONE.0 as i64 {
        Fix32::ONE
    } else {
        Fix32(value.0 as i32)
    }
}

/// Clamps a value into a closed range.
const fn clamp(value: Fix32, low: Fix32, high: Fix32) -> Fix32 {
    if value.0 < low.0 {
        low
    } else if value.0 > high.0 {
        high
    } else {
        value
    }
}
