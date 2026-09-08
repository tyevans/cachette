//! The destination field describes the last send of the frame.
//!
//! # What this holds
//!
//! The engine steers a sent unit from a field over level 1 cells and a finer
//! field over the tiles of a block. Both come from one seed set. The control
//! plane names that set through the send verb, and the built-in controller
//! calls the same verb.[^1] [^2]
//!
//! **The step derives the field once, after the controller.** The controller
//! sends several times in one frame, and the barrier of the frame derives the
//! field before it, so a frame that took three orders derived the whole field
//! four times and read three of them never. The last derivation is the one
//! that survives, so the step makes that one and no other.[^3]
//!
//! The derivation is invisible from the outside. Nothing fails when it goes
//! missing, and a unit then reads a direction that describes a seed set the
//! world no longer holds.[^4] These two tests are what fails.
//!
//! # How a reader proves that they can fail
//!
//! One test-only switch removes the derivation the step makes. Both tests
//! below then read a field that no path in a frame derives.
//!
//! ```text
//! ! cargo test --package cachette-core --features probe-stale-destinations \
//!     --test the_step_derives_the_destination_field_after_the_controller
//! ```
//!
//! # The fixture
//!
//! The world is 192 tiles on a side and holds three factions. **The extent is
//! the reason for the size.** A level 1 cell is a block of 32 tiles on a side,
//! so a world of 48 tiles on a side holds four cells and nearly every send
//! ends in the cell it started in. A cell that holds a seed carries no
//! direction, so the coarse field says nothing on that world and a test that
//! read it would measure its fixture.[^5] The larger world puts a sender and
//! its objective in different cells.
//!
//! # References
//!
//! [^1]: ADR-0125, the control plane names the seed set of a destination field, decision D1. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
//! [^2]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
//! [^3]: Findings register, FND-664. `docs/FINDINGS.md`
//! [^4]: Findings register, FND-029. `docs/FINDINGS.md`
//! [^5]: Testing rules, section 2a. `.agents/rules/testing.md`

use cachette_core::{Entity, World, WorldConfig};

/// How many tiles the world holds on a side.
const EXTENT: u32 = 192;

/// How many factions the world holds.
const FACTIONS: u16 = 3;

/// How many frames each test drives.
const FRAMES: u64 = 250;

/// Builds a seeded world whose factions the controller drives.
fn world() -> World {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: 0,
        faction_count: FACTIONS,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the shape must describe a world");
    world.seed_world().expect("the world must seed");
    world.set_win_readers_enabled(true);
    world.set_tick_limit(2500);
    world
}

/// Returns every live unit that the world is sending, with its plane.
fn sent(world: &World) -> Vec<(Entity, u16)> {
    world
        .soldiers()
        .iter()
        .filter_map(|unit| match world.sent_to(unit) {
            Some(Some(plane)) => Some((unit, plane)),
            _ => None,
        })
        .collect()
}

/// Returns the direction of every plane and every cell of the coarse field.
fn field(world: &World) -> Vec<Option<u8>> {
    let field = world.destination_field();
    let mut directions = Vec::new();
    for plane in 0..field.plane_count() {
        for cell in 0..field.cells().tile_count() {
            directions.push(field.direction(plane, cell).flatten());
        }
    }
    directions
}

/// A unit the controller sent reads a direction from the field.
///
/// The test drives the engine and reads the public field. It never sends
/// anything itself, so the controller is the only thing that seeds a plane.
/// A step that leaves the field underived steers nobody.
#[test]
fn a_unit_the_controller_sent_reads_a_direction_from_the_field() {
    let mut world = world();
    let mut steered = 0usize;
    let mut blind = 0usize;
    for _ in 0..FRAMES {
        world.step(1).expect("the step must run");
        for (unit, plane) in sent(&world) {
            let Some(address) = world.soldiers().address(unit) else {
                continue;
            };
            if matches!(world.destination_direction(plane, address), Some(Some(_))) {
                steered += 1;
            } else {
                blind += 1;
            }
        }
    }
    // **The fixture must reach the case.** A world whose controller sent
    // nobody would pass the assertion below without deriving anything.
    assert!(
        steered + blind > 0,
        "the controller sent nobody in {FRAMES} frames, so this test measured its fixture"
    );
    assert!(
        steered > 0,
        "no unit of the {} the controller sent read a direction from the field",
        steered + blind
    );
}

/// The field the step leaves equals the field a fresh derivation gives.
///
/// The public rebuild derives the field from the seed set the world holds. A
/// step that derived the field before its last send leaves a field that the
/// rebuild changes.
#[test]
fn the_field_the_step_leaves_equals_a_fresh_derivation() {
    let mut world = world();
    let mut moved = 0usize;
    let mut previous = field(&world);
    for frame in 0..FRAMES {
        world.step(1).expect("the step must run");
        let left = field(&world);
        world.rebuild_pyramid(1).expect("the rebuild must run");
        let fresh = field(&world);
        assert!(
            left == fresh,
            "frame {frame} left a field that a fresh derivation changes"
        );
        if left != previous {
            moved += 1;
        }
        previous = left;
    }
    // **The fixture must reach the case.** A field that never changed would
    // pass the assertion above without any send reaching it.
    assert!(
        moved > 0,
        "the field never changed in {FRAMES} frames, so this test measured its fixture"
    );
}
