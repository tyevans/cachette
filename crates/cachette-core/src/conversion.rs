//! The conversion of a unit from one faction to another.
//!
//! A unit that converts changes its faction. It keeps its identity, its slot
//! and its generation, so every structure that names the unit still names
//! it.[^1] Conversion adds no second allegiance value beside the faction, so
//! the presence relation, the ground a unit holds and a meeting between two
//! factions all read the change with no further work.[^2]
//!
//! **Belief is the influence field.** A faction reaches a level 1 cell
//! because the control plane put a source there and the solve spread it. The
//! pass converts a unit where another faction reaches its cell more strongly
//! than its own faction does.[^3] [^4]
//!
//! **The pass takes one keyed draw for each tile, and never one for each
//! unit.** The count that converts is exact arithmetic on the margin, and the
//! draws name which units they are. A draw for each unit would give each unit
//! an independent chance, and the number that converted would then vary
//! around the count the margin paid for.[^5] [^6]
//!
//! **Strict dominance is what stops a unit flipping every frame.** A unit
//! converts to the faction that leads at its cell. After the change that
//! faction is its own, so the margin against it is zero or less and the unit
//! cannot convert again while the field stands still.[^3]
//!
//! Every value here is an exact integer. No value is a floating point
//! number, and every arithmetic step goes through the arithmetic module.[^7]
//! [^8]
//!
//! # References
//!
//! [^1]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
//! [^2]: ADR-0132, conversion changes the faction of a unit and adds no second allegiance, decision D1. `docs/adrs/draft/adr-0132-conversion-changes-the-faction-of-a-unit.md`
//! [^3]: ADR-0133, a unit converts to the faction that leads the influence field at its cell, decision D1. `docs/adrs/draft/adr-0133-a-unit-converts-to-the-faction-that-leads-the-field.md`
//! [^4]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
//! [^5]: ADR-0133, a unit converts to the faction that leads the influence field at its cell, decision D3. `docs/adrs/draft/adr-0133-a-unit-converts-to-the-faction-that-leads-the-field.md`
//! [^6]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
//! [^7]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^8]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`

use bytemuck::{Pod, Zeroable};

use crate::bridge::{BlockLayout, UnitTileBridge};
use crate::influence::{cell_of_tile, Influence, InfluenceField};
use crate::relation::RelationMatrix;
use crate::rng;
use crate::sim_math;
use crate::slots::Slots;
use crate::soldier::SoldierArena;
use crate::types::{Accum, Entity, FactionId, Tick, TileIdx, FACTION_CEILING};

/// The number of draws that one losing group takes in one frame.
///
/// A group takes one draw for the rotation that names the converts, and one
/// for the remainder that covers no whole unit.[^1]
///
/// # References
///
/// [^1]: ADR-0133, a unit converts to the faction that leads the influence field at its cell, decision D3. `docs/adrs/draft/adr-0133-a-unit-converts-to-the-faction-that-leads-the-field.md`
const DRAWS_FOR_EACH_GROUP: u32 = 2;

/// The draw index offset of the rotation that names the converts.
const DRAW_ROTATION: u32 = 0;

/// The draw index offset of the remainder that covers no whole unit.
const DRAW_REMAINDER: u32 = 1;

/// Returns the draw index of one losing group.
///
/// **The index names the faction that loses the unit, and never the position
/// of the group inside the tile.** A position depends on who else stands
/// there, so an index taken from it would change the draw when an unrelated
/// unit arrived. The faction is a property of the group itself.[^1]
///
/// The gaining faction is not in the index, because one tile has exactly one
/// leader and two groups of one tile therefore never differ by it.
///
/// # References
///
/// [^1]: ADR-0133, a unit converts to the faction that leads the influence field at its cell, decision D3. `docs/adrs/draft/adr-0133-a-unit-converts-to-the-faction-that-leads-the-field.md`
const fn draw_index(faction: FactionId, which: u32) -> u32 {
    (faction.0 as u32) * DRAWS_FOR_EACH_GROUP + which
}

/// Returns the largest draw index that this pass ever takes.
///
/// The index packs the faction of a losing group. A test asserts against
/// this, so that a wider faction ceiling cannot silently make two groups
/// share one index.
#[must_use]
pub const fn draw_index_ceiling() -> u32 {
    (FACTION_CEILING as u32) * DRAWS_FOR_EACH_GROUP
}

/// The reason that the pass refused a caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConversionError {
    /// The caller asked for zero threads. A pass needs at least one.
    ZeroThreads,
    /// The soldier columns hold different lengths.
    ColumnsDisagree,
}

impl core::fmt::Display for ConversionError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ZeroThreads => write!(formatter, "a conversion pass needs at least one thread"),
            Self::ColumnsDisagree => {
                write!(formatter, "the soldier columns hold different lengths")
            }
        }
    }
}

impl std::error::Error for ConversionError {}

/// A unit changed faction.
///
/// The event reports one unit that changed hands. A watcher reads the log to
/// see where belief moved and who lost people to it. A mechanic that a player
/// cannot observe is a mechanic that a player cannot play.[^2]
///
/// The layout is 8 + 8 + 4 + 2 + 2 bytes, which is 24 bytes at an alignment
/// of 8. The fields fill the type exactly, so it declares no padding byte and
/// holds no uninitialised byte.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
/// [^2]: ADR-0134, a god reads conversion as an event log and as the faction counts it already reads, decision D1. `docs/adrs/draft/adr-0134-a-god-reads-conversion-as-an-event-log.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct UnitConverted {
    /// The tick at which the unit changed faction.
    pub tick: Tick,
    /// The unit that changed faction, as its identity in bits.
    pub unit: u64,
    /// The tile that the unit stood on.
    pub tile: TileIdx,
    /// The faction that lost the unit.
    pub from: FactionId,
    /// The faction that gained the unit.
    pub to: FactionId,
}

impl UnitConverted {
    /// Builds an event.
    #[must_use]
    pub const fn new(tick: Tick, unit: u64, tile: TileIdx, from: FactionId, to: FactionId) -> Self {
        Self {
            tick,
            unit,
            tile,
            from,
            to,
        }
    }
}

/// One unit that the pass marked, and the faction it goes to.
///
/// The slot is the key that the caller orders on. A unit stands on one tile,
/// and one thread owns that tile, so no two marks name one slot.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Convert {
    /// The arena slot of the unit.
    pub slot: u32,
    /// The faction that the unit goes to.
    pub faction: FactionId,
}

/// The key that every draw of this pass is taken on.
///
/// The tuple is the system, the frame, the tile and the draw index. The
/// system is this pass alone, so no other pass draws the same number from the
/// same frame and tile.[^1]
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawKey {
    /// The seed of the world.
    pub seed: u64,
    /// The frame the pass runs in.
    pub tick: Tick,
}

/// Returns the draw that decides whether the remainder converts one more
/// unit.
///
/// **The key is the seed, the frame, the tile and the group.** The pass calls
/// this, and a test calls it to prove that each field of the key reaches the
/// value. A draw keyed on the wrong field gives the same wrong answer on
/// every run, and neither determinism test can see that.[^1] [^2]
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
/// [^2]: Testing rules, section 2. `.claude/rules/testing.md`
#[must_use]
pub fn remainder_draw(key: DrawKey, tile: TileIdx, faction: FactionId) -> u64 {
    rng::draw_below(
        key.seed,
        rng::SYSTEM_CONVERSION,
        key.tick.0,
        u64::from(tile.0),
        draw_index(faction, DRAW_REMAINDER),
        u64::from(Influence::UNIT.0),
    )
}

/// Returns the offset that rotates the ordinals of one group.
///
/// **The key is the seed, the frame, the tile and the group.** The pass calls
/// this, and a test calls it to prove that each field of the key reaches the
/// value.[^1] [^2]
///
/// # References
///
/// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
/// [^2]: Testing rules, section 2. `.claude/rules/testing.md`
#[must_use]
pub fn rotation_offset(key: DrawKey, tile: TileIdx, faction: FactionId, count: u32) -> u64 {
    rng::draw_below(
        key.seed,
        rng::SYSTEM_CONVERSION,
        key.tick.0,
        u64::from(tile.0),
        draw_index(faction, DRAW_ROTATION),
        u64::from(count),
    )
}

/// Returns how many units of a group the margin converts.
///
/// The margin is how far the leading faction is above the faction of the
/// group, against one reference unit of influence. The product of the margin
/// and the headcount, divided by that reference, is the count the margin
/// pays for. The floor is what it certainly converts, and one keyed draw
/// decides whether the remainder converts one more.[^1]
///
/// The result never exceeds the headcount, because a margin never exceeds
/// one reference unit.
///
/// # References
///
/// [^1]: ADR-0133, a unit converts to the faction that leads the influence field at its cell, decision D3. `docs/adrs/draft/adr-0133-a-unit-converts-to-the-faction-that-leads-the-field.md`
#[must_use]
pub fn converts(margin: u16, present: u32, remainder_draw: u64) -> u32 {
    if margin == 0 || present == 0 {
        return 0;
    }
    let total = Accum(i64::from(present));
    let part = Accum(i64::from(margin));
    let whole = Accum(i64::from(Influence::UNIT.0));
    // The floor and the remainder come from one pair of operations over one
    // set of inputs, so the two cannot disagree about what was left over.
    let Some(taken) = sim_math::share(total, part, whole) else {
        return 0;
    };
    let Some(remainder) = sim_math::share_remainder(total, part, whole) else {
        return 0;
    };
    let taken = taken.0.clamp(0, i64::from(present)) as u32;
    if taken >= present {
        return present;
    }
    // The draw is uniform below the reference unit, so the fraction it passes
    // is the remainder itself. The expected count is therefore the product
    // exactly, and no rounding rule has to hold it up.
    taken + u32::from(remainder_draw < remainder.0.max(0) as u64)
}

/// Marks every unit that changes faction this frame.
///
/// The pass marks. It changes nothing. The caller applies the marks in
/// ascending slot order, after the pass, so the changes never follow a
/// thread.[^1]
///
/// **Each thread owns its own list, and the join sorts on the slot.** A unit
/// stands on one tile and one thread owns that tile, so the slots are
/// distinct and the sorted list is the same at any thread count.[^1] [^2]
///
/// The walk is over the blocks of the derived unit structure, in block order.
/// Inside a block the units lie in tile order, so a run of equal keys is the
/// units of one tile.[^3]
///
/// # Errors
///
/// Returns an error when the caller asks for zero threads, and when the
/// soldier columns hold different lengths.
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
/// [^2]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
/// [^3]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D2. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
#[allow(clippy::too_many_arguments)]
pub fn resolve(
    key: DrawKey,
    relations: &RelationMatrix,
    arena: &SoldierArena,
    bridge: &UnitTileBridge,
    field: &InfluenceField,
    layout: BlockLayout,
    marks: &mut Vec<Convert>,
    threads: usize,
) -> Result<(), ConversionError> {
    marks.clear();
    if threads == 0 {
        return Err(ConversionError::ZeroThreads);
    }
    let slots = arena.faction_column().len();
    if arena.tile_column().len() != slots {
        return Err(ConversionError::ColumnsDisagree);
    }
    if slots == 0 || field.faction_count() == 0 {
        return Ok(());
    }

    let blocks = bridge.layout().block_count();
    let block_chunk = (blocks as usize).div_ceil(threads).max(1);
    let mut lists: Slots<Vec<Convert>> =
        Slots::filled(threads, Vec::new()).map_err(|_| ConversionError::ZeroThreads)?;

    std::thread::scope(|scope| {
        let mut first = 0u32;
        for entry in lists.entries_mut() {
            let start = first;
            let stop = (start as usize + block_chunk).min(blocks as usize) as u32;
            first = stop;
            scope.spawn(move || {
                let mut reader = CellReader::new(field, layout);
                let mut groups = Vec::new();
                for block in start..stop {
                    resolve_block(
                        key,
                        relations,
                        arena,
                        bridge,
                        block,
                        &mut reader,
                        &mut groups,
                        entry,
                    );
                }
            });
        }
    });

    for entry in lists.entries() {
        marks.extend_from_slice(entry);
    }
    // The slot is a total order over the marks, because no two marks name one
    // slot. The sorted list therefore does not depend on which thread filled
    // which part of it.[^1]
    marks.sort_unstable_by_key(|mark| mark.slot);
    debug_assert!(
        marks.windows(2).all(|pair| pair[0].slot < pair[1].slot),
        "two marks name one slot"
    );
    Ok(())
}

/// Reads what every faction holds at one cell, and remembers the last cell.
///
/// The tiles of one block lie together, and a cell covers a whole block of
/// tiles, so consecutive tiles usually share one cell. The reader holds the
/// last answer, so the pass reads the field once for each cell it meets
/// rather than once for each tile.
struct CellReader<'a> {
    field: &'a InfluenceField,
    /// The level 1 block layout. It names the cell that covers a tile.
    layout: BlockLayout,
    /// The tile whose cell the buffer holds.
    held: Option<crate::hex::Axial>,
    /// What each faction holds at that cell.
    reach: [Influence; FACTION_CEILING as usize],
    /// The faction that leads at that cell.
    leader: FactionId,
}

impl<'a> CellReader<'a> {
    fn new(field: &'a InfluenceField, layout: BlockLayout) -> Self {
        Self {
            field,
            layout,
            held: None,
            reach: [Influence::ZERO; FACTION_CEILING as usize],
            leader: FactionId(0),
        }
    }

    /// Reads the cell that covers one tile, and returns whether it read one.
    ///
    /// Returns `false` when the tile lies outside the lattice, which the
    /// arena invariant already forbids for a live unit.
    fn read(&mut self, tile: TileIdx) -> bool {
        let Some(address) = cell_of_tile(self.layout, self.field.cells(), tile) else {
            return false;
        };
        if self.held == Some(address) {
            return true;
        }
        let mut leader = FactionId(0);
        let mut best = Influence::ZERO;
        for faction in 0..self.field.faction_count() {
            let at = self
                .field
                .at(FactionId(faction), address)
                .unwrap_or(Influence::ZERO);
            self.reach[faction as usize] = at;
            // The comparison is strict, so the first faction of a tie leads.
            // The tie therefore breaks on the faction number, which is a
            // stable key and never a thread order.[^1]
            //
            // [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
            if at > best {
                best = at;
                leader = FactionId(faction);
            }
        }
        for faction in self.field.faction_count()..FACTION_CEILING {
            self.reach[faction as usize] = Influence::ZERO;
        }
        self.held = Some(address);
        self.leader = leader;
        true
    }
}

/// Converts the units of one block.
#[allow(clippy::too_many_arguments)]
fn resolve_block(
    key: DrawKey,
    relations: &RelationMatrix,
    arena: &SoldierArena,
    bridge: &UnitTileBridge,
    block: u32,
    reader: &mut CellReader<'_>,
    groups: &mut Vec<(FactionId, u32)>,
    marks: &mut Vec<Convert>,
) {
    let (keys, units) = bridge.block_window(block);
    let factions = arena.faction_column();
    let tiles = arena.tile_column();
    let mut position = 0usize;
    while position < keys.len() {
        let mut end = position + 1;
        while end < keys.len() && keys[end] == keys[position] {
            end += 1;
        }
        let tile_units = &units[position..end];
        position = end;
        let tile = tiles[tile_units[0].index() as usize];
        if !reader.read(tile) {
            continue;
        }
        let leader = reader.leader;
        let lead = reader.reach[leader.0 as usize];
        if lead == Influence::ZERO {
            // No faction reaches this cell at all, so no faction leads it and
            // nobody converts. This is the ordinary case in a world where the
            // control plane set no source.
            continue;
        }

        groups.clear();
        for unit in tile_units {
            let faction = factions[unit.index() as usize];
            match groups.iter_mut().find(|group| group.0 == faction) {
                Some(group) => group.1 += 1,
                None => groups.push((faction, 1)),
            }
        }

        for (faction, count) in groups.iter().copied() {
            if faction == leader {
                continue;
            }
            // **A leader at peace with a faction converts none of its
            // units.** The relation adds one condition to the field, and the
            // edge it compares against is a register row.[^2]
            //
            // [^2]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D4. `docs/adrs/draft/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
            if !relations.permits_conversion(leader, faction) {
                continue;
            }
            let held = reader.reach[faction.0 as usize];
            // **The comparison is strict.** A unit converts only where
            // another faction leads its own, so the unit cannot convert back
            // on the next frame while the field stands still.[^1]
            //
            // [^1]: ADR-0133, a unit converts to the faction that leads the influence field at its cell, decision D2. `docs/adrs/draft/adr-0133-a-unit-converts-to-the-faction-that-leads-the-field.md`
            if lead <= held {
                continue;
            }
            let margin = lead.0 - held.0;
            let remainder_draw = remainder_draw(key, tile, faction);
            let taken = converts(margin, count, remainder_draw);
            if taken == 0 {
                continue;
            }
            // **The subset is the ordinals of the group, rotated by a keyed
            // offset.** A rotation is a bijection, so exactly as many units
            // convert as the margin paid for. A draw for each unit would give
            // each unit an independent chance, and the count would then vary.
            let offset = rotation_offset(key, tile, faction, count);
            let mut ordinal = 0u64;
            for unit in tile_units {
                let slot = unit.index();
                if factions[slot as usize] != faction {
                    continue;
                }
                let place = (ordinal + offset) % u64::from(count);
                if place < u64::from(taken) {
                    marks.push(Convert {
                        slot,
                        faction: leader,
                    });
                }
                ordinal += 1;
            }
        }
    }
}

/// Returns the marks that a named set of units produces for one faction.
///
/// This is the verb side. The control plane names a set and a faction, and
/// every unit of the set that does not already belong to that faction
/// changes hands. The engine holds no rule that decides when a god may do
/// this.[^1]
///
/// The marks come back in ascending slot order, which is the order the caller
/// applies them in.[^2]
///
/// # References
///
/// [^1]: ADR-0133, a unit converts to the faction that leads the influence field at its cell, decision D4. `docs/adrs/draft/adr-0133-a-unit-converts-to-the-faction-that-leads-the-field.md`
/// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
#[must_use]
pub fn marks_for_set(arena: &SoldierArena, units: &[Entity], faction: FactionId) -> Vec<Convert> {
    let mut marks: Vec<Convert> = Vec::new();
    for unit in units {
        let Some(slot) = arena.slot_of(*unit) else {
            continue;
        };
        if arena.faction_column()[slot as usize] == faction {
            continue;
        }
        marks.push(Convert { slot, faction });
    }
    marks.sort_unstable_by_key(|mark| mark.slot);
    marks.dedup_by_key(|mark| mark.slot);
    marks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_draw_index_of_two_groups_never_collides() {
        let mut seen = std::collections::BTreeSet::new();
        for faction in 0..FACTION_CEILING {
            for which in [DRAW_ROTATION, DRAW_REMAINDER] {
                let index = draw_index(FactionId(faction), which);
                assert!(index < draw_index_ceiling());
                assert!(seen.insert(index), "two groups share the draw index");
            }
        }
    }

    #[test]
    fn no_margin_converts_nobody_at_any_draw() {
        for draw in [0u64, 1, u64::from(Influence::UNIT.0) - 1] {
            assert_eq!(converts(0, 1_000_000, draw), 0);
        }
    }

    #[test]
    fn the_whole_margin_converts_the_whole_group() {
        for count in [1u32, 7, 4096, 1_000_000] {
            assert_eq!(converts(Influence::UNIT.0, count, 0), count);
        }
    }

    #[test]
    fn the_count_never_exceeds_the_group_at_any_margin() {
        for margin in [1u16, 2, 4096, 30_000, u16::MAX - 1, u16::MAX] {
            for count in [1u32, 3, 64, 65_535, 1_000_000] {
                assert!(converts(margin, count, 0) <= count);
                assert!(converts(margin, count, u64::from(Influence::UNIT.0) - 1) <= count);
            }
        }
    }

    #[test]
    fn the_remainder_draw_decides_the_last_unit_and_nothing_else() {
        // One unit and half the reference unit of margin: the floor is zero
        // and the remainder is half, so the draw alone decides.
        let half = Influence::UNIT.0 / 2;
        assert_eq!(converts(half, 1, 0), 1);
        assert_eq!(converts(half, 1, u64::from(Influence::UNIT.0) - 1), 0);
    }
}
