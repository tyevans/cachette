//! A watcher can find a unit, count a crowd, and see where a unit came from.
//!
//! The product record asks that a watcher tell one faction from another and
//! read what is happening.[^1] A review of the pictures found that a fed unit
//! disappeared below about sixteen pixels a tile, that eight units on one
//! tile drew as one disc, that a tile two factions stood on showed one of
//! them, and that nothing said where a unit came from.[^2] [^3] These tests
//! cover the four.
//!
//! # The fixture
//!
//! The fixture is built for these cases and never copied from the
//! demonstration world. A world chosen to look right supplies no crowd and
//! no shared tile, so an assertion over it never receives the input that
//! would fail it.[^4]
//!
//! Each test names the tile size it draws at, because every one of these
//! marks changes with the zoom. A test at one zoom proves nothing about the
//! zoom the defect was found at.
//!
//! # References
//!
//! [^1]: PRD-0005, a watcher can tell what is happening and why. `docs/product/shipped/prd-0005-a-watcher-can-tell-what-is-happening-and-why.md`
//! [^2]: Research report 23, defects 1 and 3. `docs/research/reports/23-demonstration-readability-review-1.md`
//! [^3]: Research report 25, defects 4 and 5. `docs/research/reports/25-demonstration-readability-upgrades-and-units.md`
//! [^4]: Findings register, FND-051. `docs/FINDINGS.md`

use cachette_core::{Axial, FactionId, World, WorldConfig};
use cachette_view::paint::{faction_colour, unit_halo_colour, unit_radius, unit_rim_colour};
use cachette_view::{paint, Camera, Canvas, FrameSize, Motion, Pace};

/// The size of every canvas these tests draw into.
const CANVAS: (usize, usize) = (400, 400);

/// Builds a world with open ground and no unit.
fn world() -> World {
    World::new(WorldConfig {
        width: 24,
        height: 24,
        seed: 11,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent describes a world")
}

/// Returns a tile that admits a unit and admits a crowd.
fn open_tile(world: &World) -> Axial {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .find(|address| {
            world.admits_a_unit(*address)
                && world.tile_capacity(*address).is_some_and(|room| room >= 8)
        })
        .expect("the world holds a tile that admits a crowd")
}

/// Returns the colour the canvas holds at one pixel.
fn pixel(canvas: &Canvas, x: i32, y: i32) -> u32 {
    assert!(x >= 0 && y >= 0, "the pixel {x}, {y} is off the canvas");
    let (x, y) = (x as usize, y as usize);
    assert!(
        x < canvas.width() && y < canvas.height(),
        "the pixel {x}, {y} is off the canvas"
    );
    canvas.pixels()[y * canvas.width() + x]
}

/// Draws one still frame of a world.
fn shot(world: &World, camera: Camera) -> Canvas<'static> {
    let mut canvas = Canvas::new(CANVAS.0, CANVAS.1);
    paint::draw(world, camera, &mut canvas).expect("the world draws");
    canvas
}

/// Returns the pixel centre of a tile, rounded as the drawing rounds it.
fn centre(camera: Camera, address: Axial) -> (i32, i32) {
    let (x, y) = camera.centre_of(address);
    (x as i32, y as i32)
}

#[test]
fn a_unit_bead_carries_a_halo_outside_a_dark_rim() {
    // The outline is what separates a bead from the tint of its own
    // faction's ground, and from any other background. It takes the outer two
    // pixels of the bead. The halo sits outside, the rim sits inside it, and
    // the faction colour holds the rest.
    let mut world = world();
    let place = open_tile(&world);
    world
        .spawn_soldier(place, FactionId(0))
        .expect("the tile admits a unit");
    world.rebuild_bridge(1).expect("the bridge rebuilds");

    let mut canvas = Canvas::new(CANVAS.0, CANVAS.1);
    let tile = 32.0;
    let camera = Camera::at_tile_size(tile).looking_at(place, &canvas);
    paint::draw(&world, camera, &mut canvas).expect("the world draws");

    // The radius comes from the reader the pass itself calls. A test that
    // repeated the arithmetic would be a second declaration site.[^1]
    //
    // [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
    let (x, y) = centre(camera, place);
    let radius = unit_radius(tile);
    assert_eq!(
        pixel(&canvas, x + radius, y),
        unit_halo_colour(),
        "the edge of a bead must carry the halo"
    );
    assert_eq!(
        pixel(&canvas, x + radius - 1, y),
        unit_rim_colour(),
        "the pixel inside the halo must carry the rim"
    );
    assert_eq!(
        pixel(&canvas, x, y),
        faction_colour(FactionId(0)),
        "the middle of a bead must carry the faction colour"
    );
}

#[test]
fn a_unit_holds_a_radius_floor_below_sixteen_pixels_a_tile() {
    // At four pixels a tile the radius from the tile width alone is one
    // pixel, and a unit is then one pixel of its faction colour over ground
    // its own faction tints. The floor holds it wide enough to carry the two
    // outline bands and a core of its faction colour.
    let mut world = world();
    let place = open_tile(&world);
    world
        .spawn_soldier(place, FactionId(0))
        .expect("the tile admits a unit");
    world.rebuild_bridge(1).expect("the bridge rebuilds");

    let mut canvas = Canvas::new(CANVAS.0, CANVAS.1);
    let camera = Camera::at_tile_size(4.0).looking_at(place, &canvas);
    paint::draw(&world, camera, &mut canvas).expect("the world draws");

    let (x, y) = centre(camera, place);
    assert_eq!(
        pixel(&canvas, x + unit_radius(4.0), y),
        unit_halo_colour(),
        "a unit must reach its floor radius from its centre at every zoom"
    );
}

#[test]
fn a_crowd_states_its_count_above_the_badge_width() {
    // Two pictures of one tile, one unit against five. The disc is the same
    // size at this zoom, so every pixel that differs above the disc is the
    // badge.
    let place = {
        let world = world();
        open_tile(&world)
    };

    let mut lone = world();
    lone.spawn_soldier(place, FactionId(0))
        .expect("the tile admits a unit");
    lone.rebuild_bridge(1).expect("the bridge rebuilds");

    let mut crowd = world();
    for _ in 0..5 {
        crowd
            .spawn_soldier(place, FactionId(0))
            .expect("the tile admits a unit");
    }
    crowd.rebuild_bridge(1).expect("the bridge rebuilds");

    let frame = FrameSize::new(CANVAS.0, CANVAS.1);
    let camera = Camera::at_tile_size(32.0).looking_at(place, &frame);
    let one = shot(&lone, camera);
    let many = shot(&crowd, camera);

    let (x, y) = centre(camera, place);
    let differs = (y - 24..y - 10).any(|row| {
        (x - 24..x + 24).any(|column| pixel(&one, column, row) != pixel(&many, column, row))
    });
    assert!(
        differs,
        "a crowd must state its count above the disc at 32 pixels a tile"
    );
}

#[test]
fn a_crowd_grows_the_disc_below_the_badge_width() {
    // At twelve pixels a tile there is no room for a glyph, so the count is
    // in the size of the disc. The count of pixels the rim covers is the
    // measure, because the rim is drawn at the edge of the disc alone.
    let place = {
        let world = world();
        open_tile(&world)
    };

    let mut lone = world();
    lone.spawn_soldier(place, FactionId(0))
        .expect("the tile admits a unit");
    lone.rebuild_bridge(1).expect("the bridge rebuilds");

    let mut crowd = world();
    for _ in 0..8 {
        crowd
            .spawn_soldier(place, FactionId(0))
            .expect("the tile admits a unit");
    }
    crowd.rebuild_bridge(1).expect("the bridge rebuilds");

    let frame = FrameSize::new(CANVAS.0, CANVAS.1);
    let camera = Camera::at_tile_size(12.0).looking_at(place, &frame);
    let one = shot(&lone, camera);
    let many = shot(&crowd, camera);

    let rim = |canvas: &Canvas| {
        canvas
            .pixels()
            .iter()
            .filter(|&&colour| colour == unit_rim_colour())
            .count()
    };
    assert!(
        rim(&many) > rim(&one),
        "eight units on one tile must draw a larger disc than one: {} against {}",
        rim(&many),
        rim(&one)
    );
}

#[test]
fn a_tile_two_factions_stand_on_shows_both() {
    // The units arrive in the order the spatial structure holds, and the
    // pass used to let the last one cover every earlier one. The wedges give
    // each faction present a share of the disc.
    let mut world = world();
    let place = open_tile(&world);
    for _ in 0..4 {
        world
            .spawn_soldier(place, FactionId(0))
            .expect("the tile admits a unit");
    }
    for _ in 0..4 {
        world
            .spawn_soldier(place, FactionId(1))
            .expect("the tile admits a unit");
    }
    world.rebuild_bridge(1).expect("the bridge rebuilds");

    let mut canvas = Canvas::new(CANVAS.0, CANVAS.1);
    let camera = Camera::at_tile_size(32.0).looking_at(place, &canvas);
    paint::draw(&world, camera, &mut canvas).expect("the world draws");

    let holds = |colour: u32| canvas.pixels().contains(&colour);
    assert!(
        holds(faction_colour(FactionId(0))),
        "the tile must show the first faction"
    );
    assert!(
        holds(faction_colour(FactionId(1))),
        "the tile must show the second faction"
    );
}

#[test]
fn a_unit_shows_the_tile_it_came_from() {
    // Two frames, with the table the caller keeps between them. The unit
    // steps to a neighbour, and the line runs from the tile it left.
    let mut world = world();
    let from = open_tile(&world);
    let to = Axial::new(from.q + 1, from.r);
    assert!(world.admits_a_unit(to), "the neighbour must admit a unit");
    let unit = world
        .spawn_soldier(from, FactionId(0))
        .expect("the tile admits a unit");
    world.rebuild_bridge(1).expect("the bridge rebuilds");

    let mut canvas = Canvas::new(CANVAS.0, CANVAS.1);
    let camera = Camera::at_tile_size(32.0).looking_at(from, &canvas);
    let mut motion = Motion::for_frame(CANVAS.0, CANVAS.1);
    paint::draw_paced(&world, camera, &mut canvas, Pace::STILL, &mut motion, None)
        .expect("the first frame draws");

    world
        .place_soldier(unit, to)
        .expect("the neighbour admits the unit");
    world.rebuild_bridge(2).expect("the bridge rebuilds");
    paint::draw_paced(&world, camera, &mut canvas, Pace::STILL, &mut motion, None)
        .expect("the second frame draws");

    // The middle of the two tile centres lies on the line and outside both
    // discs, because a tile of 32 pixels gives a radius of nine.
    let (ax, ay) = centre(camera, from);
    let (bx, by) = centre(camera, to);
    let (mx, my) = ((ax + bx) / 2, (ay + by) / 2);
    assert_eq!(
        pixel(&canvas, mx, my),
        faction_colour(FactionId(0)),
        "the line must run from the tile the unit left"
    );
}
