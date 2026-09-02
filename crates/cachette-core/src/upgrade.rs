//! Tile upgrades.
//!
//! An upgrade is the mark a unit leaves on a tile. The generator made the
//! ground and the stock of the world, and neither of those records anything a
//! unit did.[^1] [^2] An upgrade is the opposite: no function of the seed can
//! produce it, so the engine stores it.
//!
//! **The store holds one entry for each tile that carries an upgrade, and
//! nothing else.** A world in which nobody built holds no entry, so the memory
//! cost follows the building and not the size of the world.[^3]
//!
//! An upgrade under construction holds a progress accumulator. Several units
//! add to it in one tick and the contributions combine exactly, because every
//! term is a whole number and the accumulator is 64 bits wide.[^4] [^5] The
//! accumulator is clamped at the work its kind asks for. An unclamped
//! accumulator lets a builder bank surplus it can never spend, and that
//! overflow reaches the state hash.[^6]
//!
//! No item in this module uses a floating-point type.[^4]
//!
//! # References
//!
//! [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
//! [^2]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D1. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
//! [^3]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
//! [^4]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^5]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
//! [^6]: Findings register, FND-011. `docs/FINDINGS.md`

use crate::hash::StateHash;
use crate::types::{Accum, TileIdx};

/// The number of upgrade kinds that the catalogue holds.
pub const UPGRADE_KIND_COUNT: usize = 2;

/// The kind of an upgrade.
///
/// A kind is an index into the tables below. It is not a type, not a trait
/// and not a verb, so adding a kind adds a row and no code.[^1]
///
/// The catalogue starts at two kinds, because the two of them change
/// different properties of a tile. One kind would let a later reader believe
/// that an upgrade is a scalar on the tile rather than a row in a table.[^1]
///
/// # References
///
/// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D3. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum UpgradeKind {
    /// A made way. More units cross the tile at once.
    Road = 0,
    /// Worked ground. A unit takes more from the tile in one tick.
    Terrace = 1,
}

impl UpgradeKind {
    /// Every kind, in the order of the numbering.
    ///
    /// A caller that must reason over the whole catalogue reads this rather
    /// than writing a list of its own. The length is fixed by the kind count,
    /// so a new kind that is not added here is a compile error.
    pub const ALL: [Self; UPGRADE_KIND_COUNT] = [Self::Road, Self::Terrace];

    /// Returns the kind as a small integer.
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    /// Returns the kind that a small integer names.
    ///
    /// Returns `None` when the number names no kind.
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Road),
            1 => Some(Self::Terrace),
            _ => None,
        }
    }

    /// Returns the position of the kind in a table over the catalogue.
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Returns the work that finishes an upgrade of this kind.
    ///
    /// The value is content. It sits beside the effect tables below until a
    /// content pipeline exists, in the same way the ground tables do.[^1] It
    /// is not a cost figure: it says how much work the world asks for, not
    /// what the engine spends.[^2]
    ///
    /// The value is above the work that one builder adds in one tick, so a
    /// build takes several ticks and holds state between them. That is the
    /// whole point of the shape, and a test asserts it.[^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D2. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    /// [^2]: Blockers register, BLK-007. `docs/BLOCKERS.md`
    /// [^3]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D2. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    #[must_use]
    pub const fn work(self) -> i64 {
        match self {
            Self::Road => 8,
            Self::Terrace => 24,
        }
    }

    /// Returns the capacity that a finished upgrade of this kind gives a tile.
    ///
    /// Returns `None` when the kind does not change how many units a tile
    /// holds.
    ///
    /// A made way is ground that a unit crosses quickly, and the project
    /// already holds the capacity of such ground. The value is not restated
    /// here: the terrain module owns the capacity table and this row reads it
    /// from there, so no second declaration can disagree with it.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
    /// [^2]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub const fn capacity(self) -> Option<u32> {
        match self {
            Self::Road => Some(crate::terrain::CROSSING_CAPACITY),
            Self::Terrace => None,
        }
    }

    /// Returns how much more a unit takes from the tile in one tick.
    ///
    /// The row adds to the rate that the gather resolve grants. It does not
    /// change what the tile started with, which is generated and fixed.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D1. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
    #[must_use]
    pub const fn gather_bonus(self) -> u32 {
        match self {
            Self::Road => 0,
            Self::Terrace => 2,
        }
    }
}

/// The work that one builder adds to a site in one tick.
///
/// The rate is content, and the register holds the open choice of its
/// value.[^1] It is smaller than the work of every kind, so no build finishes
/// in the tick it started.
///
/// # References
///
/// [^1]: Decisions register, DEC-072. `docs/DECISIONS.md`
pub const BUILD_RATE: i64 = 1;

/// Returns the largest work that any kind in the catalogue asks for.
///
/// The value is folded over the catalogue rather than written down a second
/// time. A written ceiling is one fact in two places, and nothing fails when
/// the two disagree.[^1]
///
/// This is the bound that the progress accumulator is clamped to, so it is
/// the bound the overflow property test names.[^2]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
/// [^2]: Testing rules, section 4. `.claude/rules/testing.md`
#[must_use]
pub const fn largest_work() -> i64 {
    let mut most = 0i64;
    let mut at = 0usize;
    while at < UPGRADE_KIND_COUNT {
        let work = UpgradeKind::ALL[at].work();
        if work > most {
            most = work;
        }
        at += 1;
    }
    most
}

/// The number of key bits that hold the kind.
///
/// The width is derived from the catalogue, so a new kind widens the key
/// rather than colliding inside it.
const KIND_BITS: u32 = UPGRADE_KIND_COUNT.next_power_of_two().trailing_zeros();

/// Packs a tile and a kind into one ordering key.
///
/// The tile is the high part, so a sort by this key gives ascending tile
/// order, and the segments of one tile stay together.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
#[must_use]
pub const fn site_key(tile: TileIdx, kind: UpgradeKind) -> u64 {
    ((tile.0 as u64) << KIND_BITS) | (kind.to_u8() as u64)
}

/// Returns the largest key that a world of a given tile count produces.
#[must_use]
pub const fn key_ceiling(tile_count: u32) -> u64 {
    site_key(
        TileIdx(tile_count.saturating_sub(1)),
        UpgradeKind::ALL[UPGRADE_KIND_COUNT - 1],
    )
}

/// Returns how many units may stand on a tile.
///
/// This is the one function that answers the question. The ground states the
/// capacity, a finished upgrade may state a larger one, and the larger of the
/// two wins. The two tables meet in one place, so no caller can read one
/// without the other.[^1]
///
/// **Ground that admits nobody stays closed.** An upgrade changes how many a
/// tile holds. It never changes whether the tile holds anybody, so every
/// caller that asks only about passability reads the ground and stays
/// correct.
///
/// The argument is the finished upgrade. A site under construction changes
/// nothing.
///
/// # References
///
/// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D3. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
#[must_use]
pub const fn capacity_with(ground: u32, finished: Option<UpgradeKind>) -> u32 {
    if ground == 0 {
        return 0;
    }
    match finished {
        Some(kind) => match kind.capacity() {
            // An upgrade never lowers what a tile holds. The larger of the
            // two wins, so a kind whose row sits below the ground it stands
            // on changes nothing rather than taking room away.
            Some(given) if given > ground => given,
            _ => ground,
        },
        None => ground,
    }
}

/// Returns how much one unit takes from a tile in one tick.
///
/// The base rate is what the gather resolve grants on unimproved ground, and
/// a finished upgrade adds to it.
#[must_use]
pub const fn gather_rate_with(base: u32, finished: Option<UpgradeKind>) -> u32 {
    match finished {
        Some(kind) => base.saturating_add(kind.gather_bonus()),
        None => base,
    }
}

/// One upgrade, finished or under construction.
///
/// A tile carries at most one upgrade. Two upgrades on one tile would make
/// "the tile returns to what it was" a question with more than one answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UpgradeSite {
    /// The tile that carries the upgrade.
    pub tile: TileIdx,
    /// What is being built, or what stands there.
    pub kind: UpgradeKind,
    /// The work that has gone into it.
    ///
    /// The accumulator is 64 bits wide and every term is a whole number, so
    /// the total is the same in any order.[^1] It never rises above the work
    /// its kind asks for.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    /// [^2]: Findings register, FND-011. `docs/FINDINGS.md`
    pub progress: Accum,
}

impl UpgradeSite {
    /// Reports whether the upgrade is finished.
    #[must_use]
    pub const fn is_complete(self) -> bool {
        self.progress.0 >= self.kind.work()
    }

    /// Returns the work that the site still asks for.
    #[must_use]
    pub const fn remaining(self) -> i64 {
        let work = self.kind.work();
        if self.progress.0 >= work {
            0
        } else {
            work - self.progress.0
        }
    }
}

/// Every upgrade in one world.
///
/// The map holds one entry for each tile that carries an upgrade, and it
/// holds nothing else. A world in which nobody built holds no entry, so the
/// memory cost follows the building and not the size of the world.[^1]
///
/// The entries are held sorted by tile, so a lookup is a binary search and
/// the order never depends on which unit built first.[^2]
///
/// An entry is merged in ascending runs, never inserted one at a time.
/// Inserting into the middle of a vector moves every later entry, which is
/// quadratic in the number of tiles a frame touches.
///
/// # References
///
/// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
/// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UpgradeMap {
    sites: Vec<UpgradeSite>,
    scratch: Vec<UpgradeSite>,
    visits: u64,
}

impl UpgradeMap {
    /// Builds a map that holds no upgrade.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            sites: Vec::new(),
            scratch: Vec::new(),
            visits: 0,
        }
    }

    /// Returns the number of tiles that carry an upgrade.
    #[must_use]
    pub fn len(&self) -> usize {
        self.sites.len()
    }

    /// Reports whether no tile carries an upgrade.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sites.is_empty()
    }

    /// Returns every upgrade, in ascending tile order.
    #[must_use]
    pub fn sites(&self) -> &[UpgradeSite] {
        &self.sites
    }

    /// Returns the number of entries that the last advance read.
    ///
    /// The advance reads the sites and the builders. It takes no grid and no
    /// tile count, so it cannot read a tile that carries no upgrade.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    #[must_use]
    pub const fn last_advance_visits(&self) -> u64 {
        self.visits
    }

    /// Returns the upgrade on one tile.
    ///
    /// Returns `None` when the tile carries none.
    #[must_use]
    pub fn at(&self, tile: TileIdx) -> Option<UpgradeSite> {
        match self.sites.binary_search_by_key(&tile.0, |site| site.tile.0) {
            Ok(at) => Some(self.sites[at]),
            Err(_) => None,
        }
    }

    /// Returns the finished upgrade on one tile.
    ///
    /// Returns `None` when the tile carries none, and when the upgrade there
    /// is still under construction. A site that is not finished changes
    /// nothing about the tile.
    #[must_use]
    pub fn finished(&self, tile: TileIdx) -> Option<UpgradeKind> {
        let site = self.at(tile)?;
        if site.is_complete() {
            Some(site.kind)
        } else {
            None
        }
    }

    /// Removes the upgrade from one tile and returns what stood there.
    ///
    /// The tile returns to the world the generator made. Nothing else stores
    /// a property of the tile, so removing the entry is the whole of the
    /// return, and no second copy can survive it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D4. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
    pub fn remove(&mut self, tile: TileIdx) -> Option<UpgradeSite> {
        match self.sites.binary_search_by_key(&tile.0, |site| site.tile.0) {
            Ok(at) => Some(self.sites.remove(at)),
            Err(_) => None,
        }
    }

    /// Adds a run of work, given in ascending tile order.
    ///
    /// Each element names a tile, the kind being built there, and the work
    /// that this tick added. The caller states the order and the merge relies
    /// on it: a run out of order would silently produce an unsorted map, and
    /// every later lookup would then read the wrong tile.
    ///
    /// A tile that holds no site gains one. A tile that holds a site of the
    /// named kind advances it. **A tile that holds a site of another kind is
    /// left alone**, because a tile carries one upgrade and the one that is
    /// already there is the one the world holds.
    ///
    /// The progress is clamped at the work its kind asks for. An unclamped
    /// accumulator banks surplus that nothing can spend, and that surplus
    /// reaches the state hash.[^1]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-011. `docs/FINDINGS.md`
    pub fn merge_ascending(&mut self, run: &[(TileIdx, UpgradeKind, i64)]) {
        debug_assert!(
            run.windows(2).all(|pair| pair[0].0 .0 < pair[1].0 .0),
            "a merged run must be sorted by tile and name each tile once"
        );
        debug_assert!(
            run.iter().all(|added| added.2 >= 0),
            "work never runs backwards"
        );
        let mut visits = 0u64;
        if run.is_empty() {
            // Nothing was built, so the merge read nothing. It did not walk
            // the sites, and it did not walk the world.
            self.visits = 0;
            return;
        }
        self.scratch.clear();
        self.scratch.reserve(self.sites.len() + run.len());
        let (mut here, mut there) = (0usize, 0usize);
        while here < self.sites.len() && there < run.len() {
            let mine = self.sites[here];
            let theirs = run[there];
            visits += 1;
            if mine.tile.0 < theirs.0 .0 {
                self.scratch.push(mine);
                here += 1;
            } else if theirs.0 .0 < mine.tile.0 {
                self.scratch.push(fresh_site(theirs));
                there += 1;
            } else {
                self.scratch.push(advanced(mine, theirs.1, theirs.2));
                here += 1;
                there += 1;
            }
        }
        visits += (self.sites.len() - here) as u64;
        self.scratch.extend_from_slice(&self.sites[here..]);
        for added in &run[there..] {
            visits += 1;
            self.scratch.push(fresh_site(*added));
        }
        core::mem::swap(&mut self.sites, &mut self.scratch);
        self.visits = visits;
    }

    /// Absorbs the map into the state hash.
    ///
    /// The entries enter in tile order, which the map holds them in.[^1] An
    /// unfinished build is state that the next frame reads, so the progress
    /// enters as well.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        let mut running = hash.write_u64(self.sites.len() as u64);
        for site in &self.sites {
            running = running
                .write(&site.tile.0.to_le_bytes())
                .write(&[site.kind.to_u8()])
                .write(&site.progress.0.to_le_bytes());
        }
        running
    }

    /// Reports whether the map holds its invariants.
    ///
    /// The entries rise and name each tile once. A map that broke either
    /// would answer a lookup with the wrong tile, and nothing else would
    /// notice.
    ///
    /// The progress of every site sits between nothing and the work its kind
    /// asks for. A site above the work has banked surplus, which is the
    /// defect the register names.[^1]
    ///
    /// Every tile lies inside the world.
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-011. `docs/FINDINGS.md`
    #[must_use]
    pub fn check_invariants(&self, tile_count: u32) -> bool {
        if !self
            .sites
            .windows(2)
            .all(|pair| pair[0].tile.0 < pair[1].tile.0)
        {
            return false;
        }
        self.sites.iter().all(|site| {
            site.tile.0 < tile_count && site.progress.0 >= 0 && site.progress.0 <= site.kind.work()
        })
    }
}

/// Builds the site that a first contribution creates.
#[must_use]
fn fresh_site(added: (TileIdx, UpgradeKind, i64)) -> UpgradeSite {
    let (tile, kind, work) = added;
    UpgradeSite {
        tile,
        kind,
        progress: Accum(work.clamp(0, kind.work())),
    }
}

/// Adds work to a site that already stands on the tile.
///
/// A contribution to another kind is dropped. The tile carries one upgrade,
/// and it is not the one the contributor named.
#[must_use]
fn advanced(site: UpgradeSite, kind: UpgradeKind, work: i64) -> UpgradeSite {
    if site.kind != kind {
        return site;
    }
    UpgradeSite {
        progress: Accum(site.progress.0.saturating_add(work).clamp(0, kind.work())),
        ..site
    }
}
