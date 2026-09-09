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

/// Runs one closure for each unit of parallel work, and returns the results in
/// the order the caller gave the units.
///
/// The calling thread runs the last unit. Each other unit gets one thread.
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
                Err(payload) => std::panic::resume_unwind(payload),
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
