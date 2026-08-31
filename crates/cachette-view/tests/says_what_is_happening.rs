//! The panel says what the viewer read, and only what it read.
//!
//! These tests go through the public interface of the viewer crate. They do
//! not open a window: a window needs a display, and a test that needs a
//! display does not run in continuous integration.
//!
//! Every test drives the whole frame rather than the panel alone. A panel
//! that renders proves that a panel renders. It does not prove that anything
//! reaches it, and an earlier lesson in this crate is that a mechanism with
//! its own test ships inert.[^1]
//!
//! The camera in these tests points at the middle of the world wherever the
//! test allows it. A camera at the origin cannot see a defect in a range that
//! starts at the world edge, and this crate has already paid for that once.
//!
//! # References
//!
//! [^1]: Testing Rules, drive the real caller. `.claude/rules/testing.md`
//! [^2]: ADR-0070, the head-up display reports what the drawing pass read. `docs/adrs/draft/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
//! [^3]: ADR-0067, the viewer reads the world and never writes to it. `docs/adrs/draft/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`

// An integration test is its own crate, so the allowance at the viewer's
// crate root does not reach it. The reason is the same one: ADR-0067 D3 puts
// the float boundary at the viewer, and a camera is a viewer value.
#![allow(clippy::disallowed_types)]

use cachette_core::{Axial, FactionId, World, WorldConfig};
use cachette_view::paint::COLOURED_FACTIONS;
use cachette_view::{draw_frame, paint, Camera, Canvas, Lap, Metrics};

/// The width of the world these tests scroll around.
const WIDE: u32 = 200;
/// The height of the world these tests scroll around.
const TALL: u32 = 140;
/// The soldiers these tests spread over the world.
const SOLDIERS: u32 = 2200;
/// The factions these tests divide the soldiers between.
const FACTIONS: u16 = 4;

/// The size of the window these tests draw into.
const WINDOW: (usize, usize) = (640, 560);

/// Builds a world far larger than the window, with soldiers spread over it.
fn world() -> World {
    let mut world = World::new(WorldConfig {
        width: WIDE,
        height: TALL,
        seed: 11,
        faction_count: FACTIONS,
    })
    .expect("a large extent describes a world");
    let open = open_tiles(&world);
    for index in 0..SOLDIERS {
        let at = open[(index.wrapping_mul(9973) as usize) % open.len()];
        world
            .spawn_soldier(at, FactionId((index % u32::from(FACTIONS)) as u16))
            .expect("the address and the faction are valid");
    }
    world.rebuild_bridge(1).expect("the rebuild must succeed");
    world
}

/// Builds a canvas of the size these tests use.
fn canvas() -> Canvas {
    Canvas::new(WINDOW.0, WINDOW.1)
}

/// Returns a camera pointed at the middle of the world.
///
/// A camera at the origin sees the corner, and a corner is where a defect in
/// a range hides. Every test that can point inward does.
fn at_the_middle(world: &World, canvas: &Canvas) -> Camera {
    let opening = Camera::opening();
    let across = f32::from(WIDE as u16) / 2.0 * opening.tile_width;
    let down = f32::from(TALL as u16) / 2.0 * opening.tile_height;
    opening.panned(across, down).clamped(world, canvas)
}

/// Runs some steps and measures them, so the cost rows hold real numbers.
fn stepped(world: &mut World, ticks: u32) -> Metrics {
    let mut metrics = Metrics::start();
    for _ in 0..ticks {
        let at = Lap::start();
        world.step(2).expect("the step must run");
        metrics.step(at.elapsed());
    }
    metrics
}

/// Returns every address of a world that admits a unit, in index order.
///
/// The ground refuses a soldier on water, and which tiles hold water is a
/// property of the world seed.[^1] A test that wants soldiers spread over the
/// world therefore takes them from this list rather than naming tiles that a
/// later change to the generator may flood.
///
/// The order is the index order of the grid, which is fixed and does not
/// depend on how a caller visited the world.[^2] The list is built once for a
/// world, because the ground is computed on demand and a repeated sweep of a
/// large world is slow.
///
/// # References
///
/// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D4, a draft record. `docs/adrs/draft/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
/// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
fn open_tiles(world: &World) -> Vec<Axial> {
    let grid = world.grid();
    let open: Vec<Axial> = (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| world.admits_a_unit(*address))
        .collect();
    assert!(
        !open.is_empty(),
        "every tile of the world holds water, so no soldier can stand anywhere"
    );
    open
}

/// Returns the open tile nearest the wanted one, with the lower index winning
/// a tie.
fn nearest_open(open: &[Axial], wanted: Axial) -> Axial {
    *open
        .iter()
        .min_by_key(|address| wanted.distance(**address))
        .expect("the open list holds at least one tile")
}

#[test]
fn the_frame_draws_the_panel_over_the_world() {
    // The panel must reach the canvas through the call the binary makes. A
    // frame that drew only the world would pass every test of the painter.
    let mut world = world();
    let metrics = stepped(&mut world, 3);
    let mut with_panel = canvas();
    let mut bare = canvas();
    let camera = at_the_middle(&world, &with_panel);

    draw_frame(&world, camera, &metrics, &mut with_panel).expect("the world draws");
    paint::draw(&world, camera, &mut bare).expect("the world draws");

    assert_ne!(
        with_panel.pixels(),
        bare.pixels(),
        "the frame drew the world and no panel",
    );

    // The panel sits at the top left. A frame that changed a pixel somewhere
    // else would satisfy the assertion above.
    let corner = 20 * with_panel.width() + 20;
    assert_ne!(
        with_panel.pixels()[corner],
        bare.pixels()[corner],
        "no panel covers the corner the panel is drawn in",
    );
}

#[test]
fn the_panel_lets_the_world_show_through() {
    // PRD-0005: the panel must not hide the world. A panel painted solid
    // would pass every other test here.
    let mut world = world();
    let metrics = stepped(&mut world, 3);
    let mut here = canvas();
    let mut there = canvas();
    let camera = at_the_middle(&world, &here);
    let moved = camera.panned(37.0, 23.0).clamped(&world, &there);

    draw_frame(&world, camera, &metrics, &mut here).expect("the world draws");
    draw_frame(&world, moved, &metrics, &mut there).expect("the world draws");

    // The panel says almost the same thing in both frames, so a difference
    // under it must come from the world beneath it.
    let width = here.width();
    let under: Vec<usize> = (60..200).map(|row| row * width + 40).collect();
    let changed = under
        .iter()
        .filter(|index| here.pixels()[**index] != there.pixels()[**index])
        .count();

    assert!(
        changed > 0,
        "no pixel under the panel changed when the world moved, so the panel is opaque",
    );
}

#[test]
fn the_panel_states_the_tick_the_engine_reached() {
    // The readout must read the engine's counter. A viewer that kept its own
    // frame count would drift the moment a frame was dropped.
    let mut world = world();
    let metrics = stepped(&mut world, 7);
    let mut canvas = canvas();
    let camera = at_the_middle(&world, &canvas);

    let readout = draw_frame(&world, camera, &metrics, &mut canvas).expect("the world draws");

    assert_eq!(readout.tick(), world.tick().0);
    assert_eq!(readout.tick(), 7, "the world did not reach the tick it ran");
}

#[test]
fn a_step_changes_what_the_panel_says() {
    // A panel that stated a constant would pass every assertion about its
    // shape. The numbers must move when the world does.
    let mut world = world();
    let metrics = stepped(&mut world, 1);
    let mut before = canvas();
    let mut after = canvas();
    let camera = at_the_middle(&world, &before);

    let early = draw_frame(&world, camera, &metrics, &mut before).expect("the world draws");

    for _ in 0..5 {
        world.step(2).expect("the step must run");
    }
    let late = draw_frame(&world, camera, &metrics, &mut after).expect("the world draws");

    assert_ne!(early.tick(), late.tick());
    assert_ne!(
        before.pixels(),
        after.pixels(),
        "six steps changed no pixel of the frame",
    );
}

#[test]
fn the_panel_states_where_the_person_is_looking() {
    // PRD-0005: a developer who scrolled must be able to say where they are.
    // A readout that reported a constant centre would leave them lost.
    let mut world = world();
    let metrics = stepped(&mut world, 2);
    let mut canvas = canvas();

    let middle = at_the_middle(&world, &canvas);
    let corner = Camera::opening().clamped(&world, &canvas);

    let looking_in = draw_frame(&world, middle, &metrics, &mut canvas).expect("the world draws");
    let inward = looking_in.centre();
    let extent_in = looking_in.extent_shown();

    let looking_out = draw_frame(&world, corner, &metrics, &mut canvas).expect("the world draws");

    assert_ne!(
        inward,
        looking_out.centre(),
        "the panel named one tile from two very different cameras",
    );
    assert!(
        inward.q > looking_out.centre().q || inward.r > looking_out.centre().r,
        "the camera moved into the world and the stated centre did not follow",
    );

    // The stated extent must be the window's, not the world's.
    assert!(
        extent_in.0 > 0 && extent_in.1 > 0,
        "the panel showed no extent"
    );
    assert!(
        extent_in.0 < WIDE && extent_in.1 < TALL,
        "the panel said the window shows {extent_in:?} of a {WIDE} by {TALL} world",
    );
}

#[test]
fn a_zoom_changes_the_extent_the_panel_states() {
    // The extent must follow the camera. A fixed pair would satisfy the
    // bounds asserted above.
    let mut world = world();
    let metrics = stepped(&mut world, 1);
    let mut canvas = canvas();
    let camera = at_the_middle(&world, &canvas);
    let closer = camera.zoomed(2.0, &canvas).clamped(&world, &canvas);

    let wide = draw_frame(&world, camera, &metrics, &mut canvas).expect("the world draws");
    let wide_extent = wide.extent_shown();
    let near = draw_frame(&world, closer, &metrics, &mut canvas).expect("the world draws");

    assert!(
        near.extent_shown().1 < wide_extent.1,
        "a zoom in showed {} rows against {} rows out",
        near.extent_shown().1,
        wide_extent.1,
    );
}

#[test]
fn the_legend_counts_the_window_and_not_the_world() {
    // ADR-0070 D1 and D2: the census is the drawing pass's own count, and the
    // panel labels it as a count of the window. A viewer that scanned the
    // arena would report the world total here and look more complete.
    let mut world = world();
    let metrics = stepped(&mut world, 2);
    let mut canvas = canvas();
    let camera = at_the_middle(&world, &canvas);

    let readout = draw_frame(&world, camera, &metrics, &mut canvas).expect("the world draws");

    let counted: u32 = readout.by_faction().iter().sum();
    assert_eq!(
        counted,
        readout.soldiers_painted(),
        "the legend and the drawing pass disagree on how many units were painted",
    );
    assert!(counted > 0, "the legend counted no unit at all");
    assert!(
        counted < readout.soldiers_live(),
        "the legend counted {counted} of the {} units the world holds, so it \
         read the world rather than the window",
        readout.soldiers_live(),
    );

    // Every faction in this world must appear, or a reader cannot tell who
    // is who. A legend that counted one colour would still pass the sum.
    let seen = readout
        .by_faction()
        .iter()
        .take(FACTIONS as usize)
        .filter(|count| **count > 0)
        .count();
    assert_eq!(
        seen, FACTIONS as usize,
        "the window over the middle of the world showed {seen} of {FACTIONS} factions",
    );
}

#[test]
fn the_legend_follows_the_window_when_the_person_scrolls() {
    // The sum test above holds for a viewer that counted a fixed number. The
    // counts must change when the window moves to a different part of the
    // world.
    let mut world = world();
    let metrics = stepped(&mut world, 2);
    let mut canvas = canvas();

    let middle = at_the_middle(&world, &canvas);
    let corner = Camera::opening().clamped(&world, &canvas);

    let inward = draw_frame(&world, middle, &metrics, &mut canvas).expect("the world draws");
    let inward_counts = *inward.by_faction();
    let outward = draw_frame(&world, corner, &metrics, &mut canvas).expect("the world draws");

    assert_ne!(
        inward_counts,
        *outward.by_faction(),
        "two very different windows held the same units of each faction",
    );
}

#[test]
fn a_world_of_one_faction_counts_only_that_faction() {
    // A legend that filled every row with the same number would pass the
    // tests above on a world of four equal factions.
    let mut world = World::new(WorldConfig {
        width: 96,
        height: 96,
        seed: 3,
        faction_count: 2,
    })
    .expect("the extent describes a world");
    let open = open_tiles(&world);
    for index in 0..300u32 {
        world
            .spawn_soldier(open[(index as usize * 997) % open.len()], FactionId(1))
            .expect("the address is valid");
    }
    world.rebuild_bridge(1).expect("the rebuild must succeed");
    let metrics = stepped(&mut world, 2);

    let mut canvas = canvas();
    let camera = Camera::fitting(&world, &canvas);
    let readout = draw_frame(&world, camera, &metrics, &mut canvas).expect("the world draws");

    assert_eq!(
        readout.by_faction()[0],
        0,
        "a faction with no unit was counted"
    );
    assert!(
        readout.by_faction()[1] > 0,
        "the only faction in the world was not counted",
    );
}

#[test]
fn the_panel_still_draws_when_the_window_holds_no_unit() {
    // A person who scrolls to an empty part of the world must still be able
    // to read where they are. A panel that divided by the visible count would
    // panic here, and one that skipped its legend would lose its shape.
    let mut world = World::new(WorldConfig {
        width: 96,
        height: 96,
        seed: 3,
        faction_count: 2,
    })
    .expect("the extent describes a world");
    let open = open_tiles(&world);
    for index in 0..40u32 {
        world
            .spawn_soldier(
                nearest_open(&open, Axial::new((index % 5) as i32, (index / 5) as i32)),
                FactionId(0),
            )
            .expect("the address is valid");
    }
    world.rebuild_bridge(1).expect("the rebuild must succeed");
    let metrics = stepped(&mut world, 1);

    let mut canvas = canvas();
    // Look at the far corner, where no unit stands.
    let away = Camera::opening()
        .panned(80.0 * 12.0, 80.0 * 12.0)
        .clamped(&world, &canvas);
    let readout = draw_frame(&world, away, &metrics, &mut canvas).expect("the world draws");

    assert_eq!(readout.soldiers_painted(), 0, "the far corner held a unit");
    assert!(
        readout.soldiers_live() > 0,
        "the world holds units and the panel said it holds none",
    );
    assert_eq!(readout.by_faction(), &[0; COLOURED_FACTIONS]);

    // The panel must still be on the canvas.
    let mut bare = Canvas::new(WINDOW.0, WINDOW.1);
    paint::draw(&world, away, &mut bare).expect("the world draws");
    assert_ne!(
        canvas.pixels(),
        bare.pixels(),
        "an empty window drew no panel",
    );
}

#[test]
fn one_readout_gives_one_picture() {
    // Painting a readout reads the readout and nothing else. A panel that
    // read a clock while it drew would fail here, and the frame would stop
    // being a function of the world.
    let mut world = world();
    let metrics = stepped(&mut world, 4);
    let mut first = canvas();
    let camera = at_the_middle(&world, &first);
    let readout = draw_frame(&world, camera, &metrics, &mut first).expect("the world draws");

    let mut second = canvas();
    paint::draw(&world, camera, &mut second).expect("the world draws");
    cachette_view::hud::draw(&readout, &mut second);
    let mut third = canvas();
    paint::draw(&world, camera, &mut third).expect("the world draws");
    cachette_view::hud::draw(&readout, &mut third);

    assert_eq!(
        second.pixels(),
        third.pixels(),
        "one readout drew two pictures",
    );
}

#[test]
fn the_frame_leaves_the_world_unchanged() {
    // ADR-0067 D1: the viewer reads the world and never writes to it. The
    // shared reference makes a write a compile error, so this test proves the
    // weaker thing a test can prove: nothing observable moved.
    let mut world = world();
    let metrics = stepped(&mut world, 3);

    let hash = world.state_hash();
    let tick = world.tick();
    let live = world.soldiers().len();

    let mut canvas = canvas();
    let camera = at_the_middle(&world, &canvas);
    draw_frame(&world, camera, &metrics, &mut canvas).expect("the world draws");
    draw_frame(&world, camera, &metrics, &mut canvas).expect("the world draws");

    assert_eq!(hash, world.state_hash());
    assert_eq!(tick, world.tick());
    assert_eq!(live, world.soldiers().len());
    assert!(world.check_invariants());
}

#[test]
fn a_stale_world_is_refused_before_the_panel_is_drawn() {
    // The viewer refuses a world whose spatial structure no longer describes
    // it. A frame that drew the panel anyway would show a full set of numbers
    // over a picture with no units, which is the worst outcome of the three.
    let mut world = world();
    let metrics = stepped(&mut world, 1);
    let mut canvas = canvas();
    let camera = at_the_middle(&world, &canvas);
    draw_frame(&world, camera, &metrics, &mut canvas).expect("a fresh world draws");

    let open = nearest_open(&open_tiles(&world), Axial::new(3, 3));
    world
        .spawn_soldier(open, FactionId(0))
        .expect("the address is valid");

    assert!(
        draw_frame(&world, camera, &metrics, &mut canvas).is_err(),
        "a stale world drew a frame instead of refusing",
    );
}

#[test]
fn the_panel_stays_inside_its_own_edge() {
    // The panel is an instrument over a picture. Text that escaped it would
    // sit on the world and be unreadable against whatever is under it.
    let mut world = world();
    let metrics = stepped(&mut world, 40);
    let mut with_panel = canvas();
    let mut bare = canvas();
    let camera = at_the_middle(&world, &with_panel);

    let readout = draw_frame(&world, camera, &metrics, &mut with_panel).expect("the world draws");
    paint::draw(&world, camera, &mut bare).expect("the world draws");

    // The panel states its own rectangle. Anything it changed must sit inside
    // that rectangle, whatever the numbers are and however long they run.
    let (left, top, panel_width, panel_height) = cachette_view::hud::bounds(&readout);
    let width = with_panel.width();
    for (index, (panelled, plain)) in with_panel
        .pixels()
        .iter()
        .zip(bare.pixels().iter())
        .enumerate()
    {
        if panelled == plain {
            continue;
        }
        let (x, y) = ((index % width) as i32, (index / width) as i32);
        assert!(
            x >= left && x < left + panel_width && y >= top && y < top + panel_height,
            "the panel changed a pixel at {x} by {y}, outside the rectangle it \
             states: {left}, {top}, {panel_width} by {panel_height}",
        );
    }

    // The rectangle must be a real bound, not one that covers the window.
    let window_area = (width * with_panel.height()) as i64;
    assert!(
        i64::from(panel_width) * i64::from(panel_height) * 2 < window_area,
        "the panel claims {panel_width} by {panel_height} of a {width} by {} window, \
         so it covers more than half of what the person came to watch",
        with_panel.height(),
    );
}

#[test]
fn the_cost_rows_state_what_the_report_states() {
    // The panel and the closing report must not derive one figure two ways.
    // A second derivation would be one fact in two places, and nothing would
    // fail when the copies disagreed.
    let mut world = world();
    let metrics = stepped(&mut world, 5);
    let mut canvas = canvas();
    let camera = at_the_middle(&world, &canvas);

    let at = Lap::start();
    draw_frame(&world, camera, &metrics, &mut canvas).expect("the world draws");
    let mut metrics = metrics;
    metrics.draw(at.elapsed());

    assert_eq!(metrics.ticks(), 5);
    assert_eq!(metrics.frames(), 1);
    // A mean over no frame must be zero rather than a division by zero.
    assert!(metrics.draw_mean_micros() >= 0.0);
    assert!(metrics.step_worst_micros() >= metrics.step_mean_micros());
    // The report reads the same accessors. Calling it proves it does not
    // panic on a run whose draw count is below its tick count.
    metrics.report(
        world.grid().tile_count(),
        world.soldiers().len() as usize,
        2,
    );
}

/// Builds a world the size the demonstration binary runs, sparsely filled.
///
/// Two properties matter, and both are needed to reach the widest values the
/// panel must hold.
///
/// The world is large, so a wide view covers many blocks. The stride is
/// small, so the soldiers cluster into the first rows instead of spreading
/// over every tile. A clustered world leaves most blocks empty, so the count
/// of skipped blocks grows alongside the count of read blocks and both reach
/// two digits.
///
/// A world spread by a large stride puts a soldier in nearly every block, and
/// then nothing is ever skipped. Such a world hides the defect this file
/// tests for.
fn sparse_demonstration_world() -> World {
    let mut world = World::new(WorldConfig {
        width: 640,
        height: 440,
        seed: 0x0cac_4e77_e5ee_d001,
        faction_count: FACTIONS,
    })
    .expect("the extent describes a world");
    let open = open_tiles(&world);
    for index in 0..600u32 {
        world
            .spawn_soldier(
                open[(index.wrapping_mul(37) as usize) % open.len()],
                FactionId((index % u32::from(FACTIONS)) as u16),
            )
            .expect("the address and the faction are valid");
    }
    world.rebuild_bridge(1).expect("the rebuild must succeed");
    world
}

#[test]
fn no_value_is_cut_to_fit_its_column() {
    // The panel cuts a value that will not fit, so that text never reaches
    // the panel edge. A cut value states something other than the number it
    // was given, and it does so in silence.
    //
    // This test exists because the edge test cannot see it. A row that named
    // two block counts in one value did not fit at the widest zoom, and the
    // cut is exactly what kept it inside the rectangle. The edge test passed
    // because of the defect, not in spite of it.
    //
    // The value the old row produced on this world at the widest zoom was
    // "12 read, 14 skipped", which is nineteen characters in an eighteen
    // character column. It reached the panel as "12 read, 14 skippe".
    let mut world = sparse_demonstration_world();
    let metrics = stepped(&mut world, 3);
    let mut canvas = Canvas::new(960, 720);

    // Walk from the closest zoom to the widest. Every step must fit, and the
    // widest is where the counts are largest.
    let mut camera = Camera::opening().clamped(&world, &canvas);
    let opening = draw_frame(&world, camera, &metrics, &mut canvas)
        .expect("the world draws")
        .extent_shown();
    let mut widest = opening;

    // Twenty-four steps take the tile size from the opening twelve pixels
    // to the smallest the viewer allows.
    for _ in 0..24 {
        camera = camera.zoomed_out(&canvas).clamped(&world, &canvas);
        let readout = draw_frame(&world, camera, &metrics, &mut canvas).expect("the world draws");
        let cut = cachette_view::hud::values_that_do_not_fit(&readout);
        assert!(
            cut.is_empty(),
            "the panel cut {cut:?} while showing {:?} tiles",
            readout.extent_shown(),
        );
        widest = readout.extent_shown();
    }

    // The walk must have reached a wide view, or it proved nothing about a
    // window full of blocks and six-figure counts.
    assert!(
        widest.0 > opening.0 * 3 && widest.1 > opening.1 * 3,
        "the walk went from {opening:?} tiles to {widest:?}, which is not a wide view",
    );

    // The check must be able to answer no. A check with no proven failure
    // mode is decoration. This is the exact value the old row produced on
    // this world at the widest zoom.
    assert!(
        !cachette_view::hud::value_fits("12 read, 14 skipped"),
        "the fit check accepts the value that overran the panel, so it cannot fail",
    );
    assert!(
        cachette_view::hud::value_fits("14"),
        "the fit check rejects a value that plainly fits",
    );
}

#[test]
fn the_panel_fills_the_rectangle_it_states() {
    // The height is summed from the same list of lines that the painting
    // walks. A height that ran ahead of the content would leave a band of
    // empty panel, and a height that fell behind would let the last lines
    // paint past the edge.
    //
    // The edge test catches the second case. This one catches the first, so
    // the two together hold the height to the content from both sides.
    let mut world = world();
    let metrics = stepped(&mut world, 3);
    let mut with_panel = canvas();
    let mut bare = canvas();
    let camera = at_the_middle(&world, &with_panel);

    let readout = draw_frame(&world, camera, &metrics, &mut with_panel).expect("the world draws");
    paint::draw(&world, camera, &mut bare).expect("the world draws");

    let (_, top, _, height) = cachette_view::hud::bounds(&readout);
    let width = with_panel.width();
    let lowest = with_panel
        .pixels()
        .iter()
        .zip(bare.pixels().iter())
        .enumerate()
        .filter(|(_, (panelled, plain))| panelled != plain)
        .map(|(index, _)| (index / width) as i32)
        .max()
        .expect("the panel painted nothing");

    // The panel's own edge is the last row it paints, so the lowest changed
    // row is the bottom of the rectangle.
    assert_eq!(
        lowest,
        top + height - 1,
        "the panel states a rectangle {height} tall and paints down to {lowest}",
    );
}
