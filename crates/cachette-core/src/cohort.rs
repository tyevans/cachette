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
//! [^2]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D2. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
//! [^3]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
//! [^4]: Findings register, FND-016. `docs/FINDINGS.md`
//! [^5]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^6]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`

use bytemuck::{Pod, Zeroable};

use crate::hash::StateHash;
use crate::rng;
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
    /// The deficit at which a unit ends.
    bound: Fix32,
}

impl NeedRule {
    /// The rule that a world starts with.
    ///
    /// The ration equals the decay, so a unit that receives its whole
    /// ration holds its need level. That equality is the reason the two
    /// values are stated together rather than in two places.
    ///
    /// The bound is the deficit at which a unit ends. It is content, it is
    /// declared here until content exists, and the register holds the open
    /// choice of it.[^1] A caller changes it without touching a kernel.
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-043. `docs/DECISIONS.md`
    pub const DEFAULT: Self = Self {
        decay: Fix32(NEED_FULL.0 / 16),
        ration: Fix32(NEED_FULL.0 / 16),
        threshold: Fix32(NEED_FULL.0 / 2),
        recovery: Fix32(NEED_FULL.0 / 16),
        bound: Fix32(NEED_FULL.0 * 4),
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
        bound: Fix32,
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
        if bound.0 < 0 {
            return Err(CohortError::RateBelowZero(bound));
        }
        Ok(Self {
            decay,
            ration,
            threshold,
            recovery,
            bound,
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

    /// Returns the deficit at which a unit ends.
    ///
    /// The bound is a parameter of the rule. A caller sets it, and no
    /// kernel here holds a value of its own.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D3. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
    #[must_use]
    pub const fn bound(self) -> Fix32 {
        self.bound
    }

    /// Returns the condition that one deficit puts a unit in.
    ///
    /// The condition is the deficit read against the bound, and it is the
    /// name a watcher uses. A watcher that read the raw accumulator would
    /// hold the threshold rule in a second place.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub const fn condition(self, deficit: Fix32) -> NeedCondition {
        if deficit.0 <= 0 {
            NeedCondition::Fed
        } else if deficit.0 < self.bound.0 {
            NeedCondition::Short
        } else {
            NeedCondition::Starved
        }
    }

    /// Absorbs the rule into the state hash.
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        hash.write(&self.decay.0.to_le_bytes())
            .write(&self.ration.0.to_le_bytes())
            .write(&self.threshold.0.to_le_bytes())
            .write(&self.recovery.0.to_le_bytes())
            .write(&self.bound.0.to_le_bytes())
    }
}

/// One cohort: the units of one faction that belong to one site.
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
    /// The place of each unit inside its own cohort, in slot order.
    ordinals: Vec<u32>,
}

/// The ordinal of a unit that belongs to no cohort.
///
/// The value sits at the top of the range, which no headcount reaches,
/// because the unit reservation is far below it. It is a property of the
/// column layout and not a budget.
pub const NO_ORDINAL: u32 = u32::MAX;

impl CohortTable {
    /// Builds an empty table.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            rows: Vec::new(),
            unattached: 0,
            ordinals: Vec::new(),
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
        self.ordinals.clear();
        self.ordinals.resize(homes.len(), NO_ORDINAL);
        self.rows.clear();
        self.rows
            .resize(sites as usize * COHORTS_PER_SITE, CohortRow::default());
        for (site, chunk) in self.rows.chunks_mut(COHORTS_PER_SITE).enumerate() {
            for (faction, row) in chunk.iter_mut().enumerate() {
                row.site = site as u32;
                row.faction = faction as u16;
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
                Some(row) => {
                    // **The ordinal is the place of this unit in its own
                    // cohort, in slot order.** The rebuild already walks every
                    // unit and already counts them, so the ordinal costs the
                    // write and nothing else. It is what lets the ration pass
                    // serve exactly as many units as the share covers, without
                    // sorting anything.[^2]
                    //
                    // [^2]: ADR-0106, a cohort serves whole rations to a keyed subset, never an equal share to everybody, decision D2. `docs/adrs/draft/adr-0106-a-cohort-serves-whole-rations-to-a-keyed-subset.md`
                    self.ordinals[slot] = row.headcount;
                    row.headcount += 1;
                }
                // A home that names no live site, or a faction above the
                // ceiling, counts as unattached here. The world refuses
                // both, and its invariant check states so.
                None => self.unattached += 1,
            }
        }
    }

    /// Returns the place of each unit inside its own cohort, in slot order.
    ///
    /// A unit that belongs to no cohort holds `NO_ORDINAL`.
    ///
    /// The column is derived from the unit columns at every rebuild and it
    /// carries nothing between frames, in the same way the rows do. It does
    /// not reach the state hash, because it is an exact function of the
    /// columns that do, and hashing it as well would state one fact in two
    /// places.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub fn ordinals(&self) -> &[u32] {
        &self.ordinals
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
/// [^1]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D3. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
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
            handed = sim_math::combine(handed, Accum(1));
        }

        // The store falls by what the cohorts received, and never by what
        // the transfer meant to give. The two are the same number while the
        // split is exact, and a split that lost a unit would otherwise take
        // that unit out of the world in silence.
        let taken = Fix32(handed.0 as i32);
        stores[offset].set_quantity(commodity, sim_math::sub(held, taken));

        ledger.demanded[index] = sim_math::combine(ledger.demanded[index], demanded);
        ledger.granted[index] = sim_math::combine(ledger.granted[index], handed);
        if handed.0 < demanded.0 {
            let unmet = sim_math::combine(demanded, Accum(-handed.0));
            ledger.unmet[index] = sim_math::combine(ledger.unmet[index], unmet);
            let site = Entity::new(start + offset as u32, generations[offset])
                .expect("a live slot holds a generation of one or more");
            log.push(SiteRationed::new(
                tick,
                site.to_bits(),
                demanded,
                handed,
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

/// The draw index of the place a unit takes in the queue of its cohort.
///
/// The consumption pass takes one draw for each unit in each application. A
/// second draw in the same system and frame must take the next index.[^1]
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
const DRAW_RATION_PLACE: u32 = 0;

/// Returns the whole rations a share covers, and what is left over.
///
/// The share of a cohort covers a whole number of full rations and a
/// remainder below one. The parts sum to the whole, so nothing is created and
/// nothing is lost.[^1]
///
/// A cohort with no headcount serves nobody. A ration of zero would divide by
/// zero, so it serves every member and leaves nothing over: a rule that asks
/// for nothing is met by giving nothing.
///
/// The count never exceeds the headcount. A store that gave more than the
/// cohort asked for would otherwise serve a unit that is not there.
///
/// # References
///
/// [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
fn whole_rations(share: Accum, ration: Fix32, headcount: u32) -> (u32, Fix32) {
    if headcount == 0 || share.0 <= 0 {
        return (0, Fix32::ZERO);
    }
    if ration.0 <= 0 {
        return (headcount, Fix32::ZERO);
    }
    let whole = share.0 / i64::from(ration.0);
    let capped = u32::try_from(whole).unwrap_or(u32::MAX).min(headcount);
    let taken = i64::from(ration.0) * i64::from(capped);
    let left = share.0 - taken;
    let remainder = Fix32(i32::try_from(left).unwrap_or(i32::MAX));
    (capped, remainder)
}

/// What a keyed draw of the consumption pass takes from the frame.
///
/// The two travel together. A draw is keyed on the seed of the world and the
/// frame it runs in, and a caller that could pass one without the other could
/// key a frame against the seed of a different world.[^1]
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
#[derive(Clone, Copy, Debug)]
pub struct DrawKey {
    /// The seed of the world.
    pub seed: u64,
    /// The frame the pass runs in.
    pub tick: Tick,
}

/// Feeds a keyed subset of each cohort whole, and moves the deficit.
///
/// **A cohort serves whole rations to as many of its units as its share
/// covers, and never an equal share to everybody.**[^2] An equal split makes
/// every unit of a cohort numerically identical, because every other input to
/// a need is the same for all of them, and identical units cross the death
/// bound on one tick. Serving a subset is what puts two different needs, and
/// therefore two different deficits, on two units of one cohort.[^3]
///
/// **The subset is the ordinals of a cohort, rotated by an offset keyed on the
/// cohort and the frame.**[^4] A rotation is a bijection, so exactly as many
/// units fall below the served count as the share covered. A draw taken for
/// each unit on its own gives each unit an independent chance, and the number
/// that eats then varies around the count the store paid for.[^3]
///
/// The offset is drawn again on each application, so the block of ordinals
/// that eats slides and no unit is always first. A fixed offset would feed the
/// same units every time, and a cohort would hold a caste rather than a
/// shortage.[^4]
///
/// The parts still sum to the whole. The share covers a whole number of full
/// rations, and the remainder that covers no whole ration goes to the one unit
/// the draw would serve next.[^6]
///
/// The need clamps at the top, and the clamp is safe because a need is not a
/// conserved quantity.[^1]
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
/// [^2]: ADR-0106, a cohort serves whole rations to a keyed subset, never an equal share to everybody, decision D1. `docs/adrs/draft/adr-0106-a-cohort-serves-whole-rations-to-a-keyed-subset.md`
/// [^3]: Findings register, FND-318. `docs/FINDINGS.md`
/// [^4]: ADR-0106, a cohort serves whole rations to a keyed subset, never an equal share to everybody, decision D2. `docs/adrs/draft/adr-0106-a-cohort-serves-whole-rations-to-a-keyed-subset.md`
/// [^6]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
pub fn satisfy(
    rule: NeedRule,
    ration: Fix32,
    key: DrawKey,
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

    // What each cohort serves: the number of whole rations its share covers,
    // and the remainder that covers no whole one. Both run once for each
    // cohort and never once for each unit.
    //
    // **The ration is the one the draw asked with, and not the one the rule
    // holds.** The schedule scales a rate to one application, so a pass that
    // read the rule directly would divide a share taken at the scaled rate by
    // the unscaled one. The caller passes the same value it gave the draw, so
    // the two cannot disagree.[^7]
    //
    // [^7]: ADR-0062, production and upkeep are rates attached to a site, decision D4. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
    let served: Vec<(u32, Fix32)> = table
        .rows()
        .iter()
        .zip(shares)
        .map(|(row, share)| whole_rations(*share, ration, row.headcount))
        .collect();
    let served = &served[..];
    let ordinals = table.ordinals();
    if ordinals.len() != count {
        return Err(CohortError::ColumnsDisagree);
    }
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
                        let row = row_index(home, faction_span[offset].0);
                        if let (Some((whole, remainder)), Some(head)) = (
                            served.get(row),
                            table.rows().get(row).map(|row| row.headcount),
                        ) {
                            // **The place of a unit in the queue of its cohort
                            // is its ordinal, rotated by a keyed offset.** A
                            // rotation is a bijection on the ordinals of a
                            // cohort, so exactly as many units fall below the
                            // served count as the share covered. A draw taken
                            // for each unit on its own would give each unit an
                            // independent chance, and the number that ate
                            // would then vary around the count the store paid
                            // for. That was measured: a cohort whose share
                            // covered one ration served two units on one
                            // application and none on another.[^3]
                            //
                            // The offset is keyed on the cohort and the frame,
                            // so the block of ordinals that eats slides from
                            // one application to the next and no unit is
                            // always first.[^4]
                            let ordinal = ordinals[start + offset];
                            if ordinal == NO_ORDINAL {
                                continue;
                            }
                            let offset_of_frame = rng::draw_below(
                                key.seed,
                                rng::SYSTEM_CONSUMPTION,
                                key.tick.0,
                                row as u64,
                                DRAW_RATION_PLACE,
                                u64::from(head),
                            );
                            let place = (u64::from(ordinal) + offset_of_frame) % u64::from(head);
                            let gain = if place < u64::from(*whole) {
                                ration
                            } else if place == u64::from(*whole) {
                                *remainder
                            } else {
                                Fix32::ZERO
                            };
                            let fed = sim_math::add(need_span[offset], gain);
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

/// What a shortage has done to one unit.
///
/// The condition is a name, and a watcher reads it instead of reading the
/// accumulator. The three values cover the whole range of the accumulator,
/// so a watcher never compares a number against a rule of its own.[^1]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NeedCondition {
    /// The unit carries no deficit.
    Fed,
    /// The unit carries a deficit below the bound. It recovers when the
    /// shortage ends.
    Short,
    /// The unit carries a deficit at or above the bound. The next scan of
    /// the death plane ends it.
    Starved,
}

impl core::fmt::Display for NeedCondition {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Fed => write!(formatter, "fed"),
            Self::Short => write!(formatter, "short"),
            Self::Starved => write!(formatter, "starved"),
        }
    }
}

/// The number of unit slots that one word of the death plane covers.
const PLANE_BITS: usize = 64;

/// A unit that a shortage ended, and the tick that ended it.
///
/// The layout is 8 + 8 + 4 + 4 bytes, which is 24 bytes at an alignment of
/// 8. The trailing array declares every padding byte.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct UnitStarved {
    /// The tick at which the scan ended the unit.
    pub tick: Tick,
    /// The unit that ended, as its identity in bits.
    pub unit: u64,
    /// The deficit that the unit carried. It is at or above the bound.
    pub deficit: Fix32,
    /// The declared padding. Always zero.
    pub padding: [u8; 4],
}

impl UnitStarved {
    /// Builds an event with zero padding.
    #[must_use]
    pub const fn new(tick: Tick, unit: u64, deficit: Fix32) -> Self {
        Self {
            tick,
            unit,
            deficit,
            padding: [0; 4],
        }
    }
}

/// One bit for each unit slot: the units that a pass has marked to end.
///
/// The scan that reads the plane is ordered, because the deaths apply in the
/// order the scan finds them and the scan runs from the lowest slot to the
/// highest.[^1]
///
/// **A pass that partitions the unit slots owns whole words, and it may write
/// into one plane.** The shortage pass does that. Each thread takes a
/// contiguous run of words, so no two threads touch one word.
///
/// **A pass that partitions anything else does not own whole words, and it
/// must take one plane for each thread and join them.** The resolution of a
/// meeting partitions the tiles, and two tiles held by two threads can hold
/// units whose slots share one word. Join with [`Self::union_each`], which is
/// commutative and associative.[^2]
///
/// The safety of the first case is a property of that pass and not of this
/// type. It was documented as a property of this type, and a finding records
/// what that would have cost the second caller.[^3]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
/// [^2]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
/// [^3]: Findings register, FND-401. `docs/FINDINGS.md`
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeathPlane {
    /// One bit for each slot, in slot order, least significant bit first.
    words: Vec<u64>,
}

impl DeathPlane {
    /// Builds an empty plane.
    #[must_use]
    pub const fn new() -> Self {
        Self { words: Vec::new() }
    }

    /// Returns the words of the plane.
    #[must_use]
    pub fn words(&self) -> &[u64] {
        &self.words
    }

    /// Clears every bit and covers the given number of slots.
    pub fn cover(&mut self, slots: usize) {
        self.words.clear();
        self.words.resize(slots.div_ceil(PLANE_BITS), 0);
    }

    /// Marks one slot.
    ///
    /// A mark states that the slot must end. It ends nothing on its own: the
    /// caller applies the marks in ascending slot order, after the pass that
    /// wrote them.[^1]
    ///
    /// A mark outside the covered range writes nothing. A pass that reaches
    /// one has read a slot the plane does not cover, and the caller that
    /// covers the plane owns that.
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub fn mark(&mut self, slot: usize) {
        if let Some(word) = self.words.get_mut(slot / PLANE_BITS) {
            *word |= 1u64 << (slot % PLANE_BITS);
        }
    }

    /// Takes the union of this plane with every plane given.
    ///
    /// A bitwise union is commutative and associative, so the result does not
    /// depend on the order of the planes and does not depend on which thread
    /// wrote which.[^1] A pass whose output ranges are not disjoint gives each
    /// thread its own plane and joins them here.
    ///
    /// A plane of another length contributes the words it holds. The caller
    /// covers every plane to one length, and the invariant check of the caller
    /// owns that.
    ///
    /// # References
    ///
    /// [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    pub fn union_each(&mut self, planes: &[Self]) {
        for plane in planes {
            for (word, other) in self.words.iter_mut().zip(&plane.words) {
                *word |= *other;
            }
        }
    }

    /// Reports whether the plane marks a slot.
    #[must_use]
    pub fn marks(&self, slot: usize) -> bool {
        match self.words.get(slot / PLANE_BITS) {
            Some(word) => word & (1u64 << (slot % PLANE_BITS)) != 0,
            None => false,
        }
    }

    /// Returns the number of slots that the plane marks.
    #[must_use]
    pub fn count(&self) -> u32 {
        self.words.iter().map(|word| word.count_ones()).sum()
    }
}

/// Marks every live unit whose deficit reached the bound.
///
/// The pass is a map over the unit slots. Each thread owns a contiguous run
/// of whole words, so no two threads write one word and the plane is the
/// same at any thread count.[^1]
///
/// The pass ends nothing. It states which units the scan must end, and the
/// scan runs after the barrier.[^2]
///
/// # Errors
///
/// Returns an error when the caller asks for zero threads, when the bound
/// of the rule is below zero, and when the columns hold different lengths.
///
/// # References
///
/// [^1]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
/// [^2]: ADR Registry, row 0020. `docs/adrs/REGISTRY.md`
pub fn mark_starved(
    rule: NeedRule,
    deficits: &[Fix32],
    live: &[u8],
    plane: &mut DeathPlane,
    threads: usize,
) -> Result<(), CohortError> {
    if threads == 0 {
        return Err(CohortError::ZeroThreads);
    }
    if rule.bound().0 < 0 {
        return Err(CohortError::RateBelowZero(rule.bound()));
    }
    if live.len() != deficits.len() {
        return Err(CohortError::ColumnsDisagree);
    }
    plane.cover(deficits.len());
    if deficits.is_empty() {
        return Ok(());
    }
    let word_chunk = plane.words.len().div_ceil(threads).max(1);
    let count = deficits.len();
    std::thread::scope(|scope| {
        let mut base = 0usize;
        for span in plane.words.chunks_mut(word_chunk) {
            let start = (base * PLANE_BITS).min(count);
            base += span.len();
            let stop = (base * PLANE_BITS).min(count);
            let deficit_span = &deficits[start..stop];
            let live_span = &live[start..stop];
            scope.spawn(move || {
                for (offset, deficit) in deficit_span.iter().enumerate() {
                    if live_span[offset] != 1 {
                        continue;
                    }
                    // The condition of the rule is the one statement of
                    // when a unit ends. A comparison against the bound here
                    // would be that rule in a second place.[^1]
                    //
                    // [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
                    if rule.condition(*deficit) == NeedCondition::Starved {
                        span[offset / PLANE_BITS] |= 1u64 << (offset % PLANE_BITS);
                    }
                }
            });
        }
    });
    Ok(())
}

/// Returns the marked slots in ascending slot order.
///
/// The scan reads word by word and bit by bit, from the lowest slot to the
/// highest. It reads no thread and no join, so the order is a property of
/// the plane alone.[^1]
///
/// # Errors
///
/// Returns an error when the caller asks for zero threads.
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
#[cfg(not(feature = "probe-nondeterminism"))]
pub fn starved_order(plane: &DeathPlane, threads: usize) -> Result<Vec<u32>, CohortError> {
    if threads == 0 {
        return Err(CohortError::ZeroThreads);
    }
    let mut order = Vec::with_capacity(plane.count() as usize);
    for (index, word) in plane.words.iter().enumerate() {
        let mut rest = *word;
        while rest != 0 {
            let bit = rest.trailing_zeros() as usize;
            order.push((index * PLANE_BITS + bit) as u32);
            rest &= rest - 1;
        }
    }
    Ok(order)
}

/// Returns the marked slots in the order the output slots joined, which is
/// a defect.
///
/// This is the perturbed build. The scan collects each span of the plane
/// into an output slot and joins the slots, so the order of the deaths
/// follows the join order. The slot probe reverses that order, and the
/// reversal is visible only above one thread, so the thread-count test then
/// fails.
///
/// The whole point is that it must fail. A determinism test with no proven
/// failure mode is decoration.[^1]
///
/// # Errors
///
/// Returns an error when the caller asks for zero threads.
///
/// # References
///
/// [^1]: Testing rules, section 1. `.claude/rules/testing.md`
#[cfg(feature = "probe-nondeterminism")]
pub fn starved_order(plane: &DeathPlane, threads: usize) -> Result<Vec<u32>, CohortError> {
    if threads == 0 {
        return Err(CohortError::ZeroThreads);
    }
    let word_chunk = plane.words.len().div_ceil(threads).max(1);
    let mut slots: Slots<Vec<u32>> =
        Slots::filled(threads, Vec::new()).map_err(|_| CohortError::ZeroThreads)?;
    let mut base = 0usize;
    for (span, slot) in plane.words.chunks(word_chunk).zip(slots.entries_mut()) {
        let start = base;
        base += span.len();
        for (index, word) in span.iter().enumerate() {
            let mut rest = *word;
            while rest != 0 {
                let bit = rest.trailing_zeros() as usize;
                slot.push(((start + index) * PLANE_BITS + bit) as u32);
                rest &= rest - 1;
            }
        }
    }
    Ok(slots.combine(Vec::new(), |mut carried: Vec<u32>, found| {
        carried.extend_from_slice(found);
        carried
    }))
}
