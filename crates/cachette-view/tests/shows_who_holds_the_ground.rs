//! The viewer shows who holds the ground.
//!
//! The engine says who holds each tile, and the holding changes while the
//! world runs. A watcher who cannot see it cannot check it, and the product
//! record asks for exactly this.[^1]
//!
//! # Which faction column the layer reads
//!
//! A tile carries two values that name a faction. The first is the holder. It
//! says who holds the ground, it changes while the world runs, and it names
//! nobody where nobody holds.[^2] The second is the tile faction column of
//! the stub system. It is written when the world is built, it never changes,
//! and it covers open water as well as open ground.
//!
//! The layer reads the holder. A layer that drew the other column would paint
//! a full, still map of holdings that no rule ever made, and the picture would
//! be plausible and wrong.[^3]
//!
//! Three assertions here catch that defect. The layer must report no held
//! tile in a world that holds no soldier. The count of held tiles it reports
//! must equal the count this test reads back from the holder of the world.
//! Every held tile must take the colour of the faction that the holder names,
//! and open water is never held.
//!
//! # What the fixtures hold
//!
//! The fixture puts two bands of soldiers beside each other and steps the
//! world, so two holdings grow until they meet. It then reads the holders
//! back and refuses a world in which no two holdings meet.[^4] A fixture with
//! one holding supplies no edge, and the edge assertion would then measure
//! the fixture.[^5]
//!
//! The world is this test's own. It is not the world the demonstration binary
//! builds, because that world is chosen to look right rather than to produce
//! an edge value.[^5]
//!
//! The tests see only the public crate API. They open no window.
//!
//! # References
//!
//! [^1]: PRD-0006, a place belongs to somebody. `docs/product/accepted/prd-0006-a-place-belongs-to-somebody.md`
//! [^2]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
//! [^3]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
//! [^4]: Findings register, FND-061. `docs/FINDINGS.md`
//! [^5]: Testing Rules, a fixture supplies the input. `.claude/rules/testing.md`

// An integration test is its own crate, so the allowance at the viewer's
// crate root does not reach it. The reason is the same one: ADR-0067 D3 puts
// the float boundary at the viewer, and a camera is a viewer value.
#![allow(clippy::disallowed_types)]

use cachette_core::hex::NEIGHBOURS;
use cachette_core::terrain::TileKind;
use cachette_core::{Axial, FactionId, Holder, World, WorldConfig};
use cachette_view::paint::{faction_colour, Camera, Canvas, COLOURED_FACTIONS};
use cachette_view::{draw_frame, paint, Metrics};

/// The extent of the fixture world.
///
/// The extent is wider than the coarsest lattice spacing of the terrain
/// generator, so the world holds every kind of ground rather than one of
/// them.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-054. `docs/FINDINGS.md`
const EXTENT: u32 = 64;

/// The seed of the fixture world.
const SEED: u64 = 7;

/// The factions the fixture divides its soldiers between.
const FACTIONS: u16 = 2;

/// The corner of the square the fixture fills with soldiers.
const BAND: (i32, i32) = (20, 40);

/// The column that divides the two bands of soldiers.
const DIVIDE: i32 = 30;

/// The steps the fixture runs before it draws.
///
/// The holding spreads one ring on each step, so the world must run far
/// enough for the two holdings to reach each other.
const TICKS: u32 = 4;

/// The size of the window the tests draw into.
const WINDOW: (usize, usize) = (480, 480);

/// Returns the world settings the fixtures share.
const fn settings() -> WorldConfig {
    WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: SEED,
        faction_count: FACTIONS,
    }
}

/// Builds a world in which two holdings grow until they meet.
///
/// The fixture asserts its own outcome. It reads the holders back after the
/// world has stepped, and it refuses a world in which no two holdings meet.
/// A world with one holding supplies no edge.[^1] [^2]
///
/// # References
///
/// [^1]: Findings register, FND-061. `docs/FINDINGS.md`
/// [^2]: Testing Rules, a fixture supplies the input. `.claude/rules/testing.md`
fn two_holdings_that_meet() -> World {
    let mut world = World::new(settings()).expect("the extent describes a world");
    for row in BAND.0..BAND.1 {
        for column in BAND.0..BAND.1 {
            let at = Axial::new(column, row);
            if !world.admits_a_unit(at) {
                continue;
            }
            let faction = FactionId(u16::from(column >= DIVIDE));
            world
                .spawn_soldier(at, faction)
                .expect("the address and the faction are valid");
        }
    }
    for _ in 0..TICKS {
        world.step(2).expect("the step must run");
    }

    let mut meetings = 0;
    for address in every_address(&world) {
        let Some(holder) = held_by(&world, address) else {
            continue;
        };
        for offset in NEIGHBOURS {
            if held_by(&world, address.add(offset)).is_some_and(|other| other != holder) {
                meetings += 1;
            }
        }
    }
    assert!(
        meetings > 0,
        "no holding in the fixture meets another, so the fixture supplies no edge",
    );
    for faction in 0..FACTIONS {
        assert!(
            world.holding_of(FactionId(faction)) > 0,
            "faction {faction} holds nothing, so the fixture supplies one holding",
        );
    }
    world
}

/// Builds the same world with no soldier in it.
///
/// Nobody holds anything here, because a holding starts from the presence of
/// a unit. The terrain and the tile values are functions of the seed and the
/// tick, so this world paints the same ground as the fixture above.
fn the_same_world_with_nobody_in_it() -> World {
    let mut world = World::new(settings()).expect("the extent describes a world");
    for _ in 0..TICKS {
        world.step(2).expect("the step must run");
    }
    assert_eq!(
        world.holding_of(FactionId(0)) + world.holding_of(FactionId(1)),
        0,
        "a world with no soldier holds ground",
    );
    world
}

/// Returns every address of a world, in index order.
fn every_address(world: &World) -> Vec<Axial> {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .collect()
}

/// Returns the faction that holds a tile, or `None`.
fn held_by(world: &World, address: Axial) -> Option<FactionId> {
    world.tile_holder(address).and_then(Holder::faction)
}

/// Returns a camera that shows the middle of the fixture at a legible size.
///
/// The tile is wide enough that the border of a tile and the middle of it are
/// different pixels. A one pixel tile has no border to look at.
fn close_camera(world: &World, canvas: &Canvas) -> Camera {
    Camera::at_tile_size(16.0)
        .looking_at(Axial::new(DIVIDE, (BAND.0 + BAND.1) / 2), canvas)
        .clamped(world, canvas)
}

/// Returns the pixel at a position, and panics outside the canvas.
fn pixel(canvas: &Canvas, x: i32, y: i32) -> u32 {
    assert!(x >= 0 && y >= 0, "the position {x},{y} is off the canvas");
    let (x, y) = (x as usize, y as usize);
    assert!(
        x < canvas.width() && y < canvas.height(),
        "the position {x},{y} is off the canvas",
    );
    canvas.pixels()[y * canvas.width() + x]
}

/// The distance from the corner of a tile to the pixel a test samples.
///
/// The drawing puts a disc at the middle of a tile for each soldier standing
/// on it, and the fixture stands a soldier on most of the ground it holds. A
/// test that read the middle pixel would read the soldier and not the ground.
/// The sample therefore sits away from the middle, and inside the border.
const INSET: i32 = 2;

/// Returns the corner of a tile and a pixel inside it, or `None` when the
/// tile does not lie wholly inside the canvas.
///
/// The corner carries the border of a holding. The inside pixel carries the
/// fill, and it is far enough from the middle that no soldier disc reaches
/// it.
fn corner_and_inside(
    camera: Camera,
    address: Axial,
    canvas: &Canvas,
) -> Option<((i32, i32), (i32, i32))> {
    let side = (camera.tile_width * 0.92).max(1.0) as i32;
    let (x, y) = camera.centre_of(address);
    let (left, top) = (x as i32 - side / 2, y as i32 - side / 2);
    let inside_the_canvas = left >= 0
        && top >= 0
        && left + side <= canvas.width() as i32
        && top + side <= canvas.height() as i32;
    if !inside_the_canvas || side < 2 * INSET + 3 {
        return None;
    }
    Some(((left, top), (left + INSET, top + INSET)))
}

/// Returns one channel of a colour.
fn channel(colour: u32, offset: u32) -> i32 {
    ((colour >> offset) & 0xff) as i32
}

/// Returns the addresses the window covers, in index order.
fn visible(world: &World, camera: Camera, canvas: &Canvas) -> Vec<Axial> {
    let (first_row, last_row) = camera.visible_rows(world, canvas);
    let mut out = Vec::new();
    for row in first_row..last_row {
        let (first_column, last_column) = camera.visible_columns(row, world, canvas);
        for column in first_column..last_column {
            out.push(Axial::new(column as i32, row as i32));
        }
    }
    out
}

#[test]
fn a_tile_nobody_holds_draws_as_it_did_before() {
    // The layer must leave unheld ground alone. A layer that tinted every
    // tile would satisfy every assertion about a held one.
    let held = two_holdings_that_meet();
    let empty = the_same_world_with_nobody_in_it();
    let mut with_holdings = Canvas::new(WINDOW.0, WINDOW.1);
    let mut without = Canvas::new(WINDOW.0, WINDOW.1);
    let camera = close_camera(&held, &with_holdings);

    paint::draw(&held, camera, &mut with_holdings).expect("the world draws");
    paint::draw(&empty, camera, &mut without).expect("the world draws");

    let mut unheld = 0;
    for address in visible(&held, camera, &with_holdings) {
        if held_by(&held, address).is_some() {
            continue;
        }
        let Some((_, (x, y))) = corner_and_inside(camera, address, &with_holdings) else {
            continue;
        };
        assert_eq!(
            pixel(&with_holdings, x, y),
            pixel(&without, x, y),
            "the tile {address:?} that nobody holds changed colour",
        );
        unheld += 1;
    }
    assert!(
        unheld > 0,
        "every tile in the window is held, so the fixture supplies no unheld ground",
    );
}

#[test]
fn a_held_tile_takes_the_colour_of_its_holder() {
    // The colour must come from the one table the viewer owns. The
    // assertion names no weight, so it pins the table and not the mixing.
    let held = two_holdings_that_meet();
    let empty = the_same_world_with_nobody_in_it();
    let mut with_holdings = Canvas::new(WINDOW.0, WINDOW.1);
    let mut without = Canvas::new(WINDOW.0, WINDOW.1);
    let camera = close_camera(&held, &with_holdings);

    paint::draw(&held, camera, &mut with_holdings).expect("the world draws");
    paint::draw(&empty, camera, &mut without).expect("the world draws");

    let mut seen = std::collections::BTreeSet::new();
    for address in visible(&held, camera, &with_holdings) {
        let Some(faction) = held_by(&held, address) else {
            continue;
        };
        seen.insert(faction.0);
        let Some((_, (x, y))) = corner_and_inside(camera, address, &with_holdings) else {
            continue;
        };
        let drawn = pixel(&with_holdings, x, y);
        let ground = pixel(&without, x, y);
        let wanted = faction_colour(faction);
        assert_ne!(
            drawn, ground,
            "the tile {address:?} is held and draws as unheld ground",
        );
        // Each channel of the drawn colour lies between the ground and the
        // colour of the faction that holds the tile. A layer that read
        // another faction column would fail this on the tiles where the two
        // columns disagree.
        for offset in [0, 8, 16] {
            let (a, b) = (channel(ground, offset), channel(wanted, offset));
            let value = channel(drawn, offset);
            assert!(
                value >= a.min(b) && value <= a.max(b),
                "the tile {address:?} draws {drawn:#08x} over ground {ground:#08x}, \
                 which is not between the ground and the holder colour {wanted:#08x}",
            );
        }
    }
    assert!(
        seen.len() >= 2,
        "the window shows {} holding, so the fixture cannot tell one \
         faction's colour from another's",
        seen.len(),
    );
}

#[test]
fn open_water_never_takes_a_holder_colour() {
    // No faction ever holds open water. A layer that read the tile faction
    // column of the stub system would tint water, because that column covers
    // every tile.
    let held = two_holdings_that_meet();
    let empty = the_same_world_with_nobody_in_it();
    let mut with_holdings = Canvas::new(WINDOW.0, WINDOW.1);
    let mut without = Canvas::new(WINDOW.0, WINDOW.1);
    let camera = close_camera(&held, &with_holdings);

    paint::draw(&held, camera, &mut with_holdings).expect("the world draws");
    paint::draw(&empty, camera, &mut without).expect("the world draws");

    let mut water = 0;
    for address in visible(&held, camera, &with_holdings) {
        if held.tile_kind(address) != Some(TileKind::Water) {
            continue;
        }
        assert!(
            held_by(&held, address).is_none(),
            "a faction holds the open water at {address:?}",
        );
        let Some((_, (x, y))) = corner_and_inside(camera, address, &with_holdings) else {
            continue;
        };
        assert_eq!(
            pixel(&with_holdings, x, y),
            pixel(&without, x, y),
            "the open water at {address:?} took a holder colour",
        );
        water += 1;
    }
    assert!(
        water > 0,
        "the window holds no open water, so this test measures the fixture",
    );
}

#[test]
fn the_edge_of_a_holding_is_drawn() {
    // The edge is what the product record asks a watcher to see. A layer
    // that tinted the tiles and drew no edge would pass every assertion
    // above.
    let world = two_holdings_that_meet();
    let mut canvas = Canvas::new(WINDOW.0, WINDOW.1);
    let camera = close_camera(&world, &canvas);
    paint::draw(&world, camera, &mut canvas).expect("the world draws");

    let side = (camera.tile_width * 0.92).max(1.0) as i32;
    assert!(side >= 3, "a tile {side} pixels wide has no border to read");

    let mut edges = 0;
    let mut insides = 0;
    for address in visible(&world, camera, &canvas) {
        let Some(faction) = held_by(&world, address) else {
            continue;
        };
        let differs = NEIGHBOURS
            .iter()
            .any(|offset| held_by(&world, address.add(*offset)) != Some(faction));
        let Some(((left, top), (x, y))) = corner_and_inside(camera, address, &canvas) else {
            continue;
        };
        let fill = pixel(&canvas, x, y);
        let corner = pixel(&canvas, left, top);
        if differs {
            assert_ne!(
                corner, fill,
                "the tile {address:?} sits on an edge of its holding and has no border",
            );
            edges += 1;
        } else {
            assert_eq!(
                corner, fill,
                "the tile {address:?} sits inside its holding and has a border",
            );
            insides += 1;
        }
    }
    assert!(
        edges > 0 && insides > 0,
        "the window holds {edges} edge tiles and {insides} inside tiles, \
         so the fixture cannot tell a border from no border",
    );
}

#[test]
fn two_holdings_that_meet_are_drawn_in_two_colours() {
    // A boundary a watcher can see is two colours meeting. One colour on
    // both sides is a boundary nobody can read.
    let world = two_holdings_that_meet();
    let mut canvas = Canvas::new(WINDOW.0, WINDOW.1);
    let camera = close_camera(&world, &canvas);
    paint::draw(&world, camera, &mut canvas).expect("the world draws");

    let mut meetings = 0;
    for address in visible(&world, camera, &canvas) {
        let Some(mine) = held_by(&world, address) else {
            continue;
        };
        for offset in NEIGHBOURS {
            let beside = address.add(offset);
            let Some(theirs) = held_by(&world, beside) else {
                continue;
            };
            if theirs == mine {
                continue;
            }
            assert_ne!(
                faction_colour(mine),
                faction_colour(theirs),
                "two factions that meet at {address:?} share one colour",
            );
            meetings += 1;
        }
    }
    assert!(
        meetings > 0,
        "no two holdings meet inside the window, so the fixture supplies no boundary",
    );
}

#[test]
fn the_layer_reads_the_holder_of_the_world() {
    // The count of held tiles the pass reports must equal the count this
    // test reads back through the public holder reader. A layer that read
    // another faction column would report a different count, because that
    // column names a faction for every tile, water included.
    let world = two_holdings_that_meet();
    let mut canvas = Canvas::new(WINDOW.0, WINDOW.1);
    let camera = close_camera(&world, &canvas);
    paint::draw(&world, camera, &mut canvas).expect("the world draws");

    let wanted = visible(&world, camera, &canvas)
        .iter()
        .filter(|address| held_by(&world, **address).is_some())
        .count() as u32;
    assert_eq!(
        canvas.tiles_held(),
        wanted,
        "the pass painted {} held tiles and the world holds {wanted} in the window",
        canvas.tiles_held(),
    );
    assert!(wanted > 0, "the window shows no held tile");
}

#[test]
fn a_world_with_no_soldier_draws_no_holding() {
    // The strongest reading of the wrong column. A still column names a
    // faction for every tile of every world, so a layer that read it would
    // report held tiles in a world that holds none.
    let world = the_same_world_with_nobody_in_it();
    let mut canvas = Canvas::new(WINDOW.0, WINDOW.1);
    let camera = close_camera(&world, &canvas);
    paint::draw(&world, camera, &mut canvas).expect("the world draws");

    assert_eq!(
        canvas.tiles_held(),
        0,
        "the pass painted {} held tiles in a world that holds none",
        canvas.tiles_held(),
    );
    assert!(canvas.tiles_painted() > 0, "the pass painted no tile");
}

#[test]
fn the_layer_reads_one_holder_for_each_painted_tile_and_six_for_each_held_one() {
    // The cost of the layer follows the window. The rule is exact, so a
    // sweep of the world or a short loop over the neighbours breaks it.
    let world = two_holdings_that_meet();
    let mut canvas = Canvas::new(WINDOW.0, WINDOW.1);
    let camera = close_camera(&world, &canvas);
    paint::draw(&world, camera, &mut canvas).expect("the world draws");

    assert_eq!(
        canvas.holder_reads(),
        canvas.tiles_painted() + 6 * canvas.tiles_held(),
        "the pass read {} holders for {} painted tiles and {} held ones",
        canvas.holder_reads(),
        canvas.tiles_painted(),
        canvas.tiles_held(),
    );
    assert!(canvas.tiles_held() > 0, "the window shows no held tile");
}

#[test]
fn the_layer_touches_the_same_tiles_in_a_small_world_and_a_large_one() {
    // The cost must follow the window and not the world. A layer that swept
    // the tiles would paint the same picture and read far more in the
    // larger world.[^1]
    //
    // [^1]: Findings register, FND-071. `docs/FINDINGS.md`
    let mut canvases = Vec::new();
    for extent in [EXTENT, EXTENT * 6] {
        let mut world = World::new(WorldConfig {
            width: extent,
            height: extent,
            seed: SEED,
            faction_count: FACTIONS,
        })
        .expect("the extent describes a world");
        // The same square of tiles is held in both worlds, so the two
        // windows differ in the world around them and in nothing else.
        for row in BAND.0..BAND.1 {
            for column in BAND.0..BAND.1 {
                let at = Axial::new(column, row);
                if world.admits_a_unit(at) {
                    world
                        .spawn_soldier(at, FactionId(u16::from(column >= DIVIDE)))
                        .expect("the address and the faction are valid");
                }
            }
        }
        for _ in 0..TICKS {
            world.step(2).expect("the step must run");
        }
        let mut canvas = Canvas::new(WINDOW.0, WINDOW.1);
        let camera = Camera::at_tile_size(16.0)
            .looking_at(Axial::new(DIVIDE, (BAND.0 + BAND.1) / 2), &canvas)
            .clamped(&world, &canvas);
        paint::draw(&world, camera, &mut canvas).expect("the world draws");
        canvases.push(canvas);
    }

    assert_eq!(
        canvases[0].holder_reads(),
        canvases[1].holder_reads(),
        "the layer read {} holders in the small world and {} in the world \
         six times as wide",
        canvases[0].holder_reads(),
        canvases[1].holder_reads(),
    );
    assert!(
        canvases[0].tiles_held() > 0,
        "neither window shows a held tile",
    );
}

#[test]
fn the_panel_names_every_holder_colour_the_frame_drew() {
    // The product record asks that the window name every colour it draws.
    // The layer draws no new colour, so the legend the panel already states
    // names the holder colours too. This test is what says so.[^1]
    //
    // [^1]: PRD-0005, a watcher can tell what is happening and why. `docs/product/shipped/prd-0005-a-watcher-can-tell-what-is-happening-and-why.md`
    let world = two_holdings_that_meet();
    let mut canvas = Canvas::new(WINDOW.0, WINDOW.1);
    let camera = close_camera(&world, &canvas);
    let readout =
        draw_frame(&world, camera, &Metrics::start(), &[], &mut canvas).expect("the world draws");

    // The legend gives one row for each colour the viewer can tell apart,
    // and it stops at the faction count of the world.
    let named: Vec<u32> = (0..readout.by_faction().len().min(FACTIONS as usize))
        .map(|slot| faction_colour(FactionId(slot as u16)))
        .collect();
    assert_eq!(readout.by_faction().len(), COLOURED_FACTIONS);

    let mut drawn = 0;
    for address in visible(&world, camera, &canvas) {
        let Some(faction) = held_by(&world, address) else {
            continue;
        };
        assert!(
            named.contains(&faction_colour(faction)),
            "the frame drew the ground of faction {} in a colour the panel \
             does not name",
            faction.0,
        );
        drawn += 1;
    }
    assert!(drawn > 0, "the frame drew no held tile");
}
