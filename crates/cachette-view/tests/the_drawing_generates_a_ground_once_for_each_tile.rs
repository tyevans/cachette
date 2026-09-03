//! The drawing generates the ground of a tile once for each tile it paints.
//!
//! The ground of a tile is generated from the seed and the address, and the
//! engine holds no map of it.[^1] The generation is the largest part of what
//! a drawing costs, and at the far zoom the window covers the whole world.
//!
//! The drawing needs the ground twice: once for the colour, and once for the
//! stock the tile started with. It used to ask for the two through two
//! readers that each start from the address, so it generated the ground
//! twice. The second answer was the same answer as the first, so no picture
//! and no test could see it.
//!
//! **The count is the only thing that can see it, and a picture cannot.** The
//! canvas carries a count of the generations this layer asked for, in the
//! same way it carries a count of the holders it read.[^2] The tests below
//! read that count against the count of painted tiles.
//!
//! The count is of this layer. A reader below it that generated a ground of
//! its own would not appear here, and the tests that hold that half live in
//! the core crate.
//!
//! The tests drive the real drawing and see only the public interface.[^3]
//!
//! # References
//!
//! [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
//! [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
//! [^3]: Testing rules, sections 5 and 6. `.claude/rules/testing.md`

use cachette_core::{Axial, World, WorldConfig};
use cachette_view::{paint, Camera, Canvas};

/// The extent of the fixture world.
///
/// The extent is wider than the coarsest lattice spacing of the ground
/// generator, so the world holds every kind of ground. A world of one ground
/// would let a reader that answered from a constant pass.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
const EXTENT: u32 = 128;
/// The seed of the fixture world.
const SEED: u64 = 0x0123_4567_89ab_cdef;
/// The number of zoom steps that the second test takes.
const ZOOM_STEPS: u32 = 16;

/// Builds the fixture world.
fn world_of(seed: u64) -> World {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed,
        faction_count: 2,
        unit_capacity: 64,
    })
    .expect("the extent describes a world");
    // A draw reads the derived structure, and a fresh one is stale.
    world.rebuild_bridge(1).expect("the rebuild must succeed");
    world
}

/// Paints the whole world onto a canvas and returns both.
fn painted(width: usize, height: usize) -> (World, Canvas<'static>) {
    let world = world_of(SEED);
    let mut canvas = Canvas::new(width, height);
    let camera = Camera::fitting(&world, &canvas);
    paint::draw(&world, camera, &mut canvas).expect("the world draws");
    (world, canvas)
}

#[test]
fn the_drawing_generates_one_ground_for_each_tile_it_paints() {
    let (_, canvas) = painted(512, 512);

    assert!(
        canvas.tiles_painted() > 0,
        "the fixture painted nothing, so the count under test is zero for the wrong reason"
    );
    assert_eq!(
        canvas.ground_reads(),
        canvas.tiles_painted(),
        "the drawing must generate the ground of a tile once. It painted {} tiles and asked \
         for {} grounds",
        canvas.tiles_painted(),
        canvas.ground_reads()
    );
}

#[test]
fn the_drawing_generates_one_ground_for_each_tile_at_every_zoom() {
    let world = world_of(SEED);
    let mut canvas = Canvas::new(512, 512);
    let mut camera = Camera::fitting(&world, &canvas);

    // The far camera shows the whole world. Each step in shows fewer tiles.
    let mut last = u32::MAX;
    for step in 0..ZOOM_STEPS {
        paint::draw(&world, camera, &mut canvas).expect("the world draws");
        assert_eq!(
            canvas.ground_reads(),
            canvas.tiles_painted(),
            "at zoom step {step} the drawing painted {} tiles and asked for {} grounds",
            canvas.tiles_painted(),
            canvas.ground_reads()
        );
        assert!(
            canvas.tiles_painted() < last,
            "at zoom step {step} the window covers {} tiles, against {last} before it",
            canvas.tiles_painted()
        );
        last = canvas.tiles_painted();
        camera = camera.zoomed_in(&canvas);
    }
}

#[test]
fn the_count_follows_the_window_and_not_the_world() {
    // One window, at one tile size, over two worlds of different sizes. The
    // count must be the same, because the drawing must read the tiles the
    // window covers and no other.[^1] A world six times as wide holds
    // thirty-six times the tiles, so a sweep of the world would show here.
    //
    // A count that grew with the world would still paint the right picture,
    // which is why the test reads the count and not the pixels.[^2]
    //
    // # References
    //
    // [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    // [^2]: Findings register, FND-206. `docs/FINDINGS.md`
    let mut counts = Vec::new();
    for extent in [EXTENT, EXTENT * 6] {
        let mut world = World::new(WorldConfig {
            width: extent,
            height: extent,
            seed: SEED,
            faction_count: 2,
            unit_capacity: 64,
        })
        .expect("the extent describes a world");
        world.rebuild_bridge(1).expect("the rebuild must succeed");
        let mut canvas = Canvas::new(512, 512);
        let camera = Camera::at_tile_size(16.0)
            .looking_at(Axial::new(EXTENT as i32 / 2, EXTENT as i32 / 2), &canvas)
            .clamped(&world, &canvas);
        paint::draw(&world, camera, &mut canvas).expect("the world draws");
        assert_eq!(
            canvas.ground_reads(),
            canvas.tiles_painted(),
            "the drawing painted {} tiles of the world of {extent} and asked for {} grounds",
            canvas.tiles_painted(),
            canvas.ground_reads()
        );
        counts.push(canvas.ground_reads());
    }

    assert!(counts[0] > 0, "the window shows no tile");
    assert_eq!(
        counts[0], counts[1],
        "the drawing generated {} grounds in the small world and {} in the world six times \
         as wide",
        counts[0], counts[1]
    );
}
