//! The tile stub value field.
//!
//! The field gives every tile a value. The value has two parts. The first
//! part is generated from the world seed and the tile index, so it costs
//! nothing to hold. The second part is the change that the frames have made
//! to that tile, and the field stores it. A tile that no frame has changed
//! has no stored part.
//!
//! Building a world therefore visits no tile and allocates nothing that
//! grows with the tile count. The cost is paid by the reader that asks for a
//! tile, and by the frame that changes one. This is the shape the product
//! record asks of the world, and it is the shape two accepted records
//! already give the ground and the tile stock.[^1] [^2] [^3]
//!
//! The stored changes are held sorted by tile index, so a lookup is a binary
//! search and the order never depends on how the changes were gathered.[^4]
//! A run of changes is merged in ascending order, never inserted one at a
//! time, because inserting into the middle of a vector moves every later
//! entry.
//!
//! Every value here is an integer or a Q16.16 fixed-point value, and every
//! arithmetic step goes through the arithmetic module.[^5] No item in this
//! module uses a floating-point type.
//!
//! # References
//!
//! [^1]: PRD-0003, a developer sees a world worth looking at, what it costs at the target scale. `docs/product/accepted/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
//! [^2]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
//! [^3]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
//! [^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
//! [^5]: ADR-0002, simulated and aggregated state holds no floating point number, decisions D1 and D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`

/// The census of generated tiles.
///
/// The census counts the tiles that the field has generated. It is a
/// test-only switch, in the same way the nondeterminism probe is, and the
/// whole module compiles to nothing when the feature is off.[^1]
///
/// **The counter is not generator state.** It observes a draw and never
/// feeds one, so no result depends on which thread served which tile.
/// Thread-local generator state is what the record forbids, and this is
/// neither thread-local nor a generator.[^2]
///
/// **The counter is one counter for the whole process.** A build fills the
/// first level of the pyramid on threads it starts, and a count held per
/// thread would miss that work and report a build as free. The test binary
/// that reads it therefore runs on one thread.
///
/// # References
///
/// [^1]: Testing rules, section 1. `.claude/rules/testing.md`
/// [^2]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
#[cfg(feature = "census-generated-tiles")]
pub mod census {
    use core::sync::atomic::{AtomicU64, Ordering};

    /// The tiles generated since the last reset.
    static GENERATED: AtomicU64 = AtomicU64::new(0);

    /// Counts one generated tile.
    pub fn count_one() {
        GENERATED.fetch_add(1, Ordering::Relaxed);
    }

    /// Returns the tiles generated since the last reset.
    #[must_use]
    pub fn generated() -> u64 {
        GENERATED.load(Ordering::Relaxed)
    }

    /// Sets the count back to zero.
    pub fn reset() {
        GENERATED.store(0, Ordering::Relaxed);
    }
}

use crate::hash::StateHash;
use crate::hex::Grid;
use crate::rng;
use crate::sim_math;
use crate::types::{Accum, Fix32, TileIdx};

/// The frame that the generated part of every tile value is keyed on.
///
/// The generated part does not change with time, so the frame slot of the
/// key holds one constant. The slot stays in the key because the key shape
/// is fixed by the record.[^1]
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
const GENERATED_FRAME: u64 = 0;

/// The draw index of the generated part of a tile value.
const GENERATED_DRAW: u32 = 0;

/// The shift that reduces one draw to the width of a tile value.
///
/// The draw is sixty-four bits wide and a tile value is a signed thirty-two
/// bit fixed-point number. The shift keeps the top twenty-four bits, which
/// is what the eager column held before the field became generated.
const GENERATED_SHIFT: u32 = 40;

/// One stored change to one tile.
///
/// The tile index and the change are separate fields, so a lookup compares
/// one integer and reads one integer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Change {
    /// The tile the change belongs to.
    tile: u32,
    /// What the frames have added to the generated part of that tile.
    delta: Fix32,
}

/// The tile stub value field.
///
/// The field holds the seed, the extent, and one entry for each tile that a
/// frame has changed. It holds no entry for any other tile.
#[derive(Clone, Debug)]
pub struct TileValues {
    /// The seed that the generated part is drawn from.
    seed: u64,
    /// The extent that the field covers.
    grid: Grid,
    /// The stored changes, in ascending tile order, one entry for each tile.
    changes: Vec<Change>,
    /// The buffer that a merge builds into, kept so a merge allocates once.
    scratch: Vec<Change>,
}

impl TileValues {
    /// Builds a field over an extent.
    ///
    /// The call visits no tile and allocates nothing. A world of any size
    /// costs the same to build.
    #[must_use]
    pub const fn new(seed: u64, grid: Grid) -> Self {
        Self {
            seed,
            grid,
            changes: Vec::new(),
            scratch: Vec::new(),
        }
    }

    /// Returns the extent the field covers.
    #[must_use]
    pub const fn grid(&self) -> Grid {
        self.grid
    }

    /// Returns the number of tiles the field covers.
    #[must_use]
    pub fn tile_count(&self) -> usize {
        self.grid.tile_count() as usize
    }

    /// Returns the number of tiles that hold a stored change.
    ///
    /// A world that has never stepped holds none, at any tile count. The
    /// count grows with what the frames have changed and never with the size
    /// of the world alone.
    #[must_use]
    pub fn stored_changes(&self) -> usize {
        self.changes.len()
    }

    /// Returns the generated part of one tile.
    ///
    /// The value reads the seed and the tile index and nothing else, so two
    /// readers that visit the world in different orders read one world.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn generated(seed: u64, tile: TileIdx) -> Fix32 {
        #[cfg(feature = "census-generated-tiles")]
        census::count_one();
        let raw = rng::draw(
            seed,
            rng::SYSTEM_TILE_STUB,
            GENERATED_FRAME,
            tile.0 as u64,
            GENERATED_DRAW,
        );
        Fix32((raw >> GENERATED_SHIFT) as i32)
    }

    /// Returns the stored change of one tile, which is zero when none is
    /// stored.
    #[must_use]
    pub fn stored(&self, tile: TileIdx) -> Fix32 {
        match self
            .changes
            .binary_search_by_key(&tile.0, |entry| entry.tile)
        {
            Ok(at) => self.changes[at].delta,
            Err(_) => Fix32(0),
        }
    }

    /// Returns the value of one tile.
    ///
    /// Returns `None` when the index is outside the extent. The read costs
    /// one draw and one binary search, and neither grows with the number of
    /// tiles that have never changed.
    #[must_use]
    pub fn at(&self, tile: TileIdx) -> Option<Fix32> {
        if tile.0 >= self.grid.tile_count() {
            return None;
        }
        Some(sim_math::add(
            Self::generated(self.seed, tile),
            self.stored(tile),
        ))
    }

    /// Returns the value of one tile without checking the extent.
    ///
    /// The caller has already established that the index names a tile. The
    /// whole-field passes use this, because they walk the extent themselves
    /// and would otherwise check each index twice.
    #[must_use]
    fn at_unchecked(&self, tile: TileIdx) -> Fix32 {
        sim_math::add(Self::generated(self.seed, tile), self.stored(tile))
    }

    /// Returns every value of the field, in ascending tile order.
    ///
    /// **The call visits every tile and allocates one value for each.** It is
    /// named for the copy, because a caller that reads the whole field pays
    /// for the whole field and the call site must say so.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR Registry, row 0044. `docs/adrs/REGISTRY.md`
    #[must_use]
    pub fn copy_all(&self) -> Vec<Fix32> {
        let count = self.grid.tile_count();
        let mut values = Vec::with_capacity(count as usize);
        for index in 0..count {
            values.push(self.at_unchecked(TileIdx(index)));
        }
        values
    }

    /// Returns the sum of every tile value.
    ///
    /// The accumulator is 64 bits wide and the addition is exactly
    /// associative, so the answer does not depend on the fold order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    #[must_use]
    pub fn total(&self) -> Accum {
        let mut total = Accum(0);
        for index in 0..self.grid.tile_count() {
            total = sim_math::accumulate(total, self.at_unchecked(TileIdx(index)));
        }
        total
    }

    /// Absorbs the field into a running hash.
    ///
    /// The pass writes the value of every tile, in ascending tile order. It
    /// does not write the seed alone. The seed is the input of the generator
    /// and a change to the generator moves every tile of every world while
    /// leaving the seed untouched, so only the tiles report it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, the consequences. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        let mut running = hash;
        for index in 0..self.grid.tile_count() {
            running = running.write(&self.at_unchecked(TileIdx(index)).0.to_le_bytes());
        }
        running
    }

    /// Adds a run of changes, given in ascending tile order.
    ///
    /// Each pair is a tile index and the amount to add to what that tile
    /// already holds. The caller states the order and the merge relies on
    /// it. A run out of order would produce an unsorted result, and every
    /// later lookup would then read the wrong tile.
    pub fn merge_ascending(&mut self, run: &[(u32, Fix32)]) {
        debug_assert!(
            run.windows(2).all(|pair| pair[0].0 < pair[1].0),
            "a merged run must be sorted by tile and hold each tile once"
        );
        if run.is_empty() {
            return;
        }
        self.scratch.clear();
        self.scratch.reserve(self.changes.len() + run.len());
        let (mut here, mut there) = (0usize, 0usize);
        while here < self.changes.len() && there < run.len() {
            let (mine, theirs) = (self.changes[here], run[there]);
            if mine.tile < theirs.0 {
                self.scratch.push(mine);
                here += 1;
            } else if theirs.0 < mine.tile {
                self.scratch.push(Change {
                    tile: theirs.0,
                    delta: theirs.1,
                });
                there += 1;
            } else {
                self.scratch.push(Change {
                    tile: mine.tile,
                    delta: sim_math::add(mine.delta, theirs.1),
                });
                here += 1;
                there += 1;
            }
        }
        self.scratch.extend_from_slice(&self.changes[here..]);
        for pair in &run[there..] {
            self.scratch.push(Change {
                tile: pair.0,
                delta: pair.1,
            });
        }
        core::mem::swap(&mut self.changes, &mut self.scratch);
    }

    /// Reports whether the stored changes hold their stated shape.
    ///
    /// Every entry names a tile inside the extent, and the entries are in
    /// ascending tile order with each tile held once. A lookup is a binary
    /// search, so an unsorted entry would not fail. It would return the
    /// wrong tile.
    #[must_use]
    pub fn check_invariants(&self) -> bool {
        let count = self.grid.tile_count();
        self.changes.iter().all(|entry| entry.tile < count)
            && self
                .changes
                .windows(2)
                .all(|pair| pair[0].tile < pair[1].tile)
    }
}

/// A read-only view of the field that a worker thread carries.
///
/// A worker updates one contiguous range of tiles. It needs the generated
/// part of each tile in its range and the change already stored there, and
/// it must not write to the field, because the merge that follows the join
/// is what fixes the order of the result.[^1]
///
/// # References
///
/// [^1]: ADR-0009, parallel stages write disjoint outputs. `docs/adrs/REGISTRY.md`
#[derive(Clone, Copy, Debug)]
pub struct TileValueRange<'a> {
    /// The seed that the generated part is drawn from.
    seed: u64,
    /// The stored changes of this range, in ascending tile order.
    changes: &'a [Change],
    /// The first tile of the range.
    start: u32,
}

impl<'a> TileValueRange<'a> {
    /// Returns the first tile of the range.
    #[must_use]
    pub const fn start(&self) -> u32 {
        self.start
    }

    /// Returns the value of one tile of the range.
    ///
    /// The walk is linear in the stored changes of the range, because the
    /// caller visits the tiles in ascending order and the changes are held
    /// in that order. The cursor is what makes it linear rather than one
    /// binary search for each tile.
    #[must_use]
    pub fn value(&self, tile: TileIdx, cursor: &mut usize) -> Fix32 {
        while *cursor < self.changes.len() && self.changes[*cursor].tile < tile.0 {
            *cursor += 1;
        }
        let stored = match self.changes.get(*cursor) {
            Some(entry) if entry.tile == tile.0 => entry.delta,
            _ => Fix32(0),
        };
        sim_math::add(TileValues::generated(self.seed, tile), stored)
    }
}

impl TileValues {
    /// Returns a read-only view of one contiguous range of tiles.
    ///
    /// The range runs from `start` up to but not including `end`. The view
    /// carries only the stored changes of that range, so two views of two
    /// disjoint ranges read disjoint entries.
    #[must_use]
    pub fn range(&self, start: u32, end: u32) -> TileValueRange<'_> {
        let first = self.changes.partition_point(|entry| entry.tile < start);
        let last = self.changes.partition_point(|entry| entry.tile < end);
        TileValueRange {
            seed: self.seed,
            changes: &self.changes[first..last],
            start,
        }
    }
}
