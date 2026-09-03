//! The viewer shows the food on the ground, and says why a unit chose.
//!
//! Until this suite existed the colour of a tile came from the tile value
//! field, which is a number no other system reads or writes. A watcher read
//! noise. The resources that the ground generates, and that the founding
//! survey reads to choose a place, were drawn by nothing.[^1]
//!
//! The engine also holds a verb that reports every option score, the value
//! each option read, and the winner. No file outside the core crate called
//! it, so the answer existed and the question could not be put.[^2]
//!
//! Every test here drives the drawing pass or the panel, never a private
//! function, because a test that builds the mechanism proves the mechanism
//! and not the reach.[^3]
//!
//! # What each fixture supplies
//!
//! A fixture that holds no food supplies no case, and an assertion about
//! food then measures the fixture.[^4] Each fixture below asserts its own
//! outcome: it reads the stock back out of the engine and refuses a world in
//! which nothing carries food.
//!
//! # References
//!
//! [^1]: What a unit does in a tick, section 1. `docs/research/what-a-unit-does-in-a-tick.md`
//! [^2]: What a unit does in a tick, section 3.8. `docs/research/what-a-unit-does-in-a-tick.md`
//! [^3]: Testing rules, section 5. `.claude/rules/testing.md`
//! [^4]: Testing rules, section 2a. `.claude/rules/testing.md`

// An integration test is its own crate, so the allowance at the viewer's
// crate root does not reach it. The reason is the same one: ADR-0067 D3 puts
// the float boundary at the viewer, and a camera is a viewer value.
#![allow(clippy::disallowed_types)]

use std::collections::BTreeMap;

use cachette_core::resource::ResourceKind;
use cachette_core::{Axial, Entity, FactionId, World, WorldConfig};
use cachette_view::{draw_frame, paint, Camera, Canvas, Metrics, Overlay};

/// The extent of the fixture world.
///
/// The extent is wider than the coarsest lattice spacing of the terrain
/// generator, so the world holds more than one kind of ground.
const EXTENT: u32 = 128;

/// The size of the canvas the tests paint onto.
const CANVAS: (usize, usize) = (512, 512);

/// Builds a fixture world of one seed, with its derived structure rebuilt.
fn world_of(seed: u64, extent: u32) -> World {
    let mut world = World::new(WorldConfig {
        width: extent,
        height: extent,
        seed,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent describes a world");
    world.rebuild_bridge(1).expect("the rebuild must succeed");
    world
}

/// Returns every address of the world, in index order.
fn addresses_of(world: &World) -> Vec<Axial> {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .collect()
}

/// Returns the sum of the three channels of one colour.
///
/// The shading of a tile adds the same number to each channel, so the sum
/// orders two tiles of one kind by how much shade each carries.
fn brightness(colour: u32) -> i64 {
    let channel = |offset: u32| i64::from((colour >> offset) & 0xff);
    channel(16) + channel(8) + channel(0)
}

/// Returns the colour the canvas holds at one pixel.
fn pixel(canvas: &Canvas, x: i32, y: i32) -> u32 {
    assert!(x >= 0 && y >= 0, "the pixel {x}, {y} is off the canvas");
    let (x, y) = (x as usize, y as usize);
    assert!(
        x < canvas.width() && y < canvas.height(),
        "the pixel {x}, {y} is off the canvas"
    );
    canvas.pixels()[y * canvas.width() + x]
}

/// Returns the colour of the ground of one tile, away from the middle of it.
///
/// A unit draws as a disc at the middle of its tile, so a reader that sampled
/// the middle would read the colour of a unit and call it the ground.
fn ground_pixel(camera: Camera, canvas: &Canvas, address: Axial) -> u32 {
    // The rectangle comes from the drawing itself, so a sample cannot miss
    // the square by one pixel and read the background instead. The disc of a
    // unit covers about a third of the tile at the middle, so the last pixel
    // of the rectangle is inside the tile and outside the disc.
    let (left, top, wide, tall) = paint::tile_rect(camera, address);
    pixel(canvas, left + wide - 1, top + tall - 1)
}

#[test]
fn the_colour_of_the_ground_rises_with_the_food_on_it() {
    // The ground draws brighter where there is more food. The height also
    // brightens a tile, and the two draws are keyed differently, so height
    // is independent of food across a large sample. The comparison is
    // therefore between the mean of the tiles that carry food and the mean
    // of the tiles that carry none, over one kind of ground.
    // The world is small enough that a tile is several pixels wide in the
    // canvas. A tile of four pixels leaves no pixel that belongs to one tile
    // alone, and the sample would then read a neighbour.
    let world = world_of(0x0cac_f00d, 64);
    let mut canvas = Canvas::new(CANVAS.0, CANVAS.1);
    let camera = Camera::fitting(&world, &canvas);
    paint::draw(&world, camera, &mut canvas).expect("the world draws");

    // The comparison holds the kind of ground fixed, because the kind decides
    // the base colour. The kind is the one the world holds the most of that
    // also carries food, so the fixture picks a case rather than assuming
    // one.
    let mut by_kind: BTreeMap<u8, (Vec<i64>, Vec<i64>)> = BTreeMap::new();
    for address in addresses_of(&world) {
        let Some(kind) = world.tile_kind(address) else {
            continue;
        };
        let food = world
            .tile_stock(address, ResourceKind::Food)
            .expect("the address names a tile")
            .0;
        let shade = brightness(ground_pixel(camera, &canvas, address));
        let entry = by_kind.entry(kind.to_u8()).or_default();
        if food == 0 {
            entry.0.push(shade);
        } else if food >= 4 {
            entry.1.push(shade);
        }
    }
    let (empty, carrying) = by_kind
        .into_values()
        .max_by_key(|(empty, carrying)| empty.len().min(carrying.len()))
        .expect("the world holds ground");

    // The fixture asserts its own outcome. A world whose plains carry no
    // food supplies no case, and the comparison below would then measure
    // nothing.
    assert!(
        empty.len() > 200 && carrying.len() > 50,
        "the fixture must hold both kinds of tile: {} empty and {} carrying",
        empty.len(),
        carrying.len()
    );

    let mean = |values: &[i64]| values.iter().sum::<i64>() as f64 / values.len() as f64;
    let (bare, fed) = (mean(&empty), mean(&carrying));
    assert!(
        fed > bare + 10.0,
        "a tile carrying food must draw brighter than an empty one of the \
         same ground: {fed:.1} against {bare:.1}"
    );
}

#[test]
fn a_gather_darkens_the_tile_it_took_from() {
    // This drives the engine and then reads the picture. The gather resolve
    // is a stage of the step, so the test starts at the step and never at
    // the drawing.[^1]
    //
    // [^1]: Testing rules, section 5. `.claude/rules/testing.md`
    let mut world = world_of(0x0cac_f00d, 48);

    // A tile that carries food, and that admits a unit.
    let deposit = addresses_of(&world)
        .into_iter()
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
    // The holding spreads from a unit and mixes into the colour of a tile.
    // Two steps settle it before the first picture, so the only thing that
    // changes the colour between the two pictures is the stock.
    for _ in 0..2 {
        world.step(1).expect("the step must run");
    }
    world
        .place_soldier(unit, deposit)
        .expect("the tile admits the unit");
    world.rebuild_bridge(1).expect("the rebuild must succeed");

    let mut canvas = Canvas::new(CANVAS.0, CANVAS.1);
    let camera = Camera::fitting(&world, &canvas);
    paint::draw(&world, camera, &mut canvas).expect("the world draws");
    let before = ground_pixel(camera, &canvas, deposit);
    let held_before = world.tile_holder(deposit);
    let stock_before = world
        .tile_stock(deposit, ResourceKind::Food)
        .expect("the address names a tile")
        .0;

    assert!(
        world.order_gather(unit, ResourceKind::Food),
        "the engine must accept the order"
    );
    world.step(1).expect("the step must run");

    let stock_after = world
        .tile_stock(deposit, ResourceKind::Food)
        .expect("the address names a tile")
        .0;
    // The fixture asserts its own outcome. A step in which nobody gathered
    // gives two identical pictures, and the assertion below would then pass
    // against nothing.
    assert!(
        stock_after < stock_before,
        "the gather must take from the tile: {stock_before} then {stock_after}"
    );
    assert_eq!(
        held_before,
        world.tile_holder(deposit),
        "the holder of the tile must not change between the two pictures"
    );

    paint::draw(&world, camera, &mut canvas).expect("the world draws");
    let after = ground_pixel(camera, &canvas, deposit);
    assert!(
        brightness(after) < brightness(before),
        "a drained deposit must draw darker: {:06x} then {:06x}",
        before,
        after
    );
}

#[test]
fn the_picture_of_a_tile_holds_still_while_its_stock_holds_still() {
    // The colour used to come from a field that redrew itself on every tick,
    // so every tile flickered and the flicker meant nothing.[^1] A tile
    // whose stock nobody touched must now draw the same on two ticks.
    //
    // [^1]: What a unit does in a tick, section 1. `docs/research/what-a-unit-does-in-a-tick.md`
    let mut world = world_of(0x0cac_f00d, 48);
    let mut canvas = Canvas::new(CANVAS.0, CANVAS.1);
    let camera = Camera::fitting(&world, &canvas);

    paint::draw(&world, camera, &mut canvas).expect("the world draws");
    let first: Vec<u32> = addresses_of(&world)
        .into_iter()
        .map(|address| ground_pixel(camera, &canvas, address))
        .collect();

    for _ in 0..4 {
        world.step(1).expect("the step must run");
    }
    paint::draw(&world, camera, &mut canvas).expect("the world draws");
    let second: Vec<u32> = addresses_of(&world)
        .into_iter()
        .map(|address| ground_pixel(camera, &canvas, address))
        .collect();

    assert_eq!(
        first, second,
        "a world in which nobody gathered must draw the same ground twice"
    );
}

/// Builds a world with a group founded in it, and paints one frame.
///
/// The panel names a unit, so the fixture must hold units. A founded run puts
/// a group of people on the ground and gives them a home site, which is what
/// the site rows report.
fn founded(
    seed: u64,
) -> (
    World,
    Canvas<'static>,
    Camera,
    Vec<cachette_core::FoundingOutcome>,
) {
    let mut world = world_of(seed, EXTENT);
    let outcomes = world.found_run_for_every_faction(24);
    let place = outcomes
        .iter()
        .find_map(|outcome| outcome.founding().map(cachette_core::Founding::place))
        .expect("the world holds a place for a group");
    world.rebuild_bridge(1).expect("the rebuild must succeed");
    for _ in 0..3 {
        world.step(1).expect("the step must run");
    }

    let canvas = Canvas::new(CANVAS.0, CANVAS.1);
    let camera = Camera::opening()
        .looking_at(place, &canvas)
        .clamped(&world, &canvas);
    (world, canvas, camera, outcomes)
}

/// Returns the unit the drawing pass should name, computed a second way.
///
/// The panel reads the unit the drawing pass fixed. This walks every live
/// unit of the world instead, which the panel must never do, and picks the
/// one nearest the middle of the canvas among those the window covers.
fn nearest_by_a_full_scan(world: &World, camera: Camera, canvas: &Canvas) -> Option<Entity> {
    let arena = world.soldiers();
    let radius = ((camera.tile_width * 0.3) as i32).max(1);
    let mut best: Option<(i64, Entity)> = None;
    for unit in arena.iter() {
        let Some(address) = arena.address(unit) else {
            continue;
        };
        let (x, y) = camera.centre_of(address);
        // The same reach test the drawing pass makes. A unit whose disc
        // cannot touch the canvas is not painted, so it is not a candidate.
        let reach = radius as f32;
        if x + reach < 0.0
            || y + reach < 0.0
            || x - reach >= canvas.width() as f32
            || y - reach >= canvas.height() as f32
        {
            continue;
        }
        let across = i64::from(x as i32 - (canvas.width() / 2) as i32);
        let down = i64::from(y as i32 - (canvas.height() / 2) as i32);
        let reach = across * across + down * down;
        if best.is_none_or(|(held, _)| reach < held) {
            best = Some((reach, unit));
        }
    }
    best.map(|(_, unit)| unit)
}

#[test]
fn the_panel_names_the_unit_nearest_the_middle_of_the_window() {
    let (world, mut canvas, camera, outcomes) = founded(0x0cac_f00d);
    let readout = draw_frame(
        &world,
        camera,
        &Metrics::start(),
        &outcomes,
        Overlay::Panel,
        &mut canvas,
    )
    .expect("the world draws");

    let choice = readout
        .choice()
        .expect("the window holds units, so the panel names one");
    let expected = nearest_by_a_full_scan(&world, camera, &canvas).expect("the window holds units");
    assert_eq!(
        choice.focus().entity(),
        expected,
        "the panel must name the nearest drawn unit"
    );
    assert_eq!(
        world.soldiers().address(expected),
        Some(choice.focus().address()),
        "the panel must state the tile the engine holds for that unit"
    );
}

#[test]
fn the_panel_states_the_answer_the_engine_gave_and_derives_no_part_of_it() {
    let (world, mut canvas, camera, outcomes) = founded(0x0cac_f00d);
    let readout = draw_frame(
        &world,
        camera,
        &Metrics::start(),
        &outcomes,
        Overlay::Panel,
        &mut canvas,
    )
    .expect("the world draws");

    let choice = readout.choice().expect("the window holds units");
    let answer = choice
        .explanation()
        .expect("the engine explains a live unit");
    assert_eq!(
        Some(answer),
        world.explain_choice(choice.focus().entity()),
        "the panel must restate the engine's answer, field for field"
    );
    // The fixture asserts its own outcome. An answer whose four scores are
    // all zero would let a panel that printed zeroes pass.
    assert!(
        answer.fields.iter().any(|field| field.0 != 0),
        "the fixture must supply an option that read something: {:?}",
        answer.fields
    );
}

#[test]
fn the_unit_the_panel_names_follows_the_window() {
    // A watcher has no cursor, so the middle of the window is the pointer.
    // Scrolling must therefore change the unit the panel reports on.
    let (world, mut canvas, camera, outcomes) = founded(0x0cac_f00d);
    let readout = draw_frame(
        &world,
        camera,
        &Metrics::start(),
        &outcomes,
        Overlay::Panel,
        &mut canvas,
    )
    .expect("the world draws");
    let first = readout.choice().expect("the window holds units");

    let moved = camera.stepped(6.0, 6.0).clamped(&world, &canvas);
    assert!(
        (moved.tile_width - camera.tile_width).abs() < f32::EPSILON,
        "the scroll must not change the zoom"
    );
    let readout = draw_frame(
        &world,
        moved,
        &Metrics::start(),
        &outcomes,
        Overlay::Panel,
        &mut canvas,
    )
    .expect("the world draws");
    let second = readout.choice().expect("the window holds units");
    assert_ne!(
        first.focus().entity(),
        second.focus().entity(),
        "a scroll of six tiles must reach a different unit"
    );
}

#[test]
fn the_panel_states_the_stock_of_the_tile_under_the_crosshair() {
    let (world, mut canvas, camera, outcomes) = founded(0x0cac_f00d);
    let readout = draw_frame(
        &world,
        camera,
        &Metrics::start(),
        &outcomes,
        Overlay::Panel,
        &mut canvas,
    )
    .expect("the world draws");

    let tile = readout.tile().expect("the middle of the window is a tile");
    let mut carried = 0u32;
    for kind in ResourceKind::ALL {
        assert_eq!(
            Some(tile.stock(kind)),
            world
                .tile_stock(tile.address(), kind)
                .map(|amount| amount.0),
            "the panel must state the stock the engine holds"
        );
        assert_eq!(
            Some(tile.generated(kind)),
            world
                .original_stock(tile.address(), kind)
                .map(|amount| amount.0),
            "the panel must state the stock the ground generated"
        );
        carried += tile.generated(kind);
    }
    assert!(
        carried > 0,
        "the fixture must put the crosshair on ground that carries something"
    );
}

#[test]
fn the_panel_tells_what_is_left_from_what_the_ground_gave() {
    // A world in which nobody gathered holds the two numbers equal, so a
    // panel that printed the generated stock in both rows would pass. This
    // test gathers first, and it is the only one that separates them.[^1]
    //
    // [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
    let mut world = world_of(0x0cac_f00d, 48);
    let deposit = addresses_of(&world)
        .into_iter()
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

    let mut canvas = Canvas::new(CANVAS.0, CANVAS.1);
    let camera = Camera::opening()
        .looking_at(deposit, &canvas)
        .clamped(&world, &canvas);
    let readout = draw_frame(
        &world,
        camera,
        &Metrics::start(),
        &[],
        Overlay::Panel,
        &mut canvas,
    )
    .expect("the world draws");

    let tile = readout.tile().expect("the middle of the window is a tile");
    assert_eq!(
        tile.address(),
        deposit,
        "the crosshair must sit on the tile that was gathered from"
    );
    // The fixture asserts its own outcome. A step in which nobody gathered
    // leaves the two numbers equal, and the assertion below would then hold
    // for a panel that read the generated stock twice.
    assert!(
        tile.stock(ResourceKind::Food) < tile.generated(ResourceKind::Food),
        "the gather must separate what is left from what the ground gave: \
         {} of {}",
        tile.stock(ResourceKind::Food),
        tile.generated(ResourceKind::Food)
    );
    assert_eq!(
        Some(tile.stock(ResourceKind::Food)),
        world
            .tile_stock(deposit, ResourceKind::Food)
            .map(|amount| amount.0),
        "the panel must state the stock the engine holds"
    );
}

#[test]
fn the_panel_states_the_store_and_the_rate_of_every_site() {
    let (world, mut canvas, camera, outcomes) = founded(0x0cac_f00d);
    let readout = draw_frame(
        &world,
        camera,
        &Metrics::start(),
        &outcomes,
        Overlay::Panel,
        &mut canvas,
    )
    .expect("the world draws");

    let arena = world.settlements();
    let commodity = cachette_core::CommodityId(0);
    let held: BTreeMap<(i32, i32), Entity> = arena
        .iter()
        .filter_map(|site| arena.address(site).map(|at| ((at.q, at.r), site)))
        .collect();
    // The fixture asserts its own outcome. A run that seated nobody gives an
    // empty list, and the loop below would then assert nothing.
    assert!(!held.is_empty(), "the fixture must seat at least one site");
    assert_eq!(
        readout.sites_held() as usize,
        held.len(),
        "the panel must state how many sites the world holds"
    );
    assert!(
        !readout.sites().is_empty() && readout.sites().len() <= held.len(),
        "the panel must state a row for each site it read, and no more"
    );
    for row in readout.sites() {
        let site = held[&(row.place().q, row.place().r)];
        assert_eq!(
            Some(row.store()),
            arena
                .store(site)
                .and_then(|store| store.quantity(commodity)),
            "the panel must state the store the engine holds"
        );
        assert_eq!(
            Some(row.production()),
            world.production_rate(site, commodity),
            "the panel must state the rate the founding set"
        );
    }
    // The founding sets a production rate from what the survey reached, so a
    // world where every rate is zero would let a panel that printed zeroes
    // pass.
    assert!(
        readout.sites().iter().any(|row| row.production().0 != 0),
        "the fixture must seat a site that produces something"
    );
}
