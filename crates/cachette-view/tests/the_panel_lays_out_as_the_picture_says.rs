//! The panel's layout matches a stored picture.
//!
//! The other tests of the panel read one line at a time. They read the tick,
//! the counts and the cost rows, and they check that every pixel the panel
//! writes stays inside the rectangle the panel states.
//!
//! Three layout defects pass all of that. A value that overruns its column is
//! cut, so it stays inside the rectangle and states a different number. A row
//! that sits over the row above it stays inside the rectangle too. A row below
//! the window edge is simply absent. One of the three already happened: a row
//! of two counts ran past the panel edge, and only a rendered picture showed
//! it.
//!
//! A picture shows the layout as a whole, which is the only way those three
//! are visible.
//!
//! # What the stored picture holds
//!
//! The picture holds one character for each pixel. A pixel the panel left
//! alone is the ground character, and so is a pixel the panel only shaded.
//! Every other pixel takes the character of the colour it was written in.
//!
//! The picture therefore says where the panel put ink, and says nothing about
//! the ground under it. A change to the terrain colours does not change it.
//!
//! # How to regenerate the picture
//!
//! Run the test with the environment variable `UPDATE_PANEL_PICTURE` set to
//! `1`. The test then writes the picture it got over the stored one, and
//! passes. Read the difference before you commit it.
//!
//! ```text
//! UPDATE_PANEL_PICTURE=1 cargo test -p cachette-view --test the_panel_lays_out_as_the_picture_says
//! ```
//!
//! The test reads the expected picture from the file and never computes
//! it.[^1]
//!
//! # What makes the picture change
//!
//! The panel prints counts of the window, so the picture changes when the
//! terrain generator changes what the window holds. That is a change to what
//! the panel says, and a person must look at it. Every clock reading is out
//! of the fixture, so nothing else in the picture moves between runs.[^2]
//!
//! # References
//!
//! [^1]: Testing Rules, a determinism test must be able to fail. `.claude/rules/testing.md`
//! [^2]: ADR-0070, the head-up display reports what the drawing pass read. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`

// An integration test is its own crate, so the allowance at the viewer's
// crate root does not reach it. The reason is the same one: ADR-0067 D3 puts
// the float boundary at the viewer, and a camera is a viewer value.
#![allow(clippy::disallowed_types)]

use std::path::PathBuf;
use std::time::Duration;

use cachette_core::founding::FoundingOutcome;
use cachette_core::{Axial, FactionId, World, WorldConfig};
use cachette_view::picture::write_ppm;
use cachette_view::{draw_frame, hud, paint, Camera, Canvas, Metrics};

/// The width of the world the picture is taken of.
const WIDE: u32 = 200;

/// The height of the world the picture is taken of.
const TALL: u32 = 140;

/// The soldiers the world holds.
const SOLDIERS: u32 = 2200;

/// The factions the soldiers divide between.
///
/// Four factions give four legend rows and four bands in the bar. A world of
/// one faction would give a shorter panel and a bar of one colour, so a whole
/// section of the layout would be missing from the picture.
const FACTIONS: u16 = 4;

/// The stride that spreads the soldiers over the open ground.
const SPREAD: u32 = 9973;

/// The size of the window the picture is taken in.
///
/// The window is only as wide as the panel needs, because every pixel of it
/// is a character of the stored picture. The height holds the whole panel,
/// which two assertions below check.
///
/// The height grows when the panel gains a section. A window that cut the
/// panel would store a picture of the cut rather than a picture of the
/// layout, and the ground rows would fall off the bottom first.
const WINDOW: (usize, usize) = (340, 860);

/// The steps the fixed measurements report.
const TICKS: u64 = 40;

/// The paintings the fixed measurements report.
const FRAMES: u64 = 40;

/// The people the fixture founds its run with.
const GROUP: u32 = 24;

/// Builds the world the picture is taken of, and founds a run in it.
///
/// The soldiers spread over the whole world with a large stride, so the
/// window holds soldiers of every faction and the legend rows hold real
/// counts. A window with no soldier would give a picture of an empty bar and
/// four zeroes, and the test would then measure the fixture.[^1]
///
/// The run is founded, because the panel now states what the founding chose
/// beside a place it left. A fixture with no founding would store a picture
/// with no founding section, and the comparison would then pass against
/// nothing.
///
/// The fixture asserts its own outcome rather than its inputs.[^2] It reads
/// the survey back and refuses a survey whose best rejected place holds the
/// same quantities as the chosen one. Such a survey would give two identical
/// blocks of rows, and a panel that printed the chosen quantities twice would
/// pass.[^1]
///
/// # References
///
/// [^1]: Testing Rules, a fixture supplies the input. `.claude/rules/testing.md`
/// [^2]: Findings register, FND-061. `docs/FINDINGS.md`
fn world(factions: u16) -> (World, Vec<FoundingOutcome>) {
    let mut world = World::new(WorldConfig {
        width: WIDE,
        height: TALL,
        seed: 11,
        faction_count: factions,
    })
    .expect("the extent describes a world");
    // The run founds one group for each faction. The fixture keeps the
    // first outcome, because this picture is of a panel with one founding
    // section. The other groups stand in the world all the same.
    let outcomes = world.found_run_for_every_faction(GROUP);
    let founded = outcomes
        .first()
        .and_then(FoundingOutcome::founding)
        .expect("the world holds a place for the first group")
        .clone();
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
         place it took, so the two blocks of rows cannot be told apart",
    );

    let grid = world.grid();
    let open: Vec<Axial> = (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| world.admits_a_unit(*address))
        .collect();
    assert!(!open.is_empty(), "the world must have open ground");
    for index in 0..SOLDIERS {
        let at = open[(index.wrapping_mul(SPREAD) as usize) % open.len()];
        world
            .spawn_soldier(at, FactionId((index % u32::from(factions)) as u16))
            .expect("the address and the faction are valid");
    }
    world.rebuild_bridge(1).expect("the rebuild must succeed");
    (world, outcomes.into_iter().take(1).collect())
}

/// Returns a camera pointed at the middle of the world.
fn at_the_middle(world: &World, canvas: &Canvas) -> Camera {
    let opening = Camera::opening();
    let across = f32::from(WIDE as u16) / 2.0 * opening.tile_width;
    let down = f32::from(TALL as u16) / 2.0 * opening.tile_height;
    opening.panned(across, down).clamped(world, canvas)
}

/// Returns measurements that hold the same figures on every run.
///
/// Two rows of the panel divide by the wall clock span. A fixture that read a
/// clock would print a new number every run, and no picture of it could be
/// stored.
fn measurements() -> Metrics {
    Metrics::fixed(
        TICKS,
        FRAMES,
        Duration::from_micros(1_250),
        Duration::from_micros(3_400),
        Duration::from_millis(2_000),
    )
}

/// Returns the path of the stored picture.
fn stored_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/pictures/panel.txt")
}

/// The command that writes the stored picture again.
const REGENERATE: &str = "UPDATE_PANEL_PICTURE=1 cargo test -p cachette-view \
     --test the_panel_lays_out_as_the_picture_says";

/// Draws the frame the picture is taken of, and returns the picture.
///
/// The whole frame is drawn, because a panel that renders on its own proves
/// that a panel renders and not that anything reaches it.[^1]
///
/// The second canvas holds the same drawing without the panel. The picture is
/// the difference between the two, so the ground never reaches it.
///
/// # References
///
/// [^1]: Testing Rules, drive the real caller. `.claude/rules/testing.md`
fn taken(factions: u16) -> (String, Canvas) {
    let (world, foundings) = world(factions);
    let metrics = measurements();
    let mut panelled = Canvas::new(WINDOW.0, WINDOW.1);
    let mut bare = Canvas::new(WINDOW.0, WINDOW.1);
    let camera = at_the_middle(&world, &panelled);

    let readout =
        draw_frame(&world, camera, &metrics, &foundings, &mut panelled).expect("the world draws");
    assert_eq!(
        readout.foundings().len(),
        1,
        "the panel dropped the founding the fixture passed it",
    );
    paint::draw(&world, camera, &mut bare).expect("the world draws");

    // The window must hold the whole panel. A cut panel is a different
    // layout, and the picture would then record the cut rather than the
    // panel.
    let (_, top, _, height) = hud::bounds(&readout);
    assert!(
        top + height < WINDOW.1 as i32,
        "the panel is {height} pixels tall and the window is {} pixels tall, \
         so the window cuts the panel",
        WINDOW.1,
    );

    (hud::ink_map(&panelled, &bare), panelled)
}

#[test]
fn the_panel_matches_the_stored_picture() {
    let (got, canvas) = taken(FACTIONS);
    let path = stored_path();

    if std::env::var("UPDATE_PANEL_PICTURE").as_deref() == Ok("1") {
        std::fs::create_dir_all(path.parent().expect("the path has a directory"))
            .expect("the directory must open");
        std::fs::write(&path, &got).expect("the picture must write");
        return;
    }

    let want = std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "the stored picture at {} would not open: {error}. \
             Write it with `{REGENERATE}`",
            path.display(),
        )
    });

    if got == want {
        return;
    }

    // A message that says the pictures differ is not usable. The test writes
    // what it got, as characters and as an image, so a person can look at
    // both.
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let got_text = directory.join("panel-got.txt");
    let got_image = directory.join("panel-got.ppm");
    std::fs::write(&got_text, &got).expect("the picture must write");
    let mut file = std::io::BufWriter::new(
        std::fs::File::create(&got_image).expect("the image file must open"),
    );
    write_ppm(&canvas, &mut file).expect("the image must write");
    drop(file);

    let line = first_difference(&got, &want);
    panic!(
        "the panel does not match the stored picture. The first row that \
         differs is row {line}.\n  stored:  {}\n  got:     {}\n  image:   {}\n\
         Look at the image. When the new layout is right, write the picture \
         again with `{REGENERATE}`",
        path.display(),
        got_text.display(),
        got_image.display(),
    );
}

/// Returns the number of the first row that differs, counting from zero.
fn first_difference(got: &str, want: &str) -> usize {
    got.lines()
        .zip(want.lines())
        .position(|(a, b)| a != b)
        .unwrap_or_else(|| got.lines().count().min(want.lines().count()))
}

#[test]
fn the_picture_holds_a_panel_with_content() {
    // A fixture that drew an empty panel would store an empty picture, and
    // the comparison would then pass against nothing.
    let (got, _) = taken(FACTIONS);
    for mark in [
        '#', 'h', 'l', 'v', 'y', 'r', 'b', 'g', 'W', 'P', 'F', 'H', 'M',
    ] {
        assert!(
            got.contains(mark),
            "the picture holds no {mark}, so the fixture drew a panel without \
             that part of its layout",
        );
    }
    assert!(
        !got.contains('?'),
        "the panel wrote a colour that the ink table does not name",
    );
    let ink = got.chars().filter(|mark| *mark != '\n').count();
    assert!(
        ink > 10_000,
        "the picture holds only {ink} marks, which is too few for a full panel",
    );
}

#[test]
fn the_picture_moves_when_a_row_moves() {
    // A comparison with no proven failure mode is decoration. Two
    // perturbations prove this one answers no.
    let (got, _) = taken(FACTIONS);
    let want = std::fs::read_to_string(stored_path()).expect("the stored picture must open");
    assert_eq!(got, want, "the fixture must match the stored picture first");

    // One column of drift. Every row moves one pixel to the right.
    let drifted: String = got
        .lines()
        .map(|line| {
            if line.is_empty() {
                String::from("\n")
            } else {
                format!(".{line}\n")
            }
        })
        .collect();
    assert_ne!(
        drifted, want,
        "the picture did not notice a whole panel that drifted one column",
    );

    // A different faction count gives a different set of legend rows, so
    // every row below them moves up or down.
    let (fewer, _) = taken(2);
    assert_ne!(
        fewer, want,
        "the picture did not notice a panel with a different set of rows",
    );
}

#[test]
fn each_colour_of_ink_has_one_character() {
    // The picture reads a colour back as a character. Two characters for one
    // colour would make the picture depend on the order of the table, and
    // nothing would fail when the order changed.[^1]
    //
    // [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
    let key = hud::ink_key();
    for (colour, mark) in &key {
        for (other, other_mark) in &key {
            assert!(
                colour != other || mark == other_mark,
                "the colour {colour:#08x} maps to both {mark} and {other_mark}",
            );
        }
    }
    assert!(
        !key.iter().any(|(_, mark)| *mark == '.' || *mark == '?'),
        "an ink character must not be the ground or the unknown character",
    );
}
