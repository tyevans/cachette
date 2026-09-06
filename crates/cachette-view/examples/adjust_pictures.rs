//! Writes the five pictures that the readability reviews read.
//!
//! The three reviews read a still picture of the demonstration at four zooms
//! and at two ticks, and each ranked ten defects.[^1] [^2] [^3] This example
//! writes the same views after the adjustments, so a reader compares the
//! pictures rather than the code.
//!
//! The views are the region at eight pixels a tile at tick 1500, the city at
//! tick 300, a close-up over a build site, a storm one tick after it falls,
//! and the city with the reference key held.
//!
//! Usage: `cargo run --example adjust_pictures`
//!
//! The format is binary PPM, which every image tool reads and which needs no
//! dependency. The files land under a build directory and are not committed.
//!
//! The viewer reads the world and writes nothing to it.[^4]
//!
//! # References
//!
//! [^1]: Research report 23, demonstration readability review 1. `docs/research/reports/23-demonstration-readability-review-1.md`
//! [^2]: Research report 24, resources and weather. `docs/research/reports/24-demonstration-readability-resources-and-weather.md`
//! [^3]: Research report 25, upgrades and unit positioning. `docs/research/reports/25-demonstration-readability-upgrades-and-units.md`
//! [^4]: ADR-0067, the viewer reads the world and never writes to it, decision D1. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`

// An example is its own crate, so the allowance at the viewer's crate root
// does not reach it. ADR-0067 D3 puts the float boundary at the viewer, and a
// tile width is a viewer value.
#![allow(clippy::disallowed_types)]

use std::path::Path;

use cachette_core::founding::FoundingOutcome;
use cachette_core::weather::STRENGTH_CEILING;
use cachette_core::{Axial, FactionId, World, WorldConfig};
use cachette_view::picture::write_ppm;
use cachette_view::{draw_frame_paced, Camera, Canvas, Metrics, Motion, Overlay, Pace};

/// The extent of the demonstration world, in tiles a side.
const EXTENT: u32 = 256;

/// The seed of the demonstration world.
const SEED: u64 = 0x0cac_4e77_0032;

/// The factions the run seats.
const FACTIONS: u16 = 4;

/// The threads each step runs at.
const THREADS: usize = 4;

/// The size of every picture, in pixels.
const WINDOW: (usize, usize) = (960, 720);

/// The directory the pictures land in.
const OUT: &str = "target/adjust-1";

fn main() {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: SEED,
        faction_count: FACTIONS,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent describes a world");
    let outcomes = world.seed_world().expect("the world seeds once");
    let seat = outcomes
        .iter()
        .find_map(FoundingOutcome::founding)
        .expect("the run seated a faction")
        .place();

    std::fs::create_dir_all(OUT).expect("the output directory must open");

    run_to(&mut world, 299);
    write(&world, &outcomes, seat, 48.0, false, "t300-city.ppm");
    write(
        &world,
        &outcomes,
        seat,
        48.0,
        true,
        "t300-city-reference.ppm",
    );

    // A close-up over a site somebody is building. The controllers build, so
    // the run holds sites of its own, and the picture names the one it found.
    let site = world
        .upgrade_sites()
        .iter()
        .find(|site| !site.is_complete())
        .map(|site| address_of(&world, site.tile.0));
    match site {
        Some(address) => write(
            &world,
            &outcomes,
            address,
            64.0,
            false,
            "t300-close-site.ppm",
        ),
        None => println!("no site stands at tick 300, so no close-up was written"),
    }

    run_to(&mut world, 1499);
    write(&world, &outcomes, seat, 8.0, false, "t1500-region.ppm");

    // A storm over the seat. The picture is the frame after the step that
    // the writer takes, so it is the frame one tick after the storm fell.
    world
        .inflict_weather(FactionId(0), &[seat], STRENGTH_CEILING)
        .expect("the faction storms the ground it holds");
    write(&world, &outcomes, seat, 12.0, false, "t1500-storm.ppm");
    // The picture is one tick after the storm, and the writer stepped a copy,
    // so the drops the picture drew are the drops one step on.
    let mut after = world.clone();
    after.step(THREADS).expect("the step must run");
    println!(
        "storm: {} drops over the seat, overlay weight {} of 255",
        after.air_at(seat).unwrap_or(0),
        cachette_view::paint::air_weight(after.air_at(seat).unwrap_or(0))
    );
}

/// Steps the world until it reaches a tick.
fn run_to(world: &mut World, tick: u64) {
    while world.tick().0 < tick {
        world.step(THREADS).expect("the step must run");
    }
    println!("tick {}", world.tick().0);
}

/// Returns the address of a tile index.
fn address_of(world: &World, index: u32) -> Axial {
    let width = world.grid().width();
    Axial::new((index % width) as i32, (index / width) as i32)
}

/// Writes one picture, and returns nothing.
///
/// **The picture is the frame after one step, and the step is taken on a copy
/// of the world.** The heading line of a unit runs from the tile the unit
/// stood on at the last frame, and the table that holds that tile is the
/// caller's memory. A single frame therefore carries no line. The copy keeps
/// every picture at one tick of the run whatever order they are written in.
fn write(
    world: &World,
    outcomes: &[FoundingOutcome],
    over: Axial,
    tile: f32,
    reference: bool,
    name: &str,
) {
    let mut shot = world.clone();
    let mut canvas = Canvas::new(WINDOW.0, WINDOW.1);
    let camera = Camera::at_tile_size(tile)
        .looking_at(over, &canvas)
        .clamped(&shot, &canvas);
    let mut motion = Motion::for_frame(WINDOW.0, WINDOW.1);
    let draw = |world: &World, canvas: &mut Canvas, motion: &mut Motion| {
        draw_frame_paced(
            world,
            camera,
            &Metrics::start(),
            outcomes,
            Overlay::Glass { reference },
            None,
            Pace::STILL,
            motion,
            canvas,
        )
        .expect("the world draws");
    };
    draw(&shot, &mut canvas, &mut motion);
    shot.step(THREADS).expect("the step must run");
    draw(&shot, &mut canvas, &mut motion);

    let path = Path::new(OUT).join(name);
    let mut file =
        std::io::BufWriter::new(std::fs::File::create(&path).expect("the output file must open"));
    write_ppm(&canvas, &mut file).expect("the pixels must write");
    println!(
        "{}: tick {}, {tile} pixels a tile over {over:?}",
        path.display(),
        shot.tick().0
    );
}
