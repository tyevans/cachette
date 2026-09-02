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
//! [^2]: ADR-0070, the head-up display reports what the drawing pass read. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
//! [^3]: ADR-0067, the viewer reads the world and never writes to it. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`

// An integration test is its own crate, so the allowance at the viewer's
// crate root does not reach it. The reason is the same one: ADR-0067 D3 puts
// the float boundary at the viewer, and a camera is a viewer value.
#![allow(clippy::disallowed_types)]

use cachette_core::founding::FoundingOutcome;
use cachette_core::terrain::KIND_COUNT;
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
///
/// The height is shorter than the panel, so these tests measure a panel that
/// the window cuts, and two tests below read that case on purpose.[^1] It is
/// not the height of the demonstration window: a second copy of that number
/// here would say something false the next time the binary changes.[^2]
///
/// [^1]: Backlog item 0045. `docs/backlog/complete/0045-the-panel-has-no-answer-for-a-short-window.md`
/// [^2]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
const WINDOW: (usize, usize) = (640, 720);

/// A window tall enough to hold the whole panel, whatever it holds.
///
/// The demonstration window is shorter than the panel and cuts it, which the
/// panel says on its last line.[^1] A test that asks whether a section
/// reaches a line at all must not measure the cut instead, so it draws into
/// this height.
///
/// [^1]: Backlog item 0133. `docs/backlog/proposed/0133-let-a-watcher-reach-a-panel-longer-than-the-window.md`
const WHOLE_PANEL: usize = 1600;

/// Builds a world far larger than the window, with soldiers spread over it.
fn world() -> World {
    let mut world = World::new(WorldConfig {
        width: WIDE,
        height: TALL,
        seed: 11,
        faction_count: FACTIONS,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
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
/// [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D4. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
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

    draw_frame(&world, camera, &metrics, &[], &mut with_panel).expect("the world draws");
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

    draw_frame(&world, camera, &metrics, &[], &mut here).expect("the world draws");
    draw_frame(&world, moved, &metrics, &[], &mut there).expect("the world draws");

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

    let readout = draw_frame(&world, camera, &metrics, &[], &mut canvas).expect("the world draws");

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

    let early = draw_frame(&world, camera, &metrics, &[], &mut before).expect("the world draws");

    for _ in 0..5 {
        world.step(2).expect("the step must run");
    }
    let late = draw_frame(&world, camera, &metrics, &[], &mut after).expect("the world draws");

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

    let looking_in =
        draw_frame(&world, middle, &metrics, &[], &mut canvas).expect("the world draws");
    let inward = looking_in.centre();
    let extent_in = looking_in.extent_shown();

    let looking_out =
        draw_frame(&world, corner, &metrics, &[], &mut canvas).expect("the world draws");

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

    let wide = draw_frame(&world, camera, &metrics, &[], &mut canvas).expect("the world draws");
    let wide_extent = wide.extent_shown();
    let near = draw_frame(&world, closer, &metrics, &[], &mut canvas).expect("the world draws");

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

    let readout = draw_frame(&world, camera, &metrics, &[], &mut canvas).expect("the world draws");

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

    let inward = draw_frame(&world, middle, &metrics, &[], &mut canvas).expect("the world draws");
    let inward_counts = *inward.by_faction();
    let outward = draw_frame(&world, corner, &metrics, &[], &mut canvas).expect("the world draws");

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
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
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
    let readout = draw_frame(&world, camera, &metrics, &[], &mut canvas).expect("the world draws");

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
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
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
    let readout = draw_frame(&world, away, &metrics, &[], &mut canvas).expect("the world draws");

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
    let readout = draw_frame(&world, camera, &metrics, &[], &mut first).expect("the world draws");

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
    draw_frame(&world, camera, &metrics, &[], &mut canvas).expect("the world draws");
    draw_frame(&world, camera, &metrics, &[], &mut canvas).expect("the world draws");

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
    draw_frame(&world, camera, &metrics, &[], &mut canvas).expect("a fresh world draws");

    let open = nearest_open(&open_tiles(&world), Axial::new(3, 3));
    world
        .spawn_soldier(open, FactionId(0))
        .expect("the address is valid");

    assert!(
        draw_frame(&world, camera, &metrics, &[], &mut canvas).is_err(),
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

    let readout =
        draw_frame(&world, camera, &metrics, &[], &mut with_panel).expect("the world draws");
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
    draw_frame(&world, camera, &metrics, &[], &mut canvas).expect("the world draws");
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
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
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
    let opening = draw_frame(&world, camera, &metrics, &[], &mut canvas)
        .expect("the world draws")
        .extent_shown();
    let mut widest = opening;

    // Twenty-four steps take the tile size from the opening twelve pixels
    // to the smallest the viewer allows.
    for _ in 0..24 {
        camera = camera.zoomed_out(&canvas).clamped(&world, &canvas);
        let readout =
            draw_frame(&world, camera, &metrics, &[], &mut canvas).expect("the world draws");
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

    let readout =
        draw_frame(&world, camera, &metrics, &[], &mut with_panel).expect("the world draws");
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

#[test]
fn the_panel_names_the_ground_in_the_window() {
    // The product record asks that the kinds be few and that a person be able
    // to name them.[^1] A picture of five colours with no names leaves the
    // reader guessing which green is forest.
    //
    // [^1]: PRD-0003, a developer sees a world worth looking at. `docs/product/accepted/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
    let mut world = world();
    let metrics = stepped(&mut world, 2);
    let mut canvas = canvas();
    let camera = Camera::fitting(&world, &canvas);

    let readout = draw_frame(&world, camera, &metrics, &[], &mut canvas).expect("the world draws");

    let counted: u32 = readout.by_kind().iter().sum();
    assert_eq!(
        counted,
        readout.tiles_painted(),
        "the ground legend and the drawing pass disagree on how many tiles were painted"
    );

    // Every kind must appear, or the fixture supplies no case and the
    // assertion measures the fixture.
    let named = readout.by_kind().iter().filter(|count| **count > 0).count();
    assert_eq!(
        named, KIND_COUNT,
        "the window holds {named} of the {KIND_COUNT} kinds of ground, so the \
         panel cannot be shown to name all of them"
    );
}

#[test]
fn the_ground_legend_follows_the_window_when_the_person_scrolls() {
    // The count is of the window, not of the world. A panel that reported the
    // whole world would not move when the camera does.
    let mut world = world();
    let metrics = stepped(&mut world, 1);
    let mut canvas = canvas();

    let here = Camera::fitting(&world, &canvas);
    let first = draw_frame(&world, here, &metrics, &[], &mut canvas).expect("the world draws");
    let before = *first.by_kind();

    let there = here
        .zoomed_in(&canvas)
        .zoomed_in(&canvas)
        .zoomed_in(&canvas);
    let second = draw_frame(&world, there, &metrics, &[], &mut canvas).expect("the world draws");

    assert_ne!(
        before,
        *second.by_kind(),
        "the ground legend said the same thing at two zoom levels, so it counts the world"
    );
}

#[test]
fn the_panel_states_the_region_under_the_crosshair() {
    // Level 1 exists and only its own tests read it. A person watching the
    // world sees tiles and units and no region, so the level the engine
    // maintains every frame is invisible to the person it was built for.
    let mut world = world();
    let metrics = stepped(&mut world, 2);
    let mut canvas = canvas();
    let camera = at_the_middle(&world, &canvas);

    let readout = draw_frame(&world, camera, &metrics, &[], &mut canvas).expect("the world draws");
    let region = readout
        .region()
        .expect("the middle of the world names a cell");

    // The region is a region and not the window. A cell covers a block of
    // tiles, and this world is far larger than one block, so the two counts
    // must differ.
    let (across, down) = readout.extent_shown();
    assert!(
        region.tiles() < i64::from(across) * i64::from(down),
        "the region covers the window, so the panel reports one thing twice"
    );
    assert!(region.tiles() > 0, "the region covers no tile");

    // The engine reports the cell that covers the tile the camera reports.
    let under = world
        .summary_covering(readout.centre())
        .expect("the centre tile names a cell");
    assert_eq!(
        region, under,
        "the panel reports a cell other than the one under the crosshair"
    );
}

#[test]
fn the_panel_reports_a_region_that_holds_units() {
    // A section that always says zero would satisfy every assertion above and
    // would show a person nothing. The world spreads units over the ground,
    // so some cell holds one, and the crosshair must be able to find it.
    let mut world = world();
    let metrics = stepped(&mut world, 2);
    let mut canvas = canvas();

    let mut with_units = 0;
    for step in 0..8 {
        let camera = at_the_middle(&world, &canvas).stepped(step as f32 * 40.0, 0.0);
        let readout =
            draw_frame(&world, camera, &metrics, &[], &mut canvas).expect("the world draws");
        if let Some(region) = readout.region() {
            if region.units() > 0 {
                with_units += 1;
                // A cell that holds a unit holds open ground for it to stand
                // on, so the intensive reading must exist rather than divide
                // by nothing.
                assert!(
                    region.units_for_each_open_tile().is_some(),
                    "a region holds units and reports no crowding"
                );
            }
        }
    }
    assert!(
        with_units > 0,
        "no camera position found a region that holds a unit"
    );
}

#[test]
fn a_short_window_cuts_the_panel_and_the_panel_says_so() {
    // The panel's height follows its content, and nothing bounded it against
    // the window. A canvas shorter than the panel cut the bottom off, and
    // `bounds` then stated a rectangle the panel did not paint.
    //
    // A number below the edge of the window is a number the panel silently
    // does not have, which is the failure ADR-0070 D2 exists to prevent for a
    // number the panel cannot afford.
    let mut world = world();
    let metrics = stepped(&mut world, 1);

    let mut tall = Canvas::new(WINDOW.0, WINDOW.1);
    let camera = at_the_middle(&world, &tall);
    let full = draw_frame(&world, camera, &metrics, &[], &mut tall).expect("the world draws");
    let (_, top, _, full_height) = cachette_view::hud::bounds(&full);

    // A window that cannot hold the whole panel. The fixture must actually be
    // short, or the assertions below pass on a panel nothing cut.
    let short_height = (top + full_height) as usize - 80;
    assert!(
        short_height < WINDOW.1,
        "the short window is not shorter than the tall one"
    );
    let mut short = Canvas::new(WINDOW.0, short_height);
    let camera = at_the_middle(&world, &short);
    let cut = draw_frame(&world, camera, &metrics, &[], &mut short).expect("the world draws");
    let (_, top, _, cut_height) = cachette_view::hud::bounds(&cut);

    assert!(
        cut_height < full_height,
        "the panel states the same height in a window {} pixels shorter",
        WINDOW.1 - short_height
    );
    assert!(
        top + cut_height <= short_height as i32,
        "the panel states a rectangle {cut_height} tall that runs past a canvas of {short_height}"
    );
}

#[test]
fn a_cut_panel_paints_the_rectangle_it_states() {
    // The claim in the drawing code is that the panel cannot paint past the
    // rectangle it states. A cut panel must keep it.
    let mut world = world();
    let metrics = stepped(&mut world, 1);

    let mut bare = Canvas::new(WINDOW.0, 320);
    let camera = at_the_middle(&world, &bare);
    paint::draw(&world, camera, &mut bare).expect("the world draws");

    let mut with_panel = Canvas::new(WINDOW.0, 320);
    let readout =
        draw_frame(&world, camera, &metrics, &[], &mut with_panel).expect("the world draws");
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

    assert_eq!(
        lowest,
        top + height - 1,
        "the cut panel states a rectangle {height} tall and paints down to {lowest}"
    );
}

// The founding rows. The panel says what the founding chose, what it left,
// and how many places it compared. Every one of these numbers is a value the
// caller holds from before the first frame, and the panel recomputes none of
// them.[^1] [^2]
//
// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
// [^2]: ADR-0075, the founding choice reads a bounded sample of the world, decision D5. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`

/// The people a founded run in these tests begins with.
const GROUP: u32 = 24;

/// Builds a world and founds a run in it.
///
/// The world is this test's own. It is not the world the demonstration
/// binary builds, because that world is chosen to look right rather than to
/// produce an edge value.[^1]
///
/// The fixture asserts its own outcome. It reads the survey back and refuses
/// a survey whose best rejected place reaches the same quantities as the
/// chosen place. A panel that printed the chosen quantities in both columns
/// would pass against such a survey.[^1] [^2]
///
/// # References
///
/// [^1]: Testing Rules, a fixture supplies the input. `.claude/rules/testing.md`
/// [^2]: Findings register, FND-061. `docs/FINDINGS.md`
fn founded_world() -> (World, Vec<FoundingOutcome>) {
    let mut world = World::new(WorldConfig {
        width: WIDE,
        height: TALL,
        seed: 11,
        faction_count: FACTIONS,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("a large extent describes a world");
    // The run founds one group for each faction. These tests state one
    // founding section, so the fixture keeps the first outcome. The other
    // groups stand in the world all the same.
    let outcomes = world.found_run_for_every_faction(GROUP);
    let founded = outcomes
        .first()
        .and_then(FoundingOutcome::founding)
        .expect("the world holds a place for the first group");
    let survey = founded.survey();
    let chosen = survey.chosen().expect("the founding chose a place");
    let other = *survey
        .rejected()
        .first()
        .expect("the founding compared more than one place");
    assert_ne!(
        other.provision(),
        chosen.provision(),
        "the best place the founding left reaches the same quantities as the \
         place it took, so a panel that printed one place twice would pass",
    );
    world.rebuild_bridge(1).expect("the rebuild must succeed");
    (world, outcomes.into_iter().take(1).collect())
}

/// Returns the survey of the first seated faction.
fn first_survey(outcomes: &[FoundingOutcome]) -> &cachette_core::founding::Survey {
    outcomes
        .iter()
        .find_map(FoundingOutcome::founding)
        .expect("the run seated a faction")
        .survey()
}

/// Returns the place the first seated faction took.
fn first_place(outcomes: &[FoundingOutcome]) -> Axial {
    outcomes
        .iter()
        .find_map(|outcome| outcome.founding().map(cachette_core::Founding::place))
        .expect("the run seated a faction")
}

/// Returns a camera that looks at one tile.
fn looking_at(world: &World, canvas: &Canvas, address: Axial) -> Camera {
    Camera::opening()
        .looking_at(address, canvas)
        .clamped(world, canvas)
}

#[test]
fn the_panel_states_the_place_the_founding_chose() {
    // The row must carry the address the engine reported. A panel that
    // printed the camera centre would look right on the opening frame.
    let (world, foundings) = founded_world();
    let metrics = Metrics::start();
    let mut canvas = canvas();
    let camera = looking_at(&world, &canvas, first_place(&foundings));

    let readout =
        draw_frame(&world, camera, &metrics, &foundings, &mut canvas).expect("the world draws");

    let report = readout.foundings().first().expect("the panel read one");
    assert_eq!(
        report.place(),
        first_place(&foundings),
        "the panel names a place the founding did not choose",
    );
}

#[test]
fn the_panel_states_the_quantities_the_founding_reported() {
    // The assertion is against the report, not against a constant. A
    // constant would pin the terrain generator rather than the panel.
    let (world, foundings) = founded_world();
    let metrics = Metrics::start();
    let mut canvas = canvas();
    let camera = looking_at(&world, &canvas, first_place(&foundings));

    let readout =
        draw_frame(&world, camera, &metrics, &foundings, &mut canvas).expect("the world draws");

    let survey = first_survey(&foundings);
    let chosen = survey.chosen().expect("the founding chose a place");
    let report = readout.foundings().first().expect("the panel read one");
    assert_eq!(
        report.chosen(),
        chosen.provision(),
        "the panel states quantities the survey did not read",
    );

    // The quantities must not be the empty ones. A report of zeroes would
    // satisfy an equality against a survey that read nothing.
    assert!(
        report.chosen().open_ground > 0,
        "the chosen place reaches no open ground, so the fixture found a \
         place no group could settle",
    );
}

#[test]
fn the_panel_states_how_many_places_the_founding_compared() {
    // A watcher tells a choice from a default by this number. One place
    // compared is a default.
    let (world, foundings) = founded_world();
    let metrics = Metrics::start();
    let mut canvas = canvas();
    let camera = looking_at(&world, &canvas, first_place(&foundings));

    let readout =
        draw_frame(&world, camera, &metrics, &foundings, &mut canvas).expect("the world draws");

    let report = readout.foundings().first().expect("the panel read one");
    assert_eq!(report.considered(), first_survey(&foundings).considered());
    assert!(
        report.considered() > 1,
        "the founding compared {} places, so the panel describes a default \
         rather than a choice",
        report.considered(),
    );
}

#[test]
fn the_panel_states_a_place_the_founding_did_not_choose() {
    // The comparison is the point of the section. A panel that stated the
    // chosen place alone would answer half the question.
    let (world, foundings) = founded_world();
    let metrics = Metrics::start();
    let mut canvas = canvas();
    let camera = looking_at(&world, &canvas, first_place(&foundings));

    let readout =
        draw_frame(&world, camera, &metrics, &foundings, &mut canvas).expect("the world draws");

    let best_rejected = *first_survey(&foundings)
        .rejected()
        .first()
        .expect("the founding compared more than one place");
    let report = readout.foundings().first().expect("the panel read one");
    let (address, provision) = report.other().expect("the panel states a place it left");

    assert_eq!(address, best_rejected.address());
    assert_eq!(provision, best_rejected.provision());
    assert_ne!(
        address,
        report.place(),
        "the panel states the chosen place in both columns",
    );
    assert_ne!(
        provision,
        report.chosen(),
        "the panel states the chosen quantities in both columns",
    );
}

#[test]
fn the_panel_says_whether_the_window_shows_the_founded_place() {
    // The row must answer both ways. A row that always said yes would be
    // right on the opening frame of the demonstration and wrong after one
    // scroll.
    let (world, foundings) = founded_world();
    let metrics = Metrics::start();
    let mut canvas = canvas();
    let place = first_place(&foundings);

    let at_the_place = looking_at(&world, &canvas, place);
    let near = draw_frame(&world, at_the_place, &metrics, &foundings, &mut canvas)
        .expect("the world draws");
    assert!(
        near.foundings()[0].shown(),
        "the camera looks at {place:?} and the panel says the window does not show it",
    );

    // A camera at the far corner from the place cannot hold it, because the
    // window covers a small part of a world of this extent.
    let far = if place.q * 2 < WIDE as i32 {
        Axial::new(WIDE as i32 - 1, TALL as i32 - 1)
    } else {
        Axial::new(0, 0)
    };
    let away = looking_at(&world, &canvas, far);
    let distant =
        draw_frame(&world, away, &metrics, &foundings, &mut canvas).expect("the world draws");
    assert!(
        !distant.foundings()[0].shown(),
        "the camera looks at {far:?} and the panel says the window shows {place:?}",
    );
}

#[test]
fn the_panel_describes_every_founding_the_caller_holds() {
    // The layout must not assume one founding. A run may found more than
    // one group, and a panel written for one would then state a false
    // thing.[^1]
    //
    // [^1]: Blockers register, BLK-018. `docs/BLOCKERS.md`
    let mut world = World::new(WorldConfig {
        width: WIDE,
        height: TALL,
        seed: 11,
        faction_count: FACTIONS,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("a large extent describes a world");
    // The run founds one group for each faction, so the fixture holds every
    // outcome the engine gave and never a list the test assembled.
    let foundings = world.found_run_for_every_faction(GROUP);
    let seated = foundings
        .iter()
        .filter(|outcome| outcome.is_seated())
        .count();
    assert!(
        seated > 1,
        "the run seated {seated} factions, so a panel written for one \
         founding would pass this test",
    );
    world.rebuild_bridge(1).expect("the rebuild must succeed");

    let metrics = Metrics::start();
    let mut canvas = canvas();
    let camera = at_the_middle(&world, &canvas);

    let readout =
        draw_frame(&world, camera, &metrics, &foundings, &mut canvas).expect("the world draws");

    assert_eq!(
        readout.foundings().len(),
        seated,
        "the panel dropped a founding the caller holds",
    );
    assert_ne!(
        readout.foundings()[0].place(),
        readout.foundings()[1].place(),
        "the panel states one place twice",
    );
    // Each row names the faction that founded. Two rows with one faction
    // would be one report counted twice.
    assert_ne!(
        readout.foundings()[0].faction(),
        readout.foundings()[1].faction(),
        "the panel gives two foundings to one faction",
    );
}

#[test]
fn the_panel_states_no_founding_when_the_caller_holds_none() {
    // A caller that founded nothing must get a panel with no founding
    // section, rather than a section of zeroes.
    let mut world = world();
    let metrics = stepped(&mut world, 1);
    let mut canvas = canvas();
    let camera = at_the_middle(&world, &canvas);

    let readout = draw_frame(&world, camera, &metrics, &[], &mut canvas).expect("the world draws");

    assert!(readout.foundings().is_empty());
}

#[test]
fn the_founding_rows_fit_their_column() {
    // A value too wide for its column is cut, and a cut value states a
    // number other than the one it was given. The check must see the
    // founding rows, so the readout carries a founding.
    let (world, foundings) = founded_world();
    let metrics = Metrics::start();
    let mut canvas = canvas();
    let camera = looking_at(&world, &canvas, first_place(&foundings));

    let readout =
        draw_frame(&world, camera, &metrics, &foundings, &mut canvas).expect("the world draws");

    let cut = cachette_view::hud::values_that_do_not_fit(&readout);
    assert!(
        cut.is_empty(),
        "the panel cut {cut:?} from the founding rows"
    );
}

#[test]
fn the_panel_grows_when_the_caller_holds_a_founding() {
    // The founding section must reach the panel. A readout that held the
    // report and drew none of it would pass every assertion above.
    //
    // The canvas is taller than the demonstration window. The founding
    // sections sit last, so the demonstration window cuts them, and a cut
    // panel is the same height whatever it holds.[^2] This test asks whether
    // the section reaches a line at all, so it needs a window that holds the
    // whole panel.
    //
    // [^2]: Backlog item 0188. `docs/backlog/complete/0188-show-the-food-of-a-tile-and-the-reason-a-unit-chose.md`
    let (world, foundings) = founded_world();
    let metrics = Metrics::start();
    let mut with_founding = Canvas::new(WINDOW.0, WHOLE_PANEL);
    let mut without = Canvas::new(WINDOW.0, WHOLE_PANEL);
    let camera = looking_at(&world, &with_founding, first_place(&foundings));

    let stated = draw_frame(&world, camera, &metrics, &foundings, &mut with_founding)
        .expect("the world draws");
    let silent = draw_frame(&world, camera, &metrics, &[], &mut without).expect("the world draws");

    let (_, _, _, tall) = cachette_view::hud::bounds(&stated);
    let (_, _, _, short) = cachette_view::hud::bounds(&silent);
    assert!(
        tall > short,
        "the panel is {tall} pixels tall with a founding and {short} without \
         it, so the founding section reached no line",
    );
    assert_ne!(
        with_founding.pixels(),
        without.pixels(),
        "the founding section painted nothing",
    );
}
