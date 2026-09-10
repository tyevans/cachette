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
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        ..WorldConfig::DEFAULT
    })
    .expect("the extent describes a world");
    // The choice interval is not the subject of this file. A unit takes an
    // intent at the interval its level 1 cell schedules, and it does not move
    // before it has one, so a test about movement sets the interval to every
    // tick.[^C]
    //
    // [^C]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D4. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
    world
        .set_choice_schedule(0)
        .expect("the exponent is inside the range");
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

/// Returns the soldiers of the watched set that a storm has not taken.
///
/// A storm ends a unit that stands in the open, and every soldier of this
/// fixture stands in the open.[^1] The filter drops the units the storm log of
/// the last step names, and no others. A filter that dropped every unit the
/// world no longer holds would pass on a world that lost them all for another
/// reason.
///
/// # References
///
/// [^1]: Findings register, FND-725. `docs/FINDINGS.md`
fn survivors_of_the_last_step(world: &World, watched: &[Entity]) -> Vec<Entity> {
    let taken: Vec<u64> = world
        .units_lost_to_storms()
        .iter()
        .map(|lost| lost.unit)
        .collect();
    watched
        .iter()
        .copied()
        .filter(|soldier| !taken.contains(&soldier.to_bits()))
        .collect()
}

#[test]
fn the_step_leaves_the_derived_structure_fresh() {
    let (mut world, _) = peopled();
    for frame in 0..8 {
        world.step(2).expect("the step must run");
        assert_fresh_after_a_step(&world, frame);
    }
}

/// The barrier ordering is only under test in a frame that moved somebody.
///
/// A frame that moved nothing leaves the derived structure fresh whatever
/// order the barrier ran in, so the assertion of the test above would pass on
/// a world that never exercised it. This test states that the population
/// moves.
///
/// **A storm ends a unit that stands in the open, and the watched set holds
/// units on open ground.**[^1] The run reads the log of what the storms took
/// and drops the units the log names, and no others. A unit that leaves the
/// world without the log naming it fails the run here, rather than through a
/// bare read on the next frame. The watched set must stay large enough to
/// carry the claim, so the run states a floor on it.
///
/// # References
///
/// [^1]: Findings register, FND-725. `docs/FINDINGS.md`
#[test]
fn the_fixture_moves_a_unit_in_every_frame() {
    let (mut world, kept) = peopled();
    let mut watched = kept;
    for frame in 0..8 {
        assert!(
            watched.len() > 8,
            "the storms left {} watched soldiers before frame {frame}, \
             so the frame cannot carry the claim",
            watched.len()
        );
        let before: Vec<(Entity, Axial)> = watched
            .iter()
            .map(|soldier| {
                (
                    *soldier,
                    world
                        .soldiers()
                        .address(*soldier)
                        .expect("a watched soldier stands somewhere"),
                )
            })
            .collect();
        world.step(2).expect("the step must run");
        let taken: Vec<u64> = world
            .units_lost_to_storms()
            .iter()
            .map(|lost| lost.unit)
            .collect();
        let survivors: Vec<(Entity, Axial)> = before
            .into_iter()
            .filter(|(soldier, _)| !taken.contains(&soldier.to_bits()))
            .collect();
        for (soldier, _) in &survivors {
            assert!(
                world.soldiers().contains(*soldier),
                "soldier {soldier:?} left the world on frame {frame}, and the storm log of \
                 that frame names {taken:?}"
            );
        }
        let moved = survivors
            .iter()
            .filter(|(soldier, start)| world.soldiers().address(*soldier).as_ref() != Some(start))
            .count();
        assert!(
            moved > 0,
            "frame {frame} moved none of {} surviving soldiers, so the barrier ordering is \
             untested",
            survivors.len()
        );
        watched = survivors.into_iter().map(|(soldier, _)| soldier).collect();
    }
}

/// The case the record names.
///
/// A caller despawns between two frames, the step runs, and the array must
/// name only live entities afterwards. The structure check reads every unit of
/// every block and asks the arena whether it is live, so a dead identity
/// anywhere fails it.
///
/// **The caller may only despawn a unit the world still holds.** The first
/// step runs under the same sky as the rest, so the run drops the units the
/// storm log names before it chooses whom to kill.
#[test]
fn no_dead_identity_reaches_the_unit_array() {
    let (mut world, kept) = peopled();
    world.step(1).expect("the step must run");
    let alive = survivors_of_the_last_step(&world, &kept);

    let mut dead = Vec::new();
    for soldier in alive.iter().step_by(3) {
        assert!(
            world.despawn_soldier(*soldier),
            "the world no longer holds soldier {soldier:?}, and no storm took it"
        );
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

/// The other half of the ordering.
///
/// A structural change made outside a frame has passed no barrier, and the
/// step opens by giving it one. A step that read the stale structure would
/// admit units against an occupancy that counts the dead.
///
/// **The population falls by the despawn and by the storm log, and by nothing
/// else.** The run subtracts the exact count the engine reports it took, so
/// the equality stays exact under any sky.
#[test]
fn a_despawn_between_frames_is_visible_to_the_next_step() {
    let (mut world, kept) = peopled();
    world.step(1).expect("the step must run");
    let alive = survivors_of_the_last_step(&world, &kept);
    let before = world.soldiers().len();

    let doomed = *alive.first().expect("a storm left the fixture no soldier");
    let tile = world
        .soldiers()
        .address(doomed)
        .expect("the soldier the storm log left alone stands somewhere");
    let standing = world
        .soldier_count_on(tile)
        .expect("the structure answers before the despawn");
    assert!(standing > 0);

    assert!(world.despawn_soldier(doomed));
    world.step(1).expect("the step must run");
    let taken = world.units_lost_to_storms().len() as u32;

    assert_eq!(
        world.soldiers().len(),
        before - 1 - taken,
        "the population fell by something other than the despawn and the {taken} units the \
         storm log names"
    );
    assert_fresh_after_a_step(&world, 0);
    assert!(
        !world.soldiers().contains(doomed),
        "the despawned identity is alive again"
    );
}
