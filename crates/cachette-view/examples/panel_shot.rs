//! Writes the whole panel of a founded run to an image file.
//!
//! **This is where every number lives that the window does not show.** The
//! window draws cards, which hold what changes moment to moment. This picture
//! draws the panel, which holds every section, and no window height cuts
//! it.[^2]
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
//! [^2]: Decisions register, DEC-084. `docs/DECISIONS.md`

use cachette_core::{World, WorldConfig};
use cachette_view::picture::write_ppm;
use cachette_view::{draw_frame, Camera, Canvas, Metrics, Overlay};

/// The size of the picture in pixels.
///
/// The picture exists so that a person reads the whole panel without a
/// display, so it is taller than the panel. A shorter picture would record
/// the cut rather than the panel.[^1]
///
/// # References
///
/// [^1]: Backlog item 0133, the panel is longer than the window. `docs/backlog/complete/0133-let-a-watcher-reach-a-panel-longer-than-the-window.md`
const WINDOW: (usize, usize) = (420, 1340);

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "panel.ppm".to_string());
    let mut world = World::new(WorldConfig {
        width: 200,
        height: 140,
        seed: 11,
        faction_count: 4,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent describes a world");
    let foundings = world.found_run_for_every_faction(24);
    let place = foundings
        .iter()
        .find_map(|outcome| outcome.founding().map(cachette_core::Founding::place))
        .expect("the world holds a place for a group");
    world.rebuild_bridge(1).expect("the rebuild must succeed");

    let mut canvas = Canvas::new(WINDOW.0, WINDOW.1);
    let camera = Camera::opening()
        .looking_at(place, &canvas)
        .clamped(&world, &canvas);
    draw_frame(
        &world,
        camera,
        &Metrics::start(),
        &foundings,
        Overlay::Panel,
        &mut canvas,
    )
    .expect("the world draws");

    let mut file =
        std::io::BufWriter::new(std::fs::File::create(&path).expect("the file must open"));
    write_ppm(&canvas, &mut file).expect("the image must write");
    println!("wrote {path}");
}
