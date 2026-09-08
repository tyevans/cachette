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

mod batch;
mod camera;
mod columns;
mod errors;
pub mod logs;
mod world;

pub use crate::camera::PyCamera;
pub use crate::world::PyWorld;

use crate::batch::{PyBatch, StepRow};
use crate::errors::{
    CachetteError, ConfigError, DeterminismError, EnginePanic, FrameError, SelectorError,
    StepError, VerbError, ViewError,
};
use cachette_core::event_layout::declared_event_layouts;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::PyTypeInfo;

/// Returns the columns that every event log gives, as a `dict`.
///
/// The key is the name of an event. The value is a list of pairs. The first
/// entry of a pair is the name of a column, and the second is the name of its
/// NumPy element type. The order is the order the engine declares the fields
/// in. A padding field crosses nowhere, so it has no pair.
///
/// The engine declares the fields of an event in one place, and this function
/// reports that declaration. A reader that builds a type from it holds no
/// copy of the layout. The type stub of this module is built from it, and a
/// check fails when the stub and the engine disagree.[^1]
///
/// No element type is a floating point type. A fixed-point column crosses as
/// its raw integer.[^2]
///
/// # Errors
///
/// Returns an error when the interpreter refuses to hold the dictionary.
///
/// # References
///
/// [^1]: ADR-0163, an event declares its layout once and the binding derives every column. `docs/adrs/draft/adr-0163-an-event-declares-its-layout-once-and-the-binding-derives-every-column.md`
/// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
#[pyfunction]
fn event_schema(python: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
    let schema = PyDict::new(python);
    for event in declared_event_layouts() {
        let fields: Vec<(&str, &str)> = event
            .fields
            .iter()
            .filter_map(|field| match (field.column, field.kind) {
                (Some(column), Some(kind)) => Some((column, kind.numpy_name())),
                _ => None,
            })
            .collect();
        schema.set_item(event.name, fields)?;
    }
    Ok(schema)
}

/// Returns the colours the viewer paints the factions in, as a `list`.
///
/// Each entry is a colour as one integer, red in the highest byte, then
/// green, then blue. The entry at index `n` is the colour of faction `n`. A
/// faction beyond the end of the list wraps to a colour it shares, which is
/// a display limit and not a simulation one.
///
/// **The viewer states this table once, and a caller reads it here.** A
/// script that wrote its own copy painted a faction in a colour the viewer no
/// longer used, and nothing failed.[^1]
///
/// The colours belong to the viewer. The simulated state holds no colour.[^2]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
/// [^2]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
#[pyfunction]
fn faction_colours() -> Vec<u32> {
    cachette_view::paint::faction_colours().to_vec()
}

/// Returns the version of the `cachette` package, as a `str`.
///
/// The value is the version of the compiled extension module. The package
/// exposes the same string as `cachette.__version__`.
#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Returns the stock one settlement can hold, as an `int`.
///
/// The value is a raw Q16.16 quantity summed over every commodity of one
/// settlement: the ceiling of a fixed-point store times the commodity count.
/// A faction that holds one settlement never reports more than this. The
/// ceiling is the reason no bar could put the wealth path out of reach, and
/// the reason the path has no reader today.[^1]
///
/// # References
///
/// [^1]: ADR-0173, the wealth or wonder path has no reader, decision D1. `docs/adrs/draft/adr-0173-the-wealth-or-wonder-path-has-no-reader.md`
#[pyfunction]
fn stock_ceiling_of_one_settlement() -> i64 {
    cachette_core::STOCK_CEILING_OF_ONE_SETTLEMENT
}

/// The compiled core of the Cachette simulation engine.
///
/// The module holds the `World` class, the `Camera` class, the module
/// functions and the error classes. This sentence does not list them,
/// because a list here is a second statement of the interface and the
/// generated reference is the first.[^7] The `cachette` package re-exports
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
/// [^7]: ADR-0107, the Python reference is generated from the compiled module, decision D1. `docs/adrs/draft/adr-0107-the-python-reference-is-generated-from-the-compiled-module.md`
#[pymodule]
#[pyo3(name = "_core")]
fn cachette_core_module(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyWorld>()?;
    module.add_class::<PyBatch>()?;
    module.add_class::<StepRow>()?;
    module.add_class::<PyCamera>()?;
    module.add_function(wrap_pyfunction!(version, module)?)?;
    module.add_function(wrap_pyfunction!(stock_ceiling_of_one_settlement, module)?)?;
    module.add_function(wrap_pyfunction!(event_schema, module)?)?;
    module.add_function(wrap_pyfunction!(faction_colours, module)?)?;
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
