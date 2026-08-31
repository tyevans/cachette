//! Soldier movement under a keyed draw.
//!
//! Each tick every live soldier chooses one of the six neighbour
//! directions. The choice comes from the counter-based generator, keyed on
//! the tuple of the system, the frame, the entity and the draw index. The
//! same soldier in the same frame gets the same direction however the work
//! was scheduled.[^1]
//!
//! A soldier whose chosen neighbour falls outside the world stays put. The
//! world is a rhombus and it does not wrap.[^2]
//!
//! This covers the intent half of movement only. The step admits every
//! intent, so two soldiers may hold the same tile.[^3]
//!
//! The tests see only the public crate API.[^4]
//!
//! # References
//!
//! [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
//! [^2]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D3. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
//! [^3]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D2, a draft record. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
//! [^4]: Testing policy. `docs/TESTING.md`

use cachette_core::{Axial, Entity, FactionId, World, WorldConfig};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

/// The thread counts that the movement tests run at.
const THREAD_COUNTS: [usize; 3] = [1, 2, 12];

/// Builds a world of the given seed and fills it with soldiers.
///
/// The population is a fixed pattern, so it is the same on every run and at
/// every thread count.
fn peopled(seed: u64) -> (World, Vec<Entity>) {
    let mut world = World::new(WorldConfig {
        width: 12,
        height: 12,
        seed,
        faction_count: 3,
    })
    .expect("the extent must describe a world");
    let grid = world.grid();
    let mut kept = Vec::new();
    for index in 0..48u32 {
        let address = Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32);
        let soldier = world
            .spawn_soldier(address, FactionId((index % 3) as u16))
            .expect("the address and the faction must be valid");
        kept.push(soldier);
    }
    (world, kept)
}

/// Returns the address of each soldier after the frames, in the given order.
fn addresses_after(seed: u64, frames: u64, threads: usize) -> Vec<Axial> {
    let (mut world, kept) = peopled(seed);
    for _ in 0..frames {
        world.step(threads).expect("the step must run");
    }
    assert!(world.check_invariants());
    kept.iter()
        .map(|soldier| {
            world
                .soldiers()
                .address(*soldier)
                .expect("a soldier that nothing despawned is alive")
        })
        .collect()
}

#[test]
fn a_soldier_moves_to_another_tile() {
    // A system that never moves anything passes every equivalence test and
    // is inert. This test fails when the movement system stops moving.
    let (mut world, kept) = peopled(0x5eed);
    let before: Vec<Axial> = kept
        .iter()
        .map(|soldier| world.soldiers().address(*soldier).expect("alive"))
        .collect();
    for _ in 0..4 {
        world.step(2).expect("the step must run");
    }
    let after: Vec<Axial> = kept
        .iter()
        .map(|soldier| world.soldiers().address(*soldier).expect("alive"))
        .collect();
    let moved = before
        .iter()
        .zip(&after)
        .filter(|(start, end)| start != end)
        .count();
    assert!(
        moved > kept.len() / 2,
        "the movement system moved {moved} soldiers of {}",
        kept.len()
    );
}

#[test]
fn a_soldier_steps_to_a_neighbour_and_never_further() {
    // A single frame moves a soldier one tile at most. A larger step would
    // mean the draw reached past the neighbour set.
    let (mut world, kept) = peopled(19);
    let before: Vec<Axial> = kept
        .iter()
        .map(|soldier| world.soldiers().address(*soldier).expect("alive"))
        .collect();
    world.step(3).expect("the step must run");
    for (soldier, start) in kept.iter().zip(&before) {
        let end = world.soldiers().address(*soldier).expect("alive");
        assert!(
            start.distance(end) <= 1,
            "a soldier moved from {start:?} to {end:?} in one frame"
        );
    }
}

#[test]
fn a_soldier_at_a_corner_stays_put_rather_than_wrapping() {
    // The corner tile has neighbours outside the world. A soldier that
    // draws one of them must hold its tile. A wrapping world would put it
    // on the far edge instead.
    let corner = Axial::new(0, 0);
    let mut stayed = 0;
    for seed in 0..64u64 {
        let mut world = World::new(WorldConfig {
            width: 8,
            height: 8,
            seed,
            faction_count: 1,
        })
        .expect("the extent must describe a world");
        let soldier = world
            .spawn_soldier(corner, FactionId(0))
            .expect("the corner is inside the world");
        world.step(1).expect("the step must run");
        let end = world
            .soldiers()
            .address(soldier)
            .expect("the soldier is alive");
        // Whatever the draw chose, the soldier is on the corner or on a
        // neighbour of it. It is never on the far side of the world.
        assert!(
            corner.distance(end) <= 1,
            "the soldier wrapped from {corner:?} to {end:?}"
        );
        if end == corner {
            stayed += 1;
        }
    }
    assert!(
        stayed > 0,
        "no seed drew an outside direction, so the case is untested"
    );
}

#[test]
fn the_direction_changes_from_frame_to_frame() {
    // The key holds the frame. A key without the frame gives one soldier the
    // same direction every tick, so it walks one straight line for ever.
    // This test fails when the frame leaves the key.[^1]
    //
    // [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
    let (mut world, kept) = peopled(0x1234_5678);
    let soldier = kept[0];
    let mut steps = Vec::new();
    let mut previous = world.soldiers().address(soldier).expect("alive");
    for _ in 0..12 {
        world.step(1).expect("the step must run");
        let now = world.soldiers().address(soldier).expect("alive");
        steps.push(Axial::new(now.q - previous.q, now.r - previous.r));
        previous = now;
    }
    let distinct = steps.iter().filter(|step| **step != steps[0]).count();
    assert!(
        distinct > 0,
        "the soldier took the same step twelve times: {steps:?}"
    );
}

#[test]
fn the_generation_of_an_identity_changes_the_direction() {
    // The key holds the entity identity, which pairs the slot index with the
    // generation. A key that holds the slot index alone gives a reused slot
    // the direction of the soldier that died in it. This test fails when the
    // generation leaves the key.[^1]
    //
    // [^1]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    let start = Axial::new(4, 4);
    let mut differed = 0;
    for seed in 0..64u64 {
        let config = WorldConfig {
            width: 10,
            height: 10,
            seed,
            faction_count: 1,
        };

        let mut first = World::new(config).expect("the extent must describe a world");
        let original = first
            .spawn_soldier(start, FactionId(0))
            .expect("the address is inside the world");
        first.step(1).expect("the step must run");

        let mut second = World::new(config).expect("the extent must describe a world");
        let doomed = second
            .spawn_soldier(start, FactionId(0))
            .expect("the address is inside the world");
        assert!(second.despawn_soldier(doomed));
        let reused = second
            .spawn_soldier(start, FactionId(0))
            .expect("the address is inside the world");
        // The respawn takes the same slot at a later generation.
        assert_eq!(reused.index(), doomed.index());
        assert_ne!(reused.generation(), doomed.generation());
        second.step(1).expect("the step must run");

        if first.soldiers().address(original) != second.soldiers().address(reused) {
            differed += 1;
        }
    }
    assert!(
        differed > 0,
        "the generation never changed the direction, so the key ignores it"
    );
}

#[test]
fn every_thread_count_gives_the_same_positions() {
    let expected = addresses_after(0xfeed_face, 6, THREAD_COUNTS[0]);
    for threads in &THREAD_COUNTS[1..] {
        assert_eq!(
            addresses_after(0xfeed_face, 6, *threads),
            expected,
            "the positions differ at {threads} threads"
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        // An integration test has no lib.rs or main.rs above it, so the
        // default source-parallel persistence finds no root and silently
        // disables itself. A failing seed is then never written and never
        // replayed. Name the file, so that a seed which caught a defect runs
        // first on every later run.[^1]
        //
        // [^1]: Findings register, FND-044. `docs/FINDINGS.md`
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/soldier_movement.proptest-regressions"),
        ))),
        cases: 24,
        ..ProptestConfig::default()
    })]

    /// The same seed and the same frame give the same direction for the
    /// same soldier, at every thread count.
    #[test]
    fn the_direction_of_a_soldier_does_not_depend_on_the_thread_count(
        seed in any::<u64>(),
        frames in 1u64..5,
    ) {
        let expected = addresses_after(seed, frames, THREAD_COUNTS[0]);
        for threads in &THREAD_COUNTS[1..] {
            prop_assert_eq!(
                addresses_after(seed, frames, *threads),
                expected.clone(),
                "the positions differ at {} threads", threads
            );
        }
    }

    /// Two worlds of the same seed run the same movement.
    ///
    /// The draw reads no state, so a second world of the same seed lands
    /// every soldier on the same tile.
    #[test]
    fn the_same_seed_gives_the_same_run(seed in any::<u64>(), frames in 1u64..5) {
        prop_assert_eq!(
            addresses_after(seed, frames, 2),
            addresses_after(seed, frames, 2)
        );
    }
}
