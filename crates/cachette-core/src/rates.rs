//! Production and upkeep as rates attached to a site.
//!
//! A site holds a pooled store.[^1] A rate is what fills that store and what
//! empties it. The rate belongs to the site, so the cost of the pass follows
//! the number of sites and never the number of units that live there.[^2]
//!
//! Every rate is a Q16.16 fixed-point value and every store quantity is the
//! same type. No value here is a floating point number.[^3] Every operation
//! goes through the arithmetic module.[^4]
//!
//! Every rate is at or above zero. Upkeep is a separate non-negative rate
//! and never a negative production rate. Two reasons hold them apart. The
//! scaling multiply truncates towards negative infinity, so a rate below
//! zero carries a permanent downward bias that a rate above zero does
//! not.[^5] And a single net rate cannot say which half a store could not
//! pay, so the shortfall would have nowhere to come from. A cap is not a
//! negative rate, and neither is upkeep.[^6]
//!
//! The arithmetic saturates. A store never wraps, because a wrap turns a
//! large holding into a large debt and hides the defect. Production that
//! the store cannot hold becomes a spill, and upkeep that the store cannot
//! pay becomes a shortfall. The pass drops neither in silence.
//!
//! A store of zero is a real state and not an absent one.[^7]
//!
//! The pass writes one span of slots on each thread, and the spans are
//! disjoint.[^8] It joins the results of the threads in slot order.[^9] The
//! ledger totals are 64-bit integers, so they combine exactly in any
//! order.[^10]
//!
//! # References
//!
//! [^1]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
//! [^2]: ADR-0062, production and upkeep are rates attached to a site, decision D1, a draft record. `docs/adrs/draft/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
//! [^3]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^4]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^5]: Findings register, FND-012. `docs/FINDINGS.md`
//! [^6]: Findings register, FND-016. `docs/FINDINGS.md`
//! [^7]: Findings register, FND-043. `docs/FINDINGS.md`
//! [^8]: ADR-0009, parallel stages write disjoint outputs, decision D1, a draft record. `docs/adrs/draft/adr-0009-parallel-stages-write-disjoint-outputs.md`
//! [^9]: ADR-0004, iteration order is explicit, decision D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
//! [^10]: ADR-0023, an aggregate combines exactly, in any order, decision D3, a draft record. `docs/adrs/draft/adr-0023-an-aggregate-combines-exactly-in-any-order.md`

use bytemuck::{Pod, Zeroable};

use crate::hash::StateHash;
use crate::sim_math;
use crate::site::{CommodityId, Store, StoreUpdate, COMMODITY_COUNT};
use crate::slots::Slots;
use crate::types::{Accum, Entity, Fix32, Tick};

/// The largest period that a schedule holds.
///
/// The period scales a rate through a fixed-point multiply, and that
/// multiply takes the period as a whole number. This value is the range of
/// the whole number. It is a property of the type and not a budget.
pub const PERIOD_LIMIT: u32 = i16::MAX as u32;

/// The reason that this module refused a caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RateError {
    /// The caller asked for zero threads.
    ZeroThreads,
    /// The period is zero, or above the range that the schedule holds.
    PeriodOutsideRange(u32),
    /// The rate is below zero. Upkeep is a rate above zero that subtracts.
    RateBelowZero(Fix32),
    /// The commodity is outside the commodity set.
    CommodityOutsideSet(CommodityId),
    /// The slot is outside the table.
    SlotOutsideTable(u32),
    /// The columns that the pass reads hold different numbers of slots.
    ///
    /// One slot count lives in the arena and a second lives in the rate
    /// table. A check must fail when the two copies disagree.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    ColumnsDisagree,
}

impl core::fmt::Display for RateError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ZeroThreads => write!(formatter, "a pass needs at least one thread"),
            Self::PeriodOutsideRange(period) => write!(
                formatter,
                "the period {period} is zero or above the limit of {PERIOD_LIMIT}"
            ),
            Self::RateBelowZero(rate) => write!(formatter, "the rate {} is below zero", rate.0),
            Self::CommodityOutsideSet(commodity) => write!(
                formatter,
                "the commodity {} is outside the commodity set",
                commodity.0
            ),
            Self::SlotOutsideTable(slot) => {
                write!(formatter, "the slot {slot} is outside the rate table")
            }
            Self::ColumnsDisagree => write!(formatter, "the slot columns hold different lengths"),
        }
    }
}

impl std::error::Error for RateError {}

/// When a rate applies.
///
/// A rate does not apply on every tick. It applies on a period, with a
/// phase offset, and both are parameters of the schedule. No kernel in this
/// module holds either one as a constant.[^1]
///
/// The stored rate is what one tick earns. The pass multiplies it by the
/// period, so what a site earns over a span of ticks does not change when
/// the period changes. The period decides how often a store moves. It does
/// not decide how much the store moves over time.
///
/// # References
///
/// [^1]: ADR-0050, the frame schedule is static and known before the frame runs. `docs/adrs/REGISTRY.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RateSchedule {
    period: u32,
    phase: u32,
}

impl RateSchedule {
    /// The schedule that a world starts with.
    ///
    /// The research report gives the cadence of production and upkeep in its
    /// cadence table, and this is that cadence.[^1] It is a default and not a
    /// constant of the kernel. A caller replaces it.
    ///
    /// # References
    ///
    /// [^1]: Research report 15, needs, consumption and the input-output economy, section 12.1. `docs/research/reports/15-needs-consumption-and-economy.md`
    pub const DEFAULT: Self = Self {
        period: 10,
        phase: 0,
    };

    /// Builds a schedule.
    ///
    /// Returns `None` when the period is zero, and `None` when the period
    /// is above the range that the scaling multiply takes. A period of zero
    /// names a rate that never applies, and a rate of zero already says
    /// that.
    #[must_use]
    pub const fn new(period: u32, phase: u32) -> Option<Self> {
        if period == 0 || period > PERIOD_LIMIT {
            return None;
        }
        Some(Self {
            period,
            phase: phase % period,
        })
    }

    /// Returns the number of ticks between two applications.
    #[must_use]
    pub const fn period(self) -> u32 {
        self.period
    }

    /// Returns the phase offset inside the period.
    #[must_use]
    pub const fn phase(self) -> u32 {
        self.phase
    }

    /// Reports whether the rate applies on this tick.
    #[must_use]
    pub const fn due(self, tick: Tick) -> bool {
        tick.0 % (self.period as u64) == self.phase as u64
    }

    /// Scales a per-tick rate to the amount of one application.
    ///
    /// The multiply truncates towards negative infinity and then saturates.
    /// A rate at or above zero therefore loses at most the fraction that
    /// the scale cannot hold, and it never gains. A rate below zero would
    /// gain a permanent downward bias instead, which is why the table
    /// refuses one.[^1]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-012. `docs/FINDINGS.md`
    #[must_use]
    pub const fn per_application(self, rate: Fix32) -> Fix32 {
        sim_math::mul(rate, Fix32::from_int(self.period as i16))
    }
}

/// The production rate and the upkeep rate of one site.
///
/// Both arrays hold one rate for each commodity, in commodity order. Both
/// hold the same element type, so the layout carries no padding byte.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct SiteRate {
    /// What the site earns of each commodity in one tick.
    pub production: [Fix32; COMMODITY_COUNT],
    /// What the site owes of each commodity in one tick.
    pub upkeep: [Fix32; COMMODITY_COUNT],
}

impl SiteRate {
    /// The site that earns nothing and owes nothing.
    pub const IDLE: Self = Self {
        production: [Fix32::ZERO; COMMODITY_COUNT],
        upkeep: [Fix32::ZERO; COMMODITY_COUNT],
    };
}

/// The rates of every site, indexed by the slot of the site.
///
/// The table is a dense column beside the settlement columns. A site that
/// earns nothing still holds a row, because a dense column costs one read
/// and a sparse one costs a search.
#[derive(Clone, Debug, Default)]
pub struct RateTable {
    rows: Vec<SiteRate>,
}

impl RateTable {
    /// Builds an empty table.
    #[must_use]
    pub const fn new() -> Self {
        Self { rows: Vec::new() }
    }

    /// Returns the number of slots that the table holds.
    #[must_use]
    pub fn slot_count(&self) -> u32 {
        self.rows.len() as u32
    }

    /// Reports whether the table holds no slot.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Opens rows until the table holds this many of them.
    ///
    /// A new row earns nothing and owes nothing. The table never shrinks,
    /// because the slot index space never shrinks.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    pub fn open_to(&mut self, slots: u32) {
        if (slots as usize) > self.rows.len() {
            self.rows.resize(slots as usize, SiteRate::IDLE);
        }
    }

    /// Clears the rates of one slot.
    ///
    /// The world does this when a slot changes hands. A rate that outlived
    /// the site that earned it would pay a settlement that no longer
    /// stands.
    pub fn clear_slot(&mut self, slot: u32) {
        if let Some(row) = self.rows.get_mut(slot as usize) {
            *row = SiteRate::IDLE;
        }
    }

    /// Returns the whole table.
    #[must_use]
    pub fn rows(&self) -> &[SiteRate] {
        &self.rows
    }

    /// Returns the production rate of one slot and one commodity.
    #[must_use]
    pub fn production(&self, slot: u32, commodity: CommodityId) -> Option<Fix32> {
        self.rows
            .get(slot as usize)
            .and_then(|row| row.production.get(commodity.0 as usize))
            .copied()
    }

    /// Returns the upkeep rate of one slot and one commodity.
    #[must_use]
    pub fn upkeep(&self, slot: u32, commodity: CommodityId) -> Option<Fix32> {
        self.rows
            .get(slot as usize)
            .and_then(|row| row.upkeep.get(commodity.0 as usize))
            .copied()
    }

    /// Writes the production rate of one slot and one commodity.
    ///
    /// # Errors
    ///
    /// Returns an error when the rate is below zero, when the slot is
    /// outside the table, or when the commodity is outside the set.
    pub fn set_production(
        &mut self,
        slot: u32,
        commodity: CommodityId,
        rate: Fix32,
    ) -> Result<(), RateError> {
        let checked = Self::checked(rate)?;
        *Self::field(&mut self.rows, slot, commodity, false)? = checked;
        Ok(())
    }

    /// Writes the upkeep rate of one slot and one commodity.
    ///
    /// # Errors
    ///
    /// Returns an error when the rate is below zero, when the slot is
    /// outside the table, or when the commodity is outside the set.
    pub fn set_upkeep(
        &mut self,
        slot: u32,
        commodity: CommodityId,
        rate: Fix32,
    ) -> Result<(), RateError> {
        let checked = Self::checked(rate)?;
        *Self::field(&mut self.rows, slot, commodity, true)? = checked;
        Ok(())
    }

    /// Refuses a rate below zero.
    const fn checked(rate: Fix32) -> Result<Fix32, RateError> {
        if rate.0 < 0 {
            return Err(RateError::RateBelowZero(rate));
        }
        Ok(rate)
    }

    /// Resolves one field of one row.
    fn field(
        rows: &mut [SiteRate],
        slot: u32,
        commodity: CommodityId,
        upkeep: bool,
    ) -> Result<&mut Fix32, RateError> {
        let row = rows
            .get_mut(slot as usize)
            .ok_or(RateError::SlotOutsideTable(slot))?;
        let array = if upkeep {
            &mut row.upkeep
        } else {
            &mut row.production
        };
        array
            .get_mut(commodity.0 as usize)
            .ok_or(RateError::CommodityOutsideSet(commodity))
    }

    /// Absorbs the table into the state hash.
    ///
    /// A rate is state that a later frame reads. A hash that ignored it
    /// would pass while the rates changed underneath it.
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        hash.write_u64(u64::from(self.slot_count()))
            .write(bytemuck::cast_slice(&self.rows))
    }

    /// Reports whether every rate in the table is at or above zero.
    #[must_use]
    pub fn check_invariants(&self) -> bool {
        self.rows.iter().all(|row| {
            row.production.iter().all(|rate| rate.0 >= 0)
                && row.upkeep.iter().all(|rate| rate.0 >= 0)
        })
    }
}

/// A site could not pay its upkeep.
///
/// The event names what the upkeep asked for and could not take. The store
/// stopped at zero rather than going below it, so the amount here is what
/// the world must supply to make the site solvent.
///
/// The layout is 8 + 8 + 4 + 2 + 2 bytes, which is 24 bytes at an alignment
/// of 8. The trailing array declares every padding byte, so the type holds
/// no uninitialised byte.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct SiteShortfall {
    /// The tick at which the upkeep applied.
    pub tick: Tick,
    /// The site that could not pay, as its identity in bits.
    pub site: u64,
    /// What the upkeep could not take. It is never zero.
    pub amount: Fix32,
    /// The commodity that the site owed.
    pub commodity: u16,
    /// The declared padding. Always zero.
    pub padding: [u8; 2],
}

impl SiteShortfall {
    /// Builds an event with zero padding.
    #[must_use]
    pub const fn new(tick: Tick, site: u64, amount: Fix32, commodity: u16) -> Self {
        Self {
            tick,
            site,
            amount,
            commodity,
            padding: [0; 2],
        }
    }
}

/// The running account of every rate that has applied.
///
/// Each field is a 64-bit accumulator, so a sum over many sites and many
/// ticks is exact and combines in any order.[^1]
///
/// The fields answer one equality. What a site held, plus what landed in
/// it, minus what upkeep took, is what it holds. The spill and the
/// shortfall are the two amounts that saturation refused, and neither one
/// enters that equality. The ledger reports them so that nothing is dropped
/// in silence.
///
/// # References
///
/// [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D3, a draft record. `docs/adrs/draft/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct RateLedger {
    /// What production put into a store, for each commodity.
    pub produced: [Accum; COMMODITY_COUNT],
    /// What upkeep took out of a store, for each commodity.
    pub spent: [Accum; COMMODITY_COUNT],
    /// What production could not put into a store, because the store stood
    /// at its ceiling.
    pub spilled: [Accum; COMMODITY_COUNT],
    /// What upkeep could not take, because the store stood at zero.
    pub shortfall: [Accum; COMMODITY_COUNT],
    /// The number of site visits that the pass made.
    pub visited: [Accum; 1],
}

impl RateLedger {
    /// The ledger that records nothing.
    pub const ZERO: Self = Self {
        produced: [Accum(0); COMMODITY_COUNT],
        spent: [Accum(0); COMMODITY_COUNT],
        spilled: [Accum(0); COMMODITY_COUNT],
        shortfall: [Accum(0); COMMODITY_COUNT],
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
            joined.produced[index] =
                sim_math::combine(joined.produced[index], other.produced[index]);
            joined.spent[index] = sim_math::combine(joined.spent[index], other.spent[index]);
            joined.spilled[index] = sim_math::combine(joined.spilled[index], other.spilled[index]);
            joined.shortfall[index] =
                sim_math::combine(joined.shortfall[index], other.shortfall[index]);
        }
        joined.visited[0] = sim_math::combine(joined.visited[0], other.visited[0]);
        joined
    }

    /// Returns the net change that the ledger made to the store column.
    ///
    /// The net is what landed minus what was taken. A spill never landed
    /// and a shortfall was never taken, so neither one appears here.
    #[must_use]
    pub fn net(&self, commodity: CommodityId) -> Option<Accum> {
        let index = commodity.0 as usize;
        let produced = self.produced.get(index)?;
        let spent = self.spent.get(index)?;
        Some(Accum(produced.0 - spent.0))
    }

    /// Absorbs the ledger into the state hash.
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        hash.write(bytemuck::bytes_of(self))
    }
}

/// What one application of the rates did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RatePass {
    /// The totals of the application.
    pub ledger: RateLedger,
    /// One event for each site that could not pay, in slot order.
    pub shortfalls: Vec<SiteShortfall>,
}

impl RatePass {
    /// The application that did nothing.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            ledger: RateLedger::ZERO,
            shortfalls: Vec::new(),
        }
    }

    /// Reports whether the application changed nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ledger == RateLedger::ZERO && self.shortfalls.is_empty()
    }
}

/// Applies every rate to every live site.
///
/// The pass runs when the schedule is due, and it does nothing otherwise.
/// It visits each live slot once, produces into the store, then spends from
/// it. Production runs before upkeep, so a site pays this bill from these
/// earnings. The reverse order would make a site that earns exactly what it
/// owes insolvent on every application.
///
/// The pass changes no structure. It writes one column, and it writes the
/// entry of a slot only in the thread that owns that slot.[^1]
///
/// # Errors
///
/// Returns an error when the caller asks for zero threads, and when the
/// columns hold different numbers of slots.
///
/// # References
///
/// [^1]: ADR-0009, parallel stages write disjoint outputs, decision D1, a draft record. `docs/adrs/draft/adr-0009-parallel-stages-write-disjoint-outputs.md`
pub fn apply(
    schedule: RateSchedule,
    tick: Tick,
    table: &RateTable,
    update: StoreUpdate<'_>,
    threads: usize,
) -> Result<RatePass, RateError> {
    if threads == 0 {
        return Err(RateError::ZeroThreads);
    }
    let StoreUpdate {
        stores,
        live,
        generations,
    } = update;
    let count = stores.len();
    if live.len() != count || generations.len() != count || table.rows().len() != count {
        return Err(RateError::ColumnsDisagree);
    }
    if !schedule.due(tick) || count == 0 {
        return Ok(RatePass::empty());
    }

    let rows = table.rows();
    let chunk_len = count.div_ceil(threads).max(1);
    let mut slots: Slots<RatePass> =
        Slots::filled(threads, RatePass::empty()).map_err(|_| RateError::ZeroThreads)?;

    std::thread::scope(|scope| {
        let mut base = 0usize;
        for (span, slot) in stores.chunks_mut(chunk_len).zip(slots.entries_mut()) {
            let start = base;
            base += span.len();
            let live_span = &live[start..base];
            let generation_span = &generations[start..base];
            let rate_span = &rows[start..base];
            scope.spawn(move || {
                *slot = apply_span(
                    schedule,
                    tick,
                    start as u32,
                    span,
                    live_span,
                    generation_span,
                    rate_span,
                );
            });
        }
    });

    // The ledger combine is order-free, because every term is an integer
    // addition. The shortfall log is not order-free: a concatenation depends
    // on the order it reads the slots in, so it takes the fixed slot
    // order.[^1]
    //
    // [^1]: ADR-0004, iteration order is explicit, decisions D2 and D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    let ledger = slots.combine(RateLedger::ZERO, |carried, pass| {
        carried.combine(pass.ledger)
    });
    let shortfalls = slots.combine(Vec::new(), |mut carried: Vec<SiteShortfall>, pass| {
        carried.extend_from_slice(&pass.shortfalls);
        carried
    });
    Ok(RatePass { ledger, shortfalls })
}

/// Applies the rates over one span of slots.
///
/// The span is contiguous and it belongs to one thread. The function reads
/// and writes nothing outside the span.
fn apply_span(
    schedule: RateSchedule,
    tick: Tick,
    start: u32,
    stores: &mut [Store],
    live: &[u8],
    generations: &[u32],
    rates: &[SiteRate],
) -> RatePass {
    let mut pass = RatePass::empty();
    for offset in 0..stores.len() {
        if live[offset] != 1 {
            continue;
        }
        pass.ledger.visited[0] = sim_math::combine(pass.ledger.visited[0], Accum(1));
        let slot = start + offset as u32;
        let row = rates[offset];
        for index in 0..COMMODITY_COUNT {
            let commodity = CommodityId(index as u16);
            let held = stores[offset]
                .quantity(commodity)
                .expect("the index came from the commodity count");
            let production = schedule.per_application(row.production[index]);
            let upkeep = schedule.per_application(row.upkeep[index]);

            // Production runs first. The store saturates at its ceiling, and
            // what did not fit is a spill rather than a wrap.
            let grown = sim_math::add(held, production);
            let landed = sim_math::sub(grown, held);
            let spilled = sim_math::sub(production, landed);

            // Upkeep runs second. The store stops at zero, and what it could
            // not pay is a shortfall rather than a debt.
            let spent = if grown < upkeep { grown } else { upkeep };
            let after = sim_math::sub(grown, spent);
            let shortfall = sim_math::sub(upkeep, spent);

            stores[offset].set_quantity(commodity, after);
            pass.ledger.produced[index] = sim_math::accumulate(pass.ledger.produced[index], landed);
            pass.ledger.spent[index] = sim_math::accumulate(pass.ledger.spent[index], spent);
            pass.ledger.spilled[index] = sim_math::accumulate(pass.ledger.spilled[index], spilled);
            if shortfall != Fix32::ZERO {
                pass.ledger.shortfall[index] =
                    sim_math::accumulate(pass.ledger.shortfall[index], shortfall);
                let site = Entity::new(slot, generations[offset])
                    .expect("a live slot holds a generation of one or more");
                pass.shortfalls.push(SiteShortfall::new(
                    tick,
                    site.to_bits(),
                    shortfall,
                    commodity.0,
                ));
            }
        }
    }
    pass
}
