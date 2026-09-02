//! Writes one frame of a world to an image file.
//!
//! The demonstration binary needs a display. This example needs none, so a
//! person on a machine without one can still look at the ground, and a
//! reviewer can attach the picture to a change.
//!
//! The file format is binary PPM, which every image tool reads and which
//! needs no dependency.
//!
//! Usage: `cargo run --example picture -- <seed> <extent> <out.ppm> <soldiers>`
//!
//! The soldier count is optional. A run with no soldier shows the ground
//! alone, which is how a person reads what the ground carries: a disc over a
//! tile hides the colour of that tile.
//!
//! The viewer reads the world and writes nothing to it.[^1]
//!
//! # References
//!
//! [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D1. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`

use cachette_core::{Axial, FactionId, World, WorldConfig};
use cachette_view::picture::write_ppm;
use cachette_view::{draw_frame, paint, Camera, Canvas, Lap, Metrics, Overlay};

/// The longer side of the picture in pixels.
///
/// The other side follows the shape of the world, so the picture holds no
/// empty band.
const SIDE: usize = 900;

/// The soldiers the picture holds when the caller names no number.
const SOLDIERS: u32 = 600;

/// The stride that spreads the soldiers over the open ground.
const SPREAD: u32 = 9973;

fn main() {
    let mut arguments = std::env::args().skip(1);
    let seed: u64 = arguments
        .next()
        .map_or(0x0cac_4e77_0032, |value| value.parse().unwrap_or(0));
    let extent: u32 = arguments
        .next()
        .map_or(128, |value| value.parse().unwrap_or(128));
    let path = arguments.next().unwrap_or_else(|| "world.ppm".to_string());
    let soldiers: u32 = arguments
        .next()
        .map_or(SOLDIERS, |value| value.parse().unwrap_or(SOLDIERS));

    let mut world = World::new(WorldConfig {
        width: extent,
        height: extent,
        seed,
        faction_count: 3,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent describes a world");

    // The ground refuses a soldier on water, so the soldiers take the open
    // ground the world has.
    let grid = world.grid();
    let open: Vec<Axial> = (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| world.admits_a_unit(*address))
        .collect();
    assert!(!open.is_empty(), "the world holds no open ground");
    for index in 0..soldiers {
        world
            .spawn_soldier(
                open[(index.wrapping_mul(SPREAD) as usize) % open.len()],
                FactionId((index % 3) as u16),
            )
            .expect("the open tile admits a unit");
    }

    let mut metrics = Metrics::start();
    for _ in 0..8 {
        let lap = Lap::start();
        world.step(2).expect("the step must run");
        metrics.step(lap.elapsed());
    }

    // The world draws as a parallelogram, which does not fill a square. A
    // square picture of it leaves the bottom third empty and the cards then
    // float in a void. The canvas takes the proportions of the shape instead.
    let (width, height) = paint::canvas_for(&world, SIDE);
    let mut canvas = Canvas::new(width, height);
    let camera = Camera::fitting(&world, &canvas);
    let readout = draw_frame(
        &world,
        camera,
        &metrics,
        &[],
        // The reference layer is on, so the picture names every colour it
        // draws without a person holding a key.
        Overlay::Glass { reference: true },
        &mut canvas,
    )
    .expect("the world draws");

    let mut file =
        std::io::BufWriter::new(std::fs::File::create(&path).expect("the output file must open"));
    write_ppm(&canvas, &mut file).expect("the pixels must write");

    println!(
        "{path}: seed {seed}, {extent} x {extent} tiles, tick {}, {} tiles drawn",
        readout.tick(),
        readout.tiles_painted()
    );
    println!(
        "  step: {:.0} us mean, {:.0} us worst, on this machine and not the target",
        metrics.step_mean_micros(),
        metrics.step_worst_micros()
    );
    for (ordinal, count) in readout.by_kind().iter().enumerate() {
        println!("  kind {ordinal}: {count} tiles");
    }
}
