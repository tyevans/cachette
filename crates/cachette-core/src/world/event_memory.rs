//! The advance of the decayed event history, and the stocks a share divides
//! by.
//!
//! The history keeps a decayed count of each kind of event for each faction,
//! and it names the faction that caused each event that has a cause.[^1] This
//! module is the one caller that moves it forward.
//!
//! **The advance runs once for each step, at the end of the step.** Every
//! event log of this engine holds one step and no more, because the step
//! empties each log before any system runs. A reader that sampled the logs at
//! its own pace would see the steps it sampled and would miss the rest, so the
//! collection cannot live outside the step.
//!
//! The advance runs after the controller, because the controller founds and
//! razes and both write a log that this pass reads. It runs on the calling
//! thread, and it reads each log in the order the log holds.[^2]
//!
//! # Why three passes deposit early
//!
//! Three passes cannot leave their kinds to a log at the end of the step. The
//! starvation log names a unit and no faction, and the unit is gone by the
//! time this pass runs. The gather log names a unit that a later meeting of
//! the same step may have ended. The conversion log is cleared by the pass
//! that fills it, at the start of the next step, so a conversion that a
//! control plane asked for between two steps would never reach this pass.
//! Each of the three therefore deposits its arrivals where the faction is
//! still known, and this pass folds them with the rest.
//!
//! # References
//!
//! [^1]: The event history. [`EventMemory`]
//! [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`

use super::World;
use crate::event_memory::{EventMemory, MemoryKind, Stock};
use crate::types::FactionId;

impl World {
    /// Returns the decayed event history of this world.
    ///
    /// **It is not public, because no caller outside the engine reads it
    /// yet.** The observation builder of this crate reads it, and publishing a
    /// reader that nothing invokes would ship an inert capability.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 3. `.agents/rules/recurring-defects.md`
    pub(crate) const fn event_memory(&self) -> &EventMemory {
        &self.event_memory
    }

    /// Returns the stock of one faction that a published share divides by.
    ///
    /// **A raw count of events is not comparable between two worlds.** Each
    /// kind of event names the stock it came out of, and this is the one site
    /// that reads each stock. A world of a different size and the same event
    /// rate therefore publishes the same share.[^1]
    ///
    /// The answer is at least one, so no caller divides by zero. A faction
    /// that holds none of the stock therefore reads the bound of the share
    /// rather than a refusal.
    ///
    /// # References
    ///
    /// [^1]: Research report 42, what a policy should be able to see, section 8.1. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
    pub(crate) fn memory_stock(&self, faction: FactionId, stock: Stock) -> i64 {
        let live_units = self
            .soldiers
            .population_by_faction()
            .get(usize::from(faction.0))
            .map_or(0i64, |count| i64::from(*count));
        let raw = match stock {
            Stock::LiveUnits => live_units,
            Stock::HeldTiles => self.holding.holding_of(faction),
            Stock::Sites => self.live_site_count(faction),
            Stock::Rivals => i64::from(self.config.faction_count.max(1)) - 1,
            Stock::CarryRoom => live_units.saturating_mul(self.widest_carry_capacity()),
        };
        raw.max(1)
    }

    /// Returns how many live sites one faction holds.
    ///
    /// The walk is over the site slots and never over the tiles or the units,
    /// so the cost follows the settlements of the world and nothing else.
    fn live_site_count(&self, faction: FactionId) -> i64 {
        let factions = self.settlements.faction_column();
        let live = self.settlements.live_column();
        let mut found = 0i64;
        for (at, held) in live.iter().enumerate() {
            if *held != 0 && factions.get(at) == Some(&faction) {
                found += 1;
            }
        }
        found
    }

    /// Returns the largest load any row of the unit type table admits.
    ///
    /// The value is a property of the table the world carries, so a gather
    /// share divides by a structural quantity of the world and not by a
    /// measured figure.
    fn widest_carry_capacity(&self) -> i64 {
        let mut widest = 0i64;
        for row in self.unit_types.rows() {
            widest = widest.max(i64::from(row.carry_capacity));
        }
        widest.max(1)
    }

    /// Folds the events of this step into the history, then moves it one step.
    ///
    /// **This is the one advance, and it runs on every step.** An advance that
    /// ran only on the steps a learner decides on would measure one step in as
    /// many as the learner skipped, and every value it published would stay
    /// plausible.
    ///
    /// The pass reads the grievance list, the burn log, the site take log, the
    /// upgrade collapse log and the relation crossing log. It reads each in
    /// the order the list holds, and it visits the factions in ascending seat
    /// order.[^1]
    ///
    /// **The step calls it last, after the controller.** The controller founds
    /// cities and razes them, and both write a log this pass reads. It also
    /// runs after the pass that rewrites the holder column, because it reads
    /// the held tile count that the column settled on.
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub(super) fn advance_event_memory(&mut self) {
        let mut memory: EventMemory = core::mem::take(&mut self.event_memory);
        for grievance in &self.grievances {
            let count = i64::from(grievance.count);
            memory.record_blamed(
                grievance.victim,
                grievance.killer,
                MemoryKind::OwnUnitsFelled,
                count,
            );
            memory.record_blamed(
                grievance.killer,
                grievance.victim,
                MemoryKind::RivalUnitsFelled,
                count,
            );
        }
        for event in &self.burned_log {
            memory.record(event.faction, MemoryKind::OwnUnitsBurned, 1);
        }
        for event in &self.taken_log {
            memory.record_blamed(event.from, event.to, MemoryKind::OwnSitesLost, 1);
            memory.record_blamed(event.to, event.from, MemoryKind::SitesTaken, 1);
        }
        for event in &self.collapsed_log {
            if let Some(holder) = event.holder.faction() {
                memory.record(holder, MemoryKind::OwnUpgradesLost, 1);
            }
        }
        for event in self.relations.log() {
            if event.is_declaration() {
                memory.record_blamed(
                    event.to_faction,
                    event.from_faction,
                    MemoryKind::RelationsFellAgainstMe,
                    1,
                );
            }
        }
        for seat in 0..self.config.faction_count.max(1) {
            let faction = FactionId(seat);
            memory.note_held(faction, self.holding.holding_of(faction));
        }
        memory.advance();
        self.event_memory = memory;
    }
}
