//! The methods that build a world, name its extent, and prove it sound.
//!
//! This module holds the constructor, the getters that report the extent, the
//! seed and the faction count, the state hash, the invariant check and the
//! representation. It also holds the tick ceiling and the subsystem census,
//! because both describe the run rather than any one subsystem.
//!
//! The grouping is by what a caller reaches for first. A program builds a
//! world before it reads a tile or steps a frame.

use super::{Presenter, PyWorld};
use crate::errors::ConfigError;
use crate::world::weather::scale_of_tiles;
use cachette_core::{WeatherScale, World as CoreWorld, WorldConfig};
use cachette_view::{Metrics, Motion};
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pymethods]
impl PyWorld {
    /// Builds a world from the given extent, seed and faction count.
    ///
    /// **The prose for this call lives in the doc comment of the class.** The
    /// binding library does not copy the doc comment of a constructor onto the
    /// Python object. Prose written here reaches no reader of the published
    /// reference.[^1]
    ///
    /// # Errors
    ///
    /// Raises `ConfigError` when the arguments do not describe a world. A side
    /// of zero, a faction count above 63, and a weather pitch that is not a
    /// power of two are the cases a caller meets first.
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-325. `docs/FINDINGS.md`
    #[new]
    #[pyo3(signature = (
        width = 64,
        height = 64,
        seed = 0x0123_4567_89ab_cdef,
        faction_count = 4,
        weather_cell_tiles = None,
    ))]
    fn new(
        width: u32,
        height: u32,
        seed: u64,
        faction_count: u16,
        weather_cell_tiles: Option<u32>,
    ) -> PyResult<Self> {
        // **The default lives in the engine and nowhere else.** A caller that
        // states no pitch gets whatever the engine calls its default, so this
        // binding holds no second copy of that number.[^3]
        //
        // [^3]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
        let scale = match weather_cell_tiles {
            None => WeatherScale::DEFAULT,
            Some(tiles) => scale_of_tiles(tiles)?,
        };
        let world = CoreWorld::with_weather_scale(
            WorldConfig {
                width,
                height,
                seed,
                faction_count,
                unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
            },
            scale,
        )
        .map_err(|error| ConfigError::new_err(error.to_string()))?;
        Ok(Self {
            inner: std::sync::Mutex::new(world),
            presenter: std::sync::Mutex::new(Presenter {
                metrics: Metrics::start(),
                outcomes: Vec::new(),
                // The first frame sizes the table from its own pixels. A
                // table that records nothing tweens nothing, which is what a
                // caller that never draws wants.
                motion: Motion::none(),
            }),
        })
    }

    /// The number of tile columns in the world, as an integer.
    ///
    /// This is the value the constructor took for `width`. It never changes.
    #[getter]
    fn width(&self) -> u32 {
        self.lock().grid().width()
    }

    /// The number of tile rows in the world, as an integer.
    ///
    /// This is the value the constructor took for `height`. It never changes.
    #[getter]
    fn height(&self) -> u32 {
        self.lock().grid().height()
    }

    /// The seed the world was built from, as an integer.
    ///
    /// This is the value the constructor took for `seed`. It never changes.
    /// The value is an unsigned 64-bit number.
    ///
    /// **A run can record which world it ran.** A caller that draws a seed
    /// held that number twice before this reader existed, once for the world
    /// and once for whatever else needed it, and nothing failed when the two
    /// copies disagreed.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[getter]
    fn seed(&self) -> u64 {
        self.lock().config().seed
    }

    /// The number of factions the world holds, as an integer.
    ///
    /// This is the value the constructor took for `faction_count`. It never
    /// changes. A faction identifier runs from zero to one below it.
    #[getter]
    fn faction_count(&self) -> u16 {
        self.lock().faction_count()
    }

    /// The number of steps the world has run, as an integer.
    ///
    /// A new world is at tick zero. Each `step` call adds one.
    #[getter]
    fn tick(&self) -> u64 {
        self.lock().tick().0
    }

    /// The number of tiles in the world, as an integer.
    ///
    /// The value is the width multiplied by the height.
    #[getter]
    fn tile_count(&self) -> usize {
        self.lock().tile_count()
    }

    /// The number of tile change events the last step emitted, as an integer.
    ///
    /// The count covers the last step alone. A new world reports zero. Read
    /// the events themselves with `event_log_columns`.
    #[getter]
    fn event_count(&self) -> usize {
        self.lock().event_log().len()
    }

    /// Returns the hash of the whole world state, as an integer.
    ///
    /// The value is an unsigned 64-bit integer. Two worlds that hold the same
    /// state give the same hash. The hash does not depend on the thread count
    /// of any step that ran.[^1]
    ///
    /// **Compare hashes to check that a run repeated.** A hash that differs
    /// means the states differ. Equal hashes do not prove that either run is
    /// correct. A defect that is itself repeatable gives one hash every time.
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    fn state_hash(&self) -> u64 {
        self.lock().state_hash().finish()
    }

    /// Reports whether the world holds its own invariants, as a `bool`.
    ///
    /// The engine checks its internal rules and returns `True` when they all
    /// hold. It is a check of the engine, not of the program that drives it.
    /// It reads the stored structures of the world. A caller runs it in a
    /// test, not on every step.
    fn check_invariants(&self) -> bool {
        self.lock().check_invariants()
    }

    /// Returns a `str` that names the world, its extent and its tick.
    fn __repr__(&self) -> String {
        let world = self.lock();
        // The arguments name the constructor's own parameters, so that the
        // output can be pasted back. A repr that names a field the
        // constructor does not take is a small lie that costs a reader a
        // failed call.
        let grid = world.grid();
        format!(
            "World(width={}, height={}, tick={})",
            grid.width(),
            grid.height(),
            world.tick().0
        )
    }

    /// The tick at which the territory reader fires, as an integer.
    ///
    /// The limit is a value in the balance register.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the tick limit. `docs/reference/balance.md`
    #[getter]
    fn tick_limit(&self) -> u64 {
        self.lock().tick_limit()
    }

    /// Sets the tick at which the territory reader fires.
    ///
    /// At that tick the faction that holds the most tiles wins, and a tie
    /// goes to the lowest faction number. The limit is state that every tick
    /// reads, so two worlds that differ in it hash differently.
    fn set_tick_limit(&self, tick_limit: u64) {
        self.lock().set_tick_limit(tick_limit);
    }

    /// Returns the subsystem census, as a `dict` from a subsystem name to a
    /// count.
    ///
    /// **One Rust table declares the list.** Each row names a subsystem and
    /// the reader that counts what it produced, and this call walks that
    /// table. Nothing else declares the names, so a name here is a name the
    /// engine holds.[^1]
    ///
    /// **No count covers one step.** A count says what the world holds now,
    /// or what the run has made since the world was built. A count of the
    /// second kind never falls, so a zero in it means that the thing never
    /// happened. The build queue counts are rows of this table, and the
    /// queue has no census of its own.[^2]
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
    /// [^2]: Findings register, FND-498. `docs/FINDINGS.md`
    fn subsystem_census<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let census = self.lock().subsystem_census();
        let report = PyDict::new(python);
        for (name, count) in census {
            report.set_item(name, count)?;
        }
        Ok(report)
    }
}
