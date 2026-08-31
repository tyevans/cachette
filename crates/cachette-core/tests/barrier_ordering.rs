//! The rebuild runs after the structural apply, and the step proves it.
//!
//! The derived unit structure rebuilds at the frame barrier, after the
//! structural apply.[^1] The record calls that ordering a decision and not an
//! implementation detail, and one of its consequences rests on it: every
//! identity in the unit array is live for the whole frame, so a reader's
//! resolution cannot fail during a frame.[^2]
//!
//! **The ordering was between one operation and nothing.** For a long time no
//! structural apply existed in the step, so the rebuild was last because
//! nothing followed it. The admission step put an apply in the frame, and the
//! ordering became real. Nothing failed when it was reversed, because a later
//! caller refreshed the structure again and repaired it quietly.
//!
//! These tests make the ordering fail loudly. A structure left stale by a
//! barrier in the wrong order is stale when the step ends, and that is what
//! they read. A comment is not the mechanism this project accepts for this
//! class of fact.
//!
//! The tests see only the public crate API.[^3]
//!
//! # References
//!
//! [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
//! [^2]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, the consequences. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
//! [^3]: Testing policy. `docs/TESTING.md`

use cachette_core::{Axial, Entity, FactionId, World, WorldConfig};

/// The extent of the fixture world.
///
/// The extent is wider than the coarsest lattice spacing of the generator, so
/// the world holds open ground as well as water.
const EXTENT: u32 = 96;

/// The seed of the fixture world.
const SEED: u64 = 0x0cac_4e77_0030;

/// Builds a world and puts soldiers on the open ground of it.
fn peopled() -> (World, Vec<Entity>) {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: SEED,
        faction_count: 3,
    })
    .expect("the extent describes a world");
    let grid = world.grid();
    let open: Vec<Axial> = (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| world.admits_a_unit(*address))
        .collect();
    assert!(open.len() > 128, "the seed left {} open tiles", open.len());

    let mut kept = Vec::new();
    for (ordinal, address) in open.iter().enumerate().step_by(11) {
        kept.push(
            world
                .spawn_soldier(*address, FactionId((ordinal % 3) as u16))
                .expect("the open tile admits a unit"),
        );
    }
    assert!(kept.len() > 32, "the fixture holds {} soldiers", kept.len());
    (world, kept)
}

/// Asserts that the step left the derived structure describing the arena.
///
/// This is the ordering, read from outside. A rebuild that ran before the
/// structural apply leaves the structure describing the arena as it was, and
/// the arena has moved since, so it is stale when the step ends.
fn assert_fresh_after_a_step(world: &World, frame: u64) {
    assert!(
        world.bridge().describes(world.soldiers()).is_ok(),
        "the step left the derived structure stale after frame {frame}, \
         so the rebuild ran before the structural apply"
    );
    assert_eq!(
        world.bridge().check_invariants(world.soldiers()),
        Ok(true),
        "the derived structure disagrees with the arena after frame {frame}"
    );
}

#[test]
fn the_step_leaves_the_derived_structure_fresh() {
    let (mut world, _) = peopled();
    for frame in 0..8 {
        world.step(2).expect("the step must run");
        assert_fresh_after_a_step(&world, frame);
    }
}

#[test]
fn the_fixture_moves_a_unit_in_every_frame() {
    // A frame that moved nothing leaves the structure fresh whatever order
    // the barrier ran in, so the assertion would pass on a world that never
    // exercised it. The population must actually move.
    let (mut world, kept) = peopled();
    for frame in 0..8 {
        let before: Vec<Axial> = kept
            .iter()
            .map(|soldier| world.soldiers().address(*soldier).expect("alive"))
            .collect();
        world.step(2).expect("the step must run");
        let moved = kept
            .iter()
            .zip(&before)
            .filter(|(soldier, start)| world.soldiers().address(**soldier).as_ref() != Some(*start))
            .count();
        assert!(
            moved > 0,
            "frame {frame} moved no unit, so the barrier ordering is untested"
        );
    }
}

#[test]
fn no_dead_identity_reaches_the_unit_array() {
    // The case the record names. A caller despawns between two frames, the
    // step runs, and the array must name only live entities afterwards.
    //
    // The structure check reads every unit of every block and asks the arena
    // whether it is live, so a dead identity anywhere fails it.
    let (mut world, kept) = peopled();
    world.step(1).expect("the step must run");

    // Kill every third soldier, which leaves stale identities behind and
    // moves the free queue.
    let mut dead = Vec::new();
    for soldier in kept.iter().step_by(3) {
        assert!(world.despawn_soldier(*soldier));
        dead.push(*soldier);
    }
    assert!(dead.len() > 8, "the test killed only {}", dead.len());

    for frame in 0..4 {
        world.step(2).expect("the step must run");
        assert_fresh_after_a_step(&world, frame);
        for soldier in &dead {
            assert!(
                !world.soldiers().contains(*soldier),
                "a despawned identity came back after frame {frame}"
            );
        }
        assert!(world.check_invariants());
    }
}

#[test]
fn a_despawn_between_frames_is_visible_to_the_next_step() {
    // The other half of the ordering. A structural change made outside a
    // frame has passed no barrier, and the step opens by giving it one. A
    // step that read the stale structure would admit units against an
    // occupancy that counts the dead.
    let (mut world, kept) = peopled();
    world.step(1).expect("the step must run");
    let before = world.soldiers().len();

    let doomed = kept[0];
    let tile = world
        .soldiers()
        .address(doomed)
        .expect("the soldier is alive");
    let standing = world
        .soldier_count_on(tile)
        .expect("the structure answers before the despawn");
    assert!(standing > 0);

    assert!(world.despawn_soldier(doomed));
    world.step(1).expect("the step must run");

    assert_eq!(
        world.soldiers().len(),
        before - 1,
        "the population did not fall by one"
    );
    assert_fresh_after_a_step(&world, 0);
    assert!(
        !world.soldiers().contains(doomed),
        "the despawned identity is alive again"
    );
}
