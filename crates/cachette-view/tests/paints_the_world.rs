//! The viewer draws a world, and the drawing is only a drawing.
//!
//! These tests go through the public interface of the viewer crate. They do
//! not open a window: a window needs a display, and a test that needs a
//! display does not run in continuous integration. The painting is separable
//! from the showing for exactly that reason.
//!
//! # References
//!
//! [^1]: ADR-0067, the viewer reads the world and never writes to it. `docs/adrs/draft/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
//! [^2]: Testing Rules, drive the real caller. `.claude/rules/testing.md`

use cachette_core::{Axial, FactionId, World, WorldConfig};
use cachette_view::{paint, Camera, Canvas};

/// Builds a small world with soldiers on it.
fn world() -> World {
    let mut world = World::new(WorldConfig {
        width: 12,
        height: 8,
        seed: 5,
        faction_count: 3,
    })
    .expect("a small extent describes a world");
    for index in 0..12u32 {
        let q = (index % 12) as i32;
        let r = (index % 8) as i32;
        world
            .spawn_soldier(Axial::new(q, r), FactionId((index % 3) as u16))
            .expect("the address and the faction are valid");
    }
    world
}

#[test]
fn drawing_changes_the_canvas() {
    // A painter that paints nothing passes every test that only checks it
    // did not crash.
    let world = world();
    let mut canvas = Canvas::new(200, 160);
    let camera = Camera::fitting(&world, &canvas);

    let before = canvas.pixels().to_vec();
    paint::draw(&world, camera, &mut canvas);
    assert_ne!(before, canvas.pixels(), "the draw painted nothing");
}

#[test]
fn drawing_paints_more_than_one_colour() {
    // Tiles, soldiers and the background must be distinguishable. A single
    // flat colour would satisfy the test above.
    let world = world();
    let mut canvas = Canvas::new(200, 160);
    let camera = Camera::fitting(&world, &canvas);
    paint::draw(&world, camera, &mut canvas);

    let mut seen: Vec<u32> = canvas.pixels().to_vec();
    seen.sort_unstable();
    seen.dedup();
    assert!(
        seen.len() > 2,
        "the world drew {} colours, so tiles and soldiers are not distinct",
        seen.len()
    );
}

/// Builds the same world with no soldiers on it.
fn empty_world() -> World {
    World::new(WorldConfig {
        width: 12,
        height: 8,
        seed: 5,
        faction_count: 3,
    })
    .expect("a small extent describes a world")
}

#[test]
fn soldiers_are_drawn() {
    // PRD-0002: entities appear on the world, each one on a tile. Removing
    // the soldier pass from the painter left every other test green, so
    // this is the test that holds the requirement.
    let with = world();
    let without = empty_world();
    assert_eq!(without.soldiers().len(), 0);

    let mut painted = Canvas::new(200, 160);
    let mut bare = Canvas::new(200, 160);
    let camera = Camera::fitting(&with, &painted);
    paint::draw(&with, camera, &mut painted);
    paint::draw(&without, camera, &mut bare);

    assert_ne!(
        painted.pixels(),
        bare.pixels(),
        "a world with soldiers drew the same picture as a world without",
    );
}

#[test]
fn a_faction_has_its_own_colour() {
    // Two worlds that differ only in the faction of their soldiers must
    // draw differently, or the picture cannot show who is who.
    let mut first = empty_world();
    let mut second = empty_world();
    for index in 0..6u32 {
        let at = Axial::new(index as i32, 1);
        first
            .spawn_soldier(at, FactionId(0))
            .expect("the address is valid");
        second
            .spawn_soldier(at, FactionId(1))
            .expect("the address is valid");
    }

    let mut left = Canvas::new(200, 160);
    let mut right = Canvas::new(200, 160);
    let camera = Camera::fitting(&first, &left);
    paint::draw(&first, camera, &mut left);
    paint::draw(&second, camera, &mut right);

    assert_ne!(
        left.pixels(),
        right.pixels(),
        "two factions drew one colour",
    );
}

#[test]
fn the_tiles_are_not_one_flat_colour() {
    // The tile shade comes from the tile value. A painter that ignored the
    // value would still pass the colour count test, because the soldiers
    // supply the other colours.
    let world = empty_world();
    let mut canvas = Canvas::new(200, 160);
    let camera = Camera::fitting(&world, &canvas);
    paint::draw(&world, camera, &mut canvas);

    let mut seen: Vec<u32> = canvas.pixels().to_vec();
    seen.sort_unstable();
    seen.dedup();
    assert!(
        seen.len() > 2,
        "the tiles drew {} colours with no soldiers present",
        seen.len()
    );
}

#[test]
fn a_step_changes_what_is_drawn() {
    // The point of the viewer is that a person sees the simulation move. If
    // the picture is the same after a step, it is not showing the world.
    let mut world = world();
    let mut canvas = Canvas::new(200, 160);
    let camera = Camera::fitting(&world, &canvas);

    paint::draw(&world, camera, &mut canvas);
    let before = canvas.pixels().to_vec();

    for _ in 0..4 {
        world.step(2).expect("the step must run");
    }
    paint::draw(&world, camera, &mut canvas);

    assert_ne!(before, canvas.pixels(), "four steps changed no pixel");
}

#[test]
fn drawing_leaves_the_world_unchanged() {
    // ADR-0067 D1: the viewer reads the world and never writes to it. The
    // shared reference makes a write a compile error, so this test proves
    // the weaker thing that a test can prove: nothing observable moved.
    let mut world = world();
    world.step(2).expect("the step must run");

    let hash = world.state_hash();
    let tick = world.tick();
    let live = world.soldiers().len();

    let mut canvas = Canvas::new(200, 160);
    let camera = Camera::fitting(&world, &canvas);
    paint::draw(&world, camera, &mut canvas);
    paint::draw(&world, camera, &mut canvas);

    assert_eq!(hash, world.state_hash());
    assert_eq!(tick, world.tick());
    assert_eq!(live, world.soldiers().len());
    assert!(world.check_invariants());
}

#[test]
fn one_world_drawn_twice_gives_one_picture() {
    // The draw is a pure function of the world and the camera. A draw that
    // read a clock or a counter would fail here.
    let world = world();
    let camera_source = Canvas::new(200, 160);
    let camera = Camera::fitting(&world, &camera_source);

    let mut first = Canvas::new(200, 160);
    let mut second = Canvas::new(200, 160);
    paint::draw(&world, camera, &mut first);
    paint::draw(&world, camera, &mut second);

    assert_eq!(first.pixels(), second.pixels());
}

#[test]
fn two_worlds_from_one_seed_draw_the_same_picture() {
    // The product record requires that the same world from the same seed
    // shows the same behaviour on every run. This is that, at the pixel.
    let mut first = world();
    let mut second = world();
    for _ in 0..6 {
        first.step(1).expect("the step must run");
        second.step(7).expect("the step must run");
    }

    let mut left = Canvas::new(200, 160);
    let mut right = Canvas::new(200, 160);
    let camera = Camera::fitting(&first, &left);
    paint::draw(&first, camera, &mut left);
    paint::draw(&second, camera, &mut right);

    assert_eq!(
        left.pixels(),
        right.pixels(),
        "one seed drew two pictures at different thread counts"
    );
}

#[test]
fn the_camera_skews_the_rhombus() {
    // ADR-0017 D4: the world is a parallelogram on the screen, and the skew
    // belongs to the viewer. A row must shift the column, or the world is
    // being drawn as a rectangle it is not.
    let world = world();
    let canvas = Canvas::new(200, 160);
    let camera = Camera::fitting(&world, &canvas);

    let origin = camera.centre_of(Axial::new(0, 0));
    let one_row_down = camera.centre_of(Axial::new(0, 1));

    assert!(
        one_row_down.0 > origin.0,
        "a row down did not shift right, so the drawing has no skew",
    );
    assert!(one_row_down.1 > origin.1, "a row down did not move down");
}

#[test]
fn a_soldier_outside_the_canvas_is_clipped_rather_than_a_panic() {
    // A camera that does not fit is a viewer mistake, not a crash.
    let world = world();
    let mut canvas = Canvas::new(16, 16);
    let camera = Camera {
        tile_width: 400.0,
        tile_height: 400.0,
        origin_x: -5000.0,
        origin_y: -5000.0,
    };
    paint::draw(&world, camera, &mut canvas);
}

#[test]
fn the_metrics_count_what_happened() {
    // A report that counts nothing would print zeroes and look plausible.
    use cachette_view::{Lap, Metrics};

    let mut metrics = Metrics::start();
    assert_eq!(metrics.ticks(), 0);

    for _ in 0..3 {
        let at = Lap::start();
        metrics.step(at.elapsed());
    }
    let at = Lap::start();
    metrics.draw(at.elapsed());
    metrics.show(at.elapsed());

    assert_eq!(metrics.ticks(), 3);
    // The report writes to standard output and returns nothing. Calling it
    // proves it does not panic on a run with a draw count below the tick
    // count, which is the shape a dropped frame would give.
    metrics.report(96, 12, 2);
}
