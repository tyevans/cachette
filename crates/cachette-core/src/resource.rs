//! The resource field, and what has been taken from it.
//!
//! A tile holds a stock of each resource kind. The stock a tile started with
//! is a pure function of the world seed and the tile address, in the same way
//! the ground is.[^1] The engine stores no map of stocks, so the memory cost
//! of the field is the size of the seed and the extent, at any tile count.
//!
//! What a unit has taken is a fact, and a fact is stored. The ledger below
//! holds one entry for each tile and kind that somebody gathered from. A world
//! in which nothing was gathered holds no entry.[^2]
//!
//! Every amount here is an exact integer, so a sum over tiles gives the same
//! answer in any order.[^3]
//!
//! Every draw comes from the counter-based generator, keyed on the tuple of
//! system, frame, entity and draw index.[^4] The frame slot holds a constant,
//! because the stock a tile started with does not change with time. The entity
//! slot holds the tile address. The draw slot holds the kind and the question,
//! so the presence draw and the size draw never correlate.
//!
//! # References
//!
//! [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D1. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
//! [^2]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
//! [^3]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^4]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`

use bytemuck::{Pod, Zeroable};

use crate::hash::StateHash;
use crate::hex::{Axial, Grid};
use crate::rng;
use crate::terrain::{Terrain, TileKind, KIND_COUNT as TERRAIN_KIND_COUNT};
use crate::types::{Accum, Tick, TileIdx};

/// The frame that every stock draw is keyed on.
///
/// The stock a tile started with does not change with time, so the frame slot
/// holds one constant. The slot stays in the key because the key shape is
/// fixed by the record.[^1]
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
pub const RESOURCE_FRAME: u64 = 0;

/// The number of resource kinds.
pub const RESOURCE_KIND_COUNT: usize = 3;

/// The draw index of the first presence draw.
const DRAW_PRESENCE: u32 = 0;

/// The draw index of the first size draw.
///
/// The gap between the two bases is wider than the kind count, so no presence
/// draw and no size draw ever share a draw index. Two draws that share a key
/// are the same draw.
const DRAW_SIZE: u32 = 16;

/// The denominator of a presence chance.
///
/// A chance is stated in sixteenths, because an exact integer comparison is
/// the only comparison this project makes.[^1]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
const CHANCE_DENOMINATOR: u64 = 16;

/// A kind of resource.
///
/// The catalogue is three kinds, and it is a table rather than a set of
/// verbs.[^1] A kind is an index into the two tables below, in the same way a
/// terrain kind is an index into the capacity table.
///
/// # References
///
/// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D3. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ResourceKind {
    /// What a unit eats. Open ground carries the most of it.
    #[default]
    Food = 0,
    /// What grows on a wooded tile.
    Wood = 1,
    /// What high ground carries.
    Stone = 2,
}

impl ResourceKind {
    /// Every kind, in the order of its number.
    pub const ALL: [Self; RESOURCE_KIND_COUNT] = [Self::Food, Self::Wood, Self::Stone];

    /// Returns the kind as a small integer.
    ///
    /// The numbering is stable, because a state hash, an event and a sort key
    /// all read it.
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    /// Returns the kind that a small integer names.
    ///
    /// Returns `None` when the integer names no kind. A caller that reads a
    /// kind out of an event takes this path.
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Food),
            1 => Some(Self::Wood),
            2 => Some(Self::Stone),
            _ => None,
        }
    }

    /// Returns the kind as an index into a table.
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// An amount of one resource.
///
/// The amount is an exact whole number. It is never a fraction, because a
/// fraction of a unit of stone is not a thing the world holds, and because an
/// exact integer sums the same in any order.[^1]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct Amount(pub u32);

impl Amount {
    /// The amount zero.
    pub const ZERO: Self = Self(0);

    /// Widens the amount into an accumulator.
    #[must_use]
    pub const fn to_accum(self) -> Accum {
        Accum(self.0 as i64)
    }
}

/// The largest stock that one tile of each ground holds, for each kind.
///
/// The ground decides what a tile carries. Water carries nothing at all, a
/// wooded tile carries the most wood, and a mountain carries the most
/// stone.[^1] The table is content, and it lives beside the ground table until
/// a content pipeline exists.
///
/// Every ceiling is below the demand that one full tile of gatherers makes in
/// one tick. A deposit is therefore something units run out of, which is the
/// case the resolve exists for.
///
/// The row order is the terrain kind numbering, and the column order is the
/// resource kind numbering.
///
/// # References
///
/// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D2. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
const CEILING: [[u32; RESOURCE_KIND_COUNT]; TERRAIN_KIND_COUNT] = [
    // Water. Open water holds nothing that a unit could take, and no unit
    // stands on it.
    [0, 0, 0],
    // Plain.
    [12, 0, 2],
    // Forest.
    [6, 16, 0],
    // Hill.
    [3, 4, 12],
    // Mountain.
    [0, 0, 16],
];

/// The chance in sixteenths that a tile of each ground carries a deposit.
///
/// A resource is not spread evenly. Most tiles carry nothing of most kinds,
/// and the ground decides how often a tile carries something.[^1]
///
/// # References
///
/// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D2. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
const PRESENCE: [[u64; RESOURCE_KIND_COUNT]; TERRAIN_KIND_COUNT] = [
    // Water.
    [0, 0, 0],
    // Plain.
    [6, 0, 2],
    // Forest.
    [4, 10, 0],
    // Hill.
    [3, 4, 6],
    // Mountain.
    [0, 0, 8],
];

/// The stock that one world started with.
///
/// The type holds the ground of the world, and nothing else. It allocates
/// nothing, whatever the tile count.[^1]
///
/// # References
///
/// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D1. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceField {
    terrain: Terrain,
}

impl ResourceField {
    /// Builds the field over the ground of a world.
    #[must_use]
    pub const fn new(terrain: Terrain) -> Self {
        Self { terrain }
    }

    /// Returns the ground that the field reads.
    #[must_use]
    pub const fn terrain(self) -> Terrain {
        self.terrain
    }

    /// Returns the extent of the field.
    #[must_use]
    pub const fn grid(self) -> Grid {
        self.terrain.grid()
    }

    /// Returns the stock that one tile started with.
    ///
    /// Returns `None` when the address lies outside the world. The world does
    /// not wrap, so an address outside the extent names no tile.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D2. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
    #[must_use]
    pub fn original(self, address: Axial, kind: ResourceKind) -> Option<Amount> {
        let ground = self.terrain.kind(address)?;
        Some(generate(self.terrain.seed(), address, ground, kind))
    }

    /// Returns the stock that one tile started with, from the ground the
    /// caller already holds.
    ///
    /// The stock of a tile is a function of the seed, the address and the
    /// ground.[^1] A caller that has already read the ground of the address
    /// holds the third argument, and this reader takes it rather than
    /// generating it a second time. The ground is generated from a noise
    /// field, so the second generation is the expensive one.[^2]
    ///
    /// **The answer follows the ground the caller gives, and the reader
    /// checks nothing.** A caller that gives the ground of a different
    /// address gets the stock of a tile that does not exist. Give the ground
    /// of this address, read from the same terrain.
    ///
    /// The address still names the tile, so two tiles of one ground hold
    /// different stocks and the field keeps its texture.
    ///
    /// Returns `None` when the address lies outside the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D1. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    /// [^2]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
    #[must_use]
    pub fn original_of_ground(
        self,
        address: Axial,
        ground: TileKind,
        kind: ResourceKind,
    ) -> Option<Amount> {
        if !self.grid().contains(address) {
            return None;
        }
        Some(generate(self.terrain.seed(), address, ground, kind))
    }

    /// Returns the stock that one tile started with, by index.
    #[must_use]
    pub fn original_at(self, index: TileIdx, kind: ResourceKind) -> Option<Amount> {
        let address = self.grid().address_of(index)?;
        self.original(address, kind)
    }

    /// Folds the whole field into the state hash.
    ///
    /// The field is part of the world, and the record hashes the whole world
    /// each frame.[^1] Hashing the seed alone would not meet that, because the
    /// seed is the input of the generator and not its output. A change to a
    /// ceiling or to a chance moves every tile of every world, and a hash over
    /// the inputs would not move.
    ///
    /// The tiles enter in index order, which is fixed and does not depend on
    /// how a caller visited them.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn hash_into(self, hash: StateHash) -> StateHash {
        let mut running = hash;
        let grid = self.grid();
        let mut index = 0;
        while index < grid.tile_count() {
            let Some(address) = grid.address_of(TileIdx(index)) else {
                break;
            };
            for kind in ResourceKind::ALL {
                let amount = self.original(address, kind).unwrap_or(Amount::ZERO);
                running = running.write(&amount.0.to_le_bytes());
            }
            index += 1;
        }
        running
    }
}

/// Generates the stock of one tile and kind.
///
/// The function reads the seed, the address and the ground. It reads nothing
/// else, so two callers that visit the world in different orders, on different
/// thread counts, read the same world.[^1]
///
/// A tile carries a deposit when its presence draw falls below the chance of
/// its ground. A deposit that exists holds at least one, because a deposit of
/// nothing is the same as no deposit, and two ways to say one thing is the
/// defect shape this project keeps meeting.[^2]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
/// [^2]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
fn generate(seed: u64, address: Axial, ground: TileKind, kind: ResourceKind) -> Amount {
    let row = ground.to_u8() as usize;
    let column = kind.index();
    let ceiling = CEILING[row][column];
    let chance = PRESENCE[row][column];
    if ceiling == 0 || chance == 0 {
        return Amount::ZERO;
    }
    let node = address_key(address);
    let index = kind.to_u8() as u32;
    let present = rng::draw_below(
        seed,
        rng::SYSTEM_RESOURCE,
        RESOURCE_FRAME,
        node,
        DRAW_PRESENCE + index,
        CHANCE_DENOMINATOR,
    );
    if present >= chance {
        return Amount::ZERO;
    }
    let size = rng::draw_below(
        seed,
        rng::SYSTEM_RESOURCE,
        RESOURCE_FRAME,
        node,
        DRAW_SIZE + index,
        u64::from(ceiling),
    );
    Amount(size as u32 + 1)
}

/// Packs a tile address into the entity slot of the draw key.
///
/// Both components reach the key. A key that dropped one would give a field
/// that varies along one axis and is constant along the other, and every
/// determinism test would still pass, because the defect repeats.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2. `.claude/rules/testing.md`
#[cfg(not(feature = "probe-nondeterminism"))]
const fn address_key(address: Axial) -> u64 {
    ((address.q as u32 as u64) << 32) | (address.r as u32 as u64)
}

/// The perturbed packing. It drops the row component of the address.
///
/// This is the defect that the testing rule warns about: the field it builds
/// is identical on every run and at every thread count, so both determinism
/// tests pass over it. Only a test of the key itself sees it.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2. `.claude/rules/testing.md`
#[cfg(feature = "probe-nondeterminism")]
const fn address_key(address: Axial) -> u64 {
    (address.q as u32 as u64) << 32
}

/// Packs a tile and a kind into one ledger key.
///
/// The key is the tile index shifted up by two bits, with the kind in the low
/// bits. The kind count is three, so two bits hold it. The key rises with the
/// tile index, so a run of keys in ascending tile order is a run of keys in
/// ascending key order.
#[must_use]
pub const fn ledger_key(tile: TileIdx, kind: ResourceKind) -> u64 {
    ((tile.0 as u64) << 2) | (kind.to_u8() as u64)
}

/// The number of ticks in one simulated day.
///
/// One tick is a fixed span of simulated time, and the register holds that
/// span.[^1] A period stated in ticks alone would go stale if the span moved,
/// so every period below is stated in simulated days and converted here.
///
/// # References
///
/// [^1]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
pub const TICKS_IN_A_SIMULATED_DAY: u32 = 600;

/// How long each kind takes to regain one unit of stock, in simulated days.
///
/// A kind that holds `None` does not recover at all. Stone holds `None`,
/// because stone is not alive and does not grow back.[^1]
///
/// **This is the only declaration of the recovery period.** A caller replaces
/// the whole rule set rather than one value, so no second site holds a period
/// and no check is needed to keep two copies in step.[^2]
///
/// # References
///
/// [^1]: Decisions register, DEC-049. `docs/DECISIONS.md`
/// [^2]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
const RECOVERY_DAYS: [Option<u32>; RESOURCE_KIND_COUNT] = [Some(1), Some(4), None];

/// How fast each kind of deposit recovers.
///
/// The rules are a parameter of the world, not a constant of a kernel. The
/// value of a period is a judgement that the project owner may reverse, so the
/// engine holds it where a caller can replace it.[^1]
///
/// A period is the simulated time in which one depleted deposit regains one
/// unit of stock. A kind that states no period does not recover.
///
/// # References
///
/// [^1]: Decisions register, DEC-049 and DEC-050. `docs/DECISIONS.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecoveryRules {
    periods: [Option<u32>; RESOURCE_KIND_COUNT],
}

impl RecoveryRules {
    /// The rules that the content table above states.
    pub const DEFAULT: Self = {
        let mut periods = [None; RESOURCE_KIND_COUNT];
        let mut index = 0;
        while index < RESOURCE_KIND_COUNT {
            periods[index] = match RECOVERY_DAYS[index] {
                Some(days) => Some(days * TICKS_IN_A_SIMULATED_DAY),
                None => None,
            };
            index += 1;
        }
        Self { periods }
    };

    /// The rules under which no kind recovers.
    pub const NONE: Self = Self {
        periods: [None; RESOURCE_KIND_COUNT],
    };

    /// Builds a rule set from a period in ticks for each kind.
    ///
    /// Returns `None` when a period is zero. A period of zero returns the
    /// whole take in one tick, which is a second way to say that a deposit was
    /// never depleted, and two ways to say one thing is the defect shape this
    /// project keeps meeting.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub const fn from_ticks(periods: [Option<u32>; RESOURCE_KIND_COUNT]) -> Option<Self> {
        let mut index = 0;
        while index < RESOURCE_KIND_COUNT {
            if let Some(period) = periods[index] {
                if period == 0 {
                    return None;
                }
            }
            index += 1;
        }
        Some(Self { periods })
    }

    /// Returns the period of one kind, in ticks.
    ///
    /// Returns `None` when the kind does not recover.
    #[must_use]
    pub const fn period_of(self, kind: ResourceKind) -> Option<u32> {
        self.periods[kind.index()]
    }
}

impl Default for RecoveryRules {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// One entry of the depletion ledger.
///
/// The entry holds what is still owed to one deposit, and the tick that the
/// owed amount was last brought up to date at. The two fields are one fact
/// together: an amount without its anchor cannot be aged, and an anchor
/// without an amount describes nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LedgerEntry {
    /// The tile and kind that the entry describes.
    pub key: u64,
    /// What is still missing from the deposit.
    pub taken: u32,
    /// The tick that the amount above was brought up to date at.
    pub anchor: Tick,
}

/// Ages one entry forward to a tick.
///
/// Recovery is not growth of an amount. It is the ageing away of a stored
/// take: the deposit holds what the generator gave it, less what the ledger
/// still says was taken, so a smaller stored take is a fuller deposit.[^1]
///
/// The arithmetic is whole numbers only. The elapsed ticks divide by the
/// period, and the anchor moves forward by the whole periods that were spent.
/// The remainder therefore survives, so the same total of ticks recovers the
/// same amount however many calls it arrived in. A rule that dropped the
/// remainder would recover nothing at all when it ran on every tick and the
/// period was longer than one tick.[^2]
///
/// An entry that reaches nothing owed restarts its clock at the tick, because
/// the ticks it did not need must not carry into the next take.
///
/// # References
///
/// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
/// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
#[must_use]
fn aged(entry: LedgerEntry, tick: Tick, period: Option<u32>) -> LedgerEntry {
    let Some(period) = period else {
        return entry;
    };
    if entry.taken == 0 {
        return LedgerEntry {
            anchor: tick,
            ..entry
        };
    }
    let elapsed = tick.0.saturating_sub(entry.anchor.0);
    let whole = elapsed / u64::from(period);
    if whole == 0 {
        return entry;
    }
    let recovered = whole.min(u64::from(entry.taken)) as u32;
    let taken = entry.taken - recovered;
    let anchor = if taken == 0 {
        tick
    } else {
        Tick(entry.anchor.0 + u64::from(recovered) * u64::from(period))
    };
    LedgerEntry {
        taken,
        anchor,
        ..entry
    }
}

/// Returns the recovery period of the kind that a ledger key names.
///
/// Returns `None` when the kind does not recover, and when the key names no
/// kind. A key that named no kind would be a broken ledger, and the
/// conservation check is what reports that.
#[must_use]
fn period_of_key(rules: RecoveryRules, key: u64) -> Option<u32> {
    match ResourceKind::from_u8((key & 0b11) as u8) {
        Some(kind) => rules.period_of(kind),
        None => None,
    }
}

/// What has been taken from each tile and kind.
///
/// The ledger holds one entry for each tile and kind that somebody gathered
/// from, and it holds nothing else. A world in which nothing was gathered
/// holds no entry, so the memory cost follows the gathering and not the size
/// of the world.[^1]
///
/// The entries are held sorted by key, so a lookup is a binary search and the
/// order never depends on how the entries were gathered.[^2]
///
/// An entry is merged in ascending runs, never inserted one at a time.
/// Inserting into the middle of a vector moves every later entry, which is
/// quadratic in the number of tiles a frame touches, and the target scale is a
/// million units.
///
/// # References
///
/// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
/// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DepletionLedger {
    entries: Vec<LedgerEntry>,
    scratch: Vec<LedgerEntry>,
    rules: RecoveryRules,
    visits: u64,
    returned: [i64; RESOURCE_KIND_COUNT],
}

impl DepletionLedger {
    /// Builds an empty ledger.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
            scratch: Vec::new(),
            rules: RecoveryRules::DEFAULT,
            visits: 0,
            returned: [0; RESOURCE_KIND_COUNT],
        }
    }

    /// Returns the total that recovery has given back, for one kind.
    ///
    /// The world took a resource out of the tiles and recovery puts some of it
    /// back. A conservation check reads both terms, because the stored take
    /// alone no longer balances what the units hold.[^1]
    ///
    /// The accumulator is 64 bits wide and every term is a whole number, so
    /// the total is the same in any order.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D5. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    /// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    #[must_use]
    pub const fn returned(&self, kind: ResourceKind) -> Accum {
        Accum(self.returned[kind.index()])
    }

    /// Returns the recovery rules that the ledger reads.
    #[must_use]
    pub const fn recovery(&self) -> RecoveryRules {
        self.rules
    }

    /// Replaces the recovery rules.
    ///
    /// The caller replaces the whole rule set, so the period of a kind lives
    /// in one place and no two sites can disagree.
    pub fn set_recovery(&mut self, rules: RecoveryRules) {
        self.rules = rules;
    }

    /// Returns the number of entries that the last recovery pass read.
    ///
    /// The pass reads the depleted set and nothing else. It takes no grid and
    /// no tile count, so it cannot read a tile that holds no stored take.
    #[must_use]
    pub const fn last_recovery_visits(&self) -> u64 {
        self.visits
    }

    /// Ages every stored take forward to a tick.
    ///
    /// The pass walks the depleted set in key order, which is the order the
    /// ledger holds, so the result does not depend on how the entries
    /// arrived.[^1] It visits no tile. A world in which nothing was gathered
    /// holds no entry, so one pass over it does no work, at any tile
    /// count.[^2]
    ///
    /// The pass leaves an entry that reached nothing owed in place. Removing
    /// such an entry is a separate change.
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^2]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    pub fn recover(&mut self, tick: Tick) {
        let rules = self.rules;
        let mut visits = 0u64;
        for entry in &mut self.entries {
            visits += 1;
            let after = aged(*entry, tick, period_of_key(rules, entry.key));
            if let Some(kind) = ResourceKind::from_u8((entry.key & 0b11) as u8) {
                self.returned[kind.index()] += i64::from(entry.taken - after.taken);
            }
            *entry = after;
        }
        self.visits = visits;
    }

    /// Returns the number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Reports whether the ledger holds no entry.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns what has been taken from one tile and kind.
    #[must_use]
    pub fn taken(&self, tile: TileIdx, kind: ResourceKind) -> Amount {
        let key = ledger_key(tile, kind);
        match self.entries.binary_search_by_key(&key, |entry| entry.key) {
            Ok(at) => Amount(self.entries[at].taken),
            Err(_) => Amount::ZERO,
        }
    }

    /// Returns what has been taken from one tile and kind, as at a tick.
    ///
    /// The answer is a pure function of the stored entry and the tick. Reading
    /// it changes nothing, so two readers at one tick get one answer and a
    /// reader never moves the world forward.
    #[must_use]
    pub fn taken_at(&self, tile: TileIdx, kind: ResourceKind, tick: Tick) -> Amount {
        let key = ledger_key(tile, kind);
        match self.entries.binary_search_by_key(&key, |entry| entry.key) {
            Ok(at) => Amount(aged(self.entries[at], tick, self.rules.period_of(kind)).taken),
            Err(_) => Amount::ZERO,
        }
    }

    /// Returns the entries, in ascending key order.
    #[must_use]
    pub fn entries(&self) -> &[LedgerEntry] {
        &self.entries
    }

    /// Returns the total that has been taken, over every entry.
    ///
    /// The accumulator is 64 bits wide and the addition is exactly
    /// associative, so the answer does not depend on the fold order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    #[must_use]
    pub fn total(&self) -> Accum {
        let mut total = 0i64;
        for entry in &self.entries {
            total += i64::from(entry.taken);
        }
        Accum(total)
    }

    /// Adds a run of amounts, given in ascending key order.
    ///
    /// The caller states the order and the merge relies on it. A run out of
    /// order would silently produce an unsorted result, and every later lookup
    /// would then read the wrong tile.
    pub fn merge_ascending(&mut self, run: &[(u64, u32)], tick: Tick) {
        debug_assert!(
            run.windows(2).all(|pair| pair[0].0 < pair[1].0),
            "a merged run must be sorted by key and hold each key once"
        );
        if run.is_empty() {
            return;
        }
        let rules = self.rules;
        self.scratch.clear();
        self.scratch.reserve(self.entries.len() + run.len());
        let (mut here, mut there) = (0usize, 0usize);
        while here < self.entries.len() && there < run.len() {
            let (mine, theirs) = (self.entries[here], run[there]);
            if mine.key < theirs.0 {
                self.scratch.push(mine);
                here += 1;
            } else if theirs.0 < mine.key {
                self.scratch.push(new_entry(theirs, tick));
                there += 1;
            } else {
                // The entry ages to the tick before the new take joins it. A
                // take added to a stale amount would carry the ticks that
                // passed before it into its own recovery, and the deposit
                // would then return the new take faster than the rule says.
                let current = aged(mine, tick, period_of_key(rules, mine.key));
                if let Some(kind) = ResourceKind::from_u8((mine.key & 0b11) as u8) {
                    self.returned[kind.index()] += i64::from(mine.taken - current.taken);
                }
                self.scratch.push(LedgerEntry {
                    taken: current.taken.saturating_add(theirs.1),
                    ..current
                });
                here += 1;
                there += 1;
            }
        }
        self.scratch.extend_from_slice(&self.entries[here..]);
        for fresh in &run[there..] {
            self.scratch.push(new_entry(*fresh, tick));
        }
        core::mem::swap(&mut self.entries, &mut self.scratch);
    }

    /// Absorbs the ledger into the state hash.
    ///
    /// The entries enter in key order, which the ledger holds them in.[^1]
    ///
    /// The total that recovery returned does not enter. It is a sum of the
    /// differences between the entries of one frame and the entries of the
    /// next, and the hash covers those entries on every frame, so a hash that
    /// wrote it would say the same thing twice.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        let mut running = hash.write_u64(self.entries.len() as u64);
        for entry in &self.entries {
            running = running
                .write_u64(entry.key)
                .write(&entry.taken.to_le_bytes())
                .write_u64(entry.anchor.0);
        }
        running
    }

    /// Reports whether the ledger holds its invariants.
    ///
    /// The entries rise and hold each key once. A ledger that broke either
    /// would answer a lookup with the wrong tile, and nothing else would
    /// notice.
    #[must_use]
    pub fn check_invariants(&self) -> bool {
        self.entries
            .windows(2)
            .all(|pair| pair[0].key < pair[1].key)
    }
}

/// Builds a ledger entry for a take that no entry held before.
///
/// The clock of the entry starts at the tick of the take.
#[must_use]
const fn new_entry(run: (u64, u32), tick: Tick) -> LedgerEntry {
    LedgerEntry {
        key: run.0,
        taken: run.1,
        anchor: tick,
    }
}

/// What one unit carries.
///
/// The load is one amount for each kind, in the order of the kind numbering.
/// It is plain data, so it enters the state hash without a conversion and
/// holds no undeclared padding.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct CarryLoad {
    /// One amount for each kind.
    pub amounts: [u32; RESOURCE_KIND_COUNT],
}

impl CarryLoad {
    /// The empty load.
    pub const EMPTY: Self = Self {
        amounts: [0; RESOURCE_KIND_COUNT],
    };

    /// Returns what the load holds of one kind.
    #[must_use]
    pub const fn of(self, kind: ResourceKind) -> Amount {
        Amount(self.amounts[kind.index()])
    }

    /// Returns the total of every kind.
    #[must_use]
    pub const fn total(self) -> Accum {
        let mut total = 0i64;
        let mut index = 0;
        while index < RESOURCE_KIND_COUNT {
            total += self.amounts[index] as i64;
            index += 1;
        }
        Accum(total)
    }

    /// Returns the load with an amount of one kind added.
    ///
    /// The addition saturates. A load that wrapped would create resource out
    /// of nothing, and the conservation check is what would fail.
    #[must_use]
    pub const fn with(self, kind: ResourceKind, amount: Amount) -> Self {
        let mut amounts = self.amounts;
        amounts[kind.index()] = amounts[kind.index()].saturating_add(amount.0);
        Self { amounts }
    }

    /// Returns the load with one kind reduced by an amount.
    ///
    /// The subtract saturates at zero. A load that wrapped would turn a small
    /// delivery into a full one, and nothing would fail, which is the same
    /// underflow shape that the consumption kernel refuses.[^1]
    ///
    /// **A caller takes what the load holds and never more.** The delivery
    /// reads the amount out of the load before it calls this, so the
    /// saturation is a guard rather than a case.
    ///
    /// # References
    ///
    /// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D3. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    #[must_use]
    pub const fn less(self, kind: ResourceKind, amount: Amount) -> Self {
        let mut amounts = self.amounts;
        amounts[kind.index()] = amounts[kind.index()].saturating_sub(amount.0);
        Self { amounts }
    }
}
