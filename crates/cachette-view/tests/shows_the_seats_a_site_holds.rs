//! The viewer shows the positions a site holds and who sits in them.
//!
//! A position is a seat at a site: a named job that one unit holds. The
//! subsystem produced no instance until the pairing pass landed, and a view of
//! a subsystem that produces nothing measures its own fixture.[^1] These tests
//! therefore assert the fixture's own outcome first, from the engine, before
//! any assertion about a surface.
//!
//! Both counts come from the bounded walk the panel already makes over the
//! sites. The walk stops at a fixed number of sites and a site declares a
//! fixed number of seats, so the cost follows neither the world nor the
//! population.[^2]
//!
//! # References
//!
//! [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
//! [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`

// An integration test is its own crate, so the allowance at the viewer's
// crate root does not reach it. ADR-0067 D3 puts the float boundary at the
// viewer, and a camera is a viewer value.
#![allow(clippy::disallowed_types)]

use cachette_core::founding::FoundingOutcome;
use cachette_core::{Axial, Entity, World, WorldConfig};
use cachette_view::{draw_frame, glass, hud, Camera, Canvas, Metrics, Overlay, Readout};

/// The size of the window the tests paint onto.
const WINDOW: (usize, usize) = (512, 512);

/// The extent of the fixture world.
const EXTENT: u32 = 128;

/// The number of people each faction founds with.
const GROUP: u32 = 96;

/// The number of steps the fixture runs before it draws.
///
/// The pairing pass seats units at the positions a site declares. A world
/// drawn before it runs holds seats that nobody sits in, whatever the engine
/// does, so this count is what makes the fixture supply the case.
const STEPS: usize = 60;

/// Builds a founded world, run far enough that the seats are taken.
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

/// Returns the seats the world declares and the seats a unit holds.
///
/// This is the fixture checking itself, read straight from the engine and not
/// through any surface.
fn seats_in_world(world: &World) -> (u32, u32) {
    let arena = world.settlements();
    let mut seats = 0;
    let mut held = 0;
    for site in arena.iter() {
        let Some(rows) = world.site_positions(site) else {
            continue;
        };
        for (index, position) in rows.iter().enumerate() {
            if !position.exists() {
                continue;
            }
            seats += 1;
            // The engine's resolving reader, not the stored field. A position
            // keeps the bits of a holder that died until the pass that
            // releases the dead runs, so the field says "held" and the truth
            // is that nobody sits there.
            if world.position_holder(site, index).is_some() {
                held += 1;
            }
        }
    }
    (seats, held)
}

/// Draws one frame of the fixture and returns the readout.
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
fn the_fixture_holds_a_world_whose_seats_are_taken() {
    // This test is the fixture's own check. Every assertion below would pass
    // on a world with no seat at all, because both sides would be zero.
    let (world, _, _) = drawn(STEPS);

    let (seats, held) = seats_in_world(&world);
    assert!(
        seats > 0,
        "the world declares no position, so every seat assertion here \
         measures the fixture rather than the viewer",
    );
    assert!(
        held > 0,
        "the world declares {seats} positions and no unit holds one, so the \
         held count measures the fixture",
    );
}

#[test]
fn the_readout_states_the_seats_the_engine_holds() {
    let (world, readout, _) = drawn(STEPS);
    let (seats, held) = seats_in_world(&world);

    // The panel walks a bounded number of sites. This fixture founds three,
    // which is under that bound, so the panel sees every one of them.
    assert_eq!(
        readout.seats(),
        seats,
        "the readout and the engine disagree about how many seats exist",
    );
    assert_eq!(
        readout.seats_taken(),
        held,
        "the readout and the engine disagree about how many seats are held",
    );
}

#[test]
fn the_readout_reports_nothing_before_the_pairing_pass_opens_a_seat() {
    // The seats do not exist at the founding. A pass opens them later, and
    // the readout must report what the engine holds at both points rather
    // than a number of its own.
    //
    // A seat with no holder is a different fact from no seat, and the two
    // numbers exist to separate them. Stepping cannot reach that case: the
    // pass that opens the seats fills them in the same tick, and a group
    // small enough to leave a seat empty opens no seat at all. The test below
    // that kills a holder reaches it instead.
    let (world, early, _) = drawn(0);

    let (seats, held) = seats_in_world(&world);
    assert_eq!(seats, 0, "the fixture opened a seat before any step ran");
    assert_eq!(early.seats(), 0, "the readout invented a seat");
    assert_eq!(early.seats_taken(), held, "the readout invented a holder");

    // After the pass the readout tracks the engine rather than staying at the
    // number it first read.
    let (_, later, _) = drawn(STEPS);
    assert!(
        later.seats() > early.seats(),
        "the readout did not follow the engine when the pass opened the seats",
    );
}

#[test]
fn the_seats_do_not_take_a_place_in_the_window() {
    // **The seats hold still.** A pass opens them once and fills them in the
    // same tick, and the count reads the same on every frame after that. The
    // record's test for a place in the window is what the quantity does, not
    // how interesting it is, so the seats fail it and belong in the panel.[^1]
    //
    // The readout still carries them, because the panel draws from the same
    // reading. This test is about where they are shown, not whether they are
    // read.
    //
    // [^1]: ADR-0093, the window shows what changes, decision D1. `docs/adrs/draft/adr-0093-the-window-shows-what-changes.md`
    let (_, readout, _) = drawn(STEPS);
    assert!(readout.seats() > 0, "the fixture declares no seat");

    for reference in [false, true] {
        assert!(
            !glass::says(&readout, reference)
                .iter()
                .any(|line| line == "THE SEATS"),
            "the glass drew a card for a quantity that holds still",
        );
    }
}

#[test]
fn the_panel_states_the_seats_the_glass_left_out() {
    // The test above says where the seats are not. This one says where they
    // are. A quantity that leaves the glass because it never moves must
    // arrive in the panel, and a quantity on neither surface is a loss rather
    // than restraint.
    let (_, readout, _) = drawn(STEPS);
    let expected = format!("{} of {}", readout.seats_taken(), readout.seats());
    assert!(
        hud::says(&readout)
            .iter()
            .any(|line| line.contains(&expected)),
        "the panel does not state the seats the readout holds: {expected}",
    );
}

#[test]
fn the_seat_count_holds_still_once_the_pass_has_run() {
    // This is the evidence for the test above, and it is why that one is not
    // just an assertion of taste. If the seats ever did move frame to frame,
    // this would fail and the placement would have to be reconsidered.
    let (world, outcomes, place) = founded(STEPS);
    let mut canvas = Canvas::new(WINDOW.0, WINDOW.1);
    let camera = Camera::opening()
        .looking_at(place, &canvas)
        .clamped(&world, &canvas);

    let mut seen = Vec::new();
    let mut world = world;
    for _ in 0..5 {
        let readout = draw_frame(
            &world,
            camera,
            &Metrics::start(),
            &outcomes,
            Overlay::Glass { reference: false },
            &mut canvas,
        )
        .expect("the world draws");
        seen.push((readout.seats(), readout.seats_taken()));
        world.step(1).expect("the step must run");
    }
    assert!(seen[0].0 > 0, "the fixture declares no seat");
    assert!(
        seen.iter().all(|held| *held == seen[0]),
        "the seats moved over five frames, so they may earn a place in the \
         window after all: {seen:?}",
    );
}

#[test]
fn the_seat_count_follows_the_sites_and_not_the_window() {
    // The seats are a property of the sites, not of the picture. Two windows
    // of different sizes over one world report the same seats, which is what
    // separates this from a count of the window.
    let (world, outcomes, place) = founded(STEPS);

    let mut small = Canvas::new(256, 256);
    let camera_small = Camera::opening()
        .looking_at(place, &small)
        .clamped(&world, &small);
    let read_small = draw_frame(
        &world,
        camera_small,
        &Metrics::start(),
        &outcomes,
        Overlay::Glass { reference: false },
        &mut small,
    )
    .expect("the world draws");

    let mut large = Canvas::new(768, 768);
    let camera_large = Camera::opening()
        .looking_at(place, &large)
        .clamped(&world, &large);
    let read_large = draw_frame(
        &world,
        camera_large,
        &Metrics::start(),
        &outcomes,
        Overlay::Glass { reference: false },
        &mut large,
    )
    .expect("the world draws");

    assert!(read_small.seats() > 0, "the fixture declares no seat");
    assert_eq!(
        read_small.seats(),
        read_large.seats(),
        "the seat count changed with the size of the window",
    );
    assert_eq!(
        read_small.seats_taken(),
        read_large.seats_taken(),
        "the held count changed with the size of the window",
    );
}

/// Returns one unit that holds a seat, and the world it holds it in.
fn a_seated_unit(world: &World) -> Entity {
    let arena = world.settlements();
    for site in arena.iter() {
        let Some(rows) = world.site_positions(site) else {
            continue;
        };
        for index in 0..rows.len() {
            if let Some(holder) = world.position_holder(site, index) {
                return holder;
            }
        }
    }
    panic!("the fixture seated nobody, so this test measures the fixture");
}

#[test]
fn a_seat_whose_holder_died_is_not_counted_as_held() {
    // **The stored identity is not the question. Whether it resolves is.** A
    // position keeps the bits of its holder after that holder dies, until the
    // pass that releases the dead runs. A viewer that read the field would
    // report a fully staffed site where one is short.
    //
    // This is the only test in this suite that can tell the held count from
    // the seat count. Without it, a reader that counted every seat as held
    // would pass every other assertion here, because the pairing pass fills
    // every seat it opens.
    let (mut world, outcomes, place) = founded(STEPS);

    let (seats_before, held_before) = seats_in_world(&world);
    assert!(held_before > 0, "the fixture seated nobody");

    let victim = a_seated_unit(&world);
    assert!(world.despawn_soldier(victim), "the unit refused to end");

    // No step runs, so the pass that releases the dead has not run and the
    // position still holds the bits of the unit that died.
    let (seats_after, held_after) = seats_in_world(&world);
    assert_eq!(
        seats_after, seats_before,
        "ending a unit removed a seat, which is not what ending a unit does",
    );
    assert_eq!(
        held_after,
        held_before - 1,
        "the seat of a unit that died still counts as held, so the viewer \
         reads the stored field rather than resolving it",
    );

    // The surface says so too, and not only the counter behind it. Ending a
    // unit changes the arena, and the structure that answers "which units
    // stand on this tile" is rebuilt at the barrier, so it is stale until a
    // step or a rebuild runs. The drawing refuses a stale one rather than
    // showing a world without its units.
    world.rebuild_bridge(1).expect("the rebuild must succeed");
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
    assert_eq!(readout.seats(), seats_after, "the readout lost a seat");
    assert_eq!(
        readout.seats_taken(),
        held_after,
        "the readout counted the seat of a unit that died as held",
    );
    assert!(
        readout.seats_taken() < readout.seats(),
        "the fixture did not reach a seat with no holder, so the held column \
         is still unproven",
    );
}
