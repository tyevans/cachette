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
use cachette_core::upgrade::UpgradeKind;
use cachette_core::{Axial, Entity, FactionId, Holder, World, WorldConfig};
use cachette_view::hud::TileReadout;
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

/// The ticks a storm is given to fall out of the air onto the ground.
///
/// After this many ticks the air over every cell holds too few drops for
/// the overlay to change a pixel, and the ground is wet. The wet ground
/// test therefore sees the wet layer alone.
const FALLING_TICKS: u32 = 60;

/// The strength of the storm the fixture inflicts.
const STRENGTH: u8 = 4;

/// The most drops in the air over a cell that draw no overlay.
///
/// The overlay saturates at a count of drops the viewer chose, and it
/// deepens in whole steps from there. Below this many drops the step is
/// zero. The figure is a property of the fixture: the wet ground test waits
/// until the air has fallen this far, so that the wet layer is the sole
/// difference it sees.
const AIR_TOO_THIN_TO_SHOW: i64 = 20;

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
/// Returns the world and the soldiers it spawned, in spawn order.
fn a_held_band() -> (World, Vec<(Entity, Axial)>) {
    let mut world = World::new(settings()).expect("the extent describes a world");
    let mut soldiers = Vec::new();
    for row in BAND.0..BAND.1 {
        for column in BAND.0..BAND.1 {
            let at = Axial::new(column, row);
            if !world.admits_a_unit(at) {
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

/// Returns one tile the faction holds.
fn a_held_tile(world: &World) -> Axial {
    let held = world
        .holding()
        .tiles_held_by(FactionId(0))
        .next()
        .expect("the faction holds a tile");
    let width = world.grid().width();
    Axial::new((held.0 % width) as i32, (held.0 / width) as i32)
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
        // The air over the cell must hold too few drops to show, so the
        // wet layer is the sole difference. The air overlay moves all three
        // channels toward one pale colour, and the assertion below refuses
        // that: it holds the red and the green exactly where they were.
        if stormy.air_at(address).unwrap_or(0) > AIR_TOO_THIN_TO_SHOW {
            continue;
        }
        compared = Some(address);
        break;
    }
    let address = compared.expect("the world holds a wet tile nobody holds or stands on");
    same_ground(&dry, &stormy, address);

    let camera = camera_over(&dry, address);
    let before = corner_of(&drawn(&dry, address), camera, address);
    let after = corner_of(&drawn(&stormy, address), camera, address);
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

    let air = stormy.air_at(place).unwrap_or(0);
    let weight = paint::air_weight(air);
    assert!(
        weight >= paint::air_least_weight(),
        "the storm must put enough water over the tile to draw: {air} drops"
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
        paint::air_weight(stormy.air_at(place).unwrap_or(0)) >= paint::air_least_weight(),
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
    for _ in 0..FALLING_TICKS {
        world.step(1).expect("the step must run");
        let weight = paint::air_weight(world.air_at(place).unwrap_or(0));
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
    assert!(building.order_build(builder, UpgradeKind::Road));
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
    pixel(canvas, left + wide / 6, top + tall / 6)
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
    assert!(begun.order_build(builder, UpgradeKind::Road));
    assert!(finished.order_build(builder, UpgradeKind::Road));
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

    let bare_camera = camera_at(&world, address, SITE_TILE);
    let bare_corner = corner_of(&drawn_at(&world, address, SITE_TILE), bare_camera, address);

    let mut colours = Vec::new();
    for kind in UpgradeKind::ALL {
        let mut building = world.clone();
        assert!(building.order_build(builder, kind));
        building.step(1).expect("the step must run");
        assert!(
            building
                .upgrade_at(address)
                .is_some_and(|site| { site.kind == kind && !site.is_complete() }),
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
