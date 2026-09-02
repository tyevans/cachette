//! The Python bindings for the Cachette simulation core.
//!
//! This crate wraps the core crate. The core crate has no PyO3 dependency,
//! so no simulation function can take an interpreter token and no system can
//! call Python. That is a compile error and not a review comment.[^1]
//!
//! The step releases the global interpreter lock for its whole run. No
//! Python code runs while the simulation runs.[^2]
//!
//! # References
//!
//! [^1]: ADR-0041, a crate split enforces the boundary at compile time. `docs/adrs/REGISTRY.md`
//! [^2]: ADR-0042, the interpreter is released for the whole step. `docs/adrs/REGISTRY.md`

use cachette_core::{World as CoreWorld, WorldConfig};
use numpy::{PyArray1, ToPyArray};
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

// ADR-0046: one root exception type holds the whole hierarchy. The
// engine never raises a bare runtime error. The macro builds the types,
// because subclassing a Python class under the stable ABI needs a later
// interpreter version and the macro does not.
create_exception!(
    _core,
    CachetteError,
    PyException,
    "The root of every Cachette error."
);
create_exception!(_core, StepError, CachetteError, "A step refused to run.");
create_exception!(
    _core,
    ConfigError,
    CachetteError,
    "The world settings do not describe a world."
);
create_exception!(
    _core,
    SelectorError,
    CachetteError,
    "A selector was not valid."
);
create_exception!(_core, VerbError, CachetteError, "A verb refused a command.");
create_exception!(
    _core,
    ViewError,
    CachetteError,
    "A view was stale or out of scope."
);
create_exception!(
    _core,
    DeterminismError,
    CachetteError,
    "The engine detected a determinism defect."
);
create_exception!(
    _core,
    EnginePanic,
    CachetteError,
    "A Rust panic reached the boundary."
);

/// A simulated world.
#[pyclass(name = "World", module = "cachette._core", frozen)]
pub struct PyWorld {
    inner: std::sync::Mutex<CoreWorld>,
}

#[pymethods]
impl PyWorld {
    /// Builds a world.
    ///
    /// The world is a rhombus, so the extent is a width and a height.
    ///
    /// # Errors
    ///
    /// Raises `ConfigError` when the extent does not describe a world.
    #[new]
    #[pyo3(signature = (width = 64, height = 64, seed = 0x0123_4567_89ab_cdef, faction_count = 4))]
    fn new(width: u32, height: u32, seed: u64, faction_count: u16) -> PyResult<Self> {
        let world = CoreWorld::new(WorldConfig {
            width,
            height,
            seed,
            faction_count,
            unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        })
        .map_err(|error| ConfigError::new_err(error.to_string()))?;
        Ok(Self {
            inner: std::sync::Mutex::new(world),
        })
    }

    /// Returns the number of columns in the world.
    #[getter]
    fn width(&self) -> u32 {
        self.lock().grid().width()
    }

    /// Returns the number of rows in the world.
    #[getter]
    fn height(&self) -> u32 {
        self.lock().grid().height()
    }

    /// Returns the current tick.
    #[getter]
    fn tick(&self) -> u64 {
        self.lock().tick().0
    }

    /// Returns the number of tiles.
    #[getter]
    fn tile_count(&self) -> usize {
        self.lock().tile_count()
    }

    /// Returns the number of events that the last step emitted.
    #[getter]
    fn event_count(&self) -> usize {
        self.lock().event_log().len()
    }

    /// Returns the hash of the whole state.
    fn state_hash(&self) -> u64 {
        self.lock().state_hash().finish()
    }

    /// Reports whether the world holds its invariants.
    fn check_invariants(&self) -> bool {
        self.lock().check_invariants()
    }

    /// Runs one frame and returns the number of events.
    ///
    /// The method releases the global interpreter lock for the whole step.
    fn step(&self, python: Python<'_>, threads: usize) -> PyResult<usize> {
        // ADR-0042: release the interpreter for the whole step. The
        // closure may not capture the interpreter token, so the compiler
        // rejects a mid-step callback a second time.
        python.detach(|| {
            let mut world = self.lock();
            match world.step(threads) {
                Ok(events) => Ok(events.len()),
                Err(error) => Err(StepError::new_err(error.to_string())),
            }
        })
    }

    /// Returns the raw event log of the last step as bytes.
    fn event_log_bytes<'py>(&self, python: Python<'py>) -> Bound<'py, pyo3::types::PyBytes> {
        pyo3::types::PyBytes::new(python, self.lock().event_log_bytes())
    }

    /// Copies the tile value column into a new array.
    ///
    /// This method copies. A separate method returns the underlying array
    /// without a copy, and the project does not claim zero copy here.
    fn tile_values<'py>(&self, python: Python<'py>) -> Bound<'py, PyArray1<i32>> {
        let world = self.lock();
        let raw: Vec<i32> = world.tile_values().iter().map(|value| value.0).collect();
        raw.to_pyarray(python)
    }

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
}

impl PyWorld {
    /// Takes the lock, recovering from a poisoned lock.
    fn lock(&self) -> std::sync::MutexGuard<'_, CoreWorld> {
        self.inner.lock().unwrap_or_else(|error| error.into_inner())
    }
}

/// Returns the version of the engine.
#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Builds the extension module.
#[pymodule]
#[pyo3(name = "_core")]
fn cachette_core_module(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyWorld>()?;
    module.add_function(wrap_pyfunction!(version, module)?)?;
    module.add("CachetteError", module.py().get_type::<CachetteError>())?;
    module.add("StepError", module.py().get_type::<StepError>())?;
    module.add("ConfigError", module.py().get_type::<ConfigError>())?;
    module.add("SelectorError", module.py().get_type::<SelectorError>())?;
    module.add("VerbError", module.py().get_type::<VerbError>())?;
    module.add("ViewError", module.py().get_type::<ViewError>())?;
    module.add(
        "DeterminismError",
        module.py().get_type::<DeterminismError>(),
    )?;
    module.add("EnginePanic", module.py().get_type::<EnginePanic>())?;
    Ok(())
}
