//! The weather field, the cyclones it carries, and the ground it wets.
//!
//! The weather sits on a lattice that is coarser than the tile field, so a
//! reader must map a tile to a cell first. The mapping, the readers and the
//! pass that inflicts weather on the world sit together.

use super::gather::WET_GATHER_BONUS;
use super::World;
use crate::bridge::BlockLayout;
use crate::hex::Axial;
use crate::padded::PaddedLattice;
use crate::resource::TileGround;
use crate::types::{FactionId, TileIdx};
use crate::weather::{Cyclone, CycloneSetting, Ground, Storm, WeatherError, WeatherField, Wind};

/// Returns the carry class of one unit, from its load, its home and the mark.
///
/// **This is the one place that states the rule.** The choice pass, the
/// explanation and the public read all come through it, so no second site can
/// hold a different rule.[^1]
///
/// A unit with no home is never laden. The delivery moves a load into the
/// store of a home site, so an option that sent a homeless unit home would be
/// a capability that nothing can act on.[^2] [^3]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
/// [^2]: ADR-0109, the choice key holds a bounded class of the unit's own state, decision D3. `docs/adrs/draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
/// [^3]: Recurring defect shapes, shape 3. `.claude/rules/recurring-defects.md`
#[must_use]
/// Folds the terrain into the ground under each weather cell.
///
/// **The weather reads three numbers about the ground, and none of them
/// changes.** The tiles a cell covers, the tiles of it that admit a unit, and
/// the sum of their heights are each a pure function of the world seed and
/// the address, so the fold runs once when the world is built.[^1]
///
/// **It does not read the level 1 summary.** That summary describes a block
/// thirty-two tiles a side, and it is the wrong source at any other weather
/// pitch. The two agree exactly at the level 1 pitch, because both fold the
/// same three fields over the same tiles.[^2]
///
/// The walk is row by row and then column by column, and the combine is
/// integer addition, so the answer does not depend on the order.[^3]
///
/// # References
///
/// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
/// [^2]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
/// [^3]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
/// Returns the weather cell that covers a tile, margin offset included.
///
/// **This is the one site that adds the margin offset.** The block layout
/// gives the cell of the world, and the lattice turns that into the cell of
/// the whole plane that the solve steps. Two callers ask it: the reader that
/// every tile reader goes through, and the god that inflicts weather on a
/// place.[^1]
///
/// Returns `None` when the tile is outside the world.
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
pub(super) fn weather_cell_of(
    layout: BlockLayout,
    lattice: PaddedLattice,
    tile: TileIdx,
) -> Option<u32> {
    lattice.whole_of_inner(layout.block_of_key(layout.key_of(tile)?))
}

impl World {
    /// Returns the level 1 cell that covers one tile.
    #[must_use]
    /// Returns what one gather takes in addition, because the ground is wet.
    ///
    /// The bonus is a whole number of resource units and it is never a
    /// fraction of the ordinary rate. A rate multiplied by a fraction would
    /// be a second scale beside the one the resource ledger counts in.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0143, wet ground yields more to a gatherer, decision D2. `docs/adrs/draft/adr-0143-wet-ground-yields-more-to-a-gatherer.md`
    pub(super) fn wet_bonus(&self, tile: TileIdx) -> u32 {
        // The weather lattice has a pitch of its own, so the cell of a tile
        // comes from the weather reader and not from the level 1 reader.
        match self.weather_cell_of(tile) {
            Some(cell) if self.weather.cell_is_wet(cell) => WET_GATHER_BONUS,
            _ => 0,
        }
    }

    /// Returns the weather field of the world.
    ///
    /// The field is a plane over the level 1 cell lattice. It holds the water
    /// in the air above each cell and the water on the ground of each
    /// cell.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0140, weather is a field over the level 1 cell lattice, decision D1. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
    #[must_use]
    pub const fn weather(&self) -> &WeatherField {
        &self.weather
    }

    /// Returns the water in the air above the cell that covers one tile.
    ///
    /// The unit is drops, and a drop is a whole number. Returns `None` when
    /// the address lies outside the world.
    #[must_use]
    pub fn air_at(&self, address: Axial) -> Option<i64> {
        let tile = self.grid.index_of(address)?;
        Some(self.weather.air_at(self.weather_cell_of(tile)?).0)
    }

    /// Returns the share of the sky over one tile that a watcher sees as
    /// cloud.
    ///
    /// **Cloud is the air held against what the air of that cell can hold**,
    /// and not the air held against a mark that every cell shares. The
    /// published saturation curve puts the capacity of a tropical cell about
    /// thirty times above the capacity of a polar one, so a ramp against one
    /// mark paints a cold sky black. Every reader that paints cloud takes
    /// this one, so the rule has one declaration site.[^1]
    ///
    /// The share runs from none to a whole sky. Returns `None` when the
    /// address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub fn cloud_share_at(&self, address: Axial) -> Option<i64> {
        let tile = self.grid.index_of(address)?;
        Some(self.weather.cloud_share_at(self.weather_cell_of(tile)?))
    }

    /// Returns the water on the ground of the cell that covers one tile.
    ///
    /// The unit is drops, and a drop is a whole number. Returns `None` when
    /// the address lies outside the world.
    #[must_use]
    pub fn ground_water_at(&self, address: Axial) -> Option<i64> {
        let tile = self.grid.index_of(address)?;
        Some(self.weather.ground_at(self.weather_cell_of(tile)?).0)
    }

    /// Returns the wind over the cell that covers one tile.
    ///
    /// The wind is a bounded integer vector over the two axes of the cell
    /// lattice. It is carried state, so a watcher who reads it reads what the
    /// next step will read.[^1]
    ///
    /// Returns `None` when the address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D1. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
    #[must_use]
    pub fn wind_at(&self, address: Axial) -> Option<Wind> {
        let tile = self.grid.index_of(address)?;
        Some(self.weather.wind_at(self.weather_cell_of(tile)?))
    }

    /// Returns the temperature of the cell that covers one tile.
    ///
    /// The unit is a whole degree on the scale of the weather field, from
    /// zero to the heat ceiling. It is carried state, so a watcher who reads
    /// it reads what the next step will read.[^1]
    ///
    /// Returns `None` when the address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0166, the temperature of a cell is carried state that a season and the sky drive, decision D1. `docs/adrs/draft/adr-0166-the-temperature-of-a-cell-is-carried-state-that-a-season-and-the-sky-drive.md`
    #[must_use]
    pub fn temperature_at(&self, address: Axial) -> Option<i32> {
        let tile = self.grid.index_of(address)?;
        Some(self.weather.warmth_at(self.weather_cell_of(tile)?))
    }

    /// Reports whether the ground under one tile is wet.
    ///
    /// A unit that gathers on wet ground takes more in one tick than a unit
    /// on dry ground.[^1]
    ///
    /// Returns `None` when the address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0143, wet ground yields more to a gatherer, decision D1. `docs/adrs/draft/adr-0143-wet-ground-yields-more-to-a-gatherer.md`
    #[must_use]
    pub fn ground_is_wet(&self, address: Axial) -> Option<bool> {
        let tile = self.grid.index_of(address)?;
        Some(self.weather.cell_is_wet(self.weather_cell_of(tile)?))
    }

    /// Puts weather over a set of places, at the command of a god.
    ///
    /// The faction is the congregation the god directs. Each place names a
    /// tile, and the water lands on the level 1 cell that covers it, so two
    /// places in one cell are one place.
    ///
    /// **A god acts only where its own people hold the ground.** The cell of
    /// every place must hold at least one tile of the faction.[^1]
    ///
    /// **The call is all or nothing.** Every place is resolved and every gate
    /// is checked before anything changes.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the faction is outside the set this world holds,
    /// when the caller names more places than one call carries, when the
    /// strength is outside its range, when a place lies outside the world,
    /// when the faction holds no ground in the cell of a place, and when the
    /// faction inflicted weather too recently.
    ///
    /// # References
    ///
    /// [^1]: ADR-0142, a god inflicts weather only on ground its own faction holds, decision D1. `docs/adrs/draft/adr-0142-a-god-inflicts-weather-only-on-ground-it-holds.md`
    /// [^2]: ADR-0142, a god inflicts weather only on ground its own faction holds, decision D3. `docs/adrs/draft/adr-0142-a-god-inflicts-weather-only-on-ground-it-holds.md`
    pub fn inflict_weather(
        &mut self,
        faction: FactionId,
        places: &[Axial],
        strength: u8,
    ) -> Result<Storm, WeatherError> {
        // The water lands on the weather cell that covers the place, and the
        // weather lattice has a pitch of its own. The holder gate below still
        // asks the level 1 holding, because that is where a holder lives.
        let layout = self.weather_layout;
        let lattice = self.weather_lattice;
        let holding = &self.holding;
        let grid = self.grid;
        let ground = Ground {
            grid,
            cell_of: &move |tile: TileIdx| weather_cell_of(layout, lattice, tile),
            holders_near: &move |address: Axial| {
                let tile = grid.index_of(address)?;
                let key = holding.layout().key_of(tile)?;
                holding.block_mask(holding.layout().block_of_key(key))
            },
        };
        self.weather
            .inflict(faction, places, strength, self.tick, &ground)
    }

    /// Raises a storm over one place.
    ///
    /// The place is a tile, and the storm stands over the weather cell that
    /// covers it. **The storm is imposed and it did not form.** A field of one
    /// layer grows no low of its own, so a caller places one and the field
    /// carries it until it dies.[^1]
    ///
    /// **This is an authoring verb and not a faction power.** It names no
    /// congregation, so the gate that a god meets does not govern it.[^2] The
    /// power that a faction wields is the one that puts water over ground the
    /// faction holds.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the place lies outside the world, when the
    /// setting lies outside the range the field carries, and when the field
    /// already carries as many storms as it holds.
    ///
    /// # References
    ///
    /// [^1]: A cyclone. [`Cyclone`]
    /// [^2]: ADR-0142, a god inflicts weather only on ground its own faction holds, decision D1. `docs/adrs/draft/adr-0142-a-god-inflicts-weather-only-on-ground-it-holds.md`
    pub fn raise_cyclone(
        &mut self,
        place: Axial,
        setting: CycloneSetting,
    ) -> Result<Cyclone, WeatherError> {
        let tile = self
            .grid
            .index_of(place)
            .ok_or(WeatherError::PlaceOutsideWorld(place))?;
        let cell = self
            .weather_cell_of(tile)
            .ok_or(WeatherError::PlaceOutsideWorld(place))?;
        self.weather.raise_cyclone(cell, setting)
    }

    /// Returns the storms that the weather of the world is carrying.
    #[must_use]
    pub fn cyclones(&self) -> &[Cyclone] {
        self.weather.cyclones()
    }

    pub(super) fn cell_of(&self, tile: TileIdx) -> Option<u32> {
        let layout = self.pyramid.layout();
        Some(layout.block_of_key(layout.key_of(tile)?))
    }

    /// Returns the weather cell that covers a tile.
    ///
    /// **This is not the level 1 cell.** The weather lattice states its own
    /// pitch, and the two agree only when the world takes the level 1 pitch.
    /// Every reader of the weather field asks this rather than the level 1
    /// reader, so one fact has one declaration site.[^1]
    ///
    /// **The weather lattice is larger than the world, and this adds the
    /// margin offset.** The margin is a ring of cells that the solve steps
    /// and no reader sees, and the answer here names a cell of the whole
    /// lattice. So a reader that goes through this reads the world wherever
    /// the margin stands, and a reader that indexed a weather plane by a
    /// level 1 cell would read the margin instead.
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub fn weather_cell_of(&self, tile: TileIdx) -> Option<u32> {
        weather_cell_of(self.weather_layout, self.weather_lattice, tile)
    }

    /// Returns the weather lattice, with the margin it carries.
    #[must_use]
    pub const fn weather_lattice(&self) -> PaddedLattice {
        self.weather_lattice
    }

    /// Returns the block geometry of the weather lattice.
    ///
    /// The drawing reads it, because the pitch of the lattice decides whether
    /// a cell value paints as a field or as a tile.
    #[must_use]
    pub const fn weather_layout(&self) -> BlockLayout {
        self.weather_layout
    }

    /// Returns what the recovery rule needs to know about one tile.
    ///
    /// The reader gathers the moisture over the tile and the upgrade that
    /// stands on it. It applies neither. The recovery rule holds the whole of
    /// the arithmetic, so the moisture cannot be read one way here and
    /// another way there.[^1]
    ///
    /// The moisture is the water on the ground of the weather cell that
    /// covers the tile, which is not the level 1 cell unless the world takes
    /// the level 1 pitch.[^2] The
    /// value is the one that the solve of the previous frame left, in the
    /// same way the gather resolve reads it.
    ///
    /// A tile that carries no finished upgrade answers with bare ground.
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    /// [^2]: ADR-0140, weather is a field over the level 1 cell lattice, decision D1. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
    #[must_use]
    pub(super) fn tile_ground(&self, tile: TileIdx) -> TileGround {
        let moisture = self
            .weather_cell_of(tile)
            .map_or(0, |cell| self.weather.ground_at(cell).0);
        let Some(site) = self.upgrades.at(tile) else {
            return TileGround {
                moisture,
                ..TileGround::BARE
            };
        };
        let improvement = self
            .upgrade_table
            .row(site.category, site.level)
            .map_or(0, |row| row.recovery_change);
        TileGround {
            moisture,
            improvement,
            condition: site.condition.0,
        }
    }
}
