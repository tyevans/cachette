//! The founding of a settlement, and the survey that finds a place for one.
//!
//! A founding puts a group of people on the ground. The survey answers where
//! a group fits, the settle calls put it there, and the refusal says why a
//! place did not take it. One act, so one module.

use super::World;
use crate::event::SettlementFounded;
use crate::founding::{
    self, Founding, FoundingError, FoundingOutcome, SettleError, SettleOutcome, Survey,
};
use crate::hex::Axial;
use crate::resource::Amount;
use crate::sim_math;
use crate::site::{CommodityId, SettlementError};
use crate::types::{Entity, FactionId, Fix32};

/// The number of people each faction founds with when the seeding layer
/// founds the run.
///
/// **This is a set value and not a measured one.** The project owner set it.
/// The balance register holds the row, marks it unset, and records who set
/// this value and when.[^1]
///
/// **The Python binding reads this constant for its own default.** The
/// binding declares no number of its own, so the two cannot disagree.
///
/// # References
///
/// [^1]: Balance register, the founding group. `docs/reference/balance.md`
pub const FOUNDING_GROUP_DEFAULT: u32 = 2;

impl World {
    /// Founds a settlement in the world and returns its identity.
    ///
    /// # Errors
    ///
    /// Returns an error when the arena holds no free slot, when the address
    /// is outside the world, when the faction is at or above the ceiling,
    /// when the ground carries no unit, or when another settlement already
    /// stands on the tile.
    pub fn found_settlement(
        &mut self,
        address: Axial,
        faction: FactionId,
    ) -> Result<Entity, SettlementError> {
        // The arena refuses a faction above the project ceiling. This world
        // holds a faction count of its own, which is at most that ceiling,
        // and a settlement of a faction the world does not have is a caller
        // mistake rather than a storage one.
        if faction.0 >= self.config.faction_count.max(1) {
            return Err(SettlementError::FactionAboveCeiling(faction));
        }
        // Ground that admits no unit admits no holder, and a settlement is a
        // holder of ground.[^2] The rule reads the passability of the tile
        // and states nothing of its own about the ground, so the capacity
        // table stays the one declaration of which ground carries
        // anybody.[^3] [^4]
        //
        // The extent refusal stays with the arena, which owns the grid, so
        // this test says nothing about an address outside the world.
        //
        // [^2]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D5. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
        // [^3]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
        // [^4]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
        if self.grid.contains(address) && !self.admits_a_unit(address) {
            return Err(SettlementError::TileAdmitsNobody(address));
        }
        let settlement = self.settlements.found(address, faction)?;
        // The rate table follows the slot column of the arena, and a founding
        // may open a slot that the table has never held. A new row earns
        // nothing and owes nothing.
        //
        // The founding does not clear the row. The loss of a settlement does
        // that, and it is the only place that needs to: a slot the arena has
        // never handed out already holds an idle row. Clearing the row here as
        // well would state one fact in two places, and neither copy would fail
        // when the other was removed.[^1]
        //
        // [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
        self.rates.open_to(self.settlements.slot_count());
        // The position table follows the same slot column, for the same
        // reason. A new slot holds no position and the preference that a
        // site starts with.
        self.positions.open_to(self.settlements.slot_count());
        // The queue table follows the same slot column, for the same reason.
        // A new slot holds an empty queue.
        self.queues.open_to(self.settlements.slot_count());
        // **A founded site starts with the housing of the world parameter.**
        // A founded site must house the group that founds it, or a run starts
        // crowded. The housing is stored and the ground never sets it, so the
        // founding writes it here and no pass derives it later.[^5]
        //
        // [^5]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D1. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
        self.settlements
            .set_housing(settlement, self.founding_housing);
        // **This is the one place a settlement comes into existence, so it is
        // the one place that says so.** Every caller path and the settle
        // system pass through here. The census row is a count of what stands,
        // so it falls when a settlement is lost and it hides a founding in
        // the same tick.
        //
        // A founding between two steps stays in the log until the next step
        // clears it.
        if let Some(tile) = self.grid.index_of(address) {
            self.founded_log.push(SettlementFounded::new(
                self.tick,
                settlement.to_bits(),
                tile,
                faction,
            ));
        }
        Ok(settlement)
    }

    /// Surveys a bounded sample of the world for a place to found a group.
    ///
    /// The call reads a fixed number of candidate places and a fixed number
    /// of tiles around each one. Neither number is a function of the world
    /// extent, so the cost of the call does not grow with the world.[^1] The
    /// call writes nothing. A watcher asks it why a place is good and gets
    /// the counts that made the score.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the group holds nobody, or when the ordering of
    /// the candidates refuses to run.
    ///
    /// # References
    ///
    /// [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D1. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    /// [^2]: ADR-0075, the founding choice reads a bounded sample of the world, decision D5. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    pub fn survey_founding(&self, group: u32, faction: FactionId) -> Result<Survey, FoundingError> {
        founding::survey(self.resources, group, faction, &[])
    }

    /// Surveys a sample for a place that keeps its distance from the places
    /// taken.
    ///
    /// The faction fills the frame slot of the draw key, so two factions read
    /// two samples.[^1] A place closer than the minimum distance to a place in
    /// the list is not eligible, whatever the ground there holds.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the group holds nobody, or when the ordering of
    /// the candidates refuses to run.
    ///
    /// # References
    ///
    /// [^1]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D3. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
    /// [^2]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D1. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
    pub fn survey_founding_apart(
        &self,
        group: u32,
        faction: FactionId,
        taken: &[Axial],
    ) -> Result<Survey, FoundingError> {
        founding::survey(self.resources, group, faction, taken)
    }

    /// Surveys the places a caller names, against the places taken.
    ///
    /// A caller that wants to compare two places of its own choosing calls
    /// this. The call writes nothing.
    ///
    /// # Errors
    ///
    /// Returns an error when the group holds nobody, or when the ordering of
    /// the candidates refuses to run.
    pub fn survey_places(
        &self,
        addresses: &[Axial],
        group: u32,
        taken: &[Axial],
    ) -> Result<Survey, FoundingError> {
        founding::survey_addresses(self.resources, addresses, group, taken)
    }

    /// Founds a run: a group of people, in a place the engine chose.
    ///
    /// The size of the group is an input to the run. It is not the population
    /// the world is sized for, and the world reserves the same storage
    /// whatever it is.[^1]
    ///
    /// The founding is one of two ways to people a world. A caller that wants
    /// a unit in a place of its own choosing spawns one directly, and this
    /// call is built on that one.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the group holds nobody, when no place in the
    /// sample admits the whole group, when the ordering refuses to run, or
    /// when a person or the settlement refuses to arrive.
    ///
    /// # References
    ///
    /// [^1]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
    /// [^2]: Open decisions register, DEC-030. `docs/DECISIONS.md`
    pub fn found_run(&mut self, group: u32, faction: FactionId) -> Result<Founding, FoundingError> {
        self.found_one(group, faction, &[])
    }

    /// Founds one group for each faction the world holds.
    ///
    /// The run founds in ascending faction index. The order is a property of
    /// the run and not an input, so no caller can give one faction the better
    /// place by listing it first.[^1] Founding N keeps the minimum distance
    /// from every place a founding before it took, so the foundings are a
    /// sequence and not a set.[^2]
    ///
    /// The run reports one outcome for each faction. A faction that finds no
    /// admissible place is refused, and the foundings before it stand.[^3]
    /// The faction set comes from the world, so the loop holds no second
    /// count of its own.[^4]
    ///
    /// # References
    ///
    /// [^1]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D2. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
    /// [^2]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D1. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
    /// [^3]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
    /// [^4]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D1. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    #[must_use]
    pub fn found_run_for_every_faction(&mut self, group: u32) -> Vec<FoundingOutcome> {
        let mut taken: Vec<Axial> = Vec::new();
        let mut outcomes = Vec::new();
        for index in 0..self.config.faction_count.max(1) {
            let faction = FactionId(index);
            let result = self.found_one(group, faction, &taken);
            if let Ok(founding) = &result {
                taken.push(founding.place());
            }
            outcomes.push(FoundingOutcome::new(faction, result));
        }
        outcomes
    }

    /// Founds one group, away from the places already taken.
    fn found_one(
        &mut self,
        group: u32,
        faction: FactionId,
        taken: &[Axial],
    ) -> Result<Founding, FoundingError> {
        let survey = self.survey_founding_apart(group, faction, taken)?;
        let chosen = survey
            .chosen()
            .ok_or(FoundingError::NoPlaceFound(survey.drawn()))?;
        let place = chosen.address();
        let (settlement, people) = self.settle_group(place, group, faction)?;
        self.provision_site(settlement, chosen.provision().food);
        self.record_seat(faction, place);
        Ok(Founding::new(place, settlement, people, survey))
    }

    /// Founds a group at a place the caller names.
    ///
    /// The engine chooses the place of a run. This call exists so that a test
    /// can compare a place the engine chose against a place it did not, on a
    /// quantity the test computes for itself.
    ///
    /// # Errors
    ///
    /// Returns an error when the group holds nobody, when the address lies
    /// outside the world, when the place does not admit the whole group, or
    /// when a person or the settlement refuses to arrive.
    pub fn found_group_at(
        &mut self,
        address: Axial,
        group: u32,
        faction: FactionId,
    ) -> Result<Founding, FoundingError> {
        if !self.grid.contains(address) {
            return Err(FoundingError::OutsideWorld(address));
        }
        let survey = founding::survey_addresses(self.resources, &[address], group, &[])?;
        let chosen = survey
            .chosen()
            .ok_or(FoundingError::NoPlaceFound(survey.drawn()))?;
        let (settlement, people) = self.settle_group(address, group, faction)?;
        self.provision_site(settlement, chosen.provision().food);
        self.record_seat(faction, address);
        Ok(Founding::new(chosen.address(), settlement, people, survey))
    }

    /// Returns the place of every settlement that stands, in slot order.
    ///
    /// This is the list a founding keeps its distance from. It is derived
    /// from the arena on every call, so no second copy of it can disagree
    /// with the settlements the world holds.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub fn standing_places(&self) -> Vec<Axial> {
        self.settlements
            .iter()
            .filter_map(|site| self.settlements.address(site))
            .collect()
    }

    /// Founds a city from each settler of a set, and spends the settler.
    ///
    /// A settler is a unit whose type row holds a settle column above zero.
    /// The verb reads that column and never a type index, so no rule here
    /// names a type.[^1] It founds a settlement on the tile the unit stands
    /// on, for the faction of the unit, and it seats the group the column
    /// names.[^2]
    ///
    /// **The founding keeps the distance that the seeding keeps.** The place
    /// of every settlement that stands is the list the survey compares
    /// against, and the comparison is the one the seeding uses. A place
    /// inside that distance is refused.[^3]
    ///
    /// The verb refuses a unit that no live unit answers to, a unit whose
    /// settle column is zero, a faction the world does not hold, a tile any
    /// faction holds, a tile that carries a settlement, ground that admits no
    /// unit, and a place inside the founding distance. A refused unit changes
    /// nothing, and a set in which the verb refuses every unit changes
    /// nothing at all.[^2]
    ///
    /// **The founding spends the settler.** The unit leaves the world after
    /// the settlement stands, and the group the column names takes its
    /// place.[^2]
    ///
    /// The units are answered in the order the caller gave. A founding by an
    /// earlier unit of the set enters the distance list of the units after
    /// it, so a set of two settlers on one tile founds one city.
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decisions D1 and D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    /// [^2]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D5. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    /// [^3]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D1. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
    #[must_use]
    pub fn settle_set(&mut self, units: &[Entity]) -> Vec<SettleOutcome> {
        let outcomes: Vec<SettleOutcome> = units
            .iter()
            .map(|unit| {
                let result = self.settle_one(*unit);
                SettleOutcome::new(*unit, result)
            })
            .collect();
        // A founding seats a group and spends the settler, so the arena has
        // moved past the derived unit structure. The verb leaves the world
        // readable, in the way the seeding verb does.[^2]
        //
        // [^2]: Findings register, FND-647. `docs/FINDINGS.md`
        self.leave_the_world_readable();
        outcomes
    }

    /// Founds a city from one settler, and spends it.
    fn settle_one(&mut self, unit: Entity) -> Result<Founding, SettleError> {
        // **The rule lives in the check, and this verb reads it.** The
        // legality answer reads the same check.[^lg]
        //
        // [^lg]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
        let (address, group, faction, survey) = self.settle_refusal(unit)?;
        let chosen = survey
            .candidates()
            .first()
            .copied()
            .ok_or(SettleError::OutsideWorld(address))?;
        let (settlement, people) = self.settle_group(address, group, faction)?;
        self.provision_site(settlement, chosen.provision().food);
        // The seat of a faction is the tile of its first founding, and a
        // settler founds after that one. The call leaves a seat that stands.
        self.record_seat(faction, address);
        // The settler is spent. It leaves after the settlement stands, so a
        // refusal above never costs the unit.[^1]
        //
        // [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D5. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
        self.despawn_soldier(unit);
        Ok(Founding::new(address, settlement, people, survey))
    }

    /// Reports whether the settle verb would refuse one settler, without
    /// founding anything.
    ///
    /// **This is the one statement of the rule.** The settle verb calls it
    /// before it seats a group, and the legality answer calls it to fill one
    /// row of the action table.[^1] Nothing here mutates. It returns the
    /// place, the group size, the faction and the survey, so that the verb
    /// does not read them twice.
    ///
    /// # Errors
    ///
    /// Returns the refusal the settle verb would return.
    ///
    /// # References
    ///
    /// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    pub fn settle_refusal(
        &self,
        unit: Entity,
    ) -> Result<(Axial, u32, FactionId, Survey), SettleError> {
        let (Some(unit_type), Some(address), Some(faction)) = (
            self.soldiers.unit_type(unit),
            self.soldiers.address(unit),
            self.soldiers.faction(unit),
        ) else {
            return Err(SettleError::NoSuchUnit(unit));
        };
        let group = self.unit_types.row(unit_type).settle_group;
        if group == 0 {
            return Err(SettleError::NotASettler(unit));
        }
        if !self.grid.contains(address) {
            return Err(SettleError::OutsideWorld(address));
        }
        if faction.0 >= self.config.faction_count.max(1) {
            return Err(SettleError::FactionMayNotFound(faction));
        }
        // A tile belongs to the faction of the nearest city within reach, and
        // a settler founds only where nobody holds.[^1]
        //
        // [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decisions D1 and D5. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
        if self
            .holding
            .holder(address)
            .is_some_and(|holder| !holder.is_nobody())
        {
            return Err(SettleError::GroundIsHeld(address));
        }
        if self.settlement_on(address).is_some() {
            return Err(SettleError::SettlementStands(address));
        }
        // The distance list and the eligibility both come from the survey the
        // seeding uses, so the distance rule has one statement in the tree.
        let taken = self.standing_places();
        let survey = founding::survey_addresses(self.resources, &[address], group, &taken)?;
        let chosen = survey
            .candidates()
            .first()
            .copied()
            .ok_or(SettleError::OutsideWorld(address))?;
        if !chosen.is_separated() {
            return Err(SettleError::TooCloseToACity(address));
        }
        if !chosen.is_eligible() {
            return Err(SettleError::GroundAdmitsNobody(address));
        }
        Ok((address, group, faction, survey))
    }

    /// Returns the settlers of one faction, and why the settle verb refuses
    /// each one.
    ///
    /// The answer holds one name for each settler, in the order the settler
    /// reader gives, so a caller reads a position for every settler and not a
    /// refusal count. A settler the verb would accept carries the accepted
    /// name.[^1]
    ///
    /// **A refused verb answers one byte and names no reason.** A learner
    /// that takes the settle row and reads a refusal therefore learns
    /// nothing about which rule refused, and neither does the person who
    /// debugs the run. This reader is the reason, and it comes from the one
    /// statement of the rule that the verb itself calls.[^2]
    ///
    /// **This answers for one faction and reads no other.** It is therefore a
    /// reader the environment of a learner may call.[^3]
    ///
    /// Returns `None` when the number names no faction of this world.
    ///
    /// # References
    ///
    /// [^1]: The accepted name. [`crate::founding::SETTLE_ACCEPTED`]
    /// [^2]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    /// [^3]: PRD-0001, a faction sees only what it observes. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
    #[must_use]
    pub fn settle_refusal_names(&self, faction: FactionId) -> Option<Vec<&'static str>> {
        if faction.0 >= self.config.faction_count.max(1) {
            return None;
        }
        Some(
            self.settlers_of(faction)
                .into_iter()
                .map(|unit| match self.settle_refusal(unit) {
                    Ok(_) => founding::SETTLE_ACCEPTED,
                    Err(refusal) => refusal.name(),
                })
                .collect(),
        )
    }

    /// Returns how many settlers one faction holds.
    ///
    /// A settler is a unit whose type row holds a settle column above zero,
    /// and the reader that answers the settlers of a faction is the one
    /// statement of that rule.[^1] The observation publishes this count, so
    /// the array a policy reads and this reader cannot disagree.[^2]
    ///
    /// Returns `None` when the number names no faction of this world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    /// [^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub fn settler_count(&self, faction: FactionId) -> Option<u32> {
        if faction.0 >= self.config.faction_count.max(1) {
            return None;
        }
        Some(self.settlers_of(faction).len() as u32)
    }

    /// Sets the food a founded site produces, from the ground it reaches.
    ///
    /// A founding seats a group and gives it a store. Nothing else fills
    /// that store, so a site founded without a rate feeds nobody, and every
    /// unit in it crosses the bound at the same tick.[^1] The founding
    /// therefore sets the rate, because the founding is the one call that
    /// has both the site and the survey that measured the ground.
    ///
    /// The rule is that one unit of food the place reaches feeds one person.
    /// The ration comes from the need rule and is not repeated here, so the
    /// amount a person eats has one declaration site.[^2] A place that
    /// reaches more food than the group needs therefore holds a surplus, and
    /// a place that reaches less runs the group short. The survey score
    /// weighs food, so the choice the engine makes now decides whether the
    /// group lives.[^3]
    ///
    /// The rate never reaches the upkeep table. Upkeep is a rate above zero
    /// that subtracts, and this is production.[^4]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-124. `docs/FINDINGS.md`
    /// [^2]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    /// [^3]: ADR-0075, the founding choice reads a bounded sample of the world, decision D5. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    /// [^4]: ADR-0062, production and upkeep are rates attached to a site, decision D2. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    fn provision_site(&mut self, settlement: Entity, food: Amount) {
        // The count saturates at the range the fixed-point constructor
        // takes. A survey reads a bounded sample, so the food it reports is
        // bounded, and the saturation is a guard rather than a case.
        let reached = i16::try_from(food.0).unwrap_or(i16::MAX);
        let rate = sim_math::mul(self.need_rule.ration(), Fix32::from_int(reached));
        // The identity is live, because this call site founded it in the
        // same function. A refusal here is a programming error in the
        // founding, not a caller mistake.
        self.set_production_rate(settlement, CommodityId(0), rate)
            .expect("the rate is at or above zero and the commodity is in the set");
    }

    /// Seats a settlement at a place and spreads a group over its disc.
    ///
    /// The disc is walked in its fixed order, and each open tile takes up to
    /// the number of units that its ground holds.[^1] The order is the same
    /// on every run and at every thread count, because it is a function of
    /// the address alone.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub(super) fn settle_group(
        &mut self,
        place: Axial,
        group: u32,
        faction: FactionId,
    ) -> Result<(Entity, Vec<Entity>), FoundingError> {
        let settlement = self.found_settlement(place, faction)?;
        let mut people = Vec::with_capacity(group as usize);
        let mut remaining = group;
        for address in founding::disc(self.grid, place, founding::SURVEY_RADIUS) {
            if remaining == 0 {
                break;
            }
            let Some(kind) = self.tile_kind(address) else {
                continue;
            };
            if !kind.is_passable() {
                continue;
            }
            // The founding fills each tile to the capacity of its ground and
            // reads no occupancy count. A spawn does not read the capacity
            // either, and the derived occupancy structure is stale between
            // two frames, so a read of it here would be a third call site for
            // a rebuild that the step already owns.[^3] [^4] A second
            // founding over one disc may therefore over-fill a tile, which is
            // the caller mistake that decision permits and that movement
            // corrects, because admission never raises a tile above its
            // capacity.
            //
            // [^3]: Open decisions register, DEC-020. `docs/DECISIONS.md`
            // [^4]: Open decisions register, DEC-021. `docs/DECISIONS.md`
            for _ in 0..kind.capacity().min(remaining) {
                // A refusal here leaves a settlement standing and a part of
                // the group alive. The reservation makes that refusal
                // reachable, because a world whose unit reservation is below
                // the group runs out of slots part way through this
                // loop.[^6] Undo the founding rather than report a failure
                // over a world that half changed.
                //
                // [^6]: ADR-0084, the world reserves the unit columns at construction, decision D3. `docs/adrs/draft/adr-0084-the-world-reserves-the-unit-columns-at-construction.md`
                match self.spawn_soldier(address, faction) {
                    Ok(person) => people.push(person),
                    Err(error) => {
                        self.abandon_founding(people, settlement);
                        return Err(FoundingError::Person(error));
                    }
                }
                remaining -= 1;
            }
        }
        if remaining > 0 {
            // The eligibility rule says the disc holds the group, so this is
            // a disagreement between the rule and the placement rather than a
            // caller mistake. Report it and leave nothing half-founded.
            self.abandon_founding(people, settlement);
            return Err(FoundingError::NoPlaceFound(1));
        }
        // The group belongs to the settlement it founded. A unit draws from
        // the store of the site it belongs to, and a founding is the one
        // place today that puts a unit and a site together.[^5]
        //
        // [^5]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D2. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
        for person in &people {
            self.set_home_site(*person, Some(settlement));
        }
        Ok((settlement, people))
    }

    /// Undoes a founding that could not finish.
    ///
    /// A founding that stops part way leaves a settlement standing and a
    /// part of its group alive. Both are removed here, so a refused founding
    /// changes nothing that a caller can observe. This is the one place that
    /// undoes a founding, and every refusal after the settlement stands goes
    /// through it.
    fn abandon_founding(&mut self, people: Vec<Entity>, settlement: Entity) {
        for person in people {
            self.despawn_soldier(person);
        }
        self.destroy_settlement(settlement);
    }
}
