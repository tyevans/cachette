//! The built-in controller: what it reads, what it decides, and what it logs.
//!
//! The controller plays a faction that no outside caller drives. It reads the
//! world through the same readers a control plane uses, and it commands
//! through the same verbs. Its settings, its trade work, its project work and
//! the one call that runs it sit together.

use super::World;
use crate::action::Verb;
use crate::controller::{
    self, CarrierAssignment, Choice, ControllerCommand, FactionState, FactionWeights,
};
use crate::hex::Axial;
use crate::plan::Project;
use crate::position::WORK_COMMODITY;
use crate::production::QueueOrder;
use crate::rates::{RateError, RateSchedule};
use crate::resource::{ResourceKind, RESOURCE_KIND_COUNT};
use crate::stage::{self, Stage};
use crate::trade::{Consideration, TradeRow, TRADE_OFFERED};
use crate::types::{Entity, FactionId, TileIdx};
use crate::unit_type::{UnitTypeId, UnitTypeRow, LEADER, UNIT_TYPE_COUNT};
use crate::upgrade::UpgradeCategory;

impl World {
    /// Returns the weight vector of one faction, or `None` when the world
    /// has no such faction.
    #[must_use]
    pub fn faction_weights(&self, faction: FactionId) -> Option<FactionWeights> {
        self.controller.row(faction).map(|row| row.weights)
    }

    /// Writes the whole weight vector of one faction.
    ///
    /// The weight vector of a faction is that faction's policy, and this is
    /// the one verb that writes it.[^1] The vector is simulated state and it
    /// enters the state hash, so a write parts two worlds on the next
    /// tick.[^2]
    ///
    /// Returns `false` and changes nothing when the world has no such
    /// faction, or when a weight lies outside the range the balance register
    /// holds.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0156, a faction's option weights are policy, set through one verb, decision D3. `docs/adrs/accepted/adr-0156-a-factions-option-weights-are-policy-set-through-one-verb.md`
    /// [^2]: ADR-0156, a faction's option weights are policy, set through one verb, decision D1. `docs/adrs/accepted/adr-0156-a-factions-option-weights-are-policy-set-through-one-verb.md`
    /// [^3]: Balance register, the weight vector range. `docs/reference/balance.md`
    pub fn set_faction_weights(&mut self, faction: FactionId, weights: FactionWeights) -> bool {
        self.controller.set_weights(faction, weights)
    }

    /// Sets the flag that says an external caller controls a faction.
    ///
    /// A faction under external control receives no evaluation from the
    /// controller.[^1] Returns `false` when the world has no such faction.
    ///
    /// # References
    ///
    /// [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D6. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    pub fn set_externally_controlled(&mut self, faction: FactionId, controlled: bool) -> bool {
        self.controller
            .set_externally_controlled(faction, controlled)
    }

    /// Returns whether an external caller controls a faction.
    ///
    /// Returns `None` when the world has no such faction.
    #[must_use]
    pub fn is_externally_controlled(&self, faction: FactionId) -> Option<bool> {
        self.controller
            .row(faction)
            .map(|row| row.externally_controlled != 0)
    }

    /// Returns how many evaluations the controller makes for one faction on
    /// one tick.
    #[must_use]
    pub const fn controller_evaluations(&self) -> u32 {
        self.controller.evaluations()
    }

    /// Sets how many evaluations the controller makes for one faction on one
    /// tick.
    ///
    /// The count is a balance value, and the register holds the row.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the controller evaluations per faction per tick. `docs/reference/balance.md`
    pub const fn set_controller_evaluations(&mut self, evaluations: u32) {
        self.controller.set_evaluations(evaluations);
    }

    /// Returns the advertisement schedule: how many ticks lie between two
    /// board writes, and the offset inside that period.
    #[must_use]
    pub fn advertisement_schedule(&self) -> (u32, u32) {
        let schedule = self.controller.advert_schedule();
        (schedule.period(), schedule.phase())
    }

    /// Sets the advertisement schedule.
    ///
    /// The period and the phase are balance values, and the register holds
    /// the row.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the period is zero, and when the period is above
    /// the range that the scaling multiply takes.
    ///
    /// # References
    ///
    /// [^1]: Balance register, the advertisement schedule. `docs/reference/balance.md`
    pub fn set_advertisement_schedule(&mut self, period: u32, phase: u32) -> Result<(), RateError> {
        let schedule =
            RateSchedule::new(period, phase).ok_or(RateError::PeriodOutsideRange(period))?;
        self.controller.set_advert_schedule(schedule);
        Ok(())
    }

    /// Returns the store above which a faction offers a good, and below which
    /// it wants one.
    #[must_use]
    pub const fn surplus_mark(&self) -> u32 {
        self.controller.surplus_mark()
    }

    /// Sets the surplus mark.
    ///
    /// The mark is a balance value, and the register holds the row.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the surplus mark. `docs/reference/balance.md`
    pub const fn set_surplus_mark(&mut self, mark: u32) {
        self.controller.set_surplus_mark(mark);
    }

    /// Returns how many carriers one faction assigns to one contract.
    #[must_use]
    pub const fn carriers_per_contract(&self) -> u32 {
        self.controller.contract_carriers()
    }

    /// Sets how many carriers one faction assigns to one contract.
    ///
    /// The count is a balance value, and the register holds the row.[^1] A
    /// count of zero assigns no carrier.
    ///
    /// # References
    ///
    /// [^1]: Balance register, the carriers per contract. `docs/reference/balance.md`
    pub const fn set_carriers_per_contract(&mut self, carriers: u32) {
        self.controller.set_contract_carriers(carriers);
    }

    /// Returns how many ticks a contract that the controller opens runs for.
    #[must_use]
    pub const fn contract_term(&self) -> u32 {
        self.controller.contract_term()
    }

    /// Sets how many ticks a contract that the controller opens runs for.
    ///
    /// The term is a balance value, and the register holds the row.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the contract term. `docs/reference/balance.md`
    pub const fn set_contract_term(&mut self, term: u32) {
        self.controller.set_contract_term(term);
    }

    /// Returns every carrier the controller has assigned, in faction order
    /// and then in contract order and then in identity order.
    ///
    /// The list is simulated state and not a log of one tick. A carrier stays
    /// in it until the contract settles or fails.
    #[must_use]
    pub fn carrier_assignments(&self) -> &[CarrierAssignment] {
        self.controller.carriers()
    }

    /// Returns the identity and the contract of every carrier the controller
    /// has assigned, in the order the carrier list holds.
    ///
    /// The list holds the identity as one integer, because it is plain data.
    /// This call resolves each one against the arena, so a caller never
    /// builds an identity of its own.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D2. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    #[must_use]
    pub fn carrier_units(&self) -> Vec<(Entity, u32, FactionId)> {
        self.controller
            .carriers()
            .iter()
            .filter_map(|entry| {
                let unit = Entity::from_bits(entry.unit)?;
                self.soldiers.slot_of(unit)?;
                Some((unit, entry.row, entry.faction))
            })
            .collect()
    }

    /// Returns the commands the controller emitted on the last step, in the
    /// order they applied.
    #[must_use]
    pub fn controller_log(&self) -> &[ControllerCommand] {
        self.controller.log()
    }

    /// Returns how many choices of the last step the action table could not
    /// express.
    ///
    /// **A choice the table refuses is counted and never dropped.** Its row
    /// of the log states zero in the encoded column, and its action column
    /// holds the no-op row. A caller that reads the action column alone
    /// therefore reads a refused choice as a decision to do nothing.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub const fn controller_unencodable(&self) -> u32 {
        self.controller.unencodable()
    }

    /// Returns how many relation moves the controller made through the verb
    /// on the tick the log holds.
    pub(super) fn relation_moves_of_the_log(&self) -> i64 {
        let schema = self.action_schema();
        self.controller
            .log()
            .iter()
            .filter(|command| {
                schema.verb_of(command.action) == Some(Verb::Relation) && command.applied != 0
            })
            .count() as i64
    }

    /// Returns the site that one faction trades from: its live settlement in
    /// the lowest slot.
    ///
    /// The walk is over the settlement slots in slot order, so the answer is
    /// a property of the storage and of no thread.[^1]
    ///
    /// Returns `None` when the faction holds no settlement.
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn trading_site_of(&self, faction: FactionId) -> Option<Entity> {
        self.settlements
            .iter()
            .find(|site| self.settlements.faction(*site) == Some(faction))
    }

    /// Returns what the sites of one faction hold of each good, as whole
    /// numbers.
    ///
    /// **The accumulator is wide.** A store at the ceiling of the scale,
    /// summed over every site of a faction, passes the range of a narrower
    /// integer, and an accumulator must not depend on that margin.[^1]
    ///
    /// The sites are visited in slot order, and the goods in index order.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0023, an aggregate combines exactly in any order, decision D2. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn faction_stores(&self, faction: FactionId) -> [i64; RESOURCE_KIND_COUNT] {
        let mut totals = [0i64; RESOURCE_KIND_COUNT];
        for site in self.settlements.iter() {
            if self.settlements.faction(site) != Some(faction) {
                continue;
            }
            let Some(store) = self.settlements.store(site) else {
                continue;
            };
            for kind in ResourceKind::ALL {
                let index = kind.index();
                let Some(held) = store.quantity(WORK_COMMODITY[index]) else {
                    continue;
                };
                totals[index] = totals[index].saturating_add(i64::from(held.to_int_floor()));
            }
        }
        totals
    }

    /// Rewrites the whole board of one faction from its site economies.
    ///
    /// The rows go through the write verb a Python caller calls, and the same
    /// refusals apply.[^1] The write replaces the whole board.[^2]
    ///
    /// Returns whether the verb took the write.
    ///
    /// # References
    ///
    /// [^1]: ADR-0149, a faction's trade board is simulated state that any faction may read, decision D5. `docs/adrs/accepted/adr-0149-a-factions-trade-board-is-simulated-state-that-any-faction-may-read.md`
    /// [^2]: ADR-0149, a faction's trade board is simulated state that any faction may read, decision D3. `docs/adrs/accepted/adr-0149-a-factions-trade-board-is-simulated-state-that-any-faction-may-read.md`
    pub(super) fn controller_write_board(&mut self, faction: FactionId, draw: u32) -> bool {
        let stores = self.faction_stores(faction);
        let mark = i64::from(self.controller.surplus_mark());
        let bound = usize::from(self.market.bound());
        let rows = controller::board_of(
            self.config.seed,
            self.tick,
            faction,
            draw,
            &stores,
            mark,
            bound,
        );
        if self.advertise(faction, &rows).is_err() {
            return false;
        }
        self.controller.count_board();
        true
    }

    /// Returns the pair that one faction answers this tick, and the row of
    /// that pair.
    ///
    /// The pair is the live negotiation with the lowest other faction in
    /// which this faction speaks next. The walk is over the faction
    /// identifiers in ascending order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub(super) fn controller_answer_due(
        &self,
        faction: FactionId,
    ) -> Option<(FactionId, TradeRow)> {
        for index in 0..self.config.faction_count.max(1) {
            let other = FactionId(index);
            if other == faction {
                continue;
            }
            let Ok((proposer, responder)) = self.live_orientation(faction, other) else {
                continue;
            };
            let Some(row) = self.trade.row(proposer, responder) else {
                continue;
            };
            if row.is_bound() || Self::turn_of(row, proposer, responder) != Some(faction) {
                continue;
            }
            return Some((other, row));
        }
        None
    }

    /// Returns the faction that one faction opens a negotiation with this
    /// tick, and the terms it opens with.
    ///
    /// The other boards are read in faction order. A pair in the war band and
    /// a pair that already holds a live negotiation are both passed over,
    /// because the verb refuses the first and this stage never opens a second
    /// negotiation with one pair.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D4. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    pub(super) fn controller_match_due(
        &self,
        faction: FactionId,
    ) -> Option<(FactionId, controller::Terms)> {
        let mine = self.market.board(faction);
        if mine.is_empty() {
            return None;
        }
        for index in 0..self.config.faction_count.max(1) {
            let other = FactionId(index);
            if other == faction || !self.relations.permits_offer(faction, other) {
                continue;
            }
            if self.live_orientation(faction, other).is_ok() {
                continue;
            }
            let Some(terms) = controller::match_boards(mine, self.market.board(other)) else {
                continue;
            };
            if terms.give_amount == 0 || terms.take_amount == 0 {
                continue;
            }
            return Some((other, terms));
        }
        None
    }

    /// Takes the one negotiation step of one faction on one tick.
    ///
    /// An answer to a live negotiation comes before a new offer, so a faction
    /// that owes an answer never opens a second pair while it owes one. Every
    /// act passes the verb a Python caller calls.[^1]
    ///
    /// **The price is the integer midpoint of the two asking quantities.** No
    /// draw decides it. The faction accepts when the counteroffer asks no
    /// more than its own board asked, and refuses otherwise.[^2]
    ///
    /// Returns whether a verb took the step.
    ///
    /// # References
    ///
    /// [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    /// [^2]: Balance register, the surplus mark. `docs/reference/balance.md`
    pub(super) fn controller_trade_step(&mut self, faction: FactionId) -> bool {
        if let Some((other, row)) = self.controller_answer_due(faction) {
            let own_ask = controller::asking_quantity_of(self.market.board(faction), row.take_kind);
            let Some(own_ask) = own_ask else {
                return self.refuse_trade(faction, other).is_ok();
            };
            if row.status == TRADE_OFFERED {
                // This faction answered the offer, so it restates the terms
                // at the midpoint of the two asks. The take side is restated
                // as it stands, because a counteroffer names both sides.
                let amount = controller::midpoint(row.give_amount, own_ask).max(1);
                let give = Consideration::resource(row.give_kind, amount);
                let take = Consideration::resource(row.take_kind, row.take_amount);
                return self
                    .counter_consideration(faction, other, give, take)
                    .is_ok();
            }
            if controller::accepts(row.give_amount, own_ask) {
                if self.accept_trade(faction, other).is_ok() {
                    self.controller.count_bound();
                    return true;
                }
                return false;
            }
            return self.refuse_trade(faction, other).is_ok();
        }
        let Some((other, terms)) = self.controller_match_due(faction) else {
            return false;
        };
        let term = self.controller.contract_term();
        let give = Consideration::resource(terms.give_kind, terms.give_amount);
        let take = Consideration::resource(terms.take_kind, terms.take_amount);
        if self
            .offer_consideration(faction, other, give, take, term)
            .is_err()
        {
            return false;
        }
        self.controller.count_offer();
        true
    }

    /// Returns every contract that one faction owes a carried quantity on,
    /// and the other party of each.
    ///
    /// The walk is over the negotiation plane in pair order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn controller_debts_of(&self, faction: FactionId) -> Vec<(u32, FactionId)> {
        let mut found = Vec::new();
        for (index, row) in self.trade.rows().iter().enumerate() {
            if !row.is_bound() {
                continue;
            }
            let (proposer, responder) = self.pair_of(index);
            let (owes, other) = if proposer == faction {
                (
                    row.proposer_side_is_carried() && row.owed_by_proposer() > 0,
                    responder,
                )
            } else if responder == faction {
                (
                    row.responder_side_is_carried() && row.owed_by_responder() > 0,
                    proposer,
                )
            } else {
                continue;
            };
            if owes {
                found.push((index as u32, other));
            }
        }
        found
    }

    /// Returns how many carriers one faction has on one contract.
    fn controller_carrier_count(&self, faction: FactionId, row: u32) -> u32 {
        self.controller
            .carriers()
            .iter()
            .filter(|entry| entry.faction == faction && entry.row == row)
            .count() as u32
    }

    /// Reports whether one faction has carrier work this tick.
    ///
    /// The answer is yes when it holds a carrier of a contract that ended,
    /// and yes when it owes a carried quantity on a contract that has fewer
    /// carriers than the balance row asks for.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the carriers per contract. `docs/reference/balance.md`
    pub(super) fn controller_carry_work(&self, faction: FactionId) -> bool {
        let want = self.controller.contract_carriers();
        let ended = self.controller.carriers().iter().any(|entry| {
            entry.faction == faction
                && !self
                    .trade
                    .row_at(entry.row as usize)
                    .is_some_and(|row| row.is_bound())
        });
        if ended {
            return true;
        }
        if want == 0 || self.campaigns.live(faction).is_some() {
            return false;
        }
        self.controller_debts_of(faction)
            .into_iter()
            .any(|(row, _)| self.controller_carrier_count(faction, row) < want)
    }

    pub(super) fn controller_take_projects(&mut self, faction: FactionId) -> bool {
        // **The rule lives in the partition, and this verb reads it.** The
        // legality answer reads the same partition.[^lg]
        //
        // [^lg]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
        let Some((standing, walking)) = self.project_partition(faction) else {
            return false;
        };
        let mut applied = false;
        if !walking.is_empty() {
            let plane = faction.0;
            if plane < self.destinations.plane_count() {
                // The seeds are the projects the units took. The send verb
                // sorts and deduplicates the set itself, so the order of this
                // list decides nothing.
                let seeds: Vec<Axial> = walking
                    .iter()
                    .filter_map(|(_, project)| self.grid.address_of(project.tile))
                    .collect();
                let set: Vec<Entity> = walking.iter().map(|(unit, _)| *unit).collect();
                applied |= self.send_units_to(&set, &seeds, plane).is_ok();
            }
        }
        // The build order names the category of the project the unit stands
        // on. The units are grouped by category, in category order, so each
        // call is the set form the boundary already exposes.
        for category in UpgradeCategory::ALL {
            let group: Vec<Entity> = standing
                .iter()
                .filter(|(_, held)| *held == category)
                .map(|(unit, _)| *unit)
                .collect();
            if group.is_empty() {
                continue;
            }
            let refused = self.order_build_set(&group, category);
            // The plan counts a project refusal beside the one the build
            // verb counts. The two counts were here before the check moved
            // out of the build verb, and this call keeps them.
            for _ in 0..refused {
                self.plan.count_refusal();
            }
            applied |= refused < group.len();
        }
        applied
    }

    /// Splits the units of one faction into the ones that stand on a project
    /// and the ones that would walk to one.
    ///
    /// **This is the one statement of the rule.** The project verb calls it
    /// before it moves a unit, and the legality answer calls it to fill one
    /// row of the action table.[^1] Nothing here mutates.
    ///
    /// Returns `None` when the faction has no project, when a campaign holds
    /// the destination plane, or when the carriers hold it.
    ///
    /// # References
    ///
    /// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
    #[allow(clippy::type_complexity)]
    pub(super) fn project_partition(
        &self,
        faction: FactionId,
    ) -> Option<(Vec<(Entity, UpgradeCategory)>, Vec<(Entity, Project)>)> {
        if self.plan.projects_of(faction).is_empty() {
            return None;
        }
        if self.campaigns.live(faction).is_some()
            || self
                .controller
                .carriers()
                .iter()
                .any(|entry| entry.faction == faction)
        {
            return None;
        }
        let mut units: Vec<Entity> = self.soldiers.iter_faction(faction).collect();
        units.sort_unstable_by_key(|unit| unit.to_bits());
        // **The order does two things, and which one a unit gets depends on
        // where it stands.** A unit that already stands on a project of its
        // faction takes the build order, because the build verb refuses a
        // tile no project zones. Every other idle unit is sent toward the
        // project nearest to it. A unit that is neither is left alone.
        let mut standing: Vec<(Entity, UpgradeCategory)> = Vec::new();
        let mut walking: Vec<(Entity, Project)> = Vec::new();
        for unit in units {
            let Some(tile) = self.soldiers.tile(unit) else {
                continue;
            };
            if let Some(category) = self.plan.zones(faction, tile) {
                standing.push((unit, category));
                continue;
            }
            if self.soldiers.sent(unit) != Some(None) {
                continue;
            }
            // **A unit that carries a water crossing takes no project.** The
            // crossing order is the one order that spends such a unit well,
            // and it applies after this one, so a project order that swept up
            // the mariners of an island faction would take them on the tick
            // each one was built and the faction would never leave its
            // island. The rule has the shape of the one the campaign keeps
            // for a unit that carries command reach.
            //
            // The unit is not idle in the sense of doing nothing. A mariner
            // that stands on a project of its faction still takes the build
            // order above, because that branch reads where the unit stands
            // and not what it is.
            if self
                .soldiers
                .unit_type(unit)
                .is_some_and(|unit_type| self.unit_types.row(unit_type).water_crossing > 0)
            {
                continue;
            }
            if let Some(project) = self.project_for(faction, unit) {
                walking.push((unit, project));
            }
        }
        Some((standing, walking))
    }

    pub(super) fn controller_carriers(&mut self, faction: FactionId) -> bool {
        let mut kept: Vec<CarrierAssignment> = Vec::new();
        let mut released: Vec<Entity> = Vec::new();
        for entry in self.controller.carriers() {
            let unit = Entity::from_bits(entry.unit);
            let live = unit.is_some_and(|unit| self.soldiers.slot_of(unit).is_some());
            let bound = self
                .trade
                .row_at(entry.row as usize)
                .is_some_and(|row| row.is_bound());
            if entry.faction != faction || (bound && live) {
                if live {
                    kept.push(*entry);
                }
                continue;
            }
            if let (true, Some(unit)) = (live, unit) {
                released.push(unit);
            }
        }
        let mut acted = !released.is_empty();
        if !released.is_empty() {
            // Every unit in the list is live, because the scan above kept
            // only the live ones, so the stop verb refuses nothing.
            let _ = self.stop_sending(&released);
        }
        let want = self.controller.contract_carriers();
        let mut assigned = 0u32;
        // The destination plane of a faction is the faction number, and a
        // campaign takes the same plane. A faction that holds a live campaign
        // therefore assigns no carrier, and a faction that holds a carrier
        // raises no campaign. One plane serves one purpose at a time.
        if want > 0 && self.campaigns.live(faction).is_none() {
            let plane = faction.0;
            for (row, other) in self.controller_debts_of(faction) {
                let held = kept
                    .iter()
                    .filter(|entry| entry.faction == faction && entry.row == row)
                    .count() as u32;
                if held >= want {
                    continue;
                }
                let (Some(home), Some(theirs)) =
                    (self.trading_site_of(faction), self.trading_site_of(other))
                else {
                    continue;
                };
                let Some(address) = self
                    .settlements
                    .tile(theirs)
                    .and_then(|tile| self.grid.address_of(tile))
                else {
                    continue;
                };
                let mut idle: Vec<Entity> = self
                    .soldiers
                    .iter_faction(faction)
                    .filter(|unit| self.soldiers.sent(*unit) == Some(None))
                    .filter(|unit| {
                        self.soldiers.unit_type(*unit).is_some_and(|unit_type| {
                            self.unit_types.row(unit_type).carry_capacity > 0
                        })
                    })
                    .collect();
                idle.sort_unstable_by_key(|unit| unit.to_bits());
                idle.truncate((want - held) as usize);
                if idle.is_empty() {
                    continue;
                }
                for unit in &idle {
                    self.set_home_site(*unit, Some(home));
                }
                if self.send_units_to(&idle, &[address], plane).is_err() {
                    continue;
                }
                for unit in &idle {
                    kept.push(CarrierAssignment::new(*unit, row, faction));
                }
                assigned = assigned.saturating_add(idle.len() as u32);
                acted = true;
            }
        }
        self.controller.set_carriers(kept);
        self.controller.count_carriers(assigned);
        acted
    }

    /// Runs the controller stage.
    ///
    /// The readers run first, while the record is empty. Then the controller
    /// plans the commands of the tick, the plan is sorted by faction and
    /// sequence, and each command applies through the set form of the verb
    /// a Python caller uses.[^1] [^2]
    ///
    /// The unit set of a faction is read once for each faction that emitted
    /// a command, in one scan of the arena in slot order. That scan follows
    /// the population, as the verb it feeds does when a Python caller names
    /// the same set.
    ///
    /// # References
    ///
    /// [^1]: ADR-0148, a game end is recorded once and stops the controllers, decisions D2 and D4. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
    /// [^2]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decisions D2, D4 and D5. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    pub(super) fn run_controller(&mut self) {
        // The run total takes what the two logs hold before they are
        // emptied, so a census row says what the run did and not what the
        // last tick did.[^5]
        //
        // [^5]: Findings register, FND-498. `docs/FINDINGS.md`
        let prologue_span = stage::open(Stage::ControllerPrologue);
        self.fold_the_controller_into_the_census();
        self.fold_campaigns_into_the_census();
        self.controller.clear_log();
        self.campaigns.clear_log();
        self.check_game_end();
        let tick = self.tick;
        let factions = usize::from(self.config.faction_count.max(1));
        // One scan of the arena, in slot order, finds the speaker of each
        // faction: its lowest-slot live unit whose type has command reach.
        // The gate reads the type column of the units and no flag.[^3]
        //
        // The same scan gathers the cohort of each faction: the live units
        // sent on the plane whose number is the faction number. No second
        // scan is made for it.
        //
        // [^3]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D3. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
        let mut speakers: Vec<Option<Entity>> = vec![None; factions];
        let mut cohorts: Vec<Vec<Entity>> = vec![Vec::new(); factions];
        // The same scan reports which factions hold a settler. The gate reads
        // the settle column of the type of each unit and no flag.[^3]
        let mut settlers: Vec<bool> = vec![false; factions];
        for entity in self.soldiers.iter() {
            let (Some(faction), Some(unit_type)) = (
                self.soldiers.faction(entity),
                self.soldiers.unit_type(entity),
            ) else {
                continue;
            };
            let index = usize::from(faction.0);
            if index >= factions {
                continue;
            }
            if speakers[index].is_none() && self.unit_types.row(unit_type).command_reach > 0 {
                speakers[index] = Some(entity);
            }
            if self.unit_types.row(unit_type).settle_group > 0 {
                settlers[index] = true;
            }
            if self.soldiers.sent(entity) == Some(Some(faction.0)) {
                cohorts[index].push(entity);
            }
        }
        self.close_campaigns(&cohorts);
        let objectives = self.campaign_objectives();
        drop(prologue_span);
        // The rival of a faction is the other faction with the most held
        // tiles. A faction with no speaker has no rival, because the verb
        // would refuse it.
        let held: Vec<(FactionId, i64)> = (0..factions as u16)
            .map(|index| (FactionId(index), self.holding.holding_of(FactionId(index))))
            .collect();
        let rivals: Vec<Option<FactionId>> = (0..factions)
            .map(|index| {
                speakers[index]?;
                controller::rival_of(FactionId(index as u16), held.iter().copied())
            })
            .collect();
        // A faction that holds a carrier raises no campaign, because the
        // campaign takes the destination plane the carriers climb.
        let objectives: Vec<Option<(u8, TileIdx)>> = objectives
            .into_iter()
            .enumerate()
            .map(|(index, objective)| {
                let faction = FactionId(index as u16);
                if self
                    .controller
                    .carriers()
                    .iter()
                    .any(|entry| entry.faction == faction)
                {
                    None
                } else {
                    objective
                }
            })
            .collect();
        // The three trade commands are pushed only when there is work. A
        // faction that has nothing to advertise, nothing to say and no
        // carrier to move emits nothing, so an idle world costs no command.
        // **The solver runs before the commands are planned.** It writes the
        // plan of every faction the controller evaluates, in faction order,
        // and the project order below then reads what it wrote.[^7]
        //
        // [^7]: ADR-0152, a faction plans its roads and zones with one solver, decisions D2 and D5. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
        {
            let _span = stage::open(Stage::ControllerSolvePlan);
            for index in 0..factions {
                let faction = FactionId(index as u16);
                // A faction under external control and a faction with no seat
                // receive no evaluation, so neither gets a plan.[^8]
                //
                // [^8]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decisions D6 and D7. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
                let evaluated = self
                    .controller
                    .row(faction)
                    .is_some_and(|row| row.externally_controlled == 0 && row.seat().is_some());
                if !evaluated {
                    continue;
                }
                self.solve_plan(faction);
            }
        }
        // The rows the unit type table fills. A row whose every column is
        // zero can do nothing, so the controller does not offer it. The list
        // is read from the table, and no rule here names a type.[^10]
        //
        // [^10]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D2. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
        let offered: Vec<UnitTypeId> = (0..UNIT_TYPE_COUNT)
            .map(|index| UnitTypeId(index as u8))
            .filter(|unit_type| self.unit_types.row(*unit_type) != UnitTypeRow::NONE)
            .collect();
        let queue_draw = self.controller.queue_draw_index();
        let due = self.controller.board_due(tick);
        let states_span = stage::open(Stage::ControllerStates);
        let states: Vec<FactionState> = (0..factions)
            .map(|index| {
                let faction = FactionId(index as u16);
                FactionState {
                    rival: rivals.get(index).copied().flatten(),
                    objective: objectives.get(index).copied().flatten(),
                    board_due: due && self.trading_site_of(faction).is_some(),
                    trade_due: self.controller_answer_due(faction).is_some()
                        || self.controller_match_due(faction).is_some(),
                    carry_due: self.controller_carry_work(faction),
                    // A faction with a march to make marches. The campaign
                    // and the project order take the same idle units and the
                    // same destination plane, so one of the two must yield,
                    // and the campaign is the one the war weight asked
                    // for.[^9]
                    //
                    // [^9]: ADR-0152, a faction plans its roads and zones with one solver, decision D5. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
                    project_due: !self.plan.projects_of(faction).is_empty()
                        && objectives.get(index).copied().flatten().is_none(),
                    // A faction that owns no site with room in its queue
                    // queues nothing. The type comes from one keyed draw over
                    // the rows the table fills.[^10]
                    //
                    // **A faction with no speaker queues a leader instead of
                    // the draw.** Command reach sits on the leader row alone,
                    // and a faction with no unit that carries it moves no
                    // relation. It therefore never reaches the war band, never
                    // gets an objective, and never marches. The draw offers a
                    // leader one time in four, so a faction could run a whole
                    // game without one.
                    //
                    // The want is not a standing rule. It falls away as soon
                    // as a leader stands or a leader is on order, so a faction
                    // that has one queues by the draw again.
                    // **A faction crosses water only when it holds a unit
                    // that can.** The world asks before it plans, so a
                    // faction with no mariner reads no sample and the
                    // bounded survey costs an idle world nothing.
                    cross_to: self.controller_crossing_target(faction),
                    queue_type: self.controller_queue_site(faction).and_then(|_| {
                        if speakers.get(index).copied().flatten().is_none()
                            && !self.leader_is_on_order(faction)
                        {
                            return Some(LEADER);
                        }
                        controller::queued_type_of(
                            self.config.seed,
                            tick,
                            faction,
                            queue_draw,
                            &offered,
                        )
                    }),
                    // A faction that holds no settler founds nothing, so it
                    // emits no settle command and draws nothing for one.[^12]
                    //
                    // [^12]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D5. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
                    settle_due: settlers.get(index).copied().unwrap_or(false),
                }
            })
            .collect();
        drop(states_span);
        let plan = {
            let _span = stage::open(Stage::ControllerPlan);
            self.controller.plan(self.config.seed, tick, &states)
        };
        if plan.is_empty() {
            return;
        }
        let _apply_span = stage::open(Stage::ControllerApply);
        // One scan of the arena, in slot order, buckets the live units by
        // faction. Only a faction that emitted a command gets a bucket.
        let mut wanted = vec![false; factions];
        for (faction, _, _) in &plan {
            wanted[usize::from(faction.0)] = true;
        }
        let mut sets: Vec<Vec<Entity>> = vec![Vec::new(); factions];
        for entity in self.soldiers.iter() {
            let Some(faction) = self.soldiers.faction(entity) else {
                continue;
            };
            let index = usize::from(faction.0);
            if index < factions && wanted[index] {
                sets[index].push(entity);
            }
        }
        let schema = self.action_schema();
        for (faction, sequence, choice) in plan {
            let set = std::mem::take(&mut sets[usize::from(faction.0)]);
            let applied = match choice {
                Choice::Gather(kind) => {
                    let _span = stage::open(Stage::ControllerOrderSet);
                    self.order_gather_set(&set, kind) < set.len()
                }
                Choice::Build(kind) => {
                    let _span = stage::open(Stage::ControllerOrderSet);
                    self.order_build_set(&set, kind) < set.len()
                }
                // The relation move goes through the same verb a caller
                // uses, with the speaker the scan above found. A faction
                // with no speaker planned no move, so the refusal here is
                // the verb's own.[^4]
                //
                // [^4]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decisions D2 and D3. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
                Choice::Relation(other) => {
                    let _span = stage::open(Stage::ControllerRelation);
                    speakers[usize::from(faction.0)].is_some_and(|speaker| {
                        self.move_relation(speaker, other, controller::RELATION_STEP)
                            .is_ok()
                    })
                }
                // The raise goes through the one core function a caller
                // uses. The cohort size is a balance value.[^5]
                //
                // [^5]: Balance register, the campaign cohort size. `docs/reference/balance.md`
                Choice::Campaign { tile, .. } => {
                    let _span = stage::open(Stage::ControllerCampaign);
                    let cohort = self.campaigns.cohort_size();
                    self.grid.address_of(tile).is_some_and(|address| {
                        self.raise_campaign(faction, address, cohort).is_ok()
                    })
                }
                // The board write, the negotiation step and the carriers all
                // pass the verbs a Python caller calls, and each reads the
                // world as the commands before it in this plan left it.[^6]
                //
                // [^6]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decisions D2 and D5. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
                Choice::Advertise => {
                    let _span = stage::open(Stage::ControllerTrade);
                    self.controller_write_board(faction, sequence)
                }
                Choice::Trade => {
                    let _span = stage::open(Stage::ControllerTrade);
                    self.controller_trade_step(faction)
                }
                Choice::Carry => {
                    let _span = stage::open(Stage::ControllerTrade);
                    self.controller_carriers(faction)
                }
                Choice::Project => {
                    let _span = stage::open(Stage::ControllerProject);
                    self.controller_take_projects(faction)
                }
                // The queue order goes through the one verb a Python caller
                // calls. The site is the lowest-slot site of the faction
                // whose queue has room, and the verb counts a refusal.[^11]
                //
                // [^11]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decisions D2 and D6. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
                Choice::Queue(unit_type) => {
                    let _span = stage::open(Stage::ControllerQueue);
                    let site = self.controller_queue_site(faction);
                    site.is_some_and(|site| {
                        self.order_site_queue(faction, site, QueueOrder::Push(unit_type))
                            .is_ok()
                    })
                }
                // The crossing order goes through the send verb a Python
                // caller calls, with the tile the world chose before it
                // planned.[^12]
                //
                // [^12]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
                Choice::Cross(tile) => {
                    let _span = stage::open(Stage::ControllerCross);
                    self.controller_cross(faction, tile)
                }
                // **The settle order reads the arena and not the set above.**
                // The set is taken by the first command a faction emits, and
                // the settle order draws last, so it would read an empty set
                // on every tick a faction gathered or built. The order takes
                // the settlers of the faction from the arena instead.[^13]
                //
                // [^13]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D5. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
                Choice::Settle => {
                    let _span = stage::open(Stage::ControllerSettle);
                    self.controller_settle(faction)
                }
            };
            let applied = u8::from(applied);
            sets[usize::from(faction.0)] = set;
            // **The row carries the whole action.** A controller choice and
            // a learner action reach one column in one encoding, so no
            // field of either lives in a second log.[^14]
            //
            // [^14]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D6. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
            // **A choice the table cannot express is counted, not dropped.**
            // The row states zero in its encoded column, and the stage keeps
            // the count of the tick. A row that stated the no-op and nothing
            // else would be read as a controller that chose to do
            // nothing.[^15]
            //
            // [^15]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
            let encoded = choice.action(&schema);
            self.controller.push(ControllerCommand {
                tick,
                action: encoded.unwrap_or(0),
                sequence,
                faction,
                applied,
                encoded: u8::from(encoded.is_some()),
                padding: [0; 4],
            });
        }
    }
}
