//! The cards and the panels state what the picture holds.
//!
//! A review of the pictures found four statements that were absent or
//! wrong.[^1] [^2] Three cards covered close to half of a close-up. The row
//! that counts the units a shortage holds counted the window and sat under a
//! count of the world. The colour key named the factions and the ground and
//! no mark. The tile panel said nothing about the upgrade on the tile, and
//! the event panel said that nothing happened in the tick a contest killed
//! seven units.
//!
//! # The fixture
//!
//! Each fixture is built for its case. The fight fixture gives both sides the
//! one table row that fights and puts the pair at war, because every unit of
//! an ordinary run carries a type whose attack is zero and two cohorts then
//! stand together and nothing happens.[^2] [^3]
//!
//! # References
//!
//! [^1]: Research report 23, defects 5, 8 and 9. `docs/research/reports/23-demonstration-readability-review-1.md`
//! [^2]: Research report 25, defects 2, 3 and 7. `docs/research/reports/25-demonstration-readability-upgrades-and-units.md`
//! [^3]: Testing Rules, section 2a. `.claude/rules/testing.md`

// An integration test is its own crate, so the allowance at the viewer's
// crate root does not reach it. ADR-0067 D3 puts the float boundary at the
// viewer, and a camera is a viewer value.
#![allow(clippy::disallowed_types)]

use cachette_core::unit_type::{UnitTypeId, UnitTypeRow, WORKER_ROW};
use cachette_core::upgrade::UpgradeCategory;
use cachette_core::{Axial, FactionId, Fix32, Holder, World, WorldConfig};
use cachette_view::panel::{self, Set, View};
use cachette_view::{draw_frame, glass, Camera, Canvas, Metrics, Overlay};

/// The size of the window the tests draw into.
const WINDOW: (usize, usize) = (512, 512);

/// The first level that stands on a tile.
///
/// A site at level zero is a first build under construction, and nothing
/// stands there yet.
const FIRST_LEVEL: u8 = 1;

/// Builds a world of open ground with two factions.
fn world(width: u32) -> World {
    World::new(WorldConfig {
        width,
        height: width.clamp(1, 8),
        seed: 1,
        faction_count: 2,
        unit_capacity: 256,
    })
    .expect("the extent describes a world")
}

/// Draws one frame and returns what the readout says.
fn said(world: &World, reference: bool) -> Vec<String> {
    let mut canvas = Canvas::new(WINDOW.0, WINDOW.1);
    let camera = Camera::opening().clamped(world, &canvas);
    let readout = draw_frame(
        world,
        camera,
        &Metrics::start(),
        &[],
        Overlay::Glass { reference },
        &mut canvas,
    )
    .expect("the world draws");
    glass::says(&readout, reference)
}

/// Returns the view a panel reads, pointed at one tile.
fn view(world: &World, pointer: Option<Axial>) -> View<'_> {
    View {
        world,
        camera: Camera::opening(),
        frame_width: WINDOW.0,
        frame_height: WINDOW.1,
        focus: None,
        pointer,
    }
}

#[test]
fn the_window_draws_one_card_while_the_key_is_off() {
    // Three cards anchored to the top left covered about a tenth of a fitted
    // map and close to half of a close-up, and a watcher reads the world
    // through them.[^4]
    //
    // [^4]: Research report 23, defect 5. `docs/research/reports/23-demonstration-readability-review-1.md`
    let mut world = world(24);
    for column in 0..8 {
        let at = Axial::new(column, 1);
        if world.admits_a_unit(at) {
            world
                .spawn_soldier(at, FactionId(0))
                .expect("the tile admits a unit");
        }
    }
    world.rebuild_bridge(1).expect("the bridge rebuilds");
    for _ in 0..4 {
        world.step(1).expect("the step must run");
    }

    // The three cards the report names. The card that explains the nearest
    // unit is anchored to the far corner and covers no part of the map that
    // these three cover, so it is not counted here.
    let three = ["THE WORLD", "WHAT THEY CARRY", "THE CHARACTERS"];
    let headings = |lines: &[String]| {
        lines
            .iter()
            .filter(|line| three.contains(&line.as_str()))
            .count()
    };
    let closed = said(&world, false);
    let open = said(&world, true);
    assert_eq!(
        headings(&closed),
        1,
        "the window drew more than one card while the key was off: {closed:?}"
    );
    // The one card is shorter than the card the key holds. The row that
    // counts the units the world ended is a running total, so it sits behind
    // the key with the colours and the camera.
    assert!(
        open.iter().any(|line| line.starts_with("ended: ")),
        "the key lost the running total: {open:?}"
    );
    assert!(
        !closed.iter().any(|line| line.starts_with("ended: ")),
        "the one card kept a running total: {closed:?}"
    );
}

#[test]
fn the_row_that_counts_the_window_says_so() {
    // The row read the count of short units the drawing painted, under a row
    // labelled "in world". The two differ at every zoom.[^5]
    //
    // [^5]: Research report 23, defect 8. `docs/research/reports/23-demonstration-readability-review-1.md`
    let world = world(16);
    for reference in [false, true] {
        let lines = said(&world, reference);
        assert!(
            lines
                .iter()
                .any(|line| line.starts_with("short in window:")),
            "the card did not say which count the short row holds: {lines:?}"
        );
        assert!(
            !lines.iter().any(|line| line.starts_with("short:")),
            "the card kept the row that names no scope: {lines:?}"
        );
    }
}

#[test]
fn the_colour_key_names_every_mark_the_map_draws() {
    // A watcher who saw a white dot in a red disc had no row to look it up
    // in, and the four upgrade colours are the four that most need one,
    // because each of them is near a ground colour.[^6] [^7]
    //
    // [^6]: Research report 23, defect 9. `docs/research/reports/23-demonstration-readability-review-1.md`
    // [^7]: Research report 25, defect 7. `docs/research/reports/25-demonstration-readability-upgrades-and-units.md`
    let world = world(16);
    let lines = said(&world, true);
    for mark in [
        "shortage dot",
        "founding ring",
        "edge line",
        "luxury mark",
        "over capacity",
        "road",
        "terrace",
        "wonder",
        "store",
        "wall",
        // The map draws the category as a shape and the level as a count of
        // pips under it. A reader who is told the colours and not the count
        // has to guess what a second pip means.[^10]
        //
        // [^10]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D5. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
        "level pip",
    ] {
        assert!(
            lines
                .iter()
                .any(|line| line.starts_with(&format!("{mark}:"))),
            "the key has no row for the {mark}: {lines:?}"
        );
    }
}

#[test]
fn the_tile_panel_names_the_upgrade_under_the_pointer() {
    // One call answers the kind, the progress and the completion, and the
    // panel asked for none of them.[^8]
    //
    // [^8]: Research report 25, defect 2. `docs/research/reports/25-demonstration-readability-upgrades-and-units.md`
    let mut world = world(24);
    let place = (0..24)
        .map(|column| Axial::new(column, 1))
        .find(|at| world.admits_a_unit(*at))
        .expect("the world holds open ground");
    let builder = world
        .spawn_soldier(place, FactionId(0))
        .expect("the tile admits a unit");
    // A unit builds only on ground its own faction holds, and a faction
    // holds the ground within reach of a city it owns.[^9] The fixture
    // therefore founds a city beside the builder, and the builder stands on
    // ground its own faction holds. Standing on the tile would take no
    // ground.
    //
    // [^9]: ADR-0150, held ground is the ground within reach of a city its faction owns, decisions D1 and D4. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    let seat = (0..24)
        .flat_map(|row| (0..24).map(move |column| Axial::new(column, row)))
        .find(|at| *at != place && at.distance(place) <= 2 && world.admits_a_unit(*at))
        .expect("the world holds open ground beside the builder");
    world
        .found_settlement(seat, FactionId(0))
        .expect("the seat admits a city");
    for _ in 0..4 {
        world.step(1).expect("the step must run");
    }
    assert_eq!(
        world.tile_holder(place).and_then(Holder::faction),
        Some(FactionId(0)),
        "the builder does not hold the ground it stands on"
    );
    assert!(world.order_build(builder, UpgradeCategory::TERRACE).is_ok());
    world.step(1).expect("the step must run");
    let site = world.upgrade_at(place).expect("the order placed a site");
    assert!(!site.is_complete(), "one tick finished the terrace");

    let lines = panel::says(
        &view(&world, Some(place)),
        Set::EMPTY.with("inspector").expect("inspector registers"),
    );
    assert!(
        lines.iter().any(|line| line == "building: terrace"),
        "the panel did not name the kind: {lines:?}"
    );
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("work done: ") && line.contains(" of ")),
        "the panel did not state the progress: {lines:?}"
    );
    // **The panel names the level the map draws.** A watcher who counts the
    // pips under the mark and a watcher who reads the panel must get one
    // answer, and nothing stands on this tile yet.[^10]
    assert!(
        lines.iter().any(|line| line == "level: none yet"),
        "the panel did not say that no level stands: {lines:?}"
    );

    // The same tile once a level stands on it. The builder is put back on the
    // tile and ordered again each tick, because a unit that finished a level
    // walks away and a unit that walked away adds no work.
    let mut ticks = 0;
    while world
        .upgrade_at(place)
        .is_none_or(|site| site.level < FIRST_LEVEL)
    {
        assert!(ticks < 200, "the terrace never reached its first level");
        world
            .place_soldier(builder, place)
            .expect("the tile admits the builder");
        world.rebuild_bridge(1).expect("the bridge rebuilds");
        world
            .order_build(builder, UpgradeCategory::TERRACE)
            .expect("the engine takes the order");
        world.step(1).expect("the step must run");
        ticks += 1;
    }
    let lines = panel::says(
        &view(&world, Some(place)),
        Set::EMPTY.with("inspector").expect("inspector registers"),
    );
    assert!(
        lines.iter().any(|line| line == "level: 1"),
        "the panel did not name the level that stands: {lines:?}"
    );

    // A tile with no site holds neither row, so a watcher never reads a zero
    // for a tile nobody is building on.
    let bare = (0..24)
        .map(|column| Axial::new(column, 3))
        .find(|at| world.admits_a_unit(*at) && world.upgrade_at(*at).is_none())
        .expect("the world holds a tile with no site");
    let lines = panel::says(
        &view(&world, Some(bare)),
        Set::EMPTY.with("inspector").expect("inspector registers"),
    );
    assert!(
        !lines.iter().any(|line| line.starts_with("building: ")),
        "the panel named a site on a tile that carries none: {lines:?}"
    );
}

#[test]
fn the_event_panel_names_the_units_a_fight_felled() {
    // Sixteen units fell on two tiles and the panel said that nothing
    // happened this tick, because it read four logs and the fallen log is a
    // fifth.[^9]
    //
    // [^9]: Research report 25, defect 3. `docs/research/reports/25-demonstration-readability-upgrades-and-units.md`
    let mut world = World::new(WorldConfig {
        width: 2,
        height: 1,
        seed: 1,
        faction_count: 2,
        unit_capacity: 64,
    })
    .expect("a row of tiles is a world");
    // The one table row that fights. Every unit of an ordinary run carries a
    // type whose attack is zero, so two cohorts stand together and nothing
    // happens.
    world
        .define_unit_type(
            0,
            UnitTypeRow {
                attack: Fix32::MAX,
                armour: Fix32::ZERO,
                ..WORKER_ROW
            },
        )
        .expect("the row is inside the table");
    for (column, faction) in [(0, FactionId(0)), (1, FactionId(1))] {
        let at = Axial::new(column, 0);
        assert!(world.admits_a_unit(at), "the fixture needs open ground");
        for _ in 0..4 {
            let unit = world
                .spawn_soldier(at, faction)
                .expect("the ground admits a unit");
            assert!(
                world.set_unit_type(unit, UnitTypeId(0)),
                "the unit is alive"
            );
        }
    }
    let war = world.relation_rules().war_edge - 1;
    assert!(world.set_relation(FactionId(0), FactionId(1), war));
    world.step(1).expect("the step must run");
    assert!(
        !world.fell_log().is_empty(),
        "the contest felled nobody, so the fixture supplies no case"
    );

    let lines = panel::says(
        &view(&world, None),
        Set::EMPTY.with("events").expect("events registers"),
    );
    assert!(
        lines.iter().any(|line| line == "UNITS FELL"),
        "the panel has no section for the units that fell: {lines:?}"
    );
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("fell this tick: ")),
        "the panel did not count the units that fell: {lines:?}"
    );
    assert!(
        !lines
            .iter()
            .any(|line| line == "nothing happened this tick"),
        "the panel said that nothing happened in the tick of a fight: {lines:?}"
    );
}
