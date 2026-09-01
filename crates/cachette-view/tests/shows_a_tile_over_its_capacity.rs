//! A watcher can see that no tile holds more units than its ground admits.
//!
//! The product record asks for two things about the capacity of a tile. No
//! tile holds more units than its capacity allows, and a watcher can see that
//! this holds.[^1] These tests cover the second thing only. They state no
//! position on whether a spawn may over-fill a tile, which is an open
//! choice.[^2]
//!
//! Every test drives the public interface of the viewer crate. The viewer
//! reads the world and writes nothing to it.[^3]
//!
//! # The fixture
//!
//! The fixture is built for this case and never copied from the
//! demonstration world. A world chosen to look right supplies no extreme, so
//! an assertion over it never receives the input that would fail it.[^4]
//!
//! The fixture holds four tiles that matter: an empty tile, a tile under its
//! capacity, a tile at exactly its capacity, and a tile over its capacity.
//! The capacity comes from the terrain, so the fixture asks the ground how
//! many units a tile admits and never writes a number of its own.[^5]
//!
//! Each test asserts what the frame reported. None asserts what the spawn
//! asked for.[^6]
//!
//! One test reads the count back for a named tile. A test that compared a
//! crowded world against an empty one could not see a defect that moved both
//! worlds alike.[^7]
//!
//! # References
//!
//! [^1]: PRD-0002, a developer watches the world run. `docs/product/shipped/prd-0002-a-developer-watches-the-world-run.md`
//! [^2]: Decisions register, DEC-020. `docs/DECISIONS.md`
//! [^3]: ADR-0067, the viewer reads the world and never writes to it, decision D1. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
//! [^4]: Findings register, FND-051. `docs/FINDINGS.md`
//! [^5]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
//! [^6]: Findings register, FND-061. `docs/FINDINGS.md`
//! [^7]: Findings register, FND-078. `docs/FINDINGS.md`

use std::time::Duration;

use cachette_core::{Axial, FactionId, World, WorldConfig};
use cachette_view::{draw_frame, paint, Camera, Canvas, Metrics};

/// The tiles the fixture uses, and how full each one is.
struct Crowd {
    /// The tile that holds no unit.
    empty: Axial,
    /// The tile that holds fewer units than its ground admits.
    under: Axial,
    /// The tile that holds exactly as many units as its ground admits.
    exact: Axial,
    /// The tile that holds more units than its ground admits.
    over: Axial,
    /// The number of units on the tile that is over its capacity.
    over_units: u32,
    /// The number of units the ground of these tiles admits.
    capacity: u32,
}

/// Builds a world with one tile over its capacity.
///
/// The four tiles sit far apart, so the mark on one cannot reach the square
/// of another.
fn crowded_world() -> (World, Crowd) {
    let mut world = World::new(WorldConfig {
        width: 24,
        height: 24,
        seed: 11,
        faction_count: 1,
    })
    .expect("the extent describes a world");

    let open = open_tiles(&world);
    let empty = pick(&open, Axial::new(3, 3), &[]);
    let under = pick(&open, Axial::new(20, 3), &[empty]);
    let exact = pick(&open, Axial::new(3, 20), &[empty, under]);
    let over = pick(&open, Axial::new(20, 20), &[empty, under, exact]);

    // The ground says how many units a tile admits. A number here would be a
    // second declaration site for a value the terrain already holds.
    let capacity = capacity_of(&world, over);
    assert!(
        capacity >= 2,
        "the fixture needs room for an under-full tile"
    );
    assert_eq!(
        capacity_of(&world, exact),
        capacity,
        "the three tiles must share one capacity, or the counts mean two things",
    );
    assert_eq!(capacity_of(&world, under), capacity);
    let over_units = capacity + 4;

    for (address, count) in [(under, capacity - 1), (exact, capacity), (over, over_units)] {
        for _ in 0..count {
            world
                .spawn_soldier(address, FactionId(0))
                .expect("the ground admits a unit");
        }
    }
    // A spawn makes the derived structure stale. A caller that only spawns
    // rebuilds it, or the viewer refuses to read.
    world.rebuild_bridge(1).expect("the rebuild must succeed");

    (
        world,
        Crowd {
            empty,
            under,
            exact,
            over,
            over_units,
            capacity,
        },
    )
}

/// Returns the number of units the ground of one tile admits.
fn capacity_of(world: &World, address: Axial) -> u32 {
    world
        .terrain()
        .tile(address)
        .expect("the tile lies inside the world")
        .kind
        .capacity()
}

/// Returns every address of a world that admits a unit, in index order.
fn open_tiles(world: &World) -> Vec<Axial> {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| world.admits_a_unit(*address))
        .collect()
}

/// Returns the open tile nearest the wanted one, skipping the taken ones.
fn pick(open: &[Axial], wanted: Axial, taken: &[Axial]) -> Axial {
    *open
        .iter()
        .filter(|address| !taken.contains(address))
        .min_by_key(|address| wanted.distance(**address))
        .expect("the world holds enough open ground for the fixture")
}

/// Draws the whole world into a canvas that shows all of it.
fn draw_all(world: &World) -> (Canvas, Camera) {
    let mut canvas = Canvas::new(720, 720);
    let camera = Camera::fitting(world, &canvas);
    paint::draw(world, camera, &mut canvas).expect("the bridge must describe the arena");
    (canvas, camera)
}

#[test]
fn the_frame_counts_the_units_it_painted_on_the_fullest_tile() {
    // The number the frame reports is the number the frame painted. The test
    // reads it back for a named tile, and never compares two pictures.
    let (world, crowd) = crowded_world();
    let (canvas, _) = draw_all(&world);

    assert_eq!(
        canvas.crowd_worst(),
        crowd.over_units,
        "the frame reported {} units on the fullest tile, and it painted {}",
        canvas.crowd_worst(),
        crowd.over_units,
    );
}

#[test]
fn the_frame_agrees_with_the_engine_about_one_named_tile() {
    // The count of the drawing pass and the count the engine gives for the
    // same tile must agree. The engine answer comes through the public
    // interface, so this reads the world the way any caller would.
    let (world, crowd) = crowded_world();
    let (canvas, _) = draw_all(&world);

    let engine = world
        .bridge()
        .count_on_tile(world.soldiers(), crowd.over)
        .expect("the bridge describes the arena");
    assert_eq!(
        u32::try_from(engine).expect("a tile holds fewer units than a u32 counts"),
        canvas.crowd_worst(),
        "the engine says {engine} units stand on the fullest tile",
    );
}

#[test]
fn the_frame_counts_the_tile_at_its_capacity_and_the_one_over_it() {
    // Two tiles reach their capacity: the one at it, and the one over it.
    // The tile under its capacity and the empty tile do not.
    let (world, _) = crowded_world();
    let (canvas, _) = draw_all(&world);

    assert_eq!(
        canvas.tiles_at_capacity(),
        2,
        "the frame counted {} tiles at their capacity, and the fixture holds two",
        canvas.tiles_at_capacity(),
    );
}

#[test]
fn the_picture_marks_the_over_filled_tile_and_leaves_the_full_one_alone() {
    // A watcher must see the tile itself, not only a number. The tile at
    // exactly its capacity is not a breach, so it carries no mark.
    let (world, crowd) = crowded_world();
    let (canvas, camera) = draw_all(&world);
    let mark = paint::over_capacity_colour();

    assert!(
        marked(&canvas, camera, crowd.over, mark),
        "the tile over its capacity carries no mark",
    );
    assert!(
        !marked(&canvas, camera, crowd.exact, mark),
        "the tile at exactly its capacity carries the mark of a breach",
    );
    assert!(
        !marked(&canvas, camera, crowd.under, mark),
        "the tile under its capacity carries the mark of a breach",
    );
    assert!(
        !marked(&canvas, camera, crowd.empty, mark),
        "the empty tile carries the mark of a breach",
    );
}

/// Reports whether the square of one tile holds the mark colour.
fn marked(canvas: &Canvas, camera: Camera, address: Axial, mark: u32) -> bool {
    let (x, y) = camera.centre_of(address);
    let reach = (camera.tile_width * 0.5) as i32;
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
            if canvas.pixels()[row * canvas.width() + column] == mark {
                return true;
            }
        }
    }
    false
}

#[test]
fn both_numbers_fall_when_the_camera_leaves_the_crowded_tile() {
    // The label says the numbers count what the frame painted. This is the
    // test of that label: move the window off the crowded tiles, and both
    // numbers must fall.
    let (world, crowd) = crowded_world();
    let (whole, _) = draw_all(&world);
    assert!(whole.crowd_worst() > 0, "the whole world showed no unit");

    let mut canvas = Canvas::new(200, 200);
    let camera = Camera::at_tile_size(20.0)
        .looking_at(crowd.empty, &canvas)
        .clamped(&world, &canvas);
    paint::draw(&world, camera, &mut canvas).expect("the bridge must describe the arena");

    assert!(
        canvas.crowd_worst() < whole.crowd_worst(),
        "the fullest count stayed at {} with the window off the crowded tile",
        canvas.crowd_worst(),
    );
    assert!(
        canvas.tiles_at_capacity() < whole.tiles_at_capacity(),
        "the count of full tiles stayed at {} with the window off them",
        canvas.tiles_at_capacity(),
    );
}

#[test]
fn the_panel_states_both_numbers() {
    // The panel is the whole frame, so the test draws the whole frame. A
    // readout that renders proves nothing until something reaches it.
    let (world, crowd) = crowded_world();
    let mut canvas = Canvas::new(720, 720);
    let camera = Camera::fitting(&world, &canvas);
    let metrics = Metrics::fixed(
        1,
        1,
        Duration::from_micros(10),
        Duration::from_micros(10),
        Duration::from_millis(1),
    );
    let readout = draw_frame(&world, camera, &metrics, &[], &mut canvas).expect("the frame draws");

    assert_eq!(readout.crowd_worst(), crowd.over_units);
    assert_eq!(readout.tiles_at_capacity(), 2);
    assert!(
        crowd.capacity > 0,
        "the ground of the fixture admits no unit"
    );
}

#[test]
fn a_drawn_frame_leaves_the_crowded_world_where_it_found_it() {
    // The viewer reads the world and writes nothing to it. The state hash is
    // the check a reviewer can run, and it must not move over a draw.
    let (world, _) = crowded_world();
    let hash = world.state_hash();

    let (canvas, _) = draw_all(&world);
    assert!(
        canvas.crowd_worst() > 0,
        "the draw painted no unit, so the comparison proves nothing",
    );

    assert_eq!(hash, world.state_hash(), "the drawing moved the world");
}
