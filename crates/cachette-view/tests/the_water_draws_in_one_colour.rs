//! Open water carries one treatment over the whole map.
//!
//! The viewer used to shade a water tile the way it shades a hill. The height
//! of the ground brightened it, so the sea held a spread of brightness steps
//! that a watcher read as depth. The wet ground field then added blue over
//! part of that sea and not the rest. Two tiles of open water therefore drew
//! apart for two reasons, neither of which is a fact about the water.
//!
//! The depth is not a fact a player can act on. The terrain capacity table
//! gives every water tile the same capacity, so a unit that carries a water
//! crossing crosses the deepest tile and the shallowest one alike.[^1] A test
//! below asserts that, so a later change that makes the depth matter fails
//! here and tells the reader to bring the shading back.
//!
//! **Each fixture states the spread it supplies.** A sea of one depth cannot
//! show that the treatment is the same at every depth, and a sea that no rain
//! reaches cannot show that the wet field stops at the water line. An
//! assertion over a flat fixture measures the fixture.[^2]
//!
//! The tests drive the drawing pass and read the frame it filled. A test that
//! called the colour function would prove the function works and not that a
//! frame reaches it.[^3]
//!
//! The tests see only the public crate API.
//!
//! # References
//!
//! [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
//! [^2]: Testing rules, section 2a. `.agents/rules/testing.md`
//! [^3]: Testing rules, section 5. `.agents/rules/testing.md`

// An integration test is its own crate, so the allowance at the viewer's
// crate root does not reach it. The tile width of a camera is a viewer value,
// and the record puts the float boundary at the viewer.[^4]
//
// [^4]: ADR-0067, the viewer reads the world and never writes to it, decision D3. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
#![allow(clippy::disallowed_types)]

use std::collections::{BTreeMap, BTreeSet};

use cachette_core::terrain::{TileKind, NO_WATER_CROSSING};
use cachette_core::{Axial, World, WorldConfig};
use cachette_view::paint::kind_colour;
use cachette_view::{paint, Camera, Canvas};

/// The extent of the fixture world.
///
/// The extent is wider than the coarsest lattice spacing of the ground
/// generator, so the world holds a sea rather than one kind of tile.
const EXTENT: u32 = 128;

/// The seed of the fixture world.
const SEED: u64 = 0x0cac_4e77_0032;

/// The width of a tile in the frame, in pixels.
///
/// The width is below the width at which the viewer washes a tile with the
/// water in the air over it, and a test below asserts that against the
/// reader the viewer states. The wash covers the land in the same colour at
/// the same weight, so it is a layer over the map and not a treatment of the
/// water. This suite is about the treatment, so it draws below the wash.
const TILE: f32 = 4.0;

/// The steps of ground height that the suite needs the sea to span.
///
/// The height used to give a water tile one brightness step for each part of
/// the range it reached. A fixture that supplies fewer steps than this cannot
/// show that the steps are gone.
const DEPTHS_NEEDED: usize = 8;

/// Builds the fixture world and runs it for the given number of ticks.
fn world_after(ticks: u32) -> World {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: SEED,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent describes a world");
    world.rebuild_bridge(1).expect("the rebuild must succeed");
    for _ in 0..ticks {
        world.step(2).expect("the step must run");
    }
    world
}

/// Returns every address of the world, in index order.
fn addresses_of(world: &World) -> Vec<Axial> {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .collect()
}

/// Draws the world and returns the frame.
fn painted(world: &World) -> (Camera, Canvas<'static>) {
    let mut canvas = Canvas::new(768, 768);
    let camera = Camera::at_tile_size(TILE);
    paint::draw(world, camera, &mut canvas).expect("the world draws");
    (camera, canvas)
}

/// Returns the pixel at the centre of each water tile the frame holds, keyed
/// by the address of the tile.
///
/// A tile whose centre falls outside the canvas is left out, because the
/// frame holds no pixel for it.
fn water_pixels(world: &World, camera: Camera, canvas: &Canvas) -> BTreeMap<(i32, i32), u32> {
    let mut found = BTreeMap::new();
    for address in addresses_of(world) {
        let Some(ground) = world.tile_terrain(address) else {
            continue;
        };
        if ground.kind != TileKind::Water {
            continue;
        }
        let (x, y) = camera.centre_of(address);
        if x < 1.0
            || y < 1.0
            || x as usize + 1 >= canvas.width()
            || y as usize + 1 >= canvas.height()
        {
            continue;
        }
        let pixel = canvas.pixels()[y as usize * canvas.width() + x as usize];
        found.insert((address.q, address.r), pixel);
    }
    found
}

/// Returns the brightness step that the height of a tile used to give it.
///
/// The suite counts these to state the spread the fixture supplies. The step
/// count follows the height and not the palette, so this stays a statement
/// about the fixture even after the shading is gone.
fn depth_step(height: i32) -> i32 {
    (height.clamp(0, 0x0001_0000) * 56) >> 16
}

#[test]
fn the_frame_draws_below_the_weather_wash() {
    // The suite asserts that the water carries one colour. The weather wash
    // is a separate layer that crosses the land in the same colour, and it
    // would put a second colour on the sea at a close zoom. This states that
    // the frame is drawn below it, against the reader the viewer gives,
    // rather than against a number this file repeats.
    assert!(
        TILE < paint::air_least_tile(),
        "the fixture draws a tile {TILE} pixels wide, and the weather wash \
         starts at {}, so the suite would measure the wash",
        paint::air_least_tile()
    );
}

#[test]
fn the_depth_of_open_water_admits_no_unit_differently() {
    // The shading that this change removed carried the depth. Removing a
    // shade is only free when the shade carried nothing a player acts on.
    // The capacity table is where a unit meets the ground, so it is where
    // the question is settled.
    let world = world_after(0);
    let mut capacities = BTreeSet::new();
    for address in addresses_of(&world) {
        let Some(ground) = world.tile_terrain(address) else {
            continue;
        };
        if ground.kind != TileKind::Water {
            continue;
        }
        capacities.insert(ground.kind.capacity_for(NO_WATER_CROSSING));
        capacities.insert(ground.kind.capacity_for(1));
    }
    assert_eq!(
        capacities.len(),
        2,
        "open water gives {} capacities, so a depth may now carry a fact a \
         player acts on and the flat colour hides it",
        capacities.len()
    );
}

#[test]
fn the_dry_fixture_holds_water_at_many_depths() {
    // The fixture must supply the spread, or the assertion below measures a
    // sea that was already flat.
    let world = world_after(0);
    assert!(
        world.weather().is_dry(),
        "the fresh world already carries weather, so this fixture does not \
         isolate the depth"
    );
    let steps: BTreeSet<i32> = addresses_of(&world)
        .into_iter()
        .filter_map(|address| world.tile_terrain(address))
        .filter(|ground| ground.kind == TileKind::Water)
        .map(|ground| depth_step(ground.height.0))
        .collect();
    assert!(
        steps.len() >= DEPTHS_NEEDED,
        "the sea spans {} height steps, and the suite needs {DEPTHS_NEEDED}",
        steps.len()
    );
}

#[test]
fn every_water_tile_draws_in_one_colour_at_every_depth() {
    // The claim the owner asked for: one water treatment over the whole map.
    let world = world_after(0);
    let (camera, canvas) = painted(&world);
    let pixels = water_pixels(&world, camera, &canvas);
    assert!(
        pixels.len() > 256,
        "the frame holds {} water tiles, too few to test",
        pixels.len()
    );
    let seen: BTreeSet<u32> = pixels.values().copied().collect();
    assert_eq!(
        seen.len(),
        1,
        "the sea drew in {} colours: {:?}",
        seen.len(),
        seen.iter().take(8).collect::<Vec<_>>()
    );
    assert_eq!(
        seen.into_iter().next(),
        Some(kind_colour(TileKind::Water)),
        "the sea drew in a colour the palette does not give water"
    );
}

#[test]
fn the_wet_fixture_holds_wet_water_and_dry_water() {
    // The wet field is the second thing that used to break the sea into two
    // colours. A fixture whose sea is wet everywhere, or dry everywhere,
    // supplies no case.
    let world = world_after(40);
    assert!(
        !world.weather().is_dry(),
        "the fixture carries no weather, so the wet field cannot reach the sea"
    );
    let mut wet = 0;
    let mut dry = 0;
    for address in addresses_of(&world) {
        let Some(ground) = world.tile_terrain(address) else {
            continue;
        };
        if ground.kind != TileKind::Water {
            continue;
        }
        match world.ground_is_wet(address) {
            Some(true) => wet += 1,
            _ => dry += 1,
        }
    }
    assert!(
        wet > 0 && dry > 0,
        "the sea holds {wet} wet tiles and {dry} dry ones, so the fixture \
         cannot show that the wet field stops at the water line"
    );
}

#[test]
fn the_wet_field_does_not_reach_the_water() {
    // The same claim, over a world whose weather has run. The rain wets part
    // of the sea and not the rest, and the sea must still draw in one colour.
    let world = world_after(40);
    let (camera, canvas) = painted(&world);
    let pixels = water_pixels(&world, camera, &canvas);
    assert!(
        pixels.len() > 256,
        "the frame holds {} water tiles, too few to test",
        pixels.len()
    );
    let seen: BTreeSet<u32> = pixels.values().copied().collect();
    assert_eq!(
        seen.len(),
        1,
        "the sea of a rained-on world drew in {} colours: {:?}",
        seen.len(),
        seen.iter().take(8).collect::<Vec<_>>()
    );
}
