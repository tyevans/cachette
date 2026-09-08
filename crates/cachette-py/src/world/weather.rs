//! The weather lattice, its readers, and the storms a caller raises.
//!
//! This module holds the pitch and the count of the weather cells, the cloud,
//! wind, air and ground water readers, the totals of the lattice, the verb
//! that inflicts weather, and the cyclones.
//!
//! The weather runs on a lattice of its own, coarser than the tiles, so its
//! readers answer per cell where the tile readers answer per tile. That
//! difference is why the two groups are separate.

use super::PyWorld;
use crate::errors::{ConfigError, VerbError};
use cachette_core::weather::CLOUD_SHARE_WHOLE;
use cachette_core::TileIdx;
use cachette_core::{Axial, Cyclone, CycloneSetting, FactionId, WeatherScale, Wind};
use numpy::{PyArray1, ToPyArray};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

#[pymethods]
impl PyWorld {
    /// The side of one weather cell in tiles, as an integer.
    ///
    /// This is the value the constructor took for `weather_cell_tiles`, or
    /// the engine default when the caller stated none. One means that each
    /// tile carries its own weather. It never changes.
    #[getter]
    fn weather_cell_tiles(&self) -> u32 {
        self.lock().weather_layout().block_edge()
    }

    /// The number of weather cell columns across the world, as an integer.
    ///
    /// **A weather array is in weather cell order, not in level 1 cell
    /// order.** A watcher takes `index % weather_cells_wide` for the column
    /// of a cell and `index // weather_cells_wide` for its row. The two
    /// lattices agree only when the world takes the level 1 weather pitch.
    #[getter]
    fn weather_cells_wide(&self) -> u32 {
        self.lock().weather_layout().blocks_wide()
    }

    /// The number of weather cells in the world, as an integer.
    ///
    /// Read it before a long run at a fine pitch, because a fine pitch on a
    /// large world holds one cell for every tile.
    ///
    /// **The engine steps more cells than this.** It simulates a margin of
    /// cells around the world, and the cost of the stage follows the count
    /// with the margin in it. This count is what a weather array holds.
    #[getter]
    fn weather_cell_count(&self) -> u64 {
        let layout = self.lock().weather_layout();
        u64::from(layout.blocks_wide()) * u64::from(layout.blocks_high())
    }

    /// Copies the cloud share over every tile into a new NumPy array.
    ///
    /// Returns a one-dimensional array of `numpy.int32`, one entry for each
    /// tile, in row-major order. Entry `r * width + q` is the tile at the
    /// address `(q, r)`. The order is the order that `tile_holders` uses.
    ///
    /// **Each entry is the share of the sky that a watcher sees as cloud**,
    /// from none to `cloud_share_whole`. It is the air held over the cell
    /// against what that air can hold, and not the air against a mark that
    /// every cell shares.
    ///
    /// **The array stands at tile resolution and the weather stands on the
    /// level 1 cell**, so every tile of one cell reports the same share. Each
    /// entry is the number that `air_at` answers for that tile, put through
    /// the same share. **The engine owns the map from a tile to its weather
    /// cell.** The weather lattice carries a margin around the world, so a
    /// caller that indexed a weather plane by a world address would read the
    /// wrong cell. This reader therefore takes an address and never a cell,
    /// and it publishes no map.[^1]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-569. `docs/FINDINGS.md`
    fn cloud_shares<'py>(&self, python: Python<'py>) -> Bound<'py, PyArray1<i32>> {
        let raw: Vec<i32> = python.detach(|| {
            let world = self.lock();
            let grid = world.grid();
            (0..grid.tile_count())
                .map(|index| {
                    grid.address_of(TileIdx(index))
                        .and_then(|address| world.cloud_share_at(address))
                        .and_then(|share| i32::try_from(share).ok())
                        .unwrap_or(0)
                })
                .collect()
        });
        raw.to_pyarray(python)
    }

    /// The largest cloud share, as an integer.
    ///
    /// A `cloud_shares` entry runs from zero to this number. The engine
    /// declares it, so a caller that scales the share holds no second copy of
    /// the ceiling.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[getter]
    fn cloud_share_whole(&self) -> i64 {
        CLOUD_SHARE_WHOLE
    }

    /// Copies the wind over every tile into two NumPy arrays.
    ///
    /// Returns a `dict` with the keys `q` and `r`. Each holds a
    /// one-dimensional array of `numpy.int32`, one entry for each tile, in
    /// row-major order. Entry `r * width + q` is the tile at the address
    /// `(q, r)`. The order is the order that `tile_holders` uses.
    ///
    /// **The wind is an integer vector over the two axes of the cell
    /// lattice.** The lattice has three axes and two of them are free, so the
    /// third part is `-(q + r)` and the engine stores it nowhere. The parts
    /// are whole lattice steps and not a fixed-point value.
    ///
    /// **This is the wind itself and not a drawing of it.** A caller that
    /// wants a heading or a speed derives it from the two parts. The map
    /// overlay paints the same wind as one of six hues, and that palette is a
    /// choice of the renderer. A second copy of it here would be one fact in
    /// two places.[^2]
    ///
    /// **The array stands at tile resolution and the wind stands on the level
    /// 1 cell**, so every tile of one cell reports the same vector. The
    /// engine owns the map from a tile to its weather cell, in the way the
    /// cloud reader describes.
    ///
    /// The wind is carried state, so a watcher who reads it reads what the
    /// next step will read.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D1. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
    /// [^2]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    fn tile_winds<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let (along_q, along_r): (Vec<i32>, Vec<i32>) = python.detach(|| {
            let world = self.lock();
            let grid = world.grid();
            (0..grid.tile_count())
                .map(|index| {
                    let wind = grid
                        .address_of(TileIdx(index))
                        .and_then(|address| world.wind_at(address))
                        .unwrap_or(Wind::STILL);
                    (wind.q, wind.r)
                })
                .unzip()
        });
        let axes = PyDict::new(python);
        axes.set_item("q", along_q.to_pyarray(python))?;
        axes.set_item("r", along_r.to_pyarray(python))?;
        Ok(axes)
    }

    /// Puts weather over a set of places, at the command of a god.
    ///
    /// The faction is the number of the faction whose congregation the god
    /// directs, from 0 to one below `faction_count`. The places are a
    /// sequence of `(q, r)` pairs of integers, and each pair names a tile.
    /// The strength is an integer from 1 to `weather_strength_ceiling`.
    /// Returns a `dict`.
    ///
    /// **Weather lives on the level 1 cell, not on the tile.** The water
    /// lands on the cell that covers each place, and a cell covers a block of
    /// tiles. Two places inside one cell are therefore one place, and the
    /// report says how many cells took water.
    ///
    /// **A god acts only where its own people hold the ground.** The cell of
    /// every place must hold at least one tile of that faction. This is the
    /// gate that the engine puts on speaking to another faction. The divine
    /// power does not escape it.[^1]
    ///
    /// **One call names a whole set, and the engine answers once.** The cost
    /// follows the number of places and not the number of units. The weather
    /// that follows costs the level 1 lattice rather than the world.[^2]
    ///
    /// **The set is all or nothing.** Every place is resolved, every gate is
    /// checked, and the cooldown is checked, before anything changes. One
    /// refusal leaves the world exactly as it was.
    ///
    /// The keys of the result are:
    ///
    /// - `cells`, an integer. How many level 1 cells took water. A cell that
    ///   two places named counts once.
    /// - `drops`, an integer. The water this call put into the air, in drops.
    ///   A drop is a whole number and it is not a fixed-point value.
    /// - `ready_at`, an integer. The first tick at which this faction may
    ///   inflict weather again. Read `tick` for where the world is now.
    ///
    /// A faction waits `weather_cooldown_ticks` ticks between one storm and
    /// the next.
    ///
    /// The water enters the air. It reaches the ground at the end of the next
    /// step. The step after that is the first one whose gathering reads it.
    /// Read `ground_water_at` for what has landed.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no faction of this world,
    /// when the caller names more than `weather_places_ceiling` places, when
    /// the strength is 0 or above `weather_strength_ceiling`, when a place
    /// lies outside the world, when the faction holds no ground in the cell
    /// of a place, and when the faction inflicted weather too recently.
    ///
    /// # References
    ///
    /// [^1]: ADR-0142, a god inflicts weather only on ground its own faction holds, decision D1. `docs/adrs/draft/adr-0142-a-god-inflicts-weather-only-on-ground-it-holds.md`
    /// [^2]: ADR-0140, weather is a field over the level 1 cell lattice, decision D1. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
    #[pyo3(signature = (faction, places, strength = 1))]
    fn inflict_weather<'py>(
        &self,
        python: Python<'py>,
        faction: u16,
        places: Vec<(i32, i32)>,
        strength: u8,
    ) -> PyResult<Bound<'py, PyDict>> {
        let addresses: Vec<Axial> = places.iter().map(|(q, r)| Axial::new(*q, *r)).collect();
        let storm = self
            .lock()
            .inflict_weather(FactionId(faction), &addresses, strength)
            .map_err(|error| VerbError::new_err(error.to_string()))?;
        let report = PyDict::new(python);
        report.set_item("cells", storm.cells)?;
        report.set_item("drops", storm.drops)?;
        report.set_item("ready_at", storm.ready_at.0)?;
        Ok(report)
    }

    /// The water in the air above one place, as an integer.
    ///
    /// The place is a tile, as the pair `(q, r)` of integers. The answer is
    /// the water above the level 1 cell that covers that tile, in drops. A
    /// drop is a whole number and it is not a fixed-point value.
    ///
    /// Weather lives on the level 1 cell, so two tiles of one cell answer the
    /// same number.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the address lies outside the world.
    fn air_at(&self, q: i32, r: i32) -> PyResult<i64> {
        self.lock().air_at(Axial::new(q, r)).ok_or_else(|| {
            VerbError::new_err(format!("the address ({q}, {r}) is outside the world"))
        })
    }

    /// The water on the ground at one place, as an integer.
    ///
    /// The place is a tile, as the pair `(q, r)` of integers. The answer is
    /// the water on the ground of the level 1 cell that covers that tile, in
    /// drops. A drop is a whole number and it is not a fixed-point value.
    ///
    /// The ground of a cell counts as wet at `weather_wet_mark` drops. A unit
    /// that gathers on wet ground takes more in one tick than a unit on dry
    /// ground. Read `ground_is_wet` for the answer directly.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the address lies outside the world.
    fn ground_water_at(&self, q: i32, r: i32) -> PyResult<i64> {
        self.lock()
            .ground_water_at(Axial::new(q, r))
            .ok_or_else(|| {
                VerbError::new_err(format!("the address ({q}, {r}) is outside the world"))
            })
    }

    /// Whether the ground at one place is wet, as a `bool`.
    ///
    /// The place is a tile, as the pair `(q, r)` of integers. The answer is
    /// about the level 1 cell that covers that tile. Two tiles of one cell
    /// answer the same.
    ///
    /// A unit that gathers on wet ground takes more in one tick than a unit
    /// on dry ground.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the address lies outside the world.
    fn ground_is_wet(&self, q: i32, r: i32) -> PyResult<bool> {
        self.lock().ground_is_wet(Axial::new(q, r)).ok_or_else(|| {
            VerbError::new_err(format!("the address ({q}, {r}) is outside the world"))
        })
    }

    /// What the weather of the whole world holds, as a `dict`.
    ///
    /// The water keys are in drops. A drop is a whole number and it is not a
    /// fixed-point value.
    ///
    /// The keys are:
    ///
    /// - `air`, an integer. The water in the air over the whole world.
    /// - `ground`, an integer. The water on the ground over the whole world.
    /// - `evaporated`, an integer. The water that has left the ground since
    ///   the world was built.
    /// - `raised`, an integer. The water that has entered the air since the
    ///   world was built, from the sea and from every god.
    /// - `wet_cells`, an integer. How many weather cells of the world hold at
    ///   least `weather_wet_mark` drops on the ground.
    ///
    /// **The account is exact.** The sum of `air`, `ground` and `evaporated`
    /// equals `raised` at every moment. A pass moves water and never scales
    /// it.[^1]
    ///
    /// **`air` and `ground` cover the whole weather lattice, and that lattice
    /// is larger than the world.** The engine simulates a margin of cells
    /// around the world so that the border of the world has real upwind. The
    /// margin holds water, and the account balances only when the totals hold
    /// it too. `wet_cells` counts the world alone, and so do the weather
    /// arrays.
    ///
    /// # References
    ///
    /// [^1]: ADR-0141, a weather pass moves water and never scales it, decision D2. `docs/adrs/draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md`
    fn weather_totals<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let field = world.weather();
        let report = PyDict::new(python);
        report.set_item("air", field.air_total().0)?;
        report.set_item("ground", field.ground_total().0)?;
        report.set_item("evaporated", field.evaporated())?;
        report.set_item("raised", field.raised())?;
        report.set_item("wet_cells", field.wet_cells())?;
        Ok(report)
    }

    /// The water on the ground of every weather cell of the world, as a
    /// NumPy array.
    ///
    /// The result is a one-dimensional array of `numpy.int64`, in cell index
    /// order, and the unit is drops. A drop is a whole number and it is not a
    /// fixed-point value. Take `index % weather_cells_wide` for the column of
    /// a cell and `index // weather_cells_wide` for its row. **The weather
    /// pitch is a parameter of the constructor**, so it agrees with the level
    /// 1 pitch only when the world takes the level 1 weather pitch.
    ///
    /// **This is one crossing, and it replaces a loop.** A watcher that read
    /// each cell through `ground_water_at` would pay one crossing for each
    /// cell. The control plane never loops over the world.
    ///
    /// **The array covers the world and the totals cover the margin around
    /// it.** The sum of this array is therefore at or below the `ground`
    /// total, and it falls below it as soon as water crosses the border.
    ///
    /// The array is empty when no water has entered the world yet.
    fn weather_ground<'py>(&self, python: Python<'py>) -> Bound<'py, PyArray1<i64>> {
        let world = self.lock();
        // **The reading crops the margin away.** The weather lattice is
        // larger than the world, and a watcher indexes this array by the cell
        // columns of the world.
        let plane: Vec<i64> = world
            .weather()
            .ground_over_world()
            .iter()
            .map(|drops| drops.0)
            .collect();
        plane.to_pyarray(python)
    }

    /// The water in the air over every weather cell of the world, as a NumPy
    /// array.
    ///
    /// The shape, the order and the unit are those of `weather_ground`, and
    /// the sum stands against the `air` total in the same way.
    fn weather_air<'py>(&self, python: Python<'py>) -> Bound<'py, PyArray1<i64>> {
        let world = self.lock();
        // The reading crops the margin away, as the ground reading does.
        let plane: Vec<i64> = world
            .weather()
            .air_over_world()
            .iter()
            .map(|drops| drops.0)
            .collect();
        plane.to_pyarray(python)
    }

    /// The number of level 1 cells across the world, as an integer.
    ///
    /// **This is the level 1 pitch, and the weather has a pitch of its own.**
    /// Read `weather_cells_wide` to index a weather array. The two agree only
    /// when the world takes the level 1 weather pitch.
    #[getter]
    fn cells_wide(&self) -> u32 {
        self.lock().pyramid().layout().blocks_wide()
    }

    /// Raises a travelling storm over one place.
    ///
    /// The place is a tile, as the pair `(q, r)` of integers. The storm
    /// stands over the weather cell that covers it, and it moves, rains and
    /// dies on its own from there.
    ///
    /// **The storm is imposed and it did not form.** The field holds one
    /// layer of air, and a layer grows no low of its own, so a caller places
    /// one and the engine carries it.
    ///
    /// The kind is `"tropical"` or `"severe"`. The two are one object at two
    /// points of one parameter set, and a caller may state the three
    /// parameters instead. **The severe kind is not a resolved tornado.** One
    /// weather cell spans tens to hundreds of kilometres, and a tornado is
    /// under one, so the severe kind is an intensity carried on a cell.
    ///
    /// The answer is a dictionary of the storm that was raised.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the place lies outside the world, when the
    /// kind is not one this world holds, when a stated parameter lies outside
    /// its range, and when the field already carries as many storms as it
    /// holds.
    #[pyo3(signature = (place, kind = "tropical", depth = None, radius = None, life = None))]
    fn raise_cyclone<'py>(
        &self,
        python: Python<'py>,
        place: (i32, i32),
        kind: &str,
        depth: Option<i32>,
        radius: Option<i32>,
        life: Option<u32>,
    ) -> PyResult<Bound<'py, PyDict>> {
        let mut setting = match kind {
            "tropical" => CycloneSetting::TROPICAL,
            "severe" => CycloneSetting::SEVERE,
            other => {
                return Err(VerbError::new_err(format!(
                    "the kind {other} is not tropical and not severe"
                )))
            }
        };
        if let Some(depth) = depth {
            setting.depth = depth;
        }
        if let Some(radius) = radius {
            setting.radius = radius;
        }
        if let Some(life) = life {
            setting.life = life;
        }
        let storm = self
            .lock()
            .raise_cyclone(Axial::new(place.0, place.1), setting)
            .map_err(|error| VerbError::new_err(error.to_string()))?;
        cyclone_report(python, storm)
    }

    /// The storms that the world is carrying, as a list of dictionaries.
    ///
    /// Each entry holds the identity, the cell the eye stands over, the
    /// depth, the radius, the age and the life of one storm. The list is in
    /// the order the storms were raised in.
    fn cyclones<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let world = self.lock();
        let reports: Vec<Bound<'py, PyDict>> = world
            .cyclones()
            .iter()
            .map(|storm| cyclone_report(python, *storm))
            .collect::<PyResult<_>>()?;
        PyList::new(python, reports)
    }

    /// The storms that the field carries at once, as an integer.
    #[getter]
    fn cyclone_ceiling(&self) -> usize {
        cachette_core::CYCLONE_CEILING
    }

    /// The largest strength that one storm may carry, as an integer.
    #[getter]
    fn weather_strength_ceiling(&self) -> u8 {
        cachette_core::STRENGTH_CEILING
    }

    /// The most places that one call to `inflict_weather` may name, as an
    /// integer.
    #[getter]
    fn weather_places_ceiling(&self) -> usize {
        cachette_core::PLACES_CEILING
    }

    /// The ticks that a faction waits between one storm and the next, as an
    /// integer.
    #[getter]
    fn weather_cooldown_ticks(&self) -> u64 {
        cachette_core::COOLDOWN_TICKS
    }

    /// The water on the ground at which a cell counts as wet, in drops.
    ///
    /// The value is an integer. A drop is a whole number and it is not a
    /// fixed-point value.
    #[getter]
    fn weather_wet_mark(&self) -> i64 {
        cachette_core::WET_MARK.0
    }
}

/// Returns one storm as a dictionary.
///
/// The eye is the cell of the whole weather lattice that the storm stands
/// over, as the pair `(q, r)`. **The margin is part of that lattice**, so a
/// storm may stand at a cell that covers no tile of the world.
///
/// # Errors
///
/// Returns an error when the interpreter refuses to hold the dictionary.
fn cyclone_report(python: Python<'_>, storm: Cyclone) -> PyResult<Bound<'_, PyDict>> {
    let report = PyDict::new(python);
    let eye = storm.eye();
    report.set_item("id", storm.id)?;
    report.set_item("eye", (eye.q, eye.r))?;
    report.set_item("depth", storm.depth)?;
    report.set_item("radius", storm.radius)?;
    report.set_item("age", storm.age)?;
    report.set_item("life", storm.life)?;
    Ok(report)
}

/// Turns a weather cell side in tiles into the scale the engine takes.
///
/// **A watcher states a pitch in tiles, not as a logarithm.** The engine
/// carries the base-two logarithm of the side, because the lattice geometry
/// needs a shift. A caller of the control plane should not have to know that,
/// so the conversion happens once, here.
///
/// # Errors
///
/// Raises `ConfigError` when the side is zero, when it is not a power of two,
/// or when it is above the largest side the engine carries.
pub(crate) fn scale_of_tiles(tiles: u32) -> PyResult<WeatherScale> {
    if tiles == 0 || !tiles.is_power_of_two() {
        return Err(ConfigError::new_err(format!(
            "the weather cell side {tiles} is not a power of two"
        )));
    }
    WeatherScale::from_bits(tiles.trailing_zeros())
        .map_err(|error| ConfigError::new_err(error.to_string()))
}
