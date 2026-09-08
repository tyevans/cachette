//! Fire on the ground: how it starts, how it spreads, and how it ends.
//!
//! A fire burns the ground and the units that stand on it. The ignition, the
//! douse order, the burn pass and the three logs sit together, because a fire
//! is one story from one tile to the next.

use super::errors::SendError;
use super::World;
use crate::fire::{FireEnded, FireField, FireStarted, GroundReading, UnitBurned};
use crate::hex::{Axial, NEIGHBOUR_COUNT};
use crate::soldier::SoldierArena;
use crate::types::{Entity, TileIdx};
use crate::weather::Wind;

/// Reports whether the fire on the tile a unit stands on is work it must
/// stay for.
///
/// **A unit that was ordered to fight the fire and stands in one stays.** The
/// fire order is per unit, and movement takes its direction from a field over
/// cells, so a hold cannot be written into that field without pinning every
/// unit of the cell. The pass asks this question of each unit instead, in the
/// way the build hold does.[^1] [^2]
///
/// **Nothing stores the answer.** The pass derives it from the order and from
/// the fire on the tick it reads it. It stops being true on the tick the fire
/// goes out and on the tick the unit dies, so there is no hold to clear and no
/// unit can carry a stale one.[^1]
///
/// # References
///
/// [^1]: ADR-0168, a build order holds a unit on its tile, and the hold is derived, decisions D1, D2 and D3. `docs/adrs/draft/adr-0168-a-build-order-holds-a-unit-on-its-tile.md`
/// [^2]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
pub(super) fn douse_holds_unit(unit: Entity, soldiers: &SoldierArena, fire: &FireField) -> bool {
    if soldiers.douse_order(unit) != Some(true) {
        return false;
    }
    match soldiers.tile(unit) {
        Some(tile) => fire.is_burning(tile),
        None => false,
    }
}

impl World {
    /// Returns what the fire needs to know about one tile.
    ///
    /// The reader gathers the ground, the building that stands on it and the
    /// weather over it. It applies none of them. The fire module holds the
    /// whole of the arithmetic, so a rule cannot be read one way here and
    /// another way there.[^1]
    ///
    /// The weather values are the ones that the solve of the previous frame
    /// left, in the same way the gather resolve and the upgrade wear read
    /// them.[^2]
    ///
    /// Returns `None` when the index names no tile of this world.
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    /// [^2]: ADR-0140, weather is a field over the level 1 cell lattice, decision D3. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
    #[must_use]
    fn fire_ground(&self, tile: TileIdx) -> Option<GroundReading> {
        let address = self.grid.address_of(tile)?;
        let kind = self.tile_kind(address)?;
        let carries_upgrade = self
            .upgrades
            .at(tile)
            .is_some_and(|site| site.is_complete());
        let (is_wet, cloud_share, wind) = match self.weather_cell_of(tile) {
            Some(cell) => (
                self.weather.cell_is_wet(cell),
                self.weather.cloud_share_at(cell),
                self.weather.wind_at(cell),
            ),
            None => (false, 0, Wind::STILL),
        };
        Some(GroundReading {
            kind,
            carries_upgrade,
            is_wet,
            cloud_share,
            cloud_whole: crate::weather::CLOUD_SHARE_WHOLE,
            wind_along: crate::fire::wind_components(wind),
            // The component along a direction is twice the speed when the
            // wind points exactly that way, and the speed has a ceiling, so
            // this is the largest value the component reaches.
            wind_whole: 2 * crate::weather::SPEED_CEILING,
        })
    }

    /// Sets one tile alight, and reports whether anything caught.
    ///
    /// **This is how a caller starts a fire.** The engine starts one the same
    /// way, through the lightning draw, so both paths reach the same rule and
    /// no second site states what may catch.[^1]
    ///
    /// Returns `false` when the index names no tile, when the ground carries
    /// no fuel, when the weather has wetted the ground, when the tile already
    /// burns, and when the tile has burned already.
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    pub fn ignite(&mut self, tile: TileIdx) -> bool {
        let Some(ground) = self.fire_ground(tile) else {
            return false;
        };
        if !self.fire.ignite(tile, ground) {
            return false;
        }
        let tick = self.tick;
        self.fire_started_log.push(FireStarted::new(
            tick,
            tile,
            crate::fire::START_CAUSE_ORDERED,
        ));
        true
    }

    /// Sets a set of tiles alight, and returns how many caught.
    ///
    /// **The set form is what a control plane calls.** The caller names the
    /// whole set in one command and loops over nothing.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0125, the control plane names the seed set of a destination field, decision D1. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    pub fn ignite_set(&mut self, tiles: &[TileIdx]) -> usize {
        let mut caught = 0usize;
        for tile in tiles {
            if self.ignite(*tile) {
                caught += 1;
            }
        }
        caught
    }

    /// Returns the fire field of the world.
    #[must_use]
    pub const fn fire(&self) -> &FireField {
        &self.fire
    }

    /// Returns every burning tile, in ascending tile order.
    ///
    /// **This is the seed set that a control plane sends people to.** The
    /// caller hands it straight back to the send verb, so it names no tile of
    /// its own and it walks nothing.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0125, the control plane names the seed set of a destination field, decision D1. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    #[must_use]
    pub fn burning_tiles(&self) -> Vec<TileIdx> {
        self.fire.burning_tiles()
    }

    /// Reports whether one tile burns now.
    #[must_use]
    pub fn tile_is_burning(&self, address: Axial) -> Option<bool> {
        let tile = self.grid.index_of(address)?;
        Some(self.fire.is_burning(tile))
    }

    /// Reports whether one tile has burned already.
    ///
    /// Ground that has burned never catches again, and that is what makes a
    /// fire end.
    #[must_use]
    pub fn tile_is_spent(&self, address: Axial) -> Option<bool> {
        let tile = self.grid.index_of(address)?;
        Some(self.fire.is_spent(tile))
    }

    /// Sets the chance that lightning starts a fire on one tick.
    ///
    /// The chance is stated out of the whole that the fire module names. A
    /// world that states zero never catches by itself, and that is what a
    /// world states until a caller says otherwise.
    pub fn set_lightning_chance(&mut self, chance: u64) {
        self.lightning_chance = chance.min(crate::fire::LIGHTNING_WHOLE);
    }

    /// Returns the chance that lightning starts a fire on one tick.
    #[must_use]
    pub const fn lightning_chance(&self) -> u64 {
        self.lightning_chance
    }

    /// Orders one soldier to fight the fire on the tile it stands on.
    ///
    /// Returns `false` when the identity is dead. The order holds until a
    /// caller stops it. A unit under this order that stands on a burning tile
    /// stays there and takes intensity off it, and it risks its life doing
    /// so.
    pub fn order_douse(&mut self, entity: Entity, douses: bool) -> bool {
        self.soldiers.set_douse_order(entity, douses)
    }

    /// Returns the fire order of one soldier, or `None` when it is dead.
    #[must_use]
    pub fn douse_order(&self, entity: Entity) -> Option<bool> {
        self.soldiers.douse_order(entity)
    }

    /// Orders a set of soldiers to fight the fire, and sends the whole set at
    /// the fire.
    ///
    /// **This is one command over a set, and it starts one field.** The
    /// engine takes every burning tile as the seed set of the destination
    /// plane the caller names, and every unit of the set then climbs that one
    /// field. No unit searches for a fire and the caller loops over
    /// nothing.[^1] [^2]
    ///
    /// Returns how many identities the order refused. The send is all or
    /// nothing, in the way the send verb is.
    ///
    /// # Errors
    ///
    /// Returns the refusal of the send verb when an identity is dead or the
    /// plane number is out of range. Returns it unchanged, so a caller reads
    /// one rule.
    ///
    /// # References
    ///
    /// [^1]: ADR-0125, the control plane names the seed set of a destination field, decision D1. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    /// [^2]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
    pub fn order_douse_set(
        &mut self,
        units: &[Entity],
        destination: u16,
    ) -> Result<usize, SendError> {
        let burning = self.fire.burning_tiles();
        let seeds: Vec<Axial> = burning
            .iter()
            .filter_map(|tile| self.grid.address_of(*tile))
            .collect();
        if !seeds.is_empty() {
            self.send_units_to(units, &seeds, destination)?;
        }
        let mut refused = 0usize;
        for entity in units {
            if !self.soldiers.set_douse_order(*entity, true) {
                refused += 1;
            }
        }
        Ok(refused)
    }

    /// Returns the tiles that caught fire since the last step began.
    #[must_use]
    pub fn fires_started(&self) -> &[FireStarted] {
        &self.fire_started_log
    }

    /// Returns the tiles that stopped burning since the last step began.
    #[must_use]
    pub fn fires_ended(&self) -> &[FireEnded] {
        &self.fire_ended_log
    }

    /// Returns the units the fire ended since the last step began.
    #[must_use]
    pub fn units_burned(&self) -> &[UnitBurned] {
        &self.burned_log
    }

    /// Advances every fire by one tick.
    ///
    /// # What the pass does, in order
    ///
    /// 1. Lightning may set one tile alight.
    /// 2. Each burning tile counts the people who fight it, and each unit
    ///    standing on it draws for its life.
    /// 3. Each burning tile decides, from the fire as it stands now, which of
    ///    its six neighbours catches.
    /// 4. Each burning tile loses fuel and gains or loses intensity, and a
    ///    tile that runs out of either stops burning and becomes spent.
    /// 5. The tiles that step 3 chose catch, unless step 4 spent them.
    /// 6. The units the fire took leave the world.
    ///
    /// # Why the fire ends
    ///
    /// **Nothing here asks whether anything changed.** Step 4 takes a fixed
    /// positive quantity of fuel off every burning tile on every tick, and a
    /// tile that stops can never catch again. The burning set is therefore
    /// empty after a bounded number of ticks, whatever the wind does.[^1]
    ///
    /// # Determinism
    ///
    /// The pass walks the burning set in ascending tile order and the six
    /// directions in index order. It runs on one thread, so no thread
    /// completion order can reach it. Every draw is keyed on the fire system,
    /// the frame, a tile or a whole unit identity, and a draw index.[^2] [^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0005, a solver runs a fixed iteration count. `docs/adrs/REGISTRY.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^3]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
    pub(super) fn burn(&mut self) {
        let tick = self.tick;
        let seed = self.config.seed;
        let frame = tick.0;

        // **The engine starts a fire here, and it costs the same whatever the
        // world holds.** The draw asks whether lightning strikes at all, and
        // a second draw picks the tile. Nothing walks the lattice, so a world
        // of sixteen million tiles pays two draws for this.
        if let Some(tile) =
            crate::fire::lightning_strike(seed, frame, self.lightning_chance, self.grid)
        {
            if let Some(ground) = self.fire_ground(tile) {
                if self.fire.ignite(tile, ground) {
                    self.fire_started_log.push(FireStarted::new(
                        tick,
                        tile,
                        crate::fire::START_CAUSE_LIGHTNING,
                    ));
                }
            }
        }

        if self.fire.burning_count() == 0 {
            return;
        }

        // The set is copied because the passes below read the world while
        // they walk it. The copy holds the burning tiles and nothing else, so
        // it follows the fire and never the world.
        let burning: Vec<crate::fire::FireTile> = self.fire.burning().to_vec();

        // **What the people on a tile do, and what it costs them.** A unit
        // under a fire order takes intensity off the tile it stands on. Every
        // unit on the tile, ordered or not, draws for its life against the
        // intensity of the fire.
        let mut suppression: Vec<i64> = Vec::with_capacity(burning.len());
        let mut casualties: Vec<(TileIdx, Entity)> = Vec::new();
        let mut cursor = self.bridge.tile_cursor();
        for entry in &burning {
            let mut work = 0i64;
            let standing: Vec<Entity> = self.bridge.units_on_tile(&mut cursor, entry.tile).to_vec();
            for unit in standing {
                // **Every identity resolves against the arena.** The reap
                // above this stage freed some slots, and the bridge rebuilds
                // at the barrier below, so the list may name a dead unit. A
                // dead identity answers `None` and contributes nothing.
                if self.soldiers.tile(unit) != Some(entry.tile) {
                    continue;
                }
                if self.soldiers.douse_order(unit) == Some(true) {
                    work = work.saturating_add(crate::fire::DOUSE_WORK_FOR_EACH_UNIT);
                }
                if crate::fire::burns_unit(seed, frame, unit.to_bits(), entry.intensity) {
                    casualties.push((entry.tile, unit));
                }
            }
            suppression.push(work);
        }

        // **The spread reads the fire as this tick found it.** It runs before
        // the advance, so a tile that people put out this tick still lit what
        // it lit, and a tile that catches this tick does not spread until the
        // next one.
        let mut lit: Vec<(u32, u32)> = Vec::new();
        for entry in &burning {
            let Some(here) = self.grid.address_of(entry.tile) else {
                continue;
            };
            for direction in 0..NEIGHBOUR_COUNT {
                let Some(there) = self.grid.neighbour(here, direction) else {
                    continue;
                };
                let Some(target) = self.grid.index_of(there) else {
                    continue;
                };
                if self.fire.is_burning(target) || self.fire.is_spent(target) {
                    continue;
                }
                let Some(ground) = self.fire_ground(target) else {
                    continue;
                };
                if crate::fire::spreads(seed, frame, entry.tile, direction, entry.intensity, ground)
                {
                    lit.push((target.0, entry.tile.0));
                }
            }
        }

        for (tile, cause) in self.fire.advance(&suppression) {
            self.fire_ended_log.push(FireEnded::new(tick, tile, cause));
        }

        // **One tile catches once, whatever number of neighbours lit it.**
        // The sort is on the target and then on the source, so the source
        // recorded is the lowest neighbour and never the neighbour a
        // schedule reached first.
        lit.sort_unstable();
        lit.dedup_by_key(|(target, _)| *target);
        for (target, _) in lit {
            let target = TileIdx(target);
            let Some(ground) = self.fire_ground(target) else {
                continue;
            };
            if self.fire.ignite(target, ground) {
                self.fire_started_log.push(FireStarted::new(
                    tick,
                    target,
                    crate::fire::START_CAUSE_SPREAD,
                ));
            }
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
                self.fire.count_burned_unit();
                self.burned_log.push(UnitBurned::new(
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
}
