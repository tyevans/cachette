//! The Python bindings for the Cachette simulation core.
//!
//! This crate wraps the core crate. The core crate has no PyO3 dependency.
//! A simulation function cannot take an interpreter token. A system cannot
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
use cachette_core::character::CharacterArena;
use cachette_core::descent::{DescentId, DESCENT_CEILING};
use cachette_core::founding::FoundingOutcome;
use cachette_core::hex::NEIGHBOURS;
use cachette_core::luxury::{LuxuryId, LUXURY_CEILING};
use cachette_core::unit_type::{UnitTypeId, UnitTypeRow};
use cachette_core::upgrade::UpgradeKind;
use cachette_core::TileIdx;
use cachette_core::{Advert, Consideration, KIND_LAND, KIND_RELATION, KIND_RESOURCE};
use cachette_core::{
    Axial, CommodityId, Entity, FactionId, Fix32, Holder, Influence, ResourceKind,
    World as CoreWorld, WorldConfig,
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

A verb is a call that changes the world. This class covers a refusal the
engine makes of the command itself. The refusals are:

- a number that names no kind,
- an address the ground refuses,
- a target below zero,
- a radius above the ceiling.

The message names the value the engine refused. The ceiling of a window
census is a radius of 64 tiles.

A verb that takes a set refuses the whole set. It writes nothing and
creates no partial state."
);
create_exception!(
    _core,
    ViewError,
    CachetteError,
    "A view was stale or out of scope.

The class covers two cases. The first is an identity that names no live
entity. This includes an identity the engine gave for an entity that has
since died. The second is an address or a window outside the world.

An identity is stale rather than wrong. The engine compares the generation.
It refuses the dead identity. It never answers for the next occupant of the
slot.[^1]

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
determinism guarantee instead. Neither reports through this class. A finding
records the gap.[^1]

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
`pyo3_runtime.PanicException`. That class is not a subclass of
`CachetteError`. A caller that wants to survive a panic catches
`BaseException`. A finding records the gap.[^1]

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
/// the same number of times with the same orders, reaches the same state hash.
/// That holds whether one thread or twelve threads ran it.[^1] That guarantee
/// is what makes a run repeatable. It says nothing about whether the run is
/// correct.
///
/// **The methods that answer are the ones a program reads.** No method hands
/// out a view into the world. A method that copies says so. A method that
/// answers about one thing takes one identity or one address.
///
/// # Build a world
///
/// ```text
/// World(width=64, height=64, seed=81985529216486895, faction_count=4)
/// ```
///
/// **The parameters of the constructor are here, and not under a separate
/// entry.** The binding library does not publish the prose of a constructor.
/// This class doc comment is the one place that holds it.[^2]
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
/// The world is a rhombus of hexagonal tiles. The extent is a width in
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
/// would be the violation the boundary record names. The engine keeps no
/// camera, no founding report and no timing.[^1] The binding is the caller
/// here. The demonstration binary is the caller on the other front end. A
/// caller is allowed to keep what it owns.
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
    /// of zero and a faction count above 63 are the two cases a caller meets
    /// first.
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

    /// Runs one step of the simulation and returns the number of tile change
    /// events it emitted, as an integer.
    ///
    /// The thread count is the number of threads the step may use. It has no
    /// default, so name it. **The result does not depend on it.** One thread
    /// and twelve threads give the same events. They give them in the same
    /// order, and they leave the same state hash.[^1]
    ///
    /// The step releases the global interpreter lock for its whole run.
    /// Another Python thread may run while the simulation runs. No Python
    /// code runs inside the step.[^2]
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
    /// in ascending tile order. Each record holds the tick, the tile, the
    /// value, the holder, the change kind and its declared padding.
    ///
    /// **A caller that reads a field out of these bytes holds a copy of the
    /// record layout. Nothing fails when the layout changes.** Call
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
    ///   step left it. The value 65535 means that nobody holds it. It sits
    ///   above the faction ceiling, so no faction collides with it.[^4]
    /// - `kind`, `numpy.uint8`. The kind of change. One means that the value
    ///   rose, and two means that it fell.
    ///
    /// The keys are the field names of the event. The caller reads a field
    /// by its name. No caller holds a byte offset, a field width, or a field
    /// order. Those live in the Rust source and nowhere else.[^1]
    ///
    /// This method copies each column. The log of one step is small next to
    /// the world.[^3]
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
    /// what it named. A reader that held one would report on the next
    /// occupant of the slot, with nothing failing.[^1]
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
    /// The count covers the last step alone. A new world reports zero. Read
    /// the events themselves with `gather_log_columns`.
    #[getter]
    fn gather_count(&self) -> usize {
        self.lock().gather_log().len()
    }

    /// Returns the fallen log of the last step, as a `dict` of NumPy arrays.
    ///
    /// A fallen event says that one unit fell in a meeting between two
    /// factions. Every array has one entry for each event, and all five
    /// arrays are the same length. That length is `fell_count`. A new world
    /// gives five empty arrays.
    ///
    /// - `tick`, `numpy.uint64`. The step at which the unit fell.
    /// - `unit`, `numpy.uint64`. The identity of the unit that fell.
    /// - `tile`, `numpy.uint32`. The tile it stood on, as a row-major index.
    ///   Take `index % world.width` for the column and `index // world.width`
    ///   for the row.
    /// - `faction`, `numpy.uint16`. The faction the unit belonged to. Every
    ///   entry names a faction of this world, because a unit always holds
    ///   one.
    /// - `unit_type`, `numpy.uint8`. The row of the shared type table that
    ///   the unit carried. It is the number `define_unit_type` writes.
    ///
    /// The keys are the field names of the event. The caller reads a field by
    /// its name, so no caller holds a byte offset, a field width or a field
    /// order.[^3]
    ///
    /// The entries come in ascending slot order, which is the order the step
    /// ended the units in. That order does not depend on the thread
    /// count.[^4]
    ///
    /// **The log names no killer.** The engine resolves a meeting for a whole
    /// group of units at one tile. No single attacker owns one death.[^5]
    /// The log says who fell, at which step, where it stood, and which
    /// faction and type it carried. It does not name the enemy. The caller
    /// reads the enemy from the tile and the step.
    ///
    /// **The log covers the last step alone, and the next step destroys it.**
    /// The step empties the log before it resolves a meeting. A step with no
    /// fight gives five empty arrays, and never the entries of an earlier
    /// step.[^6] Read the log after each `step` call whose deaths the caller
    /// wants, and keep what it needs. Every other log here holds the same
    /// rule.
    ///
    /// The unit column holds the whole identity of the unit that fell. It is
    /// not a slot index. A slot index survives the death of what it named.
    /// A reader that held one would report on the next occupant of the slot,
    /// with nothing failing.[^1]
    ///
    /// **Every identity in this column is dead**, because the step ended the
    /// unit that it names. `soldier_tile` refuses a dead identity. The
    /// `tile` column carries the ground the unit stood on. The caller needs
    /// no second read to place the death.[^1]
    ///
    /// This method copies each column.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decisions D1 and D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    /// [^2]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    /// [^3]: Decisions register, DEC-060. `docs/DECISIONS.md`
    /// [^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^5]: ADR-0123, casualties are whole units served to a keyed subset, decision D1. `docs/adrs/draft/adr-0123-casualties-are-whole-units-served-to-a-keyed-subset.md`
    /// [^6]: ADR-0121, a meeting between two factions resolves at the tile, decision D4. `docs/adrs/draft/adr-0121-a-meeting-between-two-factions-resolves-at-the-tile.md`
    fn fell_log_columns<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let log = world.fell_log();
        let columns = PyDict::new(python);
        let tick: Vec<u64> = log.iter().map(|event| event.tick.0).collect();
        let unit: Vec<u64> = log.iter().map(|event| event.unit).collect();
        let tile: Vec<u32> = log.iter().map(|event| event.tile.0).collect();
        let faction: Vec<u16> = log.iter().map(|event| event.faction.0).collect();
        let unit_type: Vec<u8> = log.iter().map(|event| event.unit_type.0).collect();
        columns.set_item("tick", tick.to_pyarray(python))?;
        columns.set_item("unit", unit.to_pyarray(python))?;
        columns.set_item("tile", tile.to_pyarray(python))?;
        columns.set_item("faction", faction.to_pyarray(python))?;
        columns.set_item("unit_type", unit_type.to_pyarray(python))?;
        Ok(columns)
    }

    /// The number of units that fell in the last step, as an integer.
    ///
    /// The count covers the last step alone. A new world reports zero. Read
    /// the events themselves with `fell_log_columns`.
    #[getter]
    fn fell_count(&self) -> usize {
        self.lock().fell_log().len()
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
    /// count where a unit is created and where a unit ends. This reads a
    /// small array and starts no pass over the population.[^1] A caller that
    /// counts the units of a faction in Python crosses the boundary once for
    /// each unit. The control plane rule forbids that.[^2]
    ///
    /// The list holds one entry for each faction the world was built with.
    /// The entry of a faction reads zero when its last unit ends. Nothing
    /// else the bindings expose says so.
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

    /// Returns which factions stand on the ground of which other factions.
    ///
    /// **This is one call and it names no unit.** It answers for every pair of
    /// factions at once. A caller that walked the population to reach the same
    /// answer would cross the boundary twice for each unit. The control plane
    /// rule forbids that.[^1]
    ///
    /// Returns a one-dimensional NumPy array of `numpy.uint64`, one entry for
    /// each faction the world was built with. Entry `host` is a set of
    /// factions, held as one bit for each faction. Bit `guest` is one when a
    /// live unit of faction `guest` stands on a tile that faction `host`
    /// holds.
    ///
    /// ```python
    /// presence = world.presence_masks()
    /// may_speak = bool(presence[other_god] & (1 << my_god))
    /// ```
    ///
    /// **The size of the answer does not change when the population changes.**
    /// A faction is one bit of a 64-bit word. A world holds at most 63
    /// factions, so the whole relation is one word for each faction.[^2]
    ///
    /// **A unit that stands on ground its own faction holds sets no bit.** The
    /// question is whether the people of one side stand on the ground of
    /// another side. Bit `host` of entry `host` is therefore always zero.
    ///
    /// **The answer is exact.** The engine reads the holder of the exact tile
    /// that each unit stands on. No summary reaches the answer. A bit that is
    /// zero means that no unit of that faction is there.
    ///
    /// The answer states the world as the last step left it.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the population changed since the last step. A
    /// call to `spawn_soldiers` or to `despawn_soldiers` makes the answer
    /// stale. The engine refuses rather than answering. Call `step` and ask
    /// again.
    ///
    /// # References
    ///
    /// [^1]: ADR-0040, Python is a control plane, not a data plane, decisions D1 and D2. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
    /// [^2]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D7. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    fn presence_masks<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyArray1<u64>>> {
        let world = self.lock();
        let factions = world.config().faction_count as usize;
        let raw: Vec<u64> = world
            .presence_rows()
            .map_err(|error| ViewError::new_err(error.to_string()))?
            .iter()
            .take(factions)
            .map(|row| row.to_bits())
            .collect();
        Ok(raw.to_pyarray(python))
    }

    /// Reports whether a unit of one faction stands on ground another holds.
    ///
    /// `guest` is the faction whose units the question is about. `host` is the
    /// faction that holds the ground. Both are faction numbers, counted from
    /// zero. Returns a `bool`.
    ///
    /// The call answers one entry of `presence_masks`. It costs the same. Ask
    /// this one about one pair. Ask `presence_masks` about several.
    ///
    /// **The answer is `False` when `guest` and `host` name one faction.** A
    /// faction is never a guest on its own ground.
    ///
    /// The answer states the world as the last step left it.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the world holds no such faction. The message
    /// names the number that refused. Raises `ViewError` when the population
    /// changed since the last step.
    fn stands_in_territory(&self, guest: u16, host: u16) -> PyResult<bool> {
        let world = self.lock();
        let factions = world.config().faction_count;
        for (name, number) in [("guest", guest), ("host", host)] {
            if number >= factions {
                return Err(ViewError::new_err(format!(
                    "the {name} faction {number} is outside a world of {factions} factions"
                )));
            }
        }
        world
            .stands_in_territory(FactionId(guest), FactionId(host))
            .map_err(|error| ViewError::new_err(error.to_string()))
    }

    /// Copies the tile holder column into a new NumPy array.
    ///
    /// Returns a one-dimensional array of `numpy.uint16`, one entry for each
    /// tile, in row-major order. Entry `r * width + q` is the tile at the
    /// address `(q, r)`.
    ///
    /// **Each entry is a faction number, or 65535 for a tile that nobody
    /// holds.** A faction number counts from zero. A world holds at most 63
    /// factions, so 65535 can never name one.[^1]
    ///
    /// **This is one call and it reads no tile from Python.** The engine holds
    /// the holders as one dense column, so the call copies that column. A
    /// caller that read one address at a time with `tile_report` would cross
    /// the boundary once for each tile. The control plane rule forbids that.[^2]
    ///
    /// The array covers the whole world and never a window. It has the same
    /// shape as the array that `tile_values` returns, so the two index alike.
    ///
    /// A holder changes only inside a step, so this call needs no freshness
    /// check and raises nothing.
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    /// [^2]: ADR-0040, Python is a control plane, not a data plane, decisions D1 and D2. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
    fn tile_holders<'py>(&self, python: Python<'py>) -> Bound<'py, PyArray1<u16>> {
        let world = self.lock();
        let raw: Vec<u16> = world
            .holding()
            .holders()
            .iter()
            .map(|holder| holder.to_bits())
            .collect();
        raw.to_pyarray(python)
    }

    /// Returns the name of every panel the viewer can draw.
    ///
    /// A caller passes one of these names to `draw` as its `panels` argument.
    /// The list comes from the viewer's own registration. A panel that joins
    /// the deck appears here with no edit to this file.[^1]
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
    /// a caller repeats. A soldier is the mass tier, and no caller walks that
    /// population.[^1] The identities come back as one column, in the order
    /// of the addresses.
    ///
    /// **The set is all or nothing.** An address the world refuses removes
    /// every soldier this call made and raises. A caller that got half a
    /// population and an error would have to work out which half. The engine
    /// already knows.
    ///
    /// The verb is set-valued at the boundary. It is still a loop inside, and
    /// spawning has no cheaper whole-set algorithm today.[^2]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the arena is full. It raises when an address
    /// is outside the world, or when the ground admits no unit. It raises
    /// when the world has no such faction. The error names the address that
    /// refused. Water is the ground that admits no unit.
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
    /// **The set is all or nothing.** Every identity resolves before the call
    /// removes any soldier. One dead identity removes nothing and raises.[^1]
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
    /// are the ground kinds 0, 1 and 2. Each of those numbers also names a
    /// resource kind. The call therefore reads 0, 1 or 2 as a resource kind.
    /// It orders the resource of that number. It raises nothing, and the
    /// soldiers gather the wrong resource. The engine sees a number, and not
    /// the scale the caller meant, so no check reports this.[^1]
    ///
    /// The call gives the order. It takes nothing. Step the world to make the
    /// soldiers act, then read `gather_log_columns` for what they took.
    ///
    /// **The set is all or nothing.** Every identity resolves, and the kind is
    /// checked, before any order is given.
    ///
    /// **A unit whose type cannot gather refuses the order.** The gather
    /// rate of the row the unit indexes is zero, so the unit would keep the
    /// order and take nothing on every step. The verb refuses instead, so
    /// the caller learns at once. Read `unit_type_table` for the rate of
    /// each row.[^2]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live soldier. Raises
    /// `VerbError` when the number is three or above, because that names no
    /// resource kind. Raises `VerbError` when a unit's type has a gather
    /// rate of zero.
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-120. `docs/DECISIONS.md`
    /// [^2]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    fn order_gather(&self, units: Vec<u64>, kind: u8) -> PyResult<()> {
        let mut world = self.lock();
        let kind = ResourceKind::from_u8(kind)
            .ok_or_else(|| VerbError::new_err(format!("{kind} names no resource kind")))?;
        let mut resolved = Vec::with_capacity(units.len());
        for unit in &units {
            let entity = resolve(&world, *unit)?;
            let row = world
                .unit_type_row(entity)
                .expect("a resolved identity names a live soldier");
            if row.gather_rate == Fix32::ZERO {
                return Err(VerbError::new_err(format!(
                    "the unit {unit} is of a type whose gather rate is zero, and it cannot gather"
                )));
            }
            resolved.push(entity);
        }
        // The set form is the one path the controller takes too, so one loop
        // serves both callers.
        let refused = world.order_gather_set(&resolved, kind);
        assert_eq!(
            refused, 0,
            "a resolved identity must name a soldier the arena can order"
        );
        Ok(())
    }

    /// Writes one row of the shared unit type table.
    ///
    /// A unit type is an index into this table. The table is data that the
    /// world holds. It holds no code, and the engine reads it rather than
    /// branching on a type name.[^1]
    ///
    /// The `unit_type` is the row number, as a Python integer. The table
    /// holds eight rows, numbered zero to seven. A number of eight or above
    /// names no row. A new soldier carries row zero, which the world builds
    /// as the worker row.
    ///
    /// **The call takes the whole row.** A row is eight capability columns,
    /// and a zero in a column means that the type cannot do what the column
    /// names. There is no two-column form, because a caller that gave two
    /// columns would leave the rest at zero and would define a unit that
    /// fights and does nothing else without knowing it.[^3]
    ///
    /// The `attack` is the harm that one unit of this type delivers in one
    /// resolution. The value is a Python integer in the project fixed-point
    /// scale. The unit of that scale is a whole casualty. The scale holds 16
    /// fractional bits, so one whole casualty is the value 65536. An attack
    /// of 65536 therefore ends one unit for each attacker. An attack of
    /// 32768 ends one unit for every two.
    ///
    /// The `armour` is the attack that an attacker must exceed to reach a
    /// unit of this type. The value is in the same scale.
    ///
    /// The keyword arguments follow, and each is a Python integer.
    ///
    /// - `gather_rate`. The scale on what the unit takes from a tile in one
    ///   tick, in the same fixed-point scale. 65536 takes the tile rate.
    ///   Zero takes nothing, and `order_gather` refuses the unit.
    /// - `build_rate`. The scale on the work the unit adds to an upgrade in
    ///   one tick, in the fixed-point scale. Zero adds nothing, and
    ///   `order_build` refuses the unit.
    /// - `carry_capacity`. The most the unit carries, summed over every
    ///   kind, as a whole count. A gather never raises a load above it. Zero
    ///   means the unit never carries, and so never gathers.
    /// - `move_cost_scale`. The scale on the movement cost the unit pays, in
    ///   the fixed-point scale. **No pass reads this column yet.**
    /// - `command_reach`. A whole count. Nonzero means the unit may move a
    ///   relation. **No pass reads this column yet.**
    /// - `weather_reach`. A whole count. Nonzero means the faction may
    ///   inflict weather while it holds the unit. **No pass reads this
    ///   column yet.**
    ///
    /// **An attacker whose attack does not exceed the defender's armour
    /// contributes exactly zero, however many attackers stand there.** The
    /// engine applies that test for each attacker type before it adds
    /// anything. No number of weak attackers therefore reaches a strong
    /// defender.[^2]
    ///
    /// The call changes the table and moves nothing. Step the world to make
    /// two factions on one tile resolve their meeting. Then read
    /// `faction_population` for what it cost.
    ///
    /// Returns `None`.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no row of the table. Raises
    /// `VerbError` when a fixed-point column is below zero. Raises
    /// `OverflowError` when a whole-count column is below zero or above the
    /// range of a 32-bit unsigned integer.
    ///
    /// # References
    ///
    /// [^1]: ADR-0120, a unit carries a type, and the type is an index into a table the world is built with, decisions D1 and D2. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
    /// [^2]: ADR-0122, an attacker whose attack does not exceed the defender's armour contributes exactly zero, decision D1. `docs/adrs/draft/adr-0122-an-attacker-below-the-armour-contributes-exactly-zero.md`
    /// [^3]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decisions D2 and D5. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    #[pyo3(signature = (
        unit_type,
        attack,
        armour,
        *,
        gather_rate,
        build_rate,
        carry_capacity,
        move_cost_scale,
        command_reach,
        weather_reach,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn define_unit_type(
        &self,
        unit_type: u8,
        attack: i32,
        armour: i32,
        gather_rate: i32,
        build_rate: i32,
        carry_capacity: u32,
        move_cost_scale: i32,
        command_reach: u32,
        weather_reach: u32,
    ) -> PyResult<()> {
        let mut world = self.lock();
        let row = UnitTypeRow {
            attack: Fix32(attack),
            armour: Fix32(armour),
            gather_rate: Fix32(gather_rate),
            build_rate: Fix32(build_rate),
            carry_capacity,
            move_cost_scale: Fix32(move_cost_scale),
            command_reach,
            weather_reach,
        };
        world
            .define_unit_type(unit_type, row)
            .map_err(|error| VerbError::new_err(error.to_string()))
    }

    /// Gives every soldier the identities name one unit type.
    ///
    /// The units are a sequence of identities, or the NumPy array of
    /// `numpy.uint64` that `spawn_soldiers` returned. Returns `None`.
    ///
    /// The `unit_type` is a row of the shared table, as a Python integer. The
    /// table holds eight rows, numbered zero to seven. A number of eight or
    /// above names no row. The world builds the table with the default rows,
    /// and `define_unit_type` writes one. A row whose every column is zero
    /// is a unit that can do nothing.[^1]
    ///
    /// **The set is all or nothing.** Every identity resolves, and the type
    /// is checked, before any soldier is written. One refusal leaves the
    /// world unchanged and raises.[^2]
    ///
    /// The call gives the type. It takes nothing and it moves nothing. Step
    /// the world to make the soldiers act.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live soldier. Raises
    /// `VerbError` when the number names no row of the table.
    ///
    /// # References
    ///
    /// [^1]: ADR-0120, a unit carries a type, and the type is an index into a table the world is built with, decision D3. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
    /// [^2]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    fn set_unit_types(&self, units: Vec<u64>, unit_type: u8) -> PyResult<()> {
        let mut world = self.lock();
        let kind = UnitTypeId::from_u8(unit_type)
            .ok_or_else(|| VerbError::new_err(format!("{unit_type} names no unit type")))?;
        let mut resolved = Vec::with_capacity(units.len());
        for unit in &units {
            resolved.push(resolve(&world, *unit)?);
        }
        let refused = world.set_unit_type_set(&resolved, kind);
        assert_eq!(
            refused, 0,
            "a resolved identity must name a soldier the arena can write"
        );
        Ok(())
    }

    /// Returns the unit type of one soldier, as an integer.
    ///
    /// The unit is one identity, as a Python integer. Take an entry of the
    /// array that `spawn_soldiers` returned, or of the `unit` column of the
    /// gather log.
    ///
    /// The result is a row of the shared table. Read the row itself with
    /// `unit_type_table`, and write it with `define_unit_type`. A new
    /// soldier carries row zero.[^1]
    ///
    /// **This read stays singular while the write verb takes a set.** A set
    /// form must choose. It fails the whole call for one dead identity, or
    /// it returns a value that stands for nothing. `soldier_tile` follows
    /// the same rule.[^2]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no live soldier, and when
    /// the value is not an identity the engine ever gave.
    ///
    /// # References
    ///
    /// [^1]: ADR-0120, a unit carries a type, and the type is an index into a table the world is built with, decisions D1 and D3. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
    /// [^2]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decisions D1 and D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    fn unit_type(&self, unit: u64) -> PyResult<u8> {
        let world = self.lock();
        let entity = resolve(&world, unit)?;
        let found = world
            .unit_type(entity)
            .ok_or_else(|| ViewError::new_err(format!("{unit} names no live soldier")))?;
        Ok(found.0)
    }

    /// Returns the shared unit type table, as a `dict` of NumPy arrays.
    ///
    /// A unit type is a row of this table. A soldier carries the row number
    /// alone. The table is data that the world holds, and it is not code.[^1]
    ///
    /// Every array holds one entry for each row, and every array is the
    /// same length. **That length is the number of types the world holds.**
    /// Nothing else states the width. A caller reads the width from this
    /// return value, not from a second number that could disagree.[^3]
    ///
    /// The keys are the column names of the row, in the order the engine
    /// declares them. Each value is a `numpy.int64` array. The keys and the
    /// keyword arguments of `define_unit_type` are the same names, and each
    /// entry carries the value that call took: a fixed-point column keeps
    /// its raw Q16.16 value, so one whole casualty is 65536, and a whole
    /// count column keeps its count.
    ///
    /// **The width of the table is fixed, and the values are configurable.**
    /// The world builds the table with the default rows: a worker, a
    /// soldier, a merchant, a leader, one open row, and zero above them.
    /// `define_unit_type` writes one row. A row whose every column is zero
    /// is a unit that can do nothing: it reaches nothing, nothing reaches
    /// it, and it gathers, builds and carries nothing.[^1] [^5]
    ///
    /// The values are content. A record may not hold a number that a content
    /// choice can move, so no record holds one.[^4]
    ///
    /// This method copies each column.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0120, a unit carries a type, and the type is an index into a table the world is built with, decisions D1 and D2. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
    /// [^2]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    /// [^3]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    /// [^4]: Decision Record Scope, section 4.1. `.claude/rules/adr-scope.md`
    /// [^5]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decisions D2 and D4. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    fn unit_type_table<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let rows = world.unit_types().rows();
        let columns = PyDict::new(python);
        // The names and the values both come from the row declaration, so
        // the dictionary cannot name a column the row does not hold.
        for (index, name) in UnitTypeRow::COLUMN_NAMES.iter().enumerate() {
            let column: Vec<i64> = rows.iter().map(|row| row.columns()[index]).collect();
            columns.set_item(name, column.to_pyarray(python))?;
        }
        Ok(columns)
    }

    /// Returns the starved log of the last step, as a `dict` of NumPy arrays.
    ///
    /// A starved event says that a shortage ended one unit. The engine writes
    /// one entry for each unit that the scan of this step removed, in
    /// ascending slot order.
    ///
    /// **The log holds the last step alone.** The next step clears the log
    /// before it does anything. The entries of one step are gone once another
    /// step runs. Keep a copy of what you need. The engine holds no queue.
    ///
    /// The consumption pass runs on a schedule. The scan runs with it. On a
    /// step the schedule does not name, and on a step that ended nobody, the
    /// log is empty. A reader cannot tell the two cases apart, and nothing
    /// needs to.
    ///
    /// - `tick`, `numpy.uint64`. The step at which the scan ended the unit.
    /// - `unit`, `numpy.uint64`. The identity of the unit that ended. It is
    ///   not a slot index. It never resolves again, because the unit is
    ///   dead.[^1]
    /// - `deficit`, `numpy.int32`. What the unit went short by. The value
    ///   carries the Q16.16 fixed-point scale, so 65536 is one whole unit of
    ///   need. It is at or above the bound that ends a unit.
    ///
    /// This method copies each column.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decisions D1 and D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    /// [^2]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    fn starved_log_columns<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let log = world.starved_log();
        let columns = PyDict::new(python);
        let tick: Vec<u64> = log.iter().map(|event| event.tick.0).collect();
        let unit: Vec<u64> = log.iter().map(|event| event.unit).collect();
        let deficit: Vec<i32> = log.iter().map(|event| event.deficit.0).collect();
        columns.set_item("tick", tick.to_pyarray(python))?;
        columns.set_item("unit", unit.to_pyarray(python))?;
        columns.set_item("deficit", deficit.to_pyarray(python))?;
        Ok(columns)
    }

    /// Gives every settlement the identities name one upkeep rate.
    ///
    /// Upkeep is the amount a site spends of a commodity. It is a rate at or
    /// above zero. It subtracts from the store, and it is never a production
    /// rate below zero.[^1]
    ///
    /// The sites are a sequence of settlement identities, or the NumPy array
    /// of `numpy.uint64` that `found_settlements` returned. Returns `None`.
    ///
    /// The `commodity` is the number of a commodity. A commodity is not a
    /// resource kind. The world holds one commodity today, and its number is
    /// zero. The argument has that number by default.
    ///
    /// **The rate is a Q16.16 value as its raw integer.** Multiply the amount
    /// you want by 65536. The rate is what one tick spends. The schedule
    /// scales it to one application. A longer period therefore does not
    /// change what a site spends over a span of steps. The engine holds no
    /// floating point number in simulated state, because float addition is
    /// not associative.[^2]
    ///
    /// **This is the one call that makes a shortfall possible.** A site that
    /// spends nothing can never fall short. `shortfall_log_columns` therefore
    /// answers with an empty log until a caller writes an upkeep rate. A
    /// finding records that the rate had no caller outside a test before this
    /// binding.[^3]
    ///
    /// **The set is all or nothing.** Every identity resolves, and the rate
    /// and the commodity are checked, before any site is written. One refusal
    /// leaves the world unchanged and raises.[^4]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live settlement. Raises
    /// `VerbError` when the rate is below zero, and when the number names no
    /// commodity of this world.
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-016. `docs/FINDINGS.md`
    /// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^3]: Findings register, FND-460. `docs/FINDINGS.md`
    /// [^4]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    #[pyo3(signature = (sites, rate, commodity = 0))]
    fn spend_at_sites(&self, sites: Vec<u64>, rate: i32, commodity: u16) -> PyResult<()> {
        let mut world = self.lock();
        // **The engine holds the one check of the rate, and this call adds
        // none.** A copy here would be a second statement of a rule the engine
        // already enforces, and nothing would fail when the two disagreed. The
        // engine checks the rate before it writes anything, and the rate is
        // one value for the whole set, so the first refusal leaves every site
        // untouched.
        let goods = CommodityId(commodity);
        let mut resolved = Vec::with_capacity(sites.len());
        for site in &sites {
            resolved.push(resolve_site(&world, *site)?);
        }
        // The commodity is checked against the store of each site before
        // anything is written. A write that refused halfway would leave one
        // part of the set changed and the rest untouched. The store holds one
        // quantity for each commodity, so it is what states the set.
        for entity in &resolved {
            world
                .settlements()
                .store(*entity)
                .and_then(|held| held.quantity(goods))
                .ok_or_else(|| {
                    VerbError::new_err(format!("{commodity} names no commodity of this world"))
                })?;
        }
        for entity in resolved {
            let wrote = world
                .set_upkeep_rate(entity, goods, Fix32(rate))
                .map_err(|error| VerbError::new_err(error.to_string()))?;
            assert!(
                wrote,
                "a resolved identity must name a settlement the world can write"
            );
        }
        Ok(())
    }

    /// Returns the shortfall log of the last step, as a `dict` of NumPy
    /// arrays.
    ///
    /// A shortfall event says that one site could not pay its upkeep. The
    /// store stopped at zero rather than going below it. The amount is what
    /// the world must supply to make the site solvent.
    ///
    /// **The log holds the last step alone.** The next step clears the log
    /// before it does anything. The entries of one step are gone once another
    /// step runs. Keep a copy of what you need. The engine holds no queue.
    ///
    /// The rate pass runs on a schedule. On a step the schedule does not
    /// name, and on a step in which every site paid, the log is empty.
    ///
    /// - `tick`, `numpy.uint64`. The step at which the upkeep applied.
    /// - `site`, `numpy.uint64`. The identity of the settlement that could
    ///   not pay. It is not a slot index. Hand it back to
    ///   `site_economy`.[^1]
    /// - `amount`, `numpy.int32`. What the upkeep could not take. The value
    ///   carries the Q16.16 fixed-point scale, so 65536 is one whole unit of
    ///   the commodity. It is never zero.
    /// - `commodity`, `numpy.uint16`. The commodity that the site owed. A
    ///   commodity is not a resource kind. The world holds one commodity
    ///   today, and its number is zero.
    ///
    /// This method copies each column.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decisions D1 and D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    /// [^2]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    fn shortfall_log_columns<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let log = world.shortfall_log();
        let columns = PyDict::new(python);
        let tick: Vec<u64> = log.iter().map(|event| event.tick.0).collect();
        let site: Vec<u64> = log.iter().map(|event| event.site).collect();
        let amount: Vec<i32> = log.iter().map(|event| event.amount.0).collect();
        let commodity: Vec<u16> = log.iter().map(|event| event.commodity).collect();
        columns.set_item("tick", tick.to_pyarray(python))?;
        columns.set_item("site", site.to_pyarray(python))?;
        columns.set_item("amount", amount.to_pyarray(python))?;
        columns.set_item("commodity", commodity.to_pyarray(python))?;
        Ok(columns)
    }

    /// Returns the rationed log of the last step, as a `dict` of NumPy
    /// arrays.
    ///
    /// A rationed event says that one site could not serve every cohort that
    /// drew on it. A cohort is the group of units of one faction that draw
    /// from one site. The store stopped at zero rather than going below it.
    /// The granted amount is always below the demanded amount.
    ///
    /// **The log holds the last step alone.** The next step clears the log
    /// before it does anything. The entries of one step are gone once another
    /// step runs. Keep a copy of what you need. The engine holds no queue.
    ///
    /// The consumption pass runs on a schedule. On a step the schedule does
    /// not name, the log is empty. On a step in which every site served every
    /// cohort, the log is empty.
    ///
    /// - `tick`, `numpy.uint64`. The step at which the draw ran.
    /// - `site`, `numpy.uint64`. The identity of the settlement that could
    ///   not serve. It is not a slot index. Hand it back to
    ///   `site_economy`.[^1]
    /// - `demanded`, `numpy.int64`. What the cohorts of the site asked for.
    ///   The value carries the Q16.16 fixed-point scale, so 65536 is one
    ///   whole unit of the commodity.
    /// - `granted`, `numpy.int64`. What the store gave, in the same Q16.16
    ///   scale. It is always below the demanded amount.
    /// - `commodity`, `numpy.uint16`. The commodity that the cohorts drew. A
    ///   commodity is not a resource kind. The world holds one commodity
    ///   today, and its number is zero.
    ///
    /// This method copies each column.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decisions D1 and D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    /// [^2]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    fn rationed_log_columns<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let log = world.rationed_log();
        let columns = PyDict::new(python);
        let tick: Vec<u64> = log.iter().map(|event| event.tick.0).collect();
        let site: Vec<u64> = log.iter().map(|event| event.site).collect();
        let demanded: Vec<i64> = log.iter().map(|event| event.demanded.0).collect();
        let granted: Vec<i64> = log.iter().map(|event| event.granted.0).collect();
        let commodity: Vec<u16> = log.iter().map(|event| event.commodity).collect();
        columns.set_item("tick", tick.to_pyarray(python))?;
        columns.set_item("site", site.to_pyarray(python))?;
        columns.set_item("demanded", demanded.to_pyarray(python))?;
        columns.set_item("granted", granted.to_pyarray(python))?;
        columns.set_item("commodity", commodity.to_pyarray(python))?;
        Ok(columns)
    }

    /// Returns the promotion log of the last step, as a `dict` of NumPy
    /// arrays.
    ///
    /// A promotion event says that one soldier became a character. The engine
    /// writes one entry for each soldier the pass promoted, in rank order,
    /// with the highest deeds first.
    ///
    /// **The log holds the last step alone.** The next step clears the log
    /// before it does anything. The entries of one step are gone once another
    /// step runs. Keep a copy of what you need. The engine holds no queue.
    ///
    /// The promotion pass runs on a schedule. On a step the schedule does not
    /// name, and on a step that promoted nobody, the log is empty.
    ///
    /// - `tick`, `numpy.uint64`. The step at which the pass promoted the
    ///   soldier.
    /// - `unit`, `numpy.uint64`. The identity of the soldier. It is not a
    ///   slot index. The soldier stays alive, so `soldier_tile` answers for
    ///   it.[^1]
    /// - `character`, `numpy.uint64`. The identity of the character that the
    ///   promotion created. It is not a slot index.[^1]
    /// - `deeds`, `numpy.uint64`. What the soldier gathered, as a running
    ///   total. It is a whole number of units of stock. It carries no
    ///   fixed-point scale.
    /// - `faction`, `numpy.uint16`. The faction of the soldier and of the
    ///   character.
    ///
    /// This method copies each column.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decisions D1 and D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    /// [^2]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    fn promoted_log_columns<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let log = world.promoted_log();
        let columns = PyDict::new(python);
        let tick: Vec<u64> = log.iter().map(|event| event.tick.0).collect();
        let unit: Vec<u64> = log.iter().map(|event| event.unit).collect();
        let character: Vec<u64> = log.iter().map(|event| event.character).collect();
        let deeds: Vec<u64> = log.iter().map(|event| event.deeds).collect();
        let faction: Vec<u16> = log.iter().map(|event| event.faction.0).collect();
        columns.set_item("tick", tick.to_pyarray(python))?;
        columns.set_item("unit", unit.to_pyarray(python))?;
        columns.set_item("character", character.to_pyarray(python))?;
        columns.set_item("deeds", deeds.to_pyarray(python))?;
        columns.set_item("faction", faction.to_pyarray(python))?;
        Ok(columns)
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
    /// died leaves its slot to another soldier. This method refuses the dead
    /// identity rather than report on the new occupant.[^1]
    ///
    /// **This read stays singular while the write verbs take a set.** A set
    /// form must choose. It fails the whole call for one dead identity, or
    /// it returns a value that stands for nothing. That value is the false
    /// answer the record forbids. The read therefore answers for one
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

    /// Tells every soldier the identities name to build a kind of upgrade.
    ///
    /// The units are a sequence of identities, or the NumPy array of
    /// `numpy.uint64` that `spawn_soldiers` returned. Returns `None`.
    ///
    /// The kind is the upgrade kind, as an integer. A road is zero and a
    /// terrace is one. The argument has no default. A road lets more units
    /// stand on the tile. A terrace lets a unit take more from the tile in
    /// one step.
    ///
    /// Each soldier adds to the upgrade on the tile it stands on, at every
    /// step, until something stops it. A soldier does not have to stay. A
    /// soldier that walks away stops adding, and the work it did stays on the
    /// tile.[^1] Several soldiers on one tile add to one total. That total
    /// is the same at every thread count.[^2]
    ///
    /// **An unfinished build changes nothing about the tile.** The tile
    /// changes when the work reaches the amount its kind asks for.[^1] Read
    /// `tile_report` for the work done so far.
    ///
    /// The call gives the order and builds nothing. Step the world to make
    /// the soldiers build.
    ///
    /// **The kind here is an upgrade kind. It is not a resource kind and it
    /// is not a ground kind.** More than one scale in this module carries the
    /// name `kind`, and each of them starts at zero. The call accepts the
    /// resource kinds of food and wood. It accepts the ground kinds of water
    /// and plain. Each of those numbers also names an upgrade kind. It raises
    /// nothing, and the soldiers build the wrong thing. The engine sees a
    /// number and not the scale the caller meant.[^3]
    ///
    /// **The engine does not check who holds the ground.** A soldier builds
    /// on the tile it stands on, whatever faction holds that tile. A caller
    /// that wants the other rule holds it in Python. A finding records the
    /// gap.[^4]
    ///
    /// **The set is all or nothing.** Every identity resolves, and the kind
    /// is checked, before the engine gives any order.
    ///
    /// **A unit whose type cannot build refuses the order.** The build rate
    /// of the row the unit indexes is zero, so the unit would keep the order
    /// and add nothing on every step. The verb refuses instead, so the
    /// caller learns at once. Read `unit_type_table` for the rate of each
    /// row.[^5]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live soldier. Raises
    /// `VerbError` when the number is two or above, because that names no
    /// upgrade kind. The message names the number that refused. Raises
    /// `VerbError` when a unit's type has a build rate of zero.
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decisions D2 and D3. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    /// [^2]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    /// [^3]: Findings register, FND-352. `docs/FINDINGS.md`
    /// [^4]: Findings register, FND-380. `docs/FINDINGS.md`
    /// [^5]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    fn order_build(&self, units: Vec<u64>, kind: u8) -> PyResult<()> {
        let mut world = self.lock();
        let kind = UpgradeKind::from_u8(kind)
            .ok_or_else(|| VerbError::new_err(format!("{kind} names no upgrade kind")))?;
        let mut resolved = Vec::with_capacity(units.len());
        for unit in &units {
            let entity = resolve(&world, *unit)?;
            let row = world
                .unit_type_row(entity)
                .expect("a resolved identity names a live soldier");
            if row.build_rate == Fix32::ZERO {
                return Err(VerbError::new_err(format!(
                    "the unit {unit} is of a type whose build rate is zero, and it cannot build"
                )));
            }
            resolved.push(entity);
        }
        // The set form is the one path the controller takes too, so one loop
        // serves both callers.
        let refused = world.order_build_set(&resolved, kind);
        assert_eq!(
            refused, 0,
            "a resolved identity must name a soldier the arena can order"
        );
        Ok(())
    }

    /// Tells every soldier the identities name to stop building.
    ///
    /// The units are a sequence of identities, or the NumPy array of
    /// `numpy.uint64` that `spawn_soldiers` returned. Returns `None`.
    ///
    /// The work each soldier already did stays on its tile. A soldier that
    /// takes the order again continues rather than restarts.[^1] Nothing here
    /// removes an upgrade: call `destroy_upgrades` for that.
    ///
    /// A soldier that builds nothing takes this order and stays as it was.
    ///
    /// **The set is all or nothing.** Every identity resolves before the
    /// engine stops any order.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live soldier.
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D2. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    fn stop_build(&self, units: Vec<u64>) -> PyResult<()> {
        let mut world = self.lock();
        let mut resolved = Vec::with_capacity(units.len());
        for unit in &units {
            resolved.push(resolve(&world, *unit)?);
        }
        for entity in resolved {
            assert!(
                world.stop_build(entity),
                "a resolved identity must name a soldier the arena can order"
            );
        }
        Ok(())
    }

    /// Returns what one soldier builds, as an integer, or `None`.
    ///
    /// The unit is one identity, as a Python integer. Take an entry of the
    /// array that `spawn_soldiers` returned.
    ///
    /// The result is the upgrade kind that `order_build` took: a road is zero
    /// and a terrace is one. The result is `None` when the soldier builds
    /// nothing.
    ///
    /// **This read stays singular while the write verbs take a set.** A set
    /// form must choose. It fails the whole call for one dead identity, or
    /// it returns a value that stands for nothing. The second is a false
    /// answer the record forbids.[^1] The read therefore answers for one
    /// identity and says which one failed.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no live soldier, and when
    /// the value is not an identity the engine ever gave.
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    fn build_order(&self, unit: u64) -> PyResult<Option<u8>> {
        let world = self.lock();
        let entity = resolve(&world, unit)?;
        let order = world.build_order(entity).ok_or_else(|| {
            ViewError::new_err(format!("the identity {unit} names no live soldier"))
        })?;
        Ok(order.map(UpgradeKind::to_u8))
    }

    /// Removes the upgrade at each address and returns how many it removed.
    ///
    /// The addresses are a sequence of `(q, r)` pairs of integers. Returns an
    /// integer, which counts the tiles that carried an upgrade.
    ///
    /// Each tile returns to the world that the generator made. Nothing else
    /// stores a property of an improved tile. Removing the entry is the whole
    /// of the return.[^1] The removal takes effect at once, and it needs no
    /// step.
    ///
    /// The call removes a finished upgrade and an unfinished one alike. An
    /// unfinished upgrade loses the work that went into it.
    ///
    /// **An address that carries no upgrade is not a refusal.** The engine
    /// removes nothing there and does not count it. Two calls for one address
    /// therefore count one removal and then none.
    ///
    /// **The call removes no build order.** A soldier that stands on the tile
    /// and holds an order starts the upgrade again at the next step. Call
    /// `stop_build` for that soldier first.
    ///
    /// **The engine does not check who holds the ground.** Any caller may
    /// remove any upgrade, and the removal is instant.
    ///
    /// **The set is all or nothing.** Every address is checked against the
    /// world before the engine removes anything.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an address lies outside the world. The message
    /// names the address that refused.
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D4. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    fn destroy_upgrades(&self, addresses: Vec<(i32, i32)>) -> PyResult<usize> {
        let mut world = self.lock();
        for (q, r) in &addresses {
            if world.tile_kind(Axial::new(*q, *r)).is_none() {
                return Err(ViewError::new_err(format!(
                    "({q}, {r}) lies outside this world"
                )));
            }
        }
        let mut removed = 0;
        for (q, r) in addresses {
            if world.destroy_upgrade(Axial::new(q, r)) {
                removed += 1;
            }
        }
        Ok(removed)
    }

    /// Returns the direction home for one faction at one address.
    ///
    /// The faction is a faction number of this world. The address is the pair
    /// `q` and `r`, as integers.
    ///
    /// The result is a direction, as an integer, or `None`. A direction is an
    /// index into the list that `direction_offsets` returns. The result is
    /// `None` when the block of ground holds a settlement of that faction.
    /// It is also `None` when no settlement of that faction is in reach.
    ///
    /// **The field answers for a block of ground and not for a tile.** The
    /// engine derives one direction for each faction and each block. Two
    /// addresses in one block therefore give one answer.[^1] The engine
    /// derives the field again at every step.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the address lies outside the world, and when
    /// the world holds no such faction.
    ///
    /// # References
    ///
    /// [^1]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decision D1. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
    fn return_direction(&self, faction: u16, q: i32, r: i32) -> PyResult<Option<u8>> {
        let world = self.lock();
        world
            .return_direction(FactionId(faction), Axial::new(q, r))
            .ok_or_else(|| {
                ViewError::new_err(format!(
                    "({q}, {r}) and the faction {faction} name no entry of the return field"
                ))
            })
    }

    /// Returns the offset of each direction, as a list of `(q, r)` pairs.
    ///
    /// A tile of this world has six neighbours. A direction is an index into
    /// this list. Add the pair at that index to an address to get the
    /// neighbour in that direction. The order never changes.[^1]
    ///
    /// The list is the one that the engine itself uses. The file declares
    /// the order nowhere else, so no second statement can disagree with it.
    ///
    /// Read `return_direction` for a direction to take.
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[staticmethod]
    fn direction_offsets() -> Vec<(i32, i32)> {
        NEIGHBOURS
            .iter()
            .map(|offset| (offset.q, offset.r))
            .collect()
    }

    /// The number of settlements standing in the world, as an integer.
    ///
    /// The count covers the settlements that stand now. A settlement the
    /// world removes no longer counts.
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
    /// from the ground that a survey read. This call runs no survey. Call
    /// `found_group` for a settlement that produces.[^1]
    ///
    /// **The set is all or nothing.** When the world refuses an address, the
    /// call destroys every settlement it made and raises.
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
    /// **The command names no unit.** It says what a place wants. The engine
    /// turns that into a number of positions of each kind at the next
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
    /// The engine acts on the new target at the next rebalance.
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
    /// Every array has one entry for each position. All three arrays are the
    /// same length.
    ///
    /// - `kind`, `numpy.uint8`. The kind of work: food is zero, wood is one
    ///   and stone is two.
    /// - `rank`, `numpy.uint8`. The rank of the position inside its kind,
    ///   counting from zero.
    /// - `holder`, `numpy.uint64`. The identity of the unit that holds the
    ///   position, and zero where a position holds nobody.
    ///
    /// The columns hold the positions of the site and nothing else. An entry
    /// that is no position does not appear.
    ///
    /// **A site holds no position until a rebalance runs.** A site founded in
    /// this step reports three empty arrays. Step the world. Read
    /// `set_position_schedule` for how often the rebalance runs.
    ///
    /// The holder column carries the whole identity of the unit that holds
    /// each position. It is zero where a position holds nobody. It is not a
    /// slot index.[^1]
    ///
    /// **This read stays singular while the write verb takes a set.** A set
    /// form would have to answer for a dead identity with a value that stands
    /// for nothing. The unit read follows the same rule.[^1]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no live settlement.
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
    /// period. With a period of four and a phase of one, the engine
    /// rebalances on ticks one, five and nine. A phase at or above the
    /// period wraps into it.
    ///
    /// The world starts with a schedule already set. This call replaces it.
    ///
    /// A rebalance turns the targets that `prefer_at_sites` wrote into the
    /// positions that `site_positions` reports.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the period is zero, or above the range that
    /// the scaling multiply takes. The message names the limit.
    fn set_position_schedule(&self, period: u32, phase: u32) -> PyResult<()> {
        self.lock()
            .set_position_schedule(period, phase)
            .map_err(|error| VerbError::new_err(error.to_string()))
    }

    /// Returns the survey the engine would run for this group and faction,
    /// as a `dict`.
    ///
    /// The group is the number of people that would settle. The faction is
    /// the number of the faction that would settle them. The faction chooses
    /// the sample, so two factions read two samples.[^1]
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
    /// number of tiles around each one. Neither number grows with the size
    /// of the world, so this call costs the same in a large world as in a
    /// small one.[^2]
    ///
    /// The call writes nothing, and it founds nothing. It shows how the
    /// engine makes the score: the columns hold the counts the survey read,
    /// and the score column holds the engine's weighted sum of them.[^3]
    ///
    /// The rows are the candidates, in the order the founding ranks them,
    /// best first. Row zero is the place a founding would take. A row whose
    /// `eligible` entry is zero is a place the founding refuses.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the group holds nobody, or when the ordering
    /// of the candidates refuses to run.
    ///
    /// # References
    ///
    /// [^1]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D3. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
    /// [^2]: ADR-0075, the founding choice reads a bounded sample of the world, decision D1. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    /// [^3]: ADR-0075, the founding choice reads a bounded sample of the world, decision D5. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
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
    /// This is the whole loop in one call. The survey reads the ground. The
    /// founding takes the best place the sample offered. It seats the group
    /// over the disc around that place. It sets the production rate of the
    /// site from the food the survey read.[^1] [^2] A caller that founds at
    /// an address of its own gets a site that earns nothing. The rate comes
    /// from the survey.
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
    /// a reader asks about a cell without reading its tiles. Give the
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
    /// entry is an exact integer total over the tiles of the cell. A reader
    /// can add the tiles of the cell and get the same number back.[^1] [^2]
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
    /// **A commodity is not a resource kind.** The numbers name different
    /// things. The world holds one commodity today, and its number is zero.
    /// Every other number raises `ViewError`, so the resource kinds one and
    /// two name no commodity.
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
    /// `found_group` and `found_run_for_every_faction` run one.[^2]
    ///
    /// The ration entries come from the log of the draw that just ran. The
    /// engine keeps that log for one tick, so a site that served every
    /// cohort in full reports no shortfall.[^3]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no live settlement, or
    /// when the world holds no such commodity.
    ///
    /// # References
    ///
    /// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^2]: ADR-0062, production and upkeep are rates attached to a site, decision D2. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    /// [^3]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D3. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
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

    /// Returns why one unit chose the intent it carries, as a `dict`.
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
    ///   each option, in option order. The field is what the option read, and
    ///   the weight is what the option carried. The score is the weighted sum
    ///   the engine made from the two. **All three hold Q16.16 values as raw
    ///   integers.**
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
    ///   and `None` for ground that nobody holds.[^1]
    /// - `upgrade`, an integer or `None`. The upgrade the tile carries,
    ///   finished or under construction, and `None` for a tile that carries
    ///   none. A road is zero and a terrace is one.
    /// - `upgrade_progress`, an integer. The work that has gone into that
    ///   upgrade, and zero for a tile that carries none. The number never
    ///   rises above the work its kind asks for.[^2]
    /// - `upgrade_complete`, a `bool`. Whether that upgrade is finished.
    ///   `False` for a tile that carries none.
    ///
    /// The three upgrade entries are what a watcher of a build reads. An
    /// unfinished upgrade changes nothing else in this report, so a caller
    /// that watches only the capacity sees nothing until the build ends.[^3]
    ///
    /// For each kind, the stock entry is the generated entry less the taken
    /// entry. The engine computes that difference.[^4]
    ///
    /// The capacity composes the ground with the finished upgrade, which is
    /// what admission reads. The binding holds neither table.[^5]
    ///
    /// **This call reports no unit.** A count of the units on a tile comes
    /// from the derived bridge. The bridge answers only after a step. A
    /// reader of the ground should not be refused because the population
    /// moved. Ask `window_census` for the units.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    /// [^2]: Findings register, FND-011. `docs/FINDINGS.md`
    /// [^3]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decisions D2 and D3. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    /// [^4]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    /// [^5]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
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
        let site = world.upgrade_at(address);
        match site {
            Some(site) => report.set_item("upgrade", site.kind.to_u8())?,
            None => report.set_item("upgrade", python.None())?,
        }
        report.set_item("upgrade_progress", site.map_or(0, |site| site.progress.0))?;
        report.set_item(
            "upgrade_complete",
            site.is_some_and(cachette_core::upgrade::UpgradeSite::is_complete),
        )?;
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
    /// The engine walks the window and answers once. A caller that walked
    /// the addresses itself would loop over the world from the control
    /// plane. This boundary does not permit that.[^1]
    ///
    /// **The cost follows the radius and never the world.** The engine
    /// refuses a radius above 64.
    ///
    /// The unit counts come from the derived unit-to-tile bridge, which
    /// rebuilds at the barrier. The engine refuses a caller that changed the
    /// population and did not step. It does not answer from a stale bridge.[^2]
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
    /// faction the world has. A founding must keep its distance from the
    /// foundings before it. A caller that founded one faction at a time would
    /// carry that state across the boundary itself.
    ///
    /// The binding keeps the report. The frame marks each founded place, and
    /// the panel names each refusal. A founded place is history, and the
    /// engine holds no copy of it.
    ///
    /// The group is how many people to seat for each faction. It defaults to
    /// 64.
    ///
    /// Returns a `list` with one `dict` for each faction of the world, in
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
        founding_reports(self, python, outcomes, group)
    }

    /// Seeds the world from its seed: founds one run for every faction and
    /// places the luxuries. Takes nothing.
    ///
    /// **A caller that builds a world from a seed calls this once and names
    /// no group and no place.** The founding group and the deposit count are
    /// values in the balance register, and the engine holds them.[^1] The
    /// founding is the one `found_run_for_every_faction` makes with the
    /// default group, and the luxuries are placed by a keyed draw on the
    /// deposit index, so two worlds with one seed seed alike.[^2]
    ///
    /// Returns the same `list` of founding reports that
    /// `found_run_for_every_faction` returns, and keeps the report for the
    /// panel in the same way.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the world was seeded before, or when no
    /// faction was seated.
    ///
    /// # References
    ///
    /// [^1]: Balance register, the founding group and the luxury deposits. `docs/reference/balance.md`
    /// [^2]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
    fn seed_world<'py>(&self, python: Python<'py>) -> PyResult<Vec<Bound<'py, PyDict>>> {
        let outcomes = {
            let mut world = self.lock();
            world
                .seed_world()
                .map_err(|error| VerbError::new_err(error.to_string()))?
        };
        founding_reports(
            self,
            python,
            outcomes,
            cachette_core::FOUNDING_GROUP_DEFAULT,
        )
    }

    /// Returns the weight vector of one faction, as a `dict`.
    ///
    /// The faction is a number. The keys are `war`, `trade`, `build` and
    /// `renown`, and every value is a whole number inside the range the
    /// balance register holds.[^1] The vector is drawn from the seed when the
    /// world is built, so two worlds with one seed hold one vector. Only the
    /// build weight is read today: it biases the controller toward a build
    /// order over a gather order.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no faction of this world.
    ///
    /// # References
    ///
    /// [^1]: Balance register, the weight vector range. `docs/reference/balance.md`
    fn faction_weights<'py>(
        &self,
        python: Python<'py>,
        faction: u16,
    ) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let weights = world.faction_weights(FactionId(faction)).ok_or_else(|| {
            VerbError::new_err(format!("{faction} names no faction of this world"))
        })?;
        let report = PyDict::new(python);
        report.set_item("war", weights.war)?;
        report.set_item("trade", weights.trade)?;
        report.set_item("build", weights.build)?;
        report.set_item("renown", weights.renown)?;
        Ok(report)
    }

    /// Says whether an external caller controls a faction.
    ///
    /// **A faction under external control receives no evaluation from the
    /// controller.** The flag is off for every faction of a new world, and
    /// nothing in the engine sets it. It exists so that a later player hook
    /// has a place to stand, and so that a test can prove the controller
    /// leaves such a faction alone.[^1]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no faction of this world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D6. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    fn set_externally_controlled(&self, faction: u16, controlled: bool) -> PyResult<()> {
        let mut world = self.lock();
        if !world.set_externally_controlled(FactionId(faction), controlled) {
            return Err(VerbError::new_err(format!(
                "{faction} names no faction of this world"
            )));
        }
        Ok(())
    }

    /// Returns whether an external caller controls a faction, as a `bool`.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no faction of this world.
    fn is_externally_controlled(&self, faction: u16) -> PyResult<bool> {
        self.lock()
            .is_externally_controlled(FactionId(faction))
            .ok_or_else(|| VerbError::new_err(format!("{faction} names no faction of this world")))
    }

    /// The number of evaluations the controller makes for one faction on one
    /// tick, as an integer.
    ///
    /// The count is a value in the balance register.[^1]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the controller evaluations per faction per tick. `docs/reference/balance.md`
    #[getter]
    fn controller_evaluations(&self) -> u32 {
        self.lock().controller_evaluations()
    }

    /// Sets how many evaluations the controller makes for one faction on one
    /// tick.
    ///
    /// Zero silences the controller. The count is state that every tick
    /// reads, so two worlds that differ in it hash differently.
    fn set_controller_evaluations(&self, evaluations: u32) {
        self.lock().set_controller_evaluations(evaluations);
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

    /// Returns the game end record, as a `dict`, or `None` while no game has
    /// ended.
    ///
    /// The keys are `winner`, an integer naming the faction; `path`, a `str`
    /// naming the way it won, which is `territory` today; and `tick`, the
    /// tick the reader fired on. **The record is written once.** After it
    /// the controller emits nothing and every other pass continues, so the
    /// world keeps stepping and the picture keeps moving.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0148, a game end is recorded once and stops the controllers, decisions D2 and D4. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
    fn game_end<'py>(&self, python: Python<'py>) -> PyResult<Option<Bound<'py, PyDict>>> {
        let end = self.lock().game_end();
        let Some(path) = end.win_path() else {
            return Ok(None);
        };
        let report = PyDict::new(python);
        report.set_item("winner", end.winner.0)?;
        report.set_item("path", path.name())?;
        report.set_item("tick", end.tick.0)?;
        Ok(Some(report))
    }

    /// Returns the score of one faction on the territory path, as an
    /// integer: the tiles it holds.
    ///
    /// The count is the running total the engine keeps, so this starts no
    /// pass.[^1]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no faction of this world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    fn score(&self, faction: u16) -> PyResult<i64> {
        self.lock()
            .score(FactionId(faction))
            .ok_or_else(|| VerbError::new_err(format!("{faction} names no faction of this world")))
    }

    /// Returns what one faction feels toward another, as an integer.
    ///
    /// The relation is one signed whole number for each ordered pair. The
    /// entry for `(a, b)` is what `a` feels toward `b`, and `(b, a)` is a
    /// separate entry. A new world holds every pair at the peace edge. The
    /// edges that cut the range into bands are rows of the balance
    /// register.[^1] [^2]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when a number names no faction of this world, or
    /// when the two numbers name one faction.
    ///
    /// # References
    ///
    /// [^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decisions D1 and D2. `docs/adrs/draft/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    /// [^2]: Balance register, the relation. `docs/reference/balance.md`
    fn relation(&self, from_faction: u16, to_faction: u16) -> PyResult<i32> {
        if from_faction == to_faction {
            return Err(VerbError::new_err(
                "a faction holds no relation toward itself",
            ));
        }
        self.lock()
            .relation(FactionId(from_faction), FactionId(to_faction))
            .ok_or_else(|| VerbError::new_err("a number names no faction of this world"))
    }

    /// Returns the band of what one faction feels toward another, as an
    /// integer.
    ///
    /// The number counts the edges at or below the value. Zero is below the
    /// war edge, one is at or above it and below the peace edge, two is at or
    /// above the peace edge and below the alliance edge, and three is at or
    /// above the alliance edge. The engine holds no band name.[^1]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when a number names no faction of this world, or
    /// when the two numbers name one faction.
    ///
    /// # References
    ///
    /// [^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D2. `docs/adrs/draft/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    fn relation_band(&self, from_faction: u16, to_faction: u16) -> PyResult<u8> {
        if from_faction == to_faction {
            return Err(VerbError::new_err(
                "a faction holds no relation toward itself",
            ));
        }
        self.lock()
            .relation_band(FactionId(from_faction), FactionId(to_faction))
            .ok_or_else(|| VerbError::new_err("a number names no faction of this world"))
    }

    /// Writes what one faction feels toward another, outright.
    ///
    /// **This is the caller's own path and it holds no gate.** A god sets the
    /// relation a scenario starts from. A crossing of the war edge is logged
    /// as any other cause logs it. Returns `None`.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when a number names no faction of this world, or
    /// when the two numbers name one faction.
    fn set_relation(&self, from_faction: u16, to_faction: u16, value: i32) -> PyResult<()> {
        if !self
            .lock()
            .set_relation(FactionId(from_faction), FactionId(to_faction), value)
        {
            return Err(VerbError::new_err(
                "a number names no faction of this world, or the two numbers name one faction",
            ));
        }
        Ok(())
    }

    /// Moves what the faction of a speaker unit feels toward another faction
    /// by a bounded step, and returns the value after the move.
    ///
    /// **The verb refuses a speaker whose type has a command reach of
    /// zero.** The gate reads the type column of the unit and no per-faction
    /// flag.[^1] The step is bounded in either direction, and the bound is a
    /// row of the balance register.[^2] A leader may always declare, so the
    /// verb reads no band before it moves.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no live soldier. Raises
    /// `VerbError` when the faction number names no faction, when it names
    /// the speaker's own faction, when the speaker's type has no command
    /// reach, and when the step is above the bound.
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D3. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    /// [^2]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D5. `docs/adrs/draft/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    fn move_relation(&self, speaker: u64, faction: u16, step: i32) -> PyResult<i32> {
        let mut world = self.lock();
        let entity = resolve(&world, speaker)?;
        world
            .move_relation(entity, FactionId(faction), step)
            .map_err(|error| VerbError::new_err(error.to_string()))
    }

    /// Returns the crossings of the war edge on the last step, as columns.
    ///
    /// The result is a `dict` of one-dimensional NumPy arrays. Every array
    /// holds one entry for each ordered pair whose relation crossed the war
    /// edge on the last step, in the order the crossings happened. The keys
    /// are:
    ///
    /// - `tick`, `numpy.uint64`. The step it happened at.
    /// - `from_faction`, `numpy.uint16`. The faction whose feeling moved.
    /// - `to_faction`, `numpy.uint16`. The faction it feels toward.
    /// - `band_before`, `numpy.uint8`. The band number before the move.
    /// - `band_after`, `numpy.uint8`. The band number after the move. A
    ///   value below `band_before` is a declaration, and a value above it is
    ///   a peace.
    ///
    /// The log covers the last step alone. This method copies each
    /// column.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D6. `docs/adrs/draft/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    /// [^2]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    fn relation_log_columns<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let log = world.relation_log();
        let columns = PyDict::new(python);
        let tick: Vec<u64> = log.iter().map(|event| event.tick.0).collect();
        let from_faction: Vec<u16> = log.iter().map(|event| event.from_faction.0).collect();
        let to_faction: Vec<u16> = log.iter().map(|event| event.to_faction.0).collect();
        let band_before: Vec<u8> = log.iter().map(|event| event.band_before).collect();
        let band_after: Vec<u8> = log.iter().map(|event| event.band_after).collect();
        columns.set_item("tick", tick.to_pyarray(python))?;
        columns.set_item("from_faction", from_faction.to_pyarray(python))?;
        columns.set_item("to_faction", to_faction.to_pyarray(python))?;
        columns.set_item("band_before", band_before.to_pyarray(python))?;
        columns.set_item("band_after", band_after.to_pyarray(python))?;
        Ok(columns)
    }

    /// The number of relations that crossed the war edge in the last step,
    /// as an integer. Read the events themselves with
    /// `relation_log_columns`.
    #[getter]
    fn relation_crossed_count(&self) -> usize {
        self.lock().relation_log().len()
    }

    /// Returns the subsystem census, as a `dict` from a subsystem name to a
    /// count.
    ///
    /// **One Rust table declares the list.** Each row names a subsystem and
    /// the reader that counts what it produced, and this call walks that
    /// table. Nothing else declares the names, so a name here is a name the
    /// engine holds.[^1] The counts of the controller are counts of the last
    /// step.
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
    fn subsystem_census<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let census = self.lock().subsystem_census();
        let report = PyDict::new(python);
        for (name, count) in census {
            report.set_item(name, count)?;
        }
        Ok(report)
    }

    /// Fills the caller's pixels with one frame of this world.
    ///
    /// **The caller owns the memory before this call and owns it
    /// afterwards.** The engine writes each pixel of one frame into it and
    /// returns. It allocates no frame, keeps no frame, and holds no reference
    /// to the memory after the call ends.[^1]
    ///
    /// **This is one command and it carries no entity.** It takes a world, a
    /// camera and somewhere to put the result. It names no tile and no unit.
    /// A caller that walked tiles to draw them would cross the boundary once
    /// for each tile. The crossing costs more than the drawing.[^2]
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
    /// that writes a picture to a file wants. Set `panels` to draw the named
    /// panels as a deck beside the cards. A name that no panel carries is
    /// refused, and the message names the panels that exist. When both are
    /// set, the named panels win. Set `pointer` to an address, and the
    /// inspector panel of the deck reads that tile. It applies only when
    /// `panels` names a deck.
    ///
    /// Returns a `dict` of what the drawing pass read.[^3] A caller reports
    /// the numbers the picture was made from, and starts no second pass to
    /// find them.
    ///
    /// Most entries are plain integers. Eight are not. `promoted_deeds` may
    /// be `None`. `newest_character` is a pair of integers, or `None`.
    /// `centre` and `extent_shown` are pairs of integers. `carried_by_kind`
    /// is a list of one count for each resource kind. The three trailing
    /// entries are floating point numbers.
    ///
    /// **The three floating point entries measure this machine and not the
    /// simulation.** `step_mean_micros` and `draw_mean_micros` are mean
    /// durations in microseconds, and `ticks_each_second` is a rate. Nothing
    /// in the engine reads them, and no two runs need to agree on them.
    ///
    /// **`rationed_short_accum` is a Q16.16 value as its raw integer.**
    /// Divide by 65536. The key names the unit, because a caller that read it
    /// as a count of goods would report a quantity 65536 times too large.
    /// Every other integer entry is a whole count.
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
    /// the bound. Below one pixel per tile, a second tile falls on a pixel
    /// the first already holds. The work is provably invisible, and a caller
    /// could otherwise sweep the whole world for a picture of a few pixels.[^4]
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
    /// sequence of `(q, r)` pairs of integers. They are the places the caller
    /// wants the units at. The destination is the number of the destination
    /// plane that carries the order. The default is 0. Returns `None`.
    ///
    /// **One call names a whole set and the engine builds one field.** The
    /// engine takes the level 1 cell of each seed. It seeds the plane at all
    /// of them at once. It spreads a reach outward. Two seeds in one cell act
    /// as one, because the engine removes duplicate cells. Every unit the
    /// call names then reads one entry of the plane on each step. It takes
    /// one step along that direction. The cost of the field follows the cell
    /// count, and not the number of units. Sending a million units costs what
    /// sending one costs.[^1]
    ///
    /// **No unit searches for a route.** A unit reads the entry of its own
    /// cell. It reads no neighbouring cell. It computes nothing from its own
    /// address toward a seed. That is the rule the engine is built on. This
    /// call does not bend it.[^2]
    ///
    /// **A cell steers a whole block, so two units in one cell take one
    /// direction.** A caller cannot send half a cell one way and half the
    /// other.[^2]
    ///
    /// **A unit that cannot reach the seeds does not freeze.** A unit whose
    /// cell holds no direction takes a keyed draw instead. The draw is keyed
    /// on the frame, so the unit takes a different direction on the next
    /// frame. The same holds for a unit that arrived. It also holds for a
    /// unit whose ground refuses the direction the field gave it.[^3]
    ///
    /// **The call sends a set toward a place. It does not promise that the set
    /// arrives.** A cell steers a block of tiles. The water in front of one
    /// unit of that block is not a fact the block carries. A unit behind such
    /// a barrier walks to it, and then wanders beside it. It is not frozen. It
    /// does not get past.[^4]
    ///
    /// The order holds until the caller stops it with `stop_sending`. A unit
    /// that arrives keeps the order. It walks about inside the block it
    /// arrived in. Read `faction_units` for where the set is now.
    ///
    /// A caller that names a destination again replaces the seed set of that
    /// destination. Every unit already sent to it walks to the new one. Read
    /// `destination_count` for how many the world holds. Set it with
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
    /// it sends a set of units somewhere. The numbers run from zero to one
    /// below this.
    #[getter]
    fn destination_count(&self) -> u16 {
        self.lock().destination_count()
    }

    /// Sets the number of destination planes the world holds.
    ///
    /// The count says how many places the control plane may send units to at
    /// one time. The next place re-aims a plane the control plane already
    /// used. **The caller names the plane, and the engine allocates none.**[^1]
    ///
    /// The call clears the seed set of every plane. No order steers anything
    /// until the caller sends a set again. A unit that was sent to a plane the
    /// world no longer holds reads no direction. It takes a keyed draw rather
    /// than standing still.[^2]
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

    /// Changes the faction of every soldier the identities name.
    ///
    /// The units are a sequence of identities, or the NumPy array of
    /// `numpy.uint64` that `spawn_soldiers` or `faction_units` returned. The
    /// faction is the number of the faction that the units join. The number
    /// runs from zero to one below the faction count of the world. Returns
    /// `None`.
    ///
    /// A unit that changes faction keeps its identity. Every identity the
    /// caller holds still names the same unit. It keeps its type, the load it
    /// carries, the tile it stands on and the site it lives in. It loses its
    /// gather order, its build order and its destination. An order is an
    /// instruction from the faction that no longer holds it. A unit that
    /// carries a character takes that character with it.[^1]
    ///
    /// A unit that already belongs to the faction is left alone. Calling this
    /// twice with one set therefore has the same result as calling it once.
    ///
    /// **The set is all or nothing.** Every identity resolves, and the faction
    /// is checked, before anything changes.
    ///
    /// **This is the deliberate route.** The engine also converts a unit on
    /// its own. That happens where another faction reaches the unit's place
    /// more strongly than its own faction does. Set that reach with
    /// `set_influence_source`, and read the result with
    /// `converted_log_columns`.[^2]
    ///
    /// ```python
    /// mine = world.faction_units(0)
    /// world.convert_units(mine["unit"], 1)
    /// ```
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live soldier. Raises
    /// `VerbError` when the number names no faction of this world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0132, conversion changes the faction of a unit and adds no second allegiance, decisions D2, D3 and D4. `docs/adrs/draft/adr-0132-conversion-changes-the-faction-of-a-unit.md`
    /// [^2]: ADR-0133, a unit converts to the faction that leads the influence field at its cell, decisions D1 and D4. `docs/adrs/draft/adr-0133-a-unit-converts-to-the-faction-that-leads-the-field.md`
    fn convert_units(&self, units: Vec<u64>, faction: u16) -> PyResult<()> {
        let mut world = self.lock();
        let mut resolved = Vec::with_capacity(units.len());
        for unit in &units {
            resolved.push(resolve(&world, *unit)?);
        }
        world
            .convert_units(&resolved, FactionId(faction))
            .map_err(|error| VerbError::new_err(error.to_string()))
    }

    /// Returns the units that changed faction in the last step, as columns.
    ///
    /// The result is a `dict` of one-dimensional NumPy arrays. Every array
    /// holds one entry for each unit that changed faction. The keys are:
    ///
    /// - `tick`, `numpy.uint64`. The step it happened at.
    /// - `unit`, `numpy.uint64`. The identity of the unit. It is the identity
    ///   the unit had before, because a unit that changes faction keeps its
    ///   identity. Hand it back to `soldier_tile` or to `convert_units`.
    /// - `tile`, `numpy.uint32`. The tile the unit stood on, as a row-major
    ///   index. Take `index % world.width` for the column and
    ///   `index // world.width` for the row.
    /// - `from_faction`, `numpy.uint16`. The faction that lost the unit.
    /// - `to_faction`, `numpy.uint16`. The faction that gained it.
    ///
    /// **The log covers the last step alone.** The engine delivers it at the
    /// frame barrier.[^1] Read it after each `step`. The next step clears it.
    ///
    /// The log holds the units the engine converted and the units that
    /// `convert_units` converted. Both are the same change, so one log
    /// reports both.
    ///
    /// A step in which nobody changed faction gives arrays of length zero.
    ///
    /// This method copies each column.[^2]
    ///
    /// ```python
    /// world.step(4)
    /// changed = world.converted_log_columns()
    /// gained = int((changed["to_faction"] == 0).sum())
    /// ```
    ///
    /// # References
    ///
    /// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D2. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
    /// [^2]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    fn converted_log_columns<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let log = world.converted_log();
        let columns = PyDict::new(python);
        let tick: Vec<u64> = log.iter().map(|event| event.tick.0).collect();
        let unit: Vec<u64> = log.iter().map(|event| event.unit).collect();
        let tile: Vec<u32> = log.iter().map(|event| event.tile.0).collect();
        let from: Vec<u16> = log.iter().map(|event| event.from.0).collect();
        let to: Vec<u16> = log.iter().map(|event| event.to.0).collect();
        columns.set_item("tick", tick.to_pyarray(python))?;
        columns.set_item("unit", unit.to_pyarray(python))?;
        columns.set_item("tile", tile.to_pyarray(python))?;
        columns.set_item("from_faction", from.to_pyarray(python))?;
        columns.set_item("to_faction", to.to_pyarray(python))?;
        Ok(columns)
    }

    /// The number of units that changed faction in the last step, as an
    /// integer.
    ///
    /// The count covers the last step alone. Read the changes themselves with
    /// `converted_log_columns`.
    #[getter]
    fn converted_count(&self) -> usize {
        self.lock().converted_log().len()
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
    /// - `wet_cells`, an integer. How many level 1 cells hold at least
    ///   `weather_wet_mark` drops on the ground.
    ///
    /// **The account is exact.** The sum of `air`, `ground` and `evaporated`
    /// equals `raised` at every moment. A pass moves water and never scales
    /// it.[^1]
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

    /// The water on the ground of every level 1 cell, as a NumPy array.
    ///
    /// The result is a one-dimensional array of `numpy.int64`, in cell index
    /// order, and the unit is drops. A drop is a whole number and it is not a
    /// fixed-point value. Take `index % cells_wide` for the column of a cell
    /// and `index // cells_wide` for its row.
    ///
    /// **This is one crossing, and it replaces a loop.** A watcher that read
    /// each cell through `ground_water_at` would pay one crossing for each
    /// cell. The control plane never loops over the world.
    ///
    /// The array is empty when no water has entered the world yet.
    fn weather_ground<'py>(&self, python: Python<'py>) -> Bound<'py, PyArray1<i64>> {
        let world = self.lock();
        let plane: Vec<i64> = world
            .weather()
            .ground_plane()
            .iter()
            .map(|drops| drops.0)
            .collect();
        plane.to_pyarray(python)
    }

    /// The number of level 1 cells across the world, as an integer.
    ///
    /// A weather array is in cell index order, so a watcher takes
    /// `index % cells_wide` for the column of a cell and
    /// `index // cells_wide` for its row.
    #[getter]
    fn cells_wide(&self) -> u32 {
        self.lock().pyramid().layout().blocks_wide()
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

    /// Returns the live soldiers of one faction, as columns.
    ///
    /// The argument is the number of a faction. The result is a `dict` of
    /// one-dimensional NumPy arrays. Every array holds one entry for each
    /// live soldier of that faction. The keys are:
    ///
    /// - `unit`, `numpy.uint64`. The identity of the soldier. Pass the whole
    ///   array to `send_units_to`, `order_gather` or `despawn_soldiers`.
    /// - `tile`, `numpy.uint32`. The tile it stands on, as a row-major index.
    ///   Take `index % world.width` for the column and `index // world.width`
    ///   for the row.
    ///
    /// **This is one crossing, and it replaces a loop.** A caller that reads
    /// one unit through `soldier_tile` pays one crossing for that unit. The
    /// control plane never loops over the population.[^1] [^2]
    ///
    /// **Every entry names a live soldier, so no entry stands for nothing.**
    /// The engine builds the set at the moment of the call. It takes no
    /// identity from the caller, so nothing here can be stale. The result
    /// needs no validity mask. The singular read takes an identity, and it
    /// refuses a dead one.[^3]
    ///
    /// The order is the slot order of the arena. It is the same on every run
    /// and at every thread count. It is never a thread completion
    /// order.[^4] It is not the spawn order. A slot returns to the arena when
    /// a soldier dies, and the next soldier takes it.
    ///
    /// A faction with nobody in it gives two empty arrays. That is an answer
    /// and not an error. A number that names no faction of this world gives
    /// the same answer, because no soldier holds it.
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

    /// Opens a trade negotiation from one faction toward another.
    ///
    /// A trade has two halves. This half is the conversation. The other half
    /// is the contract that an acceptance makes, and the engine enforces that
    /// one.
    ///
    /// The proposer and the responder are faction numbers. The pair is
    /// ordered, and the row the engine writes belongs to the proposer and the
    /// responder in that order.
    ///
    /// The give side is what the proposer owes. The take side is what the
    /// responder owes. Each is a whole quantity of one resource kind: food is
    /// zero, wood is one and stone is two. No term of a contract is a
    /// fractional number.
    ///
    /// The term is how many steps the contract runs for once it binds. The
    /// acceptance turns it into a deadline. A contract that cannot fail is not
    /// a contract, so a term of zero is refused.
    ///
    /// **One unit of the proposer must stand on ground that the responder
    /// holds.** This is the same rule that governs a message between two
    /// players. A trade is a thing two players say to each other.[^1]
    ///
    /// The call gives the offer. It moves nothing. Read `trade_status` for
    /// what the pair now holds.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when a number names no faction of this world, when
    /// the two parties are one faction, when a kind names no resource, when a
    /// quantity is zero, when the term is zero, when the pair already holds a
    /// live negotiation or a live contract, when a terminal refusal closed
    /// this direction, or when no unit of the proposer stands on the
    /// responder's ground. The message says which, and a closure message
    /// states the step that opens the direction again.
    ///
    /// # References
    ///
    /// **Each side is a tagged consideration.** The tag is zero for a
    /// resource, one for land and two for a relation step. Both tags are zero
    /// when the call names none, so every call that states two resources
    /// keeps working.
    ///
    /// A land side names `give_cell` or `take_cell`, an address `(q, r)` whose
    /// level 1 cell is the set, or `give_tiles` or `take_tiles`, a list of
    /// addresses, or both. The kind and the amount of a land side are ignored,
    /// because the amount is the tile count. Every tile must be held by the
    /// party that owes it, and no tile may carry an upgrade while the question
    /// of what happens to the upgrade is open.[^2]
    ///
    /// A relation side keeps its kind and its amount. It is stored, and it
    /// delivers as a logged no-op until the relation matrix exists.
    ///
    /// # References
    ///
    /// [^1]: ADR-0126, a trade negotiation is engine state and the words are not, decision D3. `docs/adrs/draft/adr-0126-a-trade-negotiation-is-engine-state.md`
    /// [^2]: Blockers register, BLK-036. `docs/BLOCKERS.md`
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (
        proposer,
        responder,
        give_kind,
        give_amount,
        take_kind,
        take_amount,
        term,
        *,
        give_tag = 0,
        take_tag = 0,
        give_tiles = None,
        take_tiles = None,
        give_cell = None,
        take_cell = None,
    ))]
    fn offer_trade(
        &self,
        proposer: u16,
        responder: u16,
        give_kind: u8,
        give_amount: u32,
        take_kind: u8,
        take_amount: u32,
        term: u32,
        give_tag: u8,
        take_tag: u8,
        give_tiles: Option<Vec<(i32, i32)>>,
        take_tiles: Option<Vec<(i32, i32)>>,
        give_cell: Option<(i32, i32)>,
        take_cell: Option<(i32, i32)>,
    ) -> PyResult<()> {
        let mut world = self.lock();
        let give = consideration_of(
            &world,
            give_tag,
            give_kind,
            give_amount,
            give_tiles,
            give_cell,
        )?;
        let take = consideration_of(
            &world,
            take_tag,
            take_kind,
            take_amount,
            take_tiles,
            take_cell,
        )?;
        world
            .offer_consideration(FactionId(proposer), FactionId(responder), give, take, term)
            .map_err(trade_refusal)
    }

    /// Restates the terms of a live negotiation.
    ///
    /// The speaker is the party that did not speak last. The `turn` entry of
    /// `trade_status` names the faction that may speak now.
    ///
    /// The terms are always stated in the orientation of the row. The give
    /// side is what the party that opened the pair owes, whoever speaks now.
    ///
    /// **One unit of the speaker must stand on ground that the other party
    /// holds.**
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when a number names no faction of this world, when
    /// the two parties are one faction, when the pair holds no live
    /// negotiation, when the terms already bind both parties, when the other
    /// party has not answered yet, when a kind names no resource, when a
    /// quantity is zero, or when the speaker has no unit on the other party's
    /// ground.
    ///
    /// The tag and the target keyword arguments are those of `offer_trade`.
    /// A land side on the give side is checked against the party that opened
    /// the pair, and one on the take side against the other party.
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (
        speaker,
        other,
        give_kind,
        give_amount,
        take_kind,
        take_amount,
        *,
        give_tag = 0,
        take_tag = 0,
        give_tiles = None,
        take_tiles = None,
        give_cell = None,
        take_cell = None,
    ))]
    fn counter_trade(
        &self,
        speaker: u16,
        other: u16,
        give_kind: u8,
        give_amount: u32,
        take_kind: u8,
        take_amount: u32,
        give_tag: u8,
        take_tag: u8,
        give_tiles: Option<Vec<(i32, i32)>>,
        take_tiles: Option<Vec<(i32, i32)>>,
        give_cell: Option<(i32, i32)>,
        take_cell: Option<(i32, i32)>,
    ) -> PyResult<()> {
        let mut world = self.lock();
        let give = consideration_of(
            &world,
            give_tag,
            give_kind,
            give_amount,
            give_tiles,
            give_cell,
        )?;
        let take = consideration_of(
            &world,
            take_tag,
            take_kind,
            take_amount,
            take_tiles,
            take_cell,
        )?;
        world
            .counter_consideration(FactionId(speaker), FactionId(other), give, take)
            .map_err(trade_refusal)
    }

    /// Agrees to the terms of a live negotiation.
    ///
    /// The terms then bind both parties, and the engine enforces them. The
    /// deadline is the current step plus the term the offer named. A counter
    /// restates the quantities, and it never restates the term.
    ///
    /// The speaker is the party that did not speak last.
    ///
    /// **One unit of the speaker must stand on ground that the other party
    /// holds.**
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when a number names no faction of this world, when
    /// the two parties are one faction, when the pair holds no live
    /// negotiation, when the terms already bind both parties, when the other
    /// party has not answered yet, or when the speaker has no unit on the
    /// other party's ground.
    fn accept_trade(&self, speaker: u16, other: u16) -> PyResult<()> {
        let mut world = self.lock();
        world
            .accept_trade(FactionId(speaker), FactionId(other))
            .map_err(trade_refusal)
    }

    /// Declines the terms of a live negotiation.
    ///
    /// **This is a refusal and not a closed door.** The pair is idle after
    /// it, and either party may open a new negotiation at once. Call
    /// `close_trade` for the terminal refusal that stops the other party from
    /// asking again.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when a number names no faction of this world, when
    /// the two parties are one faction, when the pair holds no live
    /// negotiation, when the terms already bind both parties, when the other
    /// party has not answered yet, or when the speaker has no unit on the
    /// other party's ground.
    fn refuse_trade(&self, speaker: u16, other: u16) -> PyResult<()> {
        let mut world = self.lock();
        world
            .refuse_trade(FactionId(speaker), FactionId(other))
            .map_err(trade_refusal)
    }

    /// Declines the terms and stops the other party from asking again.
    ///
    /// **This is the terminal refusal, and it differs from `refuse_trade`.**
    /// It ends the negotiation, and it also closes the direction the other
    /// party would open, for the number of steps named here.
    ///
    /// The closure is directional. The other party cannot open a negotiation
    /// toward the speaker until the closure ends. The speaker may still open
    /// one toward the other party. The speaker closed its own door, and it
    /// promised no silence of its own.
    ///
    /// **The step that opens the direction again is readable.** Read
    /// `trade_status` with the other party first and the speaker second, and
    /// take the `closed_until` entry. An offer made before that step raises
    /// an error whose message states the step. A player that cannot tell a
    /// refusal from a closed door asks for ever.
    ///
    /// Only the speaker opens the direction early, through `reopen_trade`.
    /// Nothing the other party does shortens the closure.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when a number names no faction of this world, when
    /// the two parties are one faction, when the number of steps is zero,
    /// when the pair holds no live negotiation, when the terms already bind
    /// both parties, when the other party has not answered yet, or when the
    /// speaker has no unit on the other party's ground.
    fn close_trade(&self, speaker: u16, other: u16, steps: u32) -> PyResult<()> {
        let mut world = self.lock();
        world
            .close_trade(FactionId(speaker), FactionId(other), steps)
            .map_err(trade_refusal)
    }

    /// Opens a direction that this faction closed, before the closure ends.
    ///
    /// Only the faction that closed the direction opens it again.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when a number names no faction of this world, when
    /// the two parties are one faction, or when this faction closed nothing
    /// toward the other party.
    fn reopen_trade(&self, speaker: u16, other: u16) -> PyResult<()> {
        let mut world = self.lock();
        world
            .reopen_trade(FactionId(speaker), FactionId(other))
            .map_err(trade_refusal)
    }

    /// Returns what stands between one ordered pair of factions.
    ///
    /// The pair is ordered. The row belongs to the proposer and the responder
    /// in that order, and it holds the negotiation that the proposer opened
    /// toward the responder. A pair that nobody ever spoke about answers an
    /// idle row. Every entry of it is zero. The one exception is
    /// `closed_until`, and it holds the step that opens a direction a
    /// terminal refusal closed.
    ///
    /// The dictionary holds these entries.
    ///
    /// - `status`, an integer. Zero is idle, one means the proposer spoke
    ///   last, two means the responder spoke last, three means a contract
    ///   binds both, four means both delivered in full, and five means the
    ///   deadline passed with a debt.
    /// - `turn`, an integer or `None`. Which faction answers next. It is
    ///   `None` when nobody is waiting on an answer.
    /// - `give_tag` and `take_tag`. The kind of each side: zero a resource,
    ///   one land, two a relation step.
    /// - `give_kind` and `give_amount`. What the proposer owes. For land the
    ///   amount is the tile count.
    /// - `take_kind` and `take_amount`. What the responder owes.
    /// - `give_tiles` and `take_tiles`, `numpy.uint32`. The tile indices of a
    ///   land side, ascending. Empty for the other kinds.
    /// - `given` and `taken`. What each party has already delivered.
    /// - `opened`, the step the negotiation opened at.
    /// - `deadline`, the step a bound contract fails at. Zero until it binds.
    /// - `term`, how many steps a bound contract runs for.
    /// - `closed_until`, the step at which this direction opens again. Zero
    ///   when nothing closed it. **This is how a caller tells a refusal from a
    ///   closed door.**
    /// - `rounds`, how many times somebody spoke.
    ///
    /// **The engine answers for any pair, and it holds no notion of who is
    /// asking.** A game that keeps a negotiation private between its two
    /// parties enforces that in the control plane.[^1]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when a number names no faction of this world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0126, a trade negotiation is engine state and the words are not, decision D5. `docs/adrs/draft/adr-0126-a-trade-negotiation-is-engine-state.md`
    fn trade_status<'py>(
        &self,
        python: Python<'py>,
        proposer: u16,
        responder: u16,
    ) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let row = world
            .trade_row(FactionId(proposer), FactionId(responder))
            .ok_or_else(|| {
                ViewError::new_err(format!(
                    "the pair {proposer} and {responder} names no faction of this world"
                ))
            })?;
        let turn: Option<u16> = match row.status {
            cachette_core::TRADE_OFFERED => Some(responder),
            cachette_core::TRADE_COUNTERED => Some(proposer),
            _ => None,
        };
        let entries = PyDict::new(python);
        entries.set_item("status", row.status)?;
        entries.set_item("turn", turn)?;
        entries.set_item("give_tag", row.give_tag)?;
        entries.set_item("take_tag", row.take_tag)?;
        let give_tiles: Vec<u32> = world
            .trade_land(FactionId(proposer), FactionId(responder), false)
            .iter()
            .map(|tile| tile.0)
            .collect();
        let take_tiles: Vec<u32> = world
            .trade_land(FactionId(proposer), FactionId(responder), true)
            .iter()
            .map(|tile| tile.0)
            .collect();
        entries.set_item("give_tiles", give_tiles.to_pyarray(python))?;
        entries.set_item("take_tiles", take_tiles.to_pyarray(python))?;
        entries.set_item("give_kind", row.give_kind)?;
        entries.set_item("give_amount", row.give_amount)?;
        entries.set_item("take_kind", row.take_kind)?;
        entries.set_item("take_amount", row.take_amount)?;
        entries.set_item("given", row.given)?;
        entries.set_item("taken", row.taken)?;
        entries.set_item("opened", row.opened.0)?;
        entries.set_item("deadline", row.deadline.0)?;
        entries.set_item("term", row.term)?;
        entries.set_item("closed_until", row.closed_until.0)?;
        entries.set_item("rounds", row.rounds)?;
        Ok(entries)
    }

    /// Returns every pair the faction forms with another faction, as columns.
    ///
    /// This is the read a player uses to decide. It crosses once and it holds
    /// no loop over pairs in Python. Every array has one entry for each pair
    /// the faction forms with another faction, and every array is the same
    /// length.
    ///
    /// - `proposer` and `responder`, `numpy.uint16`. The ordered pair.
    /// - `status`, `numpy.uint8`. The same numbering `trade_status` states.
    /// - `give_tag`, `take_tag`, `numpy.uint8`. The kind of each side.
    /// - `give_kind`, `take_kind`, `numpy.uint8`.
    /// - `give_amount`, `take_amount`, `given`, `taken`, `term`,
    ///   `numpy.uint32`.
    /// - `opened`, `deadline`, `closed_until`, `numpy.uint64`.
    /// - `rounds`, `numpy.uint8`.
    ///
    /// The book is empty until somebody speaks. A world in which nobody
    /// traded holds no row at all, so every array is empty.
    ///
    /// This method copies each column.[^1]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the number names no faction of this world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    fn trade_book<'py>(&self, python: Python<'py>, faction: u16) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        if u32::from(faction) >= u32::from(world.faction_count()) {
            return Err(ViewError::new_err(format!(
                "{faction} names no faction of this world"
            )));
        }
        let width = usize::from(world.faction_count());
        let rows = world.trade_book();
        let mut chosen: Vec<(u16, u16, cachette_core::TradeRow)> = Vec::new();
        for (index, row) in rows.iter().enumerate() {
            let proposer = (index / width.max(1)) as u16;
            let responder = (index % width.max(1)) as u16;
            if proposer != faction && responder != faction {
                continue;
            }
            if proposer == responder {
                continue;
            }
            chosen.push((proposer, responder, *row));
        }
        let columns = PyDict::new(python);
        let proposer: Vec<u16> = chosen.iter().map(|entry| entry.0).collect();
        let responder: Vec<u16> = chosen.iter().map(|entry| entry.1).collect();
        let status: Vec<u8> = chosen.iter().map(|entry| entry.2.status).collect();
        let give_tag: Vec<u8> = chosen.iter().map(|entry| entry.2.give_tag).collect();
        let take_tag: Vec<u8> = chosen.iter().map(|entry| entry.2.take_tag).collect();
        let give_kind: Vec<u8> = chosen.iter().map(|entry| entry.2.give_kind).collect();
        let take_kind: Vec<u8> = chosen.iter().map(|entry| entry.2.take_kind).collect();
        let give_amount: Vec<u32> = chosen.iter().map(|entry| entry.2.give_amount).collect();
        let take_amount: Vec<u32> = chosen.iter().map(|entry| entry.2.take_amount).collect();
        let given: Vec<u32> = chosen.iter().map(|entry| entry.2.given).collect();
        let taken: Vec<u32> = chosen.iter().map(|entry| entry.2.taken).collect();
        let term: Vec<u32> = chosen.iter().map(|entry| entry.2.term).collect();
        let opened: Vec<u64> = chosen.iter().map(|entry| entry.2.opened.0).collect();
        let deadline: Vec<u64> = chosen.iter().map(|entry| entry.2.deadline.0).collect();
        let closed_until: Vec<u64> = chosen.iter().map(|entry| entry.2.closed_until.0).collect();
        let rounds: Vec<u8> = chosen.iter().map(|entry| entry.2.rounds).collect();
        columns.set_item("proposer", proposer.to_pyarray(python))?;
        columns.set_item("responder", responder.to_pyarray(python))?;
        columns.set_item("status", status.to_pyarray(python))?;
        columns.set_item("give_tag", give_tag.to_pyarray(python))?;
        columns.set_item("take_tag", take_tag.to_pyarray(python))?;
        columns.set_item("give_kind", give_kind.to_pyarray(python))?;
        columns.set_item("take_kind", take_kind.to_pyarray(python))?;
        columns.set_item("give_amount", give_amount.to_pyarray(python))?;
        columns.set_item("take_amount", take_amount.to_pyarray(python))?;
        columns.set_item("given", given.to_pyarray(python))?;
        columns.set_item("taken", taken.to_pyarray(python))?;
        columns.set_item("term", term.to_pyarray(python))?;
        columns.set_item("opened", opened.to_pyarray(python))?;
        columns.set_item("deadline", deadline.to_pyarray(python))?;
        columns.set_item("closed_until", closed_until.to_pyarray(python))?;
        columns.set_item("rounds", rounds.to_pyarray(python))?;
        Ok(columns)
    }

    /// Returns what the last step said about trade, as columns.
    ///
    /// The log holds one entry for each thing a party said. It holds one
    /// entry for each settlement or default the step resolved. It covers the
    /// last step alone, and the engine delivers it at the frame barrier.[^1]
    ///
    /// - `tick`, `numpy.uint64`. The step it happened at.
    /// - `proposer` and `responder`, `numpy.uint16`. The ordered pair.
    /// - `act`, `numpy.uint8`. Zero is an offer, one a counteroffer, two an
    ///   acceptance, three a refusal, four a terminal refusal, five an
    ///   opening, six a settlement, seven a default, eight a land transfer
    ///   and nine a relation step.
    /// - `status`, `numpy.uint8`. What the pair held after the act.
    ///
    /// This method copies each column.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D2. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
    /// [^2]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    fn trade_log_columns<'py>(&self, python: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let log = world.trade_log();
        let columns = PyDict::new(python);
        let tick: Vec<u64> = log.iter().map(|event| event.tick.0).collect();
        let proposer: Vec<u16> = log.iter().map(|event| event.proposer).collect();
        let responder: Vec<u16> = log.iter().map(|event| event.responder).collect();
        let act: Vec<u8> = log.iter().map(|event| event.act).collect();
        let status: Vec<u8> = log.iter().map(|event| event.status).collect();
        columns.set_item("tick", tick.to_pyarray(python))?;
        columns.set_item("proposer", proposer.to_pyarray(python))?;
        columns.set_item("responder", responder.to_pyarray(python))?;
        columns.set_item("act", act.to_pyarray(python))?;
        columns.set_item("status", status.to_pyarray(python))?;
        Ok(columns)
    }

    /// Replaces the whole board of one faction.
    ///
    /// A board says what a faction offers and what it wants. Each row is a
    /// tuple `(good, quantity, wants, asking_good, asking_quantity)`. The good
    /// and the asking good are resource kinds. `wants` is zero when the
    /// faction offers the good and one when it wants the good. The quantities
    /// are whole numbers.
    ///
    /// The call replaces every row the faction had. An empty list clears the
    /// board. Posting is a statement and not a speech act, so it passes no
    /// presence gate and costs no standing.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no faction of this world, when
    /// the list holds more rows than `board_rows()`, when a good names no
    /// resource kind, or when `wants` is neither zero nor one. No row changes
    /// on an error.
    fn advertise(&self, faction: u16, rows: Vec<(u8, u32, u8, u8, u32)>) -> PyResult<()> {
        let mut world = self.lock();
        let rows: Vec<Advert> = rows
            .into_iter()
            .map(|(good, quantity, wants, asking_good, asking_quantity)| {
                Advert::new(good, quantity, wants, asking_good, asking_quantity)
            })
            .collect();
        world
            .advertise(FactionId(faction), &rows)
            .map_err(trade_refusal)
    }

    /// Returns the board of any faction, as columns.
    ///
    /// Every array has one entry for each row that says something, and every
    /// array is the same length. A faction that never advertised has an empty
    /// board, and so does a faction that posted an empty list.
    ///
    /// - `good`, `numpy.uint8`. The resource kind the row is about.
    /// - `quantity`, `numpy.uint32`. How much of it.
    /// - `wants`, `numpy.uint8`. Zero when the faction offers the good, one
    ///   when it wants the good.
    /// - `asking_good`, `numpy.uint8`. The resource kind asked in return.
    /// - `asking_quantity`, `numpy.uint32`. How much of that.
    ///
    /// Reading a board costs nothing and moves no relation. This method
    /// copies each column.[^1]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the number names no faction of this world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
    fn market<'py>(&self, python: Python<'py>, faction: u16) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        if faction >= world.faction_count() {
            return Err(ViewError::new_err(format!(
                "{faction} names no faction of this world"
            )));
        }
        let rows: Vec<Advert> = world
            .market(FactionId(faction))
            .iter()
            .copied()
            .filter(|row| !row.is_empty())
            .collect();
        let columns = PyDict::new(python);
        let good: Vec<u8> = rows.iter().map(|row| row.good).collect();
        let quantity: Vec<u32> = rows.iter().map(|row| row.quantity).collect();
        let wants: Vec<u8> = rows.iter().map(|row| row.wants).collect();
        let asking_good: Vec<u8> = rows.iter().map(|row| row.asking_good).collect();
        let asking_quantity: Vec<u32> = rows.iter().map(|row| row.asking_quantity).collect();
        columns.set_item("good", good.to_pyarray(python))?;
        columns.set_item("quantity", quantity.to_pyarray(python))?;
        columns.set_item("wants", wants.to_pyarray(python))?;
        columns.set_item("asking_good", asking_good.to_pyarray(python))?;
        columns.set_item("asking_quantity", asking_quantity.to_pyarray(python))?;
        Ok(columns)
    }

    /// Returns how many rows one faction's board holds.
    ///
    /// The value is a balance parameter and the register calls it unset. The
    /// engine holds a stand-in until a caller sets one.
    fn board_rows(&self) -> u16 {
        self.lock().board_rows()
    }

    /// Sets how many rows one faction's board holds.
    ///
    /// The change empties every board, because the table is laid out by the
    /// bound.
    fn set_board_rows(&self, rows: u16) {
        self.lock().set_board_rows(rows);
    }

    /// Returns the most tiles one land side of a contract may name.
    ///
    /// The value is a balance parameter and the register calls it unset. The
    /// engine holds a stand-in until a caller sets one.
    fn land_list_bound(&self) -> u32 {
        self.lock().land_list_bound()
    }

    /// Sets the most tiles one land side of a contract may name.
    fn set_land_list_bound(&self, bound: u32) {
        self.lock().set_land_list_bound(bound);
    }

    /// Returns every tile index of the level 1 cell that covers one address,
    /// ascending, as `numpy.uint32`.
    ///
    /// A cell on the world edge is partial, and the array holds the tiles
    /// that exist. This is the set a land side names when it names a cell.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the address lies outside the world.
    fn cell_tiles<'py>(
        &self,
        python: Python<'py>,
        q: i32,
        r: i32,
    ) -> PyResult<Bound<'py, PyArray1<u32>>> {
        let world = self.lock();
        let tiles: Vec<u32> = world
            .cell_tiles(Axial::new(q, r))
            .map_err(|_| {
                ViewError::new_err(format!("the address ({q}, {r}) lies outside the world"))
            })?
            .iter()
            .map(|tile| tile.0)
            .collect();
        Ok(tiles.to_pyarray(python))
    }

    /// Reports whether the speaker has a unit on the ground the listener
    /// holds, as a boolean.
    ///
    /// This is the gate that every trade verb passes. A player speaks to
    /// another player only while one of its own units stands in that
    /// player's territory.
    ///
    /// The read costs one column read for each unit alive. It answers one
    /// ordered pair. A caller makes this read before it offers a trade.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when a number names no faction of this world.
    fn stands_in_territory_of(&self, speaker: u16, listener: u16) -> PyResult<bool> {
        let world = self.lock();
        let count = world.faction_count();
        if speaker >= count || listener >= count {
            return Err(ViewError::new_err(format!(
                "the pair {speaker} and {listener} names no faction of this world"
            )));
        }
        Ok(world.stands_in_territory_of(FactionId(speaker), FactionId(listener)))
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

    /// Seeds the luxuries of the world.
    ///
    /// A luxury is a presence and not a quantity. A tile carries a luxury or
    /// it does not, and no unit gathers one. The three gatherable kinds are a
    /// separate and fixed catalogue, and this call does not touch them.
    ///
    /// `placements` is a sequence of `(tile, luxury)` pairs of integers. The
    /// tile is the index of a tile in the world. The luxury is a number below
    /// the ceiling that `luxury_ceiling` reports. **The caller gives the
    /// whole set in one call**, in any order, and the engine sorts it. Python
    /// never loops over tiles.[^1]
    ///
    /// One tile may carry more than one luxury. A pair that repeats a
    /// luxury on a tile adds nothing, because a tile carries a luxury or it
    /// does not.
    ///
    /// **The world takes one seed.** A second call raises. The field is not a
    /// fact of a frame, so a reader of it never asks which frame it read.
    ///
    /// Read the result with `luxuries_at`, `variety_at`, `world_variety`,
    /// `cell_variety` and `faction_variety`.
    ///
    /// **Nothing in the engine reads the variety.** It is a score for the
    /// control plane, and no simulation pass consumes it.[^2]
    ///
    /// # Errors
    ///
    /// Raises `ConfigError` when the world already took a seed, when a pair
    /// names a luxury at or above the ceiling, and when a pair names a tile
    /// outside the world. A refusal changes nothing.
    ///
    /// # References
    ///
    /// [^1]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
    /// [^2]: Decisions register, DEC-200. `docs/DECISIONS.md`
    fn seed_luxuries(&self, placements: Vec<(u32, u8)>) -> PyResult<()> {
        let seeds: Vec<(TileIdx, LuxuryId)> = placements
            .into_iter()
            .map(|(tile, luxury)| (TileIdx(tile), LuxuryId(luxury)))
            .collect();
        let mut world = self.lock();
        world
            .seed_luxuries(&seeds)
            .map_err(|error| ConfigError::new_err(error.to_string()))
    }

    /// The number of luxuries the world addresses, as an integer.
    ///
    /// A set of luxuries is one 64-bit word, so the catalogue holds this many
    /// and no more. A number at or above it is refused by `seed_luxuries`,
    /// and it is never folded onto another luxury: two luxuries on one bit
    /// would report the variety as one less than it is.
    #[staticmethod]
    fn luxury_ceiling() -> u8 {
        LUXURY_CEILING
    }

    /// The luxuries that one tile carries, as an integer bit set.
    ///
    /// Bit `n` of the answer stands for the luxury numbered `n`. A tile that
    /// carries nothing gives zero, and so does a tile outside the world,
    /// because such a tile carries nothing.
    ///
    /// The answer is one fixed-width integer, so this read costs the same
    /// whatever the number of luxuries on the tile.
    fn luxuries_at(&self, tile: u32) -> u64 {
        self.lock().luxuries_at(TileIdx(tile)).to_bits()
    }

    /// The number of different luxuries on one tile, as an integer.
    ///
    /// A tile that carries nothing gives zero.
    fn variety_at(&self, tile: u32) -> u32 {
        self.lock().luxuries_at(TileIdx(tile)).variety()
    }

    /// The number of different luxuries in the whole world, as an integer.
    ///
    /// **Nothing in the engine reads this.** It is a score for the control
    /// plane, and no simulation pass consumes it.[^1]
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-200. `docs/DECISIONS.md`
    #[getter]
    fn world_variety(&self) -> u32 {
        self.lock().world_variety()
    }

    /// The number of luxury deposits in the whole world, as an integer.
    ///
    /// A deposit is one luxury on one tile. A tile that carries three
    /// luxuries holds three deposits. This counts deposits.
    /// `world_variety` counts different luxuries. The two answers differ
    /// whenever one luxury stands on more than one tile.
    #[getter]
    fn luxury_deposits(&self) -> i64 {
        self.lock().luxuries().deposits().0
    }

    /// The number of tiles that carry a luxury, as an integer.
    ///
    /// The engine stores one entry for each such tile and nothing else, so
    /// this is also the size of what the world stores for luxuries.
    #[getter]
    fn luxury_tile_count(&self) -> usize {
        self.lock().luxuries().len()
    }

    /// The number of different luxuries in one level 1 cell, as an integer.
    ///
    /// A cell summarises one block of tiles. The answer is the number of
    /// different luxuries on the tiles of the block. It is exact, because
    /// the engine combines the tiles by a set union and not by an average.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the world holds no such cell.
    fn cell_variety(&self, cell: u32) -> PyResult<u32> {
        self.lock()
            .variety_level()
            .variety(cell)
            .ok_or_else(|| ViewError::new_err(format!("the world holds no cell {cell}")))
    }

    /// The number of different luxuries on the ground one faction holds, as
    /// an integer.
    ///
    /// The answer counts a luxury once, whatever the number of held tiles
    /// that carry it. A faction that holds no ground gives zero.
    ///
    /// **Nothing in the engine reads this.** It is a score for the control
    /// plane, and no simulation pass consumes it.[^1]
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-200. `docs/DECISIONS.md`
    fn faction_variety(&self, faction: u16) -> u32 {
        self.lock().faction_variety(FactionId(faction))
    }

    /// Sets what a set of settlements earns of one commodity in one tick.
    ///
    /// Returns `None`.
    ///
    /// The sites are a sequence of settlement identities, or the NumPy array
    /// of `numpy.uint64` that `found_settlements` returned.
    ///
    /// **The rate is a Q16.16 value as its raw integer.** Multiply the amount
    /// you want by 65536. A rate of 65536 means one unit of the commodity in
    /// one tick. A rate of 0 means the site earns nothing.
    ///
    /// **The rate is what one tick earns, not what one application earns.**
    /// The engine multiplies it by the period of the economy schedule. A site
    /// earns the same amount over a span of ticks, whatever the period is.[^1]
    /// A caller may read the rate as the amount of one application. That
    /// caller writes a rate that is too large by the period.
    ///
    /// **The rate may be set at any time, and it takes effect at the next
    /// application.** It is not construction-time configuration. It is state
    /// that a later frame reads. It enters the state hash. Two worlds that
    /// hold different rates are two different worlds.[^2] No write can land
    /// inside a step. The engine releases the interpreter for the whole step.
    /// No Python line runs while the step runs.[^3]
    ///
    /// **Read the rate back with `site_economy`**, under the key
    /// `production`. This call publishes no reader of its own, because the
    /// value would then have two places to come from.[^4]
    ///
    /// The commodity defaults to zero, and the world holds one commodity.
    ///
    /// **The set is all or nothing.** Every identity resolves before anything
    /// is written. The engine refuses the rate and the commodity before it
    /// writes the first site. Neither refusal depends on the site.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live settlement. Raises
    /// `VerbError` when the rate is below zero, and when the number names no
    /// commodity of this world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0062, production and upkeep are rates attached to a site, decision D4. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    /// [^3]: ADR-0042, the interpreter is released for the whole step. `docs/adrs/REGISTRY.md`
    /// [^4]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[pyo3(signature = (sites, rate, commodity = 0))]
    fn set_production_rate(&self, sites: Vec<u64>, rate: i32, commodity: u16) -> PyResult<()> {
        let mut world = self.lock();
        let goods = CommodityId(commodity);
        let rate = Fix32(rate);
        let resolved = resolve_sites(&world, &sites)?;
        for site in resolved {
            world
                .set_production_rate(site, goods, rate)
                .map_err(|error| VerbError::new_err(error.to_string()))?;
        }
        Ok(())
    }

    /// Sets what a set of settlements owes of one commodity in one tick.
    ///
    /// Returns `None`.
    ///
    /// The sites are a sequence of settlement identities, or the NumPy array
    /// of `numpy.uint64` that `found_settlements` returned.
    ///
    /// **The rate is a Q16.16 value as its raw integer, and it is at or above
    /// zero.** Multiply the amount you want by 65536. Upkeep is a rate above
    /// zero that subtracts. It is never a production rate below zero, and the
    /// engine refuses one.[^1]
    ///
    /// **The rate is what one tick owes, not what one application owes.** The
    /// engine multiplies it by the period of the economy schedule, in the same
    /// way it does for production.[^2]
    ///
    /// Production runs before upkeep in one application, so a site pays this
    /// bill from the earnings of the same application. Upkeep that the store
    /// cannot pay is a shortfall: the store stops at zero rather than going
    /// below it.
    ///
    /// **The rate may be set at any time, and it takes effect at the next
    /// application.** It is state that a later frame reads, and it enters the
    /// state hash.[^3]
    ///
    /// **Read the rate back with `site_economy`**, under the key `upkeep`.
    ///
    /// **The set is all or nothing.** Every identity resolves before anything
    /// is written. The engine refuses the rate and the commodity before it
    /// writes the first site. Neither refusal depends on the site.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live settlement. Raises
    /// `VerbError` when the rate is below zero, and when the number names no
    /// commodity of this world.
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-016. `docs/FINDINGS.md`
    /// [^2]: ADR-0062, production and upkeep are rates attached to a site, decision D4. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    /// [^3]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[pyo3(signature = (sites, rate, commodity = 0))]
    fn set_upkeep_rate(&self, sites: Vec<u64>, rate: i32, commodity: u16) -> PyResult<()> {
        let mut world = self.lock();
        let goods = CommodityId(commodity);
        let rate = Fix32(rate);
        let resolved = resolve_sites(&world, &sites)?;
        for site in resolved {
            world
                .set_upkeep_rate(site, goods, rate)
                .map_err(|error| VerbError::new_err(error.to_string()))?;
        }
        Ok(())
    }

    /// Writes what a set of settlements holds of one commodity now.
    ///
    /// Returns `None`.
    ///
    /// The sites are a sequence of settlement identities, or the NumPy array
    /// of `numpy.uint64` that `found_settlements` returned.
    ///
    /// **The quantity is a Q16.16 value as its raw integer.** Multiply the
    /// amount you want by 65536. It is a quantity and not a rate. It says
    /// what the store holds at this tick. The next application changes it
    /// again.
    ///
    /// **The write is absolute and not relative.** The store holds the value
    /// given, whatever it held before. The engine moves its own account of the
    /// stores by the same amount, so a world-wide total stays exact.
    ///
    /// **Pass a quantity at or above zero.** The engine accepts one below zero
    /// and does not refuse it. The next application of upkeep then takes such
    /// a store to zero. It reports the upkeep, plus the amount the store sat
    /// below zero, less that application's production, as the shortfall. A
    /// store of zero is a real state and not an absent one.[^1]
    ///
    /// **The store may be written at any time.** It is simulated state and it
    /// enters the state hash.[^2]
    ///
    /// **Read the store back with `site_economy`**, under the key `store`.
    ///
    /// **The set is all or nothing.** Every identity resolves before anything
    /// is written. The engine refuses the commodity before it writes the
    /// first site, and that refusal does not depend on the site.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live settlement. Raises
    /// `VerbError` when the number names no commodity of this world.
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-043. `docs/FINDINGS.md`
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[pyo3(signature = (sites, quantity, commodity = 0))]
    fn set_settlement_store(&self, sites: Vec<u64>, quantity: i32, commodity: u16) -> PyResult<()> {
        let mut world = self.lock();
        let goods = CommodityId(commodity);
        let resolved = resolve_sites(&world, &sites)?;
        for site in resolved {
            world
                .set_settlement_store(site, goods, Fix32(quantity))
                .map_err(|error| VerbError::new_err(error.to_string()))?;
        }
        Ok(())
    }

    /// Sets how often the engine applies production and upkeep.
    ///
    /// Returns `None`.
    ///
    /// The period is a count of ticks, and it is at least one and at most
    /// 32767. A period of one applies the economy on every step. The phase is
    /// the offset inside the period, so a period of four and a phase of one
    /// apply on the ticks one, five and nine. A phase at or above the period
    /// wraps into it.
    ///
    /// **Raising the period does not raise what a site earns over a span of
    /// ticks.** A rate is what one tick earns, and the engine multiplies it by
    /// the period. The period decides how often a store moves. It does not
    /// decide how much the store moves over time.[^1]
    ///
    /// **The schedule may be set at any time.** It is world-wide, so it is one
    /// write for the world and never one write for each site. It enters the
    /// state hash, because two worlds that apply the economy on different
    /// ticks must diverge.[^2]
    /// Returns every living character, as a `dict` of columns.
    ///
    /// The faction is the number of one faction, or `None` for the whole
    /// world. The default is `None`. A faction number that names no faction
    /// of this world gives empty columns. That is an answer and not an
    /// error, because no character holds that number.
    ///
    /// **The call answers about a set and crosses once.** A caller reads the
    /// whole population in one call and then works on the arrays. It never
    /// asks about one person at a time, because that is the loop the control
    /// plane rule forbids.[^1]
    ///
    /// Seven entries are one-dimensional NumPy arrays with one entry for
    /// each living character.
    ///
    /// - `character`, `numpy.uint64`. The identity of the character. Keep it
    ///   and pass it to any other call that takes a character. Take one entry
    ///   as a Python integer.
    /// - `birth_order`, `numpy.uint32`. The position of the character in the
    ///   record of descent. It counts from zero over every character the
    ///   world has ever made. **It is data and not an identity.** No call in
    ///   this module takes it.
    /// - `faction`, `numpy.uint16`. The number of the faction the character
    ///   belongs to.
    /// - `birth`, `numpy.uint64`. The tick the character was made on.
    /// - `renown`, `numpy.int32`. How much the character is thought of.
    ///   **This is a Q16.16 value as its raw integer. Divide by 65536.**
    ///   Nothing in the engine writes it, so it is zero until a caller writes
    ///   it with `set_character_renown`.
    /// - `sex`, `numpy.uint8`. Zero is female and one is male. The engine
    ///   draws it and no caller sets it.
    /// - `house`, `numpy.uint32`. The birth order of the character who
    ///   founded the house. A character with no father founds a house, and
    ///   its own birth order names it.
    ///
    /// **Every entry names a live character, so no entry stands for
    /// nothing.** The engine builds the set at the moment of the call and
    /// takes no identity from the caller.
    ///
    /// **A character identity and a unit identity share one range of
    /// numbers.** Each arena numbers its own slots, so the first character of
    /// a world and the first unit of a world carry the same number. A call
    /// that takes a character reads the character arena. A call that takes a
    /// unit reads the soldier arena. Neither refuses the number of the other,
    /// and no check reports the mistake. Keep the two kinds of identity
    /// apart. A finding records the measurement.[^3]
    ///
    /// The order is the slot order of the arena. It is the same on every run
    /// and at every thread count. It is never a thread completion order.[^2]
    /// It is not the birth order. A slot returns to the arena when a
    /// character is removed, and the next character takes it. Sort on
    /// `birth_order` for the birth order.
    ///
    /// # References
    ///
    /// [^1]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^3]: Findings register, FND-472. `docs/FINDINGS.md`
    #[pyo3(signature = (faction = None))]
    fn characters<'py>(
        &self,
        python: Python<'py>,
        faction: Option<u16>,
    ) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let arena = world.characters();
        let mut character: Vec<u64> = Vec::new();
        let mut birth_order: Vec<u32> = Vec::new();
        let mut faction_of: Vec<u16> = Vec::new();
        let mut birth: Vec<u64> = Vec::new();
        let mut renown: Vec<i32> = Vec::new();
        let mut sex: Vec<u8> = Vec::new();
        let mut house: Vec<u32> = Vec::new();
        for entity in arena.iter() {
            let held = arena
                .faction(entity)
                .expect("a live identity from the walk names a faction");
            if faction.is_some_and(|wanted| held.0 != wanted) {
                continue;
            }
            character.push(entity.to_bits());
            birth_order.push(
                arena
                    .descent_id(entity)
                    .expect("a live identity from the walk names a descent row")
                    .birth_order(),
            );
            faction_of.push(held.0);
            birth.push(
                arena
                    .birth(entity)
                    .expect("a live identity from the walk names a birth")
                    .0,
            );
            renown.push(
                arena
                    .renown(entity)
                    .expect("a live identity from the walk names a renown")
                    .0,
            );
            sex.push(
                arena
                    .sex(entity)
                    .expect("a live identity from the walk names a sex")
                    .to_column(),
            );
            house.push(
                arena
                    .house(entity)
                    .expect("a live identity from the walk names a house")
                    .founder()
                    .birth_order(),
            );
        }
        let columns = PyDict::new(python);
        columns.set_item("character", character.to_pyarray(python))?;
        columns.set_item("birth_order", birth_order.to_pyarray(python))?;
        columns.set_item("faction", faction_of.to_pyarray(python))?;
        columns.set_item("birth", birth.to_pyarray(python))?;
        columns.set_item("renown", renown.to_pyarray(python))?;
        columns.set_item("sex", sex.to_pyarray(python))?;
        columns.set_item("house", house.to_pyarray(python))?;
        Ok(columns)
    }

    /// Returns the whole lineage of one character, as a `dict`.
    ///
    /// The character is one identity that a call of this module gave.
    ///
    /// **One call answers the whole question.** The parents, every ancestor
    /// and every descendant come back together. A caller never walks the
    /// record one step at a time. A walk across the boundary is the loop the
    /// control plane rule forbids.[^1] The engine walks the record in Rust
    /// and answers once.
    ///
    /// Three entries are plain values that describe the character asked
    /// about.
    ///
    /// - `character`, an `int`. The identity that was asked about.
    /// - `birth_order`, an `int`. The position of the character in the record
    ///   of descent, counted from zero.
    /// - `house`, an `int`. The birth order of the character who founded the
    ///   house. Two characters of one house share that number.
    ///
    /// **This call does not say whether a line has ended.** The engine
    /// reports that, and it reports it for a character who is gone. This call
    /// takes a living character. A line with a living member has not ended,
    /// so the answer would be the same word every time. A finding records
    /// the gap.[^5]
    ///
    /// Three groups of entries describe other people. Each group holds four
    /// parallel NumPy arrays of the same length. The four arrays of one group
    /// describe the same people in the same order.
    ///
    /// - `parent`, `parent_birth_order`, `parent_alive` and `parent_role`.
    ///   The mother and the father. The group holds no row at all when the
    ///   character founds a line. That is a real answer: the world invents no
    ///   parent.[^2]
    /// - `ancestor`, `ancestor_birth_order`, `ancestor_alive` and
    ///   `ancestor_role`. Every ancestor, at every depth. The character is
    ///   never in this group.
    /// - `descendant`, `descendant_birth_order`, `descendant_alive` and
    ///   `descendant_role`. Every descendant, at every depth. The character
    ///   is never in this group.
    ///
    /// The four arrays of a group carry these types.
    ///
    /// - The first array, `numpy.uint64`. The identity the engine minted for
    ///   that person at their birth. **The engine never issues one identity
    ///   twice**, so this value names one person for ever.[^3]
    /// - `_birth_order`, `numpy.uint32`. The position in the record of
    ///   descent. It is data and no call takes it.
    /// - `_alive`, `numpy.uint8`. One when the person is alive now, zero when
    ///   they are gone. **Read this before you pass an identity to another
    ///   call.** An identity with a zero here is refused everywhere, because
    ///   the record of descent outlives the person it names.
    /// - `_role`, `numpy.uint8`. Zero for a mother and one for a father, in
    ///   the parent group. It is zero in the other two groups. A person there
    ///   is reached through many steps and holds no one role.
    ///
    /// **The ancestor and descendant groups are in ascending birth order.**
    /// The order is explicit, and it is the same on every run and at every
    /// thread count.[^4] The parent group holds the mother before the father
    /// when both exist. A role is a fixed position and not a birth order.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no living character. The
    /// read changes nothing, so a refusal leaves the world as it was.
    ///
    /// # References
    ///
    /// [^1]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
    /// [^2]: Blockers register, BLK-011. `docs/BLOCKERS.md`
    /// [^3]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    /// [^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^5]: Findings register, FND-471. `docs/FINDINGS.md`
    fn character_lineage<'py>(
        &self,
        python: Python<'py>,
        character: u64,
    ) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let entity = resolve_character(&world, character)?;
        let arena = world.characters();
        let id = arena
            .descent_id(entity)
            .expect("a resolved identity names a descent row");
        let parents = arena
            .parents(entity)
            .expect("a resolved identity names a pair of parents");
        let mut parent_rows: Vec<(DescentId, u8)> = Vec::new();
        if let Some(mother) = parents.mother {
            parent_rows.push((mother, MOTHER_ROLE));
        }
        if let Some(father) = parents.father {
            parent_rows.push((father, FATHER_ROLE));
        }
        let ancestors = with_no_role(&world.character_ancestors(entity));
        let descendants = with_no_role(&world.character_descendants(entity));

        let columns = PyDict::new(python);
        columns.set_item("character", entity.to_bits())?;
        columns.set_item("birth_order", id.birth_order())?;
        columns.set_item(
            "house",
            arena
                .house(entity)
                .expect("a resolved identity names a house")
                .founder()
                .birth_order(),
        )?;
        write_kin(python, &columns, "parent", arena, &parent_rows)?;
        write_kin(python, &columns, "ancestor", arena, &ancestors)?;
        write_kin(python, &columns, "descendant", arena, &descendants)?;
        Ok(columns)
    }

    /// Returns how closely one character is related to each of a set of
    /// others.
    ///
    /// The subject is one identity. The others are a sequence of identities,
    /// or the `character` array that `characters` returned. Returns a
    /// one-dimensional NumPy array of `numpy.int32`, one value for each
    /// entry of `others`, in the order of `others`.
    ///
    /// **This call takes a set and answers once.** A caller that reads one
    /// person against the whole population crosses the boundary once per
    /// pair. The number of pairs is the thing that grows.[^1]
    ///
    /// The value is the coefficient of relationship. A parent and a child
    /// give one half. Two children of one pair of parents give one half as
    /// well. A character against itself gives one, plus its inbreeding
    /// coefficient. Two characters with no ancestor in common give zero. A
    /// character who founds a line has no ancestor. It stands at zero to
    /// every character who is not descended from it.
    ///
    /// **The value is a Q16.16 fixed-point number as its raw integer. Divide
    /// by 65536.** One half is 32768. The value is exact. Every step of the
    /// recursion halves a value, so no step rounds. No floating point number
    /// is involved.[^2]
    ///
    /// **The engine answers only for two living characters.** The record of
    /// descent outlives a character. The relation reads the row that the
    /// arena slot points at. Only a living character has one. A caller
    /// therefore cannot ask how a living person is related to a dead
    /// ancestor. `character_lineage` answers who the ancestors are.
    ///
    /// **The set is all or nothing.** Every identity resolves before any
    /// value is computed, so one dead identity answers nothing and raises.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the subject or any member of `others` names no
    /// living character. The read changes nothing.
    ///
    /// # References
    ///
    /// [^1]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
    /// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    fn character_relations<'py>(
        &self,
        python: Python<'py>,
        subject: u64,
        others: Vec<u64>,
    ) -> PyResult<Bound<'py, PyArray1<i32>>> {
        let world = self.lock();
        let left = resolve_character(&world, subject)?;
        let mut resolved = Vec::with_capacity(others.len());
        for other in &others {
            resolved.push(resolve_character(&world, *other)?);
        }
        let values: Vec<i32> = resolved
            .into_iter()
            .map(|right| world.character_relation(left, right).0)
            .collect();
        Ok(values.to_pyarray(python))
    }

    /// Returns what each unit of a set has ever gathered.
    ///
    /// The units are a sequence of identities, or the NumPy array of
    /// `numpy.uint64` that `spawn_soldiers` returned. Returns a
    /// one-dimensional NumPy array of `numpy.uint64`, one entry for each
    /// unit, in the order of the units.
    ///
    /// The value is a whole quantity of resource, summed over every kind and
    /// over the whole life of the unit. It is not a rate and it carries no
    /// fixed-point scale. It never falls while the unit lives.
    ///
    /// A unit becomes eligible to be raised into a character when this value
    /// reaches the level that `deed_threshold` reports.
    ///
    /// **The set is all or nothing.** Every identity resolves before any
    /// value is read, so one dead identity answers nothing and raises.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live soldier. The read
    /// changes nothing.
    fn unit_deeds<'py>(
        &self,
        python: Python<'py>,
        units: Vec<u64>,
    ) -> PyResult<Bound<'py, PyArray1<u64>>> {
        let world = self.lock();
        let mut resolved = Vec::with_capacity(units.len());
        for unit in &units {
            resolved.push(resolve(&world, *unit)?);
        }
        let values: Vec<u64> = resolved
            .into_iter()
            .map(|entity| {
                world
                    .unit_deeds(entity)
                    .expect("a resolved identity names a live soldier")
            })
            .collect();
        Ok(values.to_pyarray(python))
    }

    /// Returns the character that each unit of a set was raised into.
    ///
    /// The units are a sequence of identities, or the NumPy array of
    /// `numpy.uint64` that `spawn_soldiers` returned. Returns a
    /// one-dimensional NumPy array of `numpy.uint64`, one entry for each
    /// unit, in the order of the units.
    ///
    /// **A zero means the unit carries no living character.** The engine
    /// never issues zero as an identity, so zero cannot be confused with a
    /// person.[^1] A unit that was never raised reads zero, and so does a
    /// unit whose character has been removed.
    ///
    /// **A raised unit is not turned into a character.** The engine creates a
    /// character beside the unit and links the two. The unit stays a unit. It
    /// keeps its tile and keeps moving. An entity declares its tier when it
    /// is created and never changes tier.[^2]
    ///
    /// **The set is all or nothing.** Every identity resolves before any
    /// value is read, so one dead identity answers nothing and raises.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live soldier. The read
    /// changes nothing.
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D1. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    /// [^2]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D4. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
    fn unit_characters<'py>(
        &self,
        python: Python<'py>,
        units: Vec<u64>,
    ) -> PyResult<Bound<'py, PyArray1<u64>>> {
        let world = self.lock();
        let mut resolved = Vec::with_capacity(units.len());
        for unit in &units {
            resolved.push(resolve(&world, *unit)?);
        }
        let values: Vec<u64> = resolved
            .into_iter()
            .map(|entity| {
                world
                    .unit_character(entity)
                    .expect("a resolved identity names a live soldier")
                    .map_or(0, |character| character.to_bits())
            })
            .collect();
        Ok(values.to_pyarray(python))
    }

    /// Returns the deeds at which the engine raises a unit into a character.
    ///
    /// The value is a whole quantity of resource, summed over every kind. It
    /// carries no fixed-point scale. A unit whose deeds reach this level
    /// becomes eligible. The engine then chooses among the eligible by a rule
    /// of its own.
    fn deed_threshold(&self) -> u64 {
        self.lock().deed_threshold()
    }

    /// Sets the deeds at which the engine raises a unit into a character.
    ///
    /// The threshold is a whole quantity of resource, summed over every
    /// kind. It carries no fixed-point scale. Returns `None`.
    ///
    /// The threshold is a content parameter and not a budget. Raise it to
    /// make a named person rare. Lower it to make one common. A threshold of
    /// zero makes every unit eligible.
    ///
    /// **A caller sets the level. It does not choose who is raised.** The
    /// engine collects the eligible units. It ranks them by a key vector of
    /// its own. It cuts the list at a budget.[^1] Nothing in this module
    /// names a unit to raise.
    ///
    /// # References
    ///
    /// [^1]: ADR-0104, a soldier is promoted from a level that never falls, decision D4. `docs/adrs/draft/adr-0104-a-soldier-is-promoted-from-a-level-that-never-falls.md`
    fn set_deed_threshold(&self, threshold: u64) {
        self.lock().set_deed_threshold(threshold);
    }

    /// Sets the schedule that the site rates apply on.
    ///
    /// Returns `None`.
    ///
    /// The period is a count of ticks, and it must be at least one. A period
    /// of one applies on every step. The phase is the offset inside the
    /// period. A period of four and a phase of one apply on the ticks one,
    /// five and nine. A phase at or above the period wraps into it.
    ///
    /// The interval is a parameter of the schedule and not a constant of the
    /// engine.[^1] The schedule enters the state hash, because the next frame
    /// reads it.[^2]
    ///
    /// The world starts with a schedule already set. This call replaces it.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the period is zero, or above the range that the
    /// scaling multiply takes. The message names the limit it applied.
    ///
    /// # References
    ///
    /// [^1]: ADR-0062, production and upkeep are rates attached to a site, decision D4. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    fn set_economy_schedule(&self, period: u32, phase: u32) -> PyResult<()> {
        self.lock()
            .set_economy_schedule(period, phase)
            .map_err(|error| VerbError::new_err(error.to_string()))
    }

    /// Returns how fast a depleted deposit of each kind returns, as a list.
    ///
    /// The list holds one entry for each resource kind, in the order food,
    /// wood, stone. An entry is a count of ticks, or `None` for a kind that
    /// does not recover.
    ///
    /// **A period is the simulated time in which one depleted deposit regains
    /// one unit of stock.** It is a count of ticks and it is not fixed point.
    ///
    /// Write the rules with `set_recovery_rules`.
    fn recovery_rules(&self) -> Vec<Option<u32>> {
        let world = self.lock();
        let rules = world.recovery_rules();
        ResourceKind::ALL
            .iter()
            .map(|kind| rules.period_of(*kind))
            .collect()
    }

    /// Sets how fast a depleted deposit of each kind returns.
    ///
    /// Returns `None`.
    ///
    /// The periods are a sequence of three entries, one for each resource
    /// kind, in the order food, wood, stone. An entry is a count of ticks at
    /// or above one, or `None` for a kind that does not recover.
    ///
    /// **A period is the simulated time in which one depleted deposit regains
    /// one unit of stock.** It is a count of ticks and it is not fixed point.
    /// A smaller period returns a deposit faster.
    ///
    /// **The caller states every kind, and the engine takes the whole set.**
    /// No call changes one kind. A merge would put the period of a kind in
    /// two places while the call ran.[^1] A caller that wants to change one
    /// kind reads the three with `recovery_rules`, changes one, and writes the
    /// three back.
    ///
    /// **The rules may be set at any time.** They are world-wide, so this is
    /// one write for the world. The engine reads them on every tick that ages
    /// a depleted deposit.
    ///
    /// **The rules do not enter the state hash today, and that is a
    /// defect.**[^2] Two worlds that hold the same tiles and different rules
    /// hash the same and then diverge. The golden state test reports the
    /// effect of a change made here, and never the change itself. A backlog
    /// item holds the repair.[^3]
    ///
    /// # Errors
    ///
    /// Raises `ValueError` when the sequence does not hold exactly three
    /// entries. Raises `VerbError` when a period is zero. A period of zero
    /// returns the whole take in one tick. That is a second way to say that a
    /// deposit was never depleted.
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-270. `docs/DECISIONS.md`
    /// [^2]: Findings register, FND-480. `docs/FINDINGS.md`
    /// [^3]: Backlog item 0471, fold the recovery rules into the state hash. `docs/backlog/proposed/0471-fold-the-recovery-rules-into-the-state-hash.md`
    fn set_recovery_rules(&self, periods: Vec<Option<u32>>) -> PyResult<()> {
        if periods.len() != RESOURCE_KIND_COUNT {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "the rules need one period for each of the {RESOURCE_KIND_COUNT} resource kinds, and {} were given",
                periods.len()
            )));
        }
        let mut taken = [None; RESOURCE_KIND_COUNT];
        taken.copy_from_slice(&periods);
        let rules = RecoveryRules::from_ticks(taken).ok_or_else(|| {
            VerbError::new_err(
                "a recovery period of zero is not a period; use None for a kind that does not recover",
            )
        })?;
        self.lock().set_recovery_rules(rules);
        Ok(())
    }

    /// Gives a set of units the settlement that they draw from.
    ///
    /// Returns `None`.
    ///
    /// The units are a sequence of unit identities, or the NumPy array of
    /// `numpy.uint64` that `spawn_soldiers` returned. The site is one
    /// settlement identity, or `None`.
    ///
    /// A unit draws its rations from the store of its home, and it lives
    /// there. A unit whose home is `None` draws from nothing. That is a state
    /// the world represents, not an error.
    ///
    /// **The home may be set at any time.** It is simulated state, and it
    /// enters the state hash. Two worlds that feed a unit from different
    /// stores must diverge.[^1]
    ///
    /// **The set is all or nothing.** The site resolves first, then every unit
    /// identity, and nothing is written until all of them resolve.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live unit, and when the
    /// site names no live settlement.
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[pyo3(signature = (units, site = None))]
    fn set_home_site(&self, units: Vec<u64>, site: Option<u64>) -> PyResult<()> {
        let mut world = self.lock();
        let home = match site {
            Some(site) => Some(resolve_site(&world, site)?),
            None => None,
        };
        let mut resolved = Vec::with_capacity(units.len());
        for unit in &units {
            resolved.push(resolve(&world, *unit)?);
        }
        for unit in resolved {
            world.set_home_site(unit, home);
        }
        Ok(())
    }

    /// Returns what one faction reaches at the block of ground that covers a
    /// tile.
    ///
    /// **The value is unsigned fixed point against a fixed reference, and
    /// 65535 means one reference unit.** It is not the Q16.16 scale that the
    /// rates use, and no cell holds more than 65535.[^1]
    ///
    /// **The field answers for a block of ground and not for a tile.** Two
    /// addresses in one block give one answer.
    ///
    /// The field is derived at the end of a step, from the sources that
    /// `set_influence_source` wrote and from the ground. A world that has not
    /// stepped since a source was written reports what the last step
    /// left.[^2]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the address lies outside the world, and when
    /// the world holds no such faction.
    ///
    /// # References
    ///
    /// [^1]: ADR-0060, an influence map is stored as a shared basis, decision D2. `docs/adrs/draft/adr-0060-an-influence-map-is-stored-as-a-shared-basis.md`
    /// [^2]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
    fn influence(&self, faction: u16, q: i32, r: i32) -> PyResult<u16> {
        let world = self.lock();
        world
            .influence(FactionId(faction), Axial { q, r })
            .map(|value| value.0)
            .ok_or_else(|| {
                ViewError::new_err(format!(
                    "the faction {faction} or the address ({q}, {r}) is outside this world"
                ))
            })
    }

    /// Sets what one faction injects at a set of places.
    ///
    /// Returns `None`.
    ///
    /// The addresses are a sequence of `(q, r)` pairs. Each names a tile, and
    /// the engine writes the source at the block of ground that covers that
    /// tile. Two addresses in one block are two writes of one cell, and the
    /// last one stands.
    ///
    /// **The source is unsigned fixed point against a fixed reference, and
    /// 65535 means one reference unit.** It is not the Q16.16 scale that the
    /// rates use. A source of 0 is the ordinary value and it is not an
    /// absence.[^1]
    ///
    /// **The engine holds no rule that decides this value.** A rule that
    /// writes a source term lives above the engine, which is why the control
    /// plane holds the write.[^1]
    ///
    /// **A source may be set at any time, and the next step spreads it.** The
    /// solve runs last in a step, over the whole plane, for a fixed number of
    /// passes.[^2] A source enters the state hash, because the next solve
    /// starts from it.[^3]
    ///
    /// **The set is all or nothing.** Every address and the faction are
    /// checked before anything is written.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an address lies outside the world, and when the
    /// world holds no such faction.
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-041. `docs/DECISIONS.md`
    /// [^2]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
    /// [^3]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    fn set_influence_source(
        &self,
        faction: u16,
        addresses: Vec<(i32, i32)>,
        source: u16,
    ) -> PyResult<()> {
        let mut world = self.lock();
        let who = FactionId(faction);
        for (q, r) in &addresses {
            if world.influence(who, Axial { q: *q, r: *r }).is_none() {
                return Err(ViewError::new_err(format!(
                    "the faction {faction} or the address ({q}, {r}) is outside this world"
                )));
            }
        }
        for (q, r) in addresses {
            world.set_influence_source(who, Axial { q, r }, Influence(source));
        }
        Ok(())
    }
    /// Sets how often the engine looks for a unit to raise.
    ///
    /// Returns `None`.
    ///
    /// The period is a count of ticks, and it must be at least one. A period
    /// of one looks on every step. The phase is the offset inside the period.
    /// A period of four and a phase of one look on the ticks one, five and
    /// nine. A phase at or above the period wraps into it.
    ///
    /// The schedule decides which frames run the promotion pass.[^1] The
    /// schedule enters the state hash, because the next step reads it.[^2]
    ///
    /// The world starts with a schedule already set. This call replaces it.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the period is zero, or above the range that the
    /// scaling multiply takes. The message names the limit it applied.
    ///
    /// # References
    ///
    /// [^1]: ADR-0104, a soldier is promoted from a level that never falls, decision D5. `docs/adrs/draft/adr-0104-a-soldier-is-promoted-from-a-level-that-never-falls.md`
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    fn set_character_schedule(&self, period: u32, phase: u32) -> PyResult<()> {
        self.lock()
            .set_character_schedule(period, phase)
            .map_err(|error| VerbError::new_err(error.to_string()))
    }

    /// Makes a number of characters in one faction and returns their
    /// identities.
    ///
    /// The faction names the faction that owns the new characters, by its
    /// number. The count is how many to make.
    ///
    /// Returns a one-dimensional NumPy array of `numpy.uint64`, one identity
    /// for each character. Keep the array and pass it to any other call that
    /// takes a character.
    ///
    /// **Every character this call makes founds a line.** It has no parents,
    /// and its relation to every other character is zero. The world invents
    /// no ancestry, which is the same rule that governs a unit the engine
    /// raises from the ranks.[^1] Call `bear_children` to give somebody
    /// parents.
    ///
    /// A new character holds a renown of zero. Zero is a real state and not
    /// an absent one.[^2] The engine draws the sex, and no caller sets it.
    /// The character is born on the current tick of the world.
    ///
    /// **The set is all or nothing.** The call checks the room in both stores
    /// before it makes anybody. It removes every character it made when a
    /// later one refuses. A refusal therefore makes nobody.
    ///
    /// **The arena refuses the faction, and this call does not check it a
    /// second time.** A check here would be a second declaration site of one
    /// rule. Nothing would fail when the two copies disagreed.[^3] The first
    /// creation of the set refuses, so nothing is made.
    ///
    /// **A character identity and a unit identity share one range of
    /// numbers.** Each arena numbers its own slots. The first character of a
    /// world and the first unit of a world carry the same number. Neither
    /// kind of call refuses the number of the other, and no check reports the
    /// mistake.[^4]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the world has no such faction. It raises the
    /// same error when the count is above the room either store has left.
    /// The message names the value that refused.
    ///
    /// # References
    ///
    /// [^1]: Blockers register, BLK-011. `docs/BLOCKERS.md`
    /// [^2]: Findings register, FND-043. `docs/FINDINGS.md`
    /// [^3]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
    /// [^4]: Findings register, FND-472. `docs/FINDINGS.md`
    fn create_characters<'py>(
        &self,
        python: Python<'py>,
        faction: u16,
        count: u32,
    ) -> PyResult<Bound<'py, PyArray1<u64>>> {
        let mut world = self.lock();
        refuse_no_room(&world, count)?;
        let mut made: Vec<u64> = Vec::with_capacity(count as usize);
        for _ in 0..count {
            match world.create_character(FactionId(faction)) {
                Ok(entity) => made.push(entity.to_bits()),
                Err(error) => return Err(undo_births(&mut world, &made, &error.to_string())),
            }
        }
        Ok(made.to_pyarray(python))
    }

    /// Bears one child for each pair of parents and returns the children.
    ///
    /// The births are a sequence of `(mother, father)` pairs of identities.
    /// Returns a one-dimensional NumPy array of `numpy.uint64`, one identity
    /// for each child, in the order of the pairs.
    ///
    /// A child takes the faction of its mother and it records both parents.
    /// The record of descent keeps those two edges after either parent is
    /// gone. A caller reads a dead parent through a living child.[^1] The
    /// child is born on the current tick of the world, and it holds a renown
    /// of zero.
    ///
    /// **The two names are roles and not a test of sex.** The engine puts the
    /// first identity in the mother role and the second in the father role.
    /// It reads the sex column of neither. A game that wants a rule about sex
    /// reads the `sex` column that `characters` returns and applies the rule
    /// itself.
    ///
    /// **Both parents must be alive.** The record of descent outlives a
    /// character, so a caller reads a dead parent. It cannot name one as a
    /// parent of a new child.
    ///
    /// **A caller states a birth. The engine states none.** No pass in the
    /// engine bears a child on its own. Every child in a run comes from this
    /// call.
    ///
    /// **The set is all or nothing.** Every identity resolves, and every pair
    /// is checked. The call checks the room in both stores before any child
    /// is born.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no living character. Raises
    /// `VerbError` when the two parents of a pair are one character. It also
    /// raises `VerbError` when the number of births is above the room either
    /// store has left.
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-003. `docs/DECISIONS.md`
    fn bear_children<'py>(
        &self,
        python: Python<'py>,
        births: Vec<(u64, u64)>,
    ) -> PyResult<Bound<'py, PyArray1<u64>>> {
        let mut world = self.lock();
        let mut resolved = Vec::with_capacity(births.len());
        for (mother, father) in &births {
            let mother = resolve_character(&world, *mother)?;
            let father = resolve_character(&world, *father)?;
            if mother == father {
                return Err(VerbError::new_err(format!(
                    "the identity {} names both parents, and a child has two parents",
                    mother.to_bits()
                )));
            }
            resolved.push((mother, father));
        }
        refuse_no_room(&world, resolved.len() as u32)?;
        let mut made: Vec<u64> = Vec::with_capacity(resolved.len());
        for (mother, father) in resolved {
            match world.bear_character(mother, father) {
                Ok(child) => made.push(child.to_bits()),
                Err(error) => return Err(undo_births(&mut world, &made, &error.to_string())),
            }
        }
        Ok(made.to_pyarray(python))
    }

    /// Removes every character the identities name.
    ///
    /// The characters are a sequence of identities, or the NumPy array of
    /// `numpy.uint64` that `characters` or `create_characters` returned.
    /// Returns `None`.
    ///
    /// **The set is all or nothing.** Every identity resolves before anything
    /// is removed, so one dead identity removes nobody and raises.
    ///
    /// A removed character leaves its slot to the next character. Its
    /// identity is then stale, and every call that takes a character refuses
    /// it.[^1]
    ///
    /// **The record of descent keeps the person.** A removal releases the
    /// slot columns and nothing else. The parent edges stay. A living child
    /// still reads a removed parent. `character_lineage` still names the
    /// removed person with a zero in its `_alive` column.[^2]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no living character.
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    /// [^2]: ADR-0078, descent is a bounded record, and a relation is a bounded recursion, decision D1. `docs/adrs/draft/adr-0078-descent-is-a-bounded-record-and-a-relation-is-a-bounded-recursion.md`
    fn remove_characters(&self, characters: Vec<u64>) -> PyResult<()> {
        let mut world = self.lock();
        let mut resolved = Vec::with_capacity(characters.len());
        for character in &characters {
            resolved.push(resolve_character(&world, *character)?);
        }
        for entity in resolved {
            assert!(
                world.remove_character(entity),
                "a resolved identity must name a character the arena can remove"
            );
        }
        Ok(())
    }

    /// Writes how much each character of a set is thought of.
    ///
    /// The characters are a sequence of identities, or the NumPy array of
    /// `numpy.uint64` that `characters` returned. Returns `None`.
    ///
    /// **The renown is a Q16.16 fixed-point value as its raw integer.
    /// Multiply by 65536.** One whole unit of renown is 65536. The value may
    /// be negative. No floating point number crosses this boundary, because
    /// a float sum is not associative and the engine hashes this column.[^1]
    ///
    /// **One call writes one value to the whole set.** A caller that wants
    /// two values makes two calls, one for each value. That is a loop over
    /// values and not a loop over people. The number of values a game uses
    /// does not grow with the population.
    ///
    /// A renown of zero is a real state and not an absent one. A write of
    /// zero is a write.[^2]
    ///
    /// **Nothing in the engine reads this column.** It is a value for the
    /// control plane, and no simulation pass consumes it. The engine writes
    /// it once, at a creation, with a zero. After that, it stays at whatever
    /// a caller last wrote.
    ///
    /// **The set is all or nothing.** Every identity resolves before anything
    /// is written, so one dead identity writes nothing and raises.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no living character.
    ///
    /// # References
    ///
    /// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^2]: Findings register, FND-043. `docs/FINDINGS.md`
    fn set_character_renown(&self, characters: Vec<u64>, renown: i32) -> PyResult<()> {
        let mut world = self.lock();
        let mut resolved = Vec::with_capacity(characters.len());
        for character in &characters {
            resolved.push(resolve_character(&world, *character)?);
        }
        for entity in resolved {
            assert!(
                world.set_character_renown(entity, Fix32(renown)),
                "a resolved identity must name a character the arena can write"
            );
        }
        Ok(())
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
/// **The camera is a presentation value, not simulation state.** The engine
/// holds no camera. It borrows one for the length of a draw call and keeps
/// nothing of it. A frame is a pure function of a world and a camera. Two
/// calls with the same world and the same camera give the same picture. That
/// property makes a scripted flight, an agent that steers, and a
/// reproducible screenshot possible.[^1]
///
/// The state lives in Python. Python decides when to move and by how much.
/// The pan share and the zoom step live here, once. A copy on both sides of
/// the boundary would be one value in two places. Nothing fails when the
/// copies disagree.[^2]
///
/// A verb that works in pixels alone takes no size. A camera verb reads no
/// pixel, so a caller that has not drawn yet can still steer.
///
/// # Build a camera
///
/// ```text
/// Camera(tile_size=None)
/// ```
///
/// **The parameters of the constructor are here, and not under a separate
/// entry.** The binding library does not publish the prose of a constructor.
/// This class doc comment is the one place that holds it.[^3]
///
/// - `tile_size`, a `float` or `None`. The width and the height of one tile,
///   in pixels. The default is `None`, which gives 12.0 pixels. The
///   constructor holds the value inside the range 2.0 to 64.0 pixels. A size
///   outside that range gives the nearest size inside it.
///
/// The tile size is a choice of the caller and not a property of the world.
/// The new camera sits at the corner of the world, so it shows the tile at
/// the address `(0, 0)`. Call `look_at` to place it somewhere else.
///
/// **Call the `fitting` static method to get the camera to start from.** A
/// caller that draws with it sees the world rather than an empty picture.
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
    /// binding library does not copy the doc comment of a constructor onto the
    /// Python object. Prose written here reaches no reader of the published
    /// reference.[^1]
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
    /// **The size of one tile never falls below two pixels.** When the world
    /// is too large to fit at that size, the picture shows a part of the
    /// world.
    ///
    /// **This is the camera to start from.** A caller that draws with it sees
    /// the world rather than an empty picture.
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
    /// the camera, so the caller may build any camera it likes. The frame
    /// verb refuses the ones it cannot draw. It names the bound it refuses
    /// against. A setter that held the scale quietly would return a picture
    /// that did not match the camera. A caller could not tell that from a
    /// picture that did.[^1]
    ///
    /// The scroll and zoom verbs do hold the scale. They are what a person
    /// drives. A person should not be able to press a key into a refusal.
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
    /// picture. One press moves the view by the same part of it at every
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
    /// tile under the middle of the picture stays under the middle. The
    /// call holds the tile size inside the range the camera accepts.
    fn zoom_in(&mut self, width: usize, height: usize) {
        self.inner = self.inner.zoomed_in(&FrameSize::new(width, height));
    }

    /// Makes each tile smaller by one press. Returns `None`.
    ///
    /// The width and the height are the size of the picture in pixels. The
    /// tile under the middle of the picture stays under the middle. The
    /// call holds the tile size inside the range the camera accepts.
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

    /// Holds the view so the world cannot leave the picture. Returns `None`.
    ///
    /// The world is the world whose bounds hold the view. The width and
    /// the height are the size of the picture in pixels. In each direction,
    /// at least half of the smaller of the world and the picture stays on
    /// the screen.
    ///
    /// A camera that ran off the edge would show a picture of nothing. A
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
    /// into an address and reads nothing. Check the pair against `width`
    /// and `height` before you pass it on.
    ///
    /// **This is how a click reaches a tile without a loop.** The control
    /// plane sends one pixel and gets one address, rather than walking the
    /// tiles to find which one was hit.
    fn tile_at(&self, x: f32, y: f32) -> (i32, i32) {
        let address = self.inner.tile_at(x, y);
        (address.q, address.r)
    }

    /// Returns a `str` that names the camera with its tile size and its
    /// origin.
    fn __repr__(&self) -> String {
        format!(
            "Camera(tile_width={}, tile_height={}, origin_x={}, origin_y={})",
            self.inner.tile_width, self.inner.tile_height, self.inner.origin_x, self.inner.origin_y
        )
    }
}

/// Turns founding outcomes into the reports a caller prints, and keeps the
/// outcomes for the panel.
fn founding_reports<'py>(
    world: &PyWorld,
    python: Python<'py>,
    outcomes: Vec<FoundingOutcome>,
    group: u32,
) -> PyResult<Vec<Bound<'py, PyDict>>> {
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
    world.presenter().outcomes = outcomes;
    Ok(reports)
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

/// The role value that names the mother of a character.
///
/// The lineage read answers with a role column, and the two values live here
/// once. A second copy elsewhere would be one value in two places with
/// nothing that fails when the copies disagree.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
const MOTHER_ROLE: u8 = 0;

/// The role value that names the father of a character.
const FATHER_ROLE: u8 = 1;

/// Resolves a character identity that Python handed back, or raises.
///
/// The engine compares the generation, so a character who is gone never
/// answers for the character made next in their slot.[^1]
///
/// # References
///
/// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
fn resolve_character(world: &CoreWorld, character: u64) -> PyResult<Entity> {
    world
        .resolve_character(character)
        .map_err(|error| ViewError::new_err(error.to_string()))
}

/// Pairs each row of a walk with the role value that means no one role.
///
/// An ancestor and a descendant are reached through many steps, so neither
/// holds the mother role or the father role. The column exists so that the
/// three groups of a lineage answer have one shape.
fn with_no_role(rows: &[DescentId]) -> Vec<(DescentId, u8)> {
    rows.iter().map(|row| (*row, MOTHER_ROLE)).collect()
}

/// Writes one group of four parallel columns into a lineage answer.
///
/// The four keys are the name, and the name with each of the three suffixes.
/// The identity is the one the arena minted at the birth of that person, and
/// it never names anybody else.[^1] The live flag says whether the arena
/// still holds it, because the record of descent outlives the person.[^2]
///
/// # References
///
/// [^1]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
/// [^2]: ADR-0078, descent is a bounded record, and a relation is a bounded recursion, decision D1. `docs/adrs/draft/adr-0078-descent-is-a-bounded-record-and-a-relation-is-a-bounded-recursion.md`
fn write_kin(
    python: Python<'_>,
    columns: &Bound<'_, PyDict>,
    name: &str,
    arena: &CharacterArena,
    rows: &[(DescentId, u8)],
) -> PyResult<()> {
    let descent = arena.descent();
    let mut identity: Vec<u64> = Vec::with_capacity(rows.len());
    let mut birth_order: Vec<u32> = Vec::with_capacity(rows.len());
    let mut alive: Vec<u8> = Vec::with_capacity(rows.len());
    let mut role: Vec<u8> = Vec::with_capacity(rows.len());
    for (row, held) in rows {
        let entity = descent
            .born_as(*row)
            .expect("a row of the record of descent names the identity it was born as");
        identity.push(entity.to_bits());
        birth_order.push(row.birth_order());
        alive.push(u8::from(arena.contains(entity)));
        role.push(*held);
    }
    columns.set_item(name, identity.to_pyarray(python))?;
    columns.set_item(
        format!("{name}_birth_order"),
        birth_order.to_pyarray(python),
    )?;
    columns.set_item(format!("{name}_alive"), alive.to_pyarray(python))?;
    columns.set_item(format!("{name}_role"), role.to_pyarray(python))?;
    Ok(())
}

/// Refuses a set of new characters that the storage has no room for.
///
/// The check runs before anything is made, so a refusal makes nobody. The
/// arena refuses a creation beyond its own capacity whatever this says, so
/// this is a cut and never a second enforcement of the ceiling.[^1]
///
/// Two stores bound a creation. The arena holds the living characters, and a
/// slot returns to it when a character is removed. The record of descent
/// holds every character the world has ever made, and it never releases a
/// row.[^2]
///
/// # References
///
/// [^1]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decisions D2 and D3. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
/// [^2]: ADR-0078, descent is a bounded record, and a relation is a bounded recursion, decision D1. `docs/adrs/draft/adr-0078-descent-is-a-bounded-record-and-a-relation-is-a-bounded-recursion.md`
fn refuse_no_room(world: &CoreWorld, count: u32) -> PyResult<()> {
    let arena = world.characters();
    let living = arena
        .capacity()
        .saturating_sub(arena.len())
        .saturating_sub(arena.retired_count());
    if count > living {
        return Err(VerbError::new_err(format!(
            "{count} new characters do not fit, and the arena has room for {living}"
        )));
    }
    let recorded = DESCENT_CEILING.saturating_sub(arena.descent().len());
    if count > recorded {
        return Err(VerbError::new_err(format!(
            "{count} new characters do not fit, and the record of descent has room for {recorded}"
        )));
    }
    Ok(())
}

/// Removes every character a refused set made, and returns the refusal.
///
/// A set-valued creation leaves nothing half made. The room checks run before
/// the first creation, so this path is unreachable today. It stays because a
/// verb that writes must state what it does on a refusal, and the statement
/// belongs in code rather than in prose.[^1]
///
/// **The record of descent keeps the rows.** It never releases one, so the
/// characters this removes end in the same state as characters who lived and
/// were then removed.[^2]
///
/// # References
///
/// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
/// [^2]: ADR-0078, descent is a bounded record, and a relation is a bounded recursion, decision D1. `docs/adrs/draft/adr-0078-descent-is-a-bounded-record-and-a-relation-is-a-bounded-recursion.md`
fn undo_births(world: &mut CoreWorld, made: &[u64], said: &str) -> PyErr {
    for identity in made {
        if let Ok(entity) = world.resolve_character(*identity) {
            world.remove_character(entity);
        }
    }
    VerbError::new_err(format!("the character arena refused: {said}"))
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

// The economy knobs read two names that no other binding needs. The imports
// sit here, beside the helpers that use them, rather than in the block at the
// top of the file.
use cachette_core::resource::{RecoveryRules, RESOURCE_KIND_COUNT};

/// Resolves every settlement identity of a set, or raises on the first stale
/// one.
///
/// The whole set resolves before a caller writes anything, so one stale
/// identity leaves the world unchanged.[^1]
///
/// # References
///
/// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
fn resolve_sites(world: &CoreWorld, sites: &[u64]) -> PyResult<Vec<Entity>> {
    let mut resolved = Vec::with_capacity(sites.len());
    for site in sites {
        resolved.push(resolve_site(world, *site)?);
    }
    Ok(resolved)
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

/// Turns a refusal of a trade verb into the exception a caller catches.
///
/// The message states which rule refused. A closure states the step that
/// opens the direction again, because a player that cannot tell a refusal
/// from a closed door asks for ever.[^1]
///
/// # References
///
/// [^1]: ADR-0127, a terminal refusal closes an ordered pair until a named tick, decision D3. `docs/adrs/draft/adr-0127-a-terminal-refusal-closes-a-pair-until-a-named-tick.md`
/// Builds one side of a contract from the keyword arguments of a trade verb.
///
/// A land side takes its tiles from the cell, from the list, or from both.
/// The other kinds keep the kind and the amount and hold no tile.
fn consideration_of(
    world: &cachette_core::World,
    tag: u8,
    kind: u8,
    amount: u32,
    tiles: Option<Vec<(i32, i32)>>,
    cell: Option<(i32, i32)>,
) -> PyResult<Consideration> {
    match tag {
        KIND_RESOURCE => Ok(Consideration::resource(kind, amount)),
        KIND_RELATION => Ok(Consideration::relation(kind, amount)),
        KIND_LAND => {
            let mut set: Vec<TileIdx> = Vec::new();
            if let Some((q, r)) = cell {
                let found = world.cell_tiles(Axial::new(q, r)).map_err(|_| {
                    VerbError::new_err(format!(
                        "the address ({q}, {r}) lies outside the world, so it names no cell"
                    ))
                })?;
                set.extend(found);
            }
            for (q, r) in tiles.unwrap_or_default() {
                let tile = world.grid().index_of(Axial::new(q, r)).ok_or_else(|| {
                    VerbError::new_err(format!("the address ({q}, {r}) lies outside the world"))
                })?;
                set.push(tile);
            }
            if set.is_empty() {
                return Err(VerbError::new_err(
                    "a land side names a cell, a list of tiles, or both",
                ));
            }
            Ok(Consideration::land(set))
        }
        other => Err(VerbError::new_err(format!(
            "{other} names no consideration kind: zero is a resource, one is land and two is a relation step"
        ))),
    }
}

fn trade_refusal(error: cachette_core::TradeError) -> PyErr {
    use cachette_core::TradeError as Refusal;
    let said = match error {
        Refusal::SameFaction(faction) => {
            format!("the faction {} does not trade with itself", faction.0)
        }
        Refusal::NoSuchFaction(faction) => {
            format!("{} names no faction of this world", faction.0)
        }
        Refusal::NoSuchKind(kind) => format!("{kind} names no resource kind"),
        Refusal::EmptyTerms => "each side of a contract binds a quantity above zero".to_string(),
        Refusal::NoDeadline => "a contract runs for a term above zero, because a contract that cannot fail is not a contract".to_string(),
        Refusal::AlreadyOpen => {
            "the two parties already hold a live negotiation or a live contract".to_string()
        }
        Refusal::NothingOpen => "the two parties hold no live negotiation".to_string(),
        Refusal::NotYourTurn => {
            "the other party has not answered yet, so this party may not speak again".to_string()
        }
        Refusal::AlreadyBound => {
            "the terms bind both parties, so nobody restates them and nobody refuses them"
                .to_string()
        }
        Refusal::Closed(until) => format!(
            "a terminal refusal closed this direction, and it opens again at step {}",
            until.0
        ),
        Refusal::NothingClosed => "this party closed nothing toward the other party".to_string(),
        Refusal::NoPresence => {
            "no unit of the speaker stands on ground that the other party holds".to_string()
        }
        Refusal::AtWar => {
            "one of the pair is in the war band toward the other, so no offer opens".to_string()
        }
        Refusal::NoDuration => {
            "a terminal refusal closes the direction for a number of steps above zero".to_string()
        }
        Refusal::NoSuchTag(tag) => format!(
            "{tag} names no consideration kind: zero is a resource, one is land and two is a relation step"
        ),
        Refusal::NoSuchTile => "a land side names a tile that lies outside the world".to_string(),
        Refusal::NoSuchCell => "a land side names a cell that lies outside the world".to_string(),
        Refusal::LandNotHeld(tile) => format!(
            "the party that owes the land does not hold tile {}",
            tile.0
        ),
        Refusal::UpgradeOnLand(tile) => format!(
            "tile {} carries an upgrade, and whether an upgrade changes hands with the ground is open under BLK-036, so the engine refuses the trade until it is answered",
            tile.0
        ),
        Refusal::TooMuchLand(count, bound) => format!(
            "a land side names {count} tiles, and the bound is {bound}"
        ),
        Refusal::NoSuchSide(wants) => format!(
            "{wants} names neither offers, which is zero, nor wants, which is one"
        ),
        Refusal::BoardOverfull(count, bound) => format!(
            "the board holds {bound} rows, and the write names {count}"
        ),
    };
    VerbError::new_err(said)
}

/// Returns the version of the `cachette` package, as a `str`.
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
