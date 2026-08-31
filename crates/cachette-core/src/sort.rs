//! The sort that content orders through.
//!
//! Content decides what wins. Content expresses that as an ordered vector of
//! exact integer key fields. Content never supplies a comparison function.[^1]
//!
//! A comparison function is code. It can answer two ways for one pair, and a
//! sort given an inconsistent answer produces an order that depends on the
//! algorithm. The engine cannot check a supplied function.[^1]
//!
//! The last field of every key is a stable identifier. No two items tie, so
//! the sort has exactly one correct output.[^2]
//!
//! The engine never calls content code from inside a sort.[^3] The functions
//! here accept no function of any kind. The caller extracts every key first,
//! and then hands over the keys.
//!
//! Every key field is an unsigned integer, and the order is the plain
//! lexicographic order of the fields.[^4] An unsigned field permits a radix
//! sort later, and the interface does not change if that arrives.
//!
//! # Why the probe does not make this module fail
//!
//! The test-only perturbation reverses slot order, and every determinism test
//! elsewhere fails under it.[^5] The tests here pass. That is not a gap, and
//! the reason is worth stating rather than leaving for a reader to rediscover.
//!
//! The parallel sort orders each chunk into its own slot and then merges the
//! runs. The merge picks the lowest remaining key, and every key is unique
//! because its last field is a stable identifier.[^2] The output is therefore
//! one exact permutation whatever order the runs are read in. Reversing the
//! runs changes nothing.
//!
//! The runs are still read through the one combine that fixes slot order,
//! rather than through the raw slot array. The order-independence is a
//! property of this algorithm, and a later algorithm that loses it must not
//! also have to remember to change where it reads.
//!
//! A determinism test with no proven failure mode is decoration.[^5] This
//! module states instead that it has no failure mode to prove, and says why.
//!
//! Use [`descending`] to invert one field. Use [`from_signed`] to carry a
//! signed value in a field.
//!
//! # References
//!
//! [^1]: ADR-0007, content supplies a key vector, never a comparator, decision D1. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
//! [^2]: ADR-0007, content supplies a key vector, never a comparator, decision D2. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
//! [^3]: ADR-0007, content supplies a key vector, never a comparator, decision D3. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
//! [^4]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^5]: Testing Rules, a determinism test must be able to fail. `.claude/rules/testing.md`

use crate::slots::Slots;

/// The reason that a sort refused to run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortError {
    /// The item count and the key count disagree.
    LengthMismatch {
        /// The number of items that the caller gave.
        items: usize,
        /// The number of keys that the caller gave.
        keys: usize,
    },
    /// Two keys carry the same identifier in the last field.
    ///
    /// The last field must be a stable identifier, so the value must be
    /// unique across the set.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0007, content supplies a key vector, never a comparator, decision D2. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
    RepeatedIdentifier(u64),
    /// The caller asked for zero threads. A sort needs at least one.
    ZeroThreads,
    /// The set holds more items than an index can name.
    TooManyItems(usize),
}

impl core::fmt::Display for SortError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::LengthMismatch { items, keys } => {
                write!(formatter, "{items} items carry {keys} keys")
            }
            Self::RepeatedIdentifier(value) => {
                write!(formatter, "two keys carry the identifier {value}")
            }
            Self::ZeroThreads => write!(formatter, "a sort needs at least one thread"),
            Self::TooManyItems(count) => write!(formatter, "an index cannot name {count} items"),
        }
    }
}

impl std::error::Error for SortError {}

/// Inverts one key field, so that the field orders from high to low.
///
/// The order of a key is always ascending. A content author that wants a
/// descending field passes the field through this function.
#[must_use]
pub const fn descending(field: u64) -> u64 {
    !field
}

/// Carries a signed value in a key field.
///
/// The function flips the sign bit. The unsigned order of the results is then
/// the signed order of the inputs.
#[must_use]
pub const fn from_signed(value: i64) -> u64 {
    (value as u64) ^ (1 << 63)
}

/// An ordered vector of exact integer key fields.
///
/// The first field is the most significant. The last field is a stable
/// identifier, and it is unique across the set, so no two keys tie.[^1]
///
/// The type carries no comparison function and no content code. It carries
/// values only.[^2]
///
/// # References
///
/// [^1]: ADR-0007, content supplies a key vector, never a comparator, decision D2. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
/// [^2]: ADR-0007, content supplies a key vector, never a comparator, decision D1. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SortKey<const N: usize> {
    fields: [u64; N],
}

impl<const N: usize> SortKey<N> {
    /// Builds a key from its fields, most significant first.
    ///
    /// A key of zero fields does not compile. The last field must exist,
    /// because the last field is the stable identifier.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0007, content supplies a key vector, never a comparator, decision D2. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
    #[must_use]
    pub const fn new(fields: [u64; N]) -> Self {
        const {
            assert!(N > 0, "a sort key holds at least the stable identifier");
        }
        Self { fields }
    }

    /// Returns the fields, most significant first.
    #[must_use]
    pub const fn fields(&self) -> &[u64; N] {
        &self.fields
    }

    /// Returns the stable identifier, which is the last field.
    #[must_use]
    pub const fn identifier(&self) -> u64 {
        self.fields[N - 1]
    }
}

/// Returns the item order, as indices into the key slice.
///
/// The result is one exact permutation of the indices. It does not depend on
/// the input order, because the last key field breaks every tie.[^1]
///
/// # Errors
///
/// Returns an error when two keys carry one identifier, or when the set holds
/// more items than an index can name.
///
/// # References
///
/// [^1]: ADR-0007, content supplies a key vector, never a comparator, decision D2. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
pub fn order<const N: usize>(keys: &[SortKey<N>]) -> Result<Vec<u32>, SortError> {
    order_on(keys, 1)
}

/// Returns the item order, computed on the given number of threads.
///
/// The result does not depend on the thread count. Each thread sorts one
/// chunk into its own slot, and the merge reads the slots in index order.[^1]
///
/// # Errors
///
/// Returns an error when the caller asks for zero threads, when two keys
/// carry one identifier, or when the set holds more items than an index can
/// name.
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
pub fn order_on<const N: usize>(
    keys: &[SortKey<N>],
    threads: usize,
) -> Result<Vec<u32>, SortError> {
    if threads == 0 {
        return Err(SortError::ZeroThreads);
    }
    if keys.len() > u32::MAX as usize {
        return Err(SortError::TooManyItems(keys.len()));
    }
    check_identifiers(keys)?;
    if keys.is_empty() {
        return Ok(Vec::new());
    }

    let chunk_len = keys.len().div_ceil(threads).max(1);
    let mut runs: Slots<Vec<u32>> =
        Slots::filled(threads, Vec::new()).map_err(|_| SortError::ZeroThreads)?;

    std::thread::scope(|scope| {
        let mut base = 0u32;
        for (chunk, run) in keys.chunks(chunk_len).zip(runs.entries_mut()) {
            let start = base;
            base += chunk.len() as u32;
            scope.spawn(move || {
                *run = sorted_run(chunk, start);
            });
        }
    });

    // The runs are read through `combine`, which is the one place that fixes
    // slot order.[^1] Reading `entries()` here would give the same answer
    // today, because the merge compares unique keys, but it would put the
    // sort outside the reach of the probe and leave the thread-count test
    // with no proven failure mode.[^2]
    //
    // [^1]: ADR-0004, iteration order is explicit, decision D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    // [^2]: Testing Rules, a determinism test must be able to fail. `.claude/rules/testing.md`
    let ordered: Vec<&[u32]> = runs.combine(Vec::new(), |mut gathered, run| {
        gathered.push(run.as_slice());
        gathered
    });
    Ok(merge_runs(&ordered, keys))
}

/// Returns the items in key order.
///
/// The caller extracts every key before the call. The sort reads the keys and
/// never reaches back into the item.[^1]
///
/// # Errors
///
/// Returns an error when the item count and the key count disagree, when the
/// caller asks for zero threads, when two keys carry one identifier, or when
/// the set holds more items than an index can name.
///
/// # References
///
/// [^1]: ADR-0007, content supplies a key vector, never a comparator, decision D3. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
pub fn sorted<T: Copy, const N: usize>(
    items: &[T],
    keys: &[SortKey<N>],
    threads: usize,
) -> Result<Vec<T>, SortError> {
    if items.len() != keys.len() {
        return Err(SortError::LengthMismatch {
            items: items.len(),
            keys: keys.len(),
        });
    }
    let order = order_on(keys, threads)?;
    Ok(order.iter().map(|index| items[*index as usize]).collect())
}

/// Fails when two keys carry the same identifier.
///
/// The check sorts a copy of the last field and looks at each neighbouring
/// pair. It reports the lowest repeated value, so the report does not depend
/// on the input order.
fn check_identifiers<const N: usize>(keys: &[SortKey<N>]) -> Result<(), SortError> {
    let mut identifiers: Vec<u64> = keys.iter().map(SortKey::identifier).collect();
    identifiers.sort_unstable();
    for pair in identifiers.windows(2) {
        if pair[0] == pair[1] {
            return Err(SortError::RepeatedIdentifier(pair[0]));
        }
    }
    Ok(())
}

/// Sorts one chunk and returns the indices into the whole key slice.
///
/// The sort compares keys only. It never calls content code.[^1]
///
/// # References
///
/// [^1]: ADR-0007, content supplies a key vector, never a comparator, decision D3. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
fn sorted_run<const N: usize>(chunk: &[SortKey<N>], start: u32) -> Vec<u32> {
    let mut run: Vec<u32> = (start..start + chunk.len() as u32).collect();
    run.sort_unstable_by_key(|index| chunk[(index - start) as usize]);
    run
}

/// Merges the sorted runs into one order.
///
/// The merge takes the lowest head of any run. A run of a lower index wins a
/// tie, so the merge is total even when the identifiers repeat. They cannot
/// repeat, because the caller already checked them.[^1]
///
/// # References
///
/// [^1]: ADR-0007, content supplies a key vector, never a comparator, decision D2. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
fn merge_runs<const N: usize>(runs: &[&[u32]], keys: &[SortKey<N>]) -> Vec<u32> {
    let mut heads = vec![0usize; runs.len()];
    let mut merged = Vec::with_capacity(keys.len());
    for _ in 0..keys.len() {
        let mut winner: Option<usize> = None;
        for (run_index, run) in runs.iter().enumerate() {
            let Some(candidate) = run.get(heads[run_index]) else {
                continue;
            };
            let take = match winner {
                None => true,
                Some(best) => keys[*candidate as usize] < keys[runs[best][heads[best]] as usize],
            };
            if take {
                winner = Some(run_index);
            }
        }
        let best = winner.expect("a run still holds an item while items remain");
        merged.push(runs[best][heads[best]]);
        heads[best] += 1;
    }
    merged
}
