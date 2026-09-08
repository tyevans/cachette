//! The contest pass, and the log of the units that fell.
//!
//! A contest happens where units of factions at war share a tile. The pass
//! resolves it, and the log records who fell. The pass and its log sit
//! together, because a reader of one wants the other.

use super::errors::StepError;
use super::World;
use crate::cohort;
use crate::contest::{self, UnitFell};
use crate::types::Entity;

impl World {
    /// Returns the units that a meeting ended at the last resolution, in slot
    /// order.
    #[must_use]
    pub fn fell_log(&self) -> &[UnitFell] {
        &self.fell_log
    }

    /// Returns the fallen log as bytes.
    ///
    /// The thread-count equivalence test compares this slice byte for
    /// byte.[^1] The cast is safe because the event type is plain data.
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[must_use]
    pub fn fell_log_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.fell_log)
    }

    /// Resolves every meeting of this frame and ends the units that fell.
    ///
    /// The pass marks in parallel and applies in one ascending scan of the
    /// slots, so the deaths never follow a thread completion order.[^1]
    ///
    /// It runs a fixed amount of work for the world it is given. It holds no
    /// convergence test and no time budget.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D3. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    pub(super) fn contest(&mut self, threads: usize) -> Result<(), StepError> {
        self.fell_log.clear();
        // The structure the pass reads must describe the world it reads. The
        // movement of this frame has already passed its barrier, and nothing
        // between that barrier and here moves a unit.
        self.bridge.describes(&self.soldiers)?;
        contest::resolve(
            &self.unit_types,
            &self.relations,
            contest::DrawKey {
                seed: self.config.seed,
                tick: self.tick,
            },
            &self.soldiers,
            &self.bridge,
            &mut self.fell_plane,
            &mut self.grievances,
            threads,
        )?;
        let order = cohort::starved_order(&self.fell_plane, threads)?;
        if order.is_empty() {
            return Ok(());
        }
        let tick = self.tick;
        // A unit that fell lowers its faction toward the faction that killed
        // it. The list is summed by pair and sorted on it, so the writes
        // happen in pair order.[^3]
        //
        // [^3]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D3. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
        for grievance in &self.grievances {
            self.relations
                .on_units_fell(tick, grievance.victim, grievance.killer, grievance.count);
        }
        self.award_renown();
        for slot in order {
            let index = slot as usize;
            let tile = self.soldiers.tile_column()[index];
            let faction = self.soldiers.faction_column()[index];
            let unit_type = self.soldiers.type_column()[index];
            let generation = self.soldiers.generation_of(slot);
            let unit = Entity::new(slot, generation)
                .expect("a marked slot is live, so it holds a generation of one or more");
            self.fell_log.push(UnitFell::new(
                tick,
                unit.to_bits(),
                tile,
                faction,
                unit_type,
            ));
            let ended = self.despawn_soldier(unit);
            debug_assert!(ended, "a marked slot holds a live unit");
        }
        // The cohorts are derived from the home column of the units, and this
        // pass has just removed some of them. A table left as it was would
        // hold a headcount that no unit answers to, and the invariant check
        // refuses that state.
        self.cohorts.rebuild(
            self.soldiers.home_column(),
            self.soldiers.faction_column(),
            self.soldiers.live_column(),
            self.settlements.slot_count(),
        );
        Ok(())
    }
}
