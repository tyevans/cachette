//! A probe that measures the striping of the air overlay.
//!
//! This is a diagnostic, not a test. It answers one question: does the
//! drawing turn a smooth field into a regular pattern? It reports how many
//! distinct values and how many distinct paint strengths the air overlay
//! produces over a run of tiles, and it prints one row of strengths so a
//! reader can see the steps.
//!
//! It reports the same figures with the interpolation on and with it off, so
//! a reader can tell a contour of the field from a contour of the drawing.
//!
//! Run it with `cargo run -p cachette-view --release --example
//! air_striping_probe`. The arguments are the extent, the seed, the ticks and
//! the weather pitch in tiles.

// An example is its own crate, so the allowance at the viewer's crate root
// does not reach it.
#![allow(clippy::disallowed_types)]

use std::collections::BTreeSet;

use cachette_core::weather::WeatherScale;
use cachette_core::{Axial, World, WorldConfig};
use cachette_view::overlay::{self, At};
use cachette_view::picture::write_ppm;
use cachette_view::{paint, Camera, Canvas, Motion, Pace};

fn argument(position: usize, fallback: u64) -> u64 {
    std::env::args()
        .nth(position)
        .and_then(|text| text.parse::<u64>().ok())
        .unwrap_or(fallback)
}

fn main() {
    let extent = argument(1, 256) as u32;
    let seed = argument(2, 0x2f);
    let ticks = argument(3, 400) as u32;
    let pitch = argument(4, 8) as u32;
    let scale =
        WeatherScale::from_bits(pitch.trailing_zeros()).expect("the pitch describes a lattice");
    let mut world = World::with_weather_scale(
        WorldConfig {
            width: extent,
            height: extent,
            seed,
            faction_count: 4,
            unit_capacity: 1024,
        },
        scale,
    )
    .expect("the settings describe a world");
    for _ in 0..ticks {
        world.step(1).expect("the step runs");
    }

    let layer = overlay::named("cloud").expect("the deck registers the cloud overlay");
    let span = layer.span(&world);
    println!("extent {extent} seed {seed:#x} ticks {ticks} pitch {pitch} tiles a cell");
    println!("air span low {} high {}", span.low, span.high);

    let mut values = BTreeSet::new();
    let mut strengths = BTreeSet::new();
    let mut flat_values = BTreeSet::new();
    let mut flat_strengths = BTreeSet::new();
    for row in 0..extent as i32 {
        for column in 0..extent as i32 {
            let address = Axial::new(column, row);
            let ground = world.tile_terrain(address);
            let value = overlay::value_of(layer, &world, address, ground);
            values.insert(value);
            strengths.insert(layer.strength(value, span));
            let flat = layer.value(At {
                world: &world,
                address,
                ground,
            });
            flat_values.insert(flat);
            flat_strengths.insert(layer.strength(flat, span));
        }
    }
    println!(
        "interpolated: {} distinct values, {} distinct strengths",
        values.len(),
        strengths.len()
    );
    println!(
        "flat cell   : {} distinct values, {} distinct strengths",
        flat_values.len(),
        flat_strengths.len()
    );

    // One row of tiles, so a reader can see the step pattern the eye reads as
    // a stripe. A run of equal strengths is one band.
    for row in [extent as i32 / 4, extent as i32 / 2] {
        let mut line = String::new();
        let mut runs = 0usize;
        let mut last = None;
        for column in 0..48i32 {
            let address = Axial::new(column, row);
            let ground = world.tile_terrain(address);
            let value = overlay::value_of(layer, &world, address, ground);
            let strength = layer.strength(value, span);
            if last != Some(strength) {
                runs += 1;
                last = Some(strength);
            }
            line.push_str(&format!("{strength:>4}"));
        }
        println!("row {row} strengths over 48 tiles, {runs} bands:");
        println!("  {line}");
    }

    // How wide a band is, over the whole map. A band is a run of one strength
    // along a row. A field drawn at eight tiles a cell should hold bands many
    // tiles wide; a band of one or two tiles is a contour of the drawing.
    let mut bands = 0usize;
    let mut tiles = 0usize;
    for row in 0..extent as i32 {
        let mut last = None;
        for column in 0..extent as i32 {
            let address = Axial::new(column, row);
            let ground = world.tile_terrain(address);
            let value = overlay::value_of(layer, &world, address, ground);
            let strength = layer.strength(value, span);
            if last != Some(strength) {
                bands += 1;
                last = Some(strength);
            }
            tiles += 1;
        }
    }
    println!(
        "mean band width along a row, in hundredths of a tile: {}",
        if bands == 0 { 0 } else { tiles * 100 / bands }
    );

    // A picture, so a person can look at what the figures describe.
    // The tile size in pixels. Zero fits the whole world into the picture; a
    // size above zero zooms in on the middle, which is where a regular
    // pattern of a few tiles becomes visible.
    let tile = argument(6, 0) as u32;
    let (width, height) = paint::canvas_for(&world, 900);
    let mut canvas = Canvas::new(width, height);
    let camera = if tile == 0 {
        Camera::fitting(&world, &canvas)
    } else {
        let size = tile as f32;
        let middle = extent as f32 / 2.0;
        Camera {
            tile_width: size,
            tile_height: size,
            origin_x: width as f32 / 2.0 - (middle + middle / 2.0) * size,
            origin_y: height as f32 / 2.0 - middle * size,
        }
    };
    let mut motion = Motion::none();
    paint::draw_paced(
        &world,
        camera,
        &mut canvas,
        Pace::STILL,
        &mut motion,
        Some(layer),
    )
    .expect("the world draws");
    // The same frame with nothing but the overlay, so a pattern the drawing
    // makes cannot hide behind the ground. Every tile paints white at its own
    // strength over black, through the same rectangle the map uses.
    for (name, interpolate) in [("/tmp/air_alone.ppm", true), ("/tmp/air_flat.ppm", false)] {
        let mut alone = Canvas::new(width, height);
        alone.clear();
        for row in 0..extent as i32 {
            for column in 0..extent as i32 {
                let address = Axial::new(column, row);
                let ground = world.tile_terrain(address);
                let value = if interpolate {
                    overlay::value_of(layer, &world, address, ground)
                } else {
                    layer.value(At {
                        world: &world,
                        address,
                        ground,
                    })
                };
                let strength = layer.strength(value, span);
                let (left, top, wide, tall) = paint::tile_rect(camera, address);
                alone.block(left, top, wide, tall, 0x0001_0101 * u32::from(strength));
            }
        }
        let mut file = std::io::BufWriter::new(
            std::fs::File::create(name).expect("the output file must open"),
        );
        write_ppm(&alone, &mut file).expect("the pixels must write");
    }

    let path = std::env::args()
        .nth(5)
        .unwrap_or_else(|| "air.ppm".to_string());
    let mut file =
        std::io::BufWriter::new(std::fs::File::create(&path).expect("the output file must open"));
    write_ppm(&canvas, &mut file).expect("the pixels must write");
    println!("wrote {path}, {width} by {height} pixels");
}
