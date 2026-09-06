//! A probe that reports what the temperature overlay and the weather panel
//! show over a run.
//!
//! This is a diagnostic, not a test. The weather now travels because a season
//! moves a warm band across the cell lattice. A watcher must be able to see
//! that band, so this probe reports, at intervals, the span the temperature
//! overlay paints between and the strength it paints at the coldest and the
//! warmest cell. It also prints the weather panel with a pointer, and it names
//! any line the panel cuts.
//!
//! Run it with `cargo run -p cachette-view --release --example
//! weather_overlay_probe`.

// An example is its own crate, so the allowance at the viewer's crate root
// does not reach it. A camera holds a tile width, and a tile width is a viewer
// value.
#![allow(clippy::disallowed_types)]

use cachette_core::{Axial, World, WorldConfig};
use cachette_view::overlay::{self, At};
use cachette_view::panel::{lines_that_do_not_fit, Set, View};
use cachette_view::Camera;

const EXTENT: u32 = 256;
const SEED: u64 = 0x2f;
const TICKS: u32 = 400;
const EVERY: u32 = 40;

fn main() {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: SEED,
        faction_count: 4,
        unit_capacity: 1024,
    })
    .expect("the settings describe a world");

    let layer = overlay::named("temperature").expect("the deck registers the temperature overlay");
    println!("tick   low  high  coldest paints  warmest paints  hottest tile");

    for tick in 1..=TICKS {
        world.step(1).expect("the step runs");
        if tick % EVERY != 0 {
            continue;
        }
        let span = layer.span(&world);
        let plane = world.weather().warmth_plane();
        let low = i64::from(plane.iter().copied().min().unwrap_or(0));
        let high = i64::from(plane.iter().copied().max().unwrap_or(0));

        // The tile the overlay paints most strongly, so a reader can see the
        // band move over the map rather than only the two ends of the span.
        let mut best = Axial::new(0, 0);
        let mut most = i64::MIN;
        for row in 0..EXTENT as i32 {
            for column in 0..EXTENT as i32 {
                let address = Axial::new(column, row);
                let value = layer.value(At {
                    world: &world,
                    address,
                    ground: None,
                });
                if value > most {
                    most = value;
                    best = address;
                }
            }
        }
        println!(
            "{tick:4}  {:4}  {:4}  {:14}  {:14}  q {:3} r {:3}",
            span.low,
            span.high,
            layer.strength(low, span),
            layer.strength(high, span),
            best.q,
            best.r,
        );
    }

    // The panel must show the temperature and must cut no line.
    let pointer = Axial::new(3, 3);
    let view = View {
        world: &world,
        camera: Camera {
            tile_width: 1.0,
            tile_height: 1.0,
            origin_x: 0.0,
            origin_y: 0.0,
        },
        frame_width: 800,
        frame_height: 600,
        focus: None,
        pointer: Some(pointer),
    };
    let bad = lines_that_do_not_fit(&view, Set::EMPTY.with("weather").expect("a known panel"));
    println!("panel lines that do not fit: {}", bad.len());
    for line in &bad {
        println!("  cut: {line:?}");
    }
    println!("overlay names: {:?}", overlay::names());
}
