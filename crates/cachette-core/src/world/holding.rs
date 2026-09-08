//! Which faction holds a tile, and how far the reach of a city goes.
//!
//! A holding is a claim on ground. The reach rules turn a site into a set of
//! tiles, the lease rules say who else may stand there, and the occupancy
//! readers report who does. One subject, so one module.

use super::errors::StepError;
use super::World;
use crate::bridge::BridgeError;
use crate::hex::Axial;
use crate::holding::{FactionMask, Holder, Holding, LeaseRules, ReachRules};
use crate::site::SiegeRules;
use crate::slots::Slots;
use crate::types::{Entity, FactionId, Tick, TileIdx};

impl World {
    /// Returns the holding of the world.
    ///
    /// The holding says who holds each tile, and how much ground each
    /// faction holds.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    #[must_use]
    pub const fn holding(&self) -> &Holding {
        &self.holding
    }

    /// Returns which factions stand on the ground of which other factions.
    ///
    /// Row `host` names every faction that has a live unit standing on a tile
    /// that `host` holds. A unit on ground its own faction holds sets no bit,
    /// so the diagonal is always empty.[^1]
    ///
    /// **The answer is 63 words whatever the population.** The relation is a
    /// mask row for each faction, which is the shape every relation between
    /// factions takes in this project.[^2]
    ///
    /// The relation is exact. The fold reads the holder of the exact tile
    /// each unit stands on, so a clear bit means that no unit is there.
    ///
    /// # Errors
    ///
    /// Returns an error when the population changed since the last step, so
    /// that a caller meets a refusal rather than a stale answer.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decision D3. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
    /// [^2]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D7. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    /// [^3]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decision D4. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
    pub fn presence_rows(&self) -> Result<&[FactionMask], BridgeError> {
        self.presence.rows(&self.soldiers)
    }

    /// Returns one entry for each tile that carries a unit: the tile and the
    /// faction that has the most units on it.
    ///
    /// **The tie goes to the lowest faction identifier.** The units of one
    /// tile reach this pass in the order the unit index packs them, and that
    /// order changes when a unit dies, moves or is promoted. A rule that took
    /// the first faction it found would take the packing, and a packing is
    /// not a stable key.[^1] [^2]
    ///
    /// The tally holds only the factions that stand on one tile, and the
    /// capacity of the ground bounds how many units that is. It is therefore
    /// local to one tile, and no stored field is indexed by the faction.[^1]
    ///
    /// Each thread reads a contiguous run of blocks and writes its own slot.
    /// The join reads the slots in slot order, and a block holds a
    /// contiguous run of tiles in ascending order, so the joined list is in
    /// ascending tile order at every thread count. Nothing reads which thread
    /// finished first.[^3]
    ///
    /// # Errors
    ///
    /// Returns an error when the thread count is zero.
    ///
    /// # References
    ///
    /// [^1]: ADR-0153, a tile's lease follows the units that stand on it, decision D3. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
    /// [^2]: ADR-0004, iteration order is explicit, decisions D1 and D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^3]: ADR-0009, parallel stages write disjoint outputs, decisions D1, D2 and D3. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
    pub(super) fn tile_occupancy(
        &self,
        threads: usize,
    ) -> Result<Vec<(TileIdx, FactionId)>, StepError> {
        let threads = threads.max(1);
        let blocks = self.bridge.layout().block_count();
        if blocks == 0 {
            return Ok(Vec::new());
        }
        let block_chunk = (blocks as usize).div_ceil(threads).max(1);
        let slot_count = (blocks as usize).div_ceil(block_chunk);
        let mut slots: Slots<Vec<(TileIdx, FactionId)>> =
            Slots::filled(slot_count, Vec::new()).map_err(|_| StepError::ZeroThreads)?;
        let bridge = &self.bridge;
        let arena = &self.soldiers;
        std::thread::scope(|scope| {
            let mut first = 0u32;
            for slot in slots.entries_mut() {
                let start = first;
                let stop = (start as usize + block_chunk).min(blocks as usize) as u32;
                first = stop;
                scope.spawn(move || {
                    let factions = arena.faction_column();
                    let tiles = arena.tile_column();
                    let mut tally: Vec<(FactionId, u32)> = Vec::new();
                    for block in start..stop {
                        let (keys, units) = bridge.block_window(block);
                        let mut position = 0usize;
                        while position < keys.len() {
                            let mut end = position + 1;
                            while end < keys.len() && keys[end] == keys[position] {
                                end += 1;
                            }
                            let tile_units = &units[position..end];
                            position = end;
                            tally.clear();
                            for unit in tile_units {
                                let faction = factions[unit.index() as usize];
                                match tally.iter_mut().find(|(named, _)| *named == faction) {
                                    Some((_, count)) => *count += 1,
                                    None => tally.push((faction, 1)),
                                }
                            }
                            // The most units takes the tick, and a tie goes
                            // to the lowest faction identifier. Neither test
                            // reads the packing of the units.
                            let mut best: Option<(FactionId, u32)> = None;
                            for (faction, count) in &tally {
                                let better = match best {
                                    None => true,
                                    Some((named, most)) => {
                                        *count > most || (*count == most && faction.0 < named.0)
                                    }
                                };
                                if better {
                                    best = Some((*faction, *count));
                                }
                            }
                            if let Some((faction, _)) = best {
                                let tile = tiles[tile_units[0].index() as usize];
                                slot.push((tile, faction));
                            }
                        }
                    }
                });
            }
        });
        let mut occupancy = slots.combine(Vec::new(), |mut joined, slot| {
            joined.extend_from_slice(slot);
            joined
        });
        // A block holds a contiguous run of tiles, and the slots join in
        // block order, so the list is already in tile order. The sort is what
        // makes that a property of the data rather than of the layout.
        occupancy.sort_unstable_by_key(|(tile, _)| tile.0);
        Ok(occupancy)
    }

    /// Reports whether a unit of `guest` stands on ground that `host` holds.
    ///
    /// Returns `false` when `guest` and `host` are the same faction, because
    /// the relation holds no diagonal.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the population changed since the last step.
    ///
    /// # References
    ///
    /// [^1]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decision D3. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
    pub fn stands_in_territory(
        &self,
        guest: FactionId,
        host: FactionId,
    ) -> Result<bool, BridgeError> {
        self.presence.stands_in(&self.soldiers, guest, host)
    }

    /// Returns who holds one tile.
    ///
    /// The answer names a faction, or nobody. It never names two factions,
    /// because a tile carries one holder.[^1]
    ///
    /// Returns `None` when the address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    #[must_use]
    pub fn tile_holder(&self, address: Axial) -> Option<Holder> {
        self.holding.holder(address)
    }

    /// Returns the number of tiles one faction holds.
    ///
    /// The call reads a running total, so it costs the same whatever the
    /// size of the world.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    #[must_use]
    pub fn holding_of(&self, faction: FactionId) -> i64 {
        self.holding.holding_of(faction)
    }

    /// Returns how far one city reaches, in hex steps.
    ///
    /// The reach is the base plus one step for each block of finished
    /// upgrades on the ground the city held at the end of the previous step,
    /// capped at the bound.[^1] Returns `None` when the identity names no
    /// live settlement.
    ///
    /// # References
    ///
    /// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D2. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    #[must_use]
    pub fn city_reach(&self, site: Entity) -> Option<u32> {
        let slot = self.settlements.slot_of(site)?;
        self.holding
            .cities(&self.settlements, &self.upgrades)
            .into_iter()
            .find(|city| city.slot == slot)
            .map(|city| city.reach)
    }

    /// Returns how many finished upgrades one city counts toward its reach.
    ///
    /// The count is the finished upgrades that stand on ground of the faction
    /// of the city, and to which this city is the nearest city of that
    /// faction. Two cities of one faction therefore split the upgrades
    /// between them, and no upgrade counts twice.
    ///
    /// Returns `None` when the identity names no live settlement.
    #[must_use]
    pub fn city_finished_upgrades(&self, site: Entity) -> Option<u32> {
        let slot = self.settlements.slot_of(site)?;
        self.holding
            .cities(&self.settlements, &self.upgrades)
            .into_iter()
            .find(|city| city.slot == slot)
            .map(|city| city.finished)
    }

    /// Returns how many steps of reach one city may still earn.
    ///
    /// The value is the bound of the reach rules less the reach the city
    /// holds now. A city at the bound reads zero, and no upgrade it finishes
    /// widens its ground.[^1]
    ///
    /// Returns `None` when the identity names no live settlement.
    ///
    /// # References
    ///
    /// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D2. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    #[must_use]
    pub fn city_reach_headroom(&self, site: Entity) -> Option<u32> {
        let reach = self.city_reach(site)?;
        Some(self.holding.rules().cap().saturating_sub(reach))
    }

    /// Reports whether one faction holds one tile.
    ///
    /// Returns `None` when the address lies outside the world. The call reads
    /// the holder column, which the cities rewrite on every step.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D3. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    #[must_use]
    pub fn holds(&self, faction: FactionId, address: Axial) -> Option<bool> {
        Some(self.holding.holder(address)?.faction() == Some(faction))
    }

    /// Returns how far a city reaches, and what extends the reach.
    ///
    /// The three values are balance rows, and every value in that register is
    /// unset until the balance pass measures it.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the holding. `docs/reference/balance.md`
    #[must_use]
    pub const fn reach_rules(&self) -> ReachRules {
        self.holding.rules()
    }

    /// Sets how far a city reaches, and what extends the reach.
    ///
    /// The three values are balance rows.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the holding. `docs/reference/balance.md`
    pub const fn set_reach_rules(&mut self, rules: ReachRules) {
        self.holding.set_rules(rules);
    }

    /// Returns what a site resists, and what a raze costs over a capture.
    #[must_use]
    pub const fn siege_rules(&self) -> SiegeRules {
        self.siege_rules
    }

    /// Sets what a site resists, and what a raze costs over a capture.
    ///
    /// The two values are balance rows.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the siege. `docs/reference/balance.md`
    pub const fn set_siege_rules(&mut self, rules: SiegeRules) {
        self.siege_rules = rules;
    }

    /// Returns how a lease rises, falls and claims.
    ///
    /// A lease is one faction and one count for each tile. The count follows
    /// the units that stand on the tile, and a lease at or above the claim
    /// threshold holds the tile whatever city reaches it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0153, a tile's lease follows the units that stand on it, decisions D1 and D5. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
    #[must_use]
    pub const fn lease_rules(&self) -> LeaseRules {
        self.holding.lease_rules()
    }

    /// Sets how a lease rises, falls and claims.
    ///
    /// The seven values are balance rows, and one blocker governs each of
    /// them.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the lease. `docs/reference/balance.md`
    /// [^2]: Blockers register, BLK-050. `docs/BLOCKERS.md`
    pub const fn set_lease_rules(&mut self, rules: LeaseRules) {
        self.holding.set_lease_rules(rules);
    }

    /// Returns the lease of one tile: the faction it names, and the count.
    ///
    /// Returns `None` when the address lies outside the world.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0153, a tile's lease follows the units that stand on it, decision D1. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
    #[must_use]
    pub fn tile_lease(&self, address: Axial) -> Option<(Holder, i32)> {
        self.holding.lease(address)
    }

    /// Returns how many ticks a campaign runs before it expires.
    ///
    /// Zero means that no campaign expires. The value is a balance row.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the campaign deadline. `docs/reference/balance.md`
    #[must_use]
    pub const fn campaign_deadline(&self) -> Tick {
        self.campaigns.deadline()
    }

    /// Sets how many ticks a campaign runs before it expires.
    ///
    /// Zero means that no campaign expires.
    pub const fn set_campaign_deadline(&mut self, deadline: Tick) {
        self.campaigns.set_deadline(deadline);
    }

    /// Returns the factions that hold ground in the block covering a tile.
    ///
    /// Returns `None` when the address lies outside the world.
    #[must_use]
    pub fn holders_near(&self, address: Axial) -> Option<FactionMask> {
        let tile = self.grid.index_of(address)?;
        let key = self.holding.layout().key_of(tile)?;
        self.holding
            .block_mask(self.holding.layout().block_of_key(key))
    }

    /// Reports whether any live unit of one faction stands on ground that
    /// another faction holds.
    ///
    /// **This is the gate that every speech act passes.** A player speaks to
    /// another player only while one of its own units stands in that player's
    /// territory, and a trade is a thing two players say to each other.[^1]
    ///
    /// The read walks the unit column once and reads the holder of the tile
    /// each unit stands on. It reads primary state only, so it holds no copy
    /// of an answer that another structure also holds.[^2] It costs one column
    /// read for each live unit, once for each speech act, and a speech act
    /// happens between frames.
    ///
    /// # References
    ///
    /// [^1]: ADR-0126, a trade negotiation is engine state and the words are not, decision D3. `docs/adrs/draft/adr-0126-a-trade-negotiation-is-engine-state.md`
    /// [^2]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub fn stands_in_territory_of(&self, speaker: FactionId, listener: FactionId) -> bool {
        let holders = self.holding.holders();
        let target = Holder::of(listener);
        self.soldiers.iter().any(|unit| {
            self.soldiers.faction(unit) == Some(speaker)
                && self
                    .soldiers
                    .tile(unit)
                    .and_then(|tile| holders.get(tile.0 as usize).copied())
                    == Some(target)
        })
    }
}
