//! Who holds a tile.
//!
//! A tile carries one holder, and the holder is a faction or nobody. The
//! holder is one dense column over the tiles, so no field of the world is
//! indexed by the faction.[^1] [^2] Exclusivity is therefore a property of
//! the storage: one tile holds one value, so no tile can name two factions
//! and no rule has to keep two factions apart.
//!
//! A faction is one bit in a 64-bit mask. A set of factions is one mask, and
//! the world stores one mask for each block of tiles. A query that asks where
//! a faction holds reads the masks, passes over every block that does not
//! name the faction, and walks only the blocks that do.[^1]
//!
//! **The count of what a faction holds is a running total.** The rule that
//! changes a holder adds one to the total of the faction that gained and
//! takes one from the total of the faction that lost. The answer therefore
//! costs nothing at read time, and maintaining it costs the tiles that
//! changed rather than the tiles that exist.[^3]
//!
//! **The cities decide the holder.** A tile is held by the faction of the
//! nearest city that reaches it. Two cities at one distance resolve by the
//! lower settlement slot. A tile no city reaches is held by nobody, and a
//! tile whose ground admits no unit is held by nobody whatever reaches
//! it.[^5]
//!
//! **The reach of a city is a whole number of hex steps.** It is a base, plus
//! one step for each block of finished upgrades that stand on the ground the
//! city held at the end of the previous step, and it never passes a bound.
//! The three values are balance rows.[^6]
//!
//! **The rule reads one buffer and writes another.** Every candidate tile is
//! decided against the settlement table alone, so the answer does not depend
//! on the order in which the candidates were visited, and it does not depend
//! on how many threads visited them.[^4]
//!
//! # References
//!
//! [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
//! [^2]: ADR-0012, tiles are dense columns and units are a generational arena, decision D2. `docs/adrs/accepted/adr-0012-tiles-are-dense-columns-and-units-are-a-generational-arena.md`
//! [^3]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
//! [^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
//! [^5]: ADR-0150, held ground is the ground within reach of a city its faction owns, decisions D1 and D3. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
//! [^6]: Balance register, the holding. `docs/reference/balance.md`

use bytemuck::{Pod, Zeroable};

use crate::bridge::{BlockLayout, BridgeError};
/// Counts what the holding apply does on each frame.
///
/// **The switch exists because the apply has three parts and the stage table
/// names only the whole.** The apply writes the holder of each changed tile,
/// rebuilds the list of held tiles, and repairs the block mask of every block
/// a change touched. Those grow with different things, and a figure for the
/// stage says nothing about which one carries it.
///
/// The counters observe. Nothing reads them inside the engine, and no
/// simulated value depends on one, so they cannot reach a result.[^1]
///
/// The whole module compiles to nothing when the switch is off.
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
#[cfg(feature = "census-holding")]
pub mod census {
    use core::sync::atomic::{AtomicU64, Ordering};

    /// The tiles whose holder changed, since the last reset.
    static MOVED: AtomicU64 = AtomicU64::new(0);
    /// The blocks whose mask was read again, since the last reset.
    static DIRTY: AtomicU64 = AtomicU64::new(0);
    /// The entries the held list was rebuilt with, since the last reset.
    static REBUILT: AtomicU64 = AtomicU64::new(0);

    /// The candidate tiles the decide pass read, since the last reset.
    static DECIDED: AtomicU64 = AtomicU64::new(0);
    /// The supporters those tiles raised, counted with repetition.
    static SUPPORTERS: AtomicU64 = AtomicU64::new(0);
    /// The candidate tiles that raised more than one supporter.
    static SORTED: AtomicU64 = AtomicU64::new(0);
    /// The candidate tiles that had a challenger able to beat the holder.
    static CHALLENGED: AtomicU64 = AtomicU64::new(0);

    /// Records one apply.
    pub fn record(moved: u64, dirty: u64, rebuilt: u64) {
        MOVED.fetch_add(moved, Ordering::Relaxed);
        DIRTY.fetch_add(dirty, Ordering::Relaxed);
        REBUILT.fetch_add(rebuilt, Ordering::Relaxed);
    }

    /// Records one candidate tile that the decide pass read.
    ///
    /// **The decide pass runs on several threads, so these counters are
    /// shared.** They are relaxed atomics because nothing reads them during a
    /// frame and no simulated value depends on one. A count that a thread
    /// races on would be wrong by a few and would still answer the question
    /// this switch exists for, which is what the pass does per candidate.
    pub fn record_decide(supporters: u64, sorted: bool, challenged: bool) {
        DECIDED.fetch_add(1, Ordering::Relaxed);
        SUPPORTERS.fetch_add(supporters, Ordering::Relaxed);
        if sorted {
            SORTED.fetch_add(1, Ordering::Relaxed);
        }
        if challenged {
            CHALLENGED.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Returns the candidates read, the supporters raised, the candidates
    /// that sorted, and the candidates that had a challenger.
    #[must_use]
    pub fn decide_totals() -> (u64, u64, u64, u64) {
        (
            DECIDED.load(Ordering::Relaxed),
            SUPPORTERS.load(Ordering::Relaxed),
            SORTED.load(Ordering::Relaxed),
            CHALLENGED.load(Ordering::Relaxed),
        )
    }

    /// Returns the moved tiles, the dirty blocks and the rebuilt entries.
    #[must_use]
    pub fn totals() -> (u64, u64, u64) {
        (
            MOVED.load(Ordering::Relaxed),
            DIRTY.load(Ordering::Relaxed),
            REBUILT.load(Ordering::Relaxed),
        )
    }

    /// Sets every count back to zero.
    pub fn reset() {
        MOVED.store(0, Ordering::Relaxed);
        DIRTY.store(0, Ordering::Relaxed);
        REBUILT.store(0, Ordering::Relaxed);
        DECIDED.store(0, Ordering::Relaxed);
        SUPPORTERS.store(0, Ordering::Relaxed);
        SORTED.store(0, Ordering::Relaxed);
        CHALLENGED.store(0, Ordering::Relaxed);
    }
}

use crate::hash::StateHash;
use crate::hex::{Axial, Grid};
use crate::site::SettlementArena;
use crate::slots::Slots;
use crate::stage::{self, Stage};
use crate::terrain::{Terrain, TileKind};
use crate::types::{FactionId, TileIdx, FACTION_CEILING};
use crate::upgrade::UpgradeMap;

/// The number of bits in a faction mask.
///
/// A faction is one bit of a 64-bit word. The addressable set stops one below
/// this, because the top bit names every faction outside the addressable
/// set.[^1]
///
/// # References
///
/// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D1. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
pub const MASK_BITS: u32 = 64;

/// The bit that names a faction outside the addressable set.
///
/// Nothing sets it yet, because a world refuses a faction at or above the
/// ceiling. It is reserved so that a later minor faction does not have to
/// take an addressable slot, and so that a disjunctive query keeps working
/// when one arrives.[^1]
///
/// # References
///
/// [^1]: Research report 08, fog of war representation, section 6.4. `docs/research/reports/08-fog-of-war-representation.md`
pub const OVERFLOW_BIT: u32 = 63;

/// The holder of one tile.
///
/// The value names a faction, or nobody. It is one field, so a tile cannot
/// name two factions.[^1]
///
/// # References
///
/// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct Holder(u16);

impl Holder {
    /// The value that names no faction.
    pub const NOBODY: Self = Self(u16::MAX);

    /// Returns the holder that names one faction.
    #[must_use]
    pub const fn of(faction: FactionId) -> Self {
        Self(faction.0)
    }

    /// Returns the faction the holder names, or `None` for nobody.
    #[must_use]
    pub const fn faction(self) -> Option<FactionId> {
        if self.0 == u16::MAX {
            None
        } else {
            Some(FactionId(self.0))
        }
    }

    /// Reports whether the holder names no faction.
    #[must_use]
    pub const fn is_nobody(self) -> bool {
        self.0 == u16::MAX
    }

    /// Returns the holder as a raw number. The state hash reads it.
    #[must_use]
    pub const fn to_bits(self) -> u16 {
        self.0
    }
}

impl Default for Holder {
    fn default() -> Self {
        Self::NOBODY
    }
}

/// A set of factions.
///
/// The set is one 64-bit word, so it costs the same whatever the number of
/// factions in it. A field of the world holds one of these. A field of the
/// world never holds one value for each faction.[^1]
///
/// # References
///
/// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D3. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct FactionMask(u64);

impl FactionMask {
    /// The set that holds no faction.
    pub const EMPTY: Self = Self(0);

    /// Returns the set that holds one faction.
    ///
    /// A faction outside the addressable set takes the overflow bit, so a
    /// query that asks whether anybody holds the ground keeps working.
    #[must_use]
    pub const fn of(faction: FactionId) -> Self {
        if (faction.0 as u32) < OVERFLOW_BIT {
            Self(1u64 << faction.0)
        } else {
            Self(1u64 << OVERFLOW_BIT)
        }
    }

    /// Adds a faction to the set.
    #[must_use]
    pub const fn with(self, faction: FactionId) -> Self {
        Self(self.0 | Self::of(faction).0)
    }

    /// Returns the set with one faction removed.
    ///
    /// Removing a faction the set does not hold gives the same set.
    #[must_use]
    pub const fn without(self, faction: FactionId) -> Self {
        if faction.0 as u32 >= MASK_BITS {
            return self;
        }
        Self(self.0 & !(1u64 << faction.0))
    }

    /// Returns the union of two sets.
    ///
    /// The operation is associative, commutative and exact, so a fold over a
    /// group of masks gives one answer whatever the order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D2. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Reports whether the set holds a faction.
    #[must_use]
    pub const fn contains(self, faction: FactionId) -> bool {
        self.0 & Self::of(faction).0 != 0
    }

    /// Reports whether the set holds no faction.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns the number of factions in the set.
    #[must_use]
    pub const fn len(self) -> u32 {
        self.0.count_ones()
    }

    /// Returns the set as a raw word.
    #[must_use]
    pub const fn to_bits(self) -> u64 {
        self.0
    }
}

/// How far a city reaches, and what extends the reach.
///
/// The reach of a city is a base, plus one step for each block of finished
/// upgrades that stand on the ground the city held at the end of the previous
/// step, and it never passes the cap.[^1] The three numbers are balance rows,
/// and every value in that register is unset until the balance pass measures
/// it. The defaults here are the provisional values the register holds, and
/// the register holds the derivation of each.[^2]
///
/// The arithmetic is whole numbers. No fraction and no floating point number
/// reaches a holder.[^3]
///
/// # References
///
/// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D2. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
/// [^2]: Balance register, the holding. `docs/reference/balance.md`
/// [^3]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReachRules {
    base: u32,
    upgrades_per_step: u32,
    cap: u32,
}

impl ReachRules {
    /// The provisional values that the balance register holds.
    ///
    /// The bound stands above the distance the seeding keeps between two
    /// foundings, and the base stands below it. A capital is therefore out
    /// of reach of a rival capital until the faction has built the ground
    /// that extends the reach.
    pub const DEFAULT: Self = Self {
        base: 8,
        upgrades_per_step: 4,
        cap: 16,
    };

    /// Builds a rule set.
    ///
    /// A block of zero upgrades would divide by zero, so the count is raised
    /// to one. A cap below the base holds the reach at the cap, which is what
    /// a cap means.
    #[must_use]
    pub const fn new(base: u32, upgrades_per_step: u32, cap: u32) -> Self {
        Self {
            base,
            upgrades_per_step: if upgrades_per_step == 0 {
                1
            } else {
                upgrades_per_step
            },
            cap,
        }
    }

    /// Returns the reach with no finished upgrade.
    #[must_use]
    pub const fn base(self) -> u32 {
        self.base
    }

    /// Returns the finished upgrades that earn one step of reach.
    #[must_use]
    pub const fn upgrades_per_step(self) -> u32 {
        self.upgrades_per_step
    }

    /// Returns the reach that a city never passes.
    #[must_use]
    pub const fn cap(self) -> u32 {
        self.cap
    }

    /// Returns the reach of a city that holds this many finished upgrades.
    #[must_use]
    pub const fn reach_of(self, finished: u32) -> u32 {
        let grown = self.base.saturating_add(finished / self.upgrades_per_step);
        if grown > self.cap {
            self.cap
        } else {
            grown
        }
    }
}

impl Default for ReachRules {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// How much a lease rises for each tick a unit of its own faction stands on
/// the tile, when nobody has set another value.
///
/// **This is a provisional value and not a measured one.** The rules of the
/// downstream game are not written down, so a blocker governs it.[^1] The
/// balance register holds the row.[^2]
///
/// # References
///
/// [^1]: Blockers register, BLK-050. `docs/BLOCKERS.md`
/// [^2]: Balance register, the lease. `docs/reference/balance.md`
pub const LEASE_RAISE_STEP_DEFAULT: i32 = 1;

/// How much a lease falls for each tick a unit of another faction stands on
/// the tile, when nobody has set another value.
///
/// **This is a provisional value and not a measured one.** It is four times
/// the raise step, so an invader strips a lease at the bound in a quarter of
/// the ticks the holder spent to build it. A step equal to the raise step
/// would make a seat at the bound cost an invader as many ticks as the
/// holder's whole run, and the seat clause of the game end would stay out of
/// reach.[^1] [^2]
///
/// # References
///
/// [^1]: Blockers register, BLK-050. `docs/BLOCKERS.md`
/// [^2]: Balance register, the lease. `docs/reference/balance.md`
pub const LEASE_LOWER_STEP_DEFAULT: i32 = 4;

/// How much a lease falls on each decay tick, when nobody has set another
/// value.
///
/// **This is a provisional value and not a measured one.**[^1] [^2]
///
/// # References
///
/// [^1]: Blockers register, BLK-050. `docs/BLOCKERS.md`
/// [^2]: Balance register, the lease. `docs/reference/balance.md`
pub const LEASE_DECAY_STEP_DEFAULT: i32 = 1;

/// How many ticks pass between two decay ticks, when nobody has set another
/// value.
///
/// **This is a provisional value and not a measured one.** A lease at the
/// bound returns to nobody after the bound multiplied by this period, which
/// is 6144 ticks. That is under a third of the tick limit of the harness, so
/// ground a faction abandons early in a run returns inside the run.[^1] [^2]
///
/// # References
///
/// [^1]: Blockers register, BLK-050. `docs/BLOCKERS.md`
/// [^2]: Balance register, the lease. `docs/reference/balance.md`
pub const LEASE_DECAY_PERIOD_DEFAULT: u32 = 32;

/// The tick inside the decay period on which the decay runs, when nobody has
/// set another value.
///
/// **This is a provisional value and not a measured one.**[^1] [^2]
///
/// # References
///
/// [^1]: Blockers register, BLK-050. `docs/BLOCKERS.md`
/// [^2]: Balance register, the lease. `docs/reference/balance.md`
pub const LEASE_DECAY_PHASE_DEFAULT: u32 = 0;

/// The count a lease never passes, when nobody has set another value.
///
/// **This is a provisional value and not a measured one.** It is three times
/// the claim threshold, so a faction that used one tile for a whole run holds
/// no more claim on it than three claims, and another faction takes the tile
/// in a stated number of ticks rather than never.[^1] [^2]
///
/// # References
///
/// [^1]: Blockers register, BLK-050. `docs/BLOCKERS.md`
/// [^2]: Balance register, the lease. `docs/reference/balance.md`
pub const LEASE_BOUND_DEFAULT: i32 = 192;

/// The count at or above which a lease holds the tile, when nobody has set
/// another value.
///
/// **This is a provisional value and not a measured one.** A unit that merely
/// crosses a tile stands on it for one tick, and this threshold asks for 64,
/// so a crossing claims nothing and repeated use claims. It is far below the
/// tick limit of the harness, so a cohort that reaches an objective and stays
/// takes it inside the run.[^1] [^2]
///
/// # References
///
/// [^1]: Blockers register, BLK-050. `docs/BLOCKERS.md`
/// [^2]: Balance register, the lease. `docs/reference/balance.md`
pub const LEASE_CLAIM_THRESHOLD_DEFAULT: i32 = 64;

/// How a lease rises, falls and claims.
///
/// Every value is a whole number. No fraction and no floating point number
/// reaches a lease.[^1] Each one is a balance value under one blocker.[^2]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
/// [^2]: Balance register, the lease. `docs/reference/balance.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LeaseRules {
    raise_step: i32,
    lower_step: i32,
    decay_step: i32,
    decay_period: u32,
    decay_phase: u32,
    bound: i32,
    claim_threshold: i32,
}

impl LeaseRules {
    /// The provisional values that the balance register holds.
    pub const DEFAULT: Self = Self {
        raise_step: LEASE_RAISE_STEP_DEFAULT,
        lower_step: LEASE_LOWER_STEP_DEFAULT,
        decay_step: LEASE_DECAY_STEP_DEFAULT,
        decay_period: LEASE_DECAY_PERIOD_DEFAULT,
        decay_phase: LEASE_DECAY_PHASE_DEFAULT,
        bound: LEASE_BOUND_DEFAULT,
        claim_threshold: LEASE_CLAIM_THRESHOLD_DEFAULT,
    };

    /// Builds a rule set.
    ///
    /// A period of zero would divide by zero, so the count is raised to one.
    /// A claim threshold above the bound would make the lease reach nothing,
    /// so it is held at the bound.
    #[must_use]
    pub const fn new(
        raise_step: i32,
        lower_step: i32,
        decay_step: i32,
        decay_period: u32,
        decay_phase: u32,
        bound: i32,
        claim_threshold: i32,
    ) -> Self {
        let bound = if bound < 0 { 0 } else { bound };
        Self {
            raise_step,
            lower_step,
            decay_step,
            decay_period: if decay_period == 0 { 1 } else { decay_period },
            decay_phase,
            bound,
            claim_threshold: if claim_threshold > bound {
                bound
            } else {
                claim_threshold
            },
        }
    }

    /// Returns how much a lease rises for each tick of its own faction.
    #[must_use]
    pub const fn raise_step(self) -> i32 {
        self.raise_step
    }

    /// Returns how much a lease falls for each tick of another faction.
    #[must_use]
    pub const fn lower_step(self) -> i32 {
        self.lower_step
    }

    /// Returns how much a lease falls on a decay tick.
    #[must_use]
    pub const fn decay_step(self) -> i32 {
        self.decay_step
    }

    /// Returns how many ticks pass between two decay ticks.
    #[must_use]
    pub const fn decay_period(self) -> u32 {
        self.decay_period
    }

    /// Returns the tick inside the period on which the decay runs.
    #[must_use]
    pub const fn decay_phase(self) -> u32 {
        self.decay_phase
    }

    /// Returns the count a lease never passes.
    #[must_use]
    pub const fn bound(self) -> i32 {
        self.bound
    }

    /// Returns the count at or above which a lease holds the tile.
    #[must_use]
    pub const fn claim_threshold(self) -> i32 {
        self.claim_threshold
    }

    /// Returns whether the decay runs on this tick.
    ///
    /// The schedule is fixed. No convergence test, no time budget and no wall
    /// clock ends it, because a run must give one answer at any thread count
    /// and on any machine.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[must_use]
    pub const fn decays_on(self, tick: u64) -> bool {
        tick % self.decay_period as u64 == self.decay_phase as u64 % self.decay_period as u64
    }
}

impl Default for LeaseRules {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// The holding of a world.
///
/// It holds the holder of each tile, the list of tiles that somebody holds,
/// the count for each faction, and one faction mask for each block. The
/// holder column is the truth. The other three are derived from it, and the
/// invariant check derives each of them again and compares.[^1]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
#[derive(Clone, Debug)]
pub struct Holding {
    layout: BlockLayout,
    holders: Vec<Holder>,
    /// The tiles that somebody holds, in ascending tile order.
    held: Vec<TileIdx>,
    /// The number of tiles each faction holds, indexed by the faction bit.
    census: [i64; MASK_BITS as usize],
    /// The factions that hold ground in each block.
    ///
    /// The mask is derived from the counts below and is kept beside them, so
    /// that a reader of one block pays one read rather than one for each
    /// faction. The invariant check derives both from the holder column and
    /// fails when either disagrees.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    block_masks: Vec<FactionMask>,
    /// The number of tiles each faction holds in each block.
    ///
    /// The entry of a block and a faction sits at the block index times the
    /// mask width plus the faction bit.
    ///
    /// **This exists so that a mask never has to be read again from the
    /// tiles.** A block loses a faction bit exactly when that faction's count
    /// in the block reaches zero, and a count sees that in one step. Reading
    /// the block instead costs every tile of it, and a frame at the target
    /// scale dirties most blocks.[^1]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-307. `docs/FINDINGS.md`
    block_census: Vec<u32>,
    /// How far a city reaches, and what extends the reach.
    ///
    /// The rewrite reads this on every step, so it enters the state hash
    /// beside the column it decides.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decisions D2 and D3. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    rules: ReachRules,
    /// The faction each tile's lease names, or nobody.
    ///
    /// A lease is one faction and one count, and no column of it is indexed
    /// by the faction. One tile names one faction, so the storage refuses a
    /// second one.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0153, a tile's lease follows the units that stand on it, decision D1. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
    lease_holder: Vec<Holder>,
    /// The count of each tile's lease.
    ///
    /// The type is signed, so that a subtraction below zero is representable
    /// and the overflow gate does not fire on the ordinary case. The rule
    /// then reads the sign and writes a value at or above zero.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0153, a tile's lease follows the units that stand on it, decision D1. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
    lease_count: Vec<i32>,
    /// The tiles whose lease count is above zero, in ascending tile order.
    ///
    /// The decay walks this rather than the world, so the cost of the decay
    /// follows the ground in use.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0153, a tile's lease follows the units that stand on it, decision D4. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
    leased: Vec<TileIdx>,
    /// How a lease rises, falls and claims.
    ///
    /// The lease pass reads these on every step, so they enter the state hash
    /// beside the columns they decide.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0153, a tile's lease follows the units that stand on it, decision D7. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
    lease_rules: LeaseRules,
}

impl Holding {
    /// Builds a holding in which nobody holds anything.
    #[must_use]
    pub fn new(layout: BlockLayout) -> Self {
        let tiles = layout.grid().tile_count() as usize;
        Self {
            layout,
            holders: vec![Holder::NOBODY; tiles],
            held: Vec::new(),
            census: [0; MASK_BITS as usize],
            block_masks: vec![FactionMask::EMPTY; layout.block_count() as usize],
            block_census: vec![0; layout.block_count() as usize * MASK_BITS as usize],
            rules: ReachRules::DEFAULT,
            lease_holder: vec![Holder::NOBODY; tiles],
            lease_count: vec![0; tiles],
            leased: Vec::new(),
            lease_rules: LeaseRules::DEFAULT,
        }
    }

    /// Returns how far a city reaches, and what extends the reach.
    #[must_use]
    pub const fn rules(&self) -> ReachRules {
        self.rules
    }

    /// Sets how far a city reaches, and what extends the reach.
    pub const fn set_rules(&mut self, rules: ReachRules) {
        self.rules = rules;
    }

    /// Returns how a lease rises, falls and claims.
    #[must_use]
    pub const fn lease_rules(&self) -> LeaseRules {
        self.lease_rules
    }

    /// Sets how a lease rises, falls and claims.
    pub const fn set_lease_rules(&mut self, rules: LeaseRules) {
        self.lease_rules = rules;
    }

    /// Returns the lease of one tile: the faction it names and the count.
    ///
    /// Returns `None` when the address lies outside the world.
    #[must_use]
    pub fn lease(&self, address: Axial) -> Option<(Holder, i32)> {
        let tile = self.layout.grid().index_of(address)?;
        let index = tile.0 as usize;
        Some((
            *self.lease_holder.get(index)?,
            *self.lease_count.get(index)?,
        ))
    }

    /// Returns the tiles whose lease count is above zero, in tile order.
    #[must_use]
    pub fn leased(&self) -> &[TileIdx] {
        &self.leased
    }

    /// Moves the lease of every tile that carries a unit, then decays the
    /// rest on a fixed schedule.
    ///
    /// The caller gives one entry for each tile that carries at least one
    /// unit: the tile, and the faction that has the most units on it. The
    /// list must be in ascending tile order and must name each tile once.
    /// The tie between two factions with equal counts belongs to the caller,
    /// because the caller holds the units.[^1]
    ///
    /// For each entry:
    ///
    /// - when the lease names the faction present, the count gains the raise
    ///   step, and it stops at the bound;
    /// - when the lease names another faction, the count loses the lower
    ///   step, and a result at or below zero gives the lease to the faction
    ///   present with the amount by which the result passed zero;
    /// - when the lease names nobody, the lease names the faction present and
    ///   the count becomes the raise step.
    ///
    /// **The rule reads no holder.** The holder of a tile therefore never
    /// decides the lease of that tile.[^1]
    ///
    /// The decay then runs on a fixed period and phase. It subtracts the
    /// decay step from every tile whose count is above zero. A count that
    /// reaches zero stops there, and the lease then names nobody. No
    /// convergence test and no clock ends the decay.[^2]
    ///
    /// The call runs on the calling thread. Its cost follows the tiles that
    /// carry a unit and the tiles whose count is above zero. It never follows
    /// the world.
    ///
    /// Returns how many tiles changed their lease faction.
    ///
    /// # References
    ///
    /// [^1]: ADR-0153, a tile's lease follows the units that stand on it, decisions D2 and D3. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
    /// [^2]: ADR-0153, a tile's lease follows the units that stand on it, decision D4. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
    pub fn advance_leases(&mut self, occupancy: &[(TileIdx, FactionId)], tick: u64) -> usize {
        let rules = self.lease_rules;
        let mut turned = 0usize;
        let mut touched: Vec<TileIdx> = Vec::with_capacity(occupancy.len());
        for (tile, faction) in occupancy {
            let index = tile.0 as usize;
            let Some(lease) = self.lease_holder.get_mut(index) else {
                continue;
            };
            let present = Holder::of(*faction);
            let count = &mut self.lease_count[index];
            if *lease == present {
                *count = count.saturating_add(rules.raise_step()).min(rules.bound());
            } else if lease.is_nobody() {
                *lease = present;
                *count = rules.raise_step().min(rules.bound());
                turned += 1;
            } else {
                let lowered = count.saturating_sub(rules.lower_step());
                if lowered <= 0 {
                    *lease = present;
                    // The amount by which the result passed zero carries
                    // into the new lease, so a change of hands drops
                    // nothing.
                    *count = lowered.saturating_neg().min(rules.bound());
                    turned += 1;
                } else {
                    *count = lowered;
                }
            }
            if self.lease_count[index] > 0 {
                touched.push(*tile);
            } else {
                self.lease_holder[index] = Holder::NOBODY;
            }
        }

        if rules.decays_on(tick) {
            for tile in &self.leased {
                let index = tile.0 as usize;
                let count = &mut self.lease_count[index];
                if *count <= 0 {
                    continue;
                }
                *count = count.saturating_sub(rules.decay_step()).max(0);
                if *count == 0 {
                    self.lease_holder[index] = Holder::NOBODY;
                }
            }
        }

        // The live list is the union of the list this call started with and
        // the tiles this call touched, less the tiles that reached zero. The
        // merge sorts by the tile index, which is a stable key, so the list
        // is the same whatever order the caller gave the occupancy in.[^1]
        //
        // [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
        touched.extend_from_slice(&self.leased);
        touched.sort_unstable();
        touched.dedup();
        touched.retain(|tile| self.lease_count[tile.0 as usize] > 0);
        self.leased = touched;
        turned
    }

    /// Returns the block partition the holding indexes by.
    #[must_use]
    pub const fn layout(&self) -> BlockLayout {
        self.layout
    }

    /// Returns the grid the holding covers.
    #[must_use]
    pub const fn grid(&self) -> Grid {
        self.layout.grid()
    }

    /// Returns the holder column.
    #[must_use]
    pub fn holders(&self) -> &[Holder] {
        &self.holders
    }

    /// Returns the holder of one tile.
    ///
    /// Returns `None` when the address lies outside the world.
    #[must_use]
    pub fn holder(&self, address: Axial) -> Option<Holder> {
        let tile = self.layout.grid().index_of(address)?;
        self.holders.get(tile.0 as usize).copied()
    }

    /// Returns the number of tiles one faction holds.
    ///
    /// The call reads a running total. It walks no tile.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    #[must_use]
    pub fn holding_of(&self, faction: FactionId) -> i64 {
        self.census
            .get(faction.0 as usize)
            .copied()
            .unwrap_or_default()
    }

    /// Returns the number of tiles that somebody holds.
    #[must_use]
    pub fn held_tiles(&self) -> i64 {
        self.held.len() as i64
    }

    /// Returns the tiles that somebody holds, in ascending tile order.
    #[must_use]
    pub fn held(&self) -> &[TileIdx] {
        &self.held
    }

    /// Returns the factions that hold ground in one block.
    #[must_use]
    pub fn block_mask(&self, block: u32) -> Option<FactionMask> {
        self.block_masks.get(block as usize).copied()
    }

    /// Returns every block in which one faction holds ground.
    ///
    /// The call reads one mask for each block and returns the blocks whose
    /// mask names the faction. It reads no tile.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D3. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    pub fn blocks_held_by(&self, faction: FactionId) -> impl Iterator<Item = u32> + '_ {
        self.block_masks
            .iter()
            .enumerate()
            .filter(move |(_, mask)| mask.contains(faction))
            .map(|(block, _)| block as u32)
    }

    /// Returns every tile that one faction holds, in ascending tile order.
    ///
    /// The call walks the list of held tiles, which grows with the holding
    /// and not with the world.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    pub fn tiles_held_by(&self, faction: FactionId) -> impl Iterator<Item = TileIdx> + '_ {
        let holder = Holder::of(faction);
        self.held
            .iter()
            .copied()
            .filter(move |tile| self.holders[tile.0 as usize] == holder)
    }

    /// Folds the holding into a state hash, in tile order.
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        let hash = hash.write(bytemuck::cast_slice(&self.holders));
        let hash = self
            .census
            .iter()
            .fold(hash, |hash, count| hash.write_u64(*count as u64));
        // The rewrite reads the three reach values on every step, so they are
        // inputs of the column above and they enter the hash with it.
        let hash = hash
            .write_u64(u64::from(self.rules.base()))
            .write_u64(u64::from(self.rules.upgrades_per_step()))
            .write_u64(u64::from(self.rules.cap()));
        // **The lease is simulated state, so both of its columns fold.** The
        // step reads the lease on every tick and writes it on every tick. A
        // value the step reads and that the hash does not fold lets two
        // worlds that differ in it hash the same and then diverge.[^1]
        //
        // [^1]: ADR-0153, a tile's lease follows the units that stand on it, decision D7. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
        let hash = hash.write(bytemuck::cast_slice(&self.lease_holder));
        let hash = hash.write(bytemuck::cast_slice(&self.lease_count));
        hash.write_u64(self.lease_rules.raise_step() as u64)
            .write_u64(self.lease_rules.lower_step() as u64)
            .write_u64(self.lease_rules.decay_step() as u64)
            .write_u64(u64::from(self.lease_rules.decay_period()))
            .write_u64(u64::from(self.lease_rules.decay_phase()))
            .write_u64(self.lease_rules.bound() as u64)
            .write_u64(self.lease_rules.claim_threshold() as u64)
    }

    /// Rewrites the holder column from the cities and returns the number of
    /// tiles that changed hands.
    ///
    /// A tile is held by the faction of the nearest city whose reach covers
    /// it. Two cities at one distance resolve by the lower settlement slot.
    /// A tile that no city reaches is held by nobody, and a tile whose ground
    /// admits no unit is held by nobody whatever reaches it.[^1]
    ///
    /// **The rule reads no previous holder.** A faction that owns no city
    /// therefore holds nothing after this call, and a unit standing on a tile
    /// gives its faction no claim on it.[^1]
    ///
    /// The pass computes one reach for each city first, which costs the
    /// cities. It then decides the tiles the cities reach, and the tiles the
    /// held list names, and no other tile. Each thread decides a contiguous
    /// run of the candidate list and writes its own slot, and the join reads
    /// the slots in slot order.[^2] Each tile reads the settlement table and
    /// never another tile, so no thread reads what another wrote.[^2]
    ///
    /// The write goes through the apply path that the land transfer uses, so
    /// the running total, the block masks and the held list repair.[^3]
    ///
    /// # Errors
    ///
    /// Returns an error when the settlement arena or the terrain describes
    /// another world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decisions D1 and D2. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    /// [^2]: ADR-0009, parallel stages write disjoint outputs, decisions D1, D2 and D3. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
    /// [^3]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    pub fn rewrite(
        &mut self,
        terrain: Terrain,
        settlements: &SettlementArena,
        upgrades: &UpgradeMap,
        threads: usize,
    ) -> Result<usize, BridgeError> {
        let grid = self.layout.grid();
        if grid != terrain.grid() || grid != settlements.grid() {
            return Err(BridgeError::GridMismatch);
        }
        let threads = threads.max(1);

        let cities = {
            let _span = stage::open(Stage::HoldingCities);
            self.cities(settlements, upgrades)
        };
        let candidates = {
            let _span = stage::open(Stage::HoldingCandidates);
            self.candidates(&cities)
        };
        if candidates.is_empty() {
            return Ok(0);
        }

        // Each thread fills its own slot, and the join reads the slots in
        // slot order. The chunks are contiguous runs of the candidate list,
        // which is in ascending tile order, so the joined result is in tile
        // order at every thread count.[^1] Nothing reads which thread
        // finished first.
        //
        // [^1]: ADR-0009, parallel stages write disjoint outputs, decisions D1, D2 and D3. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
        let chunk_len = candidates.len().div_ceil(threads).max(1);
        let slot_count = candidates.len().div_ceil(chunk_len);
        let mut slots: Slots<Vec<(TileIdx, Holder)>> = Slots::filled(slot_count, Vec::new())
            .expect("the candidate list is not empty, so it needs at least one slot");
        let holders = &self.holders[..];
        let cities = &cities[..];
        let lease = Lease {
            holders: &self.lease_holder[..],
            counts: &self.lease_count[..],
            claim_threshold: self.lease_rules.claim_threshold(),
        };
        let decide_span = stage::open(Stage::HoldingDecide);
        crate::parallel::fan_out_each(candidates.chunks(chunk_len).zip(slots.entries_mut()).map(
            move |(chunk, slot)| {
                move || {
                    let mut changes = Vec::new();
                    for tile in chunk {
                        let decided = decide(grid, terrain, cities, lease, *tile);
                        #[cfg(feature = "census-holding")]
                        census::record_decide(
                            cities.len() as u64,
                            false,
                            decided != holders[tile.0 as usize],
                        );
                        if decided != holders[tile.0 as usize] {
                            changes.push((*tile, decided));
                        }
                    }
                    *slot = changes;
                }
            },
        ));
        drop(decide_span);

        // The change list is one pair of a tile and the value that tile
        // takes. A second per-tile column joins the pair rather than opening
        // a second pass, so the write stays one scattered store for each tile
        // that moved.
        //
        // The join and the write are one stage. The join is what fixes the
        // order of the result, and the write is what the order is for.
        let _span = stage::open(Stage::HoldingApply);
        let changes = slots.combine(Vec::new(), |mut joined, slot| {
            joined.extend_from_slice(slot);
            joined
        });
        self.apply(&changes, threads);
        Ok(changes.len())
    }

    /// Returns one entry for each live city: its address, its faction and its
    /// reach.
    ///
    /// The list is in settlement slot order, which is the order the tie rule
    /// of the decision reads.[^1]
    ///
    /// **The reach counts the finished upgrades on the ground the city held
    /// at the end of the previous step.** The holder column names a faction
    /// and not a city, so a finished upgrade on the ground of one faction
    /// counts for the nearest city of that faction, and a tie between two
    /// such cities goes to the lower slot. The count therefore reads the
    /// column as the previous step left it, and the rule has no
    /// recursion.[^1] An upgrade under construction counts for nothing.
    ///
    /// The cost is the finished upgrade entries multiplied by the live
    /// cities. It reads no tile of the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D2. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    #[must_use]
    pub fn cities(&self, settlements: &SettlementArena, upgrades: &UpgradeMap) -> Vec<City> {
        let grid = self.layout.grid();
        let tiles = settlements.tile_column();
        let factions = settlements.faction_column();
        let live = settlements.live_column();
        let mut finished = vec![0u32; live.len()];
        for site in upgrades.sites() {
            if !site.is_complete() {
                continue;
            }
            let Some(holder) = self
                .holders
                .get(site.tile.0 as usize)
                .and_then(|holder| holder.faction())
            else {
                continue;
            };
            let Some(address) = grid.address_of(site.tile) else {
                continue;
            };
            let mut best: Option<(u32, usize)> = None;
            for (slot, standing) in live.iter().enumerate() {
                if *standing != 1 || factions[slot] != holder {
                    continue;
                }
                let Some(seat) = grid.address_of(tiles[slot]) else {
                    continue;
                };
                let distance = address.distance(seat);
                if best.is_none_or(|(nearest, _)| distance < nearest) {
                    best = Some((distance, slot));
                }
            }
            if let Some((_, slot)) = best {
                finished[slot] += 1;
            }
        }

        live.iter()
            .enumerate()
            .filter(|(_, standing)| **standing == 1)
            .filter_map(|(slot, _)| {
                Some(City {
                    slot: slot as u32,
                    address: grid.address_of(tiles[slot])?,
                    faction: factions[slot],
                    reach: self.rules.reach_of(finished[slot]),
                    finished: finished[slot],
                })
            })
            .collect()
    }

    /// Returns the tiles this rewrite can change, in ascending tile order.
    ///
    /// A tile changes only when a city reaches it, or when somebody holds it
    /// now. Every other tile is held by nobody before the rewrite and after
    /// it, so the pass never visits it. The cost is therefore the cities
    /// multiplied by the area of the largest reach, plus the ground held, and
    /// never the world.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D3. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    fn candidates(&self, cities: &[City]) -> Vec<TileIdx> {
        let grid = self.layout.grid();
        let mut candidates: Vec<TileIdx> = Vec::with_capacity(self.held.len() + self.leased.len());
        candidates.extend_from_slice(&self.held);
        // A tile whose lease is live can change holder even when no city
        // reaches it and nobody holds it now, so the leased list joins the
        // candidates.[^2]
        //
        // [^2]: ADR-0153, a tile's lease follows the units that stand on it, decision D5. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
        candidates.extend_from_slice(&self.leased);
        for city in cities {
            let reach = city.reach as i32;
            for dq in -reach..=reach {
                let low = (-reach).max(-dq - reach);
                let high = reach.min(-dq + reach);
                for dr in low..=high {
                    let address = Axial::new(city.address.q + dq, city.address.r + dr);
                    if let Some(tile) = grid.index_of(address) {
                        candidates.push(tile);
                    }
                }
            }
        }
        candidates.sort_unstable();
        candidates.dedup();
        candidates
    }

    /// Gives a set of tiles to one holder, and repairs the derived parts.
    ///
    /// **This is the path a land contract takes.** No unit carries a tile, so
    /// a land set changes holder by agreement when the other side of the
    /// contract is delivered in full.[^1] The tiles are written in ascending
    /// tile index, whatever order the caller gave them in, so the write names
    /// no thread and depends on no caller order.[^2] A tile outside the world
    /// is skipped.
    ///
    /// The call runs on the calling thread and takes the thread count only
    /// for the merge that rebuilds the held list.
    ///
    /// # References
    ///
    /// [^1]: ADR-0147, a contract consideration is a tagged kind, decision D3. `docs/adrs/accepted/adr-0147-a-contract-consideration-is-a-tagged-kind.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub fn transfer(&mut self, tiles: &[TileIdx], to: Holder, threads: usize) {
        let ceiling = self.holders.len();
        let mut changes: Vec<(TileIdx, Holder)> = tiles
            .iter()
            .copied()
            .filter(|tile| (tile.0 as usize) < ceiling)
            .map(|tile| (tile, to))
            .collect();
        changes.sort_unstable_by_key(|(tile, _)| *tile);
        changes.dedup();
        self.apply(&changes, threads);
    }

    /// Releases every claim one faction has on the ground, and reports how
    /// many tiles it held.
    ///
    /// **A faction claims ground two ways, and a release must end both.** The
    /// holder column says who holds a tile now. The lease column says who has
    /// been using it, and a lease at the claim threshold outranks the reach
    /// of every city.[^1] A release that cleared only the holder would give
    /// the tile back on the next spread, from a lease that nobody can raise
    /// any more.
    ///
    /// The tiles are written in ascending tile index, so the write names no
    /// thread and depends on no caller order.[^2] The call runs on the
    /// calling thread and takes the thread count only for the merge that
    /// rebuilds the held list.
    ///
    /// # References
    ///
    /// [^1]: ADR-0153, a tile's lease follows the units that stand on it, decision D5. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub fn release(&mut self, faction: FactionId, threads: usize) -> u64 {
        let holder = Holder::of(faction);
        let held: Vec<TileIdx> = self
            .held
            .iter()
            .copied()
            .filter(|tile| self.holders[tile.0 as usize] == holder)
            .collect();
        let released = held.len() as u64;
        // The lease goes first. The holder write below rebuilds the derived
        // parts, and a lease left behind would take the tile back on the next
        // spread.
        let mut leased: Vec<TileIdx> = Vec::new();
        for tile in &self.leased {
            let index = tile.0 as usize;
            if self.lease_holder[index] == holder {
                self.lease_holder[index] = Holder::NOBODY;
                self.lease_count[index] = 0;
            } else {
                leased.push(*tile);
            }
        }
        self.leased = leased;
        if !held.is_empty() {
            self.transfer(&held, Holder::NOBODY, threads);
        }
        released
    }

    /// Writes the decided changes and repairs the three derived parts.
    ///
    /// The write is one scattered store for each change, and it runs on the
    /// calling thread because it reads the holder it is about to overwrite.
    ///
    /// **One repair follows the change count and the other follows the list
    /// it repairs.** The mask of a block is no longer derived by reading the
    /// block: the holding counts the tiles each faction holds in each block,
    /// so a mask gains a bit when a count leaves zero and loses one when a
    /// count reaches zero, and a moved tile touches two counters.[^1] The held
    /// list is still rebuilt by a merge that reads all of it, and that merge
    /// takes a thread count.[^2]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-307. `docs/FINDINGS.md`
    /// [^2]: ADR-0009, parallel stages write disjoint outputs, decisions D1, D2 and D3. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
    fn apply(&mut self, changes: &[(TileIdx, Holder)], threads: usize) {
        let threads = threads.max(1);
        let mut moved: Vec<TileIdx> = Vec::with_capacity(changes.len());
        for (tile, holder) in changes {
            let previous = self.holders[tile.0 as usize];
            if previous == *holder {
                continue;
            }
            let block = self
                .layout
                .key_of(*tile)
                .map(|key| self.layout.block_of_key(key) as usize);
            if let Some(faction) = previous.faction() {
                self.census[faction.0 as usize] -= 1;
                if let Some(block) = block {
                    self.leave_block(block, faction);
                }
            }
            if let Some(faction) = holder.faction() {
                self.census[faction.0 as usize] += 1;
                if let Some(block) = block {
                    self.enter_block(block, faction);
                }
            }
            self.holders[tile.0 as usize] = *holder;
            moved.push(*tile);
        }
        if moved.is_empty() {
            return;
        }
        moved.sort_unstable();
        moved.dedup();
        #[cfg(feature = "census-holding")]
        let moved_count = moved.len() as u64;

        self.rebuild_held(&moved, threads);

        // **No block is read again.** A block loses a faction bit exactly when
        // that faction's count in the block reaches zero, and the counts above
        // saw that as each tile moved. Rereading a block cost every tile of
        // it, and a frame at the target scale dirties most blocks.[^1]
        //
        // [^1]: Findings register, FND-307. `docs/FINDINGS.md`
        #[cfg(feature = "census-holding")]
        census::record(moved_count, 0, self.held.len() as u64);
    }

    /// Records that one faction took one more tile of one block.
    ///
    /// The mask gains the bit when the count leaves zero, and at no other
    /// time.
    fn enter_block(&mut self, block: usize, faction: FactionId) {
        let at = block * MASK_BITS as usize + faction.0 as usize;
        let Some(count) = self.block_census.get_mut(at) else {
            return;
        };
        *count += 1;
        if *count == 1 {
            if let Some(mask) = self.block_masks.get_mut(block) {
                *mask = mask.with(faction);
            }
        }
    }

    /// Records that one faction gave up one tile of one block.
    ///
    /// The mask loses the bit when the count reaches zero, and at no other
    /// time.
    fn leave_block(&mut self, block: usize, faction: FactionId) {
        let at = block * MASK_BITS as usize + faction.0 as usize;
        let Some(count) = self.block_census.get_mut(at) else {
            return;
        };
        *count = count.saturating_sub(1);
        if *count == 0 {
            if let Some(mask) = self.block_masks.get_mut(block) {
                *mask = mask.without(faction);
            }
        }
    }

    /// Rebuilds the held list from the old list and the tiles that changed.
    ///
    /// The held list stays in ascending tile order. A change either adds a
    /// tile to it or takes one out, so the merge reads the old list once and
    /// the changed tiles once.
    ///
    /// **The tile space is cut at values taken from the old list, and both
    /// lists are cut at the same values.** A thread therefore merges one band
    /// of tiles, writes its own buffer, and no two threads produce one tile.
    /// The join reads the buffers in band order, which is ascending tile
    /// order, and never in the order a thread finished.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0009, parallel stages write disjoint outputs, decisions D1, D2 and D3. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
    fn rebuild_held(&mut self, moved: &[TileIdx], threads: usize) {
        let held = std::mem::take(&mut self.held);
        let chunk_len = held.len().div_ceil(threads).max(1);

        // The cut points are tile values and not positions, so the two lists
        // are cut at the same places and every tile falls in exactly one band.
        let mut cuts: Vec<(usize, usize)> = vec![(0, 0)];
        let mut at = chunk_len;
        while at < held.len() {
            let pivot = held[at];
            cuts.push((at, moved.partition_point(|tile| *tile < pivot)));
            at += chunk_len;
        }
        cuts.push((held.len(), moved.len()));

        let held = &held[..];
        let holders = &self.holders[..];
        let bands = cuts.len() - 1;
        let mut slots: Slots<Vec<TileIdx>> = Slots::filled(bands, Vec::new())
            .expect("the cut list always holds a first and a last entry");
        crate::parallel::fan_out_each(slots.entries_mut().iter_mut().enumerate().map(
            move |(band, slot)| {
                let (held_from, moved_from) = cuts[band];
                let (held_to, moved_to) = cuts[band + 1];
                let old = &held[held_from..held_to];
                let changed = &moved[moved_from..moved_to];
                move || {
                    let mut joined: Vec<TileIdx> = Vec::with_capacity(old.len() + changed.len());
                    let mut cursor = 0usize;
                    for tile in changed {
                        while cursor < old.len() && old[cursor] < *tile {
                            joined.push(old[cursor]);
                            cursor += 1;
                        }
                        if cursor < old.len() && old[cursor] == *tile {
                            cursor += 1;
                        }
                        if !holders[tile.0 as usize].is_nobody() {
                            joined.push(*tile);
                        }
                    }
                    joined.extend_from_slice(&old[cursor..]);
                    *slot = joined;
                }
            },
        ));

        let total: usize = slots.entries().iter().map(Vec::len).sum();
        let mut rebuilt: Vec<TileIdx> = Vec::with_capacity(total);
        for band in slots.entries() {
            rebuilt.extend_from_slice(band);
        }
        self.held = rebuilt;
    }

    /// Reports whether the holding holds its invariants.
    ///
    /// The holder column is the truth. The held list, the census and the
    /// block masks are three further declarations of the same fact, so this
    /// check derives each of them from the column and compares.[^1] It also
    /// proves the two properties the record states: no tile names a faction
    /// the world does not have, and no faction holds open water.[^2]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    /// [^2]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D5. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    #[must_use]
    pub fn check_invariants(&self, terrain: Terrain, faction_ceiling: u16) -> bool {
        let grid = self.layout.grid();
        if grid != terrain.grid() {
            return false;
        }
        if self.holders.len() != grid.tile_count() as usize {
            return false;
        }
        if self.block_masks.len() != self.layout.block_count() as usize {
            return false;
        }
        if self.block_census.len() != self.layout.block_count() as usize * MASK_BITS as usize {
            return false;
        }

        let mut census = [0i64; MASK_BITS as usize];
        let mut held: Vec<TileIdx> = Vec::new();
        let mut masks = vec![FactionMask::EMPTY; self.block_masks.len()];
        // The per-block count is a second declaration of what the holder
        // column says, so it is derived here and compared like the rest.[^1]
        let mut counts = vec![0u32; self.block_census.len()];
        for (index, holder) in self.holders.iter().enumerate() {
            let tile = TileIdx(index as u32);
            let Some(faction) = holder.faction() else {
                continue;
            };
            if faction.0 >= faction_ceiling || faction.0 >= FACTION_CEILING {
                return false;
            }
            // No faction holds ground that admits no unit. The rule refuses
            // such a tile, and this is what fails when a later path forgets
            // to.
            let Some(address) = grid.address_of(tile) else {
                return false;
            };
            if !terrain.kind(address).is_some_and(TileKind::is_passable) {
                return false;
            }
            census[faction.0 as usize] += 1;
            held.push(tile);
            let Some(key) = self.layout.key_of(tile) else {
                return false;
            };
            let block = self.layout.block_of_key(key) as usize;
            masks[block] = masks[block].with(faction);
            let at = block * MASK_BITS as usize + faction.0 as usize;
            if at >= counts.len() {
                return false;
            }
            counts[at] += 1;
        }

        census == self.census
            && held == self.held
            && masks == self.block_masks
            && counts == self.block_census
    }
}

/// One live city, as the decision reads it.
///
/// The list of these is in settlement slot order, and the tie rule of the
/// decision takes the first entry at the nearest distance. The slot is
/// carried so that a caller can name the city a reach belongs to.[^1]
///
/// # References
///
/// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D1. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct City {
    /// The settlement slot of the city.
    pub slot: u32,
    /// The address the city stands on.
    pub address: Axial,
    /// The faction that owns the city.
    pub faction: FactionId,
    /// How many hex steps the city reaches.
    pub reach: u32,
    /// How many finished upgrades stand on the ground this city is nearest
    /// to, of the faction that holds that ground.
    ///
    /// **The reach rules turn this count into the reach, and the count is
    /// what a caller needs in order to read the headroom.** The reach alone
    /// cannot say how far a city stands from its next step, because the reach
    /// stops at the bound and the count does not.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D2. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    pub finished: u32,
}

/// What the holder decision reads of the lease.
///
/// The two columns and the threshold travel together, because the test of the
/// decision reads all three.[^1]
///
/// # References
///
/// [^1]: ADR-0153, a tile's lease follows the units that stand on it, decision D5. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
#[derive(Clone, Copy)]
struct Lease<'a> {
    holders: &'a [Holder],
    counts: &'a [i32],
    claim_threshold: i32,
}

/// Returns the holder of one tile, decided from the lease and the cities.
///
/// A lease at or above the claim threshold wins. Otherwise the nearest city
/// within reach wins. Two cities at one distance resolve by
/// the lower settlement slot, and the list is in slot order, so the strict
/// comparison below keeps the first of them. A tile that no city reaches is
/// held by nobody, and ground that admits no unit is held by nobody whatever
/// reaches it.[^1]
///
/// The call reads no holder, so it is a pure function of the tile, the
/// terrain and the city list. Two threads that decide two tiles therefore
/// share nothing.[^2]
///
/// **This function is the one statement of the holder rule, and the parallel
/// pass calls it for every candidate tile.** A second source of a claim adds
/// its argument here and its comparison here. A rule written inline in the
/// loop would have to be found and moved first.
///
/// # References
///
/// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D1. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
/// [^2]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
fn decide(
    grid: Grid,
    terrain: Terrain,
    cities: &[City],
    lease: Lease<'_>,
    tile: TileIdx,
) -> Holder {
    let Some(address) = grid.address_of(tile) else {
        return Holder::NOBODY;
    };
    // The ground is read first, because it refuses every city at once. No
    // faction holds ground that admits no unit, and the invariant check names
    // a tile that breaks it.
    if !terrain.kind(address).is_some_and(TileKind::is_passable) {
        return Holder::NOBODY;
    }
    // **The lease is tested before the city.** A lease at or above the claim
    // threshold holds the tile, whatever city reaches it. That is what lets a
    // faction take ground by using it, and it is what lets a seat change
    // hands.[^3]
    //
    // [^3]: ADR-0153, a tile's lease follows the units that stand on it, decision D5. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
    let index = tile.0 as usize;
    if let (Some(holder), Some(count)) = (lease.holders.get(index), lease.counts.get(index)) {
        if !holder.is_nobody() && *count >= lease.claim_threshold {
            return *holder;
        }
    }
    let mut best: Option<(u32, FactionId)> = None;
    for city in cities {
        let distance = address.distance(city.address);
        if distance > city.reach {
            continue;
        }
        if best.is_none_or(|(nearest, _)| distance < nearest) {
            best = Some((distance, city.faction));
        }
    }
    match best {
        Some((_, faction)) => Holder::of(faction),
        None => Holder::NOBODY,
    }
}
