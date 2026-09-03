//! The window shows what changes, and names its colours when asked.
//!
//! The panel grew to thirteen sections and became longer than the window. It
//! cut at the foot, and a watcher could not reach the rows below the
//! notice.[^1] The window now draws cards, which hold what changes moment to
//! moment, and the whole panel goes to a rendered picture that no window
//! height cuts.[^2]
//!
//! These tests read what the cards say, through the same list the painting
//! walks. A test that read the pixels would pin the layout rather than the
//! content, and a test that read a private field would pin the
//! implementation.[^3]
//!
//! # What the product record asks
//!
//! The record asks that the window name every colour it draws, state where the
//! person is looking, state the cost of the step and of the drawing as two
//! numbers, and state how many units the world holds beside how many it
//! shows.[^4] The cards keep the last of those always, and the other three
//! appear while a watcher holds a key. A legend on a key is the window naming
//! its colours.[^2]
//!
//! # References
//!
//! [^1]: Backlog item 0133. `docs/backlog/complete/0133-let-a-watcher-reach-a-panel-longer-than-the-window.md`
//! [^2]: Decisions register, DEC-084. `docs/DECISIONS.md`
//! [^3]: Testing rules, section 6. `.claude/rules/testing.md`
//! [^4]: PRD-0005, a watcher can tell what is happening and why. `docs/product/shipped/prd-0005-a-watcher-can-tell-what-is-happening-and-why.md`

// An integration test is its own crate, so the allowance at the viewer's
// crate root does not reach it. ADR-0067 D3 puts the float boundary at the
// viewer, and a camera is a viewer value.
#![allow(clippy::disallowed_types)]

use cachette_core::resource::ResourceKind;
use cachette_core::terrain::TileKind;
use cachette_core::{Axial, FactionId, World, WorldConfig};
use cachette_view::{draw_frame, glass, paint, Camera, Canvas, Metrics, Overlay, Readout};

/// The size of the window these tests draw into.
///
/// It matches no window in particular. The cards fit any window, and one test
/// below checks that at three sizes rather than at this one.
const WINDOW: (usize, usize) = (960, 720);

/// The factions the fixture world holds.
///
/// The colour table is larger than this, so a legend sized by the table would
/// name two colours that no faction uses. One test below reads that.
const FACTIONS: u16 = 4;

/// The people the fixture founds its run with.
const GROUP: u32 = 30;

/// Builds a founded world and steps it, so the cards have something to say.
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
    (world, outcomes, place)
}

/// Draws one frame of the fixture and returns the readout and the canvas.
fn drawn(reference: bool) -> (World, Readout, Canvas) {
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
        Overlay::Glass { reference },
        &mut canvas,
    )
    .expect("the world draws");
    (world, readout, canvas)
}

/// Returns the rows of one card, by its heading.
///
/// A heading is a line with no colon in it. The rows of a card are the lines
/// between its heading and the next one. The unit card names a faction too, so
/// a test of the legend must read the legend and not every line of the glass.
fn rows_under(said: &[String], heading: &str) -> Vec<String> {
    said.iter()
        .skip_while(|line| line.as_str() != heading)
        .skip(1)
        .take_while(|line| line.contains(": "))
        .cloned()
        .collect()
}

/// Returns the value of one row of the glass, by its label.
fn value_of(said: &[String], label: &str) -> Option<String> {
    let head = format!("{label}: ");
    said.iter()
        .find(|line| line.starts_with(&head))
        .map(|line| line[head.len()..].to_string())
}

#[test]
fn the_glass_states_both_population_counts_and_labels_them() {
    // The product record asks for both numbers, labelled, so that a reader
    // cannot mistake one for the other.[^1] The world count is always on the
    // glass and the window count sits behind the key, so the label is the only
    // thing that separates them and both must name their scope.
    //
    // [^1]: PRD-0005, a watcher can tell what is happening and why. `docs/product/shipped/prd-0005-a-watcher-can-tell-what-is-happening-and-why.md`
    let (_, readout, _) = drawn(false);
    let said = glass::says(&readout, false);

    let world = value_of(&said, "people in world").expect("the glass counts the world");
    let window = value_of(&glass::says(&readout, true), "people in window")
        .expect("the key counts the window");
    assert_eq!(
        world,
        cachette_view::hud::grouped(u64::from(readout.soldiers_live()))
    );
    assert_eq!(
        window,
        cachette_view::hud::grouped(u64::from(readout.soldiers_painted()))
    );
    // The fixture asserts its own outcome. Two equal counts would let a card
    // that read one number into both rows pass.
    assert_ne!(
        readout.soldiers_live(),
        readout.soldiers_painted(),
        "the fixture must hold units outside the window, or the two rows agree by accident"
    );
}

#[test]
fn the_glass_states_the_tick_the_engine_reached() {
    let (_, readout, _) = drawn(false);
    let said = glass::says(&readout, false);
    assert_eq!(
        value_of(&said, "tick"),
        Some(cachette_view::hud::grouped(readout.tick())),
    );
    assert!(readout.tick() > 0, "the fixture must step the world");
}

#[test]
fn the_glass_states_what_is_left_of_the_food_the_ground_gave() {
    // A world in which nobody gathered holds the two numbers equal, so a card
    // that printed the generated stock in both would pass. This gathers
    // first.[^1]
    //
    // [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
    let mut world = World::new(WorldConfig {
        width: 48,
        height: 48,
        seed: 0x0cac_f00d,
        faction_count: 1,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent describes a world");
    world.rebuild_bridge(1).expect("the rebuild must succeed");
    let grid = world.grid();
    let deposit = (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .find(|address| {
            world.admits_a_unit(*address)
                && world
                    .tile_stock(*address, ResourceKind::Food)
                    .is_some_and(|amount| amount.0 >= 2)
        })
        .expect("the fixture world holds a tile that carries food");
    let unit = world
        .spawn_soldier(deposit, FactionId(0))
        .expect("the tile admits a unit");
    assert!(
        world.order_gather(unit, ResourceKind::Food),
        "the engine must accept the order"
    );
    world.step(1).expect("the step must run");

    let mut canvas = Canvas::new(WINDOW.0, WINDOW.1);
    let camera = Camera::opening()
        .looking_at(deposit, &canvas)
        .clamped(&world, &canvas);
    let readout = draw_frame(
        &world,
        camera,
        &Metrics::start(),
        &[],
        Overlay::Glass { reference: false },
        &mut canvas,
    )
    .expect("the world draws");

    let left = world
        .tile_stock(deposit, ResourceKind::Food)
        .expect("the address names a tile")
        .0;
    let gave = world
        .original_stock(deposit, ResourceKind::Food)
        .expect("the address names a tile")
        .0;
    // The fixture asserts its own outcome. A step in which nobody gathered
    // leaves the two equal, and the assertion below then holds for a card
    // that read the generated stock twice.
    assert!(
        left < gave,
        "the gather must separate the two numbers: {left} of {gave}"
    );
    assert_eq!(
        value_of(&glass::says(&readout, false), "food here"),
        Some(format!("{left} of {gave}")),
        "the glass must state what the engine holds"
    );
}

#[test]
fn the_glass_states_the_option_the_engine_selected() {
    let (world, readout, _) = drawn(false);
    let said = glass::says(&readout, false);
    let choice = readout.choice().expect("the window holds units");
    let answer = world
        .explain_choice(choice.focus().entity())
        .expect("the engine explains a live unit");

    let chose = value_of(&said, "chose").expect("the glass names what the unit chose");
    let named = answer.best_name().expect("the engine selected an option");
    assert!(
        chose.starts_with(named),
        "the glass says {chose:?} and the engine selected {named:?}"
    );
    // The score rides in the same row, so the row must hold it too. A card
    // that named the option and dropped the score would pass a test that only
    // compared the name.
    assert!(
        chose.len() > named.len(),
        "the glass names the option and not its score: {chose:?}"
    );
}

#[test]
fn the_glass_names_no_colour_while_the_key_is_up() {
    // The colours are the reference layer. A card that drew them always would
    // spend the glass on something a watcher checks occasionally.
    let (_, readout, _) = drawn(false);
    let said = glass::says(&readout, false);
    assert!(
        !said.iter().any(|line| line == "COLOURS IN THE WINDOW"),
        "the glass draws the legend with the key up: {said:?}"
    );
    assert!(
        rows_under(&said, "COLOURS IN THE WINDOW").is_empty(),
        "the glass names a colour with the key up: {said:?}"
    );
    assert!(
        !said.iter().any(|line| line.starts_with("water: ")),
        "the glass names a ground with the key up: {said:?}"
    );
    // The key up must still leave the cards that hold what changes.
    assert!(
        said.iter().any(|line| line == "THE WORLD"),
        "the key hid a card that holds what changes: {said:?}"
    );
}

#[test]
fn the_key_names_every_colour_the_window_draws() {
    // The product record asks that the window name every colour it draws.[^1]
    // This is that statement, checked colour by colour.
    //
    // [^1]: PRD-0005, a watcher can tell what is happening and why. `docs/product/shipped/prd-0005-a-watcher-can-tell-what-is-happening-and-why.md`
    let (_, readout, _) = drawn(true);
    let said = glass::says(&readout, true);

    let legend = rows_under(&said, "COLOURS IN THE WINDOW");
    for faction in 0..readout.factions() {
        assert!(
            legend
                .iter()
                .any(|line| line.starts_with(&format!("faction {faction}: "))),
            "the key does not name faction {faction}: {legend:?}"
        );
    }
    for kind in [
        TileKind::Water,
        TileKind::Plain,
        TileKind::Forest,
        TileKind::Hill,
        TileKind::Mountain,
    ] {
        let name = cachette_view::hud::name_of(kind);
        assert!(
            legend
                .iter()
                .any(|line| line.starts_with(&format!("{name}: "))),
            "the key does not name the ground {name}: {legend:?}"
        );
    }
}

#[test]
fn the_legend_names_the_factions_the_world_holds_and_no_more() {
    // The colour table is larger than most worlds need. A legend sized by the
    // table names a colour that no faction uses, and a reader then looks for a
    // faction that is not there.
    let (_, readout, _) = drawn(true);
    let said = glass::says(&readout, true);
    let legend = rows_under(&said, "COLOURS IN THE WINDOW");
    let named = legend
        .iter()
        .filter(|line| line.starts_with("faction "))
        .count();
    assert_eq!(
        named, FACTIONS as usize,
        "the legend names {named} factions in a world of {FACTIONS}: {legend:?}"
    );
}

#[test]
fn the_key_states_where_you_are_looking_and_what_the_frame_cost() {
    // Two more statements of the product record. Both sit behind the same key
    // as the colours, because a watcher checks all three occasionally rather
    // than continuously.
    let (_, readout, _) = drawn(true);
    let said = glass::says(&readout, true);
    for label in [
        "centre tile",
        "showing",
        "step",
        "draw",
        "people in window",
        "ground here",
    ] {
        assert!(
            value_of(&said, label).is_some(),
            "the key does not state {label}: {said:?}"
        );
    }
    assert_eq!(
        value_of(&said, "centre tile"),
        Some(format!(
            "q {}  r {}",
            readout.centre().q,
            readout.centre().r
        )),
    );
    // The two cost numbers are separate, which the record asks for.
    assert_ne!(
        value_of(&said, "step"),
        None,
        "the cost card must state the step"
    );
}

#[test]
fn every_card_sits_inside_the_window() {
    // The panel states a rectangle that a short window cuts, and says so on
    // its last line. A card is sized by its content and placed against a
    // corner, so it has no such case. This checks that at three sizes and
    // with the key up and down.
    let (world, outcomes, place) = founded();
    for size in [(960usize, 720usize), (640, 480), (420, 360)] {
        for reference in [false, true] {
            let mut canvas = Canvas::new(size.0, size.1);
            let camera = Camera::opening()
                .looking_at(place, &canvas)
                .clamped(&world, &canvas);
            let readout = draw_frame(
                &world,
                camera,
                &Metrics::start(),
                &outcomes,
                Overlay::Glass { reference },
                &mut canvas,
            )
            .expect("the world draws");

            let cards = glass::card_bounds(&readout, reference, &canvas);
            assert!(!cards.is_empty(), "the glass drew no card at {size:?}");
            for (x, y, width, height) in cards {
                assert!(
                    x >= 0 && y >= 0,
                    "a card starts at {x}, {y} in a window of {size:?}"
                );
                assert!(
                    x + width <= size.0 as i32 && y + height <= size.1 as i32,
                    "a card of {width} by {height} at {x}, {y} runs past a window of {size:?}"
                );
            }
        }
    }
}

#[test]
fn the_glass_covers_less_of_the_window_than_the_panel() {
    // The map gets the window. This is the number that says so: the glass
    // writes over fewer pixels than the panel does.
    let (world, outcomes, place) = founded();
    let mut bare = Canvas::new(WINDOW.0, WINDOW.1);
    let camera = Camera::opening()
        .looking_at(place, &bare)
        .clamped(&world, &bare);
    paint::draw(&world, camera, &mut bare).expect("the world draws");
    paint::mark_foundings(camera, &mut bare, &outcomes);

    let ink = |overlay: Overlay| {
        let mut canvas = Canvas::new(WINDOW.0, WINDOW.1);
        draw_frame(
            &world,
            camera,
            &Metrics::start(),
            &outcomes,
            overlay,
            &mut canvas,
        )
        .expect("the world draws");
        canvas
            .pixels()
            .iter()
            .zip(bare.pixels())
            .filter(|(a, b)| a != b)
            .count()
    };

    let cards = ink(Overlay::Glass { reference: false });
    let panel = ink(Overlay::Panel);
    assert!(
        cards < panel,
        "the glass covered {cards} pixels and the panel {panel}"
    );
    let whole = WINDOW.0 * WINDOW.1;
    assert!(
        cards * 100 / whole < 15,
        "the glass covered {cards} of {whole} pixels, which is not a heads-up display"
    );
}

#[test]
fn the_command_the_window_names_exists() {
    // The window tells a watcher where the rest of the numbers are, and it
    // does that by naming a command. The name in the window and the recipe in
    // the build file are one fact in two places, and nothing fails when they
    // disagree.[^1] This is the check that fails.
    //
    // The name was invented in a design conversation before the recipe
    // existed, and a person was shown it. That is what this test prevents from
    // happening again.[^2]
    //
    // [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
    // [^2]: Findings register, FND-199. `docs/FINDINGS.md`
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    let build = std::fs::read_to_string(root.join("justfile")).expect("the build file must open");

    // A recipe line is the name, then either an argument or the colon. A test
    // that only asked whether the line starts with the name passes for a
    // recipe named `inspect-the-panel`, which is a different command. That
    // false pass was found by renaming the recipe and watching the test stay
    // green.[^3]
    //
    // [^3]: Testing rules, section 2a. `.claude/rules/testing.md`
    let name = glass::DETAIL_COMMAND;
    let names_it =
        |line: &str| line.starts_with(&format!("{name} ")) || line.starts_with(&format!("{name}:"));
    assert!(
        build.lines().any(names_it),
        "the build file has no recipe named {name}, which the window tells a watcher to run"
    );
    for reference in [false, true] {
        assert!(
            glass::hint_line(reference).contains(name),
            "the hint line does not name the command: {}",
            glass::hint_line(reference)
        );
    }
}

#[test]
fn a_cost_the_run_has_not_measured_is_absent_and_not_zero() {
    // **A drawing cannot measure itself.** The window states the cost of the
    // drawing, and the run records that cost after the drawing has ended. The
    // card is drawn inside the frame it cannot yet have measured, so the mean
    // it states covers the frames before it. A picture written by one call to
    // the drawing has no frame before it.
    //
    // The card said `0.0 ms` in that case, which reads as a measurement of a
    // free drawing. It was a mean over nothing. The record forbids the window
    // from stating a number it does not have.[^5] Every stored picture of this
    // window carried that row, so the one instrument the project had was
    // saying zero in every picture anyone looked at.[^6]
    //
    // [^5]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    // [^6]: Findings register, FND-208. `docs/FINDINGS.md`
    let (_, readout, _) = drawn(true);
    assert_eq!(
        readout.frames_measured(),
        0,
        "this test needs a run that has recorded no drawing, and it has one",
    );
    let said = glass::says(&readout, true);
    let drawing = value_of(&said, "draw").expect("the cost card states the drawing");
    assert!(
        !drawing.contains("0.0"),
        "the window states {drawing} for a drawing that nobody measured",
    );
    assert_eq!(drawing, "not measured yet");

    // A run that has measured states the figure. A card that said the same
    // words whatever the run did would pass the assertion above and report
    // nothing.
    let mut metrics = Metrics::start();
    metrics.draw(std::time::Duration::from_micros(2500));
    metrics.step(std::time::Duration::from_micros(1500));
    let (world, outcomes, place) = founded();
    let mut canvas = Canvas::new(WINDOW.0, WINDOW.1);
    let camera = Camera::opening()
        .looking_at(place, &canvas)
        .clamped(&world, &canvas);
    let readout = draw_frame(
        &world,
        camera,
        &metrics,
        &outcomes,
        Overlay::Glass { reference: true },
        &mut canvas,
    )
    .expect("the world draws");
    let said = glass::says(&readout, true);
    assert_eq!(value_of(&said, "draw"), Some("2.5 ms".to_string()));
    assert_eq!(value_of(&said, "step"), Some("1.5 ms".to_string()));
}
