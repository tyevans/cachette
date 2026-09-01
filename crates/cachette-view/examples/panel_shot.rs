//! Writes one frame of a founded run to an image file.
//!
//! A person reads the panel from this picture without a display. The
//! demonstration binary needs a window, and a machine in continuous
//! integration has none.
//!
//! Usage: `cargo run --example panel_shot -- <out.ppm>`
//!
//! The viewer reads the world and writes nothing to it.[^1]
//!
//! # References
//!
//! [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D1. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`

use cachette_core::{FactionId, World, WorldConfig};
use cachette_view::picture::write_ppm;
use cachette_view::{draw_frame, Camera, Canvas, Metrics};

/// The size of the picture in pixels.
const WINDOW: (usize, usize) = (420, 760);

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "panel.ppm".to_string());
    let mut world = World::new(WorldConfig {
        width: 200,
        height: 140,
        seed: 11,
        faction_count: 4,
    })
    .expect("the extent describes a world");
    let founded = world
        .found_run(24, FactionId(0))
        .expect("the world holds a place for the group");
    world.rebuild_bridge(1).expect("the rebuild must succeed");

    let mut canvas = Canvas::new(WINDOW.0, WINDOW.1);
    let camera = Camera::opening()
        .looking_at(founded.place(), &canvas)
        .clamped(&world, &canvas);
    let foundings = [founded];
    draw_frame(&world, camera, &Metrics::start(), &foundings, &mut canvas)
        .expect("the world draws");

    let mut file =
        std::io::BufWriter::new(std::fs::File::create(&path).expect("the file must open"));
    write_ppm(&canvas, &mut file).expect("the image must write");
    println!("wrote {path}");
}
