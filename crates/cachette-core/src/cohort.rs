//! Consumption: a per-unit need, and a pooled draw by cohort.
//!
//! A unit carries a need. The need falls at an interval, by a saturating
//! subtract, and it stops at zero. A wrapping subtract would turn a small
//! shortfall into a full satisfaction, which is the classic underflow
//! defect in this kernel.[^1]
//!
//! A unit never draws against a store on its own. A **cohort** is one row
//! that stands for the units of one kind that belong to one site. The row
//! holds a headcount and the site it belongs to. The draw is one segmented
//! reduction over the cohort rows, then one capped transfer out of the
//! pooled store of the site.[^2] Nothing here holds a lock, an atomic or a
//! retry, because each thread owns a contiguous span of sites and writes
//! only inside it.[^3]
//!
//! The cohort array is indexed by the slot of the site, so it is already
//! sorted by site and the reduction needs no sort.[^1] The spans of the
//! reduction are therefore contiguous, and a span never crosses a thread.
//!
//! **The need stays on the unit even though the draw is pooled.** A pure
//! cohort has a cliff: a place is fine a little above its demand and
//! starves entirely a little below it. The per-unit deficit accumulator
//! removes the cliff, because a shortage degrades before it kills.[^1]
//!
//! A need is not a conserved quantity. Nothing flows into it and nothing
//! flows out of it, so the clamp at the top of a need is safe. The
//! commodity that satisfies the need is conserved, and it is conserved at
//! the store: what leaves the store is what the cohorts received, and the
//! world keeps that account.[^1] [^4]
//!
//! Every value here is an integer or a Q16.16 fixed-point value.[^5] Every
//! operation goes through the arithmetic module.[^6]
//!
//! # References
//!
//! [^1]: Research report 15, needs, consumption and the input-output economy, sections 4.2, 5.3, 6.3 and 6.4. `docs/research/reports/15-needs-consumption-and-economy.md`
//! [^2]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D2. `docs/adrs/draft/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
//! [^3]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
//! [^4]: Findings register, FND-016. `docs/FINDINGS.md`
//! [^5]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^6]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`

use bytemuck::{Pod, Zeroable};

use crate::hash::StateHash;
use crate::sim_math;
use crate::site::{CommodityId, Store, StoreUpdate, COMMODITY_COUNT};
use crate::slots::Slots;
use crate::soldier::{NeedUpdate, NO_HOME};
use crate::types::{Accum, Entity, FactionId, Fix32, Tick, FACTION_CEILING};

/// The number of cohorts that one site holds.
///
/// A cohort is the units of one faction at one site. Two factions at one
/// place never pool their draw, because a pooled draw would feed a rival
/// out of a store it does not hold.
///
/// The number is the faction ceiling, which is a property of the faction
/// mask and not a budget.[^1]
///
/// # References
///
/// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D1. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
pub const COHORTS_PER_SITE: usize = FACTION_CEILING as usize;

/// The value of a need that is fully met.
///
/// A need runs from zero to one. The bound is a property of the scale, not
/// a budget.[^1]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
pub const NEED_FULL: Fix32 = Fix32::ONE;

/// Returns the row of one faction at one site.
///
/// The site is the high part of the key, so the array is sorted by site by
/// construction and the segmented reduction needs no sort.[^1]
///
/// # References
///
/// [^1]: Research report 15, needs, consumption and the input-output economy, section 5.3. `docs/research/reports/15-needs-consumption-and-economy.md`
#[must_use]
pub const fn row_index(site: u32, faction: u16) -> usize {
    (site as usize) * COHORTS_PER_SITE + (faction as usize)
}

/// The reason that this module refused a caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CohortError {
    /// The caller asked for zero threads.
    ZeroThreads,
    /// A rate is below zero. A need falls and rises at rates above zero.
    RateBelowZero(Fix32),
    /// The commodity is outside the commodity set.
    CommodityOutsideSet(CommodityId),
    /// The columns that the pass reads hold different numbers of rows.
    ///
    /// The cohort table states a site count that the settlement arena
    /// already holds. A check must fail when the two copies disagree.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    ColumnsDisagree,
}

impl core::fmt::Display for CohortError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ZeroThreads => write!(formatter, "a pass needs at least one thread"),
            Self::RateBelowZero(rate) => write!(formatter, "the rate {} is below zero", rate.0),
            Self::CommodityOutsideSet(commodity) => write!(
                formatter,
                "the commodity {} is outside the commodity set",
                commodity.0
            ),
            Self::ColumnsDisagree => write!(formatter, "the columns hold different lengths"),
        }
    }
}

impl std::error::Error for CohortError {}

/// What a unit needs, and how fast it needs it.
///
/// Every field is a rate at or above zero, in Q16.16, for one tick. The
/// schedule scales a rate to one application, so the interval is a
/// parameter of the schedule and never a constant of this kernel.[^1]
///
/// The values are content. They are declared here until content exists, and
/// the register holds the open choice of them.[^2]
///
/// # References
///
/// [^1]: ADR-0062, production and upkeep are rates attached to a site, decision D4. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
/// [^2]: Decisions register, DEC-034. `docs/DECISIONS.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NeedRule {
    /// What one tick takes off the need of one unit.
    decay: Fix32,
    /// What one unit asks of the store in one tick.
    ration: Fix32,
    /// The need below which a unit is in deficit.
    threshold: Fix32,
    /// What one tick takes off the deficit of a unit that is not in
    /// deficit.
    recovery: Fix32,
}

impl NeedRule {
    /// The rule that a world starts with.
    ///
    /// The ration equals the decay, so a unit that receives its whole
    /// ration holds its need level. That equality is the reason the two
    /// values are stated together rather than in two places.
    pub const DEFAULT: Self = Self {
        decay: Fix32(NEED_FULL.0 / 16),
        ration: Fix32(NEED_FULL.0 / 16),
        threshold: Fix32(NEED_FULL.0 / 2),
        recovery: Fix32(NEED_FULL.0 / 16),
    };

    /// Builds a rule.
    ///
    /// # Errors
    ///
    /// Returns an error when any rate is below zero.
    pub const fn new(
        decay: Fix32,
        ration: Fix32,
        threshold: Fix32,
        recovery: Fix32,
    ) -> Result<Self, CohortError> {
        if decay.0 < 0 {
            return Err(CohortError::RateBelowZero(decay));
        }
        if ration.0 < 0 {
            return Err(CohortError::RateBelowZero(ration));
        }
        if threshold.0 < 0 {
            return Err(CohortError::RateBelowZero(threshold));
        }
        if recovery.0 < 0 {
            return Err(CohortError::RateBelowZero(recovery));
        }
        Ok(Self {
            decay,
            ration,
            threshold,
            recovery,
        })
    }

    /// Returns what one tick takes off a need.
    #[must_use]
    pub const fn decay(self) -> Fix32 {
        self.decay
    }

    /// Returns what one unit asks of the store in one tick.
    #[must_use]
    pub const fn ration(self) -> Fix32 {
        self.ration
    }

    /// Returns the need below which a unit is in deficit.
    #[must_use]
    pub const fn threshold(self) -> Fix32 {
        self.threshold
    }

    /// Returns what one tick takes off a deficit that is recovering.
    #[must_use]
    pub const fn recovery(self) -> Fix32 {
        self.recovery
    }

    /// Absorbs the rule into the state hash.
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        hash.write(&self.decay.0.to_le_bytes())
            .write(&self.ration.0.to_le_bytes())
            .write(&self.threshold.0.to_le_bytes())
            .write(&self.recovery.0.to_le_bytes())
    }
}

/// One cohort: the units of one stratum that belong to one site.
///
/// The row holds a headcount and never a list of identities. An identity
/// lives in the arena that minted it, and a second list of identities would
/// be the same fact in two places.[^1]
///
/// The layout is 4 + 4 + 2 + 2 bytes, which is 12 bytes at an alignment of
/// 4. The trailing array declares every padding byte.[^2]
///
/// # References
///
/// [^1]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
/// [^2]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct CohortRow {
    /// The slot of the site that the cohort belongs to.
    pub site: u32,
    /// The number of units that the row stands for.
    pub headcount: u32,
    /// The faction of the units that the row stands for.
    pub faction: u16,
    /// The declared padding. Always zero.
    pub padding: [u8; 2],
}

/// The cohorts of every site, indexed by the slot of the site.
///
/// The table is derived. It states nothing that the unit columns do not
/// already state, and the check derives it again and compares.[^1]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CohortTable {
    rows: Vec<CohortRow>,
    /// The number of live units that belong to no site.
    ///
    /// A unit that belongs to no site draws from nothing. The count is here
    /// so that the headcount of the whole table plus this number is the
    /// live population, which is the equality a test asserts.
    unattached: u32,
}

impl CohortTable {
    /// Builds an empty table.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            rows: Vec::new(),
            unattached: 0,
        }
    }

    /// Returns the rows, in site order.
    #[must_use]
    pub fn rows(&self) -> &[CohortRow] {
        &self.rows
    }

    /// Returns the number of sites that the table covers.
    #[must_use]
    pub fn site_count(&self) -> u32 {
        (self.rows.len() / COHORTS_PER_SITE) as u32
    }

    /// Returns the number of live units that belong to no site.
    #[must_use]
    pub const fn unattached(&self) -> u32 {
        self.unattached
    }

    /// Returns the headcount of one cohort.
    #[must_use]
    pub fn headcount(&self, site: u32, faction: FactionId) -> Option<u32> {
        self.rows
            .get(row_index(site, faction.0))
            .map(|row| row.headcount)
    }

    /// Returns the number of units that every cohort stands for.
    ///
    /// The sum is exact. A headcount is a whole number and the accumulator
    /// is 64 bits wide, so the sum combines in any order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D3. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    #[must_use]
    pub fn headcount_total(&self) -> Accum {
        let mut total = Accum(0);
        for row in &self.rows {
            total = sim_math::combine(total, Accum(i64::from(row.headcount)));
        }
        total
    }

    /// Derives the table again from the unit columns.
    ///
    /// The home column of the units is the truth. This table is a summary
    /// of it, so nothing here decides anything: a unit that names a site is
    /// counted at that site, and a unit that names no site is counted as
    /// unattached.
    ///
    /// The rebuild visits the units in slot order, which is the same order
    /// on every run and at every thread count.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub fn rebuild(&mut self, homes: &[u32], factions: &[FactionId], live: &[u8], sites: u32) {
        self.rows.clear();
        self.rows
            .resize(sites as usize * COHORTS_PER_SITE, CohortRow::default());
        for (site, chunk) in self.rows.chunks_mut(COHORTS_PER_SITE).enumerate() {
            for (stratum, row) in chunk.iter_mut().enumerate() {
                row.site = site as u32;
                row.faction = stratum as u16;
            }
        }
        self.unattached = 0;
        for (slot, home) in homes.iter().enumerate() {
            if live.get(slot).copied() != Some(1) {
                continue;
            }
            if *home == NO_HOME {
                self.unattached += 1;
                continue;
            }
            match self.rows.get_mut(row_index(*home, factions[slot].0)) {
                Some(row) => row.headcount += 1,
                // A home that names no live site, or a faction above the
                // ceiling, counts as unattached here. The world refuses
                // both, and its invariant check states so.
                None => self.unattached += 1,
            }
        }
    }

    /// Absorbs the table into the state hash.
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        hash.write_u64(self.rows.len() as u64)
            .write_u64(u64::from(self.unattached))
            .write(bytemuck::cast_slice(&self.rows))
    }

    /// Reports whether the table describes the unit columns.
    ///
    /// The check derives the table again and compares. A summary that no
    /// check compares against its source is a second declaration site with
    /// nothing that fails on disagreement.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub fn describes(
        &self,
        homes: &[u32],
        factions: &[FactionId],
        live: &[u8],
        sites: u32,
    ) -> bool {
        let mut derived = Self::new();
        derived.rebuild(homes, factions, live, sites);
        derived == *self
    }

    /// Reports whether the table holds its own invariants.
    #[must_use]
    pub fn check_invariants(&self) -> bool {
        // Each row states its own key, and the key must be the position the
        // row sits at. That equality covers the length of the table as
        // well: a table that holds a part of a site fails on the last row.
        self.rows.iter().enumerate().all(|(index, row)| {
            row.padding == [0; 2]
                && row_index(row.site, row.faction) == index
                && (row.faction as usize) < COHORTS_PER_SITE
        })
    }
}

/// A site could not serve every cohort that drew on it.
///
/// The event names what the cohorts asked for and what the store gave. The
/// store stopped at zero rather than going below it.
///
/// The layout is 8 + 8 + 8 + 8 + 2 + 6 bytes, which is 40 bytes at an
/// alignment of 8. The trailing array declares every padding byte.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct SiteRationed {
    /// The tick at which the draw ran.
    pub tick: Tick,
    /// The site that could not serve, as its identity in bits.
    pub site: u64,
    /// What the cohorts of the site asked for.
    pub demanded: Accum,
    /// What the store gave. It is always below what was asked.
    pub granted: Accum,
    /// The commodity that the cohorts drew.
    pub commodity: u16,
    /// The declared padding. Always zero.
    pub padding: [u8; 6],
}

impl SiteRationed {
    /// Builds an event with zero padding.
    #[must_use]
    pub const fn new(
        tick: Tick,
        site: u64,
        demanded: Accum,
        granted: Accum,
        commodity: u16,
    ) -> Self {
        Self {
            tick,
            site,
            demanded,
            granted,
            commodity,
            padding: [0; 6],
        }
    }
}

/// The running account of every draw that has run.
///
/// Each field is a 64-bit accumulator, so a sum over many sites and many
/// ticks is exact and combines in any order.[^1]
///
/// # References
///
/// [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D3. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct DrawLedger {
    /// What the cohorts asked for, of each commodity.
    pub demanded: [Accum; COMMODITY_COUNT],
    /// What the stores gave, of each commodity.
    pub granted: [Accum; COMMODITY_COUNT],
    /// What the stores could not give, of each commodity.
    pub unmet: [Accum; COMMODITY_COUNT],
    /// The number of cohorts that the draw visited.
    pub visited: [Accum; 1],
}

impl DrawLedger {
    /// The ledger that records nothing.
    pub const ZERO: Self = Self {
        demanded: [Accum(0); COMMODITY_COUNT],
        granted: [Accum(0); COMMODITY_COUNT],
        unmet: [Accum(0); COMMODITY_COUNT],
        visited: [Accum(0); 1],
    };

    /// Adds one ledger into another.
    ///
    /// Every term is an integer addition, so the operation is exactly
    /// associative and exactly commutative. A parallel reduction over it
    /// gives one answer at any thread count.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D2. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn combine(self, other: Self) -> Self {
        let mut joined = self;
        for index in 0..COMMODITY_COUNT {
            joined.demanded[index] =
                sim_math::combine(joined.demanded[index], other.demanded[index]);
            joined.granted[index] = sim_math::combine(joined.granted[index], other.granted[index]);
            joined.unmet[index] = sim_math::combine(joined.unmet[index], other.unmet[index]);
        }
        joined.visited[0] = sim_math::combine(joined.visited[0], other.visited[0]);
        joined
    }

    /// Absorbs the ledger into the state hash.
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        hash.write(bytemuck::bytes_of(self))
    }
}

/// What one draw did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DrawPass {
    /// The totals of the draw.
    pub ledger: DrawLedger,
    /// What each cohort received, in row order.
    pub shares: Vec<Accum>,
    /// One event for each site that could not serve, in slot order.
    pub rationed: Vec<SiteRationed>,
}

impl DrawPass {
    /// The draw that did nothing.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            ledger: DrawLedger::ZERO,
            shares: Vec::new(),
            rationed: Vec::new(),
        }
    }
}

/// Draws one commodity out of every site store, by cohort.
///
/// The draw is one segmented reduction over the cohort rows of a site,
/// followed by one capped transfer out of the store of that site. It never
/// loops over units, and it holds no lock, no atomic and no retry.
///
/// A store that cannot serve every cohort splits what it has in proportion
/// to what each cohort asked for. The split is exact: each share truncates
/// the proportion, and the remainder goes one unit at a time to the cohorts
/// in ascending row order. The parts therefore sum to the whole, with no
/// unit lost and none created.[^1]
///
/// # Errors
///
/// Returns an error when the caller asks for zero threads, when the ration
/// is below zero, when the commodity is outside the set, and when the
/// columns disagree on how many sites there are.
///
/// # References
///
/// [^1]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D3. `docs/adrs/draft/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
pub fn draw(
    tick: Tick,
    ration: Fix32,
    commodity: CommodityId,
    table: &CohortTable,
    update: StoreUpdate<'_>,
    threads: usize,
) -> Result<DrawPass, CohortError> {
    if threads == 0 {
        return Err(CohortError::ZeroThreads);
    }
    if ration.0 < 0 {
        return Err(CohortError::RateBelowZero(ration));
    }
    if (commodity.0 as usize) >= COMMODITY_COUNT {
        return Err(CohortError::CommodityOutsideSet(commodity));
    }
    let StoreUpdate {
        stores,
        live,
        generations,
    } = update;
    let count = stores.len();
    if live.len() != count || generations.len() != count || table.site_count() as usize != count {
        return Err(CohortError::ColumnsDisagree);
    }
    if count == 0 {
        return Ok(DrawPass::empty());
    }

    let rows = table.rows();
    let chunk_len = count.div_ceil(threads).max(1);
    let mut shares = vec![Accum(0); rows.len()];
    let mut slots: Slots<(DrawLedger, Vec<SiteRationed>)> =
        Slots::filled(threads, (DrawLedger::ZERO, Vec::new()))
            .map_err(|_| CohortError::ZeroThreads)?;

    std::thread::scope(|scope| {
        let mut base = 0usize;
        for ((span, share_span), slot) in stores
            .chunks_mut(chunk_len)
            .zip(shares.chunks_mut(chunk_len * COHORTS_PER_SITE))
            .zip(slots.entries_mut())
        {
            let start = base;
            base += span.len();
            let live_span = &live[start..base];
            let generation_span = &generations[start..base];
            let row_span = &rows[start * COHORTS_PER_SITE..base * COHORTS_PER_SITE];
            scope.spawn(move || {
                *slot = draw_span(
                    tick,
                    ration,
                    commodity,
                    start as u32,
                    span,
                    share_span,
                    live_span,
                    generation_span,
                    row_span,
                );
            });
        }
    });

    // The ledger combine is order-free, because every term is an integer
    // addition. The rationed log is not order-free: a concatenation depends
    // on the order it reads the slots in, so it takes the fixed slot
    // order.[^1]
    //
    // [^1]: ADR-0004, iteration order is explicit, decisions D2 and D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    let ledger = slots.combine(DrawLedger::ZERO, |carried, (ledger, _)| {
        carried.combine(*ledger)
    });
    let rationed = slots.combine(Vec::new(), |mut carried: Vec<SiteRationed>, (_, log)| {
        carried.extend_from_slice(log);
        carried
    });
    Ok(DrawPass {
        ledger,
        shares,
        rationed,
    })
}

/// Draws over one span of sites.
///
/// The span is contiguous and it belongs to one thread. The function reads
/// and writes nothing outside the span.
#[allow(clippy::too_many_arguments)]
fn draw_span(
    tick: Tick,
    ration: Fix32,
    commodity: CommodityId,
    start: u32,
    stores: &mut [Store],
    shares: &mut [Accum],
    live: &[u8],
    generations: &[u32],
    rows: &[CohortRow],
) -> (DrawLedger, Vec<SiteRationed>) {
    let mut ledger = DrawLedger::ZERO;
    let mut log = Vec::new();
    let index = commodity.0 as usize;
    for offset in 0..stores.len() {
        if live[offset] != 1 {
            continue;
        }
        let segment = &rows[offset * COHORTS_PER_SITE..(offset + 1) * COHORTS_PER_SITE];
        let shares = &mut shares[offset * COHORTS_PER_SITE..(offset + 1) * COHORTS_PER_SITE];

        // The segmented reduction. The segment is contiguous and it never
        // crosses a thread, because the cohort array is indexed by the site
        // slot and a span is cut on a site boundary.
        let mut demanded = Accum(0);
        for row in segment {
            ledger.visited[0] = sim_math::combine(ledger.visited[0], Accum(1));
            demanded = sim_math::combine(demanded, sim_math::scale_by_count(ration, row.headcount));
        }
        if demanded.0 == 0 {
            continue;
        }

        // One capped transfer. The store stops at zero, and what it could
        // not give is unmet demand rather than a debt.
        let held = stores[offset]
            .quantity(commodity)
            .expect("the commodity is inside the set");
        let granted = if held.to_accum().0 < demanded.0 {
            held.to_accum()
        } else {
            demanded
        };
        let taken = Fix32(granted.0 as i32);
        stores[offset].set_quantity(commodity, sim_math::sub(held, taken));

        // The exact split. Each cohort takes the truncated proportion of
        // what it asked for, and the remainder goes one unit at a time in
        // ascending row order.
        let mut handed = Accum(0);
        for (row, share) in segment.iter().zip(shares.iter_mut()) {
            let asked = sim_math::scale_by_count(ration, row.headcount);
            *share = sim_math::share(granted, asked, demanded)
                .expect("the demand of the segment is above zero");
            handed = sim_math::combine(handed, *share);
        }
        let mut remainder = sim_math::combine(granted, Accum(-handed.0));
        for (row, share) in segment.iter().zip(shares.iter_mut()) {
            if remainder.0 <= 0 {
                break;
            }
            if row.headcount == 0 {
                continue;
            }
            *share = sim_math::combine(*share, Accum(1));
            remainder = sim_math::combine(remainder, Accum(-1));
        }

        ledger.demanded[index] = sim_math::combine(ledger.demanded[index], demanded);
        ledger.granted[index] = sim_math::combine(ledger.granted[index], granted);
        if granted.0 < demanded.0 {
            let unmet = sim_math::combine(demanded, Accum(-granted.0));
            ledger.unmet[index] = sim_math::combine(ledger.unmet[index], unmet);
            let site = Entity::new(start + offset as u32, generations[offset])
                .expect("a live slot holds a generation of one or more");
            log.push(SiteRationed::new(
                tick,
                site.to_bits(),
                demanded,
                granted,
                commodity.0,
            ));
        }
    }
    (ledger, log)
}

/// Takes the decay off the need of every live unit.
///
/// The subtract saturates at zero. It never wraps, because a wrapping
/// subtract turns a small shortfall into a full satisfaction and nothing
/// fails when it does.[^1]
///
/// The pass is a map. Each thread owns a contiguous span of unit slots and
/// writes only inside it, so the result does not depend on the thread
/// count.[^2]
///
/// # Errors
///
/// Returns an error when the caller asks for zero threads, when the decay
/// is below zero, and when the columns hold different lengths.
///
/// # References
///
/// [^1]: Research report 15, needs, consumption and the input-output economy, section 6.3. `docs/research/reports/15-needs-consumption-and-economy.md`
/// [^2]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
pub fn decay(rate: Fix32, update: NeedUpdate<'_>, threads: usize) -> Result<(), CohortError> {
    if threads == 0 {
        return Err(CohortError::ZeroThreads);
    }
    if rate.0 < 0 {
        return Err(CohortError::RateBelowZero(rate));
    }
    let NeedUpdate {
        needs,
        deficits,
        live,
        homes,
        factions,
    } = update;
    let count = needs.len();
    if deficits.len() != count
        || live.len() != count
        || homes.len() != count
        || factions.len() != count
    {
        return Err(CohortError::ColumnsDisagree);
    }
    if count == 0 {
        return Ok(());
    }
    let chunk_len = count.div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        let mut base = 0usize;
        for span in needs.chunks_mut(chunk_len) {
            let start = base;
            base += span.len();
            let live_span = &live[start..base];
            scope.spawn(move || {
                for (need, live) in span.iter_mut().zip(live_span) {
                    if *live != 1 {
                        continue;
                    }
                    // The saturating subtract, with the floor at zero. This
                    // is the line that a wrapping subtract would break.
                    let fallen = sim_math::sub(*need, rate);
                    *need = if fallen.0 < 0 { Fix32::ZERO } else { fallen };
                }
            });
        }
    });
    Ok(())
}

/// Feeds every unit from what its cohort received, and moves the deficit.
///
/// The share of a cohort spreads over its headcount, so a unit gains the
/// same amount as every other unit of its cohort. The need clamps at the
/// top, and the clamp is safe because a need is not a conserved
/// quantity.[^1]
///
/// A unit whose need is below the threshold adds the shortfall to its
/// deficit accumulator, and a unit at or above the threshold takes the
/// recovery off it. Both moves saturate. The accumulator is the input that
/// a later rule reads to end a unit, and this pass stops at the
/// accumulator.
///
/// # Errors
///
/// Returns an error when the caller asks for zero threads, and when the
/// columns hold different lengths.
///
/// # References
///
/// [^1]: Research report 15, needs, consumption and the input-output economy, sections 6.3 and 6.4. `docs/research/reports/15-needs-consumption-and-economy.md`
pub fn satisfy(
    rule: NeedRule,
    shares: &[Accum],
    table: &CohortTable,
    update: NeedUpdate<'_>,
    threads: usize,
) -> Result<(), CohortError> {
    if threads == 0 {
        return Err(CohortError::ZeroThreads);
    }
    if shares.len() != table.rows().len() {
        return Err(CohortError::ColumnsDisagree);
    }
    let NeedUpdate {
        needs,
        deficits,
        live,
        homes,
        factions,
    } = update;
    let count = needs.len();
    if deficits.len() != count
        || live.len() != count
        || homes.len() != count
        || factions.len() != count
    {
        return Err(CohortError::ColumnsDisagree);
    }
    if count == 0 {
        return Ok(());
    }

    // What one unit of each cohort gains. The division runs once for each
    // cohort, never once for each unit.
    let gains: Vec<Fix32> = table
        .rows()
        .iter()
        .zip(shares)
        .map(|(row, share)| sim_math::divide_by_count(*share, row.headcount).unwrap_or(Fix32::ZERO))
        .collect();
    let gains = &gains[..];
    let threshold = rule.threshold();
    let recovery = rule.recovery();

    let chunk_len = count.div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        let mut base = 0usize;
        for (need_span, deficit_span) in needs
            .chunks_mut(chunk_len)
            .zip(deficits.chunks_mut(chunk_len))
        {
            let start = base;
            base += need_span.len();
            let live_span = &live[start..base];
            let home_span = &homes[start..base];
            let faction_span = &factions[start..base];
            scope.spawn(move || {
                for offset in 0..need_span.len() {
                    if live_span[offset] != 1 {
                        continue;
                    }
                    let home = home_span[offset];
                    if home != NO_HOME {
                        if let Some(gain) = gains.get(row_index(home, faction_span[offset].0)) {
                            let fed = sim_math::add(need_span[offset], *gain);
                            need_span[offset] = if fed > NEED_FULL { NEED_FULL } else { fed };
                        }
                    }
                    let short = sim_math::sub(threshold, need_span[offset]);
                    deficit_span[offset] = if short.0 > 0 {
                        sim_math::add(deficit_span[offset], short)
                    } else {
                        let eased = sim_math::sub(deficit_span[offset], recovery);
                        if eased.0 < 0 {
                            Fix32::ZERO
                        } else {
                            eased
                        }
                    };
                }
            });
        }
    });
    Ok(())
}
