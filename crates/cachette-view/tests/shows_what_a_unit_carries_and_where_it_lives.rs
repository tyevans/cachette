//! The viewer shows the load a unit carries and the home site it keeps.
//!
//! Both facts were already true of every run and no watcher could see either.
//! Every unit of the demonstration world holds a home site, a share of them
//! haul a load, and the only way to learn it was to read the log.[^1]
//!
//! Every count here is a count of the window. The drawing asks at each unit it
//! paints, on the loop that already runs, so no layer starts a pass over the
//! arena.[^2] A layer that swept the arena would report the same totals and
//! cost the population, so the reads are what the tests check, not the
//! totals.[^3]
//!
//! # What the fixture supplies
//!
//! A world in which nothing is carried supplies no case, and an assertion
//! about a load then measures the fixture.[^4] The fixture below asserts its
//! own outcome: it reads the loads back out of the engine and refuses a world
//! in which no unit carries anything.
//!
//! # References
//!
//! [^1]: Backlog item 0274. `docs/backlog/complete/0274-show-the-load-a-unit-carries-and-the-home-it-keeps.md`
//! [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
//! [^3]: Findings register, FND-071. `docs/FINDINGS.md`
//! [^4]: Testing rules, section 2a. `.claude/rules/testing.md`

// An integration test is its own crate, so the allowance at the viewer's
// crate root does not reach it. ADR-0067 D3 puts the float boundary at the
// viewer, and a camera is a viewer value.
#![allow(clippy::disallowed_types)]

use cachette_core::founding::FoundingOutcome;
use cachette_core::resource::ResourceKind;
use cachette_core::{Axial, World, WorldConfig};
use cachette_view::{draw_frame, glass, hud, Camera, Canvas, Metrics, Overlay, Readout};

/// The size of the window the tests paint onto.
const WINDOW: (usize, usize) = (512, 512);

/// The number of people each faction founds with.
///
/// **The size is what makes the fixture supply the case.** The founding sets
/// the production rate of a site to a sixteenth of the food its survey
/// reached, and a person draws a ration of a sixteenth, so a site feeds
/// exactly as many people as the food its survey measured. A group the ground
/// carries is never short, never forages, never gathers and never carries. At
/// the extent below a group of 48 leaves every unit fed and nothing is ever
/// hauled, and every assertion here then measures the fixture.[^1]
///
/// # References
///
/// [^1]: Backlog item 0240, let the demonstration make a unit hungry. `docs/backlog/complete/0240-let-the-demonstration-make-a-unit-hungry.md`
const GROUP: u32 = 96;

/// The number of steps the fixture runs before it draws.
///
/// A unit gathers before it carries, so a world drawn at the first tick holds
/// no load whatever the engine does. This count is the number that makes the
/// fixture supply the case, and the fixture asserts that it did.
const STEPS: usize = 120;

/// The extent of the fixture world.
///
/// A smaller world seats its groups on ground that carries them, and then
/// nothing is ever short and nothing is ever carried.
const EXTENT: u32 = 128;

/// Builds a founded world, run far enough that units carry something.
fn founded(extent: u32, steps: usize) -> (World, Vec<FoundingOutcome>, Axial) {
    let mut world = World::new(WorldConfig {
        width: extent,
        height: extent,
        seed: 0x0cac_4e77_e5ee_d001,
        faction_count: 3,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent describes a world");
    let outcomes = world.found_run_for_every_faction(GROUP);
    let place = outcomes
        .iter()
        .find_map(|outcome| outcome.founding().map(cachette_core::Founding::place))
        .expect("the world holds a place for a group");
    world.rebuild_bridge(1).expect("the rebuild must succeed");
    for _ in 0..steps {
        world.step(1).expect("the step must run");
    }
    (world, outcomes, place)
}

/// Returns how many live units carry anything, read from the engine.
///
/// This is the fixture checking itself. It sweeps the arena, which is exactly
/// what the drawing must not do, and that is why it lives in the test and not
/// in the viewer.
fn carrying_in_world(world: &World) -> u32 {
    let arena = world.soldiers();
    arena
        .iter()
        .filter(|entity| {
            arena
                .carry(*entity)
                .is_some_and(|load| ResourceKind::ALL.iter().any(|kind| load.of(*kind).0 > 0))
        })
        .count() as u32
}

/// Draws one frame of a fixture and returns the world, readout and canvas.
fn drawn(extent: u32, steps: usize) -> (World, Readout, Canvas<'static>) {
    let (world, outcomes, place) = founded(extent, steps);
    let mut canvas = Canvas::new(WINDOW.0, WINDOW.1);
    let camera = Camera::opening()
        .looking_at(place, &canvas)
        .clamped(&world, &canvas);
    let readout = draw_frame(
        &world,
        camera,
        &Metrics::start(),
        &outcomes,
        Overlay::Glass { reference: false },
        &mut canvas,
    )
    .expect("the world draws");
    (world, readout, canvas)
}

#[test]
fn the_fixture_holds_a_world_in_which_something_is_carried() {
    // This test is the fixture's own check. Every assertion below about a
    // load would pass on a world where nothing carries anything, because the
    // count would be zero on both sides. This one fails instead.
    let (world, _, _) = drawn(EXTENT, STEPS);

    let carried = carrying_in_world(&world);
    assert!(
        carried > 0,
        "no unit carries anything after {STEPS} steps, so every load assertion here \
         measures the fixture rather than the viewer",
    );
    let arena = world.soldiers();
    let housed = arena
        .iter()
        .filter(|entity| matches!(arena.home(*entity), Some(Some(_))))
        .count();
    assert!(
        housed > 0,
        "no unit holds a home site, so the home count measures the fixture",
    );
}

#[test]
fn the_drawing_asks_every_unit_it_paints_and_no_other() {
    // The count of reads is a function of the window. A layer that swept the
    // arena would report the same totals and cost the population, so this is
    // the assertion that separates the two.
    let (_, _, canvas) = drawn(EXTENT, STEPS);

    assert!(canvas.soldiers_painted() > 0, "the fixture painted no unit");
    assert_eq!(
        canvas.carry_reads(),
        canvas.soldiers_painted(),
        "the drawing asked for a load somewhere other than at a painted unit",
    );
    assert_eq!(
        canvas.home_reads(),
        canvas.soldiers_painted(),
        "the drawing asked for a home somewhere other than at a painted unit",
    );
}

#[test]
fn the_cost_of_the_loads_follows_the_window_and_not_the_world() {
    // The same camera over a larger world reads the same number of loads. A
    // drawing that swept the arena would paint the same picture, so the count
    // is what proves it did not.
    let (_, _, small) = drawn(EXTENT, STEPS);
    let (_, _, large) = drawn(EXTENT * 2, STEPS);

    assert!(small.carry_reads() > 0, "the fixture read no load");
    assert_eq!(
        small.carry_reads(),
        small.soldiers_painted(),
        "the small world read a load away from a painted unit",
    );
    assert_eq!(
        large.carry_reads(),
        large.soldiers_painted(),
        "the large world read a load away from a painted unit",
    );
}

#[test]
fn a_painted_unit_that_carries_is_counted_and_never_more_than_were_painted() {
    let (world, readout, canvas) = drawn(EXTENT, STEPS);

    // The window count can only be at or under the world count, because the
    // window holds a part of the world.
    assert!(
        readout.units_carrying() <= carrying_in_world(&world),
        "the window counted more carriers than the world holds",
    );
    assert!(
        readout.units_carrying() <= canvas.soldiers_painted(),
        "the window counted more carriers than it painted units",
    );
    assert!(
        readout.units_housed() <= canvas.soldiers_painted(),
        "the window counted more homes than it painted units",
    );
    // Something is carried in the window, so the totals are not all zero.
    let carried: u32 = readout.carried_by_kind().iter().sum();
    assert!(
        carried > 0,
        "the window painted carriers and reported nothing carried",
    );
}

#[test]
fn the_card_appears_only_when_something_is_carried() {
    // A card that said "carrying 0" on every frame of every run would take
    // space from the map to report nothing, and a watcher would learn to skip
    // it. The count is what decides, so the card cannot outlive the behaviour
    // it reports.
    let (_, early, _) = drawn(EXTENT, 0);
    let (_, later, _) = drawn(EXTENT, STEPS);

    assert_eq!(
        early.units_carrying(),
        0,
        "the fixture carried something at the first tick, so this test cannot \
         tell the empty case from the full one",
    );
    assert!(later.units_carrying() > 0, "the fixture carried nothing");

    let heading = "WHAT THEY CARRY";
    assert!(
        !glass::says(&early, false)
            .iter()
            .any(|line| line == heading),
        "the glass drew the load card when nothing was carried",
    );
    assert!(
        glass::says(&later, false)
            .iter()
            .any(|line| line == heading),
        "the glass hid the load card when something was carried",
    );
}

#[test]
fn the_card_states_the_load_the_readout_holds() {
    // The card and the panel read one readout, so no number on the glass can
    // disagree with the same number in the picture.
    let (_, readout, canvas) = drawn(EXTENT, STEPS);
    let said = glass::says(&readout, false);

    let carrying = said
        .iter()
        .find_map(|line| line.strip_prefix("carrying: "))
        .expect("the card states how many carry");
    assert_eq!(
        carrying,
        format!(
            "{} of {}",
            readout.units_carrying(),
            canvas.soldiers_painted()
        ),
        "the card and the readout disagree about how many units carry",
    );
}

#[test]
fn the_home_count_does_not_take_a_place_in_the_window() {
    // A quantity earns a place in the window only if it changes moment to
    // moment, and the test is what the quantity does rather than how
    // interesting it is.[^1] Every unit founds holding a home site and keeps
    // it, so this count reads `n of n` from the first frame and stays there.
    // It belongs in the panel, which a watcher opens when they want it.
    //
    // The panel still states it, and the next test checks that it does. A
    // quantity that is on neither surface is not restraint, it is a loss.
    //
    // [^1]: ADR-0093, the window shows what changes, decision D1.
    //       `docs/adrs/draft/adr-0093-the-window-shows-what-changes.md`
    let (_, readout, _) = drawn(EXTENT, STEPS);
    assert!(
        readout.units_housed() > 0,
        "the fixture houses nobody, so this test would pass on an empty set",
    );
    assert!(
        !glass::says(&readout, false)
            .iter()
            .any(|line| line.starts_with("with a home")),
        "the glass drew a count that never moves",
    );
}

#[test]
fn the_panel_states_the_home_count_the_readout_holds() {
    // The card and the panel read one readout, so no number on one surface
    // can disagree with the same number on the other.
    let (_, readout, canvas) = drawn(EXTENT, STEPS);
    let expected = format!(
        "{} of {}",
        readout.units_housed(),
        canvas.soldiers_painted()
    );
    assert!(
        hud::says(&readout)
            .iter()
            .any(|line| line.contains(&expected)),
        "the panel does not state the home count the readout holds: {expected}",
    );
}

#[test]
fn the_rationing_count_is_the_log_the_step_wrote() {
    // The store of a site rations when it cannot serve what its cohorts asked
    // for. This is a count of the world, because the engine holds the log of
    // the step that just ran, and the row that shows it says so.
    let (world, readout, _) = drawn(EXTENT, STEPS);

    assert_eq!(
        readout.rationings() as usize,
        world.rationed_log().len(),
        "the readout and the engine disagree about how many sites rationed",
    );
    // A shortfall is never negative, because a rationed draw granted less
    // than it was asked for. The value is in accumulator units.
    if readout.rationings() > 0 {
        assert!(
            readout.rationed_short() > 0,
            "a site rationed and the shortfall came out as zero",
        );
    }
}
