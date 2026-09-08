//! The `World` class, and the modules that hold its methods.
//!
//! The class declaration and the state it holds sit here. Each sibling module
//! holds the methods of one domain, in its own `#[pymethods]` block. The
//! binding library collects the blocks of one class, so the split is a
//! grouping of source text and not a change to the class.[^1]
//!
//! The grouping is by the domain a method serves. A reader who wants the
//! weather opens one file and reads every weather method in it.
//!
//! # References
//!
//! [^1]: The `multiple-pymethods` feature of the binding library. `crates/cachette-py/Cargo.toml`

mod characters;
mod construction;
mod event_log;
mod faction_view;
mod fire;
mod founding;
pub(crate) mod identity;
mod luxuries;
mod production;
mod relations;
mod settlement;
mod step;
mod territory;
mod tiles;
mod trade;
mod units;
mod victory;
mod view;
pub(crate) mod weather;

use cachette_core::founding::FoundingOutcome;
use cachette_core::World as CoreWorld;
use cachette_view::{Metrics, Motion};
use pyo3::prelude::*;

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
/// **The weather runs on a lattice of its own, and `weather_cell_tiles`
/// states its pitch.** The number is the side of one weather cell in tiles.
/// It must be a power of two from 1 to 256. One gives each tile its own
/// weather. `None` takes the pitch the engine defaults to, which is the
/// level 1 block.
///
/// The cost of the weather stage follows the cell count, and the cell count
/// is the tile count divided by the square of the pitch. A pitch of one on a
/// large world therefore costs the frame. Read `weather_cell_count` for how
/// many cells of the world a built world holds.
///
/// **The engine steps more cells than the world holds.** It simulates a margin
/// of cells around the world, so that the border of the world has real upwind
/// rather than an edge that nothing crosses. The margin is a distance, so the
/// share it adds to the cost falls as the world grows. No reader sees a margin
/// cell, and every weather array covers the world alone.
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
    /// Where each painted unit stood on the last frame.
    ///
    /// **The world holds none of this.** A viewer that draws a unit between
    /// two tiles must remember the first of them, because the world holds one
    /// tick at a time, and that memory belongs to the caller.[^1] The binding
    /// is the caller here.
    ///
    /// The table holds an entry for a unit painted on the last frame and for
    /// no other unit, and the frame bounds it by its own pixels. A frame
    /// cannot show more units than it has pixels.
    ///
    /// # References
    ///
    /// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
    motion: Motion,
}

impl PyWorld {
    /// Takes the lock, recovering from a poisoned lock.
    pub(crate) fn lock(&self) -> std::sync::MutexGuard<'_, CoreWorld> {
        self.inner.lock().unwrap_or_else(|error| error.into_inner())
    }

    /// Takes the caller's own values, recovering from a poisoned lock.
    fn presenter(&self) -> std::sync::MutexGuard<'_, Presenter> {
        self.presenter
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }
}
