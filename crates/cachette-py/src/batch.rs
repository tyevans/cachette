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
//! # The workers live as long as the batch
//!
//! The batch builds its workers once, when a caller builds the batch, and
//! it keeps them until the batch goes. A step sends one order to each
//! worker and waits for one report from each. **No step builds a thread and
//! no step ends one.**[^5]
//!
//! The caller states the worker count when it builds the batch. The batch
//! reads no core count of the machine, because a training run puts several
//! processes on one machine and each of them would then claim the whole
//! machine.[^6]
//!
//! # The order is the index order, and never the completion order
//!
//! Each worker collects the result of every world it stepped, and it carries
//! the index of that world beside the result. The batch reads one report
//! from each worker, sorts the whole collection by that index, and returns
//! it. **No result reaches the caller in the order a thread finished.**[^4]
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
//! [^5]: ADR-0191, the workers of a batch outlive the step, decision D1. `docs/adrs/draft/adr-0191-the-workers-of-a-batch-outlive-the-step.md`
//! [^6]: ADR-0191, the workers of a batch outlive the step, decision D2. `docs/adrs/draft/adr-0191-the-workers-of-a-batch-outlive-the-step.md`

use std::panic::AssertUnwindSafe;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use pyo3::prelude::*;

use crate::{ConfigError, PyWorld, StepError};

/// What one world of a batch did in one step.
///
/// The step entry holds the event count when the world stepped, and the
/// error entry holds the message when it refused. Exactly one of the two is
/// set.
type StepOutcome = Result<usize, String>;

/// What one worker did with the worlds it holds.
///
/// The pair holds the index of the world and the outcome of its step. A
/// worker reports the pairs of every world it stepped, in the order it
/// stepped them. The batch sorts the whole collection afterwards.
type WorkerReport = Result<Vec<(usize, StepOutcome)>, ()>;

/// One order to one worker of a batch.
///
/// The order carries the thread count that the worker gives to each step it
/// runs. The worker already knows which worlds it holds, because the stride
/// never changes for the life of the batch.
struct Order {
    /// How many threads one world may give to its own step.
    threads: usize,
}

/// One worker of a batch, and the channel that reaches it.
struct Worker {
    /// The channel that carries an order to this worker.
    orders: Sender<Order>,
    /// The channel that carries this worker's report back.
    ///
    /// Each worker owns a channel of its own. A worker that dies closes its
    /// own channel, so the batch reads a refusal rather than waiting for a
    /// report that will never come.
    reports: Receiver<WorkerReport>,
    /// The thread itself, so that the batch can join it when it goes.
    handle: Option<JoinHandle<()>>,
}

/// The workers of one batch.
///
/// The pool builds its threads once and keeps them. A step sends one order
/// to each worker and reads one report from each.[^1]
///
/// # References
///
/// [^1]: ADR-0191, the workers of a batch outlive the step, decision D1. `docs/adrs/draft/adr-0191-the-workers-of-a-batch-outlive-the-step.md`
struct Pool {
    /// The workers, in the order the batch built them.
    workers: Vec<Worker>,
}

impl Pool {
    /// Builds one pool of the stated size over the worlds of a batch.
    ///
    /// Worker `w` takes the worlds `w`, `w + workers`, and so on. Each index
    /// belongs to one worker, so no two workers reach one world.
    fn build(worlds: Arc<Vec<Py<PyWorld>>>, workers: usize) -> Self {
        let count = worlds.len();
        let mut built = Vec::with_capacity(workers);
        for first in 0..workers {
            let (orders, taken) = channel::<Order>();
            let (given, reports) = channel::<WorkerReport>();
            let held = Arc::clone(&worlds);
            let handle = std::thread::spawn(move || {
                while let Ok(order) = taken.recv() {
                    // A world that panics must not take the worker with it.
                    // The batch reports the panic at the call that caused
                    // it, and the worker takes the next order.
                    let report = std::panic::catch_unwind(AssertUnwindSafe(|| {
                        let mut mine = Vec::new();
                        let mut index = first;
                        while index < count {
                            let outcome = {
                                let mut world = held[index].get().lock();
                                world
                                    .step(order.threads)
                                    .map(|events| events.len())
                                    .map_err(|error| error.to_string())
                            };
                            mine.push((index, outcome));
                            index += workers;
                        }
                        mine
                    }));
                    if given.send(report.map_err(|_| ())).is_err() {
                        break;
                    }
                }
            });
            built.push(Worker {
                orders,
                reports,
                handle: Some(handle),
            });
        }
        Self { workers: built }
    }

    /// Steps every world of the batch once, and returns the results in index
    /// order.
    ///
    /// The caller must not hold the interpreter while this runs. The batch
    /// releases it around this call.
    ///
    /// # Errors
    ///
    /// Returns a message when a world panics, and when a worker is gone.
    fn run(&self, threads: usize, count: usize) -> Result<Vec<(usize, StepOutcome)>, String> {
        for worker in &self.workers {
            if worker.orders.send(Order { threads }).is_err() {
                return Err("a worker of the batch is gone".to_owned());
            }
        }
        // Every worker that took an order reports before this call returns.
        // A report left in a channel would reach the next step and pair a
        // result with the wrong call, so the loop below reads them all
        // before it acts on a failure.
        let mut ordered: Vec<(usize, StepOutcome)> = Vec::with_capacity(count);
        let mut refusal: Option<String> = None;
        for worker in &self.workers {
            match worker.reports.recv() {
                Ok(Ok(mine)) => ordered.extend(mine),
                Ok(Err(())) => {
                    refusal.get_or_insert_with(|| "a world of the batch panicked".to_owned());
                }
                Err(_) => {
                    refusal.get_or_insert_with(|| "a worker of the batch is gone".to_owned());
                }
            }
        }
        if let Some(message) = refusal {
            return Err(message);
        }
        // The stable key is the index of the world. This is the one place
        // the order of the batch is fixed, and it is fixed here rather than
        // by the order the workers finished or by the stride that gave each
        // worker its worlds.
        #[cfg(not(feature = "perturb-batch-report-order"))]
        ordered.sort_by_key(|(index, _)| *index);
        Ok(ordered)
    }
}

impl Drop for Pool {
    /// Ends every worker and waits for it.
    ///
    /// Dropping the order channel closes it, and a worker whose order
    /// channel is closed returns. The join then waits for a worker that is
    /// still inside a step.
    fn drop(&mut self) {
        let mut handles = Vec::with_capacity(self.workers.len());
        for worker in std::mem::take(&mut self.workers) {
            let Worker {
                orders,
                reports,
                handle,
            } = worker;
            drop(orders);
            drop(reports);
            handles.push(handle);
        }
        for handle in handles.into_iter().flatten() {
            let _ = handle.join();
        }
    }
}

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
///
/// **The batch builds its workers once.** The caller states how many when it
/// builds the batch, and the workers live until the batch goes.
#[pyclass(name = "Batch", module = "cachette._core", frozen)]
pub struct PyBatch {
    /// The worlds, in the order the caller gave them.
    ///
    /// The workers hold the same list, so the list is shared and never
    /// copied. A world object is safe to hold without the interpreter,
    /// because the world class is frozen.
    worlds: Arc<Vec<Py<PyWorld>>>,
    /// The workers of this batch.
    ///
    /// The lock serialises two callers that step one batch at the same time.
    /// Two callers that shared the workers without it would read each
    /// other's reports.
    pool: Mutex<Pool>,
    /// How many workers the batch built.
    workers: usize,
}

#[pymethods]
impl PyBatch {
    /// Builds a batch over the worlds a caller already holds.
    ///
    /// The `workers` count is how many worlds the batch steps at one time.
    /// The batch builds that many threads now and keeps them. A batch that
    /// holds fewer worlds than that builds one worker for each world, and
    /// the `workers` property reports the count it built.
    ///
    /// # Errors
    ///
    /// Raises `ConfigError` when the list holds one world twice, when the
    /// list is empty, and when the worker count is zero.
    #[new]
    #[pyo3(signature = (worlds, workers))]
    fn new(worlds: Vec<Py<PyWorld>>, workers: usize) -> PyResult<Self> {
        if worlds.is_empty() {
            return Err(ConfigError::new_err("a batch holds at least one world"));
        }
        if workers == 0 {
            return Err(ConfigError::new_err("the worker count is zero"));
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
        // A worker with no world would send an empty report on every step.
        // The batch builds one worker for each world instead.
        let workers = workers.min(worlds.len());
        let worlds = Arc::new(worlds);
        let pool = Pool::build(Arc::clone(&worlds), workers);
        Ok(Self {
            worlds,
            pool: Mutex::new(pool),
            workers,
        })
    }

    /// Returns how many worlds the batch holds.
    fn __len__(&self) -> usize {
        self.worlds.len()
    }

    /// Returns how many workers the batch built.
    ///
    /// This is the count the caller asked for, or the world count when the
    /// caller asked for more workers than the batch holds worlds.
    #[getter]
    fn workers(&self) -> usize {
        self.workers
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
    /// The `threads` count is how many threads one world may give to its own
    /// step. The worker count is the one the caller gave the constructor.
    /// **They are two counts, and neither implies the other.**
    ///
    /// The call releases the global interpreter lock for the whole batch. No
    /// Python code runs while any world of the batch runs.
    ///
    /// # Errors
    ///
    /// Raises `StepError` when the thread count is zero, and when a world
    /// panics.
    #[pyo3(signature = (threads))]
    fn step(&self, python: Python<'_>, threads: usize) -> PyResult<Vec<StepRow>> {
        if threads == 0 {
            return Err(StepError::new_err("the thread count is zero"));
        }
        let count = self.worlds.len();
        // ADR-0042: release the interpreter for the whole batch. The closure
        // may not capture the interpreter token, so no Python code can run
        // inside it.
        let ordered = python.detach(|| {
            let pool = self.pool.lock().unwrap_or_else(|error| error.into_inner());
            pool.run(threads, count)
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
