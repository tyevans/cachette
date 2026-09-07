//! What the masked summary rule costs.
//!
//! A summary reader that names a faction combines only the tiles that the
//! sight rule admits, so a cell cannot state what its tiles hide.[^1] The
//! summary level holds a rebuilt cell, and the masked reader cannot use it,
//! because the rebuilt cell counts tiles the faction has not seen. The masked
//! reader therefore walks the tiles of the cell.
//!
//! **This is the one expensive clause of the reader.** Every other answer is
//! one array index and one bit test. This benchmark measures the difference
//! between reading the rebuilt cell and combining beneath the mask.
//!
//! # What it measures
//!
//! One row reads the rebuilt cell of the summary level. That row is the
//! floor, and the reader that names no faction pays it.
//!
//! One row reads the masked cell for a faction that sees no tile of it. The
//! mask admits nothing, so the row measures the walk and the bit test alone.
//!
//! One row reads the masked cell for a faction that sees part of it, and one
//! row reads it for a faction that sees the whole of it. The last row is the
//! ceiling, because every tile of the cell then reaches the combine.
//!
//! # What it does not measure
//!
//! The benchmark reads one cell at a time, which is the shape of the point
//! query. A reader that answers a window of cells amortises the block
//! lookup over the cells of that window, and no such reader exists.
//!
//! The measurement runs on the machine that runs it. The project targets
//! another platform, and a local figure misleads on the cache line.[^2] A run
//! on the target platform is what the cost register takes.[^3]
//!
//! # References
//!
//! [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D4. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
//! [^2]: Testing rules, section 3. `.agents/rules/testing.md`
//! [^3]: Blockers register, BLK-007. `docs/BLOCKERS.md`

use std::hint::black_box;
use std::time::Instant;

use cachette_core::faction_view::Admit;
use cachette_core::{Axial, FactionId, SightRules, World, WorldConfig};

/// The seed that every world in this benchmark takes.
const SEED: u64 = 0x0123_4567_89ab_cdef;

/// How many times each row reads a cell.
const READS: u32 = 200_000;

/// The faction that watches.
const WATCHER: FactionId = FactionId(0);

/// The faction that watches nothing.
const BLIND: FactionId = FactionId(1);

/// The exponent that keeps a unit still.
const KEEP_STILL: u32 = 12;

/// Reads the clock.
///
/// One lint forbids the clock across this workspace, because a simulation
/// that reads a clock gives an answer that depends on the load of the
/// machine.[^1] A benchmark is the one caller that must read it: it produces
/// no simulated state, it enters no state hash, and it asserts nothing.[^2]
///
/// # References
///
/// [^1]: ADR-0005, decision D1. `docs/adrs/REGISTRY.md`
/// [^2]: Testing rules, section 3. `.agents/rules/testing.md`
#[allow(clippy::disallowed_methods)]
fn now() -> Instant {
    Instant::now()
}

/// Builds a world and gives the watcher the sight the row needs.
fn a_world(radius: u32) -> (World, Axial) {
    let config = WorldConfig {
        width: 64,
        height: 64,
        seed: SEED,
        faction_count: 4,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    };
    let mut world = World::new(config).expect("the configuration describes a world");
    world
        .set_choice_schedule(KEEP_STILL)
        .expect("the exponent is inside the range");
    world.set_sight_rules(SightRules::new(radius, 1, 16, 0));

    // The camp sits inside one block of the lattice. Every unit stands on
    // ground that admits one, and the walk over the block is fixed.
    let mut camp = None;
    for row in 32..64i32 {
        for column in 32..64i32 {
            let address = Axial::new(column, row);
            if world.admits_a_unit(address) {
                world
                    .spawn_soldier(address, WATCHER)
                    .expect("the ground admits a unit");
                camp.get_or_insert(address);
            }
        }
    }
    let camp = camp.expect("the block holds ground that admits a unit");
    world.step(1).expect("the step runs");
    (world, camp)
}

/// Reports one row.
fn row(name: &str, tiles: i64, nanoseconds: u128) {
    let each = nanoseconds / u128::from(READS);
    println!("{name:<34} {tiles:>5} tiles   {each:>6} ns for each read");
}

fn main() {
    println!("The cost of one cell read, over {READS} reads");
    println!();

    let (world, camp) = a_world(16);
    let whole = world
        .faction_summary_covering(WATCHER, camp, Admit::SeenNow)
        .expect("the cell covers the address");
    let cell_tiles = whole.admitted() + whole.withheld();

    let start = now();
    let mut sink = 0i64;
    for _ in 0..READS {
        sink += black_box(&world)
            .summary_covering(black_box(camp))
            .expect("the cell covers the address")
            .tiles();
    }
    row(
        "the rebuilt cell, no faction",
        cell_tiles,
        start.elapsed().as_nanos(),
    );

    let start = now();
    for _ in 0..READS {
        sink += world
            .faction_summary_covering(BLIND, camp, Admit::SeenNow)
            .expect("the cell covers the address")
            .admitted();
    }
    row(
        "the masked cell, nothing seen",
        0,
        start.elapsed().as_nanos(),
    );

    let start = now();
    for _ in 0..READS {
        sink += world
            .faction_summary_covering(WATCHER, camp, Admit::SeenNow)
            .expect("the cell covers the address")
            .admitted();
    }
    row(
        "the masked cell, seen whole",
        whole.admitted(),
        start.elapsed().as_nanos(),
    );

    let (part, part_camp) = a_world(3);
    let some = part
        .faction_summary_covering(WATCHER, part_camp, Admit::SeenNow)
        .expect("the cell covers the address");
    let start = now();
    for _ in 0..READS {
        sink += part
            .faction_summary_covering(WATCHER, part_camp, Admit::SeenNow)
            .expect("the cell covers the address")
            .admitted();
    }
    row(
        "the masked cell, seen in part",
        some.admitted(),
        start.elapsed().as_nanos(),
    );

    // The last row takes the ground alone. Every unit of the watcher is
    // gone, so the watcher sees nothing and remembers everything. The
    // memory rule then admits every tile and reads no unit, no holder and
    // no value, which is the cheaper of the two tile paths.
    let (mut memory, memory_camp) = a_world(16);
    let scout = memory
        .spawn_soldier(memory_camp, BLIND)
        .expect("the ground admits a unit");
    memory.step(1).expect("the step runs");
    let away = (0..64i32)
        .flat_map(|row| (0..32i32).map(move |column| Axial::new(column, row)))
        .find(|address| memory.admits_a_unit(*address))
        .expect("the far half holds ground that admits a unit");
    memory
        .place_soldier(scout, away)
        .expect("the far ground admits a unit");
    memory.step(1).expect("the step runs");
    let remembered = memory
        .faction_summary_covering(BLIND, memory_camp, Admit::SeenEver)
        .expect("the cell covers the address");
    let start = now();
    for _ in 0..READS {
        sink += memory
            .faction_summary_covering(BLIND, memory_camp, Admit::SeenEver)
            .expect("the cell covers the address")
            .admitted();
    }
    row(
        "the masked cell, ground alone",
        remembered.admitted(),
        start.elapsed().as_nanos(),
    );

    // The last row is the comparator. The summary rebuild reads the ground
    // of every tile of every cell, once for each frame, and the masked
    // reader reads the ground of every admitted tile, once for each call.
    // The two figures say which of the two shapes the ground cost belongs
    // in.
    let mut stepping = a_world(16).0;
    let cells = u128::from(stepping.observation().layout().block_count());
    let steps = 200u32;
    let start = now();
    for _ in 0..steps {
        stepping.step(1).expect("the step runs");
    }
    let each = start.elapsed().as_nanos() / u128::from(steps);
    println!(
        "{:<34} {:>5} cells   {:>6} ns for each step",
        "one step of the same world", cells, each
    );

    println!();
    println!("The sum of every reading is {sink}. Nothing reads it.");
}
