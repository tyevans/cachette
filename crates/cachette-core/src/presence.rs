//! The presence relation between factions.
//!
//! The relation answers one question. Does any unit of one faction stand on
//! ground that another faction holds? The engine holds every part of the
//! answer already: a tile carries one holder, and a unit carries a faction
//! and the tile it stands on.[^1] [^2] This module joins the two.
//!
//! # The shape
//!
//! **The relation is one mask row for each faction.** A record fixes that
//! shape for every relation between factions in this project, and this is the
//! first one built against it.[^3] A row is a set of factions, which is one
//! 64-bit word, so the whole relation is 63 words whatever the world holds.
//! A relation expressed over the tiles would be the world multiplied by the
//! faction count squared, which is not storable.[^3]
//!
//! **The row index is the faction that holds the ground. The bits name the
//! factions that stand on it.** So `rows()[host]` holds `guest` when a unit of
//! `guest` stands on a tile that `host` holds.
//!
//! **A unit standing on ground its own faction holds sets no bit.** The
//! question is whether the people of one side stand on the ground of another
//! side, so the diagonal of the relation is always empty.[^4] A caller that
//! wants to know how much ground a faction holds reads the running total
//! instead.[^1]
//!
//! **The relation is exact and never an over-approximation.** The fold visits
//! every live unit and reads the holder of the exact tile that unit stands
//! on. No block mask, no level 1 cell and no bounding shape reaches it. A set
//! bit therefore names a unit that is genuinely there, and a clear bit means
//! that no unit is.
//!
//! # Determinism
//!
//! **The combine is a union of sets.** It is associative, commutative and
//! exact, so a fold over the population gives one answer whatever the
//! partition and whatever the thread count.[^5] [^6]
//!
//! The fold still writes disjoint outputs and joins them in slot order,
//! because that is what this project's parallel rule asks for and because a
//! reader should not have to prove the commutativity again to review a
//! change.[^7] Each thread folds a contiguous run of arena slots into its own
//! row array. The join reads the slots in slot order. **Nothing reads which
//! thread finished first, and no result takes its order from a completion.**
//!
//! The partition comes from the slot count and the thread count, and never
//! from the schedule.[^7]
//!
//! # The stale read
//!
//! A caller that spawns, despawns or moves a unit and then reads the relation
//! would get an answer that looks correct and is not. Every read that names a
//! faction therefore takes the arena and refuses when the arena has changed
//! since the fold.[^8] The refusal reuses the error type of the unit-to-tile
//! bridge, because it is the same question about the same arena, and a second
//! error type stating the same three failures would be one fact declared
//! twice.[^9]
//!
//! # References
//!
//! [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
//! [^2]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
//! [^3]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D7. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
//! [^4]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decision D3. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
//! [^5]: ADR-0023, an aggregate combines exactly, in any order, decisions D1 and D2. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
//! [^6]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
//! [^7]: ADR-0009, parallel stages write disjoint outputs, decisions D1, D2 and D3. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
//! [^8]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decision D4. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
//! [^9]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`

use crate::bridge::BridgeError;
use crate::holding::{FactionMask, Holding};
use crate::slots::Slots;
use crate::soldier::SoldierArena;
use crate::types::{FactionId, FACTION_CEILING};

/// How many rows the relation holds.
///
/// One row for each addressable faction. The value is the faction ceiling and
/// not a second declaration of it.[^1]
///
/// # References
///
/// [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D1. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
pub const PRESENCE_ROWS: usize = FACTION_CEILING as usize;

/// Which factions stand on the ground of which other factions.
///
/// The relation holds one set of factions for each faction. Row `host` names
/// every faction that has a live unit standing on a tile that `host` holds.
/// The relation is derived at the end of a step and it is never stored as a
/// fact.[^1]
///
/// # References
///
/// [^1]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decision D1. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
#[derive(Clone, Debug)]
pub struct PresenceRelation {
    rows: [FactionMask; PRESENCE_ROWS],
    /// The arena revision that the last fold read.
    built: Option<u64>,
    /// The arena that the last fold read.
    source: Option<u64>,
}

impl Default for PresenceRelation {
    fn default() -> Self {
        Self::new()
    }
}

impl PresenceRelation {
    /// Builds an empty relation.
    ///
    /// The relation holds no answer until the first fold. A read before the
    /// first fold returns [`BridgeError::NeverBuilt`].
    #[must_use]
    pub const fn new() -> Self {
        Self {
            rows: [FactionMask::EMPTY; PRESENCE_ROWS],
            built: None,
            source: None,
        }
    }

    /// Derives the whole relation from the unit columns and the holder
    /// column.
    ///
    /// The engine calls this at the end of a step, after the last structural
    /// change and after the holding has spread. A fold placed before either
    /// one would answer for a world the step had already left.[^1]
    ///
    /// The fold visits every live slot once. It reads the faction and the
    /// tile of the unit, and the holder of that tile. It allocates one row
    /// array for each thread and nothing that follows the population.
    ///
    /// # Errors
    ///
    /// Returns [`BridgeError::GridMismatch`] when the arena and the holding
    /// describe different worlds.
    ///
    /// # References
    ///
    /// [^1]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decision D2. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
    pub fn rebuild(
        &mut self,
        arena: &SoldierArena,
        holding: &Holding,
        threads: usize,
    ) -> Result<(), BridgeError> {
        if arena.grid() != holding.grid() {
            return Err(BridgeError::GridMismatch);
        }

        let slot_count = arena.live_column().len();
        self.rows = if slot_count == 0 {
            [FactionMask::EMPTY; PRESENCE_ROWS]
        } else {
            let threads = threads.max(1);
            let chunk_len = slot_count.div_ceil(threads).max(1);
            let slot_total = slot_count.div_ceil(chunk_len);
            let mut slots: Slots<[FactionMask; PRESENCE_ROWS]> =
                Slots::filled(slot_total, [FactionMask::EMPTY; PRESENCE_ROWS])
                    .expect("the slot count is derived from a non-empty arena, so it is not zero");

            // The three columns are one array each, and a slot indexes all
            // three, so one chunk boundary partitions the whole population.
            let live = arena.live_column();
            let tiles = arena.tile_column();
            let factions = arena.faction_column();
            let holders = holding.holders();

            std::thread::scope(|scope| {
                let mut handles = Vec::new();
                let chunks = live
                    .chunks(chunk_len)
                    .zip(tiles.chunks(chunk_len))
                    .zip(factions.chunks(chunk_len));
                for (((live, tiles), factions), slot) in chunks.zip(slots.entries_mut()) {
                    handles.push(scope.spawn(move || {
                        let mut rows = [FactionMask::EMPTY; PRESENCE_ROWS];
                        for (index, mark) in live.iter().enumerate() {
                            if *mark == 0 {
                                continue;
                            }
                            let guest = factions[index];
                            let tile = tiles[index].0 as usize;
                            let Some(holder) = holders.get(tile).copied() else {
                                continue;
                            };
                            let Some(host) = holder.faction() else {
                                continue;
                            };
                            // A unit on ground its own faction holds is not
                            // a guest, so the diagonal stays empty.
                            if host == guest {
                                continue;
                            }
                            let Some(row) = rows.get_mut(host.0 as usize) else {
                                continue;
                            };
                            *row = row.union(FactionMask::of(guest));
                        }
                        *slot = rows;
                    }));
                }
                for handle in handles {
                    // A thread here reads shared columns and writes its own
                    // slot. It cannot fail.
                    handle.join().expect("a presence fold thread cannot fail");
                }
            });

            // The join reads the slots in slot order. The union is
            // commutative, so the order does not change the answer, and the
            // order is fixed anyway so that nothing here depends on a
            // completion.
            slots.combine(
                [FactionMask::EMPTY; PRESENCE_ROWS],
                |mut joined, slot: &[FactionMask; PRESENCE_ROWS]| {
                    for (row, part) in joined.iter_mut().zip(slot.iter()) {
                        *row = row.union(*part);
                    }
                    joined
                },
            )
        };

        self.built = Some(arena.revision());
        self.source = Some(arena.identity());
        Ok(())
    }

    /// Returns every row of the relation, indexed by the faction that holds
    /// the ground.
    ///
    /// This is a guarded read. It names factions, so it takes the arena and
    /// refuses when the relation no longer describes it.
    ///
    /// # Errors
    ///
    /// Returns an error when the relation was never derived, when it was
    /// derived from another arena, or when that arena has changed since.
    pub fn rows(&self, arena: &SoldierArena) -> Result<&[FactionMask], BridgeError> {
        self.check_fresh(arena)?;
        Ok(&self.rows)
    }

    /// Reports whether a unit of `guest` stands on ground that `host` holds.
    ///
    /// Returns `false` when either faction is outside the addressable set,
    /// because no such faction can hold ground or stand on it.
    ///
    /// This is a guarded read.
    ///
    /// # Errors
    ///
    /// Returns an error when the relation was never derived, when it was
    /// derived from another arena, or when that arena has changed since.
    pub fn stands_in(
        &self,
        arena: &SoldierArena,
        guest: FactionId,
        host: FactionId,
    ) -> Result<bool, BridgeError> {
        self.check_fresh(arena)?;
        Ok(self
            .rows
            .get(host.0 as usize)
            .is_some_and(|row| row.contains(guest)))
    }

    /// Reports whether the relation still describes the arena.
    ///
    /// # Errors
    ///
    /// Returns an error when the relation was never derived, when it was
    /// derived from another arena, or when that arena has changed since.
    pub fn describes(&self, arena: &SoldierArena) -> Result<(), BridgeError> {
        self.check_fresh(arena)
    }

    /// Fails when the arena has changed since the fold.
    ///
    /// The revision counts changes and does not name the arena, so the
    /// identity is checked as well. Two arenas of one extent, each holding
    /// one unit, both sit at revision one.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    fn check_fresh(&self, arena: &SoldierArena) -> Result<(), BridgeError> {
        let source = self.source.ok_or(BridgeError::NeverBuilt)?;
        if source != arena.identity() {
            return Err(BridgeError::WrongArena);
        }
        let built = self.built.ok_or(BridgeError::NeverBuilt)?;
        if built == arena.revision() {
            Ok(())
        } else {
            Err(BridgeError::Stale {
                built,
                current: arena.revision(),
            })
        }
    }
}
