//! No line of a panel writes outside the panel, whatever the text says.
//!
//! Two line kinds used to write from the left margin with no right bound, so
//! the length of the text was the only thing that kept them inside the
//! rectangle. One note ran past the edge, and it was found by disabling a
//! section rather than by reading the code.[^1]
//!
//! The standard now cuts every kind at one writer, which takes a right edge.
//! These tests give each kind a text longer than any panel and assert that no
//! pixel outside the rectangle moved.
//!
//! # The tests can fail
//!
//! One test writes the same text through the bare canvas, which is the uncut
//! path the panel used to take, and asserts that the ink does escape. Without
//! it, a test that only looked outside the rectangle would pass on a panel
//! that drew nothing at all.[^2]
//!
//! # References
//!
//! [^1]: Backlog item 0300. `docs/backlog/complete/0300-cut-every-panel-line-to-the-width-of-the-panel.md`
//! [^2]: Testing Rules, a determinism test must be able to fail. `.claude/rules/testing.md`

// An integration test is its own crate, so the allowance at the viewer's crate
// root does not reach it. The reason is the same one: ADR-0067 D3 puts the
// float boundary at the viewer, and a camera is a viewer value.
#![allow(clippy::disallowed_types)]

use std::time::Duration;

use cachette_core::{Axial, FactionId, World, WorldConfig};
use cachette_view::panel::{self, Line, Set};
use cachette_view::{draw_frame, Camera, Canvas, Metrics, Overlay};

/// The width of the frame the tests draw into.
const WIDE: usize = 900;

/// The height of the frame the tests draw into.
const TALL: usize = 700;

/// The place a test panel is drawn at.
const LEFT: i32 = 40;

/// The top of the place a test panel is drawn at.
const TOP: i32 = 30;

/// A text longer than any panel, at any glyph size.
///
/// The panel is 268 pixels wide and a glyph is 8 pixels, so 34 glyphs fill it.
/// This is many times that, so no bound short of an escape lets it fit.
fn a_long_text() -> String {
    "the quick brown fox jumps over the lazy dog and keeps going".repeat(4)
}

/// Returns every line kind, each holding a text longer than the panel.
fn every_kind_too_long() -> Vec<Line> {
    let long = a_long_text();
    vec![
        Line::Title(long.clone()),
        Line::note(long.clone()),
        Line::heading(long.clone()),
        Line::Rule,
        Line::row(long.clone(), long.clone()),
        Line::swatch(0x00ff_00ff, long.clone(), long),
    ]
}

/// Reports whether a pixel sits inside the rectangle a panel states.
const fn inside(x: i32, y: i32, height: i32) -> bool {
    x >= LEFT && x < LEFT + panel::WIDTH && y >= TOP && y < TOP + height
}

/// Returns every pixel outside the rectangle that differs from the ground.
fn escaped(canvas: &Canvas, ground: &[u32], height: i32) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    for y in 0..canvas.height() {
        for x in 0..canvas.width() {
            let at = y * canvas.width() + x;
            if canvas.pixels()[at] == ground[at] {
                continue;
            }
            let (x, y) = (x as i32, y as i32);
            if !inside(x, y, height) {
                out.push((x, y));
            }
        }
    }
    out
}

#[test]
fn no_line_kind_writes_outside_the_panel() {
    let mut canvas = Canvas::new(WIDE, TALL);
    let ground = canvas.pixels().to_vec();

    let lines = every_kind_too_long();
    let height = panel::draw_one(&mut canvas, LEFT, TOP, &lines);

    let out = escaped(&canvas, &ground, height);
    assert!(
        out.is_empty(),
        "the panel wrote {} pixels outside the rectangle it states, first at {:?}",
        out.len(),
        out.first()
    );
}

#[test]
fn the_uncut_path_does_escape_the_panel() {
    // This is the proof that the test above can fail. It writes the same text
    // at the same place through the bare canvas, which is what the panel did
    // before the standard. The ink must reach outside the rectangle.
    let mut canvas = Canvas::new(WIDE, TALL);
    let ground = canvas.pixels().to_vec();

    let lines = every_kind_too_long();
    let height = panel::height_of(&lines);
    canvas.write(
        LEFT + panel::PAD,
        TOP + panel::PAD,
        &a_long_text(),
        1,
        0x00ff_ffff,
    );

    let out = escaped(&canvas, &ground, height);
    assert!(
        !out.is_empty(),
        "an uncut write of a text longer than the panel must reach outside it"
    );
}

#[test]
fn every_over_long_line_reports_that_it_was_cut() {
    // A cut keeps the ink inside the panel and still states something other
    // than the number it was given. A test that only checked the edge would
    // pass because of the cut rather than in spite of it.
    for line in every_kind_too_long() {
        if line == Line::Rule {
            assert!(!line.is_cut(), "a rule carries no text and is never cut");
            continue;
        }
        assert!(line.is_cut(), "{line:?} is longer than the panel");
    }
}

#[test]
fn a_line_that_fits_reports_no_cut() {
    // The check must be able to answer no. A check with one answer is
    // decoration.
    for line in [
        Line::Title("SHORT".to_string()),
        Line::note("a short note"),
        Line::heading("A HEADING"),
        Line::row("tick", "1 200"),
        Line::swatch(0x0000_ff00, "faction 0", "64"),
        Line::Rule,
    ] {
        assert!(!line.is_cut(), "{line:?} fits the panel");
    }
}

#[test]
fn the_fit_cuts_on_a_whole_glyph() {
    // The bound is derived from the room and the glyph table. A cut that left
    // half a glyph would put ink one pixel past the edge.
    let room = 8 * 5;
    assert_eq!(panel::fit("abcdefgh", room, 1), "abcde");
    assert_eq!(panel::fit("abcdefgh", room, 2), "ab");
    assert_eq!(panel::fit("abc", room, 1), "abc");
    assert_eq!(panel::fit("abc", -10, 1), "");
}

/// Builds a small world with units, so the deck has something to read.
fn a_world() -> World {
    let mut world = World::new(WorldConfig {
        width: 120,
        height: 90,
        seed: 0x0123_4567_89ab_cdef,
        faction_count: 3,
        ..WorldConfig::default()
    })
    .expect("the world builds");
    let mut placed = 0;
    for r in 0..90i32 {
        for q in 0..120i32 {
            if (q + r) % 7 != 0 {
                continue;
            }
            let address = Axial { q, r };
            if !world.admits_a_unit(address) {
                continue;
            }
            let faction = FactionId((placed % 3) as u16);
            if world.spawn_soldier(address, faction).is_ok() {
                placed += 1;
            }
        }
    }
    world.rebuild_bridge(1).expect("the bridge rebuilds");
    world
}

#[test]
fn the_frame_command_draws_the_deck() {
    // The engine is obligated to draw the deck, so the test starts at the
    // frame command rather than at the deck.
    let world = a_world();
    let mut canvas = Canvas::new(WIDE, TALL);
    let camera = Camera::fitting(&world, &canvas);
    let metrics = Metrics::fixed(
        10,
        10,
        Duration::from_micros(500),
        Duration::from_micros(200),
        Duration::from_millis(400),
    );

    let bare = {
        let mut bare = Canvas::new(WIDE, TALL);
        draw_frame(
            &world,
            camera,
            &metrics,
            &[],
            Overlay::Glass { reference: false },
            &mut bare,
        )
        .expect("the world draws");
        bare.pixels().to_vec()
    };

    draw_frame(
        &world,
        camera,
        &metrics,
        &[],
        Overlay::Deck {
            reference: false,
            panels: Set::all(),
            pointer: Some(Axial { q: 10, r: 10 }),
        },
        &mut canvas,
    )
    .expect("the world draws");

    assert_ne!(
        canvas.pixels(),
        bare.as_slice(),
        "the deck must change the frame"
    );
}

#[test]
fn the_deck_says_the_title_of_every_panel_it_holds() {
    let world = a_world();
    let mut canvas = Canvas::new(WIDE, TALL);
    let camera = Camera::fitting(&world, &canvas);
    let metrics = Metrics::fixed(
        10,
        10,
        Duration::from_micros(500),
        Duration::from_micros(200),
        Duration::from_millis(400),
    );
    draw_frame(
        &world,
        camera,
        &metrics,
        &[],
        Overlay::Deck {
            reference: false,
            panels: Set::all(),
            pointer: None,
        },
        &mut canvas,
    )
    .expect("the world draws");

    let view = panel::View {
        world: &world,
        camera,
        frame_width: canvas.width(),
        frame_height: canvas.height(),
        focus: canvas.focus(),
        pointer: None,
    };
    let said = panel::says(&view, Set::all());
    for registered in panel::registered() {
        assert!(
            said.iter().any(|line| line == registered.title()),
            "the deck must state the title {}",
            registered.title()
        );
    }
}

#[test]
fn no_registered_panel_holds_a_line_the_panel_must_cut() {
    // A cut line states something other than what it was given, in silence.
    // This asserts that no panel of the deck writes one.
    let world = a_world();
    let mut canvas = Canvas::new(WIDE, TALL);
    let camera = Camera::fitting(&world, &canvas);
    let metrics = Metrics::fixed(
        10,
        10,
        Duration::from_micros(500),
        Duration::from_micros(200),
        Duration::from_millis(400),
    );
    draw_frame(
        &world,
        camera,
        &metrics,
        &[],
        Overlay::Deck {
            reference: false,
            panels: Set::all(),
            pointer: Some(Axial { q: 4, r: 4 }),
        },
        &mut canvas,
    )
    .expect("the world draws");

    let view = panel::View {
        world: &world,
        camera,
        frame_width: canvas.width(),
        frame_height: canvas.height(),
        focus: canvas.focus(),
        pointer: Some(Axial { q: 4, r: 4 }),
    };
    let bad = panel::lines_that_do_not_fit(&view, Set::all());
    assert!(bad.is_empty(), "these lines do not fit the panel: {bad:?}");
}

#[test]
fn a_set_refuses_a_name_no_panel_carries() {
    assert!(Set::EMPTY.with("no such panel").is_none());
    for registered in panel::registered() {
        assert!(Set::EMPTY.with(registered.name()).is_some());
    }
}
