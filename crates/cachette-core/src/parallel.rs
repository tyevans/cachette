//! The fan-out that runs one closure for each disjoint unit of parallel work.
//!
//! Every parallel stage in this crate divides its work into chunks, gives one
//! chunk to one closure, and writes each result into a slot that the chunk
//! index names.[^1] The stages used to write that shape by hand, and each one
//! opened a scope and spawned one operating system thread for each chunk.
//!
//! # Why one helper rather than a conditional at each site
//!
//! A chunk count of one is the common case. The step divides the work by the
//! thread count, so a step at one thread gives every stage a single chunk.
//! The old shape then created an operating system thread, gave it the whole
//! stage, and joined it at once. A measurement counted about fifteen thread
//! creations for each world tick at one thread, and every one of them carried
//! a map and an unmap of a signal stack.
//!
//! A conditional at each site would fix that, and a new site would forget it.
//! One helper cannot be forgotten, because a site that does not call it does
//! not compile against the same shape.
//!
//! # Why this does not change any answer
//!
//! [`fan_out`] runs the last unit of work on the calling thread and spawns a
//! thread for each of the others. Three properties make that safe.
//!
//! The helper visits the units in the order the caller gives them, and it
//! returns the results in that same order. The order does not depend on which
//! unit finished first.[^1]
//!
//! Each unit writes only its own output. Two units never touch the same byte,
//! so the calling thread running one of them observes what a spawned thread
//! would have observed.[^2]
//!
//! Nothing in a unit reads the identity of the thread that runs it. No unit
//! draws a random number from thread-local state, and every draw is keyed on
//! the system, the frame, the entity and the draw.[^3]
//!
//! A chunk count of one therefore creates no thread at all, and a chunk count
//! of `n` creates `n - 1`.
//!
//! # References
//!
//! [^1]: ADR-0004, iteration order is explicit, decision D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
//! [^2]: ADR-0009, parallel stages write disjoint outputs. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
//! [^3]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`

/// Counts the threads that [`fan_out`] has created since the last reset.
///
/// The counter is behind a feature, so a normal build holds no atomic in the
/// path of a stage. A test reads it to prove that a single chunk creates no
/// thread, which is a claim that no assertion on a result can make.[^1]
///
/// # References
///
/// [^1]: Testing Rules, a determinism test must be able to fail. `.agents/rules/testing.md`
#[cfg(feature = "probe-spawn-count")]
static SPAWNS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Records that the helper is about to create one thread.
#[cfg(feature = "probe-spawn-count")]
fn record_spawn() {
    SPAWNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

/// Records nothing, because the probe feature is off.
#[cfg(not(feature = "probe-spawn-count"))]
fn record_spawn() {}

/// Returns how many threads [`fan_out`] has created since the last reset.
#[cfg(feature = "probe-spawn-count")]
#[must_use]
pub fn spawn_count() -> u64 {
    SPAWNS.load(std::sync::atomic::Ordering::Relaxed)
}

/// Sets the thread creation count back to zero.
#[cfg(feature = "probe-spawn-count")]
pub fn reset_spawn_count() {
    SPAWNS.store(0, std::sync::atomic::Ordering::Relaxed);
}

use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;

struct FnOnceTask<F, T> {
    func: Option<F>,
    result: Option<T>,
}

impl<F, T> FnOnceTask<F, T>
where
    F: FnOnce() -> T + Send,
    T: Send,
{
    fn run(&mut self) {
        if let Some(f) = self.func.take() {
            self.result = Some(f());
        }
    }
}

unsafe fn run_task<F, T>(data: *mut ())
where
    F: FnOnce() -> T + Send,
    T: Send,
{
    // SAFETY: `data` was cast from `*mut FnOnceTask<F, T>` and the caller
    // remains blocked until completion, ensuring exclusive access.
    let task = unsafe { &mut *(data as *mut FnOnceTask<F, T>) };
    task.run();
}

struct RawTask {
    data: *mut (),
    call: unsafe fn(*mut ()),
}

unsafe impl Send for RawTask {}

enum WorkerOrder {
    Task(RawTask),
    Stop,
}

struct Worker {
    order_tx: Sender<WorkerOrder>,
    done_rx: Receiver<Result<(), Box<dyn std::any::Any + Send + 'static>>>,
    handle: Option<JoinHandle<()>>,
}

impl Worker {
    fn spawn() -> Self {
        record_spawn();
        let (order_tx, order_rx) = channel::<WorkerOrder>();
        let (done_tx, done_rx) = channel();
        let handle = std::thread::spawn(move || {
            while let Ok(order) = order_rx.recv() {
                match order {
                    WorkerOrder::Task(raw) => {
                        let outcome = catch_unwind(AssertUnwindSafe(|| {
                            // SAFETY: The calling thread remains blocked in `fan_out`
                            // until all workers report completion, ensuring that `raw.data`
                            // remains valid and uniquely accessed.
                            unsafe { (raw.call)(raw.data) };
                        }));
                        let _ = done_tx.send(outcome);
                    }
                    WorkerOrder::Stop => break,
                }
            }
        });
        Self {
            order_tx,
            done_rx,
            handle: Some(handle),
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.order_tx.send(WorkerOrder::Stop);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

/// A persistent pool of worker threads for parallel stages of a world.
///
/// The pool retains worker threads across the simulation steps of a world,
/// eliminating repeated thread creation and kernel scheduler overhead.[^4]
///
/// # References
///
/// [^4]: ADR-0047, many worlds live in one interpreter, decision D2. `docs/adrs/draft/adr-0047-many-worlds-live-in-one-interpreter.md`
pub struct StagePool {
    workers: std::sync::Mutex<Vec<Worker>>,
}

impl StagePool {
    /// Builds an empty stage worker pool.
    #[must_use]
    pub fn new() -> Self {
        Self {
            workers: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Builds a worker pool with the given initial capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let pool = Self::new();
        pool.ensure_workers(capacity);
        pool
    }

    /// Ensures that the pool contains at least `count` workers.
    pub fn ensure_workers(&self, count: usize) {
        let mut workers = self.workers.lock().expect("worker pool lock poisoned");
        while workers.len() < count {
            workers.push(Worker::spawn());
        }
    }

    /// Returns the number of workers in the pool.
    #[must_use]
    pub fn worker_count(&self) -> usize {
        self.workers
            .lock()
            .expect("worker pool lock poisoned")
            .len()
    }

    /// Activates this pool on the calling thread for the duration of the returned guard.
    #[must_use]
    pub fn activate(&self) -> ActivePoolGuard {
        let previous = ACTIVE_POOL.with(|cell| {
            let prev = cell.get();
            cell.set(Some(self as *const StagePool));
            prev
        });
        ActivePoolGuard { previous }
    }

    /// Runs closures across the workers of this pool, running the last unit on the caller.
    pub fn fan_out<'env, I, F, T>(&self, units: I) -> Vec<T>
    where
        I: IntoIterator<Item = F>,
        F: FnOnce() -> T + Send + 'env,
        T: Send + 'env,
    {
        let mut units: Vec<F> = units.into_iter().collect();
        let Some(last) = units.pop() else {
            return Vec::new();
        };
        if units.is_empty() {
            return vec![last()];
        }
        let k = units.len();
        self.ensure_workers(k);

        let mut tasks: Vec<FnOnceTask<F, T>> = units
            .into_iter()
            .map(|f| FnOnceTask {
                func: Some(f),
                result: None,
            })
            .collect();

        let workers = self.workers.lock().expect("worker pool lock poisoned");
        for (i, task) in tasks.iter_mut().enumerate() {
            let raw = RawTask {
                data: task as *mut FnOnceTask<F, T> as *mut (),
                call: run_task::<F, T>,
            };
            workers[i]
                .order_tx
                .send(WorkerOrder::Task(raw))
                .expect("worker thread disconnected");
        }

        let own = last();

        let mut panic_payload = None;
        for (i, _) in tasks.iter().enumerate() {
            match workers[i].done_rx.recv() {
                Ok(Ok(())) => {}
                Ok(Err(payload)) => {
                    if panic_payload.is_none() {
                        panic_payload = Some(payload);
                    }
                }
                Err(_) => panic!("worker thread unexpectedly disconnected"),
            }
        }
        drop(workers);

        if let Some(payload) = panic_payload {
            resume_unwind(payload);
        }

        let mut results = Vec::with_capacity(tasks.len() + 1);
        for mut task in tasks {
            results.push(task.result.take().expect("task result missing"));
        }
        results.push(own);
        results
    }
}

impl Default for StagePool {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for StagePool {
    fn clone(&self) -> Self {
        let count = self.worker_count();
        Self::with_capacity(count)
    }
}

impl std::fmt::Debug for StagePool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StagePool")
            .field("workers", &self.worker_count())
            .finish()
    }
}

thread_local! {
    static ACTIVE_POOL: std::cell::Cell<Option<*const StagePool>> = const { std::cell::Cell::new(None) };
}

/// A scope guard that deactivates a [`StagePool`] when dropped.
pub struct ActivePoolGuard {
    previous: Option<*const StagePool>,
}

impl Drop for ActivePoolGuard {
    fn drop(&mut self) {
        ACTIVE_POOL.with(|cell| cell.set(self.previous));
    }
}

/// Runs one closure for each unit of parallel work, and returns the results in
/// the order the caller gave the units.
///
/// The calling thread runs the last unit. Each other unit gets one worker from
/// the active pool if one is present, or one scoped thread otherwise.
/// A caller that gives one unit therefore creates no thread.
///
/// The results come back in unit order and never in completion order.[^1]
///
/// # Panics
///
/// Panics when a unit panics. The panic of the first unit that failed, in unit
/// order, reaches the caller.
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
pub fn fan_out<'env, I, F, T>(units: I) -> Vec<T>
where
    I: IntoIterator<Item = F>,
    F: FnOnce() -> T + Send + 'env,
    T: Send + 'env,
{
    let mut units: Vec<F> = units.into_iter().collect();
    let Some(last) = units.pop() else {
        return Vec::new();
    };
    if units.is_empty() {
        return vec![last()];
    }
    units.push(last);

    let active = ACTIVE_POOL.with(|cell| cell.get());
    if let Some(pool_ptr) = active {
        // SAFETY: ACTIVE_POOL is valid for the duration of the ActivePoolGuard.
        let pool = unsafe { &*pool_ptr };
        return pool.fan_out(units);
    }

    fan_out_fallback(units)
}

fn fan_out_fallback<'env, F, T>(mut units: Vec<F>) -> Vec<T>
where
    F: FnOnce() -> T + Send + 'env,
    T: Send + 'env,
{
    let last = units.pop().expect("units must not be empty");
    let mut results = Vec::with_capacity(units.len() + 1);
    std::thread::scope(|scope| {
        let handles: Vec<_> = units
            .into_iter()
            .map(|unit| {
                record_spawn();
                scope.spawn(unit)
            })
            .collect();
        let own = last();
        for handle in handles {
            match handle.join() {
                Ok(value) => results.push(value),
                Err(payload) => resume_unwind(payload),
            }
        }
        results.push(own);
    });
    results
}

/// Runs one closure for each unit of parallel work, and keeps no result.
///
/// This is [`fan_out`] for a stage that writes into a slot it already holds.
/// The calling thread runs the last unit, so a stage of one chunk creates no
/// thread.
///
/// # Panics
///
/// Panics when a unit panics.
pub fn fan_out_each<'env, I, F>(units: I)
where
    I: IntoIterator<Item = F>,
    F: FnOnce() + Send + 'env,
{
    fan_out(units);
}
