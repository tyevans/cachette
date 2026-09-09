//! The gather order, the gather resolve, and the log they write.
//!
//! A gather takes stock from the tile a unit stands on. Several units can
//! stand on one deposit, so the resolve sorts the intents and admits them in
//! that order. The order, the intent and the rate sit together, because none
//! of them means anything without the others.

use super::errors::StepError;
use super::World;
use crate::event::ResourceTaken;
use crate::event_memory::MemoryKind;
use crate::resource::{ledger_key, Amount, ResourceKind, RESOURCE_KIND_COUNT};
use crate::sim_math;
use crate::slots::Slots;
use crate::soldier::SoldierArena;
#[cfg(not(feature = "probe-nondeterminism"))]
use crate::sort;
use crate::sort::{BoundedKey, SortError};
use crate::types::{Entity, TileIdx};
use crate::unit_type::UnitTypeId;
use crate::upgrade;

/// The amount that one unit takes from one tile in one step.
///
/// The rate is content. It is declared here until content exists, and the
/// register holds the open choice of its value.[^1]
///
/// The rate is high against the stock of a tile, so a full tile of gatherers
/// always empties a deposit and never divides it evenly. That is the case the
/// resolve exists for, and a lower rate would make the contested case rare
/// instead of ordinary.[^2]
///
/// # References
///
/// [^1]: Decisions register, DEC-022. `docs/DECISIONS.md`
/// [^2]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D1. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
const GATHER_RATE: u32 = 4;

/// What a unit takes in addition, in one tick, from wet ground.
///
/// The value sits here beside the ordinary rate, because both describe what
/// one gather takes and a second declaration elsewhere would be one fact in
/// two places.[^1] No measurement chose it, and a blocker holds the question
/// of what weather should be worth.[^2]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
/// [^2]: Blockers register, BLK-130. `docs/BLOCKERS.md`
pub(super) const WET_GATHER_BONUS: u32 = 2;

/// One gather order, ready for the resolve.
#[derive(Clone, Copy, Debug)]
struct GatherIntent {
    /// The unit that gathers.
    unit: Entity,
    /// The tile that the unit stands on.
    tile: TileIdx,
    /// The kind that the unit gathers.
    kind: ResourceKind,
    /// The type of the unit. The resolve reads the gather rate and the carry
    /// capacity of the row it indexes.
    unit_type: UnitTypeId,
}

/// Returns the order in which the resolve reads the gather intents.
///
/// The order is the key vector sort: by the tile and the kind together, then
/// by the identity of the unit.[^1] It depends on the key values alone, so it
/// is the same at any thread count, and it does not follow the slot order of
/// the arena.[^2]
///
/// # Errors
///
/// Returns an error when the sort refuses the keys.
///
/// # References
///
/// [^1]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D2. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
/// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D5. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
#[cfg(not(feature = "probe-nondeterminism"))]
pub(super) fn gather_order_of(keys: &[BoundedKey], ceiling: u64) -> Result<Vec<u32>, SortError> {
    sort::order_bounded(keys, ceiling)
}

/// Returns the gather intents in the order they arrived, which is a defect.
///
/// This is the perturbed build. The resolve reads the joined intent list
/// rather than the sorted one, so who empties a deposit depends on the order
/// the slots were joined in. The slot probe reverses that order, and the
/// reversal is visible only above one thread, so the thread-count test then
/// fails.
///
/// The whole point is that it must fail. A determinism test with no proven
/// failure mode is decoration.[^1]
///
/// # Errors
///
/// Never. The signature matches the sound build so that the caller does not
/// change.
///
/// # References
///
/// [^1]: Testing rules, section 1. `.claude/rules/testing.md`
#[cfg(feature = "probe-nondeterminism")]
pub(super) fn gather_order_of(keys: &[BoundedKey], _ceiling: u64) -> Result<Vec<u32>, SortError> {
    // A stable sort by the deposit alone. Each deposit still owns one
    // contiguous segment, which the resolve requires to scan a segment at all,
    // and within a segment the order is the order the intents arrived in.
    let mut order: Vec<u32> = (0..keys.len() as u32).collect();
    order.sort_by_key(|position| keys[*position as usize].order());
    Ok(order)
}

/// Returns the gather intent of each live soldier that carries an order.
///
/// The soldiers are read in slot order, each thread writes its own output
/// slot, and the join reads the slots in slot order. The result never depends
/// on thread completion order.[^1]
///
/// A soldier with no order gathers nothing and produces no intent, so a world
/// in which nobody was told to gather costs one pass over the live set.
///
/// # Errors
///
/// Returns an error when the caller asks for zero threads.
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
fn gather_intents(soldiers: &SoldierArena, threads: usize) -> Result<Vec<GatherIntent>, StepError> {
    let live: Vec<Entity> = soldiers.iter().collect();
    if live.is_empty() {
        return Ok(Vec::new());
    }
    let chunk_len = live.len().div_ceil(threads).max(1);
    let mut slots: Slots<Vec<GatherIntent>> =
        Slots::filled(threads, Vec::new()).map_err(|_| StepError::ZeroThreads)?;

    crate::parallel::fan_out_each(live.chunks(chunk_len).zip(slots.entries_mut()).map(
        move |(chunk, slot)| {
            move || {
                *slot = chunk
                    .iter()
                    .filter_map(|unit| {
                        let kind = soldiers.gather_order(*unit)??;
                        let tile = soldiers.tile(*unit)?;
                        let unit_type = soldiers.unit_type(*unit)?;
                        Some(GatherIntent {
                            unit: *unit,
                            tile,
                            kind,
                            unit_type,
                        })
                    })
                    .collect();
            }
        },
    ));

    Ok(slots.combine(Vec::new(), |mut joined, slot| {
        joined.extend_from_slice(slot);
        joined
    }))
}

impl World {
    /// Tells one soldier to gather a kind of resource.
    ///
    /// The soldier then takes from the tile it stands on, once in each step,
    /// until the caller stops it. Returns `false` when the identity is dead.
    ///
    /// The command names a unit and a kind. It never loops over a tile, and it
    /// runs no work of its own: the step resolves every order of the frame in
    /// one pass.[^1]
    ///
    /// **The order holds until the unit next chooses.** The choice pass is the
    /// engine writer of this column, and it writes the order of a unit only on
    /// the frame that the level 1 cell of that unit chooses.[^2] An order given
    /// here therefore survives the frames until then, and the choice replaces
    /// it when it comes round.
    ///
    /// # References
    ///
    /// [^1]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D1. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
    /// [^2]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D4. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
    pub fn order_gather(&mut self, entity: Entity, kind: ResourceKind) -> bool {
        self.soldiers.set_gather_order(entity, Some(kind))
    }

    /// Tells one soldier to stop gathering.
    ///
    /// Returns `false` when the identity is dead. The stop holds until the
    /// unit next chooses, in the same way an order does.
    pub fn stop_gather(&mut self, entity: Entity) -> bool {
        self.soldiers.set_gather_order(entity, None)
    }

    /// Returns the gather order of one soldier.
    ///
    /// The outer option reports whether the identity is live. The inner one
    /// reports whether the soldier gathers.
    #[must_use]
    pub fn gather_order(&self, entity: Entity) -> Option<Option<ResourceKind>> {
        self.soldiers.gather_order(entity)
    }

    /// Returns the gather events of the last step.
    ///
    /// One event reports one grant. A watcher reads the log to see a resource
    /// being taken.
    #[must_use]
    pub fn gather_log(&self) -> &[ResourceTaken] {
        &self.gather_log
    }

    /// Returns the gather events of the last step as bytes.
    ///
    /// The thread-count equivalence test compares this slice byte for
    /// byte.[^1] The cast is safe because the event type is plain data.
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[must_use]
    pub fn gather_log_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.gather_log)
    }

    pub(super) fn gather(&mut self, threads: usize) -> Result<(), StepError> {
        self.gather_log.clear();
        let intents = gather_intents(&self.soldiers, threads)?;
        if intents.is_empty() {
            return Ok(());
        }

        let keys: Vec<BoundedKey> = intents
            .iter()
            .map(|intent| {
                BoundedKey::new(ledger_key(intent.tile, intent.kind), intent.unit.to_bits())
            })
            .collect();
        let last = TileIdx(self.grid.tile_count().saturating_sub(1));
        let ceiling = ledger_key(last, ResourceKind::ALL[RESOURCE_KIND_COUNT - 1]);
        let order = gather_order_of(&keys, ceiling)?;

        let tick = self.tick;
        // The ascending run that the ledger merges. The sorted order is the
        // key order, so a run built while walking it is already ascending.
        let mut run: Vec<(u64, u32)> = Vec::new();
        let mut at = 0usize;
        while at < order.len() {
            let key = keys[order[at] as usize].order();
            let mut end = at;
            while end < order.len() && keys[order[end] as usize].order() == key {
                end += 1;
            }
            let first = intents[order[at] as usize];
            // The deposit is read once for the whole segment. The stock a tile
            // started with is generated, so reading it twice computes it
            // twice.
            let original = self
                .resources
                .original_at(first.tile, first.kind)
                .unwrap_or(Amount::ZERO);
            let mut left = original
                .0
                .saturating_sub(self.depletion.taken(first.tile, first.kind).0);
            // A finished upgrade raises what a unit takes in one tick. The
            // rate is read once for the whole segment, beside the deposit
            // that the segment draws from.[^1]
            //
            // [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D3. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
            let rate = upgrade::gather_rate_with(
                GATHER_RATE,
                self.upgrades.standing(first.tile, &self.upgrade_table),
            );
            // Wet ground yields more. The weather field is read once for the
            // whole segment, beside the deposit and the upgrade rate, and it
            // is read at the level 1 cell that covers the tile because that
            // is where the weather lives.[^2]
            //
            // The reader takes the ground as the solve of the previous frame
            // left it. The solve of this frame runs at the end of the step,
            // after level 1 rebuilds.[^3]
            //
            // [^2]: ADR-0143, wet ground yields more to a gatherer, decision D1. `docs/adrs/draft/adr-0143-wet-ground-yields-more-to-a-gatherer.md`
            // [^3]: ADR-0140, weather is a field over the level 1 cell lattice, decision D3. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
            let rate = rate.saturating_add(self.wet_bonus(first.tile));
            let mut granted = 0u32;
            for position in &order[at..end] {
                if left == 0 {
                    break;
                }
                let intent = intents[*position as usize];
                // The type of the unit scales the tile rate and caps the
                // load. A gather rate of zero takes nothing, and a load at
                // the carry capacity takes nothing more.[^4]
                //
                // **A unit that cannot gather keeps its order and takes
                // nothing.** The order is not refused here, because the
                // choice pass writes the same column from inside the step,
                // and a type can change after an order is given. The boundary
                // verb refuses the order for such a unit, so a caller learns
                // at once; the pass is what holds when the caller is the
                // engine.
                //
                // [^4]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decisions D1 and D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
                let row = self.unit_types.row(intent.unit_type);
                let unit_rate = sim_math::scale_amount(rate, row.gather_rate);
                let held = self
                    .soldiers
                    .carry(intent.unit)
                    .map_or(0i64, |load| load.total().0);
                let room = i64::from(row.carry_capacity).saturating_sub(held);
                let room = u32::try_from(room).unwrap_or(0);
                let amount = unit_rate.min(left).min(room);
                if amount == 0 {
                    continue;
                }
                left -= amount;
                granted += amount;
                let added = self
                    .soldiers
                    .add_carry(intent.unit, intent.kind, Amount(amount));
                debug_assert!(added, "the intent came from a live soldier");
                self.gather_log.push(ResourceTaken::new(
                    tick,
                    intent.unit.to_bits(),
                    intent.tile,
                    amount,
                    intent.kind.to_u8(),
                ));
                self.remember_the_take(intent.unit, amount);
            }
            if granted > 0 {
                run.push((key, granted));
            }
            at = end;
        }
        // The ledger comes out of the world for the merge, for the reason the
        // recovery pass takes it out: the merge ages each entry to this tick
        // first, and ageing reads the weather and the upgrade map.
        let mut depletion = core::mem::take(&mut self.depletion);
        depletion.merge_ascending(&run, tick, &|tile| self.tile_ground(tile));
        self.depletion = depletion;
        Ok(())
    }

    /// Records one take against the faction of the unit that took it.
    ///
    /// **The faction is read here, and not at the end of the step.** The
    /// gather event carries the unit and no faction, and a meeting later in
    /// the same step may end that unit. A reader at the end of the step would
    /// then ask the arena about a unit the arena no longer holds, and it would
    /// count nothing. This call therefore stands beside the log, where the
    /// faction column still names the faction of the slot.[^1]
    ///
    /// # References
    ///
    /// [^1]: The event history advance. [`World::advance_event_memory`]
    fn remember_the_take(&mut self, unit: Entity, amount: u32) {
        let faction = self.soldiers.faction_column()[unit.index() as usize];
        self.event_memory
            .record(faction, MemoryKind::ResourceGathered, i64::from(amount));
    }

    /// Gives every soldier in the set the order to gather one kind.
    ///
    /// **This is the set form, and it is the one path a caller and the
    /// controller share.** The Python binding resolves its identities and
    /// calls this. The controller calls this. A verb that only one of them
    /// reached would be a capability that nothing tests from the
    /// boundary.[^1]
    ///
    /// Returns how many entities the arena refused, because they name no
    /// live soldier.
    ///
    /// # References
    ///
    /// [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    pub fn order_gather_set(&mut self, units: &[Entity], kind: ResourceKind) -> usize {
        let mut refused = 0usize;
        for entity in units {
            if !self.order_gather(*entity, kind) {
                refused += 1;
            }
        }
        refused
    }
}
