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
    #[new]
    #[pyo3(signature = (tile_count = 4096, seed = 0x0123_4567_89ab_cdef, faction_count = 4))]
    fn new(tile_count: u32, seed: u64, faction_count: u16) -> Self {
        Self {
            inner: std::sync::Mutex::new(CoreWorld::new(WorldConfig {
                tile_count,
                seed,
                faction_count,
            })),
        }
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
        format!(
            "World(tile_count={}, tick={})",
            world.tile_count(),
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
