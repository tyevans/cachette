//! The choice pass that gives each unit an intent, and the weights it reads.
//!
//! The pass ranks the options of one unit and takes the highest. The weights,
//! the schedule, the need buckets and the explanation of one choice sit
//! together, because each of them shapes or reports the same ranking.

use super::carry::carry_class_of;
use super::errors::StepError;
use super::World;
use crate::choose::{self, ChoiceError, ChoiceExplanation, ChoiceSchedule, NeedBuckets};
use crate::slots::Slots;
use crate::types::{Entity, Fix32};

impl World {
    /// Writes the intent of every unit whose cell chooses on this frame.
    ///
    /// The pass is one operation over all units. Nothing loops over units
    /// outside the engine.[^1]
    ///
    /// **The pass walks the lattice, and it never walks the population.** It
    /// divides the level 1 cells into contiguous ranges, and each thread takes
    /// one range. A thread skips a cell that does not choose on this frame and
    /// a cell that holds no unit, so the deciding work follows the cell count
    /// and the population cannot raise it.[^7] The earlier shape collected
    /// every live unit into one list before any thread started, and that
    /// collect was serial and grew with the population.[^8]
    ///
    /// **The engine computes one answer once for every unit that would compute
    /// the same answer.**[^9] A cell holds one answer table over the buckets of
    /// need, and a unit reads the entry for its bucket. The table fills as a
    /// unit asks, so a cell never scores more buckets than it holds units, and
    /// it never scores more than the bucket count.[^10]
    ///
    /// **The pass writes the gather order in the same write as the intent.**
    /// One pass writes both, for the same units, on the same frame. A second
    /// stage that derived the order from the option would be a second writer
    /// of one column, and nothing would fail when the two disagreed.[^4] The
    /// kind that an option gathers comes from the option row, which is the one
    /// declaration of that map.[^5]
    ///
    /// A unit whose cell does not choose on this frame keeps the intent it
    /// held, and it keeps the gather order it held. A control-plane order
    /// therefore survives until the cell of that unit next chooses, and the
    /// choice then replaces it.[^6]
    ///
    /// A unit whose every option scores below the floor holds what it
    /// was doing, which is the case the floor exists for.[^2] It holds no
    /// intent, so it takes no gather order either.
    ///
    /// Each thread reads a range of the lattice and writes its own output
    /// slot. The join reads the slots in slot order, the cells of a slot rise,
    /// and the derived unit structure orders the units of a cell. So the
    /// result takes its order from the lattice and never from the thread that
    /// finished first.[^3]
    ///
    /// The apply walks that same order. It is the one part that touches every
    /// unit that chose, and applying an answer to a unit is per-unit by
    /// necessity.[^7]
    ///
    /// # Errors
    ///
    /// Returns an error when the caller asks for zero threads, and when the
    /// derived unit structure no longer describes the arena.
    ///
    /// # References
    ///
    /// [^1]: ADR-0010, Python is a control plane, and it never touches an entity one at a time. `docs/adrs/REGISTRY.md`
    /// [^2]: Findings register, FND-014. `docs/FINDINGS.md`
    /// [^3]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^4]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    /// [^5]: Findings register, FND-191. `docs/FINDINGS.md`
    /// [^6]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D1. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
    /// [^7]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decisions D1 and D3. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
    /// [^8]: Findings register, FND-252. `docs/FINDINGS.md`
    /// [^9]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decision D4. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
    /// [^10]: ADR-0098, the choice is decided for each cell and each bucket of need. `docs/adrs/draft/adr-0098-the-choice-is-decided-for-each-cell-and-each-bucket-of-need.md`
    pub(super) fn choose(&mut self, threads: usize) -> Result<(), StepError> {
        if threads == 0 {
            return Err(StepError::ZeroThreads);
        }
        // The occupancy bitplane and the block range are unguarded reads.
        // They answer from the last rebuild and cannot refuse a stale
        // question, so a pass that skipped a cell on a stale bitplane would
        // skip it in silence. Ask the guarded question once, here, before
        // any thread trusts the shape.
        self.bridge.describes(&self.soldiers)?;
        let layout = self.pyramid.layout();
        let cells = layout.block_count();
        if cells == 0 {
            return Ok(());
        }
        let frame = self.tick.0;
        let schedule = self.choice;
        let weights = &self.weights;
        let buckets = self.buckets;
        let mark = self.carry_mark;
        let pyramid = &self.pyramid;
        let bridge = &self.bridge;
        let soldiers = &self.soldiers;
        let chunk_len = (cells as usize).div_ceil(threads).max(1) as u32;
        let mut slots: Slots<Vec<(Entity, u8)>> =
            Slots::filled(threads, Vec::new()).map_err(|_| StepError::ZeroThreads)?;

        std::thread::scope(|scope| {
            let mut start = 0u32;
            for slot in slots.entries_mut() {
                if start >= cells {
                    break;
                }
                let end = start.saturating_add(chunk_len).min(cells);
                scope.spawn(move || {
                    let needs = soldiers.need_column();
                    let carries = soldiers.carry_column();
                    let homes = soldiers.home_column();
                    let mut chosen: Vec<(Entity, u8)> = Vec::new();
                    for cell in start..end {
                        // The stagger key is the level 1 cell. It is never
                        // the identity of the unit.
                        if !schedule.chooses_now(cell, frame) {
                            continue;
                        }
                        if !bridge.block_is_occupied(cell) {
                            continue;
                        }
                        let Some(summary) = pyramid.cell(cell) else {
                            continue;
                        };
                        let units = bridge
                            .in_block(soldiers, cell)
                            .expect("the caller checked that the bridge describes this arena");
                        let mut answers = choose::CellAnswers::new(summary, buckets);
                        for unit in units {
                            let slot = unit.index() as usize;
                            let need = needs[slot];
                            // The carry class is the third term of the key. It
                            // is a bounded class of the state of the unit
                            // itself, and it is not the load.[^17]
                            //
                            // [^17]: ADR-0109, the choice key holds a bounded class of the unit's own state, decision D1. `docs/adrs/draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
                            let carry = carry_class_of(carries[slot], homes[slot], mark);
                            chosen.push((*unit, answers.answer(need, carry, weights)));
                        }
                    }
                    *slot = chosen;
                });
                start = end;
            }
        });

        let chosen = slots.combine(Vec::new(), |mut joined, slot| {
            joined.extend_from_slice(slot);
            joined
        });
        for (unit, intent) in chosen {
            self.soldiers.set_intent_at(unit.index(), intent);
            // The same write. A unit that chose the option which gathers
            // holds an order for that kind, and a unit that chose anything
            // else, or nothing, holds none.
            self.soldiers
                .set_gather_order(unit, choose::gathers(intent));
        }
        Ok(())
    }

    /// Returns when each unit re-reads the world and chooses again.
    #[must_use]
    pub const fn choice_schedule(&self) -> ChoiceSchedule {
        self.choice
    }

    /// Returns how finely the choice tells two needs apart.
    #[must_use]
    pub const fn need_buckets(&self) -> NeedBuckets {
        self.buckets
    }

    /// Sets the width of a need bucket, as a power of two.
    ///
    /// **This changes what a unit does.** Two units whose needs share a bucket
    /// receive one answer, so a wider bucket makes two units of different need
    /// act alike and a narrower one approaches one answer for each unit.[^1]
    /// The reference table holds the value a world starts with and the
    /// derivation of it, and an open decision holds the choice of a better
    /// one.[^2] [^3]
    ///
    /// # Errors
    ///
    /// Returns an error when the exponent is outside the range that the answer
    /// table holds.
    ///
    /// # References
    ///
    /// [^1]: ADR-0098, the choice is decided for each cell and each bucket of need, decision D1. `docs/adrs/draft/adr-0098-the-choice-is-decided-for-each-cell-and-each-bucket-of-need.md`
    /// [^2]: Budgets and costs, the choice pass. `docs/reference/budgets.md`
    /// [^3]: Decisions register, DEC-097. `docs/DECISIONS.md`
    pub const fn set_need_buckets(&mut self, shift: u32) -> Result<(), ChoiceError> {
        match NeedBuckets::new(shift) {
            Ok(buckets) => {
                self.buckets = buckets;
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    /// Sets the interval between two choices, as a power of two.
    ///
    /// An exponent of zero makes every unit choose on every tick. The
    /// interval is a parameter of the world. This function holds no
    /// recommended value, and the reference table holds the derivation.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the exponent is above the ceiling.
    ///
    /// # References
    ///
    /// [^1]: Budgets and costs, the choice pass. `docs/reference/budgets.md`
    pub const fn set_choice_schedule(&mut self, period_log2: u32) -> Result<(), ChoiceError> {
        match ChoiceSchedule::new(period_log2) {
            Ok(schedule) => {
                self.choice = schedule;
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    /// Returns the weight that a unit puts on one option.
    ///
    /// Returns `None` when the option index is outside the set.
    #[must_use]
    pub const fn option_weight(&self, option: u8) -> Option<Fix32> {
        self.weights.weight(option)
    }

    /// Sets the weight that a unit puts on one option.
    ///
    /// The weight is content: a value in a table that the engine reads. The
    /// engine never calls content code inside the choice.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the option index is outside the set.
    ///
    /// # References
    ///
    /// [^1]: ADR-0007, content supplies a key vector, never a comparator, decision D3. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
    pub const fn set_option_weight(
        &mut self,
        option: u8,
        weight: Fix32,
    ) -> Result<(), ChoiceError> {
        self.weights.set(option, weight)
    }

    /// Returns the option that one soldier last chose.
    ///
    /// The outer option reports whether the identity is live. The inner one
    /// reports whether the soldier holds an intent. A soldier that holds
    /// none found nothing above the floor, and it does not move.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D3. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
    #[must_use]
    pub fn soldier_intent(&self, entity: Entity) -> Option<Option<u8>> {
        self.soldiers.intent(entity)
    }

    /// Returns why one soldier chose what it chose.
    ///
    /// The answer holds every score, the value each option read from the
    /// level 1 cell, the weight each option carried, and the floor that an
    /// option had to clear. The engine recomputes it from the world as it
    /// stands now, because it stores no score.[^1]
    ///
    /// Returns `None` when the identity is dead or names no tile of this
    /// world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D2. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
    #[must_use]
    pub fn explain_choice(&self, entity: Entity) -> Option<ChoiceExplanation> {
        let slot = self.soldiers.slot_of(entity)?;
        let tile = self.soldiers.tile(entity)?;
        let cell = self.cell_of(tile)?;
        let summary = self.pyramid.cell(cell)?;
        let need = self.soldiers.need_column()[slot as usize];
        let intent = self.soldiers.intent_column()[slot as usize];
        let carry = carry_class_of(
            self.soldiers.carry_column()[slot as usize],
            self.soldiers.home_column()[slot as usize],
            self.carry_mark,
        );
        Some(choose::explain(
            cell,
            choose::UnitState { need, carry },
            summary,
            &self.weights,
            self.buckets,
            intent,
            self.choice.chooses_now(cell, self.tick.0.wrapping_add(1)),
        ))
    }
}
