//! The sort-then-admit pass that decides which move into a tile stands.
//!
//! Several units can ask for one tile in one tick, and the tile holds a
//! capacity. The pass sorts the requests by a stable key, then admits them in
//! that order until the tile is full. The tile account it keeps while it works
//! sits here with it.

use super::errors::StepError;
use crate::bridge::UnitTileBridge;
use crate::hex::{Axial, Grid};
use crate::holding::Holder;
use crate::relation::RelationMatrix;
use crate::soldier::SoldierArena;
#[cfg(not(feature = "probe-nondeterminism"))]
use crate::sort;
use crate::sort::{BoundedKey, SortError};
use crate::terrain::Terrain;
use crate::types::{Entity, TileIdx};
use crate::upgrade::{self, UpgradeMap, UpgradeTable};

/// The number of admission passes that one frame runs.
///
/// Each pass admits what it can against the room the previous pass
/// confirmed. The engine never runs to a fixpoint, because a fixpoint needs
/// a convergence test and a solver in this project runs a fixed count.[^1]
///
/// The count is content. It is declared here until content exists, and the
/// register holds the open choice of its value.[^2]
///
/// One pass admits no chain: a unit cannot follow another out of a full
/// tile in the same frame. Two passes admit a chain of two. A longer chain
/// waits for the next frame, which is a delay and never a wrong answer.
///
/// # References
///
/// [^1]: ADR-0005, a solver runs a fixed iteration count, decision D1. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
/// [^2]: Decisions register, DEC-019. `docs/DECISIONS.md`
const ADMISSION_PASSES: usize = 2;

/// A running count for each tile that the admission touched.
///
/// The tiles are held sorted by index, so a lookup is a binary search and the
/// order never depends on how the counts were gathered. A dense array over
/// every tile would be faster to update and would cost the whole world in
/// memory for a frame that touches a handful of tiles.[^1]
///
/// A count is merged in ascending runs, never inserted one at a time.
/// Inserting into the middle of a vector moves every later entry, which is
/// quadratic in the number of tiles the frame touches, and the target scale
/// is a million units.[^2]
///
/// # References
///
/// [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
/// [^2]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
#[derive(Debug, Default)]
struct TileCounts {
    entries: Vec<(u32, u32)>,
    scratch: Vec<(u32, u32)>,
}

impl TileCounts {
    /// Returns the count that one tile carries, for a caller that asks in
    /// ascending tile order.
    ///
    /// **The caller must ask for tiles in ascending order and must not change
    /// the table between two asks.** The position only moves forward, so a
    /// tile below the last one asked for reads zero rather than its count. The
    /// debug assertion below is what states that, and
    /// `a_forward_reader_agrees_with_the_search` is what proves it.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    fn read_ascending(&self, at: &mut usize, tile: u32) -> u32 {
        while *at < self.entries.len() && self.entries[*at].0 < tile {
            *at += 1;
        }
        if *at < self.entries.len() && self.entries[*at].0 == tile {
            self.entries[*at].1
        } else {
            0
        }
    }

    /// Returns the count that one tile carries, by searching for it.
    ///
    /// **Nothing in the engine calls this.** It is the independent answer that
    /// `a_forward_reader_agrees_with_the_search` compares the forward reader
    /// against, and it is kept because a reader with nothing to disagree with
    /// proves nothing.[^1]
    ///
    /// # References
    ///
    /// [^1]: Testing Rules, section 2. `.claude/rules/testing.md`
    #[cfg(test)]
    fn get(&self, tile: u32) -> u32 {
        match self.entries.binary_search_by_key(&tile, |(key, _)| *key) {
            Ok(at) => self.entries[at].1,
            Err(_) => 0,
        }
    }

    /// Adds a run of counts, given in ascending tile order.
    ///
    /// The caller states the order and the merge relies on it. A run out of
    /// order would silently produce an unsorted result, and every later
    /// lookup would then read the wrong tile, so the merge asserts it.
    fn merge_ascending(&mut self, run: &[(u32, u32)]) {
        debug_assert!(
            run.windows(2).all(|pair| pair[0].0 < pair[1].0),
            "a merged run must be sorted by tile and hold each tile once"
        );
        if run.is_empty() {
            return;
        }
        self.scratch.clear();
        self.scratch.reserve(self.entries.len() + run.len());
        let (mut here, mut there) = (0usize, 0usize);
        while here < self.entries.len() && there < run.len() {
            let (mine, theirs) = (self.entries[here], run[there]);
            if mine.0 < theirs.0 {
                self.scratch.push(mine);
                here += 1;
            } else if theirs.0 < mine.0 {
                self.scratch.push(theirs);
                there += 1;
            } else {
                self.scratch.push((mine.0, mine.1 + theirs.1));
                here += 1;
                there += 1;
            }
        }
        self.scratch.extend_from_slice(&self.entries[here..]);
        self.scratch.extend_from_slice(&run[there..]);
        core::mem::swap(&mut self.entries, &mut self.scratch);
    }
}

/// Returns the order in which admission reads the intents.
///
/// The order is the key vector sort: by target tile, then by the identity of
/// the unit.[^1] It depends on the key values alone, so it is the same at any
/// thread count.[^2]
///
/// # Errors
///
/// Returns an error when the sort refuses the keys.
///
/// # References
///
/// [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
/// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D5. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
#[cfg(not(feature = "probe-nondeterminism"))]
fn admission_order(keys: &[BoundedKey], ceiling: u64) -> Result<Vec<u32>, SortError> {
    sort::order_bounded(keys, ceiling)
}

/// Returns the intents in the order they arrived, which is a defect.
///
/// This is the perturbed build. Admission reads the joined intent list rather
/// than the sorted one, so who enters a full tile depends on the order the
/// slots were joined in. The slot probe reverses that order, and the reversal
/// is visible only above one thread, so the thread-count test then fails.
///
/// The whole point is that it must fail. A determinism test with no proven
/// failure mode is decoration.[^1]
///
/// # Errors
///
/// Never. The signature matches the sound build so that the caller does not
/// change.
///
/// # References
///
/// [^1]: Testing rules, section 1. `.claude/rules/testing.md`
#[cfg(feature = "probe-nondeterminism")]
fn admission_order(keys: &[BoundedKey], _ceiling: u64) -> Result<Vec<u32>, SortError> {
    // A stable sort by the target alone. Each target still owns one
    // contiguous segment, which admission requires to scan a segment at all,
    // and within a segment the order is the order the intents arrived in.
    let mut order: Vec<u32> = (0..keys.len() as u32).collect();
    order.sort_by_key(|position| keys[*position as usize].order());
    Ok(order)
}

/// One target tile and the run of intents that name it.
///
/// The table is built once for a frame. The capacity and the occupancy are
/// read once for each target rather than once for each pass, because the
/// ground is computed on demand and reading it twice computes it twice.[^1]
///
/// # References
///
/// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
#[derive(Clone, Copy, Debug)]
struct Segment {
    /// The target tile that owns the segment.
    tile: u32,
    /// The first sorted position of the segment.
    start: usize,
    /// One past the last sorted position of the segment.
    end: usize,
    /// The units the ground of the target admits.
    capacity: u32,
    /// The units that stood on the target at the last barrier.
    standing: u32,
}

/// Adds one to the last entry of an ascending run, or starts a new one.
///
/// The caller visits the tiles in ascending order, so a repeat is always the
/// last entry.
fn bump(run: &mut Vec<(u32, u32)>, tile: u32) {
    match run.last_mut() {
        Some(last) if last.0 == tile => last.1 += 1,
        _ => run.push((tile, 1)),
    }
}

/// Returns the intents that admission granted, in the sorted admission order.
///
/// Admission sorts the intents by target tile, then by the identity of the
/// unit. Each target tile then owns one contiguous segment, and the identity
/// is the final key field so no two intents tie.[^1] The sort is the engine's
/// key vector sort, and it runs on one thread, so no result here takes its
/// order from a thread that finished first.[^2]
///
/// Admission scans each segment in its sorted order and admits until the
/// target reaches the capacity of its ground. The capacity comes from the
/// terrain table. This function holds no capacity value of its own.[^3]
///
/// **The occupancy of a target comes from the derived structure**, which the
/// barrier rebuilt before the intents were drawn.[^4] Admission carries no
/// dense array over every tile.
///
/// **Only an admitted departure releases room.** An intent is not a
/// departure. A unit that intends to leave and is then rejected at its own
/// target has not left, and the room it appeared to release was never
/// released. Take three tiles in a line, with the middle and the far tile
/// both full. The unit in the middle is rejected at the far tile. A rule that
/// counted its intent would admit the unit behind it into the middle tile,
/// and the middle tile would end the tick above its capacity.[^1]
///
/// That failure is deterministic, so neither determinism test can see it.
/// Only a test that asserts the capacity invariant can.[^5]
///
/// **A departure is applied after the scan, not inside it.** The segments are
/// disjoint by target, so the room of a target is read once for each segment.
/// The units leaving one tile are scattered across many segments, because
/// they chose different targets, so the departures are a separate reduction
/// over the admitted set, keyed on the source tile.[^1]
///
/// # Errors
///
/// Returns an error when the sort refuses the keys, or when the derived
/// structure cannot answer for a tile.
///
/// # References
///
/// [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
/// [^2]: ADR-0007, content supplies a key vector, never a comparator, decision D2. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
/// [^3]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
/// [^4]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
/// [^5]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
/// [^6]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
/// [^7]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
#[allow(clippy::too_many_arguments)]
pub(super) fn admit(
    intents: &[(Entity, Axial)],
    soldiers: &SoldierArena,
    bridge: &UnitTileBridge,
    terrain: Terrain,
    upgrades: &UpgradeMap,
    table: &UpgradeTable,
    grid: Grid,
    guests: Guests<'_>,
    threads: usize,
) -> Result<Vec<(Entity, Axial)>, StepError> {
    if intents.is_empty() {
        return Ok(Vec::new());
    }

    // **A holder refuses a guest it is below the guest edge toward.** The
    // rule is stated once, in the relation module, and this pass asks it for
    // each intent. The walk is in intent order and it reads the holder column
    // as the last spread left it, so the answer is the same at any thread
    // count.[^11]
    //
    // [^11]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D4. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
    let refused: Vec<bool> = intents
        .iter()
        .map(|(entity, target)| {
            let Some(index) = grid.index_of(*target) else {
                return false;
            };
            let Some(holder) = guests
                .holders
                .get(index.0 as usize)
                .and_then(|holder| holder.faction())
            else {
                return false;
            };
            let Some(guest) = soldiers.faction(*entity) else {
                return false;
            };
            guests.relations.refuses_guest(holder, guest)
        })
        .collect();

    // The ordering field is the target tile index and the identifier is the
    // entity. One unit writes one intent, so no two identifiers collide.
    let mut keys = Vec::with_capacity(intents.len());
    for (entity, target) in intents {
        let index = grid
            .index_of(*target)
            .ok_or(StepError::TargetOutsideWorld)?;
        keys.push(BoundedKey::new(u64::from(index.0), entity.to_bits()));
    }
    let ceiling = u64::from(grid.tile_count().saturating_sub(1));
    let order = admission_order(&keys, ceiling)?;

    // The sorted intents, as a tile beside the intent it belongs to. The
    // passes walk this rather than following the permutation into the keys,
    // so a pass reads its tiles in order rather than at random.
    let sorted: Vec<(u32, u32)> = order
        .iter()
        .map(|position| (keys[*position as usize].order() as u32, *position))
        .collect();

    // The segment table, built once. Each target owns one contiguous segment,
    // and the segments are disjoint.[^1]
    //
    // The capacity and the occupancy are read here and not inside the passes.
    // The ground is a pure function of the seed and the address, so reading
    // it twice computes it twice, and the record calls a repeated sweep of
    // the ground a design mistake.[^6] The occupancy comes from the structure
    // the last barrier built, and that answer does not change during the
    // frame either: what changes is the arrivals and the departures this
    // admission grants, and those are counted separately.
    let mut segments: Vec<Segment> = Vec::new();
    let mut at = 0usize;
    while at < sorted.len() {
        let tile = sorted[at].0;
        let mut end = at;
        while end < sorted.len() && sorted[end].0 == tile {
            end += 1;
        }
        segments.push(Segment {
            tile,
            start: at,
            end,
            capacity: 0,
            standing: 0,
        });
        at = end;
    }

    // The capacity and the occupancy of each target are read in parallel.
    // Each thread writes its own chunk of the table, and a chunk is named by
    // its position in the table rather than by the thread that filled it, so
    // the result never depends on which thread finished first.[^7]
    // The freshness of the derived structure is established once, here, rather
    // than on every tile the fill asks about. The borrow of the arena is what
    // stops the world changing while the fill walks it.
    bridge.describes(soldiers)?;
    let chunk_len = segments.len().div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for chunk in segments.chunks_mut(chunk_len) {
            handles.push(scope.spawn(move || {
                // The segments of a chunk are in ascending tile order, so one
                // reader walks the block rather than searching it for each
                // segment. A reader is per-thread, because it carries the
                // position of its own walk.[^9]
                //
                // [^9]: Findings register, FND-295. `docs/FINDINGS.md`
                let mut cursor = bridge.tile_cursor();
                for segment in chunk.iter_mut() {
                    let Some(address) = grid.address_of(TileIdx(segment.tile)) else {
                        // The tile came from a key the sort built out of a
                        // grid index, so it names a tile. An address that
                        // does not resolve is a defect in the caller and the
                        // whole step refuses below.
                        continue;
                    };
                    // The ground states the capacity and a finished
                    // upgrade adds to it. One function answers the whole
                    // question, so admission cannot read the ground table
                    // without the upgrade table.[^8]
                    //
                    // [^8]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D3. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
                    // **The room a target holds is the room it holds for
                    // the units that asked for it.** Every intent in a
                    // segment came through the step gate above, so a segment
                    // on open water holds crossing units alone and a segment
                    // on any other ground answers the same capacity either
                    // way. The pass therefore asks the table for the crossing
                    // capacity, and it states no rule of its own about which
                    // ground admits whom.[^12]
                    //
                    // [^12]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
                    let ground = terrain.kind(address).map_or(0, |kind| {
                        kind.capacity_for(crate::terrain::SOME_WATER_CROSSING)
                    });
                    segment.capacity = upgrade::capacity_with(
                        ground,
                        upgrades.standing(TileIdx(segment.tile), table),
                    );
                    segment.standing = bridge
                        .units_on_tile(&mut cursor, TileIdx(segment.tile))
                        .len() as u32;
                }
            }));
        }
        for handle in handles {
            // A thread here reads shared memory and writes its own chunk of
            // the table, so it has no failure of its own.
            handle.join().expect("an admission thread cannot fail");
        }
    });

    let mut granted = vec![false; intents.len()];
    let mut arrived = TileCounts::default();
    let mut departed = TileCounts::default();
    let mut admitted: Vec<(Entity, Axial)> = Vec::new();

    for _ in 0..ADMISSION_PASSES {
        let first = admitted.len();
        // The segments come in ascending tile order, so the arrivals of one
        // pass are already an ascending run.
        let mut arrivals: Vec<(u32, u32)> = Vec::new();

        // The segments are in ascending tile order and both count tables are
        // too, and neither table changes while this loop runs. One forward
        // reader for each therefore replaces a search for each segment.[^10]
        //
        // [^10]: Findings register, FND-300. `docs/FINDINGS.md`
        let mut standing_at = 0usize;
        let mut arrived_at = 0usize;
        for segment in &segments {
            // A departure only ever leaves a tile a unit stood on, so the
            // subtraction cannot go below zero. It saturates rather than
            // wrapping, because a wrap here would read as a full tile and
            // reject every unit in silence.
            let occupancy = segment
                .standing
                .saturating_sub(departed.read_ascending(&mut standing_at, segment.tile))
                + arrived.read_ascending(&mut arrived_at, segment.tile);
            let mut room = segment.capacity.saturating_sub(occupancy);
            if room == 0 {
                continue;
            }

            for (_, position) in &sorted[segment.start..segment.end] {
                if room == 0 {
                    break;
                }
                let position = *position as usize;
                if granted[position] || refused[position] {
                    continue;
                }
                granted[position] = true;
                admitted.push(intents[position]);
                bump(&mut arrivals, segment.tile);
                room -= 1;
            }
        }
        arrived.merge_ascending(&arrivals);

        // The scan is over. Only now does a departure release room, and only
        // an admitted one. The reduction is keyed on the source tile.
        let mut sources: Vec<u32> = admitted[first..]
            .iter()
            .filter_map(|(entity, _)| soldiers.tile(*entity))
            .map(|tile| tile.0)
            .collect();
        if sources.is_empty() {
            // Nothing moved in this pass, so no later pass can move anything.
            break;
        }
        sources.sort_unstable();
        let mut departures: Vec<(u32, u32)> = Vec::new();
        for tile in sources {
            bump(&mut departures, tile);
        }
        departed.merge_ascending(&departures);
    }

    Ok(admitted)
}

/// What admission reads to refuse a guest: who holds each tile, and what the
/// holder feels toward the guest.
#[derive(Clone, Copy)]
pub(super) struct Guests<'a> {
    pub(super) holders: &'a [Holder],
    pub(super) relations: &'a RelationMatrix,
}

#[cfg(test)]
mod tests {
    //! Unit tests for the tile account that the public API cannot reach.
    //!
    //! The account is private to this module, and no public reader reports
    //! it. A test of its two answers must therefore build one here.[^1]
    //!
    //! # References
    //!
    //! [^1]: Testing policy, section 2. `docs/TESTING.md`

    use super::*;

    /// The forward reader is a second way to answer what the search already
    /// answers. This drives both over a wide range of tiles, in the ascending
    /// order the reader requires, and compares them.[^1]
    ///
    /// The table is deliberately gappy, so the reader must step over tiles it
    /// holds no count for, and it holds the first and the last tile of the
    /// range so neither end is a special case.
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[test]
    fn a_forward_reader_agrees_with_the_search() {
        let mut counts = TileCounts::default();
        counts.merge_ascending(&[(0, 3), (1, 1), (7, 9), (8, 2), (40, 5), (99, 7)]);
        let mut at = 0usize;
        for tile in 0..100u32 {
            assert_eq!(
                counts.read_ascending(&mut at, tile),
                counts.get(tile),
                "the two answers disagree at tile {tile}"
            );
        }
        // The reader stays on its answer, so asking twice repeats it.
        let mut again = 0usize;
        assert_eq!(counts.read_ascending(&mut again, 7), 9);
        assert_eq!(counts.read_ascending(&mut again, 7), 9);
    }
}
