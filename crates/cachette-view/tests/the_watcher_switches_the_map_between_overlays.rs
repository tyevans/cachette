//! The map shows one quantity at a time, and the caller chooses which.
//!
//! A research report measured that the drawing served none of the three
//! things the project owner asked for. The air layer was near absent at rest,
//! a storm painted one level 1 cell flat across its tiles, and the wet shade
//! cancelled the food shade exactly.[^1] The general answer is an overlay
//! deck: the map carries one quantity, and the watcher switches it.[^2]
//!
//! # The fixture
//!
//! The fixture is built for these cases and never copied from the
//! demonstration world. A world chosen to look right supplies no storm, no
//! crowd and no site, so an assertion over it never receives the input that
//! would fail it.[^3]
//!
//! Every registered overlay must be non-zero somewhere in the window, or the
//! test that says an overlay changes the picture measures the fixture rather
//! than the overlay.
//!
//! **Every test here draws below the width at which the resting air wash
//! draws.** The wash is the layer an overlay replaces, so a test above that
//! width could pass because the wash went away rather than because the
//! overlay arrived.
//!
//! # References
//!
//! [^1]: Research report 24, demonstration readability, resources and weather. `docs/research/reports/24-demonstration-readability-resources-and-weather.md`
//! [^2]: Backlog item 0494, register an overlay deck. `docs/backlog/complete/0494-register-an-overlay-deck-and-let-the-watcher-switch-the-map-between-info-views.md`
//! [^3]: Findings register, FND-051. `docs/FINDINGS.md`

// The camera and the tile size are viewer values. The lint that bans the
// float types protects simulated state, and no value formed from these
// returns to the engine.[^4]
//
// [^4]: ADR-0067, the viewer reads the world and never writes to it, decision D3. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
#![allow(clippy::disallowed_types)]

use cachette_core::upgrade::UpgradeCategory;
use cachette_core::{Axial, Entity, FactionId, World, WorldConfig};
use cachette_view::hud::{self, Readout};
use cachette_view::metrics::Metrics;
use cachette_view::overlay::{self, Layer};
use cachette_view::paint::{self, tile_rect};
use cachette_view::{Camera, Canvas, Overlay};

/// The side of the fixture world, in tiles.
///
/// The world is wider than several level 1 cells, so a storm over one cell
/// leaves a neighbouring cell to interpolate toward.
const EXTENT: u32 = 128;

/// The seed of the fixture world.
const SEED: u64 = 7;

/// The ticks the fixture runs before a faction holds ground.
const HOLDING_TICKS: u32 = 6;

/// The ticks the fixture runs to put work into an upgrade site.
const BUILDING_TICKS: u32 = 4;

/// The ticks a storm is given to fall out of the air onto the ground.
const FALLING_TICKS: u32 = 60;

/// How far the fixture looks for a tile that nobody holds.
///
/// The reach stays inside the window the camera shows, so the site it finds
/// is drawn.
const UNHELD_REACH: i32 = 20;

/// The strength of the storm the fixture inflicts.
const STRENGTH: u8 = 4;

/// The size of a tile in every picture these tests draw, in pixels.
///
/// This is below the width at which the resting air wash draws, so a frame
/// with no overlay carries no wash and a difference is the overlay alone.
const TILE: f32 = 6.0;

/// The size of every picture these tests draw, in pixels.
const WINDOW: (usize, usize) = (640, 640);

fn settings() -> WorldConfig {
    WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: SEED,
        faction_count: 2,
        unit_capacity: 8192,
    }
}

/// Builds a world in which a faction holds ground, builds, and is stormed.
///
/// **The world seeds itself.** Ground is held within the reach of a city its
/// faction owns, so a band of units alone holds nothing.[^1]
///
/// Returns the world and the tile the storm landed on.
///
/// # References
///
/// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
fn a_stormed_world() -> (World, Axial) {
    let mut world = seeded();
    for _ in 0..HOLDING_TICKS {
        world.step(1).expect("the step must run");
    }
    assert!(
        world.holding_of(FactionId(0)) > 0,
        "the faction holds nothing, so the fixture supplies no held ground",
    );

    // The tile the camera centres on, and the tile the crowd goes on.
    let place = a_held_tile(&world, FactionId(0));

    // **A site on held ground is not enough for the upgrade overlay.** The
    // holder colour takes its share of a tile after the overlay is mixed
    // into the ground, so a held tile shows the holder and not the wash. The
    // fixture therefore builds one site on ground that nobody holds, near
    // the tile the camera centres on.
    //
    // The fixture built no such site before, and the test passed on whatever
    // the demonstration controller happened to draw. A change to the
    // category set moved that draw and the test went red.[^5]
    //
    // [^5]: Testing Rules, section 2a. `.agents/rules/testing.md`
    let free = an_unheld_tile(&world, place);
    let outsider = world
        .spawn_soldier(free, FactionId(0))
        .expect("the ground admits a unit");
    world
        .zone_project(FactionId(0), free, UpgradeCategory::ALL[0])
        .expect("a way is zoned on ground nobody holds");
    world
        .order_build(outsider, UpgradeCategory::ALL[0])
        .expect("the ground fits a way");
    for _ in 0..BUILDING_TICKS {
        world
            .order_build(outsider, UpgradeCategory::ALL[0])
            .expect("the ground fits a way");
        world.step(1).expect("the step must run");
    }
    assert!(
        world.upgrade_at(free).is_some(),
        "the fixture built nothing on unheld ground, so the upgrade overlay \
         paints nowhere the holder does not cover",
    );

    // A crowd and a site under work, so the crowding overlay and the upgrade
    // overlay each have something to paint. The units go on one tile the
    // faction already holds.
    let mut crowd: Vec<Entity> = Vec::new();
    while world.admits_a_unit(place) {
        match world.spawn_soldier(place, FactionId(0)) {
            Ok(unit) => crowd.push(unit),
            Err(_) => break,
        }
        world.rebuild_bridge(1).expect("the bridge rebuilds");
    }
    assert!(
        !crowd.is_empty(),
        "the fixture put nobody on the tile, so the crowding overlay finds nothing",
    );
    world.order_build_set(&crowd, UpgradeCategory::ALL[0]);
    for _ in 0..BUILDING_TICKS {
        world.step(1).expect("the step must run");
    }
    assert!(
        world.upgrade_at(place).is_some(),
        "the fixture built nothing, so the upgrade overlay finds nothing",
    );

    world
        .inflict_weather(FactionId(0), &[place], STRENGTH)
        .expect("the faction holds the ground it storms");
    (world, place)
}

/// Builds a world that has founded its factions and stepped no tick.
fn seeded() -> World {
    let mut world = World::new(settings()).expect("the extent describes a world");
    world.seed_world().expect("the world seeds itself");
    world
}

/// Returns one tile a faction holds.
fn a_held_tile(world: &World, faction: FactionId) -> Axial {
    let width = world.grid().width();
    for index in 0..world.grid().tile_count() {
        let address = Axial::new((index % width) as i32, (index / width) as i32);
        if world
            .tile_holder(address)
            .and_then(cachette_core::Holder::faction)
            == Some(faction)
            && world.admits_a_unit(address)
        {
            return address;
        }
    }
    panic!("the faction holds no tile, so the fixture supplies no held ground");
}

/// Returns a tile that nobody holds, near one the camera shows.
///
/// The search runs outward from the centre in rows, so it takes the nearest
/// such tile of the window and never a tile the camera cuts off.
fn an_unheld_tile(world: &World, near: Axial) -> Axial {
    for reach in 1..UNHELD_REACH {
        for dr in -reach..=reach {
            for dq in -reach..=reach {
                let address = Axial::new(near.q + dq, near.r + dr);
                if !world.grid().contains(address) {
                    continue;
                }
                if world
                    .tile_holder(address)
                    .and_then(cachette_core::Holder::faction)
                    .is_some()
                {
                    continue;
                }
                if world.admits_a_unit(address) && world.upgrade_at(address).is_none() {
                    return address;
                }
            }
        }
    }
    panic!("no tile near the camera centre is free of a holder");
}

/// Returns the world after the storm has fallen onto the ground.
fn fallen(mut world: World) -> World {
    for _ in 0..FALLING_TICKS {
        world.step(1).expect("the step must run");
    }
    assert!(
        world.weather().wet_cells() > 0,
        "no cell is wet, so the fixture supplies no moisture",
    );
    world
}

/// Returns the camera every test draws with.
fn camera(world: &World, over: Axial) -> Camera {
    let canvas = Canvas::new(WINDOW.0, WINDOW.1);
    Camera::at_tile_size(TILE)
        .looking_at(over, &canvas)
        .clamped(world, &canvas)
}

/// Draws one still frame, with the overlay of this name or with none.
fn shot(world: &World, camera: Camera, overlay: Option<&'static dyn Layer>) -> Canvas<'static> {
    let mut canvas = Canvas::new(WINDOW.0, WINDOW.1);
    paint::draw_paced(
        world,
        camera,
        &mut canvas,
        cachette_view::Pace::STILL,
        &mut cachette_view::Motion::none(),
        overlay,
    )
    .expect("the world draws");
    canvas
}

/// Returns the pixel one step inside the top left corner of a tile.
///
/// A unit is a disc at the middle of its tile, so the corner is ground
/// whether or not a unit stands there.
fn corner_of(canvas: &Canvas, camera: Camera, address: Axial) -> u32 {
    let (left, top, _, _) = tile_rect(camera, address);
    let (x, y) = (left + 1, top + 1);
    assert!(
        x >= 0 && y >= 0 && (x as usize) < canvas.width() && (y as usize) < canvas.height(),
        "the tile {address:?} is off the canvas",
    );
    canvas.pixels()[y as usize * canvas.width() + x as usize]
}

/// Builds the two worlds these tests read: one stormed, one after the fall.
fn the_two_worlds() -> (World, World, Axial) {
    let (stormy, place) = a_stormed_world();
    let wet = fallen(stormy.clone());
    (stormy, wet, place)
}

#[test]
fn every_registered_overlay_changes_the_picture() {
    // **The fixture must supply a value for every overlay.** An overlay that
    // is zero everywhere paints nothing, and this test would then pass for an
    // overlay that does nothing at all. The failure message names the overlay,
    // so a reader learns which of the two happened.
    let (stormy, wet, place) = the_two_worlds();
    let camera = camera(&stormy, place);

    for layer in overlay::registered() {
        // The air is in the air before the storm falls, and it is on the
        // ground afterwards. Each overlay reads the world in which its own
        // quantity exists.
        let world = if layer.name() == "moisture" {
            &wet
        } else {
            &stormy
        };
        let bare = shot(world, camera, None);
        let washed = shot(world, camera, Some(*layer));
        let reading = washed
            .overlay()
            .expect("the pass records what it painted of an overlay");
        assert!(
            !reading.found_nothing(),
            "the fixture holds nothing for the {} overlay, so this test would \
             measure the fixture rather than the overlay",
            layer.name(),
        );
        assert_ne!(
            bare.pixels(),
            washed.pixels(),
            "the {} overlay did not change any pixel",
            layer.name(),
        );
    }
}

#[test]
fn a_cell_value_paints_as_a_field_and_not_as_a_block() {
    // Weather lives on the level 1 cell lattice, and a cell is many tiles a
    // side. A flat read of the cell gives one value to every tile of it, and
    // the map then draws a grid of blocks that follows nothing in the
    // world.[^1]
    //
    // The two tiles sit in one cell, near opposite corners of it. A flat read
    // gives them one value. They must differ, because the cells on either
    // side of theirs differ.
    //
    // [^1]: Research report 24, defect 6. `docs/research/reports/24-demonstration-readability-resources-and-weather.md`
    let (stormy, _, _) = the_two_worlds();
    let layout = stormy.pyramid().layout();
    let edge = layout.block_edge();
    assert!(
        edge >= 4,
        "a cell of {edge} tiles is too small for this test"
    );
    let air = overlay::named("air").expect("the deck registers the air overlay");

    // The centre tile of a cell carries the value of that cell.
    let centre = |cell_column: u32, cell_row: u32| -> Axial {
        Axial::new(
            (cell_column * edge + edge / 2) as i32,
            (cell_row * edge + edge / 2) as i32,
        )
    };

    let mut found = false;
    for cell_row in 0..layout.blocks_high() {
        for cell_column in 1..layout.blocks_wide().saturating_sub(1) {
            let before = stormy.air_at(centre(cell_column - 1, cell_row));
            let after = stormy.air_at(centre(cell_column + 1, cell_row));
            if before == after {
                continue;
            }
            let column = cell_column * edge;
            let row = cell_row * edge;
            let near = Axial::new((column + 1) as i32, (row + 1) as i32);
            let far = Axial::new((column + edge - 2) as i32, (row + 1) as i32);
            assert_eq!(
                stormy.air_at(near),
                stormy.air_at(far),
                "the two tiles must sit in one cell, so a flat read would give \
                 them one value",
            );
            assert_ne!(
                overlay::value_of(air, &stormy, near, None),
                overlay::value_of(air, &stormy, far, None),
                "two tiles far apart in one cell painted one value at \
                 {near:?} and {far:?}, so the cell still paints as a rectangle",
            );
            found = true;
        }
    }
    assert!(
        found,
        "no cell of the fixture has neighbours that differ, so this test \
         would measure the fixture rather than the drawing",
    );
}

#[test]
fn two_runs_of_one_frame_give_one_picture() {
    let (_, wet, place) = the_two_worlds();
    let camera = camera(&wet, place);
    for layer in overlay::registered() {
        let first = shot(&wet, camera, Some(*layer));
        let second = shot(&wet, camera, Some(*layer));
        assert_eq!(
            first.pixels(),
            second.pixels(),
            "two runs of the {} overlay gave two pictures",
            layer.name(),
        );
    }
}

#[test]
fn the_holder_stays_readable_under_every_overlay() {
    // The overlay is mixed into the ground before the holder takes its share,
    // so the holder survives at every overlay strength. A wash over the
    // finished pixel erased it, and that is the defect the report
    // measured.[^1]
    //
    // The two worlds differ in the founding: the first founded its factions
    // and the second did not, so the first holds ground and the second holds
    // none. The seed, the extent and the tick count are the same, so the
    // ground, the resources and the weather of the tile are the same in both.
    // A pixel that is equal in the two worlds therefore carries no holder.
    // The test asserts below that the holder changes the pixel it reads, so a
    // tile at which the two worlds agree stops the test rather than passing
    // it.
    //
    // The height overlay is the strong case. It has a value at every tile of
    // the window, so it paints the widest wash of any overlay here.
    //
    // [^1]: Research report 24, defect 4. `docs/research/reports/24-demonstration-readability-resources-and-weather.md`
    let mut held = seeded();
    let mut open = World::new(settings()).expect("the extent describes a world");
    // Both worlds step the same count. The unfounded world holds no unit, so
    // nothing in it gathers, builds or moves, and its ground stays what the
    // generator gave it.
    for _ in 0..HOLDING_TICKS {
        held.step(1).expect("the step must run");
        open.step(1).expect("the step must run");
    }
    let place = a_held_tile(&held, FactionId(0));
    assert!(
        open.tile_holder(place)
            .and_then(cachette_core::Holder::faction)
            .is_none(),
        "the unfounded world holds ground, so the two worlds differ in more \
         than the holder",
    );

    // The upgrade overlay paints where a site stands, so the fixture puts one
    // on ground the faction holds rather than waiting for the controller to
    // build one. A road takes a tile the faction's plan zones, so the fixture
    // zones the tile before it orders the build. Without the site the upgrade
    // overlay paints nothing anywhere held, and the loop below would measure
    // the fixture rather than the drawing.
    let builder = held
        .spawn_soldier(place, FactionId(0))
        .expect("the held tile admits a unit");
    held.rebuild_bridge(1).expect("the bridge rebuilds");
    held.zone_project(FactionId(0), place, UpgradeCategory::ALL[0])
        .expect("the plan takes the project");
    held.order_build(builder, UpgradeCategory::ALL[0])
        .expect("the engine takes the order");
    // Both worlds step the same count here as well, so the ground of the two
    // stays the same.
    for _ in 0..BUILDING_TICKS {
        held.step(1).expect("the step must run");
        open.step(1).expect("the step must run");
    }
    assert!(
        held.upgrade_at(place).is_some(),
        "the fixture built nothing, so the upgrade overlay finds nothing",
    );
    let camera = camera(&held, place);

    // **Each overlay is read at the tile where it paints most strongly.** A
    // tile the overlay barely touches would leave the holder visible under a
    // wash as well, so a fixed tile measures the fixture rather than the
    // drawing.
    let bare = shot(&held, camera, None);
    let bare_open = shot(&open, camera, None);

    for layer in overlay::registered() {
        let span = layer.span(&held);
        let Some(place) = strongest(&held, *layer, span) else {
            panic!("the {} overlay paints nothing anywhere held", layer.name());
        };
        let bare_gap = gap(
            corner_of(&bare, camera, place),
            corner_of(&bare_open, camera, place),
        );
        assert!(
            bare_gap > 0,
            "the holder changes no pixel at {place:?} with no overlay, so this \
             test would measure the fixture rather than the drawing",
        );
        let ours = shot(&held, camera, Some(*layer));
        let nobodys = shot(&open, camera, Some(*layer));
        let washed_gap = gap(
            corner_of(&ours, camera, place),
            corner_of(&nobodys, camera, place),
        );
        assert!(
            washed_gap * 2 >= bare_gap,
            "the {} overlay left {washed_gap} of the {bare_gap} the holder \
             tint is worth at {place:?}, so the holder is no longer readable",
            layer.name(),
        );
    }
}

/// Returns the held tile at which one overlay paints most strongly.
///
/// The search walks the tiles of the fixture world once. It is a test fixture
/// and not a drawing pass, so it owes the drawing budget nothing.
fn strongest(world: &World, layer: &'static dyn Layer, span: overlay::Span) -> Option<Axial> {
    let width = world.grid().width();
    let mut best: Option<(u8, Axial)> = None;
    for index in 0..world.grid().tile_count() {
        let address = Axial::new((index % width) as i32, (index / width) as i32);
        if world
            .tile_holder(address)
            .and_then(cachette_core::Holder::faction)
            != Some(FactionId(0))
        {
            continue;
        }
        let value = overlay::value_of(layer, world, address, None);
        let strength = layer.strength(value, span);
        if strength > 0 && best.is_none_or(|(seen, _)| strength > seen) {
            best = Some((strength, address));
        }
    }
    best.map(|(_, address)| address)
}

/// Returns how far apart two colours are, summed over the three channels.
///
/// A test reads this rather than asking whether two colours are equal. A wash
/// over a finished pixel leaves the holder present and unreadable, and an
/// equality check calls that a pass.
fn gap(one: u32, other: u32) -> i32 {
    (0..3)
        .map(|channel| {
            let shift = channel * 8;
            let left = ((one >> shift) & 0xff) as i32;
            let right = ((other >> shift) & 0xff) as i32;
            (left - right).abs()
        })
        .sum()
}

#[test]
fn a_name_the_deck_did_not_publish_is_refused() {
    assert!(
        overlay::named("nonsense").is_none(),
        "the deck answered a name it never published",
    );
    for name in overlay::names() {
        assert!(
            overlay::named(name).is_some(),
            "the deck published {name} and then refused it",
        );
    }
}

#[test]
fn the_key_names_the_scale_of_the_overlay_that_is_on() {
    let (_, wet, place) = the_two_worlds();
    let camera = camera(&wet, place);
    let moisture = overlay::named("moisture").expect("the deck registers the moisture overlay");
    let mut canvas = Canvas::new(WINDOW.0, WINDOW.1);
    let readout = cachette_view::draw_frame_paced(
        &wet,
        camera,
        &Metrics::start(),
        &[],
        Overlay::Panel,
        Some(moisture),
        cachette_view::Pace::STILL,
        &mut cachette_view::Motion::none(),
        &mut canvas,
    )
    .expect("the world draws");

    let said = hud::says(&readout);
    assert!(
        said.iter().any(|line| line == "overlay: moisture"),
        "the key did not name the overlay that was on: {said:?}",
    );
    assert!(
        said.iter().any(|line| line.starts_with("full colour at: ")),
        "the key did not name the value at which the overlay draws full colour",
    );
    assert!(
        said.iter().any(|line| line.starts_with("no colour at: ")),
        "the key did not name the value at which the overlay draws no colour",
    );
    assert!(
        said.iter().any(|line| line.starts_with("in the window: ")),
        "the key did not say what the pass met in the window",
    );
}

#[test]
fn an_overlay_that_found_nothing_says_so_in_words() {
    // A subsystem that produced no instance and a subsystem that nothing
    // draws look the same on a map. The key says which, in words.[^1]
    //
    // [^1]: Backlog item 0278, say what the demonstration world never produced. `docs/backlog/complete/0278-say-what-the-demonstration-world-never-produced.md`
    let world = World::new(settings()).expect("the extent describes a world");
    assert!(
        world.weather().is_dry(),
        "the fixture must hold no water, so the moisture overlay finds nothing",
    );
    let place = Axial::new(EXTENT as i32 / 2, EXTENT as i32 / 2);
    let camera = camera(&world, place);
    let moisture = overlay::named("moisture").expect("the deck registers the moisture overlay");
    let mut canvas = Canvas::new(WINDOW.0, WINDOW.1);
    let readout = cachette_view::draw_frame_paced(
        &world,
        camera,
        &Metrics::start(),
        &[],
        Overlay::Panel,
        Some(moisture),
        cachette_view::Pace::STILL,
        &mut cachette_view::Motion::none(),
        &mut canvas,
    )
    .expect("the world draws");

    let key: Vec<(String, String)> = readout.overlay_key();
    assert!(
        key.iter()
            .any(|(_, value)| value == hud::OVERLAY_FOUND_NOTHING),
        "an empty overlay did not say so in words: {key:?}",
    );
}

#[test]
fn a_frame_with_no_overlay_names_none() {
    let (_, wet, place) = the_two_worlds();
    let camera = camera(&wet, place);
    let canvas = shot(&wet, camera, None);
    assert!(
        canvas.overlay().is_none(),
        "a frame the caller asked no overlay for reported one",
    );
    let readout = Readout::of(&wet, camera, &canvas, &Metrics::start(), &[]);
    assert!(
        readout.overlay_key().is_empty(),
        "the key named an overlay that no frame drew",
    );
}
