//! A watcher can see a shortage spread through a group, and can see how many
//! units it has ended.
//!
//! A finding records the failure these tests exist to prevent: the word
//! "watcher" covers two interfaces, the library and the window, and an item
//! can satisfy its whole list against the first while the second shows
//! nothing.[^1] Every test here reads the pixels of the window or the lines
//! of the panel. None of them is satisfied by a value that the library
//! returns.
//!
//! # The fixture
//!
//! The fixture is built to starve some units and to feed others. It is never
//! copied from the demonstration world, which is chosen to look right and
//! supplies no extreme.[^2] Half the sites produce nothing and hold a store
//! that empties. The other half produce more than their people eat.
//!
//! Each test asserts what the frame reported, and the fixture asserts its own
//! outcome by reading the conditions back before it draws.[^3]
//!
//! # References
//!
//! [^1]: Findings register, FND-100. `docs/FINDINGS.md`
//! [^2]: Testing Rules, a fixture supplies the input. `.claude/rules/testing.md`
//! [^3]: Findings register, FND-061. `docs/FINDINGS.md`

// An integration test is its own crate, so the allowance at the viewer's
// crate root does not reach it. ADR-0067 D3 puts the float boundary at the
// viewer, and a camera is a viewer value.
#![allow(clippy::disallowed_types)]

use std::time::Duration;

use cachette_core::cohort::{NeedCondition, NeedRule, NEED_FULL};
use cachette_core::site::CommodityId;
use cachette_core::{Axial, Entity, FactionId, Fix32, World, WorldConfig};
use cachette_view::{draw_frame, paint, Camera, Canvas, Metrics, Overlay};

/// The commodity that a unit eats. The set holds one.
const FOOD: CommodityId = CommodityId(0);

/// The period of the economy in this fixture.
const PERIOD: u32 = 2;

/// The number of sites the fixture founds.
///
/// Half of them feed their people and half of them cannot, so the fixture
/// holds both cases whatever the shortage does.
const SITES: usize = 8;

/// The number of units each site holds.
const PER_SITE: usize = 3;

/// The steps that leave the hungry group short and alive.
///
/// The count is fixed, so one seed and one rule give one condition for every
/// unit. Nothing here reads a clock.
///
/// The fixture asserts the outcome rather than trusting the number.
const SHORT_STEPS: usize = 16;

/// The steps that reach the scan which ends the hungry group.
///
/// The engine ends a unit inside the step that takes it to the bound, and it
/// keeps the log of one scan only. One step later the log is empty again, so
/// this count names the one step at which a watcher can read the deaths.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-119. `docs/FINDINGS.md`
const ENDING_STEPS: usize = 18;

/// The threads the fixture steps at.
const THREADS: usize = 4;

/// What the fixture built.
struct Fixture {
    /// The units of a site that feeds them.
    fed: Vec<Entity>,
    /// The units of a site that cannot feed them.
    hungry: Vec<Entity>,
}

/// Builds a world in which one group eats and another does not.
///
/// The units stand two tiles apart at most, and each unit has a tile of its
/// own, so a mark on one unit cannot reach another.
fn hungry_world() -> (World, Fixture) {
    let mut world = World::new(WorldConfig {
        width: 40,
        height: 40,
        seed: 7,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent describes a world");
    world
        .set_economy_schedule(PERIOD, 0)
        .expect("the period is inside the range");

    // The bound is a parameter of the rule, and a test states it. No kernel
    // holds one.
    let rule = NeedRule::DEFAULT;
    world.set_need_rule(
        NeedRule::new(
            rule.decay(),
            rule.ration(),
            rule.threshold(),
            rule.recovery(),
            NEED_FULL,
        )
        .expect("every rate is at or above zero"),
    );

    let ground = open_ground(&world);
    assert!(
        ground.len() > SITES * (PER_SITE + 2),
        "the world holds {} open tiles, and the fixture needs more",
        ground.len(),
    );

    let mut fixture = Fixture {
        fed: Vec::new(),
        hungry: Vec::new(),
    };
    let mut next = 0;
    for index in 0..SITES {
        let site = world
            .found_settlement(ground[next], FactionId(0))
            .expect("the tile is free");
        next += 1;
        let mut members = Vec::new();
        for _ in 0..PER_SITE {
            let unit = world
                .spawn_soldier(ground[next], FactionId(0))
                .expect("the ground admits a unit");
            next += 1;
            assert!(world.set_home_site(unit, Some(site)));
            members.push(unit);
        }
        if index % 2 == 0 {
            world
                .set_production_rate(site, FOOD, Fix32::from_int(2))
                .expect("the rate is at or above zero");
            fixture.fed.append(&mut members);
        } else {
            world
                .set_settlement_store(site, FOOD, Fix32(NEED_FULL.0 / 2))
                .expect("the commodity is in the set");
            fixture.hungry.append(&mut members);
        }
    }
    assert!(!fixture.fed.is_empty() && !fixture.hungry.is_empty());
    world.rebuild_bridge(THREADS).expect("the rebuild must run");
    (world, fixture)
}

/// Returns every address of a world that admits a unit, in index order.
fn open_ground(world: &World) -> Vec<Axial> {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| world.admits_a_unit(*address))
        .collect()
}

/// Runs the fixture until the shortage bites, and returns what it produced.
///
/// The fixture asserts its own outcome. A run in which nobody went short
/// would pass every test below without drawing anything.
fn bitten() -> (World, Fixture) {
    let (mut world, fixture) = hungry_world();
    for _ in 0..SHORT_STEPS {
        world.step(THREADS).expect("the step must run");
    }
    let short = fixture
        .hungry
        .iter()
        .filter(|unit| world.unit_condition(**unit) == Some(NeedCondition::Short))
        .count();
    assert!(
        short > 0,
        "no unit went short in {SHORT_STEPS} steps, so the fixture supplies no case",
    );
    let fed = fixture
        .fed
        .iter()
        .filter(|unit| world.unit_condition(**unit) == Some(NeedCondition::Fed))
        .count();
    assert!(
        fed > 0,
        "every unit went short, so a viewer that marked every unit would pass",
    );
    (world, fixture)
}

/// Returns measurements that repeat, so that no test reads a clock.
fn measurements() -> Metrics {
    Metrics::fixed(
        SHORT_STEPS as u64,
        1,
        Duration::from_micros(100),
        Duration::from_micros(200),
        Duration::from_millis(10),
    )
}

/// Draws the whole world at a tile size large enough to read one unit.
fn drawn(world: &World, at: Axial) -> (Canvas, Camera) {
    let mut canvas = Canvas::new(700, 700);
    let camera = Camera::at_tile_size(24.0)
        .looking_at(at, &canvas)
        .clamped(world, &canvas);
    paint::draw(world, camera, &mut canvas).expect("the bridge describes the arena");
    (canvas, camera)
}

/// Reports whether the disc of one unit holds a colour.
fn disc_holds(canvas: &Canvas, camera: Camera, address: Axial, colour: u32) -> bool {
    let (x, y) = camera.centre_of(address);
    let reach = (camera.tile_width * 0.35) as i32;
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
            if canvas.pixels()[row * canvas.width() + column] == colour {
                return true;
            }
        }
    }
    false
}

/// Returns the address a unit stands on.
fn address_of(world: &World, unit: Entity) -> Axial {
    world.soldiers().address(unit).expect("the unit lives")
}

#[test]
fn the_window_marks_a_unit_that_a_shortage_holds() {
    // The window, not the library. A watcher must see the mark on the unit
    // itself.
    let (world, fixture) = bitten();
    let hungry = *fixture
        .hungry
        .iter()
        .find(|unit| world.unit_condition(**unit) == Some(NeedCondition::Short))
        .expect("the fixture holds a unit the shortage holds");
    let place = address_of(&world, hungry);
    let (canvas, camera) = drawn(&world, place);

    assert!(
        disc_holds(&canvas, camera, place, paint::shortage_colour()),
        "the unit at {place:?} is short and its disc carries no mark",
    );
}

#[test]
fn the_window_leaves_a_fed_unit_in_the_colour_of_its_faction() {
    // A mark on every unit would pass the test above. This one fails when
    // the viewer marks a unit the shortage never touched.
    let (world, fixture) = bitten();
    let fed = *fixture
        .fed
        .iter()
        .find(|unit| world.unit_condition(**unit) == Some(NeedCondition::Fed))
        .expect("the fixture holds a fed unit");
    let place = address_of(&world, fed);
    let (canvas, camera) = drawn(&world, place);

    assert!(
        !disc_holds(&canvas, camera, place, paint::shortage_colour()),
        "the unit at {place:?} eats and its disc carries the mark of a shortage",
    );
    assert!(
        disc_holds(&canvas, camera, place, paint::faction_colour(FactionId(0))),
        "the unit at {place:?} lost the colour of its faction",
    );
}

#[test]
fn the_pass_reads_one_condition_for_each_unit_it_paints() {
    // The panel record forbids a pass of the viewer's own over the world.
    // A count above the units painted says that the layer started one.
    let (world, _) = bitten();
    let (canvas, _) = drawn(&world, Axial::new(4, 4));

    assert!(canvas.soldiers_painted() > 0, "the pass painted no unit");
    assert_eq!(
        canvas.condition_reads(),
        canvas.soldiers_painted(),
        "the pass read {} conditions and painted {} units",
        canvas.condition_reads(),
        canvas.soldiers_painted(),
    );
}

#[test]
fn the_marked_count_falls_when_the_window_leaves_the_hungry_group() {
    // The label says the count is of the drawn units. This is the test of
    // that label.
    let (world, fixture) = bitten();
    let hungry = *fixture
        .hungry
        .iter()
        .find(|unit| world.unit_condition(**unit) == Some(NeedCondition::Short))
        .expect("the fixture holds a unit the shortage holds");
    let (whole, _) = drawn(&world, address_of(&world, hungry));
    assert!(whole.units_short() > 0, "the window showed no marked unit");

    let mut canvas = Canvas::new(120, 120);
    let camera = Camera::at_tile_size(24.0)
        .looking_at(Axial::new(38, 38), &canvas)
        .clamped(&world, &canvas);
    paint::draw(&world, camera, &mut canvas).expect("the bridge describes the arena");

    assert!(
        canvas.units_short() < whole.units_short(),
        "the count stayed at {} with the window off the hungry group",
        canvas.units_short(),
    );
}

#[test]
fn the_panel_states_the_shortage() {
    // The panel is part of the frame, so the test draws the frame. A readout
    // that renders proves nothing until something reaches it.
    let (world, fixture) = bitten();
    let hungry = *fixture
        .hungry
        .iter()
        .find(|unit| world.unit_condition(**unit) == Some(NeedCondition::Short))
        .expect("the fixture holds a unit the shortage holds");
    let mut canvas = Canvas::new(700, 700);
    let camera = Camera::at_tile_size(24.0)
        .looking_at(address_of(&world, hungry), &canvas)
        .clamped(&world, &canvas);

    let readout = draw_frame(
        &world,
        camera,
        &measurements(),
        &[],
        Overlay::Panel,
        &mut canvas,
    )
    .expect("the world draws");

    assert!(
        readout.units_short() > 0,
        "the panel states no unit that a shortage holds",
    );
    assert_eq!(
        readout.units_short(),
        canvas.units_short(),
        "the panel and the pass disagree about the units they marked",
    );
    assert_eq!(
        readout.units_ended(),
        world.starved_log().len(),
        "the panel restates the log of the last scan and derives nothing",
    );
}

#[test]
fn the_shortage_mark_is_not_the_colour_of_any_faction_or_any_ground() {
    // A mark in a colour the picture already uses says nothing. The colours
    // come from the readers the picture uses, never from a literal.
    let mark = paint::shortage_colour();
    for index in 0..8u16 {
        assert_ne!(
            mark,
            paint::faction_colour(FactionId(index)),
            "the mark of a shortage is the colour of faction {index}",
        );
    }
    for kind in [
        cachette_core::terrain::TileKind::Water,
        cachette_core::terrain::TileKind::Plain,
        cachette_core::terrain::TileKind::Forest,
        cachette_core::terrain::TileKind::Hill,
        cachette_core::terrain::TileKind::Mountain,
    ] {
        assert_ne!(
            mark,
            paint::kind_colour(kind),
            "the mark of a shortage is the colour of a kind of ground",
        );
    }
    assert_ne!(
        mark,
        paint::over_capacity_colour(),
        "the mark of a shortage is the mark of a full tile",
    );
}

#[test]
fn a_drawn_frame_leaves_the_hungry_world_where_it_found_it() {
    // The viewer reads the world and writes nothing to it. The state hash is
    // the check a reviewer can run, and it must not move over a draw.
    let (world, fixture) = bitten();
    let hungry = fixture.hungry[0];
    let hash = world.state_hash();

    let mut canvas = Canvas::new(700, 700);
    let camera = Camera::at_tile_size(24.0)
        .looking_at(address_of(&world, hungry), &canvas)
        .clamped(&world, &canvas);
    let readout = draw_frame(
        &world,
        camera,
        &measurements(),
        &[],
        Overlay::Panel,
        &mut canvas,
    )
    .expect("the world draws");
    assert!(
        readout.units_short() > 0,
        "the frame marked nothing, so the comparison proves nothing",
    );

    assert_eq!(hash, world.state_hash(), "the drawing moved the world");
}

/// Runs the fixture to the scan that ends the hungry group.
///
/// The fixture asserts its own outcome. A run that ended nobody would pass a
/// test of the row that states the deaths.
fn ended() -> World {
    let (mut world, fixture) = hungry_world();
    for _ in 0..ENDING_STEPS {
        world.step(THREADS).expect("the step must run");
    }
    assert!(
        !world.starved_log().is_empty(),
        "the scan ended nobody in {ENDING_STEPS} steps, so the fixture \
         supplies no case",
    );
    assert!(
        fixture
            .fed
            .iter()
            .all(|unit| world.unit_condition(*unit) == Some(NeedCondition::Fed)),
        "the shortage reached the group that eats, so the fixture cannot tell \
         a death by shortage from a death of everybody",
    );
    world
}

#[test]
fn the_panel_states_how_many_units_the_last_scan_ended() {
    // A watcher cannot see a unit at the moment a shortage ends it, because
    // the engine ends it inside the step that takes it to the bound. This
    // row is the whole record of a death that the window holds.[^1]
    //
    // [^1]: Findings register, FND-119. `docs/FINDINGS.md`
    let world = ended();
    let mut canvas = Canvas::new(700, 700);
    let camera = Camera::at_tile_size(24.0)
        .looking_at(Axial::new(6, 6), &canvas)
        .clamped(&world, &canvas);

    let readout = draw_frame(
        &world,
        camera,
        &measurements(),
        &[],
        Overlay::Panel,
        &mut canvas,
    )
    .expect("the world draws");

    assert_eq!(
        readout.units_ended(),
        world.starved_log().len(),
        "the panel states a count that the engine did not give it",
    );
    assert!(
        readout.units_ended() > 0,
        "the panel states no death at the step that ended a group",
    );
}

#[test]
fn the_count_of_deaths_falls_back_when_a_step_ends_nobody() {
    // The row states the log of one scan. A row that summed the deaths of a
    // run would never fall, and the label would then be false.
    let mut world = ended();
    let mut canvas = Canvas::new(700, 700);
    let camera = Camera::at_tile_size(24.0)
        .looking_at(Axial::new(6, 6), &canvas)
        .clamped(&world, &canvas);
    let at_the_scan = draw_frame(
        &world,
        camera,
        &measurements(),
        &[],
        Overlay::Panel,
        &mut canvas,
    )
    .expect("the world draws");
    assert!(at_the_scan.units_ended() > 0);

    world.step(THREADS).expect("the step must run");
    let after = draw_frame(
        &world,
        camera,
        &measurements(),
        &[],
        Overlay::Panel,
        &mut canvas,
    )
    .expect("the world draws");

    assert_eq!(
        after.units_ended(),
        0,
        "the panel still states {} deaths one step after the scan",
        after.units_ended(),
    );
}

#[test]
fn no_live_unit_is_starved_after_a_completed_step() {
    // The viewer draws one mark for a unit that a shortage holds, and no
    // second mark for a unit at the bound. This test states why: the engine
    // ends a unit inside the step that takes it there, so a completed step
    // leaves no starved unit for the viewer to draw.[^1]
    //
    // A second mark would be a capability that nothing invokes.[^2]
    //
    // [^1]: Findings register, FND-119. `docs/FINDINGS.md`
    // [^2]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
    let (mut world, fixture) = hungry_world();
    let mut ended_at_all = false;
    for _ in 0..ENDING_STEPS + 4 {
        world.step(THREADS).expect("the step must run");
        ended_at_all = ended_at_all || !world.starved_log().is_empty();
        for unit in fixture.fed.iter().chain(&fixture.hungry) {
            assert_ne!(
                world.unit_condition(*unit),
                Some(NeedCondition::Starved),
                "a completed step left a unit at the bound, so the viewer \
                 must draw that condition",
            );
        }
    }
    assert!(
        ended_at_all,
        "no scan ended anybody, so the run never reached the bound",
    );
}
