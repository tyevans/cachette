//! The resolution of a meeting between two factions.
//!
//! Two factions stand on neighbouring ground. This pass decides how many units
//! of each side fall, and which ones.
//!
//! **The fight resolves at the tile.** A level 1 cell summarises a whole block
//! of tiles, and a fight resolved there kills units spread over all of
//! them.[^1] The derived unit structure already lists the units standing on
//! one tile, so the input the tile form needs exists at every barrier.[^2]
//!
//! **Contact is adjacency, and never co-occupation alone.** Admission refuses
//! a step onto a tile that stands at its capacity, and it reads the capacity
//! rather than the faction, so an army that fills a tile could never be
//! entered and would never be reached.[^6] A unit of one faction therefore
//! reaches every unit of another faction on its own tile and on the six
//! tiles beside it.
//!
//! **The pass reads a table for each ordered pair of unit types.** It never
//! loops over pairs of units. The cost of one tile therefore follows the
//! square of the type count, which is small and fixed, and never the
//! population of the tile.[^1]
//!
//! **An attacker whose attack does not exceed the defender's armour
//! contributes exactly zero.** The threshold applies for each attacker type
//! before anything is aggregated, so a sum of zeroes stays zero at any
//! count.[^3]
//!
//! **One keyed draw serves a whole group.** The pass computes the expected
//! casualties exactly in the fixed-point scale, floors them into a whole
//! count, and names the units by a keyed rotation of the defenders. A draw
//! for each unit is what this avoids.[^4]
//!
//! Every value here is an integer or a fixed-point value. No value is a
//! floating point number.[^5]
//!
//! # References
//!
//! [^1]: ADR-0121, a meeting between two factions resolves at the tile, decisions D1 and D3. `docs/adrs/draft/adr-0121-a-meeting-between-two-factions-resolves-at-the-tile.md`
//! [^2]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D2. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
//! [^3]: ADR-0122, an attacker whose attack does not exceed the defender's armour contributes exactly zero, decision D1. `docs/adrs/draft/adr-0122-an-attacker-below-the-armour-contributes-exactly-zero.md`
//! [^4]: ADR-0123, casualties are whole units served to a keyed subset, decisions D1 and D2. `docs/adrs/draft/adr-0123-casualties-are-whole-units-served-to-a-keyed-subset.md`
//! [^5]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^6]: Findings register, FND-402. `docs/FINDINGS.md`

use bytemuck::{Pod, Zeroable};

use crate::bridge::UnitTileBridge;
use crate::cohort::DeathPlane;
use crate::rng;
use crate::sim_math;
use crate::slots::Slots;
use crate::soldier::SoldierArena;
use crate::types::{Accum, Entity, FactionId, Tick, TileIdx, FACTION_CEILING};
use crate::unit_type::{UnitTypeId, UnitTypeTable, UNIT_TYPE_COUNT};

/// The number of fractional bits of the project fixed-point scale.
///
/// The floor of a fixed-point value is a shift by this many bits, and the
/// remainder is what the low bits hold. The scale is declared once, in the
/// value types, and this module reads it from there.[^1]
///
/// # References
///
/// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
use crate::types::FIX_FRACTIONAL_BITS;

/// The number of draws that one defender group takes in one frame.
///
/// A group takes one draw for the rotation that names the fallen, and one for
/// the remainder that covers no whole casualty.[^1]
///
/// # References
///
/// [^1]: ADR-0123, casualties are whole units served to a keyed subset, decision D2. `docs/adrs/draft/adr-0123-casualties-are-whole-units-served-to-a-keyed-subset.md`
const DRAWS_FOR_EACH_GROUP: u32 = 2;

/// The draw index offset of the rotation that names the fallen.
const DRAW_ROTATION: u32 = 0;

/// The draw index offset of the remainder that covers no whole casualty.
const DRAW_REMAINDER: u32 = 1;

/// Returns the draw index of one defender group.
///
/// **The index names the faction and the type, and never the position of the
/// group inside the tile.** A position depends on who else stands there, so an
/// index taken from it would change the draw when an unrelated unit arrived.
/// The faction and the type are properties of the group itself.[^1]
///
/// # References
///
/// [^1]: ADR-0123, casualties are whole units served to a keyed subset, decision D3. `docs/adrs/draft/adr-0123-casualties-are-whole-units-served-to-a-keyed-subset.md`
const fn draw_index(faction: FactionId, unit_type: UnitTypeId, which: u32) -> u32 {
    let group = (faction.0 as u32) * (UNIT_TYPE_COUNT as u32) + (unit_type.0 as u32);
    group * DRAWS_FOR_EACH_GROUP + which
}

/// The reason that the resolution refused a caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContestError {
    /// The caller asked for zero threads. A pass needs at least one.
    ZeroThreads,
    /// The soldier columns hold different lengths.
    ColumnsDisagree,
}

impl core::fmt::Display for ContestError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ZeroThreads => write!(formatter, "a contest pass needs at least one thread"),
            Self::ColumnsDisagree => {
                write!(formatter, "the soldier columns hold different lengths")
            }
        }
    }
}

impl std::error::Error for ContestError {}

/// A unit fell in a meeting between two factions.
///
/// The event reports one unit that the resolution ended. A watcher reads the
/// log to see where a fight happened and what it cost. A fight that nobody can
/// read is a fight that nobody can repair.[^2]
///
/// The layout is 8 + 8 + 4 + 2 + 1 + 1 bytes, which is 24 bytes at an
/// alignment of 8. The trailing array declares every padding byte, so the type
/// holds no uninitialised byte.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
/// [^2]: ADR-0121, a meeting between two factions resolves at the tile, decision D4. `docs/adrs/draft/adr-0121-a-meeting-between-two-factions-resolves-at-the-tile.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct UnitFell {
    /// The tick at which the unit fell.
    pub tick: Tick,
    /// The unit that fell, as its identity in bits.
    pub unit: u64,
    /// The tile that the unit stood on.
    pub tile: TileIdx,
    /// The faction that the unit belonged to.
    pub faction: FactionId,
    /// The type that the unit carried.
    pub unit_type: UnitTypeId,
    /// The declared padding. Always zero.
    pub padding: [u8; 1],
}

impl UnitFell {
    /// Builds an event with zero padding.
    #[must_use]
    pub const fn new(
        tick: Tick,
        unit: u64,
        tile: TileIdx,
        faction: FactionId,
        unit_type: UnitTypeId,
    ) -> Self {
        Self {
            tick,
            unit,
            tile,
            faction,
            unit_type,
            padding: [0; 1],
        }
    }
}

/// One group of units of one faction and one type, on one tile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Group {
    faction: FactionId,
    unit_type: UnitTypeId,
    /// The number of units of the group that stand on the tile.
    count: u32,
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

/// Returns the number of casualties that a quantity of harm produces.
///
/// The harm is a fixed-point quantity of whole units. The whole part is the
/// casualties the harm certainly produces. The fractional part covers no whole
/// casualty, and one keyed draw decides whether it produces one more.[^1]
///
/// The result never exceeds the number of defenders present, because a tile
/// cannot lose more units than it holds.
///
/// # References
///
/// [^1]: ADR-0123, casualties are whole units served to a keyed subset, decision D1. `docs/adrs/draft/adr-0123-casualties-are-whole-units-served-to-a-keyed-subset.md`
#[must_use]
pub fn casualties(harm: Accum, present: u32, remainder_draw: u64) -> u32 {
    if harm.0 <= 0 {
        return 0;
    }
    // The floor and the remainder come from one value, so the two cannot
    // disagree. The shift is the fixed-point scale, which the value types
    // declare.
    let whole = (harm.0 >> FIX_FRACTIONAL_BITS).min(i64::from(present)) as u32;
    if whole >= present {
        return present;
    }
    let remainder = (harm.0 & ((1i64 << FIX_FRACTIONAL_BITS) - 1)) as u64;
    // The draw is uniform below the scale, so the fraction it passes is the
    // remainder itself. The expected number of casualties is therefore the
    // harm exactly, and no rounding rule has to hold it up.
    whole + u32::from(remainder_draw < remainder)
}

/// Resolves every meeting in the world, and marks the units that fell.
///
/// The pass marks. It ends nothing. The caller applies the marks in ascending
/// slot order, after the pass, so the deaths never follow a thread.[^1]
///
/// **Each thread owns its own plane, and the combine is a bitwise union.** A
/// tile belongs to one thread, but two tiles of one thread and two tiles of
/// another can hold units whose slots share one word. A union is commutative
/// and associative, so the joined plane is the same at any thread count.[^2]
///
/// The walk is over the blocks of the derived unit structure, in block order.
/// Inside a block the units lie in tile order, and inside a tile they lie in
/// ascending identity order, because the structure sorted them on that key at
/// the barrier.[^3]
///
/// # Errors
///
/// Returns an error when the caller asks for zero threads, and when the
/// soldier columns hold different lengths.
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
/// [^2]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
/// [^3]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D2. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
pub fn resolve(
    table: &UnitTypeTable,
    key: DrawKey,
    arena: &SoldierArena,
    bridge: &UnitTileBridge,
    plane: &mut DeathPlane,
    threads: usize,
) -> Result<(), ContestError> {
    if threads == 0 {
        return Err(ContestError::ZeroThreads);
    }
    let slots = arena.type_column().len();
    if arena.faction_column().len() != slots || arena.tile_column().len() != slots {
        return Err(ContestError::ColumnsDisagree);
    }
    plane.cover(slots);
    if slots == 0 {
        return Ok(());
    }

    let blocks = bridge.layout().block_count();
    let block_chunk = (blocks as usize).div_ceil(threads).max(1);
    let mut planes: Slots<DeathPlane> =
        Slots::filled(threads, DeathPlane::new()).map_err(|_| ContestError::ZeroThreads)?;
    for entry in planes.entries_mut() {
        entry.cover(slots);
    }

    std::thread::scope(|scope| {
        let mut first = 0u32;
        for entry in planes.entries_mut() {
            let start = first;
            let stop = (start as usize + block_chunk).min(blocks as usize) as u32;
            first = stop;
            scope.spawn(move || {
                let mut sides = Sides::default();
                for block in start..stop {
                    resolve_block(table, key, arena, bridge, block, entry, &mut sides);
                }
            });
        }
    });

    // The union of the planes is the plane. A bitwise or is commutative and
    // associative, so the result does not depend on which thread finished
    // first.[^2]
    plane.union_each(planes.entries());
    Ok(())
}

/// Resolves every contested tile of one block.
///
/// The units of a block lie in one contiguous run, in key order. Equal keys
/// name one tile, so a run of equal keys is the units of that tile.
fn resolve_block(
    table: &UnitTypeTable,
    key: DrawKey,
    arena: &SoldierArena,
    bridge: &UnitTileBridge,
    block: u32,
    plane: &mut DeathPlane,
    sides: &mut Sides,
) {
    let (keys, units) = bridge.block_window(block);
    let grid = arena.grid();
    let factions = arena.faction_column();
    let types = arena.type_column();
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
        let Some(address) = grid.address_of(tile) else {
            // The arena invariant keeps every live unit inside the world, so
            // a tile that does not resolve is a broken arena rather than a
            // caller mistake. The world refuses it elsewhere.
            continue;
        };

        // **The defenders are the units of this tile alone.** Every unit is a
        // defender exactly once, in the pass over the tile it stands on, so
        // nothing is resolved twice.
        sides.defenders.clear();
        add_groups(tile_units, factions, types, &mut sides.defenders);

        // **The attackers are the units of this tile and of its six
        // neighbours.** Contact is adjacency. Two factions that face each
        // other stand on neighbouring ground, and admission refuses a step
        // onto a tile that is at its capacity, so co-occupation is not the
        // ordinary shape of a meeting.[^1]
        //
        // The six neighbours are a fixed set in a fixed order, so this is a
        // pass over tile pairs. It is not a search from a unit.[^2]
        //
        // [^1]: ADR-0121, a meeting between two factions resolves at the tile, decision D1. `docs/adrs/draft/adr-0121-a-meeting-between-two-factions-resolves-at-the-tile.md`
        // [^2]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
        sides.attackers.clear();
        add_groups(tile_units, factions, types, &mut sides.attackers);
        for neighbour in grid.neighbours(address).into_iter().flatten() {
            let Some(index) = grid.index_of(neighbour) else {
                continue;
            };
            add_groups(
                bridge.on_tile_unguarded(index),
                factions,
                types,
                &mut sides.attackers,
            );
        }

        // A tile is contested when some attacker belongs to a faction that
        // some defender does not. The test costs the product of two small
        // group counts, and it skips every tile that no fight touches.
        let contested = sides.attackers.iter().any(|attacker| {
            sides
                .defenders
                .iter()
                .any(|defender| defender.faction != attacker.faction)
        });
        if !contested {
            continue;
        }
        resolve_tile(table, key, tile, tile_units, factions, types, sides, plane);
    }
}

/// The two sides of one tile, reused across the tiles of one thread.
///
/// The buffers are per-thread and they are cleared for each tile, so the pass
/// allocates once for each thread and never once for each tile.
#[derive(Debug, Default)]
struct Sides {
    /// The groups standing on the tile that is resolving.
    defenders: Vec<Group>,
    /// The groups standing on that tile and on its six neighbours.
    attackers: Vec<Group>,
}

/// Adds the units of one tile to a group list, by faction and by type.
///
/// The list holds one entry for each pair that the tiles carry, so its length
/// follows the units read and never the faction count or the type count.
fn add_groups(
    tile_units: &[Entity],
    factions: &[FactionId],
    types: &[UnitTypeId],
    groups: &mut Vec<Group>,
) {
    for unit in tile_units {
        let slot = unit.index() as usize;
        let faction = factions[slot];
        let unit_type = types[slot];
        match groups
            .iter_mut()
            .find(|group| group.faction == faction && group.unit_type == unit_type)
        {
            Some(group) => group.count += 1,
            None => groups.push(Group {
                faction,
                unit_type,
                count: 1,
            }),
        }
    }
}

/// Resolves one contested tile and marks the units that fell.
///
/// The harm against one defender group is a sum over the attacker groups. The
/// threshold applies to each attacker group before the sum, so a group that
/// does not reach adds exactly zero however many units it holds.[^1]
///
/// # References
///
/// [^1]: ADR-0122, an attacker whose attack does not exceed the defender's armour contributes exactly zero, decision D1. `docs/adrs/draft/adr-0122-an-attacker-below-the-armour-contributes-exactly-zero.md`
#[allow(clippy::too_many_arguments)]
fn resolve_tile(
    table: &UnitTypeTable,
    key: DrawKey,
    tile: TileIdx,
    tile_units: &[Entity],
    factions: &[FactionId],
    types: &[UnitTypeId],
    sides: &Sides,
    plane: &mut DeathPlane,
) {
    for defender in &sides.defenders {
        // **The threshold applies for each attacker type before the sum.** A
        // group that does not reach adds exactly zero, and zero is the
        // identity of integer addition, so a sum of zeroes stays zero at any
        // count. That is what makes one tank survive any number of bowmen,
        // and it holds without a rate, a cap or a balance figure.[^1]
        let mut harm = Accum(0);
        for attacker in &sides.attackers {
            if attacker.faction == defender.faction {
                continue;
            }
            if !table.penetrates(attacker.unit_type, defender.unit_type) {
                continue;
            }
            let attack = table.row(attacker.unit_type).attack;
            harm = sim_math::combine(harm, sim_math::scale_by_count(attack, attacker.count));
        }
        if harm.0 <= 0 {
            continue;
        }
        let remainder_draw = rng::draw_below(
            key.seed,
            rng::SYSTEM_CONTEST,
            key.tick.0,
            u64::from(tile.0),
            draw_index(defender.faction, defender.unit_type, DRAW_REMAINDER),
            1u64 << FIX_FRACTIONAL_BITS,
        );
        let fallen = casualties(harm, defender.count, remainder_draw);
        if fallen == 0 {
            continue;
        }
        // **The subset is the ordinals of the group, rotated by a keyed
        // offset.** A rotation is a bijection, so exactly as many units fall
        // below the count as the harm paid for. A draw for each unit would
        // give each unit an independent chance, and the number that fell
        // would then vary around the count the harm paid for.[^2]
        //
        // [^2]: ADR-0123, casualties are whole units served to a keyed subset, decision D2. `docs/adrs/draft/adr-0123-casualties-are-whole-units-served-to-a-keyed-subset.md`
        let offset = rng::draw_below(
            key.seed,
            rng::SYSTEM_CONTEST,
            key.tick.0,
            u64::from(tile.0),
            draw_index(defender.faction, defender.unit_type, DRAW_ROTATION),
            u64::from(defender.count),
        );
        let mut ordinal = 0u64;
        for unit in tile_units {
            let slot = unit.index() as usize;
            if factions[slot] != defender.faction || types[slot] != defender.unit_type {
                continue;
            }
            let place = (ordinal + offset) % u64::from(defender.count);
            if place < u64::from(fallen) {
                plane.mark(slot);
            }
            ordinal += 1;
        }
    }
}

/// Returns the largest draw index that this pass ever takes.
///
/// The index packs the faction and the type of a defender group. This is what
/// a test asserts against, so that a wider faction ceiling or a wider table
/// cannot silently make two groups share one index.
#[must_use]
pub const fn draw_index_ceiling() -> u32 {
    (FACTION_CEILING as u32) * (UNIT_TYPE_COUNT as u32) * DRAWS_FOR_EACH_GROUP
}

/// Returns the harm that one group of attackers delivers to one defender type.
///
/// This is the aggregation of the pass, taken out so that a test can drive it
/// with a count no fixture could reach. The threshold applies before the
/// multiply, so the result is exactly zero for a pair that does not reach,
/// whatever the count.[^1]
///
/// # References
///
/// [^1]: ADR-0122, an attacker whose attack does not exceed the defender's armour contributes exactly zero, decision D1. `docs/adrs/draft/adr-0122-an-attacker-below-the-armour-contributes-exactly-zero.md`
#[must_use]
pub fn harm_of(
    table: &UnitTypeTable,
    attacker: UnitTypeId,
    defender: UnitTypeId,
    count: u32,
) -> Accum {
    if !table.penetrates(attacker, defender) {
        return Accum(0);
    }
    sim_math::scale_by_count(table.row(attacker).attack, count)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::types::Fix32;

    /// A table in which the bowman cannot reach the tank, and the tank ends
    /// four units of anything in one resolution.
    fn tank_and_bowman() -> UnitTypeTable {
        use crate::unit_type::UnitTypeRow;
        let mut table = UnitTypeTable::empty();
        let bowman = UnitTypeRow {
            attack: Fix32::from_int(1),
            armour: Fix32::ZERO,
            ..UnitTypeRow::NONE
        };
        let tank = UnitTypeRow {
            attack: Fix32::from_int(4),
            armour: Fix32::from_int(2),
            ..UnitTypeRow::NONE
        };
        table
            .define(0, bowman)
            .expect("the row is inside the table");
        table.define(1, tank).expect("the row is inside the table");
        table
    }

    #[test]
    fn ten_thousand_bowmen_deliver_exactly_no_harm_to_a_tank() {
        let table = tank_and_bowman();
        let bowman = UnitTypeId(0);
        let tank = UnitTypeId(1);
        for count in [1u32, 4, 64, 10_000, u32::MAX] {
            assert_eq!(
                harm_of(&table, bowman, tank, count),
                Accum(0),
                "a sum of zeroes must stay zero at {count} attackers"
            );
        }
    }

    #[test]
    fn no_remainder_draw_turns_no_harm_into_a_casualty() {
        // The remainder of zero harm is zero, and no draw is below zero.
        for draw in [0u64, 1, (1 << FIX_FRACTIONAL_BITS) - 1] {
            assert_eq!(casualties(Accum(0), 8, draw), 0);
        }
    }

    #[test]
    fn the_draw_index_of_two_groups_never_collides() {
        let mut seen = std::collections::BTreeSet::new();
        for faction in 0..FACTION_CEILING {
            for unit_type in 0..UNIT_TYPE_COUNT as u8 {
                for which in [DRAW_ROTATION, DRAW_REMAINDER] {
                    let index = draw_index(FactionId(faction), UnitTypeId(unit_type), which);
                    assert!(index < draw_index_ceiling());
                    assert!(
                        seen.insert(index),
                        "two groups share the draw index {index}"
                    );
                }
            }
        }
    }
}
