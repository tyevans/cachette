//! The state hash reads the state as raw bytes, and every byte it reads is
//! initialised.
//!
//! The hash writes whole structures and whole columns as bytes.[^1] That read
//! is sound only while every type it reaches is plain data with declared
//! padding.[^2] A structure that gains an undeclared padding byte still
//! compiles, still passes every other test, and puts an uninitialised byte
//! into the hash. The hash then differs between two runs of the same binary,
//! and the failure looks like a simulation defect rather than a layout one.
//!
//! No ordinary test sees this. A padding byte holds whatever the allocator
//! left there, and on one machine that is reliably zero. Miri sees it,
//! because Miri tracks which bytes are initialised and stops the read.
//!
//! **This test drives the engine.** A test that hashed a structure it built
//! itself would prove that the structure is sound and not that the engine
//! reaches only sound structures.[^3]
//!
//! **Every extent here is chosen for what Miri can reach, and each one is
//! still chosen for what it covers.** Miri interprets every instruction, so
//! the cost of this fixture is its tile count multiplied by its frame count.
//!
//! The unit capacity is small because a world reserves its unit columns when
//! it is built.[^4] A world at the target population reserves a million slots
//! and does not finish under Miri, whatever its extent. This world reserves a
//! few hundred, which is enough to hash a populated arena rather than an empty
//! one.
//!
//! The world is wide and short rather than square. The coarsest lattice of the
//! terrain generator spans sixty-four tiles, so a world narrower than that on
//! both axes sits inside one lattice cell and holds one kind of ground. A
//! fixture like that would hash a tile column of one repeated value, and a
//! change to how the ground is stored could not move it. This world crosses
//! the lattice along its width and stays short along its height, so it holds
//! more than one kind of ground at a fraction of the tile count.
//!
//! Run it with `just miri`.
//!
//! # References
//!
//! [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
//! [^2]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
//! [^3]: Testing rules, drive the real caller. `.claude/rules/testing.md`
//! [^4]: ADR-0084, the world reserves the unit columns at construction. `docs/adrs/draft/adr-0084-the-world-reserves-the-unit-columns-at-construction.md`

use cachette_core::census::census;
use cachette_core::{Axial, FactionId, World, WorldConfig};

/// The width of the fixture.
///
/// The coarsest lattice of the terrain generator spans sixty-four tiles, and
/// this width crosses it. A world inside one lattice cell holds one kind of
/// ground, and its tile columns would hash one repeated value.
const WIDTH: u32 = 72;

/// The height of the fixture.
///
/// The width already crosses the lattice, so the height buys tile count and
/// nothing else. Under Miri the tile count is most of the cost, and the cost
/// is close to linear in it. This world was sixteen tiles tall while the gate
/// was being written, and one test of it did not finish in ten minutes.
const HEIGHT: u32 = 4;

/// The unit capacity of the fixture.
///
/// The world reserves this many slots when it is built, and Miri interprets
/// every write. The target population is one million and does not finish.
const CAPACITY: u32 = 128;

/// The number of soldiers the fixture spawns.
const SOLDIERS: u32 = 8;

/// The number of frames the fixture runs.
const FRAMES: u64 = 1;

fn config() -> WorldConfig {
    WorldConfig {
        width: WIDTH,
        height: HEIGHT,
        seed: 0x0123_4567_89ab_cdef,
        faction_count: 2,
        unit_capacity: CAPACITY,
    }
}

/// Asserts that the fixture produced the case it claims to cover.
///
/// The world must hold more than one kind of ground, and it must hold the
/// soldiers this fixture spawned. A world of one repeated ground value hashes
/// a tile column that no change to how the ground is stored could move, and an
/// empty arena hashes a column of zero length. Either would pass this gate for
/// the wrong reason.
///
/// **Call this after a step.** The census reads a bridge from a unit to its
/// tile, and the engine rebuilds that bridge in the step. A census taken
/// between a spawn and the next step reports the bridge as stale rather than
/// answering.
fn assert_the_fixture_reaches_the_case(world: &World, placed: u32) {
    // The centre and the reach cover every address of this world, and the
    // reach stays under the ceiling the census enforces.
    let centre = Axial::new(
        i32::try_from(WIDTH / 2).expect("the width fits an i32"),
        i32::try_from(HEIGHT / 2).expect("the height fits an i32"),
    );
    let reach = WIDTH / 2;
    let seen = census(world, centre, reach).expect("the reach is under the ceiling");

    let kinds = seen.by_kind().iter().filter(|count| **count > 0).count();
    assert!(
        kinds > 1,
        "the world holds one kind of ground, so its tile columns hash one repeated value"
    );
    assert!(
        placed > 0,
        "the fixture placed no soldier, so the hash would read an empty arena"
    );
    assert_eq!(
        seen.units(),
        i64::from(placed),
        "the census does not see the soldiers the fixture spawned"
    );
}

/// Builds a world, spawns soldiers into it, and reports how many it placed.
fn populate(world: &mut World) -> u32 {
    let mut placed = 0;
    let mut index = 0;
    while placed < SOLDIERS && index < WIDTH * HEIGHT {
        let q = i32::try_from(index % WIDTH).expect("the width fits an i32");
        let r = i32::try_from(index / WIDTH).expect("the height fits an i32");
        let faction = FactionId(u16::try_from(placed % 2).expect("the count fits a u16"));
        if world.spawn_soldier(Axial::new(q, r), faction).is_ok() {
            placed += 1;
        }
        index += 1;
    }
    placed
}

#[test]
fn the_state_hash_reads_no_uninitialised_byte() {
    let mut world = World::new(config()).expect("the extent describes a world");
    let placed = populate(&mut world);

    // Hashing before the first step covers the state the constructor wrote.
    let first = world.state_hash();

    for _ in 0..FRAMES {
        world.step(1).expect("the step must run");
    }
    let second = world.state_hash();

    assert_the_fixture_reaches_the_case(&world, placed);

    // The world moved, so the hash must move with it. A hash that did not
    // change would mean the fixture ran nothing, and the byte read this test
    // exists for would never have happened over the changed state.
    assert_ne!(
        first, second,
        "the state did not change, so the fixture stepped nothing"
    );
}

#[test]
fn the_hash_repeats_over_the_same_state() {
    let mut world = World::new(config()).expect("the extent describes a world");
    let placed = populate(&mut world);
    world.step(1).expect("the step must run");
    assert_the_fixture_reaches_the_case(&world, placed);

    // Two reads of one state must agree. An uninitialised padding byte is the
    // one thing that can make them disagree, and Miri stops the read before
    // it can.
    assert_eq!(world.state_hash(), world.state_hash());
}
