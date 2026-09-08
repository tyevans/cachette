//! The checks that say whether the world holds its own invariants.
//!
//! Each check reads one column against another and reports whether the two
//! agree. The checks sit together because none of them changes the world, and
//! a reader who wants to know what the world promises reads them as a set.

use super::World;
use crate::bridge::BridgeError;
use crate::cohort::NeedCondition;
use crate::resource::{ResourceKind, RESOURCE_KIND_COUNT};
use crate::sim_math;
use crate::site::{CommodityId, COMMODITY_COUNT};
use crate::types::{Accum, Entity, FactionId, TileIdx, FACTION_CEILING};

impl World {
    /// Reports whether the world holds its invariants.
    ///
    /// The Python state machine calls this method after every rule.[^1]
    ///
    /// # References
    ///
    /// [^1]: The testing rule, drive the real caller. `.claude/rules/testing.md`
    #[must_use]
    pub fn check_invariants(&self) -> bool {
        // The field holds one entry for each tile a frame changed, in
        // ascending tile order. A lookup is a binary search, so an entry out
        // of order does not fail. It returns the wrong tile.
        if !self.values.check_invariants() {
            return false;
        }
        // Both sets of the fire are ascending and hold each tile once, and no
        // tile is both burning and spent. A lookup is a binary search, so an
        // entry out of order does not fail. It answers about the wrong tile.
        if !self.fire.check_invariants() {
            return false;
        }
        if self.values.grid() != self.grid {
            return false;
        }
        if self.grid.width() != self.config.width || self.grid.height() != self.config.height {
            return false;
        }
        // The influence lattice and the block layout state the shape of
        // level 1, and they state it in two places. This is what fails when
        // the two disagree.[^1]
        //
        // [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
        if self.influence.cells().tile_count() != self.pyramid.layout().block_count()
            || self.influence.faction_count() != self.config.faction_count
        {
            return false;
        }

        // The upgrade map rises, names each tile once, names a tile inside
        // the world, and banks no progress beyond the work that the row
        // above each entry asks for.[^2] The table holds a row for every
        // level an entry stands at.
        //
        // [^2]: Findings register, FND-011. `docs/FINDINGS.md`
        if !self
            .upgrades
            .check_invariants(self.grid.tile_count(), &self.upgrade_table)
        {
            return false;
        }
        if !self.upgrade_table.check_invariants() {
            return false;
        }
        if !self.plan.check_invariants() {
            return false;
        }
        let ceiling = self.config.faction_count.max(1);
        // The soldier faction column is a second population under the same
        // ceiling. Checking one and not the other let the test suite spawn
        // soldiers of factions the world did not have, and pass.
        if self
            .soldiers
            .faction_column()
            .iter()
            .any(|faction| faction.0 >= ceiling)
        {
            return false;
        }
        // The arena holds a copy of the grid. A check must fail when the two
        // copies disagree.
        if self.soldiers.grid() != self.grid {
            return false;
        }
        // Level 1 covers the world once, and at a barrier it counts the
        // population the arena holds.
        //
        // The full equality between a level and the level below is a sweep of
        // every tile, and this check runs after every rule the control plane
        // applies, so it reads the totals instead. The equality itself is a
        // test.[^4]
        //
        // The tile total holds at every moment, because the ground does not
        // change. The unit total holds at a barrier only: a spawn made between
        // two frames leaves the level stale, which is the documented state and
        // not a defect.
        //
        // **Level 1 says which moment it describes, and the derived unit
        // structure does not say it for level 1.** A verb restores that
        // structure and rebuilds no level, so a check that read the structure
        // here would ask the level about an arena it never counted.[^5]
        //
        // [^4]: ADR-0023, an aggregate combines exactly, in any order, decision D5. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
        // [^5]: Findings register, FND-647. `docs/FINDINGS.md`
        let total = self.pyramid.total();
        if total.tiles() != i64::from(self.grid.tile_count()) {
            return false;
        }
        if self.level_1_arena == Some(self.soldiers.revision())
            && total.units() != i64::from(self.soldiers.len())
        {
            return false;
        }

        // No soldier stands on ground that admits no unit of its own type.
        // The spawn, the placement and the movement each refuse such a tile,
        // and this check is what fails when a later path forgets to.[^1]
        //
        // **The gate is the capacity table, and the crossing column of the
        // unit is an argument to it.** The movement pass admits a step by
        // that table, so a check that read the ground alone would state a
        // second, stricter rule and fail on a mariner that crossed open
        // water exactly as its row permits.[^11]
        //
        // [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D4. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
        // [^11]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
        if self.soldiers.iter().any(|soldier| {
            self.soldiers
                .address(soldier)
                .is_some_and(|address| !self.admits_this_unit(soldier, address))
        }) {
            return false;
        }
        // The terrain holds a second copy of the seed and of the extent. One
        // value declared twice needs a check that fails when the copies
        // disagree, because a silently wrong copy reads back correctly and
        // changes the whole world.[^1]
        //
        // [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
        if self.terrain.seed() != self.config.seed || self.terrain.grid() != self.grid {
            return false;
        }
        if !self.soldiers.check_invariants() {
            return false;
        }
        // The settlement arena holds a copy of the grid, and its faction
        // column stands under the same ceiling as every other faction
        // column. A check must fail when a copy disagrees.[^1]
        //
        // [^1]: Findings register, FND-040. `docs/FINDINGS.md`
        if self.settlements.grid() != self.grid {
            return false;
        }
        if self
            .settlements
            .faction_column()
            .iter()
            .any(|faction| faction.0 >= ceiling)
        {
            return false;
        }
        if !self.settlements.check_invariants() {
            return false;
        }
        // The character faction column stands under the same ceiling as
        // every other faction column.
        if self
            .characters
            .faction_column()
            .iter()
            .any(|faction| faction.0 >= ceiling)
        {
            return false;
        }
        // A unit that names a character must name one the arena still holds.
        // A link to a removed character is the stale identity that the
        // generation exists to catch.[^1]
        //
        // [^1]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
        for (slot, bits) in self.soldiers.character_column().iter().enumerate() {
            let Some(character) = Entity::from_bits(*bits) else {
                continue;
            };
            if self.soldiers.live_column()[slot] != 1 {
                return false;
            }
            if !self.characters.contains(character) {
                return false;
            }
        }
        if !self.characters.check_invariants() {
            return false;
        }
        // The plane is either empty or one row for each ordered pair, and
        // no party ever delivered more than it owed.
        if !self.trade.check_invariants() {
            return false;
        }
        if self.trade.factions() != self.config.faction_count {
            return false;
        }
        // The board is either empty or one block for each faction.
        if !self.market.check_invariants() {
            return false;
        }
        if !self.check_store_conservation() {
            return false;
        }
        if !self
            .shortfall_log
            .iter()
            .all(|event| event.padding == [0; 2] && event.amount.0 > 0)
        {
            return false;
        }
        if !self.check_cohorts() {
            return false;
        }
        if !self.check_contest() {
            return false;
        }
        // Every drop that entered the air is in the air, on the ground, or
        // counted as evaporated. A pass that scaled the water rather than
        // moving it would break this, and nothing else reports it.[^9]
        //
        // [^9]: ADR-0141, a weather pass moves water and never scales it, decision D2. `docs/adrs/draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md`
        if !self.weather.check_account() {
            return false;
        }
        if !self.check_positions() {
            return false;
        }
        // The bridge is a second declaration of where a soldier stands, and
        // the tile column is the first. The check fails when the two
        // disagree.[^1] A stale bridge cannot be compared against columns it
        // was not derived from, so the structure check stands alone there.
        //
        // [^1]: Findings register, FND-040. `docs/FINDINGS.md`
        if !self.bridge.check_structure() {
            return false;
        }
        if self.bridge.layout().grid() != self.grid {
            return false;
        }
        match self.bridge.check_invariants(&self.soldiers) {
            Ok(held) => {
                if !held {
                    return false;
                }
            }
            Err(BridgeError::Stale { .. }) => {}
            Err(_) => return false,
        }
        // The holding covers the same world, no tile names a faction the
        // world does not have, and no faction holds ground that admits no
        // unit. The check derives the held list, the census and the block
        // masks again and compares them against the stored ones.[^1]
        //
        // [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
        if self.holding.grid() != self.grid {
            return false;
        }
        if !self.holding.check_invariants(self.terrain, ceiling) {
            return false;
        }
        // Level 1 and the running census are two statements of how much
        // ground is held. The check fails when they disagree.
        let census: i64 = (0..ceiling)
            .map(|faction| self.holding.holding_of(FactionId(faction)))
            .sum();
        if census != self.holding.held_tiles() {
            return false;
        }
        if self.bridge.describes(&self.soldiers).is_ok() && total.held_tiles() != census {
            return false;
        }
        if !self.check_conservation() {
            return false;
        }
        if !self
            .gather_log
            .iter()
            .all(|event| event.padding == [0; 7] && (event.tile.0 as usize) < self.tile_count())
        {
            return false;
        }
        // The luxury entries rise, they name each tile once, and no entry is
        // empty. A field that broke any of those answers a lookup with the
        // wrong tile, and nothing else notices.
        if !self.luxuries.check_invariants(self.grid.tile_count()) {
            return false;
        }
        // Level 1 of the variety states the same fact a second time, and it
        // states it over the cells. A check must fail when the two copies
        // disagree, because a derived level that drifts reads back correctly
        // and answers the wrong question.[^5]
        //
        // The check compares the union of every cell against the union of
        // every tile. It is a fold over the cells and a fold over the
        // entries, so it visits no tile that carries nothing.
        //
        // [^5]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
        if self.variety.total() != self.luxuries.set() {
            return false;
        }
        if self.variety.deposit_total() != self.luxuries.deposits() {
            return false;
        }
        if self.variety.layout() != self.bridge.layout() {
            return false;
        }
        self.log
            .iter()
            .all(|event| event.padding == [0; 5] && (event.tile.0 as usize) < self.tile_count())
    }

    /// Reports whether the store column agrees with the account of it.
    ///
    /// What a site held, plus what production put in, minus what upkeep
    /// took, is what the site holds. This check states that equality over
    /// every live site at once.
    ///
    /// The account moves at four places: a write from the control plane,
    /// the loss of a settlement, the rate pass, and nowhere else. A fifth
    /// place that changes a store and forgets the account fails here, and it
    /// fails whatever the thread count was, because a rule that leaks the
    /// same amount on every run repeats perfectly and no determinism test
    /// can see it.[^1]
    ///
    /// The check is exact. Every term is a whole number in a 64-bit
    /// accumulator, so the sum is the same in any order and nothing
    /// rounds.[^2]
    ///
    /// The rate table also states a slot count that the arena already
    /// holds. A check must fail when the two copies disagree.[^3]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-048. `docs/FINDINGS.md`
    /// [^2]: ADR-0023, an aggregate combines exactly, in any order, decision D3. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    /// [^3]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    fn check_store_conservation(&self) -> bool {
        if !self.rates.check_invariants() {
            return false;
        }
        if self.rates.slot_count() < self.settlements.slot_count() {
            return false;
        }
        let mut held = [Accum(0); COMMODITY_COUNT];
        for settlement in self.settlements.iter() {
            let Some(store) = self.settlements.store(settlement) else {
                return false;
            };
            for (index, total) in held.iter_mut().enumerate() {
                let Some(quantity) = store.quantity(CommodityId(index as u16)) else {
                    return false;
                };
                *total = sim_math::accumulate(*total, quantity);
            }
        }
        held == self.store_account
    }

    /// Reports whether the positions of every site hold their rules.
    ///
    /// Three statements must hold together, and each of them is a place
    /// where one fact could be stored twice.[^1]
    ///
    /// The table and the settlement arena state the same slot count. Every
    /// position names a unit that still exists, so a holder that died leaves
    /// no stale identity behind.[^2] No site holds more positions than the
    /// ground under it admits, and both bounds come from the terrain
    /// capacity table.[^3]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    /// [^2]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    /// [^3]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
    fn check_positions(&self) -> bool {
        if self.positions.slot_count() != self.settlements.slot_count() {
            return false;
        }
        if !self.positions.check_invariants() {
            return false;
        }
        if !self.positions.check_holders(&self.soldiers) {
            return false;
        }
        matches!(
            self.positions.check_capacity(
                self.settlements.tile_column(),
                self.settlements.live_column(),
                self.terrain,
            ),
            Ok(true)
        )
    }

    /// Reports whether the cohort table and the home column agree.
    ///
    /// The table is a summary of the home column of the units, and the pass
    /// derives it again on every application. Between two applications a
    /// spawn or a home write leaves it behind, in the same way that a
    /// structural change leaves the derived unit structure stale.[^1] This
    /// check therefore states what is true at every moment: the table holds
    /// its own key, every home names a live site, and every reported event
    /// says what it means.
    ///
    /// The equality between the headcounts and the population is true right
    /// after an application, and a test asserts it there.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    /// [^2]: Testing rules, section 5. `.claude/rules/testing.md`
    #[must_use]
    fn check_cohorts(&self) -> bool {
        if !self.cohorts.check_invariants() {
            return false;
        }
        // Every home names a live site. A home left on a lost site would
        // feed the settlement founded next in that slot.
        for (slot, home) in self.soldiers.home_column().iter().enumerate() {
            if self.soldiers.live_column()[slot] != 1 || *home == crate::soldier::NO_HOME {
                continue;
            }
            if self.settlements.live_column().get(*home as usize) != Some(&1) {
                return false;
            }
        }
        if !self
            .rationed_log
            .iter()
            .all(|event| event.padding == [0; 6] && event.granted.0 < event.demanded.0)
        {
            return false;
        }
        // A starved unit is dead by the time anyone reads the log, so the
        // check states what the event itself must hold: declared padding,
        // an identity that packs, and a deficit that reached the bound.
        self.starved_log.iter().all(|event| {
            event.padding == [0; 4]
                && event.unit != 0
                && self.need_rule.condition(event.deficit) == NeedCondition::Starved
        })
    }

    /// Reports whether the unit type table and the fallen log hold their
    /// invariants.
    ///
    /// The table holds no negative value, because an attack is a quantity of
    /// harm and an armour is a threshold. The log holds declared padding, an
    /// identity that packs, a tile inside the world, and a type the table
    /// holds. A fallen unit is dead by the time anyone reads the log, so the
    /// check states what the event itself must hold.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
    fn check_contest(&self) -> bool {
        if !self.queues.check_invariants(self.settlements.slot_count()) {
            return false;
        }
        if !self.unit_types.check_invariants() {
            return false;
        }
        let tiles = self.grid.tile_count();
        if !self.fell_log.iter().all(|event| {
            event.padding == [0; 1]
                && event.unit != 0
                && event.tile.0 < tiles
                && event.faction.0 < FACTION_CEILING
                && event.unit_type.index() < crate::unit_type::UNIT_TYPE_COUNT
        }) {
            return false;
        }
        // A conversion event names two different factions of this world, and
        // a live tile. An event that named one faction twice would report a
        // change that did not happen.
        self.converted_log.iter().all(|event| {
            event.unit != 0
                && event.tile.0 < tiles
                && event.from.0 < FACTION_CEILING
                && event.to.0 < FACTION_CEILING
                && event.from != event.to
        })
    }

    /// Reports whether the cohorts describe the unit columns.
    ///
    /// The check derives the table again from the home column and compares.
    /// A summary that nothing compares against its source is a second
    /// declaration site with nothing that fails on disagreement.[^1]
    ///
    /// The answer is true right after an application of the consumption
    /// pass, and it is false after a spawn that no application has seen.
    /// The caller states which moment it means.
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub fn cohorts_describe_the_units(&self) -> bool {
        self.cohorts.describes(
            self.soldiers.home_column(),
            self.soldiers.faction_column(),
            self.soldiers.live_column(),
            self.settlements.slot_count(),
        )
    }

    /// Reports whether the world conserves every resource.
    ///
    /// What left the tiles equals what the live units carry, plus what left
    /// the world in the hands of a dead unit. The equality holds for each kind
    /// on its own, because a gather never turns one kind into another.[^1]
    ///
    /// The check is exact. Every term is a whole number in a 64-bit
    /// accumulator, so the sum is the same in any order and nothing rounds.[^2]
    ///
    /// A determinism test cannot see a broken invariant, because a rule that
    /// leaks the same amount on every run repeats perfectly.[^3] This check is
    /// what fails instead.
    ///
    /// # References
    ///
    /// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D5. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    /// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^3]: Findings register, FND-048. `docs/FINDINGS.md`
    #[must_use]
    fn check_conservation(&self) -> bool {
        if !self.depletion.check_invariants() {
            return false;
        }
        let mut left_the_tiles = [0i64; RESOURCE_KIND_COUNT];
        for entry in self.depletion.entries() {
            let (key, amount) = (&entry.key, &entry.taken);
            let Some(kind) = ResourceKind::from_u8((key & 0b11) as u8) else {
                return false;
            };
            let tile = TileIdx((key >> 2) as u32);
            // Nothing takes more from a tile than the tile ever held.
            let Some(original) = self.resources.original_at(tile, kind) else {
                return false;
            };
            if *amount > original.0 {
                return false;
            }
            left_the_tiles[kind.index()] += i64::from(*amount);
        }
        let mut arrived = [0i64; RESOURCE_KIND_COUNT];
        for soldier in self.soldiers.iter() {
            let Some(load) = self.soldiers.carry(soldier) else {
                return false;
            };
            for kind in ResourceKind::ALL {
                arrived[kind.index()] += i64::from(load.of(kind).0);
            }
        }
        for kind in ResourceKind::ALL {
            let index = kind.index();
            // Recovery gives a part of the take back to the tile, so the
            // stored take alone no longer balances what the units hold. The
            // returned total is the second term, and it is what makes the
            // equality hold across a recovery.
            let returned = self.depletion.returned(kind).0;
            // A delivered quantity has left the carries and reached a store,
            // so it is neither held nor departed. It is the term that links
            // this check to the store check.
            let delivered = self.delivered[index] as i64;
            if left_the_tiles[index] + returned
                != arrived[index] + self.departed[index] as i64 + delivered
            {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for the state that the public API cannot reach.
    //!
    //! The testing policy allows a unit test where a test cannot observe
    //! the case through the public interface. The public API cannot build
    //! a world that breaks its own invariants, so a test of the invariant
    //! check must build one here.[^1]
    //!
    //! # References
    //!
    //! [^1]: Testing policy, section 2. `docs/TESTING.md`

    use super::*;

    /// Builds a world with a broken part.
    fn broken(change: impl FnOnce(&mut World)) -> World {
        let mut world = World::new(WorldConfig {
            width: 4,
            height: 2,
            seed: 1,
            faction_count: 2,
            unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        })
        .expect("the extent must describe a world");
        change(&mut world);
        world
    }

    #[test]
    fn a_sound_world_holds_its_invariants() {
        assert!(broken(|_| {}).check_invariants());
    }

    #[test]
    fn a_stored_change_outside_the_extent_fails_the_check() {
        assert!(!broken(|world| {
            world.values.merge_ascending(&[(64, Fix32(1))]);
        })
        .check_invariants());
    }

    #[test]
    fn an_event_with_padding_fails_the_check() {
        assert!(!broken(|world| {
            let mut event = TileChanged::new(Tick(1), TileIdx(0), Fix32::ZERO, Holder::NOBODY, 1);
            event.padding[0] = 1;
            world.log.push(event);
        })
        .check_invariants());
    }

    #[test]
    fn an_event_that_names_no_tile_fails_the_check() {
        assert!(!broken(|world| {
            world.log.push(TileChanged::new(
                Tick(1),
                TileIdx(8),
                Fix32::ZERO,
                Holder::NOBODY,
                1,
            ));
        })
        .check_invariants());
        // The bound is exclusive. The highest valid index passes.
        assert!(broken(|world| {
            world.log.push(TileChanged::new(
                Tick(1),
                TileIdx(7),
                Fix32::ZERO,
                Holder::NOBODY,
                1,
            ));
        })
        .check_invariants());
    }
}
