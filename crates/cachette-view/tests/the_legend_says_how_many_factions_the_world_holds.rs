//! A watcher can tell a whole faction list from a short one.
//!
//! The viewer holds six faction colours and the engine holds up to sixty-three
//! factions. A faction past the colour table shares a colour with an earlier
//! one, so the legend draws six rows and each count is the sum of every
//! faction of that colour.
//!
//! Both surfaces once drew those six rows and said nothing else. A watcher of
//! a world of ten factions read six rows, could not tell that four were
//! missing, and could not tell the count of one faction from the sum of two.
//!
//! # The fixture
//!
//! The fixture holds ten factions on purpose. Every other viewer test holds
//! four or six, which is at or below the colour table, so the clamp never
//! fires and the assertion never receives the case it exists for.[^1] This
//! world seats factions above the table and asserts that it did.
//!
//! # References
//!
//! [^1]: Testing Rules, section 2a. `.claude/rules/testing.md`

// An integration test is its own crate, so the allowance at the viewer's
// crate root does not reach it. ADR-0067 D3 puts the float boundary at the
// viewer, and a camera is a viewer value.
#![allow(clippy::disallowed_types)]

use cachette_core::{Axial, World, WorldConfig};
use cachette_view::paint::COLOURED_FACTIONS;
use cachette_view::{draw_frame, glass, hud, Camera, Canvas, Metrics, Overlay, Readout};

/// The size of the window the test draws into.
const WINDOW: (usize, usize) = (960, 720);

/// The factions the fixture world holds.
///
/// The number is above the colour table, so the legend cannot draw one row
/// for each faction and cannot give one count for each faction.
const FACTIONS: u16 = 10;

/// The people the fixture founds its run with.
const GROUP: u32 = 30;

/// What the card and the panel both say when the factions pass the table.
const COUNT_LABEL: &str = "factions in the world";

/// Builds a world of ten factions and steps it.
fn founded() -> (World, Vec<cachette_core::FoundingOutcome>, Axial) {
    let mut world = World::new(WorldConfig {
        width: 200,
        height: 140,
        seed: 0x0cac_4e77_e5ee_d001,
        faction_count: FACTIONS,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent describes a world");
    let outcomes = world.found_run_for_every_faction(GROUP);
    let place = outcomes
        .iter()
        .find_map(|outcome| outcome.founding().map(cachette_core::Founding::place))
        .expect("the world holds a place for a group");
    world.rebuild_bridge(1).expect("the rebuild must succeed");
    for _ in 0..8 {
        world.step(1).expect("the step must run");
    }

    // The fixture must produce a faction the colour table cannot name. A
    // world that seated six or fewer would leave the assertions below green
    // with the whole repair removed.
    let population = world.population_by_faction();
    let beyond = (COLOURED_FACTIONS..usize::from(FACTIONS))
        .filter(|faction| population[*faction] > 0)
        .count();
    assert!(
        beyond > 0,
        "no faction above the colour table holds a unit, so the legend never \
         reaches the case it must report",
    );
    (world, outcomes, place)
}

/// Draws one frame of the fixture and returns the readout and the canvas.
fn drawn_on() -> (Readout, Canvas<'static>) {
    let (world, outcomes, place) = founded();
    let mut canvas = Canvas::new(WINDOW.0, WINDOW.1);
    let camera = Camera::opening()
        .looking_at(place, &canvas)
        .clamped(&world, &canvas);
    let readout = draw_frame(
        &world,
        camera,
        &Metrics::start(),
        &outcomes,
        Overlay::Glass { reference: true },
        &mut canvas,
    )
    .expect("the world draws");
    (readout, canvas)
}

/// Draws one frame of the fixture and returns the readout.
fn drawn() -> Readout {
    drawn_on().0
}

/// Returns the rows of one card of the glass, by its heading.
fn rows_under(said: &[String], heading: &str) -> Vec<String> {
    said.iter()
        .skip_while(|line| line.as_str() != heading)
        .skip(1)
        .take_while(|line| line.contains(": "))
        .cloned()
        .collect()
}

#[test]
fn the_glass_says_how_many_factions_the_world_holds() {
    let readout = drawn();
    let said = glass::says(&readout, true);
    let legend = rows_under(&said, "COLOURS IN THE WINDOW");

    let named = legend
        .iter()
        .filter(|line| line.starts_with("faction "))
        .count();
    assert!(
        named < usize::from(readout.factions()),
        "the card names one row for each faction, so this test measures \
         nothing: {legend:?}",
    );
    assert!(
        legend
            .iter()
            .any(|line| line == &format!("{COUNT_LABEL}: {}", readout.factions())),
        "the card shows {named} faction rows of {} factions and never says \
         how many the world holds: {legend:?}",
        readout.factions(),
    );
}

#[test]
fn the_glass_says_that_a_row_counts_more_than_one_faction() {
    let readout = drawn();
    let said = glass::says(&readout, true);
    let legend = rows_under(&said, "COLOURS IN THE WINDOW");

    assert!(
        legend
            .iter()
            .any(|line| line == "each faction row counts: every faction of its colour"),
        "the card gives a count for a shared colour and calls it the count of \
         one faction: {legend:?}",
    );
}

#[test]
fn the_panel_says_how_many_factions_the_world_holds() {
    let readout = drawn();
    let said = hud::says(&readout);

    assert!(
        said.iter()
            .any(|line| line == &format!("{COUNT_LABEL}: {}", readout.factions())),
        "the panel draws six faction rows and never says that the world holds \
         {}: {said:?}",
        readout.factions(),
    );
    assert!(
        said.iter()
            .any(|line| line == "each row above counts every"),
        "the panel gives a count for a shared colour and calls it the count \
         of one faction: {said:?}",
    );
}

#[test]
fn the_lines_that_report_the_shared_colours_fit_their_surfaces() {
    // A line the panel cuts states something other than what it was given,
    // and it does so in silence. A statement that repairs a silent defect
    // must not arrive as a second one.
    let (readout, canvas) = drawn_on();

    // The reading names every line the panel would cut. This test owns the
    // lines that report the shared colours, and it asserts about those alone.
    // Other lines of the panel are the business of the panel width test.
    let cut = hud::lines_that_do_not_fit(&readout);
    let mine: Vec<&String> = cut
        .iter()
        .filter(|line| line.contains("colour") || line.starts_with(COUNT_LABEL))
        .collect();
    assert!(
        mine.is_empty(),
        "the panel cuts a line that reports the shared colours: {mine:?}",
    );

    for (x, y, width, height) in glass::card_bounds(&readout, true, &canvas) {
        assert!(
            x >= 0 && y >= 0,
            "a card starts outside the window at ({x}, {y})",
        );
        assert!(
            x + width <= WINDOW.0 as i32 && y + height <= WINDOW.1 as i32,
            "a card of {width} by {height} at ({x}, {y}) runs past the window",
        );
    }
}

#[test]
fn a_legend_row_holds_every_unit_of_its_colour() {
    // The legend is short, and it drops nothing. Each row is the sum of the
    // factions that share its colour, so the rows together must hold every
    // unit the pass painted. A row that truncated instead of summing would
    // lose the units of the factions above the table, and the note this test
    // reads elsewhere would then be false.
    let readout = drawn();
    let drawn_units: u32 = readout.by_faction().iter().sum();
    assert_eq!(
        drawn_units,
        readout.soldiers_painted(),
        "the legend rows hold {drawn_units} units of the {} the pass painted",
        readout.soldiers_painted(),
    );
}
