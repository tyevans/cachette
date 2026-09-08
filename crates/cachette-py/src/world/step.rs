//! The frame step, and nothing else.
//!
//! The step releases the global interpreter lock for its whole run. No Python
//! code runs while the simulation runs, and no system calls Python.[^1] Events
//! reach Python in batches at the frame barrier.[^2]
//!
//! This module holds one method. The step is the whole boundary, so a change
//! to it is a change to this file alone and a reader sees it whole.
//!
//! # References
//!
//! [^1]: ADR-0042, the interpreter is released for the whole step, decision D1. `docs/adrs/draft/adr-0042-the-interpreter-is-released-for-the-whole-step.md`
//! [^2]: ADR-0040, Python is a control plane, not a data plane. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`

use super::PyWorld;
use crate::errors::StepError;
use cachette_view::Lap;
use pyo3::prelude::*;

#[pymethods]
impl PyWorld {
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
}
