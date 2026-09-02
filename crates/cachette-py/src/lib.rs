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

use cachette_core::{
    Axial, Entity, FactionId, Fix32, ResourceKind, World as CoreWorld, WorldConfig,
};
use numpy::{PyArray1, ToPyArray};
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::PyDict;

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
    /// This method copies, and it also generates. The world holds no array
    /// of tile values, so the call visits every tile. A caller that wants
    /// one tile asks for one tile.
    fn tile_values<'py>(&self, python: Python<'py>) -> Bound<'py, PyArray1<i32>> {
        let world = self.lock();
        let raw: Vec<i32> = world
            .copy_tile_values()
            .iter()
            .map(|value| value.0)
            .collect();
        raw.to_pyarray(python)
    }

    /// Returns the tile change log of the last step, one column for each
    /// field.
    ///
    /// The keys are the field names of the event. The caller reads a field
    /// by its name, so no caller holds a byte offset, a field width or a
    /// field order. Those live in the Rust source and nowhere else.[^1]
    ///
    /// The value column carries the fixed-point value as its raw integer.
    /// It is never a floating point number, because a float in simulated
    /// state does not add associatively and this is that state leaving the
    /// engine.[^2]
    ///
    /// This method copies each column. The log of one step is small next to
    /// the world.[^3]
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-060. `docs/DECISIONS.md`
    /// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^3]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the dictionary cannot be built.
    fn event_log_columns<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let log = world.event_log();
        let columns = PyDict::new(python);
        let tick: Vec<u64> = log.iter().map(|event| event.tick.0).collect();
        let tile: Vec<u32> = log.iter().map(|event| event.tile.0).collect();
        let value: Vec<i32> = log.iter().map(|event| event.value.0).collect();
        let holder: Vec<u16> = log.iter().map(|event| event.holder.to_bits()).collect();
        let kind: Vec<u8> = log.iter().map(|event| event.kind).collect();
        columns.set_item("tick", tick.to_pyarray(python))?;
        columns.set_item("tile", tile.to_pyarray(python))?;
        columns.set_item("value", value.to_pyarray(python))?;
        columns.set_item("holder", holder.to_pyarray(python))?;
        columns.set_item("kind", kind.to_pyarray(python))?;
        Ok(columns)
    }

    /// Returns the gather log of the last step, one column for each field.
    ///
    /// The unit column holds the whole identity of the unit that took the
    /// amount. It is not a slot index. A slot index survives the death of
    /// what it named, and a reader that held one would report on the next
    /// occupant of the slot with nothing failing.[^1]
    ///
    /// Hand a value from this column back to `soldier_tile` to read the
    /// unit. The engine resolves it, and it refuses a dead one.[^1]
    ///
    /// This method copies each column.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decisions D1 and D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    /// [^2]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the dictionary cannot be built.
    fn gather_log_columns<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let log = world.gather_log();
        let columns = PyDict::new(python);
        let tick: Vec<u64> = log.iter().map(|event| event.tick.0).collect();
        let unit: Vec<u64> = log.iter().map(|event| event.unit).collect();
        let tile: Vec<u32> = log.iter().map(|event| event.tile.0).collect();
        let amount: Vec<u32> = log.iter().map(|event| event.amount).collect();
        let kind: Vec<u8> = log.iter().map(|event| event.kind).collect();
        columns.set_item("tick", tick.to_pyarray(python))?;
        columns.set_item("unit", unit.to_pyarray(python))?;
        columns.set_item("tile", tile.to_pyarray(python))?;
        columns.set_item("amount", amount.to_pyarray(python))?;
        columns.set_item("kind", kind.to_pyarray(python))?;
        Ok(columns)
    }

    /// Returns the number of gather events that the last step emitted.
    #[getter]
    fn gather_count(&self) -> usize {
        self.lock().gather_log().len()
    }

    /// Returns the number of soldiers alive in the world.
    ///
    /// The engine counts them. A caller never counts a population by
    /// walking it, because a soldier is the mass tier.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D1. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
    #[getter]
    fn soldier_count(&self) -> u32 {
        self.lock().soldiers().len()
    }

    /// Adds a soldier at each address and returns the identity column.
    ///
    /// The call takes a set and answers once. It is not a per-unit verb that
    /// a caller repeats, because a soldier is the mass tier and no caller
    /// walks that population.[^1] The identities come back as one column, in
    /// the order of the addresses.
    ///
    /// **The set is all or nothing.** An address the world refuses removes
    /// every soldier this call made and raises. A caller that got half a
    /// population and an error would have to work out which half, and the
    /// engine already knows.
    ///
    /// The verb is set-valued at the boundary. It is still a loop inside, and
    /// spawning has no cheaper whole-set algorithm today.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D1. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
    /// [^2]: Decisions register, DEC-063. `docs/DECISIONS.md`
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the arena is full, when an address is outside
    /// the world, when the ground admits no unit, or when the world has no
    /// such faction. The error names the address that refused.
    fn spawn_soldiers<'py>(
        &self,
        python: Python<'py>,
        addresses: Vec<(i32, i32)>,
        faction: u16,
    ) -> PyResult<Bound<'py, PyArray1<u64>>> {
        let mut world = self.lock();
        let mut made: Vec<u64> = Vec::with_capacity(addresses.len());
        for (q, r) in addresses {
            match world.spawn_soldier(Axial::new(q, r), FactionId(faction)) {
                Ok(unit) => made.push(unit.to_bits()),
                Err(error) => {
                    // Leave nothing half-made. The founding takes the same
                    // path when a group will not fit the place it chose.
                    for unit in &made {
                        let entity = world
                            .resolve_soldier(*unit)
                            .expect("this call made the identity a moment ago");
                        world.despawn_soldier(entity);
                    }
                    return Err(VerbError::new_err(format!(
                        "the address ({q}, {r}) refused a soldier: {error}"
                    )));
                }
            }
        }
        Ok(made.to_pyarray(python))
    }

    /// Removes every soldier the identities name.
    ///
    /// **The set is all or nothing.** Every identity resolves before anything
    /// is removed, so one dead identity removes nothing and raises.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live soldier.
    fn despawn_soldiers(&self, units: Vec<u64>) -> PyResult<()> {
        let mut world = self.lock();
        let mut resolved = Vec::with_capacity(units.len());
        for unit in &units {
            resolved.push(resolve(&world, *unit)?);
        }
        for entity in resolved {
            assert!(
                world.despawn_soldier(entity),
                "a resolved identity must name a soldier the arena can remove"
            );
        }
        Ok(())
    }

    /// Tells every soldier the identities name to gather a kind of resource.
    ///
    /// The kind is the number the gather event carries in its `kind` column.
    ///
    /// **The set is all or nothing.** Every identity resolves, and the kind is
    /// checked, before any order is given.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live soldier. Raises
    /// `VerbError` when the number names no kind.
    fn order_gather(&self, units: Vec<u64>, kind: u8) -> PyResult<()> {
        let mut world = self.lock();
        let kind = ResourceKind::from_u8(kind)
            .ok_or_else(|| VerbError::new_err(format!("{kind} names no resource kind")))?;
        let mut resolved = Vec::with_capacity(units.len());
        for unit in &units {
            resolved.push(resolve(&world, *unit)?);
        }
        for entity in resolved {
            assert!(
                world.order_gather(entity, kind),
                "a resolved identity must name a soldier the arena can order"
            );
        }
        Ok(())
    }

    /// Returns the tile that one soldier stands on.
    ///
    /// The engine resolves the identity against the arena. A soldier that
    /// died leaves its slot to another soldier, and this method refuses the
    /// dead identity rather than report on the new occupant.[^1]
    ///
    /// **This read stays singular while the write verbs take a set.** A set
    /// form would have to choose between failing the whole call for one dead
    /// identity and returning a value that stands for nothing. That value is
    /// the false answer the record forbids, so the read answers for one
    /// identity and says which one failed.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no live soldier.
    fn soldier_tile(&self, unit: u64) -> PyResult<u32> {
        let world = self.lock();
        let entity = resolve(&world, unit)?;
        world
            .soldiers()
            .tile(entity)
            .map(|tile| tile.0)
            .ok_or_else(|| ViewError::new_err(format!("the identity {unit} names no live soldier")))
    }

    /// Returns the number of settlements standing in the world.
    #[getter]
    fn settlement_count(&self) -> u32 {
        self.lock().settlements().len()
    }

    /// Founds a settlement at each address and returns the identity column.
    ///
    /// The call takes a set and answers once. The identities come back as
    /// one column, in the order of the addresses.
    ///
    /// **The set is all or nothing.** An address the world refuses destroys
    /// every settlement this call made and raises.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the arena is full, when an address is outside
    /// the world, when the ground admits nobody, when the world has no such
    /// faction, or when a settlement already stands on the tile. The error
    /// names the address that refused.
    fn found_settlements<'py>(
        &self,
        python: Python<'py>,
        addresses: Vec<(i32, i32)>,
        faction: u16,
    ) -> PyResult<Bound<'py, PyArray1<u64>>> {
        let mut world = self.lock();
        let mut made: Vec<u64> = Vec::with_capacity(addresses.len());
        for (q, r) in addresses {
            match world.found_settlement(Axial::new(q, r), FactionId(faction)) {
                Ok(site) => made.push(site.to_bits()),
                Err(error) => {
                    // Leave nothing half-made, in the same way the soldier
                    // spawn does.
                    for site in &made {
                        let entity = world
                            .resolve_settlement(*site)
                            .expect("this call made the identity a moment ago");
                        world.destroy_settlement(entity);
                    }
                    return Err(VerbError::new_err(format!(
                        "the address ({q}, {r}) refused a settlement: {error}"
                    )));
                }
            }
        }
        Ok(made.to_pyarray(python))
    }

    /// Changes what a set of sites wants of one kind of work.
    ///
    /// **The command names no unit.** It says what a place wants, and the
    /// engine turns that into a number of positions of each kind at the next
    /// rebalance. A caller that named the workers would be looping over
    /// entities, and the control plane never does that.[^1]
    ///
    /// The kind is the number that the gather event carries in its `kind`
    /// column. The target is a Q16.16 value as its raw integer, because a
    /// float in simulated state does not add associatively.[^2]
    ///
    /// **The set is all or nothing.** Every identity resolves, and the target
    /// is checked, before anything is written.
    ///
    /// # References
    ///
    /// [^1]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
    /// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live settlement. Raises
    /// `VerbError` when the number names no kind, or when the target is
    /// below zero.
    fn prefer_at_sites(&self, sites: Vec<u64>, kind: u8, target: i32) -> PyResult<()> {
        let mut world = self.lock();
        let kind = ResourceKind::from_u8(kind)
            .ok_or_else(|| VerbError::new_err(format!("{kind} names no kind of work")))?;
        let mut resolved = Vec::with_capacity(sites.len());
        for site in &sites {
            resolved.push(resolve_site(&world, *site)?);
        }
        world
            .prefer_at_sites(&resolved, kind, Fix32(target))
            .map_err(|error| VerbError::new_err(error.to_string()))
    }

    /// Returns the positions that one site holds, one column for each field.
    ///
    /// The columns hold the positions of the site and nothing else. An entry
    /// of the storage that is no position does not appear.
    ///
    /// The holder column carries the whole identity of the unit that holds
    /// each position, and zero where a position holds nobody. It is not a
    /// slot index.[^1]
    ///
    /// **This read stays singular while the write verb takes a set**, for
    /// the same reason the unit read does: a set form would have to answer
    /// for a dead identity with a value that stands for nothing.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no live settlement, or
    /// when the dictionary cannot be built.
    fn site_positions<'py>(&self, python: Python<'py>, site: u64) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let entity = resolve_site(&world, site)?;
        let row = world
            .site_positions(entity)
            .ok_or_else(|| ViewError::new_err(format!("the identity {site} names no live site")))?;
        let held: Vec<_> = row.iter().filter(|entry| entry.exists()).collect();
        let columns = PyDict::new(python);
        let kind: Vec<u8> = held.iter().map(|entry| entry.kind_number()).collect();
        let rank: Vec<u8> = held.iter().map(|entry| entry.rank()).collect();
        let holder: Vec<u64> = held.iter().map(|entry| entry.holder_bits()).collect();
        columns.set_item("kind", kind.to_pyarray(python))?;
        columns.set_item("rank", rank.to_pyarray(python))?;
        columns.set_item("holder", holder.to_pyarray(python))?;
        Ok(columns)
    }

    /// Returns what one site wants of each kind of work.
    ///
    /// The column holds one Q16.16 value for each kind, as its raw integer,
    /// in the order of the kind numbering.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no live settlement.
    fn site_preference<'py>(
        &self,
        python: Python<'py>,
        site: u64,
    ) -> PyResult<Bound<'py, PyArray1<i32>>> {
        let world = self.lock();
        let entity = resolve_site(&world, site)?;
        let preference = world
            .site_preference(entity)
            .ok_or_else(|| ViewError::new_err(format!("the identity {site} names no live site")))?;
        let raw: Vec<i32> = ResourceKind::ALL
            .iter()
            .map(|kind| preference.target(*kind).0)
            .collect();
        Ok(raw.to_pyarray(python))
    }

    /// Sets how often the engine rebalances the positions of every site.
    ///
    /// The period is in ticks, and the phase is the offset inside it. The
    /// interval is a parameter of the world.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the period is zero, or above the range that
    /// the scaling multiply takes.
    fn set_position_schedule(&self, period: u32, phase: u32) -> PyResult<()> {
        self.lock()
            .set_position_schedule(period, phase)
            .map_err(|error| VerbError::new_err(error.to_string()))
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

/// Resolves an identity that Python handed back, or raises.
///
/// Python cannot build an identity, so the value it gives is one the engine
/// gave it. That value can still be stale. The engine compares the
/// generation, and this function turns a refusal into the typed error for a
/// stale view.[^1]
///
/// # References
///
/// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
fn resolve(world: &CoreWorld, unit: u64) -> PyResult<Entity> {
    world
        .resolve_soldier(unit)
        .map_err(|error| ViewError::new_err(error.to_string()))
}

/// Resolves a settlement identity that Python handed back, or raises.
///
/// The engine compares the generation, so a settlement that was lost never
/// answers for the settlement founded next in its slot.[^1]
///
/// # References
///
/// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
fn resolve_site(world: &CoreWorld, site: u64) -> PyResult<Entity> {
    world
        .resolve_settlement(site)
        .map_err(|error| ViewError::new_err(error.to_string()))
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
