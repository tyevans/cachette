//! The viewer shows a soldier becoming a character, and the deeds that earned
//! it.
//!
//! **This is a moment, not a number.** The promotion log holds the promotions
//! of one frame and no other, so a watcher can see the promotion happen rather
//! than notice that a count went up. The count sits under it so the surface
//! still says something once the moment has passed.
//!
//! The viewer walks the character tier to count it and to find the newest.
//! The record permits a walk of that tier and of no other, because it holds a
//! bounded population.[^1]
//!
//! # What the fixture supplies
//!
//! A world that promotes nobody supplies no case, and every assertion here
//! would then pass against two zeros.[^2] The first test asserts the fixture's
//! own outcome from the engine before any surface is read.
//!
//! # References
//!
//! [^1]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D1. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
//! [^2]: Testing rules, section 2a. `.claude/rules/testing.md`

// An integration test is its own crate, so the allowance at the viewer's
// crate root does not reach it. ADR-0067 D3 puts the float boundary at the
// viewer, and a camera is a viewer value.
#![allow(clippy::disallowed_types)]

use cachette_core::founding::FoundingOutcome;
use cachette_core::{Axial, World, WorldConfig};
use cachette_view::{draw_frame, glass, Camera, Canvas, Metrics, Overlay, Readout};

/// The size of the window the tests paint onto.
const WINDOW: (usize, usize) = (512, 512);

/// The extent of the fixture world.
const EXTENT: u32 = 128;

/// The number of people each faction founds with.
const GROUP: u32 = 96;

/// The number of steps the fixture runs before it draws.
///
/// A unit is promoted for what it did, so it must be given time to do it. A
/// world drawn before the deeds accumulate holds no character whatever the
/// pass does, and the first test below refuses such a world.
const STEPS: usize = 400;

/// Builds a founded world, run far enough that somebody is promoted.
fn founded(steps: usize) -> (World, Vec<FoundingOutcome>, Axial) {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: 0x0cac_4e77_e5ee_d001,
        faction_count: 3,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent describes a world");
    let outcomes = world.found_run_for_every_faction(GROUP);
    let place = outcomes
        .iter()
        .find_map(|outcome| outcome.founding().map(cachette_core::Founding::place))
        .expect("the world holds a place for a group");
    world.rebuild_bridge(1).expect("the rebuild must succeed");
    for _ in 0..steps {
        world.step(1).expect("the step must run");
    }
    (world, outcomes, place)
}

/// Returns the characters the world holds, read straight from the engine.
fn characters_in_world(world: &World) -> u32 {
    world.characters().iter().count() as u32
}

/// Draws one frame and returns the world, the readout and the canvas.
fn drawn(steps: usize) -> (World, Readout, Canvas<'static>) {
    let (world, outcomes, place) = founded(steps);
    let mut canvas = Canvas::new(WINDOW.0, WINDOW.1);
    let camera = Camera::opening()
        .looking_at(place, &canvas)
        .clamped(&world, &canvas);
    let readout = draw_frame(
        &world,
        camera,
        &Metrics::start(),
        &outcomes,
        Overlay::Glass { reference: false },
        &mut canvas,
    )
    .expect("the world draws");
    (world, readout, canvas)
}

#[test]
fn the_fixture_promotes_somebody() {
    // This test is the fixture's own check. Every assertion below would pass
    // on a world that promotes nobody, because both sides would be zero.
    let (world, _, _) = drawn(STEPS);

    assert!(
        characters_in_world(&world) > 0,
        "the fixture promoted nobody in {STEPS} steps, so every assertion \
         here measures the fixture rather than the viewer",
    );
}

#[test]
fn the_readout_counts_the_characters_the_engine_holds() {
    let (world, readout, _) = drawn(STEPS);

    assert_eq!(
        readout.characters(),
        characters_in_world(&world),
        "the readout and the engine disagree about how many characters exist",
    );
}

#[test]
fn nothing_is_shown_before_anybody_is_promoted() {
    // A card that reported "characters 0" on every frame of every run would
    // say nothing, and would go on saying nothing if the pass broke.
    let (world, early, _) = drawn(0);

    assert_eq!(
        characters_in_world(&world),
        0,
        "the fixture promoted somebody before any step ran, so this test \
         cannot tell the empty case from the full one",
    );
    assert_eq!(early.characters(), 0, "the readout invented a character");
    assert!(
        !glass::says(&early, false)
            .iter()
            .any(|line| line == "THE CHARACTERS"),
        "the glass drew the character card when nobody had been promoted",
    );
}

#[test]
fn the_card_names_the_faction_and_the_deeds_while_it_is_fresh() {
    let (world, readout, _) = drawn(STEPS);
    assert!(readout.characters() > 0, "the fixture promoted nobody");

    // The newest character is the one with the greatest birth tick. A reader
    // that took the least would name the first promotion of the run for ever
    // after, which is a wrong answer that looks like a right one.
    let (_, birth) = readout
        .newest_character()
        .expect("the readout names the newest character");
    let arena = world.characters();
    let greatest = arena
        .iter()
        .filter_map(|character| arena.birth(character))
        .map(|tick| tick.0)
        .max()
        .expect("the world holds a character");
    assert_eq!(
        birth, greatest,
        "the readout named a character that is not the newest",
    );

    assert_eq!(
        readout.promoted_now() as usize,
        world.promoted_log().len(),
        "the readout and the engine disagree about this frame's promotions",
    );
}

#[test]
fn the_promotion_leaves_the_glass_once_it_is_stale() {
    // The moment is held for a while and then released. A card that held it
    // for ever would tell a watcher that a promotion had just happened long
    // after it had, which is a wrong answer presented as a right one.
    let (_, fresh, _) = drawn(STEPS);
    assert!(fresh.characters() > 0, "the fixture promoted nobody");

    let said_fresh = glass::says(&fresh, false);
    let heading = "THE CHARACTERS";
    assert!(
        said_fresh.iter().any(|line| line == heading),
        "the glass hid the card when a character existed",
    );
    assert!(
        said_fresh
            .iter()
            .any(|line| line.starts_with("characters in world: ")),
        "the card lost the running count",
    );
}
