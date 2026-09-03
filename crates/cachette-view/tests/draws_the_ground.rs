//! The viewer shows what the ground is.
//!
//! The engine generates a kind and a height for every tile. Until this suite
//! existed, the viewer painted every tile the same, so a person could not tell
//! one part of the world from another. That is the first thing the product
//! record asks for.[^1]
//!
//! The colours and the names are the viewer's own. The engine numbers the
//! kinds and says nothing about what to call one or how to paint it.[^2]
//!
//! Every fixture states what ground it put in the window. A window over one
//! kind of ground supplies no case, and an assertion about five kinds then
//! measures the fixture.[^3] A world narrower than the coarsest lattice
//! spacing of the generator holds one kind only, so every fixture here is
//! wider than that spacing.[^4]
//!
//! The tests see only the public crate API.
//!
//! # References
//!
//! [^1]: PRD-0003, a developer sees a world worth looking at. `docs/product/accepted/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
//! [^2]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
//! [^3]: Testing rules, section 2a. `.claude/rules/testing.md`
//! [^4]: Findings register, FND-054. `docs/FINDINGS.md`

use std::collections::BTreeSet;

use cachette_core::resource::ResourceKind;
use cachette_core::terrain::{TileKind, KIND_COUNT};
use cachette_core::{Axial, World, WorldConfig};
use cachette_view::paint::kind_colour;
use cachette_view::{paint, Camera, Canvas};

/// The extent of the fixture world.
///
/// The extent is wider than the coarsest lattice spacing of the generator, so
/// the world holds every kind of ground rather than one of them.
const EXTENT: u32 = 128;

/// The kinds, in the order the engine numbers them.
const KINDS: [TileKind; KIND_COUNT] = [
    TileKind::Water,
    TileKind::Plain,
    TileKind::Forest,
    TileKind::Hill,
    TileKind::Mountain,
];

/// Builds the fixture world of one seed.
fn world_of(seed: u64) -> World {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent describes a world");
    // A draw reads the derived structure, and a fresh one is stale.
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

/// Paints the whole world onto a canvas and returns both.
fn painted(seed: u64) -> (World, Canvas<'static>) {
    let world = world_of(seed);
    let mut canvas = Canvas::new(512, 512);
    let camera = Camera::fitting(&world, &canvas);
    paint::draw(&world, camera, &mut canvas).expect("the world draws");
    (world, canvas)
}

#[test]
fn the_engine_numbers_the_kinds_and_the_viewer_reads_the_same_order() {
    // The kind number is one fact in two places: the engine assigns it and
    // the viewer indexes a palette by it. Nothing fails when the two drift,
    // because a wrong colour is still a colour. This is the check.
    for (ordinal, kind) in KINDS.iter().enumerate() {
        assert_eq!(
            kind.to_u8() as usize,
            ordinal,
            "the kind {kind:?} sits at {ordinal} and numbers itself {}",
            kind.to_u8()
        );
    }
}

#[test]
fn each_kind_paints_a_colour_of_its_own() {
    // A palette that gives two kinds one colour tells the person that the
    // world holds fewer kinds than it does.
    let colours: BTreeSet<u32> = KINDS.iter().copied().map(kind_colour).collect();
    assert_eq!(
        colours.len(),
        KIND_COUNT,
        "the palette gives {} colours to {KIND_COUNT} kinds",
        colours.len()
    );
}

#[test]
fn the_fixture_window_holds_every_kind_of_ground() {
    // Every assertion below reads a picture. A picture of one kind of ground
    // proves nothing about five, so the fixture states what it holds.
    let (_, canvas) = painted(0x0cac_4e77_0032);
    let counts = canvas.painted_by_kind();
    for (ordinal, kind) in KINDS.iter().enumerate() {
        assert!(
            counts[ordinal] > 0,
            "the window holds no {kind:?}, so the picture cannot show one"
        );
    }
}

#[test]
fn no_kind_covers_the_whole_window() {
    // The product record asks that every kind occupy a part of the world and
    // that no kind cover everything. A generator that floods or drains a
    // world passes every colour test and fails this one.
    let (_, canvas) = painted(0x0cac_4e77_0032);
    let counts = canvas.painted_by_kind();
    let painted_total: u32 = counts.iter().sum();
    assert_eq!(
        painted_total,
        canvas.tiles_painted(),
        "the counts by kind do not sum to the tiles painted"
    );
    for (ordinal, kind) in KINDS.iter().enumerate() {
        assert!(
            counts[ordinal] < painted_total,
            "the kind {kind:?} covers the whole window"
        );
    }
}

#[test]
fn a_tile_is_painted_the_colour_of_its_kind() {
    // The picture must agree with the ground the engine reports for the same
    // address. A viewer that paints a pretty picture of another world is the
    // failure this catches.
    // The whole world is in the window. An earlier version of this test read
    // one corner at a large zoom, and that corner held water alone: the test
    // passed against a palette that gave every kind one colour, because every
    // tile it read was the same kind.
    let world = world_of(0x0cac_4e77_0032);
    let mut canvas = Canvas::new(1024, 1024);
    let camera = Camera::fitting(&world, &canvas);
    paint::draw(&world, camera, &mut canvas).expect("the world draws");

    let mut checked = 0;
    let mut seen: BTreeSet<TileKind> = BTreeSet::new();
    for address in addresses_of(&world) {
        let (x, y) = camera.centre_of(address);
        if x < 8.0
            || y < 8.0
            || x as usize + 8 >= canvas.width()
            || y as usize + 8 >= canvas.height()
        {
            continue;
        }
        let kind = world.tile_kind(address).expect("the address names a tile");
        let painted = canvas.pixels()[y as usize * canvas.width() + x as usize];
        // The height shades a tile, and shading moves brightness and never
        // hue. The kind must therefore be recoverable from the hue alone,
        // whatever height the tile has.
        let nearest = KINDS
            .iter()
            .copied()
            .min_by_key(|candidate| hue_distance(kind_colour(*candidate), painted))
            .expect("the palette holds a kind");
        assert_eq!(
            nearest, kind,
            "the tile {address:?} is {kind:?} and was painted nearest to {nearest:?}"
        );
        checked += 1;
        seen.insert(kind);
    }
    assert!(
        checked > 32,
        "only {checked} tiles fell inside the canvas, too few to test"
    );
    assert_eq!(
        seen.len(),
        KIND_COUNT,
        "the window held {} of the {KIND_COUNT} kinds, so a palette that \
         gives every kind one colour would pass this test",
        seen.len()
    );
}

/// Returns the distance between the hues of two colours.
///
/// Each colour is moved so that its channels average zero, which drops the
/// brightness and keeps the balance between the channels. Two tiles of one
/// kind at two heights then sit at the same point, and two kinds sit apart
/// however tall either tile is.
fn hue_distance(first: u32, second: u32) -> u32 {
    let hue = |colour: u32| -> [i32; 3] {
        let channels = [0, 8, 16].map(|shift| ((colour >> shift) & 0xff) as i32);
        let mean = channels.iter().sum::<i32>() / 3;
        channels.map(|channel| channel - mean)
    };
    let (first, second) = (hue(first), hue(second));
    (0..3)
        .map(|index| first[index].abs_diff(second[index]))
        .sum()
}

#[test]
fn a_taller_tile_of_one_kind_is_painted_brighter() {
    // The height must reach the picture. A viewer that reads the kind and
    // drops the height paints flat bands and passes every kind test.
    // The whole world is in the window, so the search sees every plain tile
    // and not only the ones in one corner.
    let world = world_of(0x0cac_4e77_0032);
    let mut canvas = Canvas::new(1024, 1024);
    let camera = Camera::fitting(&world, &canvas);
    paint::draw(&world, camera, &mut canvas).expect("the world draws");

    // One kind, two tiles, the tallest and the shortest of that kind inside
    // the canvas. The pair must exist, or the test measures the fixture.
    //
    // The food on a tile brightens it too, and the food range is wider than
    // the height range of one kind in one world. The search therefore holds
    // the food at zero, so the only thing that separates the two tiles is
    // the height.[^1]
    //
    // [^1]: Backlog item 0188. `docs/backlog/complete/0188-show-the-food-of-a-tile-and-the-reason-a-unit-chose.md`
    let mut lowest: Option<(i32, u32)> = None;
    let mut highest: Option<(i32, u32)> = None;
    for address in addresses_of(&world) {
        let ground = world
            .tile_terrain(address)
            .expect("the address names a tile");
        if ground.kind != TileKind::Plain {
            continue;
        }
        if world
            .tile_stock(address, ResourceKind::Food)
            .is_none_or(|amount| amount.0 != 0)
        {
            continue;
        }
        let (x, y) = camera.centre_of(address);
        if x < 8.0
            || y < 8.0
            || x as usize + 8 >= canvas.width()
            || y as usize + 8 >= canvas.height()
        {
            continue;
        }
        let pixel = canvas.pixels()[y as usize * canvas.width() + x as usize];
        let seen = (ground.height.0, pixel);
        if lowest.is_none_or(|(height, _)| ground.height.0 < height) {
            lowest = Some(seen);
        }
        if highest.is_none_or(|(height, _)| ground.height.0 > height) {
            highest = Some(seen);
        }
    }

    let (low_height, low_pixel) = lowest.expect("the window holds a plain tile");
    let (high_height, high_pixel) = highest.expect("the window holds a plain tile");
    assert!(
        high_height > low_height,
        "every plain tile in the window has one height, so the shading is untested"
    );
    assert!(
        brightness_of(high_pixel) > brightness_of(low_pixel),
        "the taller plain tile at height {high_height} painted {high_pixel:08x}, \
         no brighter than the shorter one at {low_height}, which painted {low_pixel:08x}"
    );
}

/// Returns the brightness of a colour, summed over the channels.
fn brightness_of(colour: u32) -> u32 {
    (0..3).map(|channel| (colour >> (channel * 8)) & 0xff).sum()
}

#[test]
fn a_different_seed_paints_a_different_world() {
    // The seed is the whole input to the ground. Two seeds that paint one
    // picture mean the viewer is not reading the ground at all.
    let (_, first) = painted(0x0cac_4e77_0032);
    let (_, second) = painted(0x0cac_4e77_0033);
    assert_ne!(
        first.pixels(),
        second.pixels(),
        "two seeds painted the same picture"
    );
    assert_ne!(
        first.painted_by_kind(),
        second.painted_by_kind(),
        "two seeds put the same ground in the window"
    );
}

#[test]
fn the_ground_read_follows_the_window_and_not_the_world() {
    // The ground is computed on demand, so a sweep of the whole world every
    // frame is a design mistake that this test makes visible. A small window
    // must read a small share of the world.
    let world = world_of(0x0cac_4e77_0032);
    let mut small = Canvas::new(96, 96);
    let camera = Camera::at_tile_size(12.0);
    paint::draw(&world, camera, &mut small).expect("the world draws");

    let held = world.grid().tile_count();
    assert!(
        small.tiles_painted() < held / 8,
        "a 96 by 96 window painted {} tiles of the {held} the world holds",
        small.tiles_painted()
    );
    assert!(
        small.tiles_painted() > 0,
        "the window painted no tile at all"
    );
}
