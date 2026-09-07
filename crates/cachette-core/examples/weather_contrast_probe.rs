//! A probe that measures how much local contrast the weather field keeps.
//!
//! This is a diagnostic, not a test. **A field that mixes more than it needs
//! reads as one colour.** The probe reports, for each plane, the spread over
//! the whole world beside the mean step between two neighbours. The ratio of
//! the two is the roughness of the plane: a plane whose neighbours differ by a
//! small part of its own spread is smooth at the cell, whatever its map looks
//! like.
//!
//! **The terrain is the control.** It is the input contrast, it is generated
//! and never mixed, and every weather plane is driven by it. A weather plane
//! far smoother than the terrain under it has lost contrast that the world
//! offered.
//!
//! Run it with `cargo run -p cachette-core --release --example
//! weather_contrast_probe`. The arguments are the extent, the seed, the
//! weather scale and the ticks to settle for.

use cachette_core::hex::{Axial, NEIGHBOUR_COUNT};
use cachette_core::{TileIdx, WeatherScale, World, WorldConfig};

/// The threads that one step runs on.
const THREADS: usize = 4;

fn argument(position: usize, fallback: u64) -> u64 {
    std::env::args()
        .nth(position)
        .and_then(|text| match text.strip_prefix("0x") {
            Some(rest) => u64::from_str_radix(rest, 16).ok(),
            None => text.parse::<u64>().ok(),
        })
        .unwrap_or(fallback)
}

/// Returns the spread of a plane over the world, and the mean step between
/// two neighbours, both scaled by one thousand so that whole numbers carry
/// the answer.
///
/// The spread is the mean absolute difference from the mean of the plane. It
/// is used rather than a standard deviation because it needs no square root
/// and it reads in the same unit as the step.
fn roughness(values: &[(u32, i64)], world: &World) -> (i64, i64, i64) {
    let field = world.weather();
    let cells = field.cells();
    let count = values.len().max(1) as i64;
    let total: i64 = values.iter().map(|(_, value)| *value).sum();
    let mean = total / count;
    let spread: i64 = values
        .iter()
        .map(|(_, value)| (value - mean).abs())
        .sum::<i64>()
        / count;

    // The mean step between a cell and each neighbour that the lattice holds.
    let lookup: std::collections::HashMap<u32, i64> = values.iter().copied().collect();
    let mut steps = 0i64;
    let mut counted = 0i64;
    for (cell, value) in values {
        let Some(address) = cells.address_of(TileIdx(*cell)) else {
            continue;
        };
        for direction in 0..NEIGHBOUR_COUNT {
            let Some(neighbour) = cells.neighbour(address, direction) else {
                continue;
            };
            let Some(at) = cells.index_of(neighbour) else {
                continue;
            };
            let Some(other) = lookup.get(&at.0) else {
                continue;
            };
            steps += (value - other).abs();
            counted += 1;
        }
    }
    let step = steps / counted.max(1);
    // The roughness, in thousandths. A plane whose neighbours differ by as
    // much as the plane varies over the world reads one thousand.
    let ratio = step * 1000 / spread.max(1);
    (spread, step, ratio)
}

fn main() {
    let extent = argument(1, 128) as u32;
    let seed = argument(2, 0x2f);
    let bits = argument(3, 0) as u32;
    let settle = argument(4, 400);
    let scale = WeatherScale::from_bits(bits).expect("the scale describes a lattice");
    let mut world = World::with_weather_scale(
        WorldConfig {
            width: extent,
            height: extent,
            seed,
            faction_count: 1,
            unit_capacity: 64,
        },
        scale,
    )
    .expect("the settings describe a world");

    for _ in 0..settle {
        world.step(THREADS).expect("the step must run");
    }

    let lattice = world.weather().lattice();
    let inner = lattice.inner();
    let side = scale.side();

    // Every plane is read at the padded cell index and never at a world
    // address, and the walk covers the world and not the margin.[^1]
    //
    // [^1]: Findings register, FND-569. `docs/FINDINGS.md`
    let mut warmth = Vec::new();
    let mut ground = Vec::new();
    let mut air = Vec::new();
    let mut relief = Vec::new();
    for index in 0..inner.tile_count() {
        let Some(address) = inner.address_of(TileIdx(index)) else {
            continue;
        };
        let Some(cell) = lattice.whole_of_inner(index) else {
            continue;
        };
        let field = world.weather();
        warmth.push((cell, i64::from(field.warmth_at(cell))));
        ground.push((cell, field.ground_at(cell).0));
        air.push((cell, field.air_at(cell).0));
        // The terrain under the cell, as the mean height of every tile it
        // covers. It is the contrast the world offers before the field mixes.
        let mut height = 0i64;
        let mut tiles = 0i64;
        for row in 0..side {
            for column in 0..side {
                let at = Axial::new(
                    address.q * side as i32 + column as i32,
                    address.r * side as i32 + row as i32,
                );
                let Some(tile) = world.tile_terrain(at) else {
                    continue;
                };
                height += i64::from(tile.height.0);
                tiles += 1;
            }
        }
        relief.push((cell, height / tiles.max(1)));
    }

    println!("extent {extent} seed {seed:#x} scale bits {bits} settle {settle}");
    println!(
        "world {} by {} cells, margin {}",
        inner.width(),
        inner.height(),
        lattice.ring()
    );
    println!();
    println!("  plane        spread over the world   mean step to a neighbour   roughness/1000");
    for (name, values) in [
        ("terrain", &relief),
        ("warmth", &warmth),
        ("air", &air),
        ("ground", &ground),
    ] {
        let (spread, step, ratio) = roughness(values, &world);
        println!("  {name:<12} {spread:>21} {step:>26} {ratio:>16}");
    }
    println!();
    println!(
        "The terrain is the control: it is generated and never mixed. A weather plane \
         whose roughness is far below the terrain's has lost contrast the world offered."
    );
}
