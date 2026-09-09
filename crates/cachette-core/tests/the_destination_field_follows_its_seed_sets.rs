//! The destination field of a plane follows the seed set of that plane.
//!
//! # What this holds
//!
//! The control plane names a set of tiles and a set of units in one call. The
//! engine seeds one plane at those tiles and spreads a reach outward, and a
//! sent unit reads the entry of its own cell.[^1]
//!
//! The engine derives the field only when a send changed a seed set or a
//! crossing. **A guard is only an optimisation when the field still follows
//! its input.** These tests state what the field depends on: re-aim one plane,
//! and that plane changes; leave another plane alone, and it does not.[^2]
//!
//! A stale field is wrong behaviour that repeats exactly. The thread count
//! test and the stored state hash both pass on it, so neither of them is the
//! test that finds it.[^2]
//!
//! # How a reader proves that these tests can fail
//!
//! One test-only switch removes the mark that the send verb writes. The engine
//! then derives the field once and never again.
//!
//! ```text
//! ! cargo test --package cachette-core --features probe-frozen-destinations \
//!     --test the_destination_field_follows_its_seed_sets
//! ```
//!
//! # The fixture
//!
//! Every faction of the world is externally controlled, so the built-in
//! controller sends nothing and each send below is the only thing that touches
//! a plane. The world is wide enough to hold several level 1 cells, and the
//! two places the tests name sit in different cells. A world of one cell would
//! carry no direction at all, and the tests would then measure the
//! fixture.[^3]
//!
//! # References
//!
//! [^1]: ADR-0125, the control plane names the seed set of a destination field, decision D1. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
//! [^2]: Testing rules, section 2. `.agents/rules/testing.md`
//! [^3]: Testing rules, section 2a. `.agents/rules/testing.md`

use cachette_core::{Axial, Entity, FactionId, World, WorldConfig};

/// How many tiles the world holds on a side.
const EXTENT: u32 = 256;

/// How many factions the world holds.
const FACTIONS: u16 = 3;

/// The seed of the fixture world.
const SEED: u64 = 7;

/// The plane that the first test re-aims.
const FIRST: u16 = 0;

/// The plane that the first test leaves alone.
const SECOND: u16 = 1;

/// Builds a world that sends nothing of its own.
fn world() -> World {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: SEED,
        faction_count: FACTIONS,
        unit_capacity: 1024,
    })
    .expect("the shape must describe a world");
    for faction in 0..FACTIONS {
        world.set_externally_controlled(FactionId(faction), true);
    }
    world
}

/// Returns the first address at or after one column and row that admits a
/// unit.
///
/// The generator decides where the water lies, so a test that named a fixed
/// address would fail when the generator changed. This walk states the
/// property the test needs and finds the ground that holds it.
fn passable_from(world: &World, column: i32, row: i32) -> Axial {
    for step in 0..EXTENT as i32 {
        let address = Axial::new(column + step, row);
        if world.admits_a_unit(address) {
            return address;
        }
    }
    panic!("the row from {column},{row} holds no ground that admits a unit");
}

/// Spawns one soldier of a faction at an address that admits it.
fn spawn(world: &mut World, column: i32, row: i32, faction: u16) -> (Entity, Axial) {
    let address = passable_from(world, column, row);
    let unit = world
        .spawn_soldier(address, FactionId(faction))
        .expect("the ground must take a soldier");
    (unit, address)
}

/// Returns the direction of every cell of one plane.
fn plane(world: &World, number: u16) -> Vec<Option<u8>> {
    let field = world.destination_field();
    (0..field.cells().tile_count())
        .map(|cell| field.direction(number, cell).flatten())
        .collect()
}

/// Re-aiming one plane changes that plane and leaves another alone.
#[test]
fn re_aiming_one_plane_changes_that_plane_and_no_other() {
    let mut world = world();
    let (walker, _) = spawn(&mut world, 8, 8, 0);
    let (other, _) = spawn(&mut world, 8, 24, 1);
    let near = passable_from(&world, 16, 16);
    let far = passable_from(&world, 200, 200);
    let elsewhere = passable_from(&world, 16, 200);

    world
        .send_units_to(&[walker], &[near], FIRST)
        .expect("the send must run");
    world
        .send_units_to(&[other], &[far], SECOND)
        .expect("the send must run");
    world.step(1).expect("the step must run");
    let first_before = plane(&world, FIRST);
    let second_before = plane(&world, SECOND);

    // **The fixture must reach the case.** A plane whose every cell held no
    // direction would pass both assertions below without the field saying
    // anything.
    assert!(
        first_before.iter().any(Option::is_some),
        "the first plane steers no cell, so this test measured its fixture"
    );
    assert!(
        second_before.iter().any(Option::is_some),
        "the second plane steers no cell, so this test measured its fixture"
    );

    world
        .send_units_to(&[walker], &[elsewhere], FIRST)
        .expect("the send must run");
    world.step(1).expect("the step must run");

    assert!(
        plane(&world, FIRST) != first_before,
        "the plane the caller re-aimed carries the direction of the place it left"
    );
    assert_eq!(
        plane(&world, SECOND),
        second_before,
        "the plane the caller left alone changed"
    );
}

/// Sending one plane to the set it already holds changes nothing.
#[test]
fn sending_a_plane_the_set_it_already_holds_changes_no_direction() {
    let mut world = world();
    let (walker, _) = spawn(&mut world, 8, 8, 0);
    let near = passable_from(&world, 16, 16);

    world
        .send_units_to(&[walker], &[near], FIRST)
        .expect("the send must run");
    world.step(1).expect("the step must run");
    let before = plane(&world, FIRST);
    assert!(
        before.iter().any(Option::is_some),
        "the plane steers no cell, so this test measured its fixture"
    );

    world
        .send_units_to(&[walker], &[near], FIRST)
        .expect("the send must run");
    world.step(1).expect("the step must run");

    assert_eq!(
        plane(&world, FIRST),
        before,
        "a send of the set the plane already held changed the field"
    );
}
