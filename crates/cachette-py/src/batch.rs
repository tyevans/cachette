//! A batch of worlds steps in one call, and reports in index order.
//!
//! A training run steps many worlds. The single step of the world type
//! takes one world, so a caller that wants many worlds crosses the boundary
//! once for each of them. This module adds a batch type beside the world
//! type, and the batch steps every world it holds in one call.[^1]
//!
//! # A world of a batch is the world a caller builds alone
//!
//! The batch holds the world objects the caller built. It builds no world
//! of its own, and it copies no world. A caller reads the observation of a
//! world of the batch through that same world object.[^2]
//!
//! # Two counts, and they are not the same count
//!
//! The world count of one call and the thread count of one world are
//! separate parameters. The batch spreads its worlds over its own workers,
//! and each worker gives the thread count of one world to the step it
//! runs.[^3]
//!
//! # The order is the index order, and never the completion order
//!
//! Each worker collects the result of every world it stepped, and it carries
//! the index of that world beside the result. The batch joins the workers,
//! sorts the whole collection by that index, and returns it. **No result
//! reaches the caller in the order a thread finished.**[^4]
//!
//! # A world that fails names its own index
//!
//! A step that refuses reports its refusal at the index of the world that
//! refused, and the other worlds of the batch finish.[^1] A failure a caller
//! cannot attribute to one index is worse than a failure that stops the
//! call, so the batch attributes every one.
//!
//! # Determinism
//!
//! The worlds of a batch share nothing. Each worker locks one world at a
//! time, and no two workers hold one world, because the batch refuses a
//! world it already holds. A batch of one world at one worker and a batch of
//! many worlds at many workers therefore leave each world in the state the
//! single step would have left it.
//!
//! # References
//!
//! [^1]: ADR-0155, a batch of worlds steps in one call, in index order, decision D1. `docs/adrs/accepted/adr-0155-a-batch-of-worlds-steps-in-one-call-in-index-order.md`
//! [^2]: ADR-0155, a batch of worlds steps in one call, in index order, decision D3. `docs/adrs/accepted/adr-0155-a-batch-of-worlds-steps-in-one-call-in-index-order.md`
//! [^3]: ADR-0155, a batch of worlds steps in one call, in index order, decision D4. `docs/adrs/accepted/adr-0155-a-batch-of-worlds-steps-in-one-call-in-index-order.md`
//! [^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`

use pyo3::prelude::*;

use crate::{ConfigError, PyWorld, StepError};

/// What one world of a batch did in one step.
///
/// The step entry holds the event count when the world stepped, and the
/// error entry holds the message when it refused. Exactly one of the two is
/// set.
type StepOutcome = Result<usize, String>;

/// Many worlds, stepped in one crossing of the boundary.
///
/// A caller builds the worlds it wants, hands them to a batch, and steps the
/// batch. The batch holds a reference to each world, so a caller reads the
/// observation, the legality answer and the reward of a world through the
/// world object it already holds.
///
/// **The batch refuses to hold one world twice.** Two entries of one world
/// would put two workers on one lock, and the second worker would wait for
/// the first for as long as the batch runs.
#[pyclass(name = "Batch", module = "cachette._core", frozen)]
pub struct PyBatch {
    /// The worlds, in the order the caller gave them.
    worlds: Vec<Py<PyWorld>>,
}

#[pymethods]
impl PyBatch {
    /// Builds a batch over the worlds a caller already holds.
    ///
    /// # Errors
    ///
    /// Raises `ConfigError` when the list holds one world twice, and when
    /// the list is empty.
    #[new]
    fn new(worlds: Vec<Py<PyWorld>>) -> PyResult<Self> {
        if worlds.is_empty() {
            return Err(ConfigError::new_err("a batch holds at least one world"));
        }
        for (index, world) in worlds.iter().enumerate() {
            for (earlier, other) in worlds.iter().enumerate().take(index) {
                if std::ptr::eq(world.as_ptr(), other.as_ptr()) {
                    return Err(ConfigError::new_err(format!(
                        "the world at index {index} is the world at index {earlier}, \
                         and a batch holds one world once"
                    )));
                }
            }
        }
        Ok(Self { worlds })
    }

    /// Returns how many worlds the batch holds.
    fn __len__(&self) -> usize {
        self.worlds.len()
    }

    /// Returns the world at one index of the batch.
    ///
    /// # Errors
    ///
    /// Raises `IndexError` when the index names no world of this batch.
    fn world(&self, index: usize, python: Python<'_>) -> PyResult<Py<PyWorld>> {
        self.worlds
            .get(index)
            .map(|world| world.clone_ref(python))
            .ok_or_else(|| {
                pyo3::exceptions::PyIndexError::new_err(format!(
                    "{index} names no world of this batch of {}",
                    self.worlds.len()
                ))
            })
    }

    /// Steps every world of the batch once, and returns one result for each,
    /// in index order.
    ///
    /// The result at index `i` belongs to the world at index `i`. It is the
    /// event count of that step, or `None` when the world refused the step.
    /// Read `errors` for the message of a world that refused.
    ///
    /// The `workers` count is how many worlds the batch steps at one time.
    /// The `threads` count is how many threads one world may give to its own
    /// step. **They are two counts, and neither implies the other.**
    ///
    /// The call releases the global interpreter lock for the whole batch. No
    /// Python code runs while any world of the batch runs.
    ///
    /// # Errors
    ///
    /// Raises `StepError` when either count is zero, and when a world panics.
    #[pyo3(signature = (workers, threads))]
    fn step(&self, python: Python<'_>, workers: usize, threads: usize) -> PyResult<Vec<StepRow>> {
        if workers == 0 {
            return Err(StepError::new_err("the worker count is zero"));
        }
        if threads == 0 {
            return Err(StepError::new_err("the thread count is zero"));
        }
        // The frozen class is `Sync`, so a reference to one world crosses to
        // a worker without the interpreter. The batch owns the strong
        // reference for the whole call, so no world can be collected here.
        let worlds: Vec<&PyWorld> = self.worlds.iter().map(|world| world.get()).collect();
        let count = worlds.len();
        let workers = workers.min(count);
        // ADR-0042: release the interpreter for the whole batch. The closure
        // may not capture the interpreter token, so no Python code can run
        // inside it.
        let ordered = python.detach(|| -> Result<Vec<(usize, StepOutcome)>, String> {
            std::thread::scope(|scope| {
                let mut handles = Vec::with_capacity(workers);
                for worker in 0..workers {
                    let worlds = &worlds;
                    handles.push(scope.spawn(move || {
                        // Worker `w` takes the worlds `w`, `w + workers`,
                        // and so on. Each index belongs to one worker, so
                        // no two workers reach one world.
                        let mut mine = Vec::new();
                        let mut index = worker;
                        while index < count {
                            let outcome = {
                                let mut world = worlds[index].lock();
                                world
                                    .step(threads)
                                    .map(|events| events.len())
                                    .map_err(|error| error.to_string())
                            };
                            mine.push((index, outcome));
                            index += workers;
                        }
                        mine
                    }));
                }
                let mut ordered: Vec<(usize, StepOutcome)> = Vec::with_capacity(count);
                for handle in handles {
                    match handle.join() {
                        Ok(mine) => ordered.extend(mine),
                        Err(_) => return Err("a world of the batch panicked".to_owned()),
                    }
                }
                // The stable key is the index of the world. This is the one
                // place the order of the batch is fixed, and it is fixed
                // here rather than by the order the workers finished.
                ordered.sort_by_key(|(index, _)| *index);
                Ok(ordered)
            })
        });
        let ordered = ordered.map_err(StepError::new_err)?;
        Ok(ordered
            .into_iter()
            .map(|(index, outcome)| match outcome {
                Ok(events) => StepRow {
                    index,
                    events: Some(events),
                    error: None,
                },
                Err(message) => StepRow {
                    index,
                    events: None,
                    error: Some(message),
                },
            })
            .collect())
    }
}

/// What one world of a batch did in one step of the batch.
///
/// The index entry is the index of the world in the batch. The events entry
/// is the event count of the step, and it is `None` when the world refused
/// the step. The error entry is the message of that refusal, and it is
/// `None` when the world stepped.
#[pyclass(name = "StepRow", module = "cachette._core", frozen, get_all)]
pub struct StepRow {
    /// The index of the world in the batch.
    index: usize,
    /// The event count of the step, or `None` when the world refused.
    events: Option<usize>,
    /// The message of the refusal, or `None` when the world stepped.
    error: Option<String>,
}

#[pymethods]
impl StepRow {
    /// Returns a readable form of the row.
    fn __repr__(&self) -> String {
        match (&self.events, &self.error) {
            (Some(events), _) => format!("StepRow(index={}, events={events})", self.index),
            (None, Some(error)) => format!("StepRow(index={}, error={error:?})", self.index),
            (None, None) => format!("StepRow(index={})", self.index),
        }
    }
}
