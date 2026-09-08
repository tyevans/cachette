//! What a unit carries, and the delivery that empties it into a store.
//!
//! A carried load has a class, and the class steers what the unit chooses to
//! do next. The delivery moves the load into the store of a site. The class
//! and the delivery sit together, because the class is what the delivery
//! clears.

use super::errors::StepError;
use super::gather::gather_order_of;
use super::World;
use crate::choose::CarryClass;
use crate::position::WORK_COMMODITY;
use crate::resource::{Amount, CarryLoad, ResourceKind, RESOURCE_KIND_COUNT};
use crate::sim_math;
use crate::soldier::NO_HOME;
use crate::sort::BoundedKey;
use crate::types::{Entity, Fix32};

/// The load at which a unit counts as laden, when the caller states none.
///
/// A unit that reaches the mark takes the option that carries its load home,
/// and a unit below it does not. **The value is a parameter of the world, and
/// no record sets it.** A low mark sends a unit home for almost nothing and
/// spends its whole life walking. A high mark keeps a unit in the field until
/// a deposit near it runs dry. The reference table holds the value and the
/// derivation of it.[^1] [^2]
///
/// # References
///
/// [^1]: ADR-0109, the choice key holds a bounded class of the unit's own state, decision D3. `docs/adrs/draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
/// [^2]: Budgets and costs, the choice pass. `docs/reference/budgets.md`
pub const CARRY_MARK_DEFAULT: Amount = Amount(32);

pub(super) fn carry_class_of(load: CarryLoad, home: u32, mark: Amount) -> CarryClass {
    if home == NO_HOME {
        return CarryClass::Free;
    }
    if load.total().0 < i64::from(mark.0) {
        return CarryClass::Free;
    }
    CarryClass::Laden
}

impl World {
    /// Returns what left the world in the hands of a dead unit, by kind.
    #[must_use]
    pub const fn departed_carry(&self) -> &[u64; RESOURCE_KIND_COUNT] {
        &self.departed
    }

    /// Returns what every delivery has moved into a store, for each kind.
    #[must_use]
    pub const fn delivered_carry(&self) -> &[u64; RESOURCE_KIND_COUNT] {
        &self.delivered
    }

    /// Returns the load at which a unit counts as laden.
    #[must_use]
    pub const fn carry_mark(&self) -> Amount {
        self.carry_mark
    }

    /// Sets the load at which a unit counts as laden.
    ///
    /// A mark of zero makes every unit that holds a home laden, whatever it
    /// carries.
    pub const fn set_carry_mark(&mut self, mark: Amount) {
        self.carry_mark = mark;
    }

    /// Returns the carry class of one unit.
    ///
    /// A unit is laden when it holds a home site and its load reaches the
    /// carry mark. **A unit with no home is never laden**, because the
    /// delivery moves a load into the store of a home site, and a unit that
    /// has none can deliver to nothing.[^1]
    ///
    /// Returns `None` when the identity is dead or names no unit of this
    /// world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0109, the choice key holds a bounded class of the unit's own state, decision D3. `docs/adrs/draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
    #[must_use]
    pub fn carry_class(&self, entity: Entity) -> Option<CarryClass> {
        let slot = self.soldiers.slot_of(entity)? as usize;
        Some(carry_class_of(
            self.soldiers.carry_column()[slot],
            self.soldiers.home_column()[slot],
            self.carry_mark,
        ))
    }

    /// Resolves every gather order of the frame in one pass.
    ///
    /// The resolve sorts the intents by the deposit they name, then by the
    /// identity of the unit. Each deposit then owns one contiguous segment,
    /// and the identity is the final key field so no two intents tie.[^1] The
    /// sort runs on one thread, so no result here takes its order from a
    /// thread that finished first.[^2]
    ///
    /// The resolve scans each segment in its sorted order and grants until the
    /// deposit is empty. A unit that reaches an empty deposit takes nothing
    /// and produces no event. One pass over the sorted intents resolves the
    /// whole set, so the cost follows the number of units that gather and not
    /// the number of deposits.[^3]
    ///
    /// **The resolve never locks a tile and never retries.** Two units that
    /// name one deposit sit in one segment, and the sort decides which of them
    /// takes the last of it.[^1]
    ///
    /// What leaves each deposit goes to the ledger, and the same amount goes
    /// into the load of the unit. The two writes come from one grant, so
    /// nothing is created and nothing is lost.[^4]
    ///
    /// # Errors
    ///
    /// Returns an error when the sort refuses the keys.
    ///
    /// # References
    ///
    /// [^1]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D2. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
    /// [^2]: ADR-0007, content supplies a key vector, never a comparator, decision D2. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
    /// [^3]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D1. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
    /// [^4]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D5. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    /// Moves the load of every unit that stands on the tile of its home site
    /// into the store of that site.
    ///
    /// **The resource loop had no sink.** A unit gathered into a carry column
    /// and no verb moved that load into a store, so the store of a site rose
    /// only by the fixed rate the founding set from the survey, and the ground
    /// the units stood on did not change it. The whole economy was a constant
    /// decided before the first frame.[^1]
    ///
    /// **A delivery is admitted by sort, then by transfer.** Two units of one
    /// site deliver into one store and the store saturates at its ceiling, so
    /// a saturating add is not order-free.[^2] The pass therefore orders the
    /// deliveries by the site and then by the identity of the unit, and it
    /// transfers in that order. That is the shape the gather resolve already
    /// uses against a deposit.[^3] [^4]
    ///
    /// **A load the store cannot hold stays in the carry.** A quantity that
    /// vanished without a record would break the conservation equality, and
    /// nothing would fail.[^2] The unit keeps the remainder and delivers it on
    /// a later tick.
    ///
    /// **The transfer moves whole units only.** A carry holds a whole number
    /// and a store holds a fixed-point quantity, so the room a store has may
    /// end between two whole numbers. The pass takes the whole part of that
    /// room, which converts exactly in both directions. A conversion that
    /// rounded would create or destroy a quantity.[^5]
    ///
    /// **The commodity comes from the declared map and never from a literal.**
    /// The engine already writes the number of a commodity at two sites, and a
    /// third literal would be one value in three places with nothing to fail
    /// when the copies disagree.[^6] [^7]
    ///
    /// The pass runs on the calling thread. It writes one store at a time in a
    /// stated order, so it names no thread and depends on no thread count.[^4]
    ///
    /// # Errors
    ///
    /// Returns an error when the ordering refuses to run.
    ///
    /// # References
    ///
    /// [^1]: ADR-0062, production and upkeep are rates attached to a site, decision D2. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    /// [^2]: ADR-0062, production and upkeep are rates attached to a site, decision D3. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    /// [^3]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D2. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
    /// [^4]: ADR-0004, iteration order is explicit, decisions D1, D3 and D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^5]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^6]: Findings register, FND-191. `docs/FINDINGS.md`
    /// [^7]: Decisions register, DEC-073. `docs/DECISIONS.md`
    pub(super) fn deliver(&mut self, threads: usize) -> Result<(), StepError> {
        let carriers = self.carriers_at_home(threads);
        if carriers.is_empty() {
            return Ok(());
        }
        let keys: Vec<BoundedKey> = carriers
            .iter()
            .map(|(unit, site)| BoundedKey::new(u64::from(*site), unit.to_bits()))
            .collect();
        let ceiling = u64::from(self.settlements.slot_count().saturating_sub(1));
        let order = gather_order_of(&keys, ceiling)?;

        for index in order {
            let (unit, site) = carriers[index as usize];
            let Some(load) = self.soldiers.carry(unit) else {
                continue;
            };
            for kind in ResourceKind::ALL {
                let held_by_unit = load.of(kind);
                if held_by_unit.0 == 0 {
                    continue;
                }
                let commodity = WORK_COMMODITY[kind.index()];
                let Some(held) = self
                    .settlements
                    .store_column()
                    .get(site as usize)
                    .and_then(|store| store.quantity(commodity))
                else {
                    continue;
                };
                // The room of the store, in whole units. The subtract cannot
                // go below zero because the ceiling is the largest value the
                // scale holds.
                let room = sim_math::sub(Fix32::MAX, held).to_int_floor();
                let moved = held_by_unit.0.min(u32::try_from(room).unwrap_or(0));
                if moved == 0 {
                    continue;
                }
                // The conversion is exact in both directions: a whole number
                // that the room admits fits the scale, and the scale holds it
                // with no fractional part.
                let quantity = Fix32::from_int(i16::try_from(moved).unwrap_or(i16::MAX));
                let moved = u32::try_from(quantity.to_int_floor()).unwrap_or(0);
                if moved == 0 {
                    continue;
                }
                let after = sim_math::add(held, quantity);
                if !self.set_store_quantity(site, commodity, after) {
                    continue;
                }
                self.soldiers.take_carry(unit, kind, Amount(moved));
                self.delivered[kind.index()] += u64::from(moved);
            }
        }
        Ok(())
    }
}
