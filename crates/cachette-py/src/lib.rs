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

use cachette_core::census::{census, CensusError};
use cachette_core::founding::FoundingOutcome;
use cachette_core::{
    Axial, CommodityId, Entity, FactionId, Fix32, Holder, ResourceKind, World as CoreWorld,
    WorldConfig,
};
use cachette_view::{fill_frame, Camera, FrameSize, Lap, Metrics, Overlay, Surface};
use numpy::{PyArray1, PyReadwriteArray1, ToPyArray};
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
create_exception!(_core, FrameError, CachetteError, "A frame refused to fill.");
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
    presenter: std::sync::Mutex<Presenter>,
}

/// What the caller keeps between frames.
///
/// **The world holds none of this.** A field that existed for the viewer
/// would be the violation the boundary record names, so the engine keeps no
/// camera, no founding report and no timing.[^1] The binding is the caller
/// here, in the same way the demonstration binary is the caller on the other
/// front end, and a caller is allowed to keep what it owns.
///
/// # References
///
/// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
struct Presenter {
    /// What the step and the frame cost, for the panel to report.
    metrics: Metrics,
    /// The founding report the caller kept when it founded the run.
    outcomes: Vec<FoundingOutcome>,
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
            presenter: std::sync::Mutex::new(Presenter {
                metrics: Metrics::start(),
                outcomes: Vec::new(),
            }),
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
        let at = Lap::start();
        let events = python.detach(|| {
            let mut world = self.lock();
            match world.step(threads) {
                Ok(events) => Ok(events.len()),
                Err(error) => Err(StepError::new_err(error.to_string())),
            }
        })?;
        // The clock is read here and nowhere that decides anything. The
        // engine runs the same steps whatever this number says.
        self.presenter().metrics.step(at.elapsed());
        Ok(events)
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

    /// Returns what the founding would read if a group of this size looked
    /// for a place now.
    ///
    /// The survey draws a fixed number of candidate places and reads a fixed
    /// number of tiles around each one. Neither number follows the extent of
    /// the world, so this call costs the same in a large world as in a small
    /// one.[^1]
    ///
    /// The call writes nothing, and it founds nothing. It answers why a
    /// place scores what it scores: the counts that made the score are the
    /// columns, and the engine's own weighted sum is the score column.[^2]
    ///
    /// The columns are the candidates in the order the founding ranks them,
    /// best first, so row zero is the place a founding would take. A row
    /// whose `eligible` entry is zero is a place the founding refuses.
    ///
    /// # References
    ///
    /// [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D1. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    /// [^2]: ADR-0075, the founding choice reads a bounded sample of the world, decision D5. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the group holds nobody, or when the ordering
    /// of the candidates refuses to run.
    fn founding_survey<'py>(
        &self,
        python: Python<'py>,
        group: u32,
        faction: u16,
    ) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let survey = world
            .survey_founding(group, FactionId(faction))
            .map_err(|error| VerbError::new_err(error.to_string()))?;
        let ranked = survey.candidates();
        let columns = PyDict::new(python);
        let q: Vec<i32> = ranked.iter().map(|place| place.address().q).collect();
        let r: Vec<i32> = ranked.iter().map(|place| place.address().r).collect();
        let score: Vec<i64> = ranked.iter().map(|place| place.score().0).collect();
        let food: Vec<u32> = ranked
            .iter()
            .map(|place| place.provision().food.0)
            .collect();
        let wood: Vec<u32> = ranked
            .iter()
            .map(|place| place.provision().wood.0)
            .collect();
        let stone: Vec<u32> = ranked
            .iter()
            .map(|place| place.provision().stone.0)
            .collect();
        let open_ground: Vec<u32> = ranked
            .iter()
            .map(|place| place.provision().open_ground)
            .collect();
        let room: Vec<u32> = ranked.iter().map(|place| place.provision().room).collect();
        let water_edge: Vec<u32> = ranked
            .iter()
            .map(|place| place.provision().water_edge)
            .collect();
        let eligible: Vec<u8> = ranked
            .iter()
            .map(|place| u8::from(place.is_eligible()))
            .collect();
        let separated: Vec<u8> = ranked
            .iter()
            .map(|place| u8::from(place.is_separated()))
            .collect();
        columns.set_item("q", q.to_pyarray(python))?;
        columns.set_item("r", r.to_pyarray(python))?;
        columns.set_item("score", score.to_pyarray(python))?;
        columns.set_item("food", food.to_pyarray(python))?;
        columns.set_item("wood", wood.to_pyarray(python))?;
        columns.set_item("stone", stone.to_pyarray(python))?;
        columns.set_item("open_ground", open_ground.to_pyarray(python))?;
        columns.set_item("room", room.to_pyarray(python))?;
        columns.set_item("water_edge", water_edge.to_pyarray(python))?;
        columns.set_item("eligible", eligible.to_pyarray(python))?;
        columns.set_item("separated", separated.to_pyarray(python))?;
        // The survey counts these as it reads. They are measurements of the
        // run and not a second copy of the sample size.
        columns.set_item("drawn", survey.drawn())?;
        columns.set_item("considered", survey.considered())?;
        columns.set_item("tiles_read", survey.tiles_read())?;
        Ok(columns)
    }

    /// Founds a group the way the engine founds one, and reports what it
    /// chose.
    ///
    /// This is the whole loop in one call: the survey reads the ground, the
    /// founding takes the best place the sample offered, it seats the group
    /// over the disc around that place, and it sets the production rate of
    /// the site from the food the survey read.[^1] [^2] A caller that
    /// founded at an address of its own would get a site that earns nothing,
    /// because the rate comes from the survey.
    ///
    /// The report holds the identity of the site, the place, the number of
    /// people seated, and the counts the survey read at the chosen place.
    /// Every number is the engine's own. This binding recomputes no score.
    ///
    /// # References
    ///
    /// [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D5. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    /// [^2]: ADR-0062, production and upkeep are rates attached to a site, decision D2. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the group holds nobody, when the sample
    /// offered no place that admits the group, or when the seating refuses.
    fn found_group<'py>(
        &self,
        python: Python<'py>,
        group: u32,
        faction: u16,
    ) -> PyResult<Bound<'py, PyDict>> {
        let mut world = self.lock();
        let founding = world
            .found_run(group, FactionId(faction))
            .map_err(|error| VerbError::new_err(error.to_string()))?;
        let survey = founding.survey();
        let chosen = survey.chosen().ok_or_else(|| {
            VerbError::new_err("the founding reports no chosen place".to_string())
        })?;
        let provision = chosen.provision();
        let report = PyDict::new(python);
        report.set_item("site", founding.settlement().to_bits())?;
        report.set_item("q", founding.place().q)?;
        report.set_item("r", founding.place().r)?;
        report.set_item("faction", faction)?;
        report.set_item("seated", founding.people().len())?;
        report.set_item("score", chosen.score().0)?;
        report.set_item("food", provision.food.0)?;
        report.set_item("wood", provision.wood.0)?;
        report.set_item("stone", provision.stone.0)?;
        report.set_item("open_ground", provision.open_ground)?;
        report.set_item("room", provision.room)?;
        report.set_item("water_edge", provision.water_edge)?;
        report.set_item("drawn", survey.drawn())?;
        report.set_item("considered", survey.considered())?;
        report.set_item("tiles_read", survey.tiles_read())?;
        Ok(report)
    }

    /// Returns the level 1 summary of the cell that covers one tile.
    ///
    /// Level 0 is the only truth, and this level is derived from it. Every
    /// field is an exact integer total over the tiles of the cell, so a
    /// reader can add the level 0 tiles of the cell and get this number
    /// back.[^1] [^2]
    ///
    /// The call reads one cell. It starts no pass over the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
    /// [^2]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the address lies outside the world, or when
    /// the pyramid holds no cell for it.
    fn region_summary<'py>(
        &self,
        python: Python<'py>,
        q: i32,
        r: i32,
    ) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let summary = world
            .summary_covering(Axial::new(q, r))
            .ok_or_else(|| ViewError::new_err(format!("({q}, {r}) names no cell of this world")))?;
        let fields = PyDict::new(python);
        fields.set_item("tiles", summary.tiles())?;
        fields.set_item("open_tiles", summary.open_tiles())?;
        fields.set_item("units", summary.units())?;
        fields.set_item("held_tiles", summary.held_tiles())?;
        fields.set_item("value_total", summary.value_total().0)?;
        fields.set_item("height_total", summary.height_total().0)?;
        fields.set_item("food_total", summary.food_total().0)?;
        Ok(fields)
    }

    /// Returns what one site earns, holds and owes, and what it last
    /// rationed.
    ///
    /// The store is what the site holds now. The production is what it adds
    /// each time the rate pass runs, and the upkeep is what it owes. Every
    /// one is a Q16.16 value as its raw integer, because a float in
    /// simulated state does not add associatively.[^1]
    ///
    /// The ration row comes from the log of the draw that just ran. The
    /// engine keeps that log for one tick, so a site that served every
    /// cohort in full reports no shortfall.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^2]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D3. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no live settlement, or
    /// when the world holds no such commodity.
    #[pyo3(signature = (site, commodity = 0))]
    fn site_economy<'py>(
        &self,
        python: Python<'py>,
        site: u64,
        commodity: u16,
    ) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let entity = resolve_site(&world, site)?;
        let goods = CommodityId(commodity);
        let arena = world.settlements();
        let refuse = || ViewError::new_err(format!("{commodity} names no commodity of this world"));
        let store = arena
            .store(entity)
            .and_then(|held| held.quantity(goods))
            .ok_or_else(refuse)?;
        let production = world.production_rate(entity, goods).ok_or_else(refuse)?;
        let upkeep = world.upkeep_rate(entity, goods).ok_or_else(refuse)?;
        let address = arena
            .address(entity)
            .ok_or_else(|| ViewError::new_err(format!("the identity {site} names no live site")))?;
        let faction = arena
            .faction(entity)
            .ok_or_else(|| ViewError::new_err(format!("the identity {site} names no live site")))?;
        let report = PyDict::new(python);
        report.set_item("q", address.q)?;
        report.set_item("r", address.r)?;
        report.set_item("faction", faction.0)?;
        report.set_item("commodity", commodity)?;
        report.set_item("store", store.0)?;
        report.set_item("production", production.0)?;
        report.set_item("upkeep", upkeep.0)?;
        // The identity crosses whole. The comparison is against the value
        // the engine wrote into the log, and this code takes neither apart.
        let rationed = world
            .rationed_log()
            .iter()
            .find(|event| event.site == site && event.commodity == commodity);
        match rationed {
            Some(event) => {
                report.set_item("rationed", true)?;
                report.set_item("demanded", event.demanded.0)?;
                report.set_item("granted", event.granted.0)?;
            }
            None => {
                report.set_item("rationed", false)?;
                report.set_item("demanded", python.None())?;
                report.set_item("granted", python.None())?;
            }
        }
        Ok(report)
    }

    /// Returns why one unit chose what it chose.
    ///
    /// The engine recomputes the answer from the world as it stands. It
    /// stores no score, so the explanation costs nothing when nobody
    /// asks.[^1]
    ///
    /// Every score, field value, weight and floor is a Q16.16 value as its
    /// raw integer. The best entry is the option the scores select, or the
    /// no-intent value when every score is below the floor. The name entry
    /// is the engine's own name for that option, and it is `None` for a
    /// hold.
    ///
    /// The address entries name the tile the unit stands on. Pass them to
    /// `region_summary` to read the cell the unit scored.
    ///
    /// # References
    ///
    /// [^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D2. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no live unit, or when the
    /// engine would say nothing about it.
    fn explain_choice<'py>(&self, python: Python<'py>, unit: u64) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let entity = resolve(&world, unit)?;
        let answer = world.explain_choice(entity).ok_or_else(|| {
            ViewError::new_err(format!(
                "the engine explains nothing about the identity {unit}"
            ))
        })?;
        let tile = world.soldiers().tile(entity).ok_or_else(|| {
            ViewError::new_err(format!("the identity {unit} names no live soldier"))
        })?;
        let address = world.grid().address_of(tile).ok_or_else(|| {
            ViewError::new_err(format!("the identity {unit} stands on no address"))
        })?;
        let report = PyDict::new(python);
        report.set_item("tile", tile.0)?;
        report.set_item("q", address.q)?;
        report.set_item("r", address.r)?;
        report.set_item("cell", answer.cell)?;
        report.set_item("need", answer.need.0)?;
        report.set_item(
            "scores",
            answer
                .scores
                .iter()
                .map(|value| value.0)
                .collect::<Vec<i32>>(),
        )?;
        report.set_item(
            "fields",
            answer
                .fields
                .iter()
                .map(|value| value.0)
                .collect::<Vec<i32>>(),
        )?;
        report.set_item(
            "weights",
            answer
                .weights
                .iter()
                .map(|value| value.0)
                .collect::<Vec<i32>>(),
        )?;
        report.set_item("floor", answer.floor.0)?;
        report.set_item("best", answer.best)?;
        report.set_item("best_name", answer.best_name())?;
        report.set_item("intent", answer.intent)?;
        report.set_item("chooses_next_frame", answer.chooses_next_frame)?;
        Ok(report)
    }

    /// Returns what one tile holds.
    ///
    /// The stock of a tile is what the generator put there less what units
    /// took from it. The generated entry is the first, the taken entry is
    /// the second, and the stock entry is the difference the engine
    /// computes.[^1] Each is one entry for each kind of resource, in the
    /// order of the kind numbering.
    ///
    /// The holder entry names the faction that holds the ground, and it is
    /// `None` for ground that nobody holds.[^2]
    ///
    /// The capacity composes the ground with the finished upgrade, which is
    /// what admission reads. This call holds neither table.[^3]
    ///
    /// **This call reports no unit.** A count of the units on a tile comes
    /// from the derived bridge, which answers only after a step, and a
    /// reader of the ground should not be refused because the population
    /// moved. Ask `window_census` for the units.
    ///
    /// # References
    ///
    /// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    /// [^2]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    /// [^3]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the address lies outside the world.
    fn tile_report<'py>(
        &self,
        python: Python<'py>,
        q: i32,
        r: i32,
    ) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let address = Axial::new(q, r);
        let outside = || ViewError::new_err(format!("({q}, {r}) lies outside this world"));
        let kind = world.tile_kind(address).ok_or_else(outside)?;
        let capacity = world.tile_capacity(address).ok_or_else(outside)?;
        let mut stock: Vec<u32> = Vec::with_capacity(ResourceKind::ALL.len());
        let mut generated: Vec<u32> = Vec::with_capacity(ResourceKind::ALL.len());
        let mut taken: Vec<u32> = Vec::with_capacity(ResourceKind::ALL.len());
        for resource in ResourceKind::ALL {
            stock.push(world.tile_stock(address, resource).ok_or_else(outside)?.0);
            generated.push(
                world
                    .original_stock(address, resource)
                    .ok_or_else(outside)?
                    .0,
            );
            taken.push(world.taken_from(address, resource).ok_or_else(outside)?.0);
        }
        let report = PyDict::new(python);
        report.set_item("q", q)?;
        report.set_item("r", r)?;
        report.set_item("kind", kind.to_u8())?;
        report.set_item("passable", kind.is_passable())?;
        report.set_item("capacity", capacity)?;
        report.set_item("stock", stock)?;
        report.set_item("generated", generated)?;
        report.set_item("taken", taken)?;
        report.set_item("value", world.tile_value(address).ok_or_else(outside)?.0)?;
        match world.tile_holder(address).and_then(Holder::faction) {
            Some(faction) => report.set_item("holder", faction.0)?,
            None => report.set_item("holder", python.None())?,
        }
        Ok(report)
    }

    /// Returns what one window of the world holds.
    ///
    /// The window is the square of the given radius around the address,
    /// clipped to the world. The engine walks it and answers once. A caller
    /// that walked the addresses itself would be looping over the world from
    /// the control plane, which this boundary does not permit.[^1]
    ///
    /// **The cost follows the radius and never the world.** The engine
    /// refuses a radius above its ceiling.
    ///
    /// The unit counts come from the derived unit-to-tile bridge, which
    /// rebuilds at the barrier. A caller that changed the population and did
    /// not step is refused rather than answered from a stale bridge.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
    /// [^2]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the radius is above the ceiling. Raises
    /// `ViewError` when the window covers no address of the world, or when
    /// the bridge holds no answer.
    #[pyo3(signature = (q, r, radius = 8))]
    fn window_census<'py>(
        &self,
        python: Python<'py>,
        q: i32,
        r: i32,
        radius: u32,
    ) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let counted = census(&world, Axial::new(q, r), radius).map_err(|error| match error {
            CensusError::RadiusAboveCeiling { .. } => VerbError::new_err(error.to_string()),
            _ => ViewError::new_err(error.to_string()),
        })?;
        let report = PyDict::new(python);
        report.set_item("q", q)?;
        report.set_item("r", r)?;
        report.set_item("radius", radius)?;
        report.set_item("first_q", counted.first().q)?;
        report.set_item("first_r", counted.first().r)?;
        report.set_item("last_q", counted.last().q)?;
        report.set_item("last_r", counted.last().r)?;
        report.set_item("tiles", counted.tiles())?;
        report.set_item("by_kind", counted.by_kind().to_vec())?;
        report.set_item("open_tiles", counted.open_tiles())?;
        report.set_item("units", counted.units())?;
        report.set_item("crowd_worst", counted.crowd_worst())?;
        report.set_item("tiles_at_capacity", counted.tiles_at_capacity())?;
        match counted.crowded_most() {
            Some(address) => {
                report.set_item("crowded_q", address.q)?;
                report.set_item("crowded_r", address.r)?;
            }
            None => {
                report.set_item("crowded_q", python.None())?;
                report.set_item("crowded_r", python.None())?;
            }
        }
        Ok(report)
    }

    /// Founds one run for every faction and keeps the report.
    ///
    /// **This is a set-valued command, not a loop.** One call seats every
    /// faction the world has, because a founding must keep its distance from
    /// the foundings before it, and a caller that founded one faction at a
    /// time would have to carry that state across the boundary itself.
    ///
    /// The binding keeps the report, because the frame marks each founded
    /// place and the panel names each refusal. A founded place is history,
    /// and the engine holds no copy of it.
    ///
    /// Returns one dictionary for each faction. A seated faction gives its
    /// place, its people and what the ground it chose reaches. A refused
    /// faction gives the reason.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when no faction was seated. A run with no group is
    /// not a run.
    #[pyo3(signature = (group = 64))]
    fn found_run_for_every_faction<'py>(
        &self,
        python: Python<'py>,
        group: u32,
    ) -> PyResult<Vec<Bound<'py, PyDict>>> {
        let outcomes = {
            let mut world = self.lock();
            world.found_run_for_every_faction(group)
        };
        if !outcomes.iter().any(FoundingOutcome::is_seated) {
            return Err(VerbError::new_err(
                "no faction found a place, so the run has nothing in it".to_string(),
            ));
        }

        let mut reports = Vec::with_capacity(outcomes.len());
        for outcome in &outcomes {
            let report = PyDict::new(python);
            report.set_item("faction", outcome.faction().0)?;
            match outcome.result() {
                Ok(founding) => {
                    let place = founding.place();
                    report.set_item("seated", true)?;
                    report.set_item("q", place.q)?;
                    report.set_item("r", place.r)?;
                    report.set_item("people", founding.people().len())?;
                    report.set_item("considered", founding.survey().considered())?;
                    if let Some(chosen) = founding.survey().chosen() {
                        let reached = chosen.provision();
                        report.set_item("food", reached.food.0)?;
                        report.set_item("wood", reached.wood.0)?;
                        report.set_item("stone", reached.stone.0)?;
                        report.set_item("open_ground", reached.open_ground)?;
                        report.set_item("water_edge", reached.water_edge)?;
                        // The food the survey reached is the number of people
                        // the site can carry, because the production rate and
                        // the ration are both a sixteenth and the two cancel.
                        report.set_item("carries_its_group", reached.food.0 >= group)?;
                    }
                }
                Err(error) => {
                    report.set_item("seated", false)?;
                    report.set_item("refusal", error.to_string())?;
                }
            }
            reports.push(report);
        }
        self.presenter().outcomes = outcomes;
        Ok(reports)
    }

    /// Fills the caller's pixels with one frame of this world.
    ///
    /// **The caller owns the memory before this call and owns it
    /// afterwards.** The engine writes each pixel of one frame into it and
    /// returns. It allocates no frame, keeps no frame, and holds no reference
    /// to the memory after the call ends.[^1]
    ///
    /// **This is one command and it carries no entity.** It takes a world, a
    /// camera and somewhere to put the result, and it names no tile and no
    /// unit. A caller that walked tiles to draw them would cross the boundary
    /// once for each tile, and the crossing costs more than the drawing.[^2]
    ///
    /// The array must hold `width * height` unsigned 32-bit values and must be
    /// contiguous. Each value holds red, green and blue in its low three
    /// bytes.
    ///
    /// Set `reference` to show the layer that names the colours. Set `panel`
    /// to draw the whole panel instead of the cards, which is what a caller
    /// that writes a picture to a file wants.
    ///
    /// Returns what the drawing pass read, so a caller reports the numbers
    /// the picture was made from rather than starting a second pass to find
    /// them.[^3]
    ///
    /// # Errors
    ///
    /// Raises `FrameError` when the array does not match the width and the
    /// height, when a side is zero, when the array is not contiguous, or when
    /// the camera draws a tile smaller than one pixel. The last refusal names
    /// the bound: below one pixel for each tile a second tile falls on a pixel
    /// the first already holds, so the work is provably invisible and a
    /// caller could otherwise sweep the whole world for a picture of a few
    /// pixels.[^4]
    ///
    /// # References
    ///
    /// [^1]: ADR-0094, the caller owns the camera and the pixels, decision D2. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
    /// [^2]: ADR-0094, the caller owns the camera and the pixels, decision D1. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
    /// [^3]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    /// [^4]: ADR-0094, the caller owns the camera and the pixels, decision D6. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
    // The arguments are the interface a Python caller types by name, so
    // bundling them would hide the contract rather than simplify it.
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (camera, width, height, pixels, reference = false, panel = false))]
    fn draw<'py>(
        &self,
        python: Python<'py>,
        camera: &PyCamera,
        width: usize,
        height: usize,
        pixels: PyReadwriteArray1<'py, u32>,
        reference: bool,
        panel: bool,
    ) -> PyResult<Bound<'py, PyDict>> {
        let mut pixels = pixels;
        let buffer = pixels.as_slice_mut().map_err(|_| {
            FrameError::new_err(
                "the frame needs a contiguous array of unsigned 32-bit values".to_string(),
            )
        })?;

        let surface = Surface::new(width, height, buffer)
            .map_err(|error| FrameError::new_err(error.to_string()))?;

        let overlay = if panel {
            Overlay::Panel
        } else {
            Overlay::Glass { reference }
        };

        let at = Lap::start();
        let readout = {
            let world = self.lock();
            let presenter = self.presenter();
            fill_frame(
                &world,
                camera.inner,
                &presenter.metrics,
                &presenter.outcomes,
                overlay,
                surface,
            )
            .map_err(|error| FrameError::new_err(error.to_string()))?
        };
        self.presenter().metrics.draw(at.elapsed());

        let report = PyDict::new(python);
        report.set_item("tick", readout.tick())?;
        report.set_item("tiles_painted", readout.tiles_painted())?;
        report.set_item("soldiers_painted", readout.soldiers_painted())?;
        report.set_item("soldiers_live", readout.soldiers_live())?;
        report.set_item("sites_held", readout.sites_held())?;
        report.set_item("units_short", readout.units_short())?;
        report.set_item("tiles_at_capacity", readout.tiles_at_capacity())?;
        report.set_item("crowd_worst", readout.crowd_worst())?;
        let centre = readout.centre();
        report.set_item("centre", (centre.q, centre.r))?;
        let (columns, rows) = readout.extent_shown();
        report.set_item("extent_shown", (columns, rows))?;
        report.set_item("step_mean_micros", readout.step_mean())?;
        report.set_item("draw_mean_micros", readout.draw_mean())?;
        report.set_item("ticks_each_second", readout.rate())?;
        Ok(report)
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

/// A camera the control plane owns.
///
/// **The engine holds no camera.** It is given one for the length of a call
/// and keeps nothing of it afterwards, so a frame is a pure function of a
/// world and a camera. Two calls with the same world and the same camera give
/// the same picture, which is the property that makes a scripted flight, an
/// agent that steers, and a reproducible screenshot possible at all.[^1]
///
/// The state lives in Python. Python decides when to move and by how much.
/// The arithmetic lives here, once, because a pan share and a zoom step
/// written on both sides of the boundary would be one value in two places
/// with nothing failing when the copies disagreed.[^2]
///
/// Every verb takes the width and the height of the picture the camera aims
/// at. A camera verb reads no pixel, so a caller that has not drawn yet can
/// still steer.
///
/// # References
///
/// [^1]: ADR-0094, the caller owns the camera and the pixels, decision D3. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
/// [^2]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
// **The camera is a viewer value, and the record that bans the float types
// allows them for rendering.**[^1] The allowance is scoped to this type and
// its methods, and not to the crate, because everything else in this file
// carries simulation values, which are exact by construction.
//
// A float cannot travel back into the engine from here. The camera is passed
// by value into a drawing call, the drawing borrows the world shared, and
// every value the engine accepts is an exact integer.[^2]
//
// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D3. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
// [^2]: ADR-0094, the caller owns the camera and the pixels, decision D3. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
#[allow(clippy::disallowed_types)]
#[pyclass(name = "Camera", module = "cachette._core", skip_from_py_object)]
#[derive(Clone, Copy)]
pub struct PyCamera {
    inner: Camera,
}

#[allow(clippy::disallowed_types)]
#[pymethods]
impl PyCamera {
    /// Builds the camera the demonstration opens with.
    ///
    /// The tile size is a viewer choice and not a world property.
    #[new]
    #[pyo3(signature = (tile_size = None))]
    fn new(tile_size: Option<f32>) -> Self {
        Self {
            inner: tile_size.map_or_else(Camera::opening, Camera::at_tile_size),
        }
    }

    /// Returns a camera that fits the whole world into a picture of this size.
    #[staticmethod]
    fn fitting(world: &PyWorld, width: usize, height: usize) -> Self {
        Self {
            inner: Camera::fitting(&world.lock(), &FrameSize::new(width, height)),
        }
    }

    /// The width of one tile in pixels.
    #[getter]
    const fn tile_width(&self) -> f32 {
        self.inner.tile_width
    }

    /// Sets the width of one tile in pixels.
    ///
    /// **The setter does not hold the value to any bound.** The caller owns
    /// the camera, so the caller may build any camera it likes, and the frame
    /// verb refuses the ones it cannot draw and names the bound it refused
    /// against. A setter that held the scale quietly would return a picture
    /// that did not match the camera the caller asked for, and a caller could
    /// not tell that from a picture that did.[^1]
    ///
    /// The scroll and zoom verbs do hold the scale, because they are what a
    /// person drives and a person should not be able to press a key into a
    /// refusal.
    ///
    /// # References
    ///
    /// [^1]: ADR-0094, the caller owns the camera and the pixels, decision D6. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
    #[setter]
    const fn set_tile_width(&mut self, pixels: f32) {
        self.inner.tile_width = pixels;
    }

    /// The height of one tile in pixels.
    #[getter]
    const fn tile_height(&self) -> f32 {
        self.inner.tile_height
    }

    /// Sets the height of one tile in pixels, and holds it to no bound.
    #[setter]
    const fn set_tile_height(&mut self, pixels: f32) {
        self.inner.tile_height = pixels;
    }

    /// The pixel offset of the tile at the origin.
    #[getter]
    const fn origin_x(&self) -> f32 {
        self.inner.origin_x
    }

    /// Sets the pixel offset of the tile at the origin.
    #[setter]
    const fn set_origin_x(&mut self, pixels: f32) {
        self.inner.origin_x = pixels;
    }

    /// The pixel offset of the tile at the origin.
    #[getter]
    const fn origin_y(&self) -> f32 {
        self.inner.origin_y
    }

    /// Sets the pixel offset of the tile at the origin.
    #[setter]
    const fn set_origin_y(&mut self, pixels: f32) {
        self.inner.origin_y = pixels;
    }

    /// Moves the view by whole presses of a scroll key.
    ///
    /// **This is the call a person drives.** The step is a share of the
    /// picture, so one press moves the view by the same part of it at every
    /// zoom. Pass minus one, zero or one for each direction.
    fn nudge(&mut self, across: f32, down: f32, width: usize, height: usize) {
        self.inner = self
            .inner
            .nudged(across, down, &FrameSize::new(width, height));
    }

    /// Moves the view by a count of pixels.
    fn pan(&mut self, across: f32, down: f32) {
        self.inner = self.inner.panned(across, down);
    }

    /// Makes each tile larger by one press.
    fn zoom_in(&mut self, width: usize, height: usize) {
        self.inner = self.inner.zoomed_in(&FrameSize::new(width, height));
    }

    /// Makes each tile smaller by one press.
    fn zoom_out(&mut self, width: usize, height: usize) {
        self.inner = self.inner.zoomed_out(&FrameSize::new(width, height));
    }

    /// Puts a tile in the middle of the picture.
    fn look_at(&mut self, q: i32, r: i32, width: usize, height: usize) {
        self.inner = self
            .inner
            .looking_at(Axial::new(q, r), &FrameSize::new(width, height));
    }

    /// Holds the view inside the world.
    ///
    /// A camera that ran off the edge would show a picture of nothing, and a
    /// person could not tell that from an empty world.
    fn clamp(&mut self, world: &PyWorld, width: usize, height: usize) {
        self.inner = self
            .inner
            .clamped(&world.lock(), &FrameSize::new(width, height));
    }

    /// Returns the tile under a pixel, as a pair of axial coordinates.
    ///
    /// **This is how a click reaches a tile without a loop.** The control
    /// plane sends one pixel and gets one address, rather than walking the
    /// tiles to find which one was hit.
    fn tile_at(&self, x: f32, y: f32) -> (i32, i32) {
        let address = self.inner.tile_at(x, y);
        (address.q, address.r)
    }

    fn __repr__(&self) -> String {
        format!(
            "Camera(tile_width={}, tile_height={}, origin_x={}, origin_y={})",
            self.inner.tile_width, self.inner.tile_height, self.inner.origin_x, self.inner.origin_y
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

    /// Takes the caller's own values, recovering from a poisoned lock.
    fn presenter(&self) -> std::sync::MutexGuard<'_, Presenter> {
        self.presenter
            .lock()
            .unwrap_or_else(|error| error.into_inner())
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
    module.add_class::<PyCamera>()?;
    module.add_function(wrap_pyfunction!(version, module)?)?;
    module.add("CachetteError", module.py().get_type::<CachetteError>())?;
    module.add("StepError", module.py().get_type::<StepError>())?;
    module.add("FrameError", module.py().get_type::<FrameError>())?;
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
