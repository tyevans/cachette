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
use cachette_view::panel::Set as PanelSet;
use cachette_view::{fill_frame, Camera, FrameSize, Lap, Metrics, Overlay, Surface};
use numpy::{PyArray1, PyReadwriteArray1, ToPyArray};
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::PyTypeInfo;

// ADR-0046: one root exception type holds the whole hierarchy. The
// engine never raises a bare runtime error. The macro builds the types,
// because subclassing a Python class under the stable ABI needs a later
// interpreter version and the macro does not.
create_exception!(
    _core,
    CachetteError,
    PyException,
    "The base class of every error this module raises.

Catch this class to catch every refusal the engine makes. It is a subclass
of the Python built-in `Exception`.

Every other error class in this module is a subclass of this one. The engine
reports every refusal of its own through one of them, and never through a
bare `RuntimeError`.

The binding still refuses an argument before the call reaches the engine, and
it refuses with a built-in class. A wrong argument type raises `TypeError`.
An integer outside the range of the parameter raises `OverflowError`. A
sequence of the wrong shape raises `ValueError`. None of the three is a
subclass of this class."
);
create_exception!(
    _core,
    StepError,
    CachetteError,
    "A step refused to run.

`World.step` raises this class when the thread count is zero. A step needs
at least one thread."
);
create_exception!(
    _core,
    FrameError,
    CachetteError,
    "A frame refused to fill.

`World.draw` raises this class when the pixel array does not match the width
and the height, when a side is zero, when the array is not one contiguous
block, or when the camera draws a tile smaller than one pixel.

An array of the wrong element type raises `TypeError` instead, because the
interpreter refuses the argument before the engine reads it. Build the array
with the `numpy.uint32` element type."
);
create_exception!(
    _core,
    ConfigError,
    CachetteError,
    "The arguments do not describe a world.

The `World` constructor raises this class. A side of zero and a faction count
above 63 are the two cases a caller meets first. The doc comment of the
`World` class names every argument the constructor takes, its default and its
bound, so a caller checks a value before the call."
);
create_exception!(
    _core,
    SelectorError,
    CachetteError,
    "A selector was not valid.

**No call in this module raises this class today.** The module declares it
for the selector interface, and that interface is not written. A finding
records the gap.[^1]

# References

[^1]: Findings register, FND-326. `docs/FINDINGS.md`"
);
create_exception!(
    _core,
    VerbError,
    CachetteError,
    "A verb refused a command.

A verb is a call that changes the world. The class covers a refusal the
engine makes about the command itself: a number that names no kind, an
address the ground refuses, a target below zero, or a radius above the
ceiling. The message names the value that refused. The ceiling of a window
census is a radius of 64 tiles.

A verb that takes a set refuses the whole set. It writes nothing and leaves
nothing half made."
);
create_exception!(
    _core,
    ViewError,
    CachetteError,
    "A view was stale or out of scope.

The class covers two cases. The first is an identity that names no live
entity, which includes an identity the engine gave for an entity that has
since died. The second is an address or a window outside the world.

An identity is stale rather than wrong. The engine compares the generation,
so it refuses the dead identity and never answers for the next occupant of
the slot.[^1]

# References

[^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`"
);
create_exception!(
    _core,
    DeterminismError,
    CachetteError,
    "The engine detected a determinism defect.

**No call in this module raises this class today.** The module declares it
for a check that is not written. Two tests in the Rust workspace hold the
determinism guarantee instead, and neither reports through this class. A
finding records the gap.[^1]

# References

[^1]: Findings register, FND-326. `docs/FINDINGS.md`"
);
create_exception!(
    _core,
    EnginePanic,
    CachetteError,
    "A Rust panic reached the boundary.

**No call in this module raises this class today, and a panic does not
produce it.** The binding library catches a panic and raises its own
`pyo3_runtime.PanicException`, which is not a subclass of `CachetteError`.
A caller that wants to survive a panic catches `BaseException`. A finding
records the gap.[^1]

# References

[^1]: Findings register, FND-326. `docs/FINDINGS.md`"
);

/// A simulated world, and the whole of the engine a program drives.
///
/// Build a world, put units in it, give the units orders, then step it. Read
/// what the step did from the logs and the reports.
///
/// **The world is the unit of simulation, not a global.** A process may hold
/// many worlds. Two worlds share nothing.
///
/// **A step gives one answer at any thread count.** The same world, stepped
/// the same number of times with the same orders, reaches the same state hash
/// whether one thread or twelve threads ran it.[^1] That guarantee is what
/// makes a run repeatable. It says nothing about whether the run is correct.
///
/// **The methods that answer are the ones a program reads.** No method hands
/// out a view into the world. A method that copies says so, and a method that
/// answers about one thing takes one identity or one address.
///
/// # Build a world
///
/// ```text
/// World(width=64, height=64, seed=81985529216486895, faction_count=4)
/// ```
///
/// **The parameters of the constructor are here, and not under a separate
/// entry.** The binding library does not publish the prose of a
/// constructor, so this class doc comment is the one place that holds
/// it.[^2]
///
/// - `width`, an integer. The number of columns of tiles. It counts tiles,
///   and it must be at least one. The default is 64.
/// - `height`, an integer. The number of rows of tiles. It counts tiles, and
///   it must be at least one. The default is 64.
/// - `seed`, an integer. An unsigned 64-bit number that fixes the ground.
///   The default is 81985529216486895.
/// - `faction_count`, an integer. How many factions the world holds. The
///   ceiling is 63, and zero is a legal value that gives a world with no
///   faction. The default is 4.
///
/// The world is a rhombus of hexagonal tiles, so the extent is a width in
/// columns and a height in rows. The tile at column `q` and row `r` has the
/// axial address `(q, r)`.
///
/// The seed fixes the ground. Two worlds of one extent and one seed hold the
/// same terrain, the same heights and the same resources. Change the seed to
/// get another world.
///
/// A faction is a number from zero to one below the faction count.
///
/// **The new world holds no unit and no settlement.** Call
/// `found_run_for_every_faction` to seat a group for each faction, or
/// `spawn_soldiers` to put units at addresses you choose.
///
/// **The world reserves slots for the target unit population, whatever the
/// extent is.** The constructor exposes no capacity, so a small world
/// reserves as much unit storage as a large one. That reservation is what
/// `spawn_soldiers` means by a full arena.
///
/// The constructor raises `ConfigError` when the arguments do not describe a
/// world. A side of zero and a faction count above the ceiling are the two
/// cases a caller meets first.
///
/// # References
///
/// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
/// [^2]: Findings register, FND-325. `docs/FINDINGS.md`
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
    /// Builds a world of the given extent, from the given seed.
    ///
    /// **The prose for this call lives in the doc comment of the class.** The
    /// binding library does not copy the doc comment of a constructor onto
    /// the Python object, so prose written here reaches no reader of the
    /// published reference.[^1]
    ///
    /// # Errors
    ///
    /// Raises `ConfigError` when the arguments do not describe a world.
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-325. `docs/FINDINGS.md`
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

    /// The number of columns of tiles, as an integer.
    ///
    /// This is the width the constructor took. It never changes.
    #[getter]
    fn width(&self) -> u32 {
        self.lock().grid().width()
    }

    /// The number of rows of tiles, as an integer.
    ///
    /// This is the height the constructor took. It never changes.
    #[getter]
    fn height(&self) -> u32 {
        self.lock().grid().height()
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
    /// state give the same hash, and the hash does not depend on the thread
    /// count of any step that ran.[^1]
    ///
    /// **Compare hashes to check that a run repeated.** A hash that differs
    /// means the states differ. Equal hashes do not prove that either run is
    /// correct, because a defect that is itself repeatable gives one hash
    /// every time.
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
    /// It reads the stored structures of the world, so a caller runs it in a
    /// test rather than on every step.
    fn check_invariants(&self) -> bool {
        self.lock().check_invariants()
    }

    /// Runs one step of the simulation and returns the number of tile change
    /// events it emitted, as an integer.
    ///
    /// The thread count is how many threads the step may use. It has no
    /// default, so name it. **The result does not depend on it.** One thread
    /// and twelve threads give the same events. They give them in the same
    /// order, and they leave the same state hash.[^1]
    ///
    /// The step releases the global interpreter lock for its whole run, so
    /// another Python thread runs while the simulation runs. No Python code
    /// runs inside the step.[^2]
    ///
    /// The step replaces the logs of the step before it. Read
    /// `event_log_columns` and `gather_log_columns` after the call and before
    /// the next one.
    ///
    /// # Errors
    ///
    /// Raises `StepError` when the thread count is zero.
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    /// [^2]: ADR-0042, the interpreter is released for the whole step, decision D1. `docs/adrs/draft/adr-0042-the-interpreter-is-released-for-the-whole-step.md`
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

    /// Returns the tile change log of the last step as a `bytes` object.
    ///
    /// The bytes are the event records of the last step, one after another,
    /// in the order the engine wrote them. Each record holds the tick, the
    /// tile, the value, the holder, the change kind and its declared padding.
    ///
    /// **A caller that reads a field out of these bytes holds a copy of the
    /// record layout, and nothing fails when the layout changes.** Call
    /// `event_log_columns` instead. It gives the same events as arrays, by
    /// field name.
    ///
    /// This method exists for a caller that stores or ships the log without
    /// reading it.
    fn event_log_bytes<'py>(&self, python: Python<'py>) -> Bound<'py, pyo3::types::PyBytes> {
        pyo3::types::PyBytes::new(python, self.lock().event_log_bytes())
    }

    /// Copies the tile value column into a new NumPy array.
    ///
    /// Returns a one-dimensional array of `numpy.int32`, one entry for each
    /// tile, in row-major order. Entry `r * width + q` is the tile at the
    /// address `(q, r)`.
    ///
    /// **Each entry is a Q16.16 fixed-point value as its raw integer.**
    /// Divide by 65536 to read it as a quantity.[^1]
    ///
    /// This method copies, and it also generates. The world holds no array
    /// of tile values, so the call visits every tile. A caller that wants
    /// one tile calls `tile_report` instead.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^2]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    fn tile_values<'py>(&self, python: Python<'py>) -> Bound<'py, PyArray1<i32>> {
        let world = self.lock();
        let raw: Vec<i32> = world
            .copy_tile_values()
            .iter()
            .map(|value| value.0)
            .collect();
        raw.to_pyarray(python)
    }

    /// Returns the tile change log of the last step, as a `dict` of NumPy
    /// arrays.
    ///
    /// Every array has one entry for each event, and all five arrays are the
    /// same length. That length is `event_count`. A new world gives five
    /// empty arrays.
    ///
    /// - `tick`, `numpy.uint64`. The step at which the change happened.
    /// - `tile`, `numpy.uint32`. The tile that changed, as a row-major index.
    ///   Take `index % world.width` for the column and `index // world.width`
    ///   for the row.
    /// - `value`, `numpy.int32`. The tile value after the change. **This is a
    ///   Q16.16 fixed-point value as its raw integer. Divide by 65536.**[^2]
    /// - `holder`, `numpy.uint16`. The faction that holds the tile, as the
    ///   step left it. The value 65535 means that nobody holds it, and it
    ///   sits above the faction ceiling, so no faction collides with it.[^4]
    /// - `kind`, `numpy.uint8`. The kind of change. One means that the value
    ///   rose, and two means that it fell.
    ///
    /// The keys are the field names of the event. The caller reads a field
    /// by its name, so no caller holds a byte offset, a field width or a
    /// field order. Those live in the Rust source and nowhere else.[^1]
    ///
    /// This method copies each column. The log of one step is small next to
    /// the world.[^3]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the dictionary cannot be built.
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-060. `docs/DECISIONS.md`
    /// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^3]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    /// [^4]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
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

    /// Returns the gather log of the last step, as a `dict` of NumPy arrays.
    ///
    /// A gather event says that one unit took an amount of one resource from
    /// one tile. Every array has one entry for each event, and all five
    /// arrays are the same length. That length is `gather_count`.
    ///
    /// - `tick`, `numpy.uint64`. The step at which the unit took the amount.
    /// - `unit`, `numpy.uint64`. The identity of the unit that took it.
    /// - `tile`, `numpy.uint32`. The tile it took from, as a row-major index.
    /// - `amount`, `numpy.uint32`. How much it took. This is a whole number
    ///   of units of stock, and it is not fixed point.
    /// - `kind`, `numpy.uint8`. The resource kind. Food is zero, wood is one
    ///   and stone is two.
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
    /// # Errors
    ///
    /// Raises `ViewError` when the dictionary cannot be built.
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decisions D1 and D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    /// [^2]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
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

    /// The number of gather events the last step emitted, as an integer.
    ///
    /// The count covers the last step alone. Read the events themselves with
    /// `gather_log_columns`.
    #[getter]
    fn gather_count(&self) -> usize {
        self.lock().gather_log().len()
    }

    /// The number of soldiers alive in the world, as an integer.
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

    /// Returns the number of live units of each faction, by faction number.
    ///
    /// **This is one call and it names no unit.** The engine maintains the
    /// count where a unit is created and where a unit ends, so this reads a
    /// small array and starts no pass over the population.[^1] A caller that
    /// counted the units of a faction in Python would cross the boundary once
    /// for each unit, which the control plane rule forbids.[^2]
    ///
    /// The list holds one entry for each faction the world was built with. A
    /// faction whose last unit ends reads zero, and nothing else the bindings
    /// expose says so.
    ///
    /// # References
    ///
    /// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    /// [^2]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
    fn faction_population(&self) -> Vec<u32> {
        let world = self.lock();
        let counts = world.population_by_faction();
        let factions = world.config().faction_count as usize;
        counts.iter().copied().take(factions).collect()
    }

    /// Returns the name of every panel the viewer can draw.
    ///
    /// A caller passes one of these names to the drawing command. The list
    /// comes from the viewer's own registration, so a panel that joins the
    /// deck appears here with no edit to this file.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[staticmethod]
    fn panel_names() -> Vec<&'static str> {
        cachette_view::panel::registered()
            .iter()
            .map(|panel| panel.name())
            .collect()
    }

    /// Adds a soldier at each address and returns their identities.
    ///
    /// The addresses are a sequence of `(q, r)` pairs of integers. The
    /// faction is the number of the faction that owns the new soldiers.
    ///
    /// Returns a one-dimensional NumPy array of `numpy.uint64`, one identity
    /// for each address, in the order of the addresses. Keep the array and
    /// pass it to `order_gather` or `despawn_soldiers`. Take one entry as a
    /// Python integer for `soldier_tile` or `explain_choice`.
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
    /// # Errors
    ///
    /// Raises `VerbError` when the arena is full, when an address is outside
    /// the world, when the ground admits no unit, or when the world has no
    /// such faction. The error names the address that refused. Water is the
    /// ground that admits no unit.
    ///
    /// **A spawn reads no occupancy, so it may put a tile above its
    /// capacity.** An over-full tile is a state of the world and not a
    /// fault. Movement is what holds a tile to its capacity, and it only
    /// ever takes units off an over-full tile.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D1. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
    /// [^2]: Decisions register, DEC-063. `docs/DECISIONS.md`
    /// [^3]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity, decisions D1 and D2. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
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
    /// The units are a sequence of identities, or the NumPy array of
    /// `numpy.uint64` that `spawn_soldiers` returned. Returns `None`.
    ///
    /// **The set is all or nothing.** Every identity resolves before anything
    /// is removed, so one dead identity removes nothing and raises.[^1]
    ///
    /// A removed soldier leaves its slot to the next soldier. Its identity is
    /// then stale, and every call that takes an identity refuses it.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live soldier.
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
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
    /// The units are a sequence of identities, or the NumPy array of
    /// `numpy.uint64` that `spawn_soldiers` returned. Returns `None`.
    ///
    /// The kind is the resource kind: food is zero, wood is one and stone is
    /// two. It is the same number the gather log carries in its `kind`
    /// column.
    ///
    /// **The kind here is a resource kind and not a ground kind.** The two
    /// scales are separate, and both start at zero. Water, plain and forest
    /// are the ground kinds 0, 1 and 2, and each of those numbers also names
    /// a resource kind. This call therefore accepts a ground kind of 0, 1 or
    /// 2 and orders the resource of that number. It raises nothing, and the
    /// soldiers gather the wrong resource. The engine sees a number, and not
    /// the scale the caller meant, so no check reports this.[^1]
    ///
    /// The call gives the order. It takes nothing. Step the world to make the
    /// soldiers act, then read `gather_log_columns` for what they took.
    ///
    /// **The set is all or nothing.** Every identity resolves, and the kind is
    /// checked, before any order is given.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live soldier. Raises
    /// `VerbError` when the number names no resource kind, which means three
    /// and above.
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-120. `docs/DECISIONS.md`
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

    /// Returns the tile that one soldier stands on, as an integer.
    ///
    /// The unit is one identity, as a Python integer. Take an entry of the
    /// array that `spawn_soldiers` returned, or of the `unit` column of the
    /// gather log.
    ///
    /// The result is a row-major tile index. Take `index % world.width` for
    /// the column and `index // world.width` for the row.
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
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no live soldier, and when
    /// the value is not an identity the engine ever gave.
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    fn soldier_tile(&self, unit: u64) -> PyResult<u32> {
        let world = self.lock();
        let entity = resolve(&world, unit)?;
        world
            .soldiers()
            .tile(entity)
            .map(|tile| tile.0)
            .ok_or_else(|| ViewError::new_err(format!("the identity {unit} names no live soldier")))
    }

    /// The number of settlements standing in the world, as an integer.
    #[getter]
    fn settlement_count(&self) -> u32 {
        self.lock().settlements().len()
    }

    /// Founds a settlement at each address and returns their identities.
    ///
    /// The addresses are a sequence of `(q, r)` pairs of integers. The
    /// faction is the number of the faction that owns the new settlements.
    ///
    /// Returns a one-dimensional NumPy array of `numpy.uint64`, one identity
    /// for each address, in the order of the addresses. Pass an entry as a
    /// Python integer to `site_economy`, `site_positions`, `site_preference`
    /// or `prefer_at_sites`.
    ///
    /// **A settlement founded here earns nothing.** The production rate comes
    /// from the ground that a survey read, and this call runs no survey. Call
    /// `found_group` for a settlement that produces.[^1]
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
    ///
    /// # References
    ///
    /// [^1]: ADR-0062, production and upkeep are rates attached to a site, decision D2. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
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
    /// The sites are a sequence of settlement identities, or the NumPy array
    /// of `numpy.uint64` that `found_settlements` returned. Returns `None`.
    ///
    /// The kind is the kind of work: food is zero, wood is one and stone is
    /// two. It is the same numbering the gather log uses in its `kind`
    /// column.
    ///
    /// **The target is a Q16.16 value as its raw integer.** Multiply the
    /// share you want by 65536. A new site wants 65536, which is one, of
    /// every kind. The engine holds no floating point number in simulated
    /// state, because float addition is not associative.[^2]
    ///
    /// The engine acts on the new target at the next rebalance, and
    /// `set_position_schedule` says how often that runs.
    ///
    /// **The set is all or nothing.** Every identity resolves, and the target
    /// is checked, before anything is written.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live settlement. Raises
    /// `VerbError` when the number names no kind, or when the target is
    /// below zero.
    ///
    /// # References
    ///
    /// [^1]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
    /// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
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

    /// Returns the positions that one site holds, as a `dict` of NumPy
    /// arrays.
    ///
    /// A position is one seat of work at a settlement. The site is one
    /// settlement identity, as a Python integer.
    ///
    /// Every array has one entry for each position, and all three arrays are
    /// the same length.
    ///
    /// - `kind`, `numpy.uint8`. The kind of work: food is zero, wood is one
    ///   and stone is two.
    /// - `rank`, `numpy.uint8`. The rank of the position inside its kind,
    ///   counting from zero.
    /// - `holder`, `numpy.uint64`. The identity of the unit that holds the
    ///   position, and zero where a position holds nobody.
    ///
    /// The columns hold the positions of the site and nothing else. An entry
    /// of the storage that is no position does not appear.
    ///
    /// **A site holds no position until a rebalance runs**, so a site founded
    /// in this step reports three empty arrays. Step the world, and read
    /// `set_position_schedule` for how often the rebalance runs.
    ///
    /// The holder column carries the whole identity of the unit that holds
    /// each position, and zero where a position holds nobody. It is not a
    /// slot index.[^1]
    ///
    /// **This read stays singular while the write verb takes a set**, for
    /// the same reason the unit read does: a set form would have to answer
    /// for a dead identity with a value that stands for nothing.[^1]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no live settlement, or
    /// when the dictionary cannot be built.
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
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

    /// Returns what one site wants of each kind of work, as a NumPy array.
    ///
    /// The site is one settlement identity, as a Python integer.
    ///
    /// Returns a one-dimensional array of `numpy.int32`, one entry for each
    /// resource kind, in the order food, wood, stone.
    ///
    /// **Each entry is a Q16.16 value as its raw integer.** Divide by 65536.
    /// A new site wants 65536, which is one, of every kind. Write a target
    /// with `prefer_at_sites`.
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
    /// Returns `None`.
    ///
    /// The period is a count of ticks, and it must be at least one. A period
    /// of one rebalances on every step. The phase is the offset inside the
    /// period, so a period of four and a phase of one rebalance on the ticks
    /// one, five and nine. A phase at or above the period wraps into it.
    ///
    /// The world starts with a schedule already set. This call replaces it.
    ///
    /// A rebalance is what turns the targets that `prefer_at_sites` wrote
    /// into the positions that `site_positions` reports.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the period is zero, or above the range that
    /// the scaling multiply takes. The message names the limit it applied.
    fn set_position_schedule(&self, period: u32, phase: u32) -> PyResult<()> {
        self.lock()
            .set_position_schedule(period, phase)
            .map_err(|error| VerbError::new_err(error.to_string()))
    }

    /// Returns what the founding would read if a group of this size looked
    /// for a place now, as a `dict`.
    ///
    /// The group is the number of people that would settle. The faction is
    /// the number of the faction that would settle them, and it chooses the
    /// sample, so two factions read two samples.[^3]
    ///
    /// Eleven entries are NumPy arrays with one entry for each candidate
    /// place, and three are plain integers.
    ///
    /// - `q` and `r`, `numpy.int32`. The address of the candidate place.
    /// - `score`, `numpy.int64`. The weighted sum that ranks the place.
    ///   **This is a Q16.16 value as its raw integer. Divide by 65536.**
    /// - `food`, `wood` and `stone`, `numpy.uint32`. How much of each
    ///   resource the disc around the place holds. These are whole units of
    ///   stock, and they are not fixed point.
    /// - `open_ground`, `numpy.uint32`. How many tiles of the disc admit a
    ///   unit.
    /// - `room`, `numpy.uint32`. How many units the open tiles of the disc
    ///   hold together.
    /// - `water_edge`, `numpy.uint32`. How many of the six neighbours of the
    ///   centre hold open water.
    /// - `eligible`, `numpy.uint8`. One when the founding would accept the
    ///   place, and zero when it refuses it.
    /// - `separated`, `numpy.uint8`. One when the place keeps its distance
    ///   from every place a founding before it took.
    /// - `drawn`, a plain integer. How many candidate places the survey drew.
    /// - `considered`, a plain integer. How many distinct places it read. Two
    ///   draws may name one tile, and the survey reads such a tile once.
    /// - `tiles_read`, a plain integer. How many tiles it read in all.
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
    /// # Errors
    ///
    /// Raises `VerbError` when the group holds nobody, or when the ordering
    /// of the candidates refuses to run.
    ///
    /// # References
    ///
    /// [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D1. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    /// [^2]: ADR-0075, the founding choice reads a bounded sample of the world, decision D5. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    /// [^3]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D3. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
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
    /// chose, as a `dict`.
    ///
    /// The group is the number of people to settle. The faction is the number
    /// of the faction that settles them.
    ///
    /// Every entry is a plain integer.
    ///
    /// - `site`. The identity of the settlement the founding made. Pass it to
    ///   `site_economy`, `site_positions` or `prefer_at_sites`.
    /// - `q` and `r`. The address the founding took.
    /// - `faction`. The faction the call took.
    /// - `seated`. **How many people the founding seated**, and not whether
    ///   it seated any. The report of `found_run_for_every_faction` uses the
    ///   same key for a `bool`.
    /// - `score`. The weighted sum of the chosen place. **This is a Q16.16
    ///   value as its raw integer. Divide by 65536.**
    /// - `food`, `wood`, `stone`, `open_ground`, `room` and `water_edge`.
    ///   What the survey read at the chosen place. Each is a whole count, and
    ///   `founding_survey` describes each one.
    /// - `drawn`, `considered` and `tiles_read`. What the survey did.
    ///
    /// This is the whole loop in one call: the survey reads the ground, the
    /// founding takes the best place the sample offered, it seats the group
    /// over the disc around that place, and it sets the production rate of
    /// the site from the food the survey read.[^1] [^2] A caller that
    /// founded at an address of its own would get a site that earns nothing,
    /// because the rate comes from the survey.
    ///
    /// Every number in the report is the engine's own. This binding
    /// recomputes no score.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the group holds nobody, when the world has no
    /// such faction, when the sample offered no place that admits the group,
    /// or when the seating refuses.
    ///
    /// # References
    ///
    /// [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D5. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    /// [^2]: ADR-0062, production and upkeep are rates attached to a site, decision D2. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
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

    /// Returns the summary of the cell that covers one tile, as a `dict`.
    ///
    /// A cell is a square block of tiles. The world summarises each block, so
    /// a reader asks about a region without reading its tiles. Give the
    /// address of any tile, and the call answers about the cell that covers
    /// it.
    ///
    /// Every entry is a plain integer.
    ///
    /// - `tiles`. How many tiles the cell covers.
    /// - `open_tiles`. How many of them admit a unit.
    /// - `units`. How many units stand on them.
    /// - `held_tiles`. How many of them a faction holds. The entry does not
    ///   say which faction.
    /// - `value_total`. The sum of the tile values. **This is a Q16.16 value
    ///   as its raw integer. Divide by 65536.**
    /// - `height_total`. The sum of the tile heights. **This is also a Q16.16
    ///   value as its raw integer. Divide by 65536.**
    /// - `food_total`. The food the tiles still hold. **This one is a whole
    ///   count of units of stock. Do not divide it.**
    ///
    /// **Two of the three totals carry the fixed-point scale and the third
    /// does not.** A reader that divides all three reports a food total
    /// 65536 times too small.
    ///
    /// Level 0 is the only truth, and this level is derived from it. Every
    /// entry is an exact integer total over the tiles of the cell, so a
    /// reader can add the tiles of the cell and get this number back.[^1] [^2]
    ///
    /// The call reads one cell. It starts no pass over the world.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the address lies outside the world, or when
    /// the pyramid holds no cell for it.
    ///
    /// # References
    ///
    /// [^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
    /// [^2]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
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

    /// Returns what one site earns, holds and owes, as a `dict`.
    ///
    /// The site is one settlement identity, as a Python integer. The
    /// commodity is the number of the commodity to report on, and it
    /// defaults to zero.
    ///
    /// **A commodity is not a resource kind.** The two scales are separate.
    /// The world holds one commodity today, and its number is zero. Every
    /// other number raises `ViewError`, so the resource kinds one and two
    /// name no commodity here.
    ///
    /// - `q` and `r`, integers. The address of the site.
    /// - `faction`, an integer. The faction that owns the site.
    /// - `commodity`, an integer. The commodity the call took.
    /// - `store`, an integer. What the site holds now. **A Q16.16 value as
    ///   its raw integer. Divide by 65536.**
    /// - `production`, an integer. What it adds each time the rate pass runs.
    ///   **Also Q16.16.**
    /// - `upkeep`, an integer. What it owes each time. **Also Q16.16.**
    /// - `rationed`, a `bool`. Whether the last draw could not serve every
    ///   cohort in full.
    /// - `demanded` and `granted`, integers or `None`. What the cohorts asked
    ///   for and what the store gave. **Both are Q16.16 when they are
    ///   integers.** Both are `None` when `rationed` is `False`.
    ///
    /// The engine holds no floating point number in simulated state, because
    /// float addition is not associative.[^1]
    ///
    /// **A settlement founded by `found_settlements` produces nothing**, and
    /// reports a production of zero. The rate comes from the survey, and only
    /// `found_group` and `found_run_for_every_faction` run one.[^3]
    ///
    /// The ration row comes from the log of the draw that just ran. The
    /// engine keeps that log for one tick, so a site that served every
    /// cohort in full reports no shortfall.[^2]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no live settlement, or
    /// when the world holds no such commodity.
    ///
    /// # References
    ///
    /// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^2]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D3. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
    /// [^3]: ADR-0062, production and upkeep are rates attached to a site, decision D2. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
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

    /// Returns why one unit chose what it chose, as a `dict`.
    ///
    /// The unit is one soldier identity, as a Python integer.
    ///
    /// - `tile`, an integer. The row-major index of the tile the unit stands
    ///   on.
    /// - `q` and `r`, integers. The address of that tile. Pass them to
    ///   `region_summary` to read the cell the unit scored.
    /// - `cell`, an integer. The index of that cell.
    /// - `need`, an integer. What the unit still needs. **A Q16.16 value as
    ///   its raw integer. Divide by 65536.**
    /// - `scores`, `fields` and `weights`, lists of integers. One entry for
    ///   each option, in option order. The field is what the option read, the
    ///   weight is what the option carried, and the score is what the engine
    ///   made of the two. **All three hold Q16.16 values as raw integers.**
    /// - `floor`, an integer. The score an option had to reach. **Also
    ///   Q16.16.**
    /// - `best`, an integer. The option the scores select, or the no-intent
    ///   value when every score is below the floor.
    /// - `best_name`, a `str` or `None`. The engine's own name for that
    ///   option. It is `None` for a hold.
    /// - `intent`, an integer. The intent the unit carries now.
    /// - `chooses_next_frame`, a `bool`. Whether the unit reads the world
    ///   again on the next step.
    ///
    /// The engine recomputes the answer from the world as it stands. It
    /// stores no score, so the explanation costs nothing when nobody
    /// asks.[^1]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no live unit, or when the
    /// engine would say nothing about it.
    ///
    /// # References
    ///
    /// [^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D2. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
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

    /// Returns what one tile holds, as a `dict`.
    ///
    /// The address is the column `q` and the row `r`, both counting from
    /// zero.
    ///
    /// - `q` and `r`, integers. The address the call took.
    /// - `kind`, an integer. The ground: water is zero, plain is one, forest
    ///   is two, hill is three and mountain is four.
    /// - `passable`, a `bool`. Whether the ground admits a unit.
    /// - `capacity`, an integer. How many units the tile holds.
    /// - `stock`, `generated` and `taken`, lists of integers. One entry for
    ///   each resource kind, in the order food, wood, stone. Each is a whole
    ///   count of units of stock, and none of them is fixed point.
    /// - `value`, an integer. The tile value. **A Q16.16 value as its raw
    ///   integer. Divide by 65536.**
    /// - `holder`, an integer or `None`. The faction that holds the ground,
    ///   and `None` for ground that nobody holds.[^2]
    ///
    /// The stock of a tile is what the generator put there less what units
    /// took from it. The generated entry is the first, the taken entry is
    /// the second, and the stock entry is the difference the engine
    /// computes.[^1]
    ///
    /// The capacity composes the ground with the finished upgrade, which is
    /// what admission reads. This call holds neither table.[^3]
    ///
    /// **This call reports no unit.** A count of the units on a tile comes
    /// from the derived bridge, which answers only after a step, and a
    /// reader of the ground should not be refused because the population
    /// moved. Ask `window_census` for the units.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    /// [^2]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    /// [^3]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
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

    /// Returns what one window of the world holds, as a `dict`.
    ///
    /// The window is the square of the given radius around the address,
    /// clipped to the world. **The radius counts tiles.** A radius of zero
    /// reads one tile. The default radius is 8, and the ceiling is 64.
    ///
    /// - `q`, `r` and `radius`, integers. The arguments the call took.
    /// - `first_q`, `first_r`, `last_q` and `last_r`, integers. The corners
    ///   of the window after the engine clipped it to the world.
    /// - `tiles`, an integer. How many tiles the window covers.
    /// - `by_kind`, a list of integers. One count for each ground kind, in
    ///   the order water, plain, forest, hill, mountain.
    /// - `open_tiles`, an integer. How many tiles admit a unit.
    /// - `units`, an integer. How many units stand in the window.
    /// - `crowd_worst`, an integer. The largest number of units on any one
    ///   tile of the window.
    /// - `tiles_at_capacity`, an integer. How many tiles hold as many units
    ///   as their capacity.
    /// - `crowded_q` and `crowded_r`, integers or `None`. The address that
    ///   holds the most units. Both are `None` when the window holds none.
    ///
    /// The engine walks the window and answers once. A caller
    /// that walked the addresses itself would be looping over the world from
    /// the control plane, which this boundary does not permit.[^1]
    ///
    /// **The cost follows the radius and never the world.** The engine
    /// refuses a radius above 64.
    ///
    /// The unit counts come from the derived unit-to-tile bridge, which
    /// rebuilds at the barrier. A caller that changed the population and did
    /// not step is refused rather than answered from a stale bridge.[^2]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the radius is above 64. The message names the
    /// ceiling. Raises `ViewError` when the window covers no
    /// address of the world, or when the bridge holds no answer.
    ///
    /// # References
    ///
    /// [^1]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
    /// [^2]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
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
    /// The group is how many people to seat for each faction.
    ///
    /// Returns a `list` of one `dict` for each faction of the world, in
    /// faction order. Every entry is a plain integer, a `bool` or a `str`.
    ///
    /// Every report holds these two.
    ///
    /// - `faction`, an integer. The faction the report is about.
    /// - `seated`, a `bool`. **Whether the faction got a place**, and not how
    ///   many people it seated. The report of `found_group` uses the same key
    ///   for a count.
    ///
    /// A report whose `seated` entry is `True` holds these as well.
    ///
    /// - `q` and `r`, integers. The address the founding took.
    /// - `people`, an integer. How many people it seated.
    /// - `considered`, an integer. How many distinct places the survey read.
    /// - `food`, `wood`, `stone`, `open_ground` and `water_edge`, integers.
    ///   What the survey read at the chosen place. Each is a whole count.
    /// - `carries_its_group`, a `bool`. Whether the food the survey read
    ///   holds the whole group.
    ///
    /// A report whose `seated` entry is `False` holds `refusal` instead, a
    /// `str` that says why the faction got no place.
    ///
    /// **A report holds no site identity.** Call `found_group` for a founding
    /// that hands one back.
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
    /// The camera says what part of the world the picture shows. The width
    /// and the height are the size of the picture in pixels.
    ///
    /// The pixels are a one-dimensional NumPy array of `numpy.uint32`. It
    /// must hold `width * height` entries and must be one contiguous block.
    /// Each entry holds red, green and blue in its low three bytes. Build one
    /// with `numpy.zeros(width * height, dtype=numpy.uint32)`, and reshape it
    /// to `(height, width)` after the call to show it.
    ///
    /// Set `reference` to show the layer that names the colours. Set `panel`
    /// to draw the whole panel instead of the cards, which is what a caller
    /// that writes a picture to a file wants.
    ///
    /// Returns a `dict` of what the drawing pass read.[^3] A caller reports
    /// the numbers the picture was made from, and starts no second pass to
    /// find them.
    ///
    /// Most entries are plain integers. Five are not. `promoted_deeds` and
    /// `newest_character` may be `None`. `centre` and `extent_shown` are
    /// pairs of integers. `carried_by_kind` is a list of one count for each
    /// resource kind. The three trailing entries are floating point numbers.
    ///
    /// **The three floating point entries measure this machine and not the
    /// simulation.** `step_mean_micros` and `draw_mean_micros` are mean
    /// durations in microseconds, and `ticks_each_second` is a rate. Nothing
    /// in the engine reads them, and no two runs need agree on them.
    ///
    /// **`rationed_short_accum` is a Q16.16 value as its raw integer.**
    /// Divide by 65536. Its name says so, because a caller that read it as a
    /// count of goods would report a quantity 65536 times too large. Every
    /// other integer entry is a whole count.
    ///
    /// `panel_height` is the picture height that holds the whole panel. A
    /// caller that writes the panel to a file draws once, reads this entry,
    /// then draws again at that height.
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
    /// Raises `TypeError` when the array does not hold `numpy.uint32`
    /// entries. The interpreter refuses the argument before the engine reads
    /// it, so that refusal is not a `CachetteError`.
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
    #[pyo3(signature = (camera, width, height, pixels, reference = false, panel = false, panels = None, pointer = None))]
    fn draw<'py>(
        &self,
        python: Python<'py>,
        camera: &PyCamera,
        width: usize,
        height: usize,
        pixels: PyReadwriteArray1<'py, u32>,
        reference: bool,
        panel: bool,
        panels: Option<Vec<String>>,
        pointer: Option<(i32, i32)>,
    ) -> PyResult<Bound<'py, PyDict>> {
        let mut pixels = pixels;
        let buffer = pixels.as_slice_mut().map_err(|_| {
            FrameError::new_err(
                "the frame needs a contiguous array of unsigned 32-bit values".to_string(),
            )
        })?;

        let surface = Surface::new(width, height, buffer)
            .map_err(|error| FrameError::new_err(error.to_string()))?;

        // The named panels win over the whole panel, because a caller that
        // named a deck asked for the deck. A name that no panel carries is
        // refused, and the message names the panels that exist: a frame with
        // nothing on it looks the same as a frame the caller mistyped.
        let mut deck = PanelSet::EMPTY;
        for name in panels.unwrap_or_default() {
            deck = deck.with(&name).ok_or_else(|| {
                let known: Vec<&str> = cachette_view::panel::registered()
                    .iter()
                    .map(|panel| panel.name())
                    .collect();
                FrameError::new_err(format!(
                    "no panel is called {name:?}; the panels are {}",
                    known.join(", ")
                ))
            })?;
        }

        let overlay = if !deck.is_empty() {
            Overlay::Deck {
                reference,
                panels: deck,
                pointer: pointer.map(|(q, r)| Axial { q, r }),
            }
        } else if panel {
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
        // A position is a seat at a site. Both numbers come from the bounded
        // walk over the sites, so neither follows the world.
        report.set_item("seats", readout.seats())?;
        report.set_item("seats_taken", readout.seats_taken())?;
        // The character tier, walked once. A caller may walk this tier and
        // no other, because it holds a bounded population.
        report.set_item("characters", readout.characters())?;
        // How many units the last step promoted, and the deeds that earned
        // the first of them. Both come from the log of that step, so a caller
        // sees the promotion on the frame it happened rather than watching a
        // number go up.
        report.set_item("promoted_now", readout.promoted_now())?;
        report.set_item("promoted_deeds", readout.promoted_deeds())?;
        report.set_item(
            "newest_character",
            readout
                .newest_character()
                .map(|(faction, birth)| (faction.0, birth)),
        )?;
        // The height a picture needs to hold the whole panel. A caller that
        // writes the panel to a file resizes to this rather than guessing a
        // constant, because the panel grows with the faction count, with the
        // number of foundings, and with every section a count switches on.
        report.set_item("panel_height", readout.height_for_whole_panel())?;
        report.set_item("units_short", readout.units_short())?;
        // What the drawn units are hauling and where they live. Both are
        // counts of the window, taken on the loop that painted them, so a
        // caller reports them without starting a pass of its own.
        report.set_item("units_carrying", readout.units_carrying())?;
        report.set_item("carried_by_kind", *readout.carried_by_kind())?;
        report.set_item("units_housed", readout.units_housed())?;
        // The store of a site rations when it cannot serve its cohorts.
        // This is a count of the world, because the engine holds the log of
        // the step that just ran.
        report.set_item("sites_rationed", readout.rationings())?;
        // The shortfall is in accumulator units, which are fixed point at a
        // scale of 65536. The key names the unit, because a caller that read
        // this as a count of goods would report a quantity sixty-five
        // thousand times the real one.
        report.set_item("rationed_short_accum", readout.rationed_short())?;
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

    /// Sends every soldier the identities name to a set of tiles.
    ///
    /// The units are a sequence of identities, or the NumPy array of
    /// `numpy.uint64` that `spawn_soldiers` returned. The seeds are a
    /// sequence of `(q, r)` pairs of integers, and they are the places the
    /// caller wants the units at. The destination is the number of the
    /// destination plane that carries the order. Returns `None`.
    ///
    /// **One call names a whole set and the engine builds one field.** The
    /// engine takes the level 1 cell of each seed, seeds a plane at every one
    /// of them at once, and spreads a reach outward. Every unit the call
    /// names then reads one entry of that plane on each step and takes one
    /// step. The cost of the field follows the cell count and not the number
    /// of units, so sending a million units costs what sending one costs.[^1]
    ///
    /// **No unit searches for a route.** A unit reads the entry of its own
    /// cell. It reads no neighbouring cell and it computes nothing from its
    /// own address toward a seed. That is the rule the engine is built on,
    /// and this call does not bend it.[^2]
    ///
    /// **A cell steers a whole block, so two units in one cell take one
    /// direction.** A caller cannot send half a cell one way and half the
    /// other.[^2]
    ///
    /// **A unit that cannot reach the seeds does not freeze.** A unit whose
    /// cell holds no direction takes a keyed draw instead, and the draw is
    /// keyed on the frame, so it takes a different direction on the next
    /// frame. The same holds for a unit that arrived, and for a unit whose
    /// ground refuses the direction the field gave it.[^3]
    ///
    /// **The call sends a set toward a place. It does not promise that the set
    /// arrives.** A cell steers a block of tiles, and the water in front of one
    /// unit of that block is not a fact the block carries. A unit behind such a
    /// barrier walks to it and then wanders beside it. It is not frozen, and it
    /// does not get past.[^4]
    ///
    /// The order holds until the caller stops it with `stop_sending`. A unit
    /// that arrives keeps the order and walks about inside the block it
    /// arrived in. Read `faction_units` for where the set is now.
    ///
    /// A caller that names a destination again replaces the seed set of that
    /// destination, and every unit already sent to it walks to the new one.
    /// Read `destination_count` for how many the world holds, and set it with
    /// `set_destination_count`.
    ///
    /// **The set is all or nothing.** Every identity resolves, every address
    /// is checked, and the destination is checked, before anything changes.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no destination plane of this
    /// world, and when a seed address is outside the world. Raises `ViewError`
    /// when an identity names no live soldier.
    ///
    /// # References
    ///
    /// [^1]: ADR-0095, a behavioural strategy arrives as a field over cells, never as a search from a unit, decision D3. `docs/adrs/draft/adr-0095-a-behavioural-strategy-arrives-as-a-field-over-cells.md`
    /// [^2]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
    /// [^3]: ADR-0125, the control plane names the seed set of a destination field, decision D4. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    /// [^4]: Findings register, FND-411. `docs/FINDINGS.md`
    #[pyo3(signature = (units, seeds, destination = 0))]
    fn send_units_to(
        &self,
        units: Vec<u64>,
        seeds: Vec<(i32, i32)>,
        destination: u16,
    ) -> PyResult<()> {
        let mut world = self.lock();
        let mut resolved = Vec::with_capacity(units.len());
        for unit in &units {
            resolved.push(resolve(&world, *unit)?);
        }
        let addresses: Vec<Axial> = seeds.iter().map(|(q, r)| Axial::new(*q, *r)).collect();
        world
            .send_units_to(&resolved, &addresses, destination)
            .map_err(|error| VerbError::new_err(error.to_string()))
    }

    /// The number of destination planes the world holds, as an integer.
    ///
    /// A destination plane carries one order. The caller names the plane when
    /// it sends a set of units somewhere, and the numbers run from zero to one
    /// below this.
    #[getter]
    fn destination_count(&self) -> u16 {
        self.lock().destination_count()
    }

    /// Sets the number of destination planes the world holds.
    ///
    /// The count says how many places the control plane may send units to at
    /// one time, before it re-aims a plane it already used. **The caller names
    /// the plane, and the engine allocates none.**[^1]
    ///
    /// The call clears the seed set of every plane, so no order steers
    /// anything until the caller sends a set again. A unit that was sent to a
    /// plane the world no longer holds reads no direction, and it takes a
    /// keyed draw rather than standing still.[^2]
    ///
    /// Set this before the run, in the way the other world parameters are set.
    /// Returns `None`.
    ///
    /// # References
    ///
    /// [^1]: ADR-0125, the control plane names the seed set of a destination field, decision D3. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    /// [^2]: ADR-0125, the control plane names the seed set of a destination field, decision D4. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    fn set_destination_count(&self, count: u16) {
        self.lock().set_destination_count(count);
    }

    /// Stops sending every soldier the identities name.
    ///
    /// The units are a sequence of identities, or the NumPy array of
    /// `numpy.uint64` that `spawn_soldiers` returned. Returns `None`.
    ///
    /// Each unit goes back to the option that it chose for itself.
    ///
    /// **The set is all or nothing.** Every identity resolves before anything
    /// changes.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live soldier.
    fn stop_sending(&self, units: Vec<u64>) -> PyResult<()> {
        let mut world = self.lock();
        let mut resolved = Vec::with_capacity(units.len());
        for unit in &units {
            resolved.push(resolve(&world, *unit)?);
        }
        world
            .stop_sending(&resolved)
            .map_err(|error| VerbError::new_err(error.to_string()))
    }

    /// Returns the live soldiers of one faction, as columns.
    ///
    /// The faction is the number of the faction. The result is a `dict` of
    /// one-dimensional NumPy arrays, and every array holds one entry for each
    /// live soldier of that faction. The keys are:
    ///
    /// - `unit`, `numpy.uint64`. The identity of the soldier. Pass the whole
    ///   array to `send_units_to`, `order_gather` or `despawn_soldiers`.
    /// - `tile`, `numpy.uint32`. The tile it stands on, as a row-major index.
    ///   Take `index % world.width` for the column and `index // world.width`
    ///   for the row.
    ///
    /// **This is one crossing, and it replaces a loop.** A caller that read
    /// each unit through `soldier_tile` paid one crossing for each unit, and
    /// the control plane never loops over the population.[^1] [^2]
    ///
    /// **Every entry names a live soldier, so no entry stands for nothing.**
    /// The engine builds the set at the moment of the call, and it takes no
    /// identity from the caller, so nothing here can be stale and the result
    /// needs no validity mask. The singular read takes an identity and must
    /// refuse a dead one, and it still does.[^3]
    ///
    /// The order is the slot order of the arena. It is the same on every run
    /// and at every thread count, and it is never a thread completion
    /// order.[^4] It is not the spawn order: a slot returns to the arena when
    /// a soldier dies, and the next soldier takes it.
    ///
    /// A faction with nobody in it gives two empty arrays, which is an answer
    /// and not an error. A number that names no faction of this world does
    /// the same, because no soldier holds it.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the dictionary cannot be built.
    ///
    /// # References
    ///
    /// [^1]: Project orientation, the design principles. `CLAUDE.md`
    /// [^2]: Research report 20, what the Python interface should be, section 2.3. `docs/research/reports/20-the-python-interface.md`
    /// [^3]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    /// [^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn faction_units<'py>(
        &self,
        python: Python<'py>,
        faction: u16,
    ) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let soldiers = world.soldiers();
        let mut unit: Vec<u64> = Vec::new();
        let mut tile: Vec<u32> = Vec::new();
        for entity in soldiers.iter_faction(FactionId(faction)) {
            unit.push(entity.to_bits());
            tile.push(
                soldiers
                    .tile(entity)
                    .expect("a live identity from the walk names a tile")
                    .0,
            );
        }
        let columns = PyDict::new(python);
        columns.set_item("unit", unit.to_pyarray(python))?;
        columns.set_item("tile", tile.to_pyarray(python))?;
        Ok(columns)
    }

    /// Returns a `str` that names the world, its extent and its tick.
    ///
    /// The arguments in the text are the parameters the constructor takes, so
    /// a reader can paste the text back.
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

/// A camera the control plane owns, which says what a picture shows.
///
/// Build a camera, steer it, then pass it to `World.draw`. The camera holds
/// the size of one tile in pixels and the pixel offset of the tile at the
/// origin. Nothing else.
///
/// **A camera is not attached to a world.** One camera draws any world, and
/// two cameras draw one world. The camera verbs that need the size of the
/// picture take the width and the height as arguments.
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
/// # Build a camera
///
/// ```text
/// Camera(tile_size=None)
/// ```
///
/// **The parameters of the constructor are here, and not under a separate
/// entry.** The binding library does not publish the prose of a
/// constructor, so this class doc comment is the one place that holds
/// it.[^3]
///
/// - `tile_size`, a `float` or `None`. The width and the height of one tile,
///   in pixels. The default is `None`, which gives 12.0 pixels. The
///   constructor holds the value inside the range 2.0 to 64.0 pixels, so a
///   size outside that range gives the nearest size inside it.
///
/// The tile size is a choice of the caller and not a property of the world.
/// The new camera sits at the corner of the world, so it shows the tile at
/// the address `(0, 0)`. Call `look_at` to place it somewhere else.
///
/// **Call the `fitting` static method to get the camera to start from.** It
/// shows every tile of a world, so a caller that draws with it sees the world
/// rather than an empty picture.
///
/// # References
///
/// [^1]: ADR-0094, the caller owns the camera and the pixels, decision D3. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
/// [^2]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
/// [^3]: Findings register, FND-325. `docs/FINDINGS.md`
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
    /// Builds a camera.
    ///
    /// **The prose for this call lives in the doc comment of the class.** The
    /// binding library does not copy the doc comment of a constructor onto
    /// the Python object, so prose written here reaches no reader of the
    /// published reference.[^1]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-325. `docs/FINDINGS.md`
    #[new]
    #[pyo3(signature = (tile_size = None))]
    fn new(tile_size: Option<f32>) -> Self {
        Self {
            inner: tile_size.map_or_else(Camera::opening, Camera::at_tile_size),
        }
    }

    /// Returns a camera that fits the whole world into a picture of this
    /// size.
    ///
    /// The world is the world to fit. The width and the height are the size
    /// of the picture in pixels.
    ///
    /// **This is the camera to start from.** It shows every tile, so a caller
    /// that draws with it sees the world rather than an empty picture.
    #[staticmethod]
    fn fitting(world: &PyWorld, width: usize, height: usize) -> Self {
        Self {
            inner: Camera::fitting(&world.lock(), &FrameSize::new(width, height)),
        }
    }

    /// The width of one tile in pixels, as a `float`.
    #[getter]
    const fn tile_width(&self) -> f32 {
        self.inner.tile_width
    }

    /// Sets the width of one tile in pixels, as a `float`.
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

    /// The height of one tile in pixels, as a `float`.
    #[getter]
    const fn tile_height(&self) -> f32 {
        self.inner.tile_height
    }

    /// Sets the height of one tile in pixels, as a `float`.
    ///
    /// The setter holds the value to no bound, in the same way the width
    /// setter does. `World.draw` refuses a camera it cannot draw.
    #[setter]
    const fn set_tile_height(&mut self, pixels: f32) {
        self.inner.tile_height = pixels;
    }

    /// The pixel offset across the picture of the tile at the origin, as a
    /// `float`.
    ///
    /// The tile at the address `(0, 0)` is drawn at this offset. A larger
    /// value moves the world to the right in the picture.
    #[getter]
    const fn origin_x(&self) -> f32 {
        self.inner.origin_x
    }

    /// Sets the pixel offset across the picture of the tile at the origin, as
    /// a `float`.
    #[setter]
    const fn set_origin_x(&mut self, pixels: f32) {
        self.inner.origin_x = pixels;
    }

    /// The pixel offset down the picture of the tile at the origin, as a
    /// `float`.
    ///
    /// A larger value moves the world down in the picture.
    #[getter]
    const fn origin_y(&self) -> f32 {
        self.inner.origin_y
    }

    /// Sets the pixel offset down the picture of the tile at the origin, as a
    /// `float`.
    #[setter]
    const fn set_origin_y(&mut self, pixels: f32) {
        self.inner.origin_y = pixels;
    }

    /// Moves the view by whole presses of a scroll key. Returns `None`.
    ///
    /// The across value and the down value are counts of presses, as
    /// `float` values. Pass minus one, zero or one for each direction. The
    /// width and the height are the size of the picture in pixels.
    ///
    /// **This is the call a person drives.** The step is a share of the
    /// picture, so one press moves the view by the same part of it at every
    /// zoom.
    fn nudge(&mut self, across: f32, down: f32, width: usize, height: usize) {
        self.inner = self
            .inner
            .nudged(across, down, &FrameSize::new(width, height));
    }

    /// Moves the view by a count of pixels. Returns `None`.
    ///
    /// The across value and the down value are counts of pixels, as `float`
    /// values. The call needs no picture size, because a pixel is a pixel at
    /// every zoom.
    fn pan(&mut self, across: f32, down: f32) {
        self.inner = self.inner.panned(across, down);
    }

    /// Makes each tile larger by one press. Returns `None`.
    ///
    /// The width and the height are the size of the picture in pixels. The
    /// call keeps the tile under the middle of the picture under the middle,
    /// and it holds the tile size inside the range the camera accepts.
    fn zoom_in(&mut self, width: usize, height: usize) {
        self.inner = self.inner.zoomed_in(&FrameSize::new(width, height));
    }

    /// Makes each tile smaller by one press. Returns `None`.
    ///
    /// The width and the height are the size of the picture in pixels. The
    /// call keeps the tile under the middle of the picture under the middle,
    /// and it holds the tile size inside the range the camera accepts.
    fn zoom_out(&mut self, width: usize, height: usize) {
        self.inner = self.inner.zoomed_out(&FrameSize::new(width, height));
    }

    /// Puts a tile in the middle of the picture. Returns `None`.
    ///
    /// The address is the column `q` and the row `r`. The width and the
    /// height are the size of the picture in pixels. The call changes the
    /// offset alone, and it changes no tile size.
    fn look_at(&mut self, q: i32, r: i32, width: usize, height: usize) {
        self.inner = self
            .inner
            .looking_at(Axial::new(q, r), &FrameSize::new(width, height));
    }

    /// Holds the view inside the world. Returns `None`.
    ///
    /// The world is the world to stay inside. The width and the height are
    /// the size of the picture in pixels.
    ///
    /// A camera that ran off the edge would show a picture of nothing, and a
    /// person could not tell that from an empty world.
    fn clamp(&mut self, world: &PyWorld, width: usize, height: usize) {
        self.inner = self
            .inner
            .clamped(&world.lock(), &FrameSize::new(width, height));
    }

    /// Returns the tile under a pixel, as a `tuple` of two integers.
    ///
    /// The pixel offsets are `float` values, across and then down. The result
    /// is the column `q` and the row `r`.
    ///
    /// **The result may lie outside the world.** The call converts a pixel
    /// into an address and reads nothing, so check the pair against `width`
    /// and `height` before you pass it on.
    ///
    /// **This is how a click reaches a tile without a loop.** The control
    /// plane sends one pixel and gets one address, rather than walking the
    /// tiles to find which one was hit.
    fn tile_at(&self, x: f32, y: f32) -> (i32, i32) {
        let address = self.inner.tile_at(x, y);
        (address.q, address.r)
    }

    /// Returns a `str` that names the camera, its tile size and its origin.
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

/// Returns the version of the engine, as a `str`.
///
/// The value is the version of the compiled extension module. The package
/// exposes the same string as `cachette.__version__`.
#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// The compiled core of the Cachette simulation engine.
///
/// The module holds the `World` class, the `Camera` class, the `version`
/// function and the error classes. The `cachette` package re-exports all of
/// them, so import from `cachette` rather than from here.
///
/// **This module is for a programmer who drives a simulation from Python.**
/// One example is a game server that must replay a run exactly from a seed.
/// The caller builds a world, puts units in it, gives the units orders, and
/// steps the world.
///
/// # Install the package, then read this page
///
/// **No public package index carries this engine.** A checkout of the
/// repository builds it. The index name `cachette` belongs to a different
/// project, so do not install that name.[^1]
///
/// ```text
/// git clone https://github.com/tyevans/cachette
/// cd cachette
/// uv sync
/// uv run python -c "import cachette; print(cachette.version())"
/// ```
///
/// **This reference is the whole documentation site today.** No tutorial, no
/// how-to guide and no explanation page exists yet. Read the five conventions
/// below, then read the `World` class, which holds the parameters of its own
/// constructor.
///
/// Five conventions run through the whole interface. Read them once, and each
/// member below is then readable on its own.
///
/// # A fixed-point number crosses as a raw integer
///
/// The engine holds no floating point number in simulated state, because
/// float addition is not associative and an aggregate must combine exactly in
/// any order.[^2] It uses one fixed-point scale everywhere, Q16.16.
///
/// **A Q16.16 value reaches Python as a plain integer that is 65536 times the
/// value it stands for.** Divide by 65536 to read it as a quantity. Multiply
/// by 65536 to write one. A caller that reads such an integer as a count
/// reports a number 65536 times too large.
///
/// Each member below names the entries that carry this scale. An entry that
/// no member calls fixed point is a whole number, and the totals of a region
/// summary are the case where the two sit side by side.
///
/// # An entity crosses as one opaque identity
///
/// A soldier or a settlement reaches Python as one unsigned 64-bit integer.
/// The integer holds a slot and a generation together, and Python cannot take
/// it apart or build one.[^3] Pass it back to the engine, and the engine
/// resolves it.
///
/// The generation is what makes the identity safe. A soldier that dies leaves
/// its slot to the next soldier. The engine compares the generation, refuses
/// the dead identity, and raises `ViewError` rather than answer for the new
/// occupant.
///
/// # A tile crosses as a row-major index or as an axial address
///
/// The world is a rhombus of hexagonal tiles. An address is the pair `q` and
/// `r`, where `q` is the column and `r` is the row. Both start at zero.
///
/// A column of tiles uses the index instead, which is `r * width + q`. Take
/// `index % world.width` for the column and `index // world.width` for the
/// row.
///
/// # A kind crosses as a small integer
///
/// The resource kinds are food, wood and stone, numbered in that order from
/// zero. A column or a list of one entry for each resource kind is in that
/// order.
///
/// The ground kinds are water, plain, forest, hill and mountain, numbered in
/// that order from zero.
///
/// **The two scales are separate, and they overlap.** The numbers 0, 1 and 2
/// name a resource kind and a ground kind. A member that takes a resource
/// kind accepts a ground kind of 0, 1 or 2 and acts on the resource of that
/// number. It raises nothing, because the number does name a resource kind.
/// Read which scale a member takes before you pass a number to it.[^4]
///
/// **A commodity is a third scale, and it is not a kind.** `site_economy`
/// takes a commodity number. The world holds one commodity today, and its
/// number is zero.
///
/// A faction is a number from zero to one below the faction count of the
/// world. Where a value may name nobody, the entry is either `None` or the
/// number 65535, and each member below says which.[^5]
///
/// # Python sends one command over a set
///
/// Python builds a set and sends one command. Python does not loop over the
/// units of a population.[^6] The verbs that take a set accept a list of
/// identities, or the array of identities that an earlier call returned.
///
/// **A set-valued verb is all or nothing.** It resolves every identity and
/// checks every argument before it writes. One refusal leaves the world as it
/// was and raises.
///
/// **This module does not enforce the rule.** No type here refuses a loop. A
/// program that loops works on a small world and fails to scale.
///
/// # References
///
/// [^1]: Findings register, FND-341. `docs/FINDINGS.md`
/// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
/// [^3]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decisions D1 and D2. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
/// [^4]: Decisions register, DEC-120. `docs/DECISIONS.md`
/// [^5]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
/// [^6]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
#[pymodule]
#[pyo3(name = "_core")]
fn cachette_core_module(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyWorld>()?;
    module.add_class::<PyCamera>()?;
    module.add_function(wrap_pyfunction!(version, module)?)?;
    add_error::<CachetteError>(module, "CachetteError")?;
    add_error::<StepError>(module, "StepError")?;
    add_error::<FrameError>(module, "FrameError")?;
    add_error::<ConfigError>(module, "ConfigError")?;
    add_error::<SelectorError>(module, "SelectorError")?;
    add_error::<VerbError>(module, "VerbError")?;
    add_error::<ViewError>(module, "ViewError")?;
    add_error::<DeterminismError>(module, "DeterminismError")?;
    add_error::<EnginePanic>(module, "EnginePanic")?;
    Ok(())
}

/// The dotted path that every member of this module reports as its own.
const MODULE_PATH: &str = "cachette._core";

/// Adds one error class to the module, under the dotted module path.
///
/// The macro that declares an error writes the bare module name into
/// `__module__`. Every other member of this module reports the dotted path,
/// because the binding library writes it. A documentation build reads the
/// import and skips a member whose module does not match the module it
/// documents, so an error class published no prose and nothing failed.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-333. `docs/FINDINGS.md`
fn add_error<T: PyTypeInfo>(module: &Bound<'_, PyModule>, name: &str) -> PyResult<()> {
    let class = module.py().get_type::<T>();
    class.setattr("__module__", MODULE_PATH)?;
    module.add(name, class)
}
