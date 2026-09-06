//! A unit that moved draws between the two tiles, and the frame names the
//! speed.
//!
//! The engine moves a unit one whole tile in one tick, and the world holds
//! one tick at a time. A viewer that draws a unit between two tiles keeps its
//! own record of where the unit was.[^1]
//!
//! # The fixture
//!
//! The fixture is not the demonstration world. It holds one unit on known
//! ground, and the test moves that unit by the engine's own verb, so the
//! addresses come from the engine and not from the test.[^2]
//!
//! Each test reads the pixels of the frame. None of them is satisfied by a
//! value the library returns.
//!
//! **The tween is put back to a no-op in one test**, and the midpoint
//! assertion then fails. That is the proof that the fixture reaches the
//! case.[^2]
//!
//! # References
//!
//! [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
//! [^2]: Testing Rules, a fixture supplies the input. `.claude/rules/testing.md`

// An integration test is its own crate, so the allowance at the viewer's
// crate root does not reach it. ADR-0067 D3 puts the float boundary at the
// viewer, and a camera is a viewer value.
#![allow(clippy::disallowed_types)]

use cachette_core::{Axial, Entity, FactionId, World, WorldConfig};
use cachette_view::paint::{draw_paced, faction_colour, Camera, Canvas};
use cachette_view::tween::{Motion, Pace, TWEEN_REACH};

/// The side of the fixture world, in tiles.
const SIDE: u32 = 24;

/// The side of the canvas, in pixels.
const CANVAS: usize = 480;

/// The faction of the one unit.
const FACTION: FactionId = FactionId(0);

/// Builds a world that holds one unit, and returns the unit and its tile.
fn one_unit_world() -> (World, Entity, Axial) {
    let mut world = World::new(WorldConfig {
        width: SIDE,
        height: SIDE,
        seed: 11,
        faction_count: 1,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent describes a world");

    let home = open_tiles(&world)
        .into_iter()
        .find(|address| {
            // The unit needs an open neighbour to step to, and a second tile
            // far away for the jump.
            neighbour_of(&world, *address).is_some()
        })
        .expect("the world holds open ground with an open neighbour");
    let unit = world
        .spawn_soldier(home, FACTION)
        .expect("the ground admits a unit");
    world.rebuild_bridge(1).expect("the rebuild must succeed");
    (world, unit, home)
}

/// Returns every address of a world that admits a unit, in index order.
fn open_tiles(world: &World) -> Vec<Axial> {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| world.admits_a_unit(*address))
        .collect()
}

/// Returns an open tile one step from this one.
fn neighbour_of(world: &World, address: Axial) -> Option<Axial> {
    [
        Axial::new(address.q + 1, address.r),
        Axial::new(address.q - 1, address.r),
        Axial::new(address.q, address.r + 1),
        Axial::new(address.q, address.r - 1),
    ]
    .into_iter()
    .find(|other| world.admits_a_unit(*other))
}

/// Returns an open tile further away than the tween reaches.
fn far_from(world: &World, address: Axial) -> Axial {
    open_tiles(world)
        .into_iter()
        .find(|other| address.distance(*other) > TWEEN_REACH)
        .expect("the world holds open ground beyond the reach")
}

/// Moves the unit to another tile, by the engine's own verb.
fn move_unit(world: &mut World, unit: Entity, to: Axial) {
    assert!(
        world.place_soldier(unit, to).expect("the ground admits it"),
        "the identity must name a live unit",
    );
    world.rebuild_bridge(1).expect("the rebuild must succeed");
}

/// Draws one frame at a pace, and returns the canvas.
fn frame(world: &World, camera: Camera, motion: &mut Motion, pace: Pace) -> Canvas<'static> {
    let mut canvas = Canvas::new(CANVAS, CANVAS);
    draw_paced(world, camera, &mut canvas, pace, motion, None)
        .expect("the bridge describes the arena");
    canvas
}

/// Returns the middle of the painted unit, in pixels, as the mean of every
/// pixel that carries the faction colour.
///
/// The disc is a solid colour, so the mean of its pixels is its middle. The
/// test reads the pixels of the frame and never a value the library returns.
fn painted_middle(canvas: &Canvas) -> (f32, f32) {
    let colour = faction_colour(FACTION);
    let (mut across, mut down, mut found) = (0.0f32, 0.0f32, 0u32);
    for row in 0..canvas.height() {
        for column in 0..canvas.width() {
            if canvas.pixels()[row * canvas.width() + column] == colour {
                across += column as f32;
                down += row as f32;
                found += 1;
            }
        }
    }
    assert!(found > 0, "the frame painted no unit");
    (across / found as f32, down / found as f32)
}

/// Asserts that two points lie within one tolerance of each other.
fn near(got: (f32, f32), wanted: (f32, f32), tolerance: f32, what: &str) {
    let apart = ((got.0 - wanted.0).powi(2) + (got.1 - wanted.1).powi(2)).sqrt();
    assert!(
        apart <= tolerance,
        "{what}: the unit painted at ({}, {}) and the point wanted is ({}, {}), \
         which is {apart} pixels away and the tolerance is {tolerance}",
        got.0,
        got.1,
        wanted.0,
        wanted.1,
    );
}

#[test]
fn a_unit_that_moved_one_tile_paints_at_the_midpoint_at_half_a_tick() {
    let (mut world, unit, home) = one_unit_world();
    let camera = Camera::fitting(&world, &cachette_view::FrameSize::new(CANVAS, CANVAS));
    let mut motion = Motion::for_frame(CANVAS, CANVAS);

    // The first frame records where the unit stands. Nothing tweens, because
    // the table held nothing before it.
    let first = frame(&world, camera, &mut motion, Pace::new(0.0, 500));
    near(
        painted_middle(&first),
        camera.centre_of(home),
        2.0,
        "the first frame",
    );

    let next = neighbour_of(&world, home).expect("the fixture chose a tile with a neighbour");
    move_unit(&mut world, unit, next);

    // Half way through the tick the unit stands half way between the tiles.
    let half = frame(&world, camera, &mut motion, Pace::new(0.5, 500));
    let (from_x, from_y) = camera.centre_of(home);
    let (to_x, to_y) = camera.centre_of(next);
    near(
        painted_middle(&half),
        ((from_x + to_x) / 2.0, (from_y + to_y) / 2.0),
        2.0,
        "at half a tick",
    );

    // The next tick opens at a phase of zero, and the unit stands on its new
    // tile.
    let whole = frame(&world, camera, &mut motion, Pace::new(0.0, 500));
    near(
        painted_middle(&whole),
        camera.centre_of(next),
        2.0,
        "at the start of the next tick",
    );
}

#[test]
fn a_still_pace_paints_the_unit_at_its_tile() {
    // This is the tween put back to a no-op. The frame runs the same fixture
    // at a phase of zero, and the midpoint assertion of the test above then
    // fails, which is the proof that the fixture reaches the case.[^1]
    //
    // [^1]: Testing Rules, a fixture supplies the input. `.claude/rules/testing.md`
    let (mut world, unit, home) = one_unit_world();
    let camera = Camera::fitting(&world, &cachette_view::FrameSize::new(CANVAS, CANVAS));
    let mut motion = Motion::for_frame(CANVAS, CANVAS);

    frame(&world, camera, &mut motion, Pace::STILL);
    let next = neighbour_of(&world, home).expect("the fixture chose a tile with a neighbour");
    move_unit(&mut world, unit, next);

    let still = frame(&world, camera, &mut motion, Pace::STILL);
    let painted = painted_middle(&still);
    near(painted, camera.centre_of(next), 2.0, "at a still pace");

    let (from_x, from_y) = camera.centre_of(home);
    let (to_x, to_y) = camera.centre_of(next);
    let midpoint = ((from_x + to_x) / 2.0, (from_y + to_y) / 2.0);
    let apart = ((painted.0 - midpoint.0).powi(2) + (painted.1 - midpoint.1).powi(2)).sqrt();
    assert!(
        apart > 2.0,
        "a no-op tween must not paint at the midpoint, and it painted {apart} pixels from it",
    );
}

#[test]
fn a_unit_that_jumped_past_the_reach_paints_at_its_tile() {
    let (mut world, unit, home) = one_unit_world();
    let camera = Camera::fitting(&world, &cachette_view::FrameSize::new(CANVAS, CANVAS));
    let mut motion = Motion::for_frame(CANVAS, CANVAS);

    frame(&world, camera, &mut motion, Pace::new(0.0, 500));
    let far = far_from(&world, home);
    move_unit(&mut world, unit, far);

    // A jump is a respawn, a conversion, or a placement by a verb. A tween
    // across one would draw a unit crossing ground it never crossed.
    let jumped = frame(&world, camera, &mut motion, Pace::new(0.5, 500));
    near(
        painted_middle(&jumped),
        camera.centre_of(far),
        2.0,
        "after a jump",
    );
}

#[test]
fn a_unit_the_frame_did_not_paint_leaves_the_table() {
    let (mut world, unit, home) = one_unit_world();
    let camera = Camera::fitting(&world, &cachette_view::FrameSize::new(CANVAS, CANVAS));
    let mut motion = Motion::for_frame(CANVAS, CANVAS);

    frame(&world, camera, &mut motion, Pace::new(0.0, 500));
    assert_eq!(motion.len(), 1, "the frame painted one unit");
    assert_eq!(motion.moving_from(unit), Some(home));

    assert!(
        world.despawn_soldier(unit),
        "the identity names a live unit"
    );
    world.rebuild_bridge(1).expect("the rebuild must succeed");

    frame(&world, camera, &mut motion, Pace::new(0.0, 500));
    assert!(
        motion.is_empty(),
        "a unit the frame did not paint must leave the table",
    );
}

#[test]
fn the_table_never_holds_more_units_than_its_bound() {
    // The bound is the caller's. A frame that paints more units than the
    // bound admits records the bound and no more.
    let mut world = World::new(WorldConfig {
        width: SIDE,
        height: SIDE,
        seed: 11,
        faction_count: 1,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent describes a world");

    let open = open_tiles(&world);
    for address in open.iter().take(12) {
        world
            .spawn_soldier(*address, FACTION)
            .expect("the ground admits a unit");
    }
    world.rebuild_bridge(1).expect("the rebuild must succeed");

    let camera = Camera::fitting(&world, &cachette_view::FrameSize::new(CANVAS, CANVAS));
    let bound = 3;
    let mut motion = Motion::bounded(bound);
    let canvas = frame(&world, camera, &mut motion, Pace::new(0.0, 500));

    assert!(
        canvas.soldiers_painted() > u32::try_from(bound).expect("the bound is small"),
        "the fixture must paint more units than the bound admits, and it painted {}",
        canvas.soldiers_painted(),
    );
    assert_eq!(motion.bound(), bound);
    assert_eq!(motion.len(), bound, "the table must stop at its bound");
}

/// Returns the value the panel or the glass stated against a label.
fn value_of(said: &[String], label: &str) -> Option<String> {
    let head = format!("{label}: ");
    said.iter()
        .find(|line| line.starts_with(&head))
        .map(|line| line[head.len()..].to_string())
}

#[test]
fn the_window_and_the_panel_name_each_speed_beside_the_tick() {
    // The caller sends a number and the viewer holds the words, so no text
    // crosses the boundary.[^1] The test drives the whole frame command, and
    // not the word function alone, because a word nothing reaches is a
    // capability nobody invokes.[^2]
    //
    // [^1]: ADR-0094, the caller owns the camera and the pixels, decision D5. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
    // [^2]: Testing Rules, drive the real caller. `.claude/rules/testing.md`
    let (world, _, _) = one_unit_world();
    let camera = Camera::fitting(&world, &cachette_view::FrameSize::new(CANVAS, CANVAS));
    let metrics = cachette_view::Metrics::start();

    for (speed_milli, word) in [
        (0u32, "paused"),
        (250, "x1/4"),
        (500, "x1/2"),
        (1000, "x1"),
        (8000, "x8"),
    ] {
        for overlay in [
            cachette_view::Overlay::Glass { reference: false },
            cachette_view::Overlay::Panel,
        ] {
            let mut canvas = Canvas::new(CANVAS, CANVAS);
            let mut motion = Motion::for_frame(CANVAS, CANVAS);
            let readout = cachette_view::draw_frame_paced(
                &world,
                camera,
                &metrics,
                &[],
                overlay,
                None,
                Pace::new(0.0, speed_milli),
                &mut motion,
                &mut canvas,
            )
            .expect("the bridge describes the arena");

            assert_eq!(readout.speed_milli(), speed_milli);
            assert_eq!(readout.speed_word(), word);
            let said = match overlay {
                cachette_view::Overlay::Panel => cachette_view::hud::says(&readout),
                _ => cachette_view::glass::says(&readout, false),
            };
            assert_eq!(
                value_of(&said, "speed").as_deref(),
                Some(word),
                "the layout must state the speed beside the tick, and it said {said:?}",
            );
            assert!(
                value_of(&said, "tick").is_some(),
                "the speed sits beside the tick",
            );
        }
    }
}
