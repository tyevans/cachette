//! Level 1 drawing shows a macroscopic view of a world.
//!
//! When the world is too large for its tiles to fit on the screen at one pixel
//! for each tile, the viewer draws from the Level 1 pyramid summaries rather
//! than Level 0 tiles.[^1]
//!
//! # References
//!
//! [^1]: ADR-0022, level 0 is the only truth and every level above it is derived, decision D4. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
//! [^2]: ADR-0094, the caller owns the camera and the pixels, decision D6. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
//! [^3]: Decisions register, DEC-084. `docs/DECISIONS.md`

#![allow(clippy::disallowed_types)]

use std::time::Instant;

use cachette_core::terrain::TileKind;
use cachette_core::{Axial, FactionId, World, WorldConfig};
use cachette_view::frame::{fill_frame, fill_frame_level1, FrameError, RenderLevel, Surface};
use cachette_view::paint::{Camera, Canvas};
use cachette_view::{draw_frame_level1, Metrics, Overlay};

/// The seed for the fixture worlds.
const SEED: u64 = 42;

/// Checks that Level 0 refuses sub-pixel cameras and Level 1 succeeds.
#[test]
fn level_0_refuses_sub_pixel_camera_and_level_1_succeeds() {
    let config = WorldConfig {
        width: 128,
        height: 128,
        seed: SEED,
        ..WorldConfig::DEFAULT
    };
    let mut world = World::new(config).expect("builds world");
    world.step(1).expect("steps world");

    let width = 256;
    let height = 256;
    let mut pixels = vec![0u32; width * height];
    let surface = Surface::new(width, height, &mut pixels).expect("builds surface");

    // Camera with sub-pixel tile width (0.25 < 1.0).
    // With block edge = 16, cell width is 0.25 * 16 = 4.0 >= 1.0.
    let camera = Camera {
        origin_x: 0.0,
        origin_y: 0.0,
        tile_width: 0.25,
        tile_height: 0.25,
    };
    let metrics = Metrics::start();

    // Level 0 must refuse sub-pixel tile scale.
    let l0_result = fill_frame(&world, camera, &metrics, &[], Overlay::Panel, surface);
    match l0_result {
        Err(FrameError::ScaleBelowLattice {
            tile_width,
            tile_height,
        }) => {
            assert!((tile_width - 0.25).abs() < f32::EPSILON);
            assert!((tile_height - 0.25).abs() < f32::EPSILON);
        }
        other => panic!("expected ScaleBelowLattice error, got {other:?}"),
    }

    // Level 1 must succeed for the same sub-pixel tile scale.
    let surface_l1 = Surface::new(width, height, &mut pixels).expect("builds surface");
    let readout = fill_frame_level1(&world, camera, &metrics, &[], Overlay::Panel, surface_l1)
        .expect("level 1 fill succeeds");

    assert_eq!(readout.level(), 1);
    assert_eq!(readout.render_level(), RenderLevel::Level1);
    assert!(readout.tiles_painted() > 0);
}

/// Checks that metadata explicitly identifies the source pyramid level.
#[test]
fn metadata_identifies_source_level() {
    let config = WorldConfig {
        width: 64,
        height: 64,
        seed: SEED,
        ..WorldConfig::DEFAULT
    };
    let mut world = World::new(config).expect("builds world");
    world.step(1).expect("steps world");

    let width = 256;
    let height = 256;
    let mut pixels = vec![0u32; width * height];
    let metrics = Metrics::start();
    let camera = Camera {
        origin_x: 0.0,
        origin_y: 0.0,
        tile_width: 4.0,
        tile_height: 4.0,
    };

    // Level 0 reports Level 0.
    let surface_l0 = Surface::new(width, height, &mut pixels).expect("builds surface");
    let readout_l0 = fill_frame(&world, camera, &metrics, &[], Overlay::Panel, surface_l0)
        .expect("level 0 fill succeeds");
    assert_eq!(readout_l0.level(), 0);
    assert_eq!(readout_l0.render_level(), RenderLevel::Level0);

    // Level 1 reports Level 1.
    let surface_l1 = Surface::new(width, height, &mut pixels).expect("builds surface");
    let readout_l1 = fill_frame_level1(&world, camera, &metrics, &[], Overlay::Panel, surface_l1)
        .expect("level 1 fill succeeds");
    assert_eq!(readout_l1.level(), 1);
    assert_eq!(readout_l1.render_level(), RenderLevel::Level1);
}

/// Checks that rendering an overview of a 1,048,576-tile world completes in under 16 ms.
#[test]
fn renders_overview_of_one_million_tile_world_under_16_ms() {
    // 1024 x 1024 = 1,048,576 tiles.
    let config = WorldConfig {
        width: 1024,
        height: 1024,
        seed: SEED,
        ..WorldConfig::DEFAULT
    };
    let mut world = World::new(config).expect("builds 1M tile world");
    world.step(1).expect("steps world to populate pyramid");

    let width = 1024;
    let height = 1024;
    let mut pixels = vec![0u32; width * height];

    // Camera zoomed out to show the whole world in 1024x1024 pixels.
    // Tile width ~ 0.5 pixels.
    let camera = Camera {
        origin_x: 0.0,
        origin_y: 0.0,
        tile_width: 0.5,
        tile_height: 0.5,
    };
    let metrics = Metrics::start();

    // Warm-up draw.
    let surface_warm = Surface::new(width, height, &mut pixels).expect("builds surface");
    let _ = fill_frame_level1(&world, camera, &metrics, &[], Overlay::Panel, surface_warm)
        .expect("warm-up fill succeeds");

    // Timed draw.
    let start = Instant::now();
    let surface_timed = Surface::new(width, height, &mut pixels).expect("builds surface");
    let readout = fill_frame_level1(&world, camera, &metrics, &[], Overlay::Panel, surface_timed)
        .expect("timed fill succeeds");
    let elapsed = start.elapsed();

    assert_eq!(readout.level(), 1);
    assert_eq!(readout.render_level(), RenderLevel::Level1);
    assert!(readout.tiles_painted() > 0);
    // Budget is 16 ms (60 fps frame budget).
    assert!(
        elapsed.as_millis() < 16,
        "rendering overview of 1,048,576 tiles took {elapsed:?}, exceeding 16 ms budget"
    );
}

/// Checks that macroscopic features (water, land, mountains, faction borders) are distinguishable.
#[test]
fn macroscopic_features_are_distinguishable() {
    let config = WorldConfig {
        width: 128,
        height: 128,
        seed: SEED,
        faction_count: 2,
        ..WorldConfig::DEFAULT
    };
    let mut world = World::new(config).expect("builds world");

    // Found two cities for two factions in adjacent blocks (edge is 16 tiles)
    // so each faction holds the majority in its respective block.
    world
        .found_settlement(Axial::new(28, 40), FactionId(0))
        .expect("founds city for faction 0");
    world
        .found_settlement(Axial::new(36, 40), FactionId(1))
        .expect("founds city for faction 1");

    // Run a step to update holding and pyramid summaries.
    world.step(1).expect("steps world");

    let width = 512;
    let height = 512;
    let mut pixels = vec![0u32; width * height];
    let metrics = Metrics::start();

    let camera = Camera {
        origin_x: 0.0,
        origin_y: 0.0,
        tile_width: 2.0,
        tile_height: 2.0,
    };

    let mut canvas = Canvas::borrowing(&mut pixels, width, height);
    let readout = draw_frame_level1(&world, camera, &metrics, &[], Overlay::Panel, &mut canvas)
        .expect("draws level 1 frame");

    assert_eq!(readout.level(), 1);
    assert!(readout.tiles_painted() > 0);

    // Verify presence of water and land in the painted summary.
    let painted = canvas.painted_by_kind();
    let water_painted = painted[TileKind::Water.to_u8() as usize];
    let plain_painted = painted[TileKind::Plain.to_u8() as usize];
    let mountain_painted = painted[TileKind::Mountain.to_u8() as usize];

    assert!(
        water_painted > 0,
        "Level 1 summary must paint macroscopic water bodies"
    );
    assert!(
        plain_painted > 0,
        "Level 1 summary must paint macroscopic land bodies"
    );
    assert!(
        mountain_painted > 0,
        "Level 1 summary must paint macroscopic mountain ridges"
    );

    // Verify faction holdings were recorded on canvas.
    assert!(
        canvas.tiles_held() > 0,
        "Level 1 summary must paint held tiles"
    );
    assert!(
        canvas.painted_by_faction()[0] > 0,
        "Faction 0 holdings must be painted"
    );
    assert!(
        canvas.painted_by_faction()[1] > 0,
        "Faction 1 holdings must be painted"
    );

    // Verify pixel variation across the canvas (image is not blank or flat).
    let mut distinct_colours = std::collections::HashSet::new();
    for &p in pixels.iter() {
        if p != 0 {
            distinct_colours.insert(p);
        }
    }
    assert!(
        distinct_colours.len() >= 5,
        "rendered level 1 image must show multiple distinct feature colours, found {}",
        distinct_colours.len()
    );
}
