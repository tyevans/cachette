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
//! lexicographic order of the fields.[^4]
//!
//! # Two orders, one answer
//!
//! This module holds two orders. [`order_on`] takes a key vector of any
//! width and compares the keys on many threads. [`order_bounded`] takes two
//! fields and a ceiling, and it runs a radix sort on one thread.[^6]
//!
//! An unsigned integer key is what permits the radix sort. A comparison
//! function would forbid it.[^7]
//!
//! The two give the same permutation for the same keys. A property test
//! holds them together. A caller that can state a ceiling for its ordering
//! field takes the bounded order. A caller that cannot takes the general
//! one.
//!
//! **No engine code calls [`order_on`] today, and that is not the inert
//! capability shape.**[^8] The general order is the independent oracle that
//! the bounded order is tested against. Two algorithms that agree are
//! evidence. One algorithm compared against itself is not. An oracle whose
//! only caller is a test is doing the job it exists for, so do not delete
//! this one because nothing in the engine reaches it.
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
//! A determinism test with no proven failure mode is decoration.[^5] The
//! parallel sort has no failure mode to prove under the probe, and the
//! paragraphs above say why.
//!
//! The bounded order does have one. A radix pass is stable, so it keeps the
//! order that the caller gave for keys that tie. A second step replaces that
//! order with the order of the identifiers. Delete the second step, and the
//! properties that rotate the input fail. That failure was demonstrated when
//! the order was written.
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
//! [^6]: ADR-0071, the bridge rebuild orders on one thread, decisions D1 and D2. `docs/adrs/accepted/adr-0071-the-bridge-rebuild-orders-on-one-thread.md`
//! [^7]: ADR-0007, content supplies a key vector, never a comparator, the consequences. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
//! [^8]: Recurring defect shapes, section 3. `.claude/rules/recurring-defects.md`

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
    /// An ordering field lies above the ceiling that the caller stated.
    KeyAboveCeiling {
        /// The ordering field that the caller gave.
        key: u64,
        /// The ceiling that the caller stated.
        ceiling: u64,
    },
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
            Self::KeyAboveCeiling { key, ceiling } => {
                write!(formatter, "the key {key} lies above the ceiling {ceiling}")
            }
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

/// The number of bits in one radix digit.
///
/// Eight bits give a histogram of 256 counters, which is one kilobyte. A
/// histogram of that size stays in the first level cache, so one pass reads
/// the keys once and writes the indices once.
const DIGIT_BITS: u32 = 8;

/// The number of counters in one radix histogram.
const DIGIT_VALUES: usize = 1 << DIGIT_BITS;

/// A key of two fields, whose first field lies below a stated ceiling.
///
/// The first field orders the set. The second field is the stable
/// identifier, and it breaks every tie.[^1]
///
/// The type carries no comparison function and no content code. It carries
/// values only.[^2] The ceiling is a value too. The caller states it, and the
/// sort reads it. Nothing derives it from the data, so the cost of a sort
/// does not change when the data changes.
///
/// The derived order is the lexicographic order of the two fields, which is
/// the order of the equivalent general key.[^3]
///
/// # References
///
/// [^1]: ADR-0007, content supplies a key vector, never a comparator, decision D2. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
/// [^2]: ADR-0007, content supplies a key vector, never a comparator, decision D1. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
/// [^3]: ADR-0007, content supplies a key vector, never a comparator, decision D3. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoundedKey {
    order: u64,
    identifier: u64,
}

impl BoundedKey {
    /// Builds a key from the ordering field and the stable identifier.
    #[must_use]
    pub const fn new(order: u64, identifier: u64) -> Self {
        Self { order, identifier }
    }

    /// Returns the ordering field.
    #[must_use]
    pub const fn order(self) -> u64 {
        self.order
    }

    /// Returns the stable identifier.
    #[must_use]
    pub const fn identifier(self) -> u64 {
        self.identifier
    }
}

/// Returns the item order for a key whose first field has a stated ceiling.
///
/// The result equals the result of the general sort on the same keys, and it
/// is one exact permutation of the indices.[^1] The two functions differ in
/// cost, never in answer.
///
/// The sort is a radix sort on the ordering field, which an integer key
/// permits and a comparison function forbids.[^2] It runs on one thread, so
/// no result here takes its order from a thread that finished first.[^3]
///
/// The radix pass is stable, so items that share an ordering field keep the
/// order that the caller gave. That order is the caller's, not the sort's, so
/// a second step orders each such run by the stable identifier. The output
/// then depends on the key values alone.
///
/// # Errors
///
/// Returns an error when an ordering field lies above the ceiling, when two
/// keys carry one identifier, or when the set holds more items than an index
/// can name.
///
/// # References
///
/// [^1]: ADR-0007, content supplies a key vector, never a comparator, decision D2. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
/// [^2]: ADR-0007, content supplies a key vector, never a comparator, the consequences. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
/// [^3]: ADR-0071, the bridge rebuild orders on one thread, decision D2. `docs/adrs/accepted/adr-0071-the-bridge-rebuild-orders-on-one-thread.md`
pub fn order_bounded(keys: &[BoundedKey], ceiling: u64) -> Result<Vec<u32>, SortError> {
    if keys.len() > u32::MAX as usize {
        return Err(SortError::TooManyItems(keys.len()));
    }
    for key in keys {
        if key.order > ceiling {
            return Err(SortError::KeyAboveCeiling {
                key: key.order,
                ceiling,
            });
        }
    }
    check_bounded_identifiers(keys)?;
    if keys.is_empty() {
        return Ok(Vec::new());
    }

    let mut order = radix_order(keys, digit_count(ceiling));
    order_ties(&mut order, keys);
    Ok(order)
}

/// Returns the number of radix digits that a ceiling needs.
///
/// A ceiling of zero still needs one digit, because the set holds keys.
const fn digit_count(ceiling: u64) -> u32 {
    let bits = u64::BITS - ceiling.leading_zeros();
    if bits == 0 {
        1
    } else {
        bits.div_ceil(DIGIT_BITS)
    }
}

/// Fails when two keys carry the same identifier.
///
/// The check reports the lowest repeated value, so the report does not depend
/// on the input order.
fn check_bounded_identifiers(keys: &[BoundedKey]) -> Result<(), SortError> {
    let mut identifiers: Vec<u64> = keys.iter().map(|key| key.identifier).collect();
    identifiers.sort_unstable();
    for pair in identifiers.windows(2) {
        if pair[0] == pair[1] {
            return Err(SortError::RepeatedIdentifier(pair[0]));
        }
    }
    Ok(())
}

/// Orders the indices by the ordering field, least significant digit first.
///
/// Each pass counts the digits, turns the counts into starts, and then writes
/// each index to the start of its digit. A pass is stable, so a later pass
/// keeps the order that an earlier pass gave. After the last pass the indices
/// are in ascending order of the whole ordering field.
fn radix_order(keys: &[BoundedKey], digits: u32) -> Vec<u32> {
    let count = keys.len();
    let mut read: Vec<u32> = (0..count as u32).collect();
    let mut write: Vec<u32> = vec![0; count];
    for digit in 0..digits {
        let shift = digit * DIGIT_BITS;
        let mut starts = [0u32; DIGIT_VALUES];
        for index in &read {
            starts[digit_of(keys[*index as usize].order, shift)] += 1;
        }
        let mut total = 0u32;
        for start in &mut starts {
            let first = total;
            total += *start;
            *start = first;
        }
        for index in &read {
            let value = digit_of(keys[*index as usize].order, shift);
            write[starts[value] as usize] = *index;
            starts[value] += 1;
        }
        core::mem::swap(&mut read, &mut write);
    }
    read
}

/// Returns one digit of an ordering field.
const fn digit_of(order: u64, shift: u32) -> usize {
    ((order >> shift) & (DIGIT_VALUES as u64 - 1)) as usize
}

/// Orders each run of equal ordering fields by the stable identifier.
///
/// The radix passes leave such a run in the order that the caller gave. The
/// caller's order is not part of the key, so it must not reach the output.
/// This step replaces it with the order of the identifiers, which are unique
/// across the set.[^1]
///
/// # References
///
/// [^1]: ADR-0007, content supplies a key vector, never a comparator, decision D2. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
fn order_ties(order: &mut [u32], keys: &[BoundedKey]) {
    let mut start = 0usize;
    while start < order.len() {
        let field = keys[order[start] as usize].order;
        let mut end = start + 1;
        while end < order.len() && keys[order[end] as usize].order == field {
            end += 1;
        }
        if end - start > 1 {
            order[start..end].sort_unstable_by_key(|index| keys[*index as usize].identifier);
        }
        start = end;
    }
}
