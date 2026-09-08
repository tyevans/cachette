//! The production queue of a site, and the pass that advances it.
//!
//! A queue holds the orders a site works through. The build cost table says
//! what an order costs, the store supplies it, and the pass moves work into
//! it each period. The cost, the store readers and the pass sit together.

use super::World;
use crate::production::{BuildCostRow, QueueEntry, QueueError, QueueOrder, WORK_PER_ADVANCE};
use crate::rates::RateSchedule;
use crate::sim_math;
use crate::site::{CommodityId, COMMODITY_COUNT};
use crate::soldier::NO_HOME;
use crate::types::{Entity, FactionId, Fix32};
use crate::unit_type::UnitTypeId;

impl World {
    /// Returns the queue of one site, in queue position order.
    ///
    /// Returns `None` when the identity names no site that stands. The order
    /// is the order the entries were queued, and nothing reorders them.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D1. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    #[must_use]
    pub fn site_queue(&self, site: Entity) -> Option<&[QueueEntry]> {
        let slot = self.settlements.slot_of(site)?;
        Some(self.queues.entries_of(slot))
    }

    /// Orders the queue of one site.
    ///
    /// **This is the one verb that reaches a queue.** A Python caller, the
    /// built-in controller and a learner all call it, and no other path
    /// writes an entry.[^1] The engine holds the mechanism, the bound and the
    /// refusals, and it holds no rule about what to queue.
    ///
    /// A push puts one entry of the named type at the back. A clear takes the
    /// entry at one position out and closes the gap, and the work the store
    /// already paid for is lost.
    ///
    /// **Every refusal is counted.** A refused order changes nothing, and the
    /// count of the refusals sits beside the count of what the queue
    /// produced, so a watcher reading a queue that never moves can tell the
    /// two apart.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the identity names no site that stands, when the
    /// site belongs to another faction, when the number names no row of the
    /// unit type table, when the queue already holds its bound, and when the
    /// position holds no entry.
    ///
    /// # References
    ///
    /// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D2. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    /// [^2]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D6. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    pub fn order_site_queue(
        &mut self,
        faction: FactionId,
        site: Entity,
        order: QueueOrder,
    ) -> Result<(), QueueError> {
        let outcome = self.take_queue_order(faction, site, order);
        if outcome.is_err() {
            self.queues.count_refused_at_the_verb();
        }
        outcome
    }

    /// Takes one queue order, and states every refusal.
    ///
    /// The verb above counts what this refuses. The two are apart so that the
    /// count sits at one place and no path can refuse without counting.
    fn take_queue_order(
        &mut self,
        faction: FactionId,
        site: Entity,
        order: QueueOrder,
    ) -> Result<(), QueueError> {
        let (Some(slot), Some(owner)) = (
            self.settlements.slot_of(site),
            self.settlements.faction(site),
        ) else {
            return Err(QueueError::NoSuchSite(site));
        };
        if owner != faction {
            return Err(QueueError::SiteBelongsToAnother {
                owner,
                asked: faction,
            });
        }
        self.queues.open_to(self.settlements.slot_count());
        match order {
            QueueOrder::Push(unit_type) => {
                if UnitTypeId::from_u8(unit_type.0).is_none() {
                    return Err(QueueError::TypeAboveCeiling(unit_type.0));
                }
                self.queues.push(slot, unit_type)
            }
            QueueOrder::Clear(position) => self.queues.remove(slot, position).map(|_| ()),
        }
    }

    /// Returns the build cost row of one unit type.
    #[must_use]
    pub const fn build_cost(&self, unit_type: UnitTypeId) -> BuildCostRow {
        self.build_costs.row(unit_type)
    }

    /// Writes the build cost row of one unit type.
    ///
    /// The costs are data that the world is built with, in the way the unit
    /// type table and the upgrade table are.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the number names no row of the unit type table.
    ///
    /// # References
    ///
    /// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D3. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    pub const fn define_build_cost(
        &mut self,
        unit_type: u8,
        row: BuildCostRow,
    ) -> Result<(), QueueError> {
        self.build_costs.define(unit_type, row)
    }

    /// Returns the entries one site may hold in its queue.
    ///
    /// The bound is a parameter of the world and never a function of the
    /// population.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D1. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    #[must_use]
    pub const fn queue_bound(&self) -> usize {
        self.queues.bound()
    }

    /// Sets the entries one site may hold in its queue.
    ///
    /// **A bound of zero turns the queue off.** The verb then refuses every
    /// push, no site holds an entry, and no unit is built. A test that wants
    /// the engine to leave its units alone sets it.
    ///
    /// Returns `false` when the bound is above the width of the stored block,
    /// which the balance register holds as a row.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the production queue, the queue bound row. `docs/reference/balance.md`
    pub const fn set_queue_bound(&mut self, bound: usize) -> bool {
        self.queues.set_bound(bound)
    }

    /// Returns when the queue advance acts.
    #[must_use]
    pub const fn queue_schedule(&self) -> RateSchedule {
        self.queue_schedule
    }

    /// Sets when the queue advance acts.
    pub const fn set_queue_schedule(&mut self, schedule: RateSchedule) {
        self.queue_schedule = schedule;
    }

    /// Sets the quantity of one good that one advance of a queue costs.
    ///
    /// Returns `false` when the commodity is outside the set.
    pub fn set_queue_charge(&mut self, commodity: CommodityId, quantity: Fix32) -> bool {
        self.queues.set_charge(commodity, quantity)
    }

    /// Returns how many units the queues produced on the last step.
    #[must_use]
    pub const fn queue_produced(&self) -> u32 {
        self.queues.produced()
    }

    /// Returns how many finished entries the last advance refused, because
    /// the site held no resident to spend.
    #[must_use]
    pub const fn queue_refused_without_a_person(&self) -> u32 {
        self.queues.refused_without_a_person()
    }

    /// Returns how many finished entries the last advance refused, because
    /// the store could not pay the goods.
    ///
    /// The two refusals are counted apart, because they mean different things
    /// to a watcher and to a learner.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D6. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
    #[must_use]
    pub const fn queue_refused_without_goods(&self) -> u32 {
        self.queues.refused_without_goods()
    }

    /// Returns how many queue orders the verb refused since the last advance.
    #[must_use]
    pub const fn queue_refused_at_the_verb(&self) -> u32 {
        self.queues.refused_at_the_verb()
    }

    /// Reports whether the store of one slot holds every good of a cost.
    pub(super) fn store_holds(&self, slot: u32, goods: &[Fix32; COMMODITY_COUNT]) -> bool {
        let Some(store) = self.settlements.store_column().get(slot as usize) else {
            return false;
        };
        goods.iter().enumerate().all(|(index, wanted)| {
            store
                .quantity(CommodityId(index as u16))
                .is_some_and(|held| held.0 >= wanted.0)
        })
    }

    /// Takes every good of a cost out of the store of one slot.
    ///
    /// The caller reads the store first, so no subtract here goes below zero.
    /// The write goes through the one path that keeps the account of the
    /// stores, so the conservation check still balances.
    pub(super) fn take_from_store(&mut self, slot: u32, goods: &[Fix32; COMMODITY_COUNT]) {
        for (index, wanted) in goods.iter().enumerate() {
            let commodity = CommodityId(index as u16);
            let Some(held) = self
                .settlements
                .store_column()
                .get(slot as usize)
                .and_then(|store| store.quantity(commodity))
            else {
                continue;
            };
            self.set_store_quantity(slot, commodity, sim_math::sub(held, *wanted));
        }
    }

    /// Advances the front entry of every queue, and applies what finished.
    ///
    /// # Where it runs, and why
    ///
    /// The stage runs after the shortage scan of this frame and before the
    /// barrier that follows it. It reads the store, so it runs after the rate
    /// pass and after the consumption pass, which are what move a quantity
    /// into and out of a store in this frame.[^1] [^2] It removes units and
    /// adds units, so it is a structural change, and the barrier below it is
    /// the barrier of that change.[^3]
    ///
    /// **It runs after the shortage scan and not before it.** The scan holds
    /// a plane of the slots it ends. A stage that freed a slot and filled it
    /// again before the scan applied would give the scan a live unit that it
    /// never marked.
    ///
    /// # The cost
    ///
    /// The first pass visits the sites and their front entries, so its cost
    /// follows the site count and never the population.[^4] **The second pass
    /// walks the unit arena once, and only on a tick where an entry
    /// finishes.** A finished entry must name the residents it takes, the
    /// residence of a unit is the home column it carries, and the engine
    /// stores no list of the residents of a site.[^5] One walk for the whole
    /// tick is the shape the controller stage already uses when it buckets
    /// the units of a faction for one command.[^6]
    ///
    /// # The order
    ///
    /// The first pass runs in site slot order. The second walks the units in
    /// ascending slot order and keeps the lowest slots of each site, so the
    /// residents an entry takes are fixed by the arena and not by a thread.
    /// The third applies in site slot order and then in queue position
    /// order.[^7]
    ///
    /// # References
    ///
    /// [^1]: ADR-0062, production and upkeep are rates attached to a site, decision D5. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    /// [^2]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D5. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
    /// [^3]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    /// [^4]: ADR-0096, cost follows the lattice, not the population, decision D1. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
    /// [^5]: ADR-0157, a site's free places are its built housing less the residents the engine already counts, decision D3. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
    /// [^6]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D1. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    /// [^7]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub(super) fn advance_queues(&mut self) {
        // The counts are a census of one tick, and this stage is where the
        // tick starts for them. The run total takes what they hold before
        // they are emptied.
        self.fold_queues_into_the_census();
        self.queues.clear_counts();
        if !self.queue_schedule.due(self.tick) {
            return;
        }
        self.queues.open_to(self.settlements.slot_count());
        let charge = *self.queues.charge();

        // Pass one. The sites, in slot order, and one front entry each.
        let mut finished: Vec<(u32, QueueEntry)> = Vec::new();
        for slot in 0..self.settlements.slot_count() {
            if self.settlements.entity_at(slot).is_none() {
                continue;
            }
            let Some(mut entry) = self.queues.front(slot) else {
                continue;
            };
            let work = self.build_costs.row(entry.unit_type).work;
            if entry.work < work {
                // **A queue is never free, and the store pays as the entry
                // advances.** A site whose store cannot pay makes no
                // progress, and its entry stays where it is. An entry that
                // has already reached its work costs nothing further,
                // because only an advance charges.[^8]
                //
                // [^8]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D3. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
                if !self.store_holds(slot, &charge) {
                    continue;
                }
                self.take_from_store(slot, &charge);
                // The accumulator is a whole number and it is clamped at the
                // work the entry needs, so a long build carries no credit
                // into the next entry.
                entry.work = entry.work.saturating_add(WORK_PER_ADVANCE).min(work);
                self.queues.set_front(slot, entry);
            }
            if entry.work >= work {
                finished.push((slot, entry));
            }
        }
        if finished.is_empty() {
            return;
        }

        // Pass two. One walk of the unit arena for the whole tick, which
        // buckets the residents of every site that finished an entry.
        let mut place = vec![usize::MAX; self.settlements.slot_count() as usize];
        for (index, (slot, _)) in finished.iter().enumerate() {
            place[*slot as usize] = index;
        }
        let mut residents: Vec<Vec<Entity>> = vec![Vec::new(); finished.len()];
        {
            let homes = self.soldiers.home_column();
            let owners = self.soldiers.faction_column();
            let live = self.soldiers.live_column();
            let sites = self.settlements.faction_column();
            for slot in 0..homes.len() {
                if live[slot] != 1 || homes[slot] == NO_HOME {
                    continue;
                }
                let Some(index) = place.get(homes[slot] as usize).copied() else {
                    continue;
                };
                if index == usize::MAX {
                    continue;
                }
                let (site_slot, entry) = finished[index];
                if sites[site_slot as usize] != owners[slot] {
                    continue;
                }
                let people = self.build_costs.row(entry.unit_type).people as usize;
                if residents[index].len() >= people {
                    continue;
                }
                let generation = self.soldiers.generation_of(slot as u32);
                if let Some(unit) = Entity::new(slot as u32, generation) {
                    residents[index].push(unit);
                }
            }
        }

        // Pass three. The completions apply in site slot order.
        for (index, (slot, entry)) in finished.iter().copied().enumerate() {
            let row = self.build_costs.row(entry.unit_type);
            // **A finished entry is refused when the site holds no spare
            // person or cannot pay the goods.** It is refused and not
            // discarded: the entry stays at the front of the queue. The two
            // reasons are counted apart.[^9]
            //
            // [^9]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decisions D4 and D6. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
            if residents[index].len() < row.people as usize {
                self.queues.count_without_a_person();
                continue;
            }
            if !self.store_holds(slot, &row.goods) {
                self.queues.count_without_goods();
                continue;
            }
            let Some(site) = self.settlements.entity_at(slot) else {
                continue;
            };
            let (Some(address), Some(faction)) = (
                self.settlements.address(site),
                self.settlements.faction(site),
            ) else {
                continue;
            };
            let taken = std::mem::take(&mut residents[index]);
            // **The unit that leaves and the unit that arrives hold distinct
            // identities**, so no reader confuses the two. The world removes
            // each resident through its own despawn, which accounts for what
            // the unit carried, so conservation still balances.[^10]
            //
            // The despawns run before the spawn, so the arena holds a free
            // slot whatever its capacity. The spawn can therefore refuse only
            // when the row takes no person at all, and the balance register
            // fixes the people of every row above zero.[^11]
            //
            // [^10]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D5. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
            // [^11]: Balance register, the production queue, the people row. `docs/reference/balance.md`
            for unit in &taken {
                self.despawn_soldier(*unit);
            }
            let Ok(made) = self.spawn_soldier(address, faction) else {
                debug_assert!(
                    taken.is_empty(),
                    "a despawn frees a slot, so a spawn after one cannot refuse"
                );
                continue;
            };
            self.soldiers.set_unit_type(made, entry.unit_type);
            self.set_home_site(made, Some(site));
            self.take_from_store(slot, &row.goods);
            let popped = self.queues.remove(slot, 0);
            debug_assert!(popped.is_ok(), "the front entry was read above");
            self.queues.count_produced();
        }

        // The cohort table summarises the home column, and this stage changed
        // that column. A table left stale would state a headcount that no
        // unit backs, and the invariant check refuses that state.[^12]
        //
        // [^12]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
        self.cohorts.rebuild(
            self.soldiers.home_column(),
            self.soldiers.faction_column(),
            self.soldiers.live_column(),
            self.settlements.slot_count(),
        );
    }

    /// Reports whether a faction already has a leader on order.
    ///
    /// A leader is a unit whose type row carries a command reach above zero.
    /// The scan walks the queue of every site of the faction, so its cost
    /// follows the site count and the queue bound, and never the
    /// population.[^1]
    ///
    /// The gate reads the type column and no per-faction flag, in the way
    /// every other reader of the capability does.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0096, cost follows the lattice, not the population, decision D1. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
    /// [^2]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D3. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    pub(super) fn leader_is_on_order(&self, faction: FactionId) -> bool {
        let sites = self.settlements.faction_column();
        (0..self.settlements.slot_count()).any(|slot| {
            self.settlements.entity_at(slot).is_some()
                && sites[slot as usize] == faction
                && self
                    .queues
                    .entries_of(slot)
                    .iter()
                    .any(|entry| self.unit_types.row(entry.unit_type).command_reach > 0)
        })
    }

    /// Returns the lowest-slot site of one faction whose queue has room.
    ///
    /// The scan walks the settlements in slot order and no unit, so its cost
    /// follows the site count.
    pub(super) fn controller_queue_site(&self, faction: FactionId) -> Option<Entity> {
        (0..self.settlements.slot_count())
            .find(|slot| {
                self.settlements.entity_at(*slot).is_some()
                    && self.settlements.faction_column()[*slot as usize] == faction
                    && self.queues.len_of(*slot) < self.queues.bound()
            })
            .and_then(|slot| self.settlements.entity_at(slot))
    }
}
