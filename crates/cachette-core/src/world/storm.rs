//! What a storm does to the world under it: the food, the people, and the
//! ground they stand on.
//!
//! A storm puts a pressure deficit on the cells it reaches, and the deficit
//! is the whole of what this pass reads. The rates live in the storm module
//! and the pass applies them, so a deficit becomes harm at one site.[^1]
//!
//! **The upgrade harm is not here.** The wear pass owns every removal of an
//! upgrade, and it reads the same rate. A second removal site would leave two
//! statements of when a site falls.[^1]
//!
//! # References
//!
//! [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`

use super::World;
use crate::resource::{ledger_key, ResourceKind};
use crate::storm::{self, UnitLostToAStorm};
use crate::types::{Entity, TileIdx};

impl World {
    /// Returns the units that a storm ended since the last step began.
    #[must_use]
    pub fn units_lost_to_storms(&self) -> &[UnitLostToAStorm] {
        &self.storm_lost_log
    }

    /// Returns the pressure deficit that the storms put over one tile.
    ///
    /// The answer is the sum over every storm that reaches the cell covering
    /// the tile, and it is zero where no storm reaches. Every harm of this
    /// module is a rate against this number, so a caller that wants to know
    /// how hard a storm presses on a place reads it here.
    ///
    /// The answer is the coarseness of the weather lattice, so two tiles of
    /// one weather cell answer alike.[^1]
    ///
    /// Returns `None` when the address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0140, weather is a field over the level 1 cell lattice, decision D1. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
    #[must_use]
    pub fn storm_depth_at(&self, address: crate::hex::Axial) -> Option<i32> {
        let tile = self.grid.index_of(address)?;
        Some(self.weather.depression_at(self.weather_cell_of(tile)?))
    }

    /// Takes the food and the people that the storms of this tick reach.
    ///
    /// # What the pass does, in order
    ///
    /// 1. It walks the cells of the weather lattice in ascending cell order,
    ///    and it skips every cell that carries no pressure deficit.
    /// 2. For each cell it reaches, it walks the tiles of that cell in
    ///    ascending offset order. A tile that carries food loses a share of
    ///    what it still holds, and each unit standing on it in the open draws
    ///    for its life.
    /// 3. The food losses are sorted by ledger key and merged into the one
    ///    ledger that records every take.
    /// 4. The units the storm took leave the world.
    ///
    /// # What is not here
    ///
    /// **The pass creates no water.** The ground under a storm gets wet
    /// because the deficit cuts the capacity of the air above it, so the air
    /// pours what it holds onto the ground. That is an exact move of water
    /// the weather field already holds, and a second site that added drops
    /// would put water into the world that no cell gave up.[^1]
    ///
    /// **The pass removes no upgrade.** The wear pass reads the same deficit
    /// through the same rate, and it owns every removal.[^2]
    ///
    /// # What it costs
    ///
    /// A world that carries no storm walks nothing at all. A world that
    /// carries storms walks the cells of the lattice once, and the tiles of
    /// the cells the storms reach. The storm count and the reach of a storm
    /// both have ceilings, so the tile work is bounded by the footprint of
    /// the storms and never by the extent of the world.[^3] **No measurement
    /// on the target platform holds this figure.**[^4]
    ///
    /// # Determinism
    ///
    /// The pass runs on one thread, so no thread completion order can reach
    /// it. It walks the cells in ascending cell order and the tiles of a cell
    /// in ascending offset order.[^5] The food run is sorted by ledger key
    /// before it is merged, and the casualties are sorted by the tile and
    /// then by the whole unit identity, so neither follows the order the walk
    /// found them in. Every draw is keyed on the storm system, the frame, a
    /// whole unit identity and a draw index.[^6]
    ///
    /// The pass reads the weather field that the previous step left, because
    /// the weather solve runs later in this step. That is a fixed order and
    /// not a stale read, and it is the order the fire and the upgrade wear
    /// read it in.[^7]
    ///
    /// # References
    ///
    /// [^1]: ADR-0141, a weather pass moves water and never scales it, decision D2. `docs/adrs/draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md`
    /// [^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    /// [^3]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decision D1. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
    /// [^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
    /// [^5]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^6]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
    /// [^7]: ADR-0140, weather is a field over the level 1 cell lattice, decision D3. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
    pub(super) fn take_what_the_storms_reach(&mut self) {
        // **A world with no storm does no work here, at any tile count.** The
        // deficit plane is empty when no storm stands, so this one test is
        // what keeps the pass free for every world that carries no weather.
        if self.weather.cyclones().is_empty() {
            return;
        }
        let tick = self.tick;
        let seed = self.config.seed;
        let frame = tick.0;
        let layout = self.weather_layout;
        let lattice = self.weather_lattice;
        let offsets = u64::from(layout.block_edge()) * u64::from(layout.block_edge());

        let mut flattened: Vec<(u64, u32)> = Vec::new();
        let mut casualties: Vec<(TileIdx, Entity)> = Vec::new();
        for inner in 0..layout.block_count() {
            let Some(whole) = lattice.whole_of_inner(inner) else {
                continue;
            };
            let deficit = self.weather.depression_at(whole);
            if deficit <= 0 {
                continue;
            }
            for offset in 0..offsets {
                let key = (u64::from(inner) << (2 * layout.block_bits())) | offset;
                let Some(tile) = layout.tile_of_key(key) else {
                    continue;
                };
                let carried = self.food_a_tile_carries(tile);
                let lost = storm::food_lost_at(deficit, carried);
                if lost > 0 {
                    flattened.push((
                        ledger_key(tile, ResourceKind::Food),
                        u32::try_from(lost).unwrap_or(u32::MAX),
                    ));
                }
                // **Shelter is what a faction built.** A unit that stands on
                // ground carrying a finished upgrade is not in the open, so a
                // road, a terrace, a lodging or a wall answers the storm. The
                // upgrade map states what stands there, and no second site
                // says what shelters.
                if self.tile_shelters_a_unit(tile) {
                    continue;
                }
                for unit in self.bridge.on_tile_unguarded(tile) {
                    // **Every identity resolves against the arena.** The reap
                    // above this stage freed some slots, and the bridge
                    // rebuilds at the barrier below, so the list may name a
                    // dead unit. A dead identity answers `None` and
                    // contributes nothing.
                    if self.soldiers.tile(*unit) != Some(tile) {
                        continue;
                    }
                    if storm::takes_unit(seed, frame, unit.to_bits(), deficit) {
                        casualties.push((tile, *unit));
                    }
                }
            }
        }

        if !flattened.is_empty() {
            // The walk runs block by block, and a block covers a rectangle of
            // rows, so the tiles of two blocks interleave in the tile order.
            // The merge asks for one run in ascending key order, so the sort
            // is here and never the order the walk found the tiles in.[^8]
            //
            // [^8]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
            flattened.sort_unstable();
            // **Food a storm flattens leaves the world, and the world says
            // so.** The ledger records what left a tile, and the conservation
            // check balances that against what units hold, what a delivery
            // moved and what left the world. Flattened food reaches no unit
            // and no store, so it goes in the last of those. Without this the
            // account of the world stops balancing on the first storm.[^9]
            //
            // [^9]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D5. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
            let gone: u64 = flattened.iter().map(|(_, taken)| u64::from(*taken)).sum();
            self.departed[ResourceKind::Food.index()] += gone;
            // The ledger comes out of the world for the merge, for the reason
            // the gather pass takes it out: the merge ages each entry to this
            // tick first, and ageing reads the weather and the upgrade map.
            let mut depletion = core::mem::take(&mut self.depletion);
            depletion.merge_ascending(&flattened, tick, &|tile| self.tile_ground(tile));
            self.depletion = depletion;
        }

        if casualties.is_empty() {
            return;
        }
        // The order is the tile and then the whole identity. Both are stable
        // properties of the world and neither is a slot order.
        casualties.sort_unstable_by_key(|(tile, unit)| (tile.0, unit.to_bits()));
        for (tile, unit) in casualties {
            let Some(faction) = self.soldiers.faction(unit) else {
                continue;
            };
            let Some(unit_type) = self.soldiers.unit_type(unit) else {
                continue;
            };
            if self.despawn_soldier(unit) {
                self.storm_lost_log.push(UnitLostToAStorm::new(
                    tick,
                    unit.to_bits(),
                    tile,
                    faction,
                    unit_type,
                ));
            }
        }
        self.cohorts.rebuild(
            self.soldiers.home_column(),
            self.soldiers.faction_column(),
            self.soldiers.live_column(),
            self.settlements.slot_count(),
        );
    }

    /// Returns the food that one tile still carries.
    ///
    /// The answer is what the tile started with, less what has been taken.
    /// The engine stores the second term only, so a tile nobody touched costs
    /// nothing.[^1]
    ///
    /// Returns zero for a tile the world does not hold.
    ///
    /// # References
    ///
    /// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    fn food_a_tile_carries(&self, tile: TileIdx) -> i64 {
        let Some(original) = self.resources.original_at(tile, ResourceKind::Food) else {
            return 0;
        };
        let taken = self.depletion.taken(tile, ResourceKind::Food);
        i64::from(original.0.saturating_sub(taken.0))
    }

    /// Reports whether what stands on a tile shelters a unit from a storm.
    ///
    /// **This is the one declaration of what shelters.** A finished upgrade
    /// shelters, and bare ground does not. A caller that asked the upgrade
    /// map itself would hold a second statement of the rule.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    fn tile_shelters_a_unit(&self, tile: TileIdx) -> bool {
        self.upgrades
            .at(tile)
            .is_some_and(|site| site.is_complete())
    }
}
