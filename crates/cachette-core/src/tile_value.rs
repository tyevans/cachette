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
//! record asks of the world, and a record states it as a claim over any tile
//! field.[^1] [^2] It is the shape two accepted records already give the
//! ground and the tile stock.[^3] [^4]
//!
//! The stored changes are held sorted by tile index, so a lookup is a binary
//! search and the order never depends on how the changes were gathered.[^5]
//! A run of changes is merged in ascending order, never inserted one at a
//! time, because inserting into the middle of a vector moves every later
//! entry.
//!
//! Every value here is an integer or a Q16.16 fixed-point value, and every
//! arithmetic step goes through the arithmetic module.[^6] No item in this
//! module uses a floating-point type.
//!
//! # References
//!
//! [^1]: PRD-0003, a developer sees a world worth looking at, what it costs at the target scale. `docs/product/accepted/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
//! [^2]: ADR-0088, a tile field is a generated base and a stored change, decision D1. `docs/adrs/draft/adr-0088-a-tile-field-is-a-generated-base-and-a-stored-change.md`
//! [^3]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
//! [^4]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
//! [^5]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
//! [^6]: ADR-0002, simulated and aggregated state holds no floating point number, decisions D1 and D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`

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

/// The tile stub value field.
///
/// The field holds the seed, the extent, and one delta for every tile once
/// any tile has changed.
///
/// **The stored part is dense, and it is never a sorted list of changes.**[^1]
/// A sparse list is smaller only while few tiles have changed, and a
/// measurement of the target extent found that the list reaches almost every
/// tile within ten frames. After that the sparse form costs more memory than
/// a dense one, because it carries a tile index beside every value and needs
/// a second buffer to merge into, and it costs more time, because a merge
/// that rebuilds the list walks every entry to apply a handful.[^2]
///
/// **The array is allocated at the first change and never at build.** A world
/// that has not changed a tile holds no array, so building a world still
/// visits no tile and allocates nothing for the field.[^3] [^4]
///
/// # References
///
/// [^1]: ADR-0103, the tile value field stores a dense delta, never a sparse change list, decision D1. `docs/adrs/draft/adr-0103-the-tile-value-field-stores-a-dense-delta.md`
/// [^2]: Findings register, FND-292. `docs/FINDINGS.md`
/// [^3]: ADR-0103, the tile value field stores a dense delta, never a sparse change list, decision D2. `docs/adrs/draft/adr-0103-the-tile-value-field-stores-a-dense-delta.md`
/// [^4]: ADR-0088, a tile field is a generated base and a stored change. `docs/adrs/draft/adr-0088-a-tile-field-is-a-generated-base-and-a-stored-change.md`
#[derive(Clone, Debug)]
pub struct TileValues {
    /// The seed that the generated part is drawn from.
    seed: u64,
    /// The extent that the field covers.
    grid: Grid,
    /// One delta for every tile, in tile order.
    ///
    /// The vector is empty until the first change, and it holds exactly one
    /// entry for each tile afterwards. Those are the only two shapes it
    /// takes, and the invariant check states that.
    deltas: Vec<Fix32>,
    /// The number of tiles whose delta is not zero.
    changed: usize,
    /// Whether a run has ever named a tile outside the extent.
    ///
    /// **A dense array cannot hold such a tile, so it would otherwise leave
    /// no trace.** The sorted list this replaced stored the bad entry, and
    /// the invariant check found it there. The flag keeps that defect
    /// detectable, and it is sticky for the same reason the bad entry was
    /// permanent: the caller wrote something the grid does not name, and a
    /// later good run does not make that untrue.
    outside: bool,
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
            deltas: Vec::new(),
            changed: 0,
            outside: false,
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

    /// Returns the number of tiles whose value differs from the generated one.
    ///
    /// A world that has never stepped holds none, at any tile count.
    ///
    /// **This counts tiles and not entries.** The field stores a delta for
    /// every tile once any tile has changed, so the count is a property of
    /// the world rather than of the allocation. A tile whose deltas cancel
    /// back to zero leaves the count, which the sorted list this replaced
    /// could not express.
    #[must_use]
    pub fn stored_changes(&self) -> usize {
        self.changed
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
    ///
    /// The read is one index. A world that has changed no tile holds no
    /// array, and every tile of it reads zero.
    fn stored(&self, tile: TileIdx) -> Fix32 {
        self.deltas
            .get(tile.0 as usize)
            .copied()
            .unwrap_or(Fix32(0))
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

    /// Adds a run of changes.
    ///
    /// Each pair is a tile index and the amount to add to what that tile
    /// already holds.
    ///
    /// **The cost follows the run and never the field.** The pass writes one
    /// entry for each pair and reads nothing else, so a frame that changes a
    /// handful of tiles pays for a handful. The sorted list this replaced
    /// rebuilt itself on every call, so its cost followed the number of tiles
    /// that had ever changed, and a measurement found that number reaching
    /// almost every tile within ten frames.[^1]
    ///
    /// **The result does not depend on the order of the run**, because each
    /// pair writes its own tile and no pair reads another. The caller still
    /// passes an ascending run with each tile once, and the assertion below
    /// holds it to that, because a repeated tile would add twice and mean
    /// something the caller did not say.
    ///
    /// The array is allocated here, on the first call that carries a change,
    /// and never when the world is built.[^2]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-292. `docs/FINDINGS.md`
    /// [^2]: ADR-0103, the tile value field stores a dense delta, never a sparse change list, decision D2. `docs/adrs/draft/adr-0103-the-tile-value-field-stores-a-dense-delta.md`
    pub fn merge_ascending(&mut self, run: &[(u32, Fix32)]) {
        debug_assert!(
            run.windows(2).all(|pair| pair[0].0 < pair[1].0),
            "a merged run must be sorted by tile and hold each tile once"
        );
        if run.is_empty() {
            return;
        }
        if self.deltas.is_empty() {
            self.deltas = vec![Fix32(0); self.grid.tile_count() as usize];
        }
        for (tile, delta) in run {
            let Some(slot) = self.deltas.get_mut(*tile as usize) else {
                // A tile outside the extent is a caller defect. The array
                // cannot hold it, and growing to fit it would put an entry in
                // the array for something the grid says is not a tile. The
                // field records the defect instead, so that the invariant
                // check reports it rather than the write vanishing.
                self.outside = true;
                continue;
            };
            let before = *slot;
            let after = sim_math::add(before, *delta);
            *slot = after;
            match (before.0 == 0, after.0 == 0) {
                (true, false) => self.changed += 1,
                (false, true) => self.changed -= 1,
                _ => {}
            }
        }
    }

    /// Reports whether the stored deltas hold their stated shape.
    ///
    /// The array takes one of two shapes and no other. It is empty, which is
    /// a world that has changed no tile, or it holds exactly one entry for
    /// each tile of the extent. A shorter array would read zero for a tile
    /// that holds a change, and a longer one would hold an entry for a tile
    /// the grid does not name.
    ///
    /// **A run that named a tile outside the extent fails this check**, and
    /// it keeps failing. The array cannot hold such a tile, so the field
    /// records that it was asked to and reports it here.
    ///
    /// The count of changed tiles is derived from the array, so this checks
    /// it against the array rather than trusting it. The count is maintained
    /// as the array is written, which is a second place that one fact
    /// lives, and this is what fails when the two disagree.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub fn check_invariants(&self) -> bool {
        if self.outside {
            return false;
        }
        if self.deltas.is_empty() {
            return self.changed == 0;
        }
        self.deltas.len() == self.grid.tile_count() as usize
            && self.changed == self.deltas.iter().filter(|delta| delta.0 != 0).count()
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
    /// The tile the slice below starts at.
    start: u32,
    /// The stored delta of each tile of this range, in tile order.
    ///
    /// The slice is empty when the world has changed no tile, and every tile
    /// of the range then reads zero.
    deltas: &'a [Fix32],
}

impl<'a> TileValueRange<'a> {
    /// Returns the value of one tile of the range.
    ///
    /// **The read is one index.** The sorted list this replaced needed a
    /// cursor, which the caller carried and advanced, so that a walk over the
    /// range cost one pass rather than one binary search for each tile. A
    /// dense slice needs neither, so the cursor is gone and the caller no
    /// longer carries one.
    ///
    /// A tile outside the range reads the generated value alone. The caller
    /// walks the range it asked for, so that case does not arise, and
    /// returning the generated value is the answer for a tile that holds no
    /// change in any event.
    #[must_use]
    pub fn value(&self, tile: TileIdx) -> Fix32 {
        let stored = tile
            .0
            .checked_sub(self.start)
            .and_then(|at| self.deltas.get(at as usize))
            .copied()
            .unwrap_or(Fix32(0));
        sim_math::add(TileValues::generated(self.seed, tile), stored)
    }
}

/// One contiguous range of tiles that a worker may write.
///
/// **The slice bounds the worker.** A chunk carries the deltas of its own
/// range and nothing else, so two chunks of one field write disjoint memory
/// and a worker cannot reach a tile that another worker holds. That is the
/// requirement on a parallel stage, met by construction rather than by a rule
/// a reviewer has to check.[^1]
///
/// A chunk cannot name a tile outside the extent either, because the field
/// built it from the array and the array covers the extent exactly.
///
/// # References
///
/// [^1]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
#[derive(Debug)]
pub struct TileValueChunk<'a> {
    /// The seed that the generated part is drawn from.
    seed: u64,
    /// The tile the slice starts at.
    start: u32,
    /// The delta of each tile of this range, in tile order.
    deltas: &'a mut [Fix32],
    /// The net change this chunk made to the count of non-zero deltas.
    changed: i64,
}

impl TileValueChunk<'_> {
    /// Returns the first tile of the range.
    #[must_use]
    pub const fn start(&self) -> u32 {
        self.start
    }

    /// Returns the tile after the last tile of the range.
    #[must_use]
    pub fn end(&self) -> u32 {
        self.start + self.deltas.len() as u32
    }

    /// Returns the value of one tile of the range.
    ///
    /// A tile outside the range reads the generated value alone. The caller
    /// walks the range it was given, so that case does not arise.
    #[must_use]
    pub fn value(&self, tile: TileIdx) -> Fix32 {
        let stored = tile
            .0
            .checked_sub(self.start)
            .and_then(|at| self.deltas.get(at as usize))
            .copied()
            .unwrap_or(Fix32(0));
        sim_math::add(TileValues::generated(self.seed, tile), stored)
    }

    /// Adds to one tile of the range and returns the value it then holds.
    ///
    /// Returns `None` when the tile is outside the range. The field cannot
    /// see that case, because the range is the whole of what this chunk owns,
    /// so the caller is the one that must not ignore it.
    #[must_use]
    pub fn add(&mut self, tile: TileIdx, delta: Fix32) -> Option<Fix32> {
        let at = tile.0.checked_sub(self.start)? as usize;
        let slot = self.deltas.get_mut(at)?;
        let before = *slot;
        let after = sim_math::add(before, delta);
        *slot = after;
        match (before.0 == 0, after.0 == 0) {
            (true, false) => self.changed += 1,
            (false, true) => self.changed -= 1,
            _ => {}
        }
        Some(sim_math::add(TileValues::generated(self.seed, tile), after))
    }

    /// Returns the net change this chunk made to the count of changed tiles.
    #[must_use]
    pub const fn changed(&self) -> i64 {
        self.changed
    }
}

impl TileValues {
    /// Allocates the delta array if it is not allocated already.
    ///
    /// A caller that is about to write the field calls this once, before it
    /// divides the field between workers. **Building a world does not call
    /// it**, so a world that has run no frame still allocates nothing for
    /// this field.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0103, the tile value field stores a dense delta, never a sparse change list, decision D2. `docs/adrs/draft/adr-0103-the-tile-value-field-stores-a-dense-delta.md`
    pub fn prepare(&mut self) {
        if self.deltas.is_empty() {
            self.deltas = vec![Fix32(0); self.grid.tile_count() as usize];
        }
    }

    /// Divides the field into contiguous chunks that workers may write.
    ///
    /// The chunks are disjoint and they cover the extent in ascending tile
    /// order. The caller must have called `prepare` first; a field that holds
    /// no array yields no chunk, and the caller would then write nothing.
    ///
    /// **The division is by tile and never by thread completion.** Two runs
    /// at two thread counts give each tile to one chunk, and each chunk
    /// writes only its own tiles, so the field does not depend on which
    /// worker finished first.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub fn chunks_mut(&mut self, chunk_len: u32) -> impl Iterator<Item = TileValueChunk<'_>> {
        let seed = self.seed;
        let stride = chunk_len.max(1) as usize;
        self.deltas
            .chunks_mut(stride)
            .enumerate()
            .map(move |(index, deltas)| TileValueChunk {
                seed,
                start: (index * stride) as u32,
                deltas,
                changed: 0,
            })
    }

    /// Applies the net change that the chunks reported.
    ///
    /// The sum is over integers, so it does not depend on the order the
    /// chunks are added in.
    pub fn absorb_changed(&mut self, delta: i64) {
        self.changed = self.changed.saturating_add_signed(delta as isize);
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
        let first = (start as usize).min(self.deltas.len());
        let last = (end as usize).clamp(first, self.deltas.len());
        TileValueRange {
            seed: self.seed,
            start,
            deltas: &self.deltas[first..last],
        }
    }
}
