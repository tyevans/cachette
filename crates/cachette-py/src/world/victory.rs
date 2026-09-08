//! The rules that end a run, and the readers that score it.
//!
//! This module holds the renown target, the renown a fell earns, the wonder
//! work and its victory claim, the switch that turns the win readers on, the
//! end of the game, the score and the standing.
//!
//! The grouping is by what the values govern. Every method here writes or
//! reads a condition on which the run ends.

use super::PyWorld;
use crate::errors::VerbError;
use cachette_core::FactionId;
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pymethods]
impl PyWorld {
    /// Sets the renown at which the renown reader fires.
    ///
    /// The argument is a raw Q16.16 integer. Multiply a whole number of
    /// renown points by 65536 to reach it.
    ///
    /// **This is a threshold.** It decides when the renown reader fires and
    /// changes nothing else that the simulation does. The value is state
    /// that every tick reads, so two worlds that differ in it hash
    /// differently.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decisions D1 and D2. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
    fn set_renown_target(&self, raw: i32) {
        self.lock().set_renown_target(raw);
    }

    /// Sets the renown that one felled unit gives the champion of the faction
    /// that felled it.
    ///
    /// The argument is a raw Q16.16 integer. Multiply a whole number of
    /// renown points by 65536 to reach it.
    ///
    /// **This is a rate.** The renown column is state that a later frame
    /// reads, so a change to this value changes what the simulation
    /// does.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decision D2. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
    fn set_renown_per_fell(&self, raw: i32) {
        self.lock().set_renown_per_fell(raw);
    }

    /// Sets the work that finishes a wonder.
    ///
    /// The work is a column of the upgrade table row that holds the wonder.
    /// This writes that column and leaves every other column of the row where
    /// it is.
    ///
    /// **This is a rate.** A wonder that costs more work takes longer to
    /// build, so the value changes what the simulation does.[^1]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the table holds no wonder row.
    ///
    /// # References
    ///
    /// [^1]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decision D2. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
    fn set_wonder_work(&self, work: u32) -> PyResult<()> {
        if self.lock().set_wonder_work(work) {
            Ok(())
        } else {
            Err(VerbError::new_err("the table holds no wonder row"))
        }
    }

    /// Sets the victory claim that the wonder row carries.
    ///
    /// The claim is a column of the upgrade table row that holds the wonder.
    /// This writes that column and leaves every other column of the row where
    /// it is.
    ///
    /// **This is a threshold.** The wonder reader fires for the faction that
    /// holds a standing claim above zero, so a claim of zero takes the wonder
    /// path out of the game and changes nothing else.[^1]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the table holds no wonder row.
    ///
    /// # References
    ///
    /// [^1]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decision D2. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
    fn set_wonder_victory_claim(&self, claim: u32) -> PyResult<()> {
        if self.lock().set_wonder_victory_claim(claim) {
            Ok(())
        } else {
            Err(VerbError::new_err("the table holds no wonder row"))
        }
    }

    /// Sets whether the game end readers run.
    ///
    /// While they do not run, no reader records a game end, `game_end`
    /// returns `None`, and the world runs to the tick limit.
    ///
    /// **A run with the readers off holds the same event log as a run with
    /// the readers on that never fires.** A reader decides when the step
    /// stops watching, and it changes nothing else. One run to the limit with
    /// the readers off therefore gives the trajectory that scores any set of
    /// thresholds, and a second run is not necessary.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decision D3. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
    fn set_win_readers_enabled(&self, enabled: bool) {
        self.lock().set_win_readers_enabled(enabled);
    }

    /// The renown at which the renown reader fires, as a raw Q16.16 integer.
    ///
    /// A world that nobody configures holds the constant that names the
    /// default. Divide by 65536 for whole renown points.
    #[getter]
    fn renown_target(&self) -> i32 {
        self.lock().balance().renown_target()
    }

    /// The renown one felled unit gives the champion of the faction that
    /// felled it, as a raw Q16.16 integer.
    #[getter]
    fn renown_per_fell(&self) -> i32 {
        self.lock().balance().renown_per_fell().0
    }

    /// Whether the game end readers run, as a `bool`.
    #[getter]
    fn win_readers_enabled(&self) -> bool {
        self.lock().balance().win_readers_enabled()
    }

    /// The work that finishes a wonder, as an integer.
    ///
    /// The value is the work column of the upgrade table row that holds the
    /// wonder. Returns `None` when the table holds no wonder row.
    #[getter]
    fn wonder_work(&self) -> Option<u32> {
        self.lock()
            .upgrade_table()
            .row(
                cachette_core::upgrade::UpgradeCategory::WONDER,
                cachette_core::upgrade::WONDER_LEVEL,
            )
            .map(|row| row.work)
    }

    /// The victory claim the wonder row carries, as an integer.
    ///
    /// The wonder reader fires for the faction that holds a standing claim
    /// above zero. Returns `None` when the table holds no wonder row.
    #[getter]
    fn wonder_victory_claim(&self) -> Option<u32> {
        self.lock()
            .upgrade_table()
            .row(
                cachette_core::upgrade::UpgradeCategory::WONDER,
                cachette_core::upgrade::WONDER_LEVEL,
            )
            .map(|row| row.victory_claim)
    }

    /// Returns the game end record, as a `dict`, or `None` while no game has
    /// ended.
    ///
    /// The keys are `winner`, an integer naming the faction; `path`, a `str`
    /// naming the way it won, one of `domination`, `territory`, `wonder` and
    /// `renown`; and `tick`, the tick the reader fired on. **The record is
    /// written once.** After it the controller emits nothing and every other
    /// pass continues, so the world keeps stepping and the picture keeps
    /// moving.[^1] [^2]
    ///
    /// **The method returns `None` while the readers are off.** A caller
    /// that turns them off asks the world to run to the tick limit and to
    /// record no end.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0148, a game end is recorded once and stops the controllers, decisions D2 and D4. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
    /// [^2]: ADR-0174, a wonder is a win path and a stock total is not, decisions D1 and D3. `docs/adrs/draft/adr-0174-a-wonder-is-a-win-path-and-a-stock-total-is-not.md`
    /// [^3]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decision D3. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
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

    /// Returns the running value of one faction on each win path, as a
    /// `dict`.
    ///
    /// The keys are `held_tiles`, the tiles the faction holds; `seats_held`,
    /// the seats it holds, its own and every rival's; `live_units`, the units
    /// of the faction that are alive; `store_total`, the sum of every store
    /// of every settlement of the faction **as a raw Q16.16 integer**;
    /// `best_renown`, the highest renown of any live character of the
    /// faction, **as a raw Q16.16 integer**; and `wonder_progress`, the most
    /// work any wonder on ground the faction holds has reached.
    ///
    /// Every key except `store_total` feeds a reader, so a caller can watch
    /// each path approach its end.[^1] `held_tiles` feeds territory,
    /// `seats_held` and `live_units` feed domination, `best_renown` feeds
    /// renown, and `wonder_progress` is how far the furthest unfinished
    /// wonder has come. **`store_total` feeds no reader**, because a stock
    /// total wins no game. It is reported so that a caller may watch a
    /// faction grow rich.[^2]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the number names no faction of this world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0148, a game end is recorded once and stops the controllers, decision D1. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
    /// [^2]: ADR-0174, a wonder is a win path and a stock total is not, decisions D2 and D4. `docs/adrs/draft/adr-0174-a-wonder-is-a-win-path-and-a-stock-total-is-not.md`
    fn standing<'py>(&self, python: Python<'py>, faction: u16) -> PyResult<Bound<'py, PyDict>> {
        let standing = self.lock().standing(FactionId(faction)).ok_or_else(|| {
            VerbError::new_err(format!("{faction} names no faction of this world"))
        })?;
        let report = PyDict::new(python);
        report.set_item("held_tiles", standing.held_tiles)?;
        report.set_item("seats_held", standing.seats_held)?;
        report.set_item("live_units", standing.live_units)?;
        report.set_item("store_total", standing.store_total)?;
        report.set_item("best_renown", standing.best_renown)?;
        report.set_item("wonder_progress", standing.wonder_progress)?;
        Ok(report)
    }
}
