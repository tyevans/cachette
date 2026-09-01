//! A watcher can see where each faction founded, and can judge the distance
//! between them.
//!
//! A finding records the failure these tests exist to prevent: the word
//! "watcher" covers two interfaces, the library and the window, and an item
//! can satisfy its whole list against the first while the second shows
//! nothing.[^1] Every test here reads the pixels of the window or the lines
//! of the panel.
//!
//! # The fixture
//!
//! The fixture is crowded on purpose. A four-faction world put its foundings
//! tens of tiles apart by chance once, and a distance test then stayed green
//! with the whole rule removed.[^2] This world is small enough that the
//! minimum distance refuses at least one faction, and the fixture asserts
//! that it did.
//!
//! A founded place is history. The world holds no record that a place was
//! founded, so the mark comes from the outcomes the caller kept.[^3]
//!
//! # References
//!
//! [^1]: Findings register, FND-100. `docs/FINDINGS.md`
//! [^2]: Findings register, FND-107. `docs/FINDINGS.md`
//! [^3]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`

// An integration test is its own crate, so the allowance at the viewer's
// crate root does not reach it. ADR-0067 D3 puts the float boundary at the
// viewer, and a camera is a viewer value.
#![allow(clippy::disallowed_types)]

use std::time::Duration;

use cachette_core::founding::FoundingOutcome;
use cachette_core::{Axial, World, WorldConfig};
use cachette_view::{draw_frame, paint, Camera, Canvas, Metrics};

/// The number of people each founding starts with.
const GROUP: u32 = 12;

/// The factions the fixture asks the engine to seat.
const FACTIONS: u16 = 6;

/// The extent of the crowded world.
///
/// The world is small against the minimum distance the engine keeps between
/// two foundings, so a run for six factions cannot seat them all. The fixture
/// asserts that outcome rather than trusting this number.
const EXTENT: u32 = 28;

/// Builds a world too small to seat every faction.
fn crowded_world() -> (World, Vec<FoundingOutcome>) {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: 3,
        faction_count: FACTIONS,
    })
    .expect("the extent describes a world");
    let outcomes = world.found_run_for_every_faction(GROUP);
    world.rebuild_bridge(1).expect("the rebuild must run");

    let seated = outcomes.iter().filter(|out| out.is_seated()).count();
    let refused = outcomes.len() - seated;
    assert!(
        seated > 1,
        "the run seated {seated} factions, and a picture of one mark proves \
         nothing about a second",
    );
    assert!(
        refused > 0,
        "every faction found a place, so the panel row for a refusal never \
         receives the case it exists for",
    );
    (world, outcomes)
}

/// Returns measurements that repeat, so that no test reads a clock.
fn measurements() -> Metrics {
    Metrics::fixed(
        1,
        1,
        Duration::from_micros(100),
        Duration::from_micros(200),
        Duration::from_millis(10),
    )
}

/// Draws the whole frame over a window that covers the world.
fn frame(world: &World, outcomes: &[FoundingOutcome]) -> (Canvas, Camera) {
    let mut canvas = Canvas::new(760, 760);
    let camera = Camera::fitting(world, &canvas);
    draw_frame(world, camera, &measurements(), outcomes, &mut canvas).expect("the world draws");
    (canvas, camera)
}

/// Draws the whole frame with one place at the middle of the window.
///
/// The panel sits at the left margin and paints over the picture, so a place
/// near the left edge of the window would sit under it. A test that skipped
/// such a place would pass because of the panel rather than in spite of
/// it.[^1] Putting the place at the middle keeps every mark in view.
///
/// The camera is not clamped, because clamping is a policy for the person who
/// scrolls and would pull a place near the world edge back under the panel.
///
/// # References
///
/// [^1]: Findings register, FND-093. `docs/FINDINGS.md`
fn frame_at(world: &World, outcomes: &[FoundingOutcome], place: Axial) -> (Canvas, Camera) {
    let mut canvas = Canvas::new(760, 900);
    let camera = Camera::at_tile_size(20.0).looking_at(place, &canvas);
    draw_frame(world, camera, &measurements(), outcomes, &mut canvas).expect("the world draws");
    (canvas, camera)
}

/// Reports whether the ring around one place holds a colour.
///
/// The ring surrounds the tile, so the search covers a square wider than one
/// tile and skips the middle of it. A search over the middle alone would find
/// the ground and never the mark.
fn ring_holds(canvas: &Canvas, camera: Camera, place: Axial, colour: u32) -> bool {
    let (x, y) = camera.centre_of(place);
    let reach = ((camera.tile_width * 1.2) as i32).max(5);
    let (cx, cy) = (x as i32, y as i32);
    for row in cy - reach..=cy + reach {
        for column in cx - reach..=cx + reach {
            if row < 0 || column < 0 {
                continue;
            }
            let (row, column) = (row as usize, column as usize);
            if row >= canvas.height() || column >= canvas.width() {
                continue;
            }
            if canvas.pixels()[row * canvas.width() + column] == colour {
                return true;
            }
        }
    }
    false
}

#[test]
fn the_window_marks_each_place_a_faction_founded() {
    // The window, not the library. A watcher must find the place on the
    // picture.
    let (world, outcomes) = crowded_world();

    for outcome in &outcomes {
        let Some(founding) = outcome.founding() else {
            continue;
        };
        let (canvas, camera) = frame_at(&world, &outcomes, founding.place());
        assert!(
            ring_holds(
                &canvas,
                camera,
                founding.place(),
                paint::founding_core_colour()
            ),
            "the place faction {} founded carries no mark",
            outcome.faction().0,
        );
    }
}

#[test]
fn each_mark_carries_the_colour_of_the_faction_that_founded() {
    // The mark comes from the one faction colour table the viewer owns. A
    // mark in one colour for every faction would pass the test above.
    let (world, outcomes) = crowded_world();

    for outcome in &outcomes {
        let Some(founding) = outcome.founding() else {
            continue;
        };
        let (canvas, camera) = frame_at(&world, &outcomes, founding.place());
        assert!(
            ring_holds(
                &canvas,
                camera,
                founding.place(),
                paint::faction_colour(outcome.faction())
            ),
            "the mark at the place of faction {} does not carry its colour",
            outcome.faction().0,
        );
    }
}

#[test]
fn the_frame_marks_one_place_for_each_seated_faction() {
    // A count of the marks the pass painted. A pass that marked a refused
    // faction would state a place that nothing founded.
    let (world, outcomes) = crowded_world();
    let (canvas, _) = frame(&world, &outcomes);
    let seated = outcomes.iter().filter(|out| out.is_seated()).count();

    assert_eq!(
        canvas.foundings_marked() as usize,
        seated,
        "the frame painted {} marks for {seated} foundings",
        canvas.foundings_marked(),
    );
}

#[test]
fn the_frame_marks_nothing_when_the_caller_founded_nothing() {
    // A caller that founded nothing gets a picture with no mark, rather than
    // a mark at the origin.
    let (world, _) = crowded_world();
    let (canvas, _) = frame(&world, &[]);

    assert_eq!(canvas.foundings_marked(), 0);
}

#[test]
fn the_mark_leaves_the_window_when_the_person_scrolls_away() {
    // The mark follows the camera, because it is drawn at the place the
    // camera puts it. A mark pinned to the canvas would stay.
    let (world, outcomes) = crowded_world();
    let (whole, _) = frame(&world, &outcomes);
    assert!(whole.foundings_marked() > 0);

    let mut canvas = Canvas::new(120, 120);
    let camera = Camera::at_tile_size(20.0)
        .looking_at(Axial::new(EXTENT as i32 - 2, EXTENT as i32 - 2), &canvas)
        .clamped(&world, &canvas);
    draw_frame(&world, camera, &measurements(), &outcomes, &mut canvas).expect("the world draws");

    assert!(
        canvas.foundings_marked() < whole.foundings_marked(),
        "the frame still marked {} places with the window off them",
        canvas.foundings_marked(),
    );
}

#[test]
fn the_panel_names_each_faction_that_founded_and_each_that_did_not() {
    // A panel that listed the foundings alone would say nothing about a
    // faction that found no place, and a watcher would read a short list and
    // learn nothing from the gap.
    let (world, outcomes) = crowded_world();
    let mut canvas = Canvas::new(760, 760);
    let camera = Camera::fitting(&world, &canvas);

    let readout = draw_frame(&world, camera, &measurements(), &outcomes, &mut canvas)
        .expect("the world draws");

    let seated: Vec<_> = outcomes.iter().filter(|out| out.is_seated()).collect();
    let refused: Vec<_> = outcomes.iter().filter(|out| !out.is_seated()).collect();
    assert_eq!(
        readout.foundings().len(),
        seated.len(),
        "the panel dropped a founding the caller holds",
    );
    assert_eq!(
        readout.refusals().len(),
        refused.len(),
        "the panel dropped a faction the run refused",
    );
    for (report, outcome) in readout.foundings().iter().zip(&seated) {
        assert_eq!(
            report.faction(),
            outcome.faction(),
            "the panel gives a founding to the wrong faction",
        );
        assert_eq!(
            report.place(),
            outcome.founding().expect("seated").place(),
            "the panel names a place the faction did not take",
        );
    }
    for (stated, outcome) in readout.refusals().iter().zip(&refused) {
        assert_eq!(stated.0, outcome.faction());
    }
}

#[test]
fn the_panel_grows_when_the_caller_holds_the_outcomes() {
    // The section must reach the panel. A readout that held the outcomes and
    // drew none of them would pass every assertion above.
    let (world, outcomes) = crowded_world();
    let mut with = Canvas::new(760, 900);
    let mut without = Canvas::new(760, 900);
    let camera = Camera::fitting(&world, &with);

    let stated =
        draw_frame(&world, camera, &measurements(), &outcomes, &mut with).expect("the world draws");
    let silent =
        draw_frame(&world, camera, &measurements(), &[], &mut without).expect("the world draws");

    let (_, _, _, tall) = cachette_view::hud::bounds(&stated);
    let (_, _, _, short) = cachette_view::hud::bounds(&silent);
    assert!(
        tall > short,
        "the panel is {tall} pixels tall with the outcomes and {short} \
         without them, so the section reached no line",
    );
    assert_ne!(
        with.pixels(),
        without.pixels(),
        "the outcomes painted nothing",
    );
}

#[test]
fn a_watcher_can_judge_the_distance_between_two_foundings() {
    // The picture must separate the places. Two marks at one place would
    // pass a test that only looked for a mark.
    let (world, outcomes) = crowded_world();
    let (canvas, camera) = frame(&world, &outcomes);
    assert!(canvas.foundings_marked() > 1);

    let places: Vec<Axial> = outcomes
        .iter()
        .filter_map(|out| out.founding().map(cachette_core::Founding::place))
        .collect();
    for (index, place) in places.iter().enumerate() {
        for other in places.iter().skip(index + 1) {
            assert_ne!(place, other, "two factions founded at one place");
            let (ax, ay) = camera.centre_of(*place);
            let (bx, by) = camera.centre_of(*other);
            let apart = ((ax - bx).abs()).max((ay - by).abs());
            assert!(
                apart > camera.tile_width * 2.0,
                "two marks stand {apart} pixels apart, so they overlap and a \
                 watcher cannot judge the distance",
            );
        }
    }
}

#[test]
fn a_drawn_frame_leaves_the_founded_world_where_it_found_it() {
    // The viewer reads the world and writes nothing to it. The state hash is
    // the check a reviewer can run, and it must not move over a draw.
    let (world, outcomes) = crowded_world();
    let hash = world.state_hash();

    let (canvas, _) = frame(&world, &outcomes);
    assert!(
        canvas.foundings_marked() > 0,
        "the frame marked nothing, so the comparison proves nothing",
    );

    assert_eq!(hash, world.state_hash(), "the drawing moved the world");
}
