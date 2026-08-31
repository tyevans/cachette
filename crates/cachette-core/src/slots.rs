//! The slot reduction, for a reduction that is not order-free.
//!
//! Integer addition and bitwise OR combine in any order.[^1] A minimum, a
//! maximum and a first-wins do not. Each of those depends on the order when
//! two values tie.
//!
//! Every reduction that is not order-free writes into a slot. A slot is
//! indexed by a stable key, never by a thread identifier. The combine step
//! then reads the slots in index order.[^2]
//!
//! A slot array costs one entry for each unit of parallel work. That memory
//! is the price of the order.[^2]
//!
//! Every rank here is an exact integer, so the comparison is exact.[^3]
//!
//! # References
//!
//! [^1]: ADR-0004, iteration order is explicit, decision D2. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
//! [^2]: ADR-0004, iteration order is explicit, decision D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
//! [^3]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`

/// The reason that a slot array refused to build.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotError {
    /// The caller asked for zero slots. A reduction needs at least one.
    ZeroSlots,
}

impl core::fmt::Display for SlotError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ZeroSlots => write!(formatter, "a slot reduction needs at least one slot"),
        }
    }
}

impl std::error::Error for SlotError {}

/// One candidate for a minimum or a maximum.
///
/// The rank decides the order. The payload rides along. The rank is an exact
/// integer, so two ranks never compare in two ways.[^1]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Candidate<P> {
    /// The value that the reduction compares.
    pub rank: i64,
    /// The value that the reduction carries.
    pub payload: P,
}

impl<P> Candidate<P> {
    /// Builds a candidate from a rank and a payload.
    pub const fn new(rank: i64, payload: P) -> Self {
        Self { rank, payload }
    }
}

/// An indexed output array for a reduction that is not order-free.
///
/// The caller gives one slot to each unit of parallel work. Each unit writes
/// only its own slot, so the units never contend. The combine step reads the
/// slots in index order, so the answer does not depend on which unit finished
/// first.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slots<T> {
    entries: Vec<T>,
}

impl<T: Clone> Slots<T> {
    /// Builds a slot array of the given length, with every slot at one value.
    ///
    /// # Errors
    ///
    /// Returns an error when the caller asks for zero slots.
    pub fn filled(count: usize, value: T) -> Result<Self, SlotError> {
        if count == 0 {
            return Err(SlotError::ZeroSlots);
        }
        Ok(Self {
            entries: vec![value; count],
        })
    }
}

impl<T> Slots<T> {
    /// Returns the number of slots.
    #[must_use]
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Returns the slots in index order.
    #[must_use]
    pub fn entries(&self) -> &[T] {
        &self.entries
    }

    /// Returns the slots for writing, in index order.
    ///
    /// A unit of parallel work takes one entry of this slice. The slots are
    /// disjoint, so the units write in parallel and share nothing.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub fn entries_mut(&mut self) -> &mut [T] {
        &mut self.entries
    }

    /// Folds the slots in index order.
    ///
    /// This is the combine step. It is the one place that fixes the order of
    /// a reduction that is not order-free.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub fn combine<A>(&self, initial: A, mut fold: impl FnMut(A, &T) -> A) -> A {
        let mut carried = initial;
        for entry in self.in_combine_order() {
            carried = fold(carried, entry);
        }
        carried
    }

    /// Returns the slots in the order that the combine step reads them.
    ///
    /// The order is index order. Index order does not depend on the thread
    /// count.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[cfg(not(feature = "probe-nondeterminism"))]
    fn in_combine_order(&self) -> impl Iterator<Item = &T> {
        self.entries.iter()
    }

    /// Returns the slots in reverse index order.
    ///
    /// This is the test-only switch. It breaks the order rule on purpose, so
    /// that the determinism tests have a proven failure mode. Never build a
    /// shipped artefact with this feature.
    #[cfg(feature = "probe-nondeterminism")]
    fn in_combine_order(&self) -> impl Iterator<Item = &T> {
        self.entries.iter().rev()
    }
}

impl<T: Copy> Slots<Option<T>> {
    /// Returns the value of the first slot that holds one.
    ///
    /// This is the first-wins reduction. First means lowest slot index. It
    /// does not mean the unit that finished first.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn first_wins(&self) -> Option<T> {
        self.combine(None, |carried, entry| carried.or(*entry))
    }
}

impl<P: Copy> Slots<Option<Candidate<P>>> {
    /// Returns the candidate of lowest rank.
    ///
    /// Two candidates of equal rank tie. The candidate in the lower slot wins
    /// the tie, because the combine step reads the slots in index order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn minimum(&self) -> Option<Candidate<P>> {
        self.best(|candidate, best| candidate.rank < best.rank)
    }

    /// Returns the candidate of highest rank.
    ///
    /// Two candidates of equal rank tie. The candidate in the lower slot wins
    /// the tie.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn maximum(&self) -> Option<Candidate<P>> {
        self.best(|candidate, best| candidate.rank > best.rank)
    }

    /// Returns the winning candidate under a strict rank test.
    ///
    /// The test is strict, so an equal rank never displaces the candidate
    /// that the combine step read first.
    fn best(&self, wins: impl Fn(&Candidate<P>, &Candidate<P>) -> bool) -> Option<Candidate<P>> {
        self.combine(None, |carried: Option<Candidate<P>>, entry| {
            match (carried, entry) {
                (None, entry) => *entry,
                (Some(best), Some(candidate)) if wins(candidate, &best) => Some(*candidate),
                (best, _) => best,
            }
        })
    }
}
