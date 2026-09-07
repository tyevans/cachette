//! The drawing shows an upgrade, a storm, wet ground and a luxury.
//!
//! Each test paints a world with the feature and the same world without it,
//! and reads one pixel of one tile in both. A layer put back to a no-op
//! makes the two pixels equal, and the test fails. Each fixture asserts that
//! nothing else about the tile differs, so the pixel cannot differ for a
//! reason the layer did not cause.[^1]
//!
//! The drawing reads the world through a shared reference and writes
//! nothing to it, so every fixture is built before the drawing starts.[^2]
//!
//! # References
//!
//! [^1]: Testing Rules, section 2a. `.claude/rules/testing.md`
//! [^2]: ADR-0067, the viewer reads the world and never writes to it, decision D1. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`

#![allow(clippy::disallowed_types)]

use cachette_core::luxury::LuxuryId;
use cachette_core::resource::ResourceKind;
use cachette_core::upgrade::{UpgradeCategory, UPGRADE_LEVEL_COUNT};
use cachette_core::{Axial, Entity, FactionId, Holder, World, WorldConfig};
use cachette_view::hud::TileReadout;
use cachette_view::overlay;
use cachette_view::paint;
use cachette_view::paint::{tile_rect, Camera, Canvas};

/// The side of the fixture world, in tiles.
const EXTENT: u32 = 64;

/// The seed of the fixture world.
const SEED: u64 = 7;

/// The band of tiles the fixture fills with one faction, as a half-open
/// range on both axes.
const BAND: (i32, i32) = (20, 40);

/// The ticks the fixture runs before the faction holds ground.
const HOLDING_TICKS: u32 = 4;

/// How far around the city the fixture leaves the ground empty.
///
/// A unit covers most of its tile below sixteen pixels a tile, so a unit
/// standing on a tile decides the colour of that tile's corner at a close
/// zoom and not at a far one. The tests that read the air compare one tile at
/// two zooms, and they need ground with nothing standing on it. The tiles
/// inside this radius are held, because the city reaches further than it.
const QUIET_RADIUS: u32 = 3;

/// The ticks a storm is given to fall out of the air onto the ground.
///
/// After this many ticks the sky over the cell stands below the mark at which
/// the overlay changes a pixel, and the ground is wet. The wet ground test
/// therefore sees the wet layer alone.
///
/// **The count rose when the capacity of the air became the published
/// curve.** The engine pours out whatever stands above the capacity of a cell
/// within one step, and the cell then drizzles the rest away a share at a
/// time. The share is the same, and the sky it empties is measured against a
/// capacity that follows the temperature, so a warm cell takes longer to fall
/// under the mark than it did against one ceiling for the whole plane.[^1]
///
/// # References
///
/// [^1]: ADR-0177, the row axis of a world is a latitude that the world states, decision D4. `docs/adrs/draft/adr-0177-the-row-axis-of-a-world-is-a-latitude-that-the-world-states.md`
const FALLING_TICKS: u32 = 60;

/// The ticks the rest test gives a stormed sky to fall under the mark at which
/// the overlay draws.
///
/// **This is longer than the fall of a storm onto the ground.** The engine
/// pours out whatever stands above the capacity of a cell within one step, and
/// the cell then drizzles the rest away a share at a time. The overlay reads
/// the sky against the capacity of that cell, so the share it must cross is
/// the same at every temperature and the drizzle alone has to carry it.
const THINNING_TICKS: u32 = 900;

/// The strength of the storm the fixture inflicts.
const STRENGTH: u8 = 4;

/// The size of a tile in the picture, in pixels.
const TILE: f32 = 8.0;

/// The size of the picture, in pixels.
const WINDOW: (usize, usize) = (320, 320);

fn settings() -> WorldConfig {
    WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: SEED,
        faction_count: 2,
        unit_capacity: 4096,
    }
}

/// Builds a world in which one faction holds a band of ground.
///
/// A faction holds the ground within reach of a city it owns, and a unit
/// standing on a tile gives its faction no claim on it.[^1] The fixture
/// therefore founds a city in the middle of the band, and the soldiers of
/// the band are the units the tests draw and order about. A fixture that
/// took ground by standing units on it would be a second declaration of the
/// holding rule.[^2]
///
/// Returns the world and the soldiers it spawned, in spawn order.
///
/// # References
///
/// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D1. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
/// [^2]: Findings register, FND-487. `docs/FINDINGS.md`
fn a_held_band() -> (World, Vec<(Entity, Axial)>) {
    let mut world = World::new(settings()).expect("the extent describes a world");
    let middle = (BAND.0 + BAND.1) / 2;
    let seat = (BAND.0..BAND.1)
        .flat_map(|row| (BAND.0..BAND.1).map(move |column| Axial::new(column, row)))
        .min_by_key(|at| (at.q - middle).abs() + (at.r - middle).abs())
        .filter(|at| world.admits_a_unit(*at))
        .expect("the middle of the band admits a city");
    world
        .found_settlement(seat, FactionId(0))
        .expect("the seat admits a city");
    let mut soldiers = Vec::new();
    for row in BAND.0..BAND.1 {
        for column in BAND.0..BAND.1 {
            let at = Axial::new(column, row);
            if !world.admits_a_unit(at) || at.distance(seat) <= QUIET_RADIUS {
                continue;
            }
            let soldier = world
                .spawn_soldier(at, FactionId(0))
                .expect("the address and the faction are valid");
            soldiers.push((soldier, at));
        }
    }
    for _ in 0..HOLDING_TICKS {
        world.step(1).expect("the step must run");
    }
    assert!(
        world.holding_of(FactionId(0)) > 0,
        "the faction holds nothing, so the fixture supplies no held ground",
    );
    (world, soldiers)
}

/// Returns one held tile that carries nothing, away from the edge.
///
/// The picture draws the edge of a holding, a city draws a mark, and a unit
/// covers most of its tile at a close zoom. Each of the three covers a
/// different share of a tile at each zoom, and the tests that read the air
/// compare one tile at two zooms. The tile this returns therefore has six
/// held neighbours, no city and no unit, so the only layer over its ground is
/// the one under test.
fn a_held_tile(world: &World) -> Axial {
    let width = world.grid().width();
    let held_by_the_faction =
        |at: Axial| world.tile_holder(at).and_then(Holder::faction) == Some(FactionId(0));
    world
        .holding()
        .tiles_held_by(FactionId(0))
        .map(|held| Axial::new((held.0 % width) as i32, (held.0 / width) as i32))
        .find(|at| {
            world.settlements().on_tile(*at).is_none()
                && TileReadout::of(world, *at).and_then(|tile| tile.units()) == Some(0)
                && world
                    .grid()
                    .neighbours(*at)
                    .into_iter()
                    .all(|side| side.is_some_and(held_by_the_faction))
        })
        .expect("the faction holds an empty tile away from the edge of its holding")
}

fn address_of(world: &World, index: u32) -> Axial {
    let width = world.grid().width();
    Axial::new((index % width) as i32, (index / width) as i32)
}

/// Draws a world at a fixed camera over a tile, and returns the canvas.
fn drawn(world: &World, over: Axial) -> Canvas<'static> {
    let mut canvas = Canvas::new(WINDOW.0, WINDOW.1);
    let camera = Camera::at_tile_size(TILE)
        .looking_at(over, &canvas)
        .clamped(world, &canvas);
    paint::draw(world, camera, &mut canvas).expect("the world draws");
    canvas
}

/// Returns the camera the drawing used, so a test finds a tile in pixels.
fn camera_over(world: &World, over: Axial) -> Camera {
    let canvas = Canvas::new(WINDOW.0, WINDOW.1);
    Camera::at_tile_size(TILE)
        .looking_at(over, &canvas)
        .clamped(world, &canvas)
}

fn pixel(canvas: &Canvas, x: i32, y: i32) -> u32 {
    assert!(x >= 0 && y >= 0, "the position {x},{y} is off the canvas");
    let (x, y) = (x as usize, y as usize);
    assert!(
        x < canvas.width() && y < canvas.height(),
        "the position {x},{y} is off the canvas",
    );
    canvas.pixels()[y * canvas.width() + x]
}

/// Returns the pixel one step inside the top left corner of a tile.
///
/// A soldier is a disc at the middle of its tile, so the corner is ground
/// whether or not a soldier stands there.
fn corner_of(canvas: &Canvas, camera: Camera, address: Axial) -> u32 {
    let (left, top, _, _) = tile_rect(camera, address);
    pixel(canvas, left + 1, top + 1)
}

/// Asserts that two worlds agree about everything the drawing reads at one
/// tile, other than the layer under test.
fn same_ground(one: &World, other: &World, address: Axial) {
    let this = TileReadout::of(one, address).expect("the tile is inside the world");
    let that = TileReadout::of(other, address).expect("the tile is inside the world");
    assert_eq!(
        this.kind(),
        that.kind(),
        "the ground differs at {address:?}"
    );
    for kind in ResourceKind::ALL {
        assert_eq!(
            this.stock(kind),
            that.stock(kind),
            "the stock differs at {address:?}"
        );
    }
    assert_eq!(
        one.tile_holder(address).unwrap_or(Holder::NOBODY),
        other.tile_holder(address).unwrap_or(Holder::NOBODY),
        "the holder differs at {address:?}"
    );
}

#[test]
fn a_storm_in_the_air_changes_the_tile_under_it() {
    let (mut stormy, _) = a_held_band();
    let dry = stormy.clone();
    let place = a_held_tile(&stormy);
    stormy
        .inflict_weather(FactionId(0), &[place], STRENGTH)
        .expect("the faction holds the ground it storms");
    assert!(stormy.air_at(place).unwrap_or(0) > 0);
    same_ground(&dry, &stormy, place);

    let camera = camera_over(&dry, place);
    let before = corner_of(&drawn(&dry, place), camera, place);
    let after = corner_of(&drawn(&stormy, place), camera, place);
    assert_ne!(
        before, after,
        "a storm over the tile did not change its pixel at {place:?}"
    );
}

#[test]
fn wet_ground_gains_blue_and_holds_its_brightness() {
    let (mut stormy, _) = a_held_band();
    let mut dry = stormy.clone();
    let place = a_held_tile(&stormy);
    stormy
        .inflict_weather(FactionId(0), &[place], STRENGTH)
        .expect("the faction holds the ground it storms");
    for _ in 0..FALLING_TICKS {
        stormy.step(1).expect("the step must run");
        dry.step(1).expect("the step must run");
    }
    assert!(
        stormy.weather().wet_cells() > 0,
        "no cell is wet after the storm fell, so the fixture supplies no wet ground",
    );

    // A wet tile that nobody holds and nobody stands on, in a cell whose air
    // holds too few drops for the overlay to show. Every tile of the world is
    // a candidate, and the first that fits is the one compared.
    let grid = stormy.grid();
    let mut compared = None;
    for index in 0..grid.tile_count() {
        let address = address_of(&stormy, index);
        // **The world makes its own weather, so the dry clone rains too.** A
        // tile that is wet in both worlds shows no difference at all, and the
        // longer the fixture runs the more of them there are.
        if dry.ground_is_wet(address) != Some(false) {
            continue;
        }
        if stormy.ground_is_wet(address) != Some(true) {
            continue;
        }
        if stormy
            .tile_holder(address)
            .is_some_and(|holder| !holder.is_nobody())
        {
            continue;
        }
        if dry
            .tile_holder(address)
            .is_some_and(|holder| !holder.is_nobody())
        {
            continue;
        }
        let stood_on = |world: &World| {
            TileReadout::of(world, address).and_then(|tile| tile.units()) != Some(0)
        };
        if stood_on(&stormy) || stood_on(&dry) {
            continue;
        }
        compared = Some(address);
        break;
    }
    let address = compared.expect("the world holds a wet tile nobody holds or stands on");
    same_ground(&dry, &stormy, address);

    // **The picture is drawn below the tile width at which the air overlay
    // runs, so the wet layer is the sole difference.** The overlay moves all
    // three channels toward one pale colour, and the assertions below refuse
    // that: they hold the red and the green exactly where they were. The
    // earlier fixture waited for the sky over the cell to thin instead. It
    // cannot any more, because the overlay reads the sky against the capacity
    // of its own cell and a stormed cell stands at that capacity for as long
    // as the ground under it stays wet.[^10]
    //
    // [^10]: ADR-0177, the row axis of a world is a latitude that the world states, decision D4. `docs/adrs/draft/adr-0177-the-row-axis-of-a-world-is-a-latitude-that-the-world-states.md`
    let narrow = paint::air_least_tile() - 1.0;
    let camera = camera_at(&dry, address, narrow);
    let before = corner_of(&drawn_at(&dry, address, narrow), camera, address);
    let after = corner_of(&drawn_at(&stormy, address, narrow), camera, address);
    assert_ne!(
        before, after,
        "wet ground did not change the pixel at {address:?}"
    );

    // **Wet ground moves the blue and nothing else.** The layer used to take
    // the same number from every channel, which is the number the full food
    // ramp added, so a wet tile with the most food drew as a dry tile with
    // none.[^9]
    //
    // [^9]: Research report 24, defect 3. `docs/research/reports/24-demonstration-readability-resources-and-weather.md`
    let channel = |colour: u32, offset: u32| i64::from((colour >> offset) & 0xff);
    assert!(
        channel(after, 0) > channel(before, 0),
        "wet ground must add blue at {address:?}: {before:06x} then {after:06x}"
    );
    assert_eq!(
        channel(after, 16),
        channel(before, 16),
        "wet ground must not move the red at {address:?}: {before:06x} then {after:06x}"
    );
    assert_eq!(
        channel(after, 8),
        channel(before, 8),
        "wet ground must not move the green at {address:?}: {before:06x} then {after:06x}"
    );
}

/// Draws a world at a named tile size over a tile, and returns the canvas.
fn drawn_at(world: &World, over: Axial, tile: f32) -> Canvas<'static> {
    let mut canvas = Canvas::new(WINDOW.0, WINDOW.1);
    paint::draw(world, camera_at(world, over, tile), &mut canvas).expect("the world draws");
    canvas
}

/// Returns the camera a picture at a named tile size used.
fn camera_at(world: &World, over: Axial, tile: f32) -> Camera {
    let canvas = Canvas::new(WINDOW.0, WINDOW.1);
    Camera::at_tile_size(tile)
        .looking_at(over, &canvas)
        .clamped(world, &canvas)
}

/// Returns the distance between two colours, as a sum over the channels.
fn apart(one: u32, other: u32) -> i64 {
    [16, 8, 0]
        .iter()
        .map(|offset| {
            let channel = |colour: u32| i64::from((colour >> offset) & 0xff);
            (channel(one) - channel(other)).abs()
        })
        .sum()
}

#[test]
fn a_storm_keeps_the_faction_that_holds_the_ground() {
    // The overlay used to cover the finished pixel at a weight near the
    // holder weight, in a colour no faction uses, so a stormed holding lost
    // its colour. The air now reaches the ground before the holder takes its
    // share.[^3]
    //
    // [^3]: Research report 24, defect 4. `docs/research/reports/24-demonstration-readability-resources-and-weather.md`
    let (mut stormy, _) = a_held_band();
    let dry = stormy.clone();
    let place = a_held_tile(&stormy);
    stormy
        .inflict_weather(FactionId(0), &[place], STRENGTH)
        .expect("the faction holds the ground it storms");
    stormy.step(1).expect("the step must run");
    let mut settled = dry.clone();
    settled.step(1).expect("the step must run");

    // **The overlay reads the share of the sky and not the drops.** The
    // capacity of a cell follows its temperature, so one quantity of drops
    // fills a cold sky and leaves a warm one clear.
    let air = stormy.cloud_share_at(place).unwrap_or(0);
    let weight = paint::air_weight(air);
    assert!(
        weight >= paint::air_least_weight(),
        "the storm must put enough water over the tile to draw: {air} of a whole sky"
    );

    let camera = camera_over(&settled, place);
    let before = corner_of(&drawn(&settled, place), camera, place);
    let after = corner_of(&drawn(&stormy, place), camera, place);
    assert_ne!(before, after, "the storm must change the tile at {place:?}");

    // The colour the old order gave: the overlay mixed over the finished
    // pixel. The picture must not give it, and it must keep more of the
    // world than it did.
    let overlaid = paint::mixed(before, paint::air_colour(), weight);
    assert_ne!(
        after, overlaid,
        "the air must reach the ground before the holder mix at {place:?}"
    );
    assert!(
        apart(after, paint::air_colour()) > apart(overlaid, paint::air_colour()),
        "a storm must leave more of the holding than an overlay on the \
         finished pixel: {after:06x} against {overlaid:06x}"
    );
}

#[test]
fn the_air_overlay_is_off_below_eight_pixels_a_tile() {
    // A layer that covers every tile of the picture carries no information
    // and costs contrast, so the overlay is off at the region scale.[^4]
    //
    // [^4]: Research report 23, defect 2. `docs/research/reports/23-demonstration-readability-review-1.md`
    let (mut stormy, _) = a_held_band();
    let mut settled = stormy.clone();
    let place = a_held_tile(&stormy);
    stormy
        .inflict_weather(FactionId(0), &[place], STRENGTH)
        .expect("the faction holds the ground it storms");
    stormy.step(1).expect("the step must run");
    settled.step(1).expect("the step must run");
    assert!(
        paint::air_weight(stormy.cloud_share_at(place).unwrap_or(0)) >= paint::air_least_weight(),
        "the storm must put enough water over the tile to draw"
    );

    let small = 6.0;
    let large = 10.0;
    let read = |world: &World, tile: f32| {
        corner_of(
            &drawn_at(world, place, tile),
            camera_at(world, place, tile),
            place,
        )
    };
    // The ground colour of a tile does not depend on the zoom, so a world
    // with no storm draws the same colour at both sizes. That is what makes
    // the difference in the stormed world the overlay and nothing else.
    assert_eq!(
        read(&settled, small),
        read(&settled, large),
        "a world with no storm must draw one colour at both sizes"
    );
    assert_ne!(
        read(&stormy, small),
        read(&stormy, large),
        "the overlay must be off at {small} pixels a tile and on at {large}"
    );
}

#[test]
fn a_cell_at_rest_draws_no_air() {
    // The air over a cell at rest gives a weight of a few parts in 255. A
    // cell that still tinted its tiles put an edge on the cell lattice that
    // followed nothing in the world.[^5]
    //
    // [^5]: Research report 24, defect 10. `docs/research/reports/24-demonstration-readability-resources-and-weather.md`
    let (mut world, _) = a_held_band();
    let place = a_held_tile(&world);
    world
        .inflict_weather(FactionId(0), &[place], STRENGTH)
        .expect("the faction holds the ground it storms");

    // The air falls tick by tick. The moment this test needs is the one at
    // which the weight is above zero and below the floor, because that is
    // the band the floor exists for.
    let mut found = false;
    for _ in 0..THINNING_TICKS {
        world.step(1).expect("the step must run");
        let weight = paint::air_weight(world.cloud_share_at(place).unwrap_or(0));
        if weight > 0 && weight < paint::air_least_weight() {
            found = true;
            break;
        }
    }
    assert!(
        found,
        "the fixture must reach a tick at which the air is thin and not gone"
    );

    // The overlay is off below eight pixels a tile, so the colour there is
    // the colour of the tile with no air. A large tile must give the same
    // colour, because a weight under the floor draws nothing.
    let read = |tile: f32| {
        corner_of(
            &drawn_at(&world, place, tile),
            camera_at(&world, place, tile),
            place,
        )
    };
    assert_eq!(
        read(6.0),
        read(32.0),
        "a cell at rest must draw no overlay at any zoom"
    );
}

#[test]
fn a_building_site_changes_the_tile_it_stands_on() {
    let (world, soldiers) = a_held_band();
    let mut building = world.clone();
    let mut idle = world;
    // The first soldier that stands on ground its faction holds is the
    // builder. It is ordered in one world and left alone in the other.
    let (builder, _) = soldiers
        .iter()
        .copied()
        .find(|(soldier, _)| {
            building
                .soldiers()
                .address(*soldier)
                .and_then(|at| building.tile_holder(at))
                .and_then(Holder::faction)
                == Some(FactionId(0))
        })
        .expect("a soldier stands on held ground");
    // A road takes a tile the faction's plan zones, so the fixture zones the
    // tile the builder stands on. Without a project the engine refuses the
    // order, and this test would measure the refusal rather than the mark.
    let stands = building
        .soldiers()
        .address(builder)
        .expect("the builder stands on a tile of the world");
    building
        .zone_project(FactionId(0), stands, UpgradeCategory::ROAD)
        .expect("the plan takes the project");
    assert!(building.order_build(builder, UpgradeCategory::ROAD).is_ok());
    building.step(1).expect("the step must run");
    idle.step(1).expect("the step must run");

    let site = building
        .upgrade_sites()
        .first()
        .copied()
        .expect("the ordered build placed a site");
    assert!(
        idle.upgrade_sites().is_empty(),
        "the idle world built something",
    );
    let address = address_of(&building, site.tile.0);
    same_ground(&idle, &building, address);

    let camera = camera_over(&idle, address);
    let before = corner_of(&drawn(&idle, address), camera, address);
    let after = corner_of(&drawn(&building, address), camera, address);
    assert_ne!(
        before, after,
        "a building site did not change the pixel at {address:?}"
    );
}

/// The size of a tile in the pictures that read a build site, in pixels.
///
/// The glyph of a site sits in the middle of its tile, and so does the disc
/// of a unit. A tile this wide leaves a corner of the glyph outside the
/// disc, so a test reads the mark and not the builder standing on it.
const SITE_TILE: f32 = 32.0;

/// Returns a pixel inside the glyph of a build site.
///
/// The position lies outside the disc of a unit on the same tile, at the
/// tile width the site tests draw at.
fn site_pixel(canvas: &Canvas, camera: Camera, address: Axial) -> u32 {
    let (left, top, wide, tall) = tile_rect(camera, address);
    // The glyph fills the middle half of the tile, so its corner sits a
    // quarter of the way in. One pixel further in is inside the glyph.
    pixel(canvas, left + wide / 4 + 1, top + tall / 4 + 1)
}

/// Returns the first soldier standing on ground its own faction holds.
fn a_builder(world: &World, soldiers: &[(Entity, Axial)]) -> Entity {
    soldiers
        .iter()
        .copied()
        .find(|(soldier, _)| {
            world
                .soldiers()
                .address(*soldier)
                .and_then(|at| world.tile_holder(at))
                .and_then(Holder::faction)
                == Some(FactionId(0))
        })
        .expect("a soldier stands on held ground")
        .0
}

#[test]
fn a_site_under_work_marks_the_middle_and_a_finished_site_washes_the_tile() {
    // A site under work drew as a wash whose weight ran from a floor of 56.
    // Over a tinted tile that changes nothing a person can see, so a store
    // two work units into its forty-eight was absent and not weak.[^6]
    //
    // [^6]: Research report 25, defect 1. `docs/research/reports/25-demonstration-readability-upgrades-and-units.md`
    let (world, soldiers) = a_held_band();
    let mut begun = world.clone();
    let mut finished = world;
    let builder = a_builder(&begun, &soldiers);
    // A road takes a tile the faction's plan zones, so each world zones the
    // tile the builder stands on. Without a project the engine refuses the
    // order, and this test would measure the refusal rather than the marks.
    let stands = begun
        .soldiers()
        .address(builder)
        .expect("the builder stands on a tile of the world");
    begun
        .zone_project(FactionId(0), stands, UpgradeCategory::ROAD)
        .expect("the plan takes the project");
    finished
        .zone_project(FactionId(0), stands, UpgradeCategory::ROAD)
        .expect("the plan takes the project");
    assert!(begun.order_build(builder, UpgradeCategory::ROAD).is_ok());
    assert!(finished.order_build(builder, UpgradeCategory::ROAD).is_ok());
    begun.step(1).expect("the step must run");
    let site = begun
        .upgrade_sites()
        .first()
        .copied()
        .expect("the ordered build placed a site");
    assert!(!site.is_complete(), "one tick finished the road");
    let address = address_of(&begun, site.tile.0);

    let mut ticks = 0;
    while finished
        .upgrade_at(address)
        .is_none_or(|site| !site.is_complete())
    {
        assert!(ticks < 200, "the road was never finished");
        finished.step(1).expect("the step must run");
        ticks += 1;
    }

    // Each world is drawn as it is and again with the site destroyed, so the
    // difference is the mark of the site and never the tick.
    let mut bare_begun = begun.clone();
    let mut bare_finished = finished.clone();
    assert!(bare_begun.destroy_upgrade(address));
    assert!(bare_finished.destroy_upgrade(address));
    let camera = camera_at(&begun, address, SITE_TILE);
    let read = |world: &World, at: fn(&Canvas, Camera, Axial) -> u32| {
        at(&drawn_at(world, address, SITE_TILE), camera, address)
    };

    // A site under work marks the middle of its tile and leaves the corner.
    assert_ne!(
        read(&begun, site_pixel),
        read(&bare_begun, site_pixel),
        "a site under work did not mark the middle of its tile"
    );
    assert_eq!(
        read(&begun, corner_of),
        read(&bare_begun, corner_of),
        "a site under work washed the whole tile"
    );

    // A finished site washes the whole tile, corner included.
    assert_ne!(
        read(&finished, corner_of),
        read(&bare_finished, corner_of),
        "a finished site did not wash its tile"
    );
}

#[test]
fn each_kind_of_build_site_draws_its_own_glyph() {
    // The four kinds separated unevenly as washes, and none of them read as
    // a thing that somebody made.[^6]
    let (world, soldiers) = a_held_band();
    let builder = a_builder(&world, &soldiers);
    let address = world
        .soldiers()
        .address(builder)
        .expect("the builder stands somewhere");

    // A tile outside the band, which no unit of the fixture stands on.
    let aside = (0..EXTENT as i32)
        .map(|column| Axial::new(column, 0))
        .find(|at| world.admits_a_unit(*at))
        .expect("the world holds open ground outside the band");

    let bare_camera = camera_at(&world, address, SITE_TILE);
    let bare_corner = corner_of(&drawn_at(&world, address, SITE_TILE), bare_camera, address);

    // The table holds no row for every category on every ground. A category
    // the ground under the builder does not fit is refused by the engine, and
    // the open category holds no row at all, so the fixture asks the table
    // rather than assuming the whole set is buildable.
    let ground = world
        .tile_kind(address)
        .expect("the builder stands on a tile of the world");
    let buildable: Vec<UpgradeCategory> = UpgradeCategory::ALL
        .into_iter()
        .filter(|category| {
            world
                .upgrade_table()
                .row(*category, 1)
                .is_some_and(|row| row.fits(ground))
        })
        .collect();
    assert!(
        buildable.len() > 1,
        "the ground {ground:?} fits fewer than two categories, so this test \
         compares nothing"
    );

    let mut colours = Vec::new();
    for kind in buildable {
        let mut building = world.clone();
        // A category whose row asks for no held ground takes a tile the
        // faction's plan zones, so the fixture zones this kind on this tile
        // before it orders the build. A project names one category, so the
        // loop zones the kind it is about to order rather than one kind for
        // the whole set.
        building
            .zone_project(FactionId(0), address, kind)
            .expect("the plan takes the project");
        assert!(building.order_build(builder, kind).is_ok());
        building.step(1).expect("the step must run");
        // The builder stands on the site, and its disc is wider than the
        // glyph, so it would cover the mark this test reads. The fixture
        // moves it aside before the picture is drawn.
        building
            .place_soldier(builder, aside)
            .expect("the tile aside admits the unit");
        building.rebuild_bridge(1).expect("the bridge rebuilds");
        assert!(
            building
                .upgrade_at(address)
                .is_some_and(|site| { site.category == kind && !site.is_complete() }),
            "the order for {kind:?} placed no site under work at {address:?}"
        );
        let camera = camera_at(&building, address, SITE_TILE);
        let canvas = drawn_at(&building, address, SITE_TILE);
        // The glyph of each kind fills a different set of pixels, so the
        // whole tile tells the four apart. One row does not: the bar of a
        // road and the middle of a diamond cover the same pixels.
        // A glyph leaves the ground around it. A wash covers the corner as
        // well, and that is the mark this test refuses.
        assert_eq!(
            corner_of(&canvas, camera, address),
            bare_corner,
            "the mark of {kind:?} washed the whole tile"
        );
        let (left, top, wide, tall) = tile_rect(camera, address);
        let block: Vec<u32> = (top..top + tall)
            .flat_map(|row| (left..left + wide).map(move |column| (column, row)))
            .map(|(column, row)| pixel(&canvas, column, row))
            .collect();
        colours.push((kind, block));
    }
    for (first, one) in colours.iter().enumerate() {
        for other in colours.iter().skip(first + 1) {
            assert_ne!(
                one.1, other.1,
                "the glyph of {:?} draws as the glyph of {:?}",
                one.0, other.0
            );
        }
    }
}

#[test]
fn a_luxury_marks_the_tile_that_holds_it() {
    let bare = World::new(settings()).expect("the extent describes a world");
    let mut rich = bare.clone();
    let place = Axial::new(10, 10);
    let tile = rich.grid().index_of(place).expect("the place is inside");
    rich.seed_luxuries(&[(tile, LuxuryId(0))])
        .expect("one placement seeds");
    assert!(!rich.luxuries_at(tile).is_empty());
    same_ground(&bare, &rich, place);

    let camera = camera_over(&bare, place);
    let (left, top, wide, tall) = tile_rect(camera, place);
    let middle = |canvas: &Canvas| pixel(canvas, left + wide / 2, top + tall / 2);
    let before = middle(&drawn(&bare, place));
    let after = middle(&drawn(&rich, place));
    assert_ne!(
        before, after,
        "a luxury did not change the middle of its tile at {place:?}"
    );
    // The mark sits inside the tile, so the corner still shows the ground.
    assert_eq!(
        corner_of(&drawn(&bare, place), camera, place),
        corner_of(&drawn(&rich, place), camera, place),
        "the mark covered the whole tile"
    );
}

#[test]
fn two_kinds_of_luxury_draw_in_two_colours() {
    // One colour stood for every kind, and the catalogue admits sixty-four,
    // so the mark said that a tile holds a luxury and never which one.[^7]
    //
    // [^7]: Research report 24, defect 7. `docs/research/reports/24-demonstration-readability-resources-and-weather.md`
    let bare = World::new(settings()).expect("the extent describes a world");
    let place = Axial::new(10, 10);
    let tile = bare.grid().index_of(place).expect("the place is inside");

    let mut first = bare.clone();
    first
        .seed_luxuries(&[(tile, LuxuryId(0))])
        .expect("one placement seeds");
    let mut second = bare;
    second
        .seed_luxuries(&[(tile, LuxuryId(3))])
        .expect("one placement seeds");

    let camera = camera_over(&first, place);
    let (left, top, wide, tall) = tile_rect(camera, place);
    let middle = |canvas: &Canvas| pixel(canvas, left + wide / 2, top + tall / 2);
    assert_ne!(
        middle(&drawn(&first, place)),
        middle(&drawn(&second, place)),
        "two kinds of luxury drew in one colour at {place:?}"
    );
}

#[test]
fn a_deposit_marks_the_corner_of_its_tile() {
    // The ground colour carried the food alone, so a deposit of wood or of
    // stone changed no pixel.[^8]
    //
    // [^8]: Research report 24, defect 5. `docs/research/reports/24-demonstration-readability-resources-and-weather.md`
    let world = World::new(settings()).expect("the extent describes a world");
    let grid = world.grid();
    let stocked = |address: Axial, kind: ResourceKind| {
        world.tile_stock(address, kind).map_or(0, |stock| stock.0)
    };
    let mut carrying = None;
    let mut without = None;
    for index in 0..grid.tile_count() {
        let address = address_of(&world, index);
        if world.tile_kind(address).is_none() {
            continue;
        }
        if stocked(address, ResourceKind::Stone) > 0 {
            carrying.get_or_insert(address);
        } else {
            without.get_or_insert(address);
        }
    }
    let carrying = carrying.expect("the world holds a tile that carries stone");
    let without = without.expect("the world holds a tile that carries no stone");

    // The stone pip takes the lower left corner of the tile, inside the
    // margin the drawing keeps.
    let read = |address: Axial| {
        let camera = camera_at(&world, address, SITE_TILE);
        let canvas = drawn_at(&world, address, SITE_TILE);
        let (left, top, wide, tall) = tile_rect(camera, address);
        pixel(&canvas, left + wide / 5, top + tall - tall / 5)
    };
    assert_eq!(
        read(carrying),
        paint::resource_pip_colour(ResourceKind::Stone),
        "a tile that carries stone must mark its corner at {carrying:?}"
    );
    assert_ne!(
        read(without),
        paint::resource_pip_colour(ResourceKind::Stone),
        "a tile that carries no stone must mark nothing at {without:?}"
    );
}

/// What a fixture builds toward on one tile.
///
/// The work is the work that must stand in the entry when the fixture stops.
/// It separates a site that stands at a level from a site of the same level
/// that somebody is raising to the next one.
#[derive(Clone, Copy)]
struct Target {
    /// The category the builder is ordered to build.
    category: UpgradeCategory,
    /// The level that must stand on the tile.
    level: u8,
    /// The work that must stand toward the level above.
    work: i64,
}

impl Target {
    /// Returns a target of one level with no work toward the next.
    const fn standing(category: UpgradeCategory, level: u8) -> Self {
        Self {
            category,
            level,
            work: 0,
        }
    }

    /// Reports whether a world has reached this target on one tile.
    fn reached(self, world: &World, address: Axial) -> bool {
        world.upgrade_at(address).is_some_and(|site| {
            site.level > self.level || (site.level == self.level && site.progress.0 >= self.work)
        })
    }
}

/// Builds two worlds that differ in one tile and in nothing else.
///
/// **Both worlds step the same number of ticks.** A world stopped at an
/// earlier tick than the other would differ in the weather, in the holding
/// and in where every unit stands, and the picture of the tile would then
/// carry those differences as well as the site. Each world stops its own
/// build when it reaches its target and keeps stepping.
///
/// The builder is moved off the tile in both worlds, because the disc of a
/// unit is wider than the mark of a site and would cover it.
///
/// Returns the two worlds and the tile they built on.
fn two_sites(one: Target, other: Target) -> (World, World, Axial) {
    let (world, soldiers) = a_held_band();
    let builder = a_builder(&world, &soldiers);
    let address = world
        .soldiers()
        .address(builder)
        .expect("the builder stands on a tile of the world");
    // A tile outside the band, which no unit of the fixture stands on.
    let aside = (0..EXTENT as i32)
        .map(|column| Axial::new(column, 0))
        .find(|at| world.admits_a_unit(*at))
        .expect("the world holds open ground outside the band");
    let mut first = world.clone();
    let mut second = world;
    let mut builders = [builder, builder];
    // A plan holds one project for each tile, so each world zones the
    // category it is about to build and never the other one.
    for (world, target) in [(&mut first, one), (&mut second, other)] {
        world
            .zone_project(FactionId(0), address, target.category)
            .expect("the plan takes the project");
    }
    let mut ticks = 0;
    loop {
        let done = [
            one.reached(&first, address),
            other.reached(&second, address),
        ];
        // **A builder stands on the tile and takes the order each tick.** A
        // unit that finished a level walks away, a unit that walked away adds
        // no work, and a unit of this fixture can die of what the band does
        // to it. A fixture that ordered once therefore waits for ever at the
        // level it reached.
        for (index, world) in [&mut first, &mut second].into_iter().enumerate() {
            if done[index] {
                world.stop_build(builders[index]);
                continue;
            }
            let target = if index == 0 { one } else { other };
            builders[index] = press_a_builder(world, builders[index], address, target.category);
        }
        if done[0] && done[1] {
            break;
        }
        assert!(ticks < 400, "the fixture never reached both targets");
        first.step(1).expect("the step must run");
        second.step(1).expect("the step must run");
        ticks += 1;
    }
    for (index, world) in [&mut first, &mut second].into_iter().enumerate() {
        world
            .place_soldier(builders[index], aside)
            .expect("the tile aside admits the unit");
        world.rebuild_bridge(1).expect("the bridge rebuilds");
    }
    (first, second, address)
}

/// Puts a live builder of the first faction on one tile and orders the build.
///
/// Returns the builder, which is the one the caller named while it lives and
/// a new unit once it dies.
fn press_a_builder(
    world: &mut World,
    builder: Entity,
    address: Axial,
    category: UpgradeCategory,
) -> Entity {
    let builder = match world.soldiers().address(builder) {
        Some(_) => builder,
        None => world
            .spawn_soldier(address, FactionId(0))
            .expect("the tile admits a unit"),
    };
    world
        .place_soldier(builder, address)
        .expect("the tile admits the builder");
    world.rebuild_bridge(1).expect("the bridge rebuilds");
    world
        .order_build(builder, category)
        .expect("the engine takes the order");
    builder
}

/// Returns every pixel of one tile, in row order.
fn tile_block(canvas: &Canvas, camera: Camera, address: Axial) -> Vec<u32> {
    let (left, top, wide, tall) = tile_rect(camera, address);
    (top..top + tall)
        .flat_map(|row| (left..left + wide).map(move |column| (column, row)))
        .map(|(column, row)| pixel(canvas, column, row))
        .collect()
}

/// Asserts that two worlds draw one tile apart, and that they draw it alike
/// when the site is taken off it.
///
/// The second half is what makes the first half mean something. Two worlds
/// that differ anywhere else at that tile would draw it apart whatever the
/// site did.
fn the_site_is_the_only_difference(one: &World, other: &World, address: Axial, why: &str) {
    let camera = camera_at(one, address, SITE_TILE);
    let mut bare_one = one.clone();
    let mut bare_other = other.clone();
    assert!(bare_one.destroy_upgrade(address));
    assert!(bare_other.destroy_upgrade(address));
    assert_eq!(
        tile_block(&drawn_at(&bare_one, address, SITE_TILE), camera, address),
        tile_block(&drawn_at(&bare_other, address, SITE_TILE), camera, address),
        "the two worlds differ at {address:?} in more than the site, so {why} \
         proves nothing"
    );
    assert_ne!(
        tile_block(&drawn_at(one, address, SITE_TILE), camera, address),
        tile_block(&drawn_at(other, address, SITE_TILE), camera, address),
        "{why}"
    );
}

#[test]
fn two_levels_of_one_category_draw_apart() {
    // **The level had no channel of its own.** The wash carries the build
    // progress, and a road that stands at level 1 with no work toward level 2
    // washes at the same weight as a road at the top of its category, so the
    // two drew one picture.[^7]
    //
    // [^7]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D5. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    let (lower, higher, address) = two_sites(
        Target::standing(UpgradeCategory::ROAD, 1),
        Target::standing(UpgradeCategory::ROAD, 2),
    );
    assert_eq!(
        lower.upgrade_at(address).map(|site| site.level),
        Some(1),
        "the lower world does not stand at level 1"
    );
    assert_eq!(
        higher.upgrade_at(address).map(|site| site.level),
        Some(2),
        "the higher world does not stand at level 2"
    );
    the_site_is_the_only_difference(
        &lower,
        &higher,
        address,
        "a level 1 road draws as a level 2 road",
    );
}

#[test]
fn two_categories_at_one_level_draw_apart() {
    let (road, terrace, address) = two_sites(
        Target::standing(UpgradeCategory::ROAD, 1),
        Target::standing(UpgradeCategory::TERRACE, 1),
    );
    for (world, category) in [
        (&road, UpgradeCategory::ROAD),
        (&terrace, UpgradeCategory::TERRACE),
    ] {
        assert_eq!(
            world
                .upgrade_at(address)
                .map(|site| (site.category, site.level)),
            Some((category, 1)),
            "the fixture did not stand {category:?} at level 1"
        );
    }
    the_site_is_the_only_difference(
        &road,
        &terrace,
        address,
        "a level 1 road draws as a level 1 terrace",
    );
}

/// The work that stands in an entry that somebody is raising.
///
/// It is below the work of the level above, so the entry is still at the
/// lower level when the fixture stops.
const WORK_UNDER_WAY: i64 = 4;

#[test]
fn a_site_under_work_draws_apart_from_one_that_stands_at_the_same_level() {
    // A site under work and a site that stands are two states of one tile,
    // and the wash weight is what separates them at a level that stands.
    let (resting, rising, address) = two_sites(
        Target::standing(UpgradeCategory::ROAD, 1),
        Target {
            category: UpgradeCategory::ROAD,
            level: 1,
            work: WORK_UNDER_WAY,
        },
    );
    assert_eq!(
        resting.upgrade_at(address).map(|site| site.progress.0),
        Some(0),
        "the resting world holds work toward the level above"
    );
    assert!(
        rising
            .upgrade_at(address)
            .is_some_and(|site| site.level == 1 && site.progress.0 >= WORK_UNDER_WAY),
        "the rising world does not hold a level 1 road under work"
    );
    the_site_is_the_only_difference(
        &resting,
        &rising,
        address,
        "a road nobody is raising draws as a road somebody is raising",
    );
}

#[test]
fn every_category_the_table_holds_draws_a_shape_of_its_own() {
    // **A shape and not a colour.** Every category already draws in a colour
    // of its own, so a test that compares the pixels of two tiles passes
    // while two categories share one shape. This test compares the set of
    // pixels each mark covers, which is the shape alone. The wall and the
    // open category shared one shape under that reading.[^7]
    let (world, soldiers) = a_held_band();
    let builder = a_builder(&world, &soldiers);
    let address = world
        .soldiers()
        .address(builder)
        .expect("the builder stands somewhere");
    let aside = (0..EXTENT as i32)
        .map(|column| Axial::new(column, 0))
        .find(|at| world.admits_a_unit(*at))
        .expect("the world holds open ground outside the band");
    let ground = world
        .tile_kind(address)
        .expect("the builder stands on a tile of the world");
    let buildable: Vec<UpgradeCategory> = UpgradeCategory::ALL
        .into_iter()
        .filter(|category| {
            world
                .upgrade_table()
                .row(*category, 1)
                .is_some_and(|row| row.fits(ground))
        })
        .collect();
    assert!(
        buildable.len() > 1,
        "the ground {ground:?} fits fewer than two categories, so this test \
         compares nothing"
    );

    let camera = camera_at(&world, address, SITE_TILE);
    let bare = tile_block(&drawn_at(&world, address, SITE_TILE), camera, address);
    let mut shapes = Vec::new();
    for category in buildable {
        let mut building = world.clone();
        building
            .zone_project(FactionId(0), address, category)
            .expect("the plan takes the project");
        building
            .order_build(builder, category)
            .expect("the engine takes the order");
        building.step(1).expect("the step must run");
        building
            .place_soldier(builder, aside)
            .expect("the tile aside admits the unit");
        building.rebuild_bridge(1).expect("the bridge rebuilds");
        assert!(
            building
                .upgrade_at(address)
                .is_some_and(|site| site.category == category),
            "the order for {category:?} placed no site at {address:?}"
        );
        let block = tile_block(&drawn_at(&building, address, SITE_TILE), camera, address);
        // The shape is where the mark covers the ground, and never which
        // colour it covers it with.
        let shape: Vec<bool> = block
            .iter()
            .zip(bare.iter())
            .map(|(mark, ground)| mark != ground)
            .collect();
        assert!(
            shape.iter().any(|covered| *covered),
            "the mark of {category:?} covered nothing"
        );
        shapes.push((category, shape));
    }
    for (first, one) in shapes.iter().enumerate() {
        for other in shapes.iter().skip(first + 1) {
            assert_ne!(
                one.1, other.1,
                "the shape of {:?} draws as the shape of {:?}",
                one.0, other.0
            );
        }
    }
}

/// The value the upgrade overlay paints at a tile that carries nothing.
const NOTHING_BUILT: i64 = 0;

#[test]
fn the_upgrade_overlay_reads_the_level_and_not_the_category() {
    // **The overlay called the category ordinal a level.** A road is category
    // zero and a terrace is category one, so a level 2 road painted as the
    // first step of the ramp and a level 1 terrace painted as the second. The
    // overlay named the level in its own key and drew the category.[^7]
    let (road, terrace, address) = two_sites(
        Target::standing(UpgradeCategory::ROAD, 2),
        Target::standing(UpgradeCategory::TERRACE, 1),
    );
    let layer = overlay::named("upgrade").expect("the deck registers the upgrade overlay");
    let value = |world: &World, at: Axial| overlay::value_of(layer, world, at, None);

    assert_eq!(
        value(&road, address),
        i64::from(road.upgrade_at(address).expect("a road stands").level) + 1,
        "the overlay does not read the level of the entry"
    );
    assert_eq!(
        value(&road, address),
        3,
        "a level 2 road does not paint at the top of the ramp"
    );
    assert_eq!(
        value(&terrace, address),
        2,
        "a level 1 terrace does not paint at the first standing step"
    );
    assert!(
        value(&road, address) > value(&terrace, address),
        "the overlay does not put a level 2 road above a level 1 terrace"
    );

    // The span scales against the table and not against the category count.
    let span = layer.span(&road);
    assert_eq!(
        span.high,
        i64::try_from(UPGRADE_LEVEL_COUNT).expect("the level count is small") + 1,
        "the overlay does not scale against the level count of the table"
    );

    // A tile that carries nothing paints nothing, so a watcher tells a bare
    // tile from a first build.
    let mut bare = road.clone();
    assert!(bare.destroy_upgrade(address));
    assert_eq!(
        value(&bare, address),
        NOTHING_BUILT,
        "the overlay paints a tile that carries no upgrade"
    );
}
