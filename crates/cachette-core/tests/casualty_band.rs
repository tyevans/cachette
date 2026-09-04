//! How wide is the band of tiles that holds the casualties of a fight?
//!
//! This suite measures a shape. It writes no combat pass into the engine, and
//! the engine holds none.[^1] A design sketch resolves a fight for each level
//! 1 cell, as a small table over unit types. A cell summarises a block of
//! tiles, and the block edge is a power of two that one layout constant sets.
//! A fight resolved for a whole block kills units across the whole block, so
//! the casualties may not form a front line.[^2]
//!
//! # What is the engine and what is the model
//!
//! **The engine supplies** the world, the terrain and its passability, the
//! unit arena, the faction column, the tile column, the unit-to-tile bridge
//! and the block layout that fixes the block edge. Every geometric figure
//! this suite reports comes from those structures.
//!
//! **This file supplies** three things that the engine does not hold. The
//! first is an approach rule, because the control plane cannot name a
//! destination today.[^3] The second is a provisional casualty rule, because
//! nothing kills a unit in a fight. The third is the selection of which units
//! die, which copies the shape of the rule the project already holds for a
//! cohort: one keyed draw serves a whole group by rotating the ordinals.[^4]
//!
//! **The provisional rule is thrown away.** It exists to place casualties on
//! tiles so that the band can be measured. It states no game rule, and item
//! 0345 owes it nothing.
//!
//! # The fixtures
//!
//! A fixture that models the typical case supplies no extreme, so the
//! assertion never receives the input that would fail it.[^5] This suite
//! therefore builds three arrangements rather than one.
//!
//! - `wall` places each army as a deep line across the whole world, and puts
//!   the contact column in the middle of a block.
//! - `blob` places each army as a compact square, and puts the contact point
//!   at the corner of a block. This is the extreme: a unit at the far corner
//!   of a contested block stands most of a block edge away from any enemy.
//! - `skirmish` places each army two tiles deep. This is the arrangement that
//!   hides the defect, and it is here to prove that the other two do not.
//!
//! **The defect is put back on purpose.** Every fixture resolves twice: once
//! for each tile, and once for each level 1 cell. A fixture that cannot show
//! the bad case measures itself.[^5]
//!
//! # References
//!
//! [^1]: Backlog item 0344, measure whether a fight makes a front line. `docs/backlog/complete/0344-measure-whether-a-fight-makes-a-front-line.md`
//! [^2]: Blockers register, BLK-052. `docs/BLOCKERS.md`
//! [^3]: Research report 21, what a god needs from this engine, section 1.1. `docs/research/reports/21-what-a-god-needs.md`
//! [^4]: ADR-0106, a cohort serves whole rations to a keyed subset, decisions D1 and D2. `docs/adrs/draft/adr-0106-a-cohort-serves-whole-rations-to-a-keyed-subset.md`
//! [^5]: Testing rules, section 2a. `.claude/rules/testing.md`

use std::collections::VecDeque;

use cachette_core::rng;
use cachette_core::terrain::TileKind;
use cachette_core::{Axial, Entity, FactionId, TileIdx, World, WorldConfig};

/// The seed of the measured world.
///
/// The suite asserts the shape of the ground this seed gives, so a change to
/// the generator fails here rather than moving a number quietly.
const SEED: u64 = 0x2a;

/// The width of the measured world, in tiles.
const WIDTH: u32 = 128;

/// The height of the measured world, in tiles.
const HEIGHT: u32 = 96;

/// The faction that starts on the west side.
const WEST: FactionId = FactionId(1);

/// The faction that starts on the east side.
const EAST: FactionId = FactionId(2);

/// The identifier this harness keys its draws on.
///
/// It is local to this file. The engine holds no combat system, so it holds
/// no identifier for one, and this suite must not add one.
const SYSTEM_HARNESS_FIGHT: rng::SystemId = 0x0344;

/// The number of frames the suite resolves after the armies touch.
const RESOLVED_FRAMES: u64 = 24;

/// The number of units one side removes from one resolution unit in one
/// frame.
///
/// The value is small on purpose. A large value melts an army inside the
/// measured frames, and the suite would then measure attrition rather than
/// the standing arrangement.
const KILLS_FOR_EACH_UNIT: usize = 1;

/// The granularity at which the provisional rule resolves a fight.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Granularity {
    /// Resolve for each tile. Only a tile that holds both factions fights.
    Tile,
    /// Resolve for each level 1 cell. Every unit inside a contested block
    /// may die, whatever tile it stands on.
    Cell,
}

/// A rectangle of ground that one army fills.
#[derive(Clone, Copy)]
struct Block {
    /// The first column.
    q: i32,
    /// The number of columns.
    columns: i32,
    /// The first row.
    r: i32,
    /// The number of rows.
    rows: i32,
    /// The number of units the fixture places on one tile.
    stack: u32,
}

/// One arrangement of two armies.
#[derive(Clone, Copy)]
struct Fixture {
    /// The name that the report uses.
    name: &'static str,
    /// The ground the west army fills.
    west: Block,
    /// The ground the east army fills.
    east: Block,
    /// The number of frames the armies march before the fight resolves.
    march: u64,
}

/// A deep line across the whole world, with the contact column inside a
/// block rather than on its edge.
const WALL: Fixture = Fixture {
    name: "wall",
    west: Block {
        q: 60,
        columns: 16,
        r: 0,
        rows: HEIGHT as i32,
        stack: 2,
    },
    east: Block {
        q: 84,
        columns: 16,
        r: 0,
        rows: HEIGHT as i32,
        stack: 2,
    },
    march: 8,
};

/// A compact square that meets another near the corner of a block.
///
/// The block edge is 32 and a block starts at a multiple of it. The armies
/// meet near the column 64 and the row 32, which is the corner of a block,
/// so a contested block holds units in both directions from the contact.
const BLOB: Fixture = Fixture {
    name: "blob",
    west: Block {
        q: 40,
        columns: 24,
        r: 8,
        rows: 48,
        stack: 4,
    },
    east: Block {
        q: 68,
        columns: 24,
        r: 8,
        rows: 48,
        stack: 4,
    },
    march: 8,
};

/// Two tiles of depth. This arrangement holds little behind the front, so it
/// is the arrangement that hides the smear.
const SKIRMISH: Fixture = Fixture {
    name: "skirmish",
    west: Block {
        q: 74,
        columns: 2,
        r: 0,
        rows: HEIGHT as i32,
        stack: 2,
    },
    east: Block {
        q: 80,
        columns: 2,
        r: 0,
        rows: HEIGHT as i32,
        stack: 2,
    },
    march: 8,
};

/// The adversarial extreme. One army fills a whole level 1 cell, and the
/// other touches one corner of it.
///
/// The block that this fixture contests spans the columns 64 to 95 and the
/// rows 32 to 63, which the layout fixes and this file does not. A unit of
/// the west army at the far corner of that block stands most of a block
/// diagonal from the nearest enemy, and a resolution for the whole cell may
/// kill it.
const CORNER: Fixture = Fixture {
    name: "corner",
    west: Block {
        q: 64,
        columns: 27,
        r: 32,
        rows: 32,
        stack: 2,
    },
    east: Block {
        q: 91,
        columns: 5,
        r: 58,
        rows: 6,
        stack: 4,
    },
    march: 0,
};

/// What one run of the provisional rule produced.
struct Measurement {
    /// The name of the fixture.
    fixture: &'static str,
    /// The granularity the run resolved at.
    grain: Granularity,
    /// The number of casualties the run produced.
    casualties: usize,
    /// The distance of the fifth percentile casualty, in tiles.
    low: u32,
    /// The distance of the ninety-fifth percentile casualty, in tiles.
    high: u32,
    /// The width of the band that holds the middle 90 percent, in tiles.
    band: u32,
    /// The largest distance any casualty stood at, in tiles.
    furthest: u32,
    /// The number of casualties that stood on a tile holding no enemy.
    away_from_an_enemy: usize,
    /// The largest number of resolution units that any frame contested.
    contested: usize,
    /// The number of units the two armies started with.
    started: u32,
    /// The number of units that lived at the end of the run.
    survived: u32,
}

/// Builds the world that every fixture runs in.
fn build() -> World {
    World::new(WorldConfig {
        width: WIDTH,
        height: HEIGHT,
        seed: SEED,
        faction_count: 3,
        unit_capacity: 32_768,
    })
    .expect("the measured world is inside every ceiling")
}

/// Returns the address of a tile index.
fn address_of(index: TileIdx, width: u32) -> Axial {
    Axial::new((index.0 % width) as i32, (index.0 / width) as i32)
}

/// Places one army and returns the number of units it placed.
fn place_army(world: &mut World, block: Block, faction: FactionId) -> u32 {
    let mut placed = 0;
    for q in block.q..block.q + block.columns {
        for r in block.r..block.r + block.rows {
            let address = Axial::new(q, r);
            if !world.admits_a_unit(address) {
                continue;
            }
            for _ in 0..block.stack {
                if world.spawn_soldier(address, faction).is_ok() {
                    placed += 1;
                }
            }
        }
    }
    placed
}

/// Returns every live unit with the tile it stands on and the faction it
/// belongs to, in slot order.
fn snapshot(world: &World) -> Vec<(Entity, TileIdx, FactionId)> {
    world
        .soldiers()
        .iter()
        .map(|unit| {
            let tile = world
                .soldiers()
                .tile(unit)
                .expect("a live unit holds a tile");
            let faction = world
                .soldiers()
                .faction(unit)
                .expect("a live unit holds a faction");
            (unit, tile, faction)
        })
        .collect()
}

/// Moves every unit one tile toward the enemy, where the ground allows it.
///
/// **This is the model and not the engine.** The engine steers a unit from a
/// field over cells, and the control plane cannot seed that field at a place
/// it names, so no engine call sends an army at another army.
fn advance(world: &mut World) {
    let width = world.grid().width();
    let capacity = TileKind::Plain.capacity();
    let units = snapshot(world);
    let mut occupancy = vec![0u32; world.tile_count()];
    let mut west_here = vec![false; world.tile_count()];
    let mut east_here = vec![false; world.tile_count()];
    for (_, tile, faction) in &units {
        occupancy[tile.0 as usize] += 1;
        if *faction == WEST {
            west_here[tile.0 as usize] = true;
        } else {
            east_here[tile.0 as usize] = true;
        }
    }
    for (unit, tile, faction) in units {
        // A unit that already stands with an enemy is engaged, and it holds
        // its ground. Without this rule two thin armies walk through each
        // other and never contest a tile twice.
        let enemy_here = if faction == WEST {
            east_here[tile.0 as usize]
        } else {
            west_here[tile.0 as usize]
        };
        if enemy_here {
            continue;
        }
        let here = address_of(tile, width);
        let step = if faction == WEST { 1 } else { -1 };
        let there = Axial::new(here.q + step, here.r);
        let Some(target) = world.grid().index_of(there) else {
            continue;
        };
        if !world.admits_a_unit(there) || occupancy[target.0 as usize] >= capacity {
            continue;
        }
        if world.place_soldier(unit, there).unwrap_or(false) {
            occupancy[tile.0 as usize] -= 1;
            occupancy[target.0 as usize] += 1;
        }
    }
}

/// Returns the distance from every tile to the nearest tile of the set, in
/// tiles.
///
/// The walk crosses impassable ground, so the answer is the geometry of the
/// world and not the route a unit would take.
fn distance_field(world: &World, sources: &[TileIdx]) -> Vec<u32> {
    let grid = world.grid();
    let mut distance = vec![u32::MAX; world.tile_count()];
    let mut queue: VecDeque<TileIdx> = VecDeque::new();
    for tile in sources {
        if distance[tile.0 as usize] != 0 {
            distance[tile.0 as usize] = 0;
            queue.push_back(*tile);
        }
    }
    while let Some(tile) = queue.pop_front() {
        let here = address_of(tile, grid.width());
        let next = distance[tile.0 as usize] + 1;
        for neighbour in grid.neighbours(here).into_iter().flatten() {
            let Some(index) = grid.index_of(neighbour) else {
                continue;
            };
            if distance[index.0 as usize] > next {
                distance[index.0 as usize] = next;
                queue.push_back(index);
            }
        }
    }
    distance
}

/// Returns the key of the resolution unit that a tile belongs to.
fn resolution_unit(world: &World, tile: TileIdx, grain: Granularity) -> u64 {
    match grain {
        Granularity::Tile => u64::from(tile.0),
        Granularity::Cell => {
            let layout = world.bridge().layout();
            let key = layout.key_of(tile).expect("a live tile holds a key");
            u64::from(layout.block_of_key(key))
        }
    }
}

/// Runs one fixture at one granularity and reports the band.
fn measure(fixture: Fixture, grain: Granularity, threads: usize) -> Measurement {
    let mut world = build();
    let west = place_army(&mut world, fixture.west, WEST);
    let east = place_army(&mut world, fixture.east, EAST);
    assert!(west > 0 && east > 0, "the fixture placed no army");
    let started = west + east;
    world.rebuild_bridge(threads).expect("the bridge rebuilds");

    for _ in 0..fixture.march {
        advance(&mut world);
        world.rebuild_bridge(threads).expect("the bridge rebuilds");
    }

    let mut distances: Vec<u32> = Vec::new();
    let mut away = 0usize;
    let mut contested = 0usize;
    let width = world.grid().width();

    for frame in 0..RESOLVED_FRAMES {
        let units = snapshot(&world);
        let mut west_tiles: Vec<TileIdx> = Vec::new();
        let mut east_tiles: Vec<TileIdx> = Vec::new();
        for (_, tile, faction) in &units {
            if *faction == WEST {
                west_tiles.push(*tile);
            } else {
                east_tiles.push(*tile);
            }
        }
        west_tiles.sort_unstable();
        west_tiles.dedup();
        east_tiles.sort_unstable();
        east_tiles.dedup();
        let to_west = distance_field(&world, &west_tiles);
        let to_east = distance_field(&world, &east_tiles);

        // Group the units by the resolution unit the granularity names. The
        // key is an integer, and the walk is over a sorted vector, so no
        // hash order reaches the result.
        let mut rows: Vec<(u64, FactionId, Entity, TileIdx)> = units
            .iter()
            .map(|(unit, tile, faction)| {
                (
                    resolution_unit(&world, *tile, grain),
                    *faction,
                    *unit,
                    *tile,
                )
            })
            .collect();
        rows.sort_unstable_by_key(|(key, faction, unit, _)| (*key, faction.0, unit.to_bits()));

        let mut casualties: Vec<(Entity, TileIdx, FactionId)> = Vec::new();
        let mut contested_now = 0usize;
        let mut at = 0;
        while at < rows.len() {
            let key = rows[at].0;
            let mut end = at;
            while end < rows.len() && rows[end].0 == key {
                end += 1;
            }
            let group = &rows[at..end];
            let west_side: Vec<(Entity, TileIdx)> = group
                .iter()
                .filter(|(_, faction, _, _)| *faction == WEST)
                .map(|(_, _, unit, tile)| (*unit, *tile))
                .collect();
            let east_side: Vec<(Entity, TileIdx)> = group
                .iter()
                .filter(|(_, faction, _, _)| *faction == EAST)
                .map(|(_, _, unit, tile)| (*unit, *tile))
                .collect();
            if !west_side.is_empty() && !east_side.is_empty() {
                contested_now += 1;
                serve(SEED, frame, key, 0, &east_side, EAST, &mut casualties);
                serve(SEED, frame, key, 1, &west_side, WEST, &mut casualties);
            }
            at = end;
        }

        contested = contested.max(contested_now);
        for (unit, tile, faction) in &casualties {
            let field = if *faction == WEST { &to_east } else { &to_west };
            let distance = field[tile.0 as usize];
            assert!(distance != u32::MAX, "every casualty reaches an enemy");
            distances.push(distance);
            if distance > 0 {
                away += 1;
            }
            let _ = address_of(*tile, width);
            world.despawn_soldier(*unit);
        }

        advance(&mut world);
        world.rebuild_bridge(threads).expect("the bridge rebuilds");
    }

    distances.sort_unstable();
    let count = distances.len();
    assert!(count > 0, "the fixture produced no casualty");
    let low = distances[count * 5 / 100];
    let high = distances[(count * 95 / 100).min(count - 1)];
    Measurement {
        fixture: fixture.name,
        grain,
        casualties: count,
        low,
        high,
        band: high - low + 1,
        furthest: *distances.last().expect("the run holds a casualty"),
        away_from_an_enemy: away,
        contested,
        started,
        survived: world.soldiers().len(),
    }
}

/// Serves the casualties of one side of one resolution unit.
///
/// The victims are the ordinals of the group rotated by a keyed offset, which
/// is the rule the project already holds for a ration.
fn serve(
    seed: u64,
    frame: u64,
    key: u64,
    index: u32,
    side: &[(Entity, TileIdx)],
    faction: FactionId,
    out: &mut Vec<(Entity, TileIdx, FactionId)>,
) {
    let count = side.len();
    let kills = KILLS_FOR_EACH_UNIT.min(count);
    let offset =
        rng::draw_below(seed, SYSTEM_HARNESS_FIGHT, frame, key, index, count as u64) as usize;
    for step in 0..kills {
        let (unit, tile) = side[(offset + step) % count];
        out.push((unit, tile, faction));
    }
}

/// Prints one row of the table that the review reports.
fn report(measurement: &Measurement) {
    println!(
        "{:9} {:5?} started {:6} survived {:6} casualties {:6} p5 {:3} p95 {:3} band {:3} furthest {:3} away {:6} contested {:5}",
        measurement.fixture,
        measurement.grain,
        measurement.started,
        measurement.survived,
        measurement.casualties,
        measurement.low,
        measurement.high,
        measurement.band,
        measurement.furthest,
        measurement.away_from_an_enemy,
        measurement.contested,
    );
}

/// The ground of the measured world is mixed, and the fixture says so.
///
/// A world of one kind of ground would give a straight contact line that no
/// terrain interrupts, and the band would then describe the fixture rather
/// than a world.
#[test]
fn the_measured_world_holds_mixed_ground() {
    let world = build();
    let mut open = 0;
    let mut closed = 0;
    for index in 0..world.tile_count() {
        let address = address_of(TileIdx(index as u32), world.grid().width());
        if world.admits_a_unit(address) {
            open += 1;
        } else {
            closed += 1;
        }
    }
    println!("open tiles {open}, closed tiles {closed}");
    assert!(closed > 0, "the world holds ground that admits no unit");
    assert!(open > closed * 3, "the world is mostly open ground");
    assert_eq!(
        world.bridge().layout().block_edge(),
        32,
        "the measured block edge is the one the layout gives"
    );
}

/// The band is narrow at the tile and wide at the level 1 cell.
///
/// This is the measurement that backlog item 0344 owes, and it is the
/// evidence that blocker BLK-052 asked for.
#[test]
fn a_cell_resolution_smears_and_a_tile_resolution_does_not() {
    let mut wide = 0;
    for fixture in [WALL, BLOB, SKIRMISH, CORNER] {
        let tile = measure(fixture, Granularity::Tile, 1);
        let cell = measure(fixture, Granularity::Cell, 1);
        report(&tile);
        report(&cell);
        assert_eq!(
            tile.band, 1,
            "a tile resolution kills only where the factions stand together"
        );
        assert_eq!(
            tile.away_from_an_enemy, 0,
            "no casualty of a tile resolution stands away from an enemy"
        );
        assert!(
            cell.band >= tile.band,
            "the cell band is never narrower than the tile band"
        );
        if cell.band > 1 {
            wide += 1;
        }
    }
    assert!(
        wide >= 3,
        "the fixtures show the smear that the sketch risks"
    );
}

/// The band does not depend on the thread count of the barrier.
///
/// The bridge rebuilds in parallel, and every figure in this suite is read
/// through it. A measurement that changed with the thread count would be
/// evidence about the schedule and not about the world.
#[test]
fn the_band_does_not_depend_on_the_thread_count() {
    let one = measure(CORNER, Granularity::Cell, 1);
    let many = measure(CORNER, Granularity::Cell, 12);
    assert_eq!(one.casualties, many.casualties);
    assert_eq!(one.low, many.low);
    assert_eq!(one.high, many.high);
    assert_eq!(one.furthest, many.furthest);
    assert_eq!(one.away_from_an_enemy, many.away_from_an_enemy);
}

/// A tile resolution reads what the engine already builds.
///
/// The bridge lists the units that stand on one tile, and it rebuilds at the
/// barrier. This test drives the engine structure rather than the vector this
/// file builds, so the claim that the input exists is checked and not
/// asserted.
#[test]
fn the_bridge_supplies_the_input_a_tile_resolution_needs() {
    let mut world = build();
    place_army(&mut world, WALL.west, WEST);
    place_army(&mut world, WALL.east, EAST);
    world.rebuild_bridge(1).expect("the bridge rebuilds");
    for _ in 0..WALL.march {
        advance(&mut world);
        world.rebuild_bridge(1).expect("the bridge rebuilds");
    }

    let mut mixed = 0;
    let mut counted = 0;
    for index in 0..world.tile_count() {
        let address = address_of(TileIdx(index as u32), world.grid().width());
        let units = world.soldiers_on(address).expect("the bridge answers");
        counted += units.len();
        let mut west = false;
        let mut east = false;
        for unit in units {
            match world.soldiers().faction(*unit) {
                Some(WEST) => west = true,
                Some(EAST) => east = true,
                _ => {}
            }
        }
        if west && east {
            mixed += 1;
        }
    }
    println!("the bridge lists {counted} units and {mixed} contested tiles");
    assert_eq!(
        counted,
        world.soldiers().len() as usize,
        "the bridge lists every live unit"
    );
    assert!(mixed > 0, "the armies reached contact");
}

/// A tile of ordinary ground holds a small number of units, and a full tile
/// admits nobody.
///
/// This bounds any table that a tile resolution reads, and it is the case a
/// player would call wrong: an army that fills its tiles cannot be attacked
/// at all by a rule that needs the two factions on one tile.
#[test]
fn a_full_tile_refuses_the_enemy_that_would_fight_it() {
    let capacity = TileKind::Plain.capacity();
    println!("the capacity of ordinary ground is {capacity} units");
    assert!(capacity > 0);
    assert!(
        capacity < 16,
        "a tile table is bounded by the capacity of the ground"
    );
}

/// One tank kills four bowmen, and no number of bowmen kills the tank.
///
/// **This is a model and not the engine.** A unit carries no type and no
/// strength, so the engine cannot express either side of this test. The model
/// is four integers and two folds.
#[test]
fn the_threshold_before_aggregation_gives_the_tank_the_win() {
    // The effect of one bowman, and the threshold of the tank.
    let bowman_effect: i64 = 10;
    let tank_threshold: i64 = 100;
    // The effect of the tank, and the threshold of one bowman.
    let tank_effect: i64 = 60;
    let bowman_threshold: i64 = 20;

    // The tank kills the four bowmen.
    assert!(
        before(tank_effect, bowman_threshold, 1) > 0,
        "the tank penetrates a bowman"
    );

    // No count of bowmen reaches the tank.
    for count in [4i64, 40, 400, 4000, 1_000_000] {
        assert_eq!(
            before(bowman_effect, tank_threshold, count),
            0,
            "a sum of zeroes is zero at any count"
        );
    }
}

/// The same table loses the tank when the threshold is applied after the
/// sum.
///
/// This is the comparison that makes the first test mean something. The two
/// runs differ in the order of one operation and in nothing else.
#[test]
fn the_threshold_after_aggregation_loses_the_tank() {
    let bowman_effect: i64 = 10;
    let tank_threshold: i64 = 100;

    assert_eq!(
        after(bowman_effect, tank_threshold, 4),
        0,
        "four bowmen do not reach the tank in either order"
    );
    // Eleven bowmen reach it, and the acceptance test fails from there
    // upward.
    for count in [11i64, 40, 400, 4000] {
        assert!(
            after(bowman_effect, tank_threshold, count) > 0,
            "a sum taken before the threshold lets a crowd through"
        );
    }
    // The smallest crowd that breaks the acceptance test.
    let mut smallest = 0;
    for count in 1..=200i64 {
        if after(bowman_effect, tank_threshold, count) > 0 {
            smallest = count;
            break;
        }
    }
    println!("the threshold after the sum fails at {smallest} bowmen");
    assert_eq!(smallest, 11);
}

/// The cliff, stated as a number.
///
/// One point of effect turns a whole class of attacker from harmless into
/// lethal. A player who improves a unit by the smallest possible step, and
/// then watches a war change, reads that as a defect.
#[test]
fn one_point_of_effect_turns_a_harmless_army_lethal() {
    let threshold: i64 = 100;
    let crowd: i64 = 1000;
    assert_eq!(before(99, threshold, crowd), 0, "the crowd is harmless");
    assert_eq!(
        before(101, threshold, crowd),
        crowd,
        "the same crowd is lethal"
    );
}

/// Sums the effect of a crowd, with the threshold applied to each type
/// before the sum.
fn before(effect: i64, threshold: i64, count: i64) -> i64 {
    let term = if effect > threshold {
        effect - threshold
    } else {
        0
    };
    term * count
}

/// Sums the effect of a crowd, with the threshold applied after the sum.
fn after(effect: i64, threshold: i64, count: i64) -> i64 {
    let total = effect * count;
    if total > threshold {
        total - threshold
    } else {
        0
    }
}
