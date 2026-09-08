//! A probe that measures the row structure of the weather field.
//!
//! This is a diagnostic, not a test. The project owner reports three tight
//! moisture bands that stand at the same place whatever extent the world
//! takes, and reports that the field stops changing once they form. This
//! probe answers all three in numbers.
//!
//! It prints one line for each row of the weather lattice. The line holds the
//! latitude of the row, the mean temperature, the mean air, the mean cloud
//! share, the mean wind along each axis, and the mean absolute deviation of
//! the air from the mean of its own row. The last column is the zonal
//! variability. A field that is constant along every row prints zero there,
//! and a perfect band is that field.
//!
//! It also prints, at a set of stops, the mean absolute change of the air
//! plane over one tick. A field at a fixed point prints zero.
//!
//! Run it with `cargo run -p cachette-core --release --example
//! weather_band_probe`. The arguments are the extent, the seed, the ticks,
//! and the weather scale in bits.
//!
//! Every figure is an integer.

use cachette_core::{WeatherScale, World, WorldConfig};

/// Reads one command line argument, in decimal or in hexadecimal.
fn argument(position: usize, fallback: u64) -> u64 {
    std::env::args()
        .nth(position)
        .and_then(|text| match text.strip_prefix("0x") {
            Some(rest) => u64::from_str_radix(rest, 16).ok(),
            None => text.parse::<u64>().ok(),
        })
        .unwrap_or(fallback)
}

/// The ticks at which the probe reports the change of the field.
const STOPS: [u32; 9] = [1, 2, 4, 8, 16, 64, 256, 1024, 4096];

/// Returns the mean of a run of numbers, or zero when the run is empty.
fn mean(values: &[i64]) -> i64 {
    if values.is_empty() {
        0
    } else {
        values.iter().sum::<i64>() / values.len() as i64
    }
}

/// Returns the mean absolute deviation of a run of numbers from its own mean.
fn spread(values: &[i64]) -> i64 {
    if values.is_empty() {
        return 0;
    }
    let middle = mean(values);
    values
        .iter()
        .map(|value| (value - middle).abs())
        .sum::<i64>()
        / values.len() as i64
}

/// What the probe gathers over one row of the lattice.
struct Row {
    latitude: i32,
    warmth: Vec<i64>,
    air: Vec<i64>,
    cloud: Vec<i64>,
    across: Vec<i64>,
    down: Vec<i64>,
}

impl Row {
    fn new(latitude: i32) -> Self {
        Self {
            latitude,
            warmth: Vec::new(),
            air: Vec::new(),
            cloud: Vec::new(),
            across: Vec::new(),
            down: Vec::new(),
        }
    }
}

/// Prints one line for each row of the world, over the cells of that row.
fn say_rows(world: &World) {
    let field = world.weather();
    let lattice = field.lattice();
    let inner = lattice.inner();
    let height = inner.height();
    let mut rows: Vec<Row> = (0..height)
        .map(|row| Row::new(field.latitudes().of_row(row, height)))
        .collect();
    for cell in lattice.inner_cells() {
        let row = lattice.inner_row_of(cell) as usize;
        let Some(slot) = rows.get_mut(row) else {
            continue;
        };
        let wind = field.wind_at(cell);
        slot.warmth.push(i64::from(field.warmth_at(cell)));
        slot.air.push(field.air_at(cell).0);
        slot.cloud.push(field.cloud_share_at(cell));
        slot.across.push(i64::from(wind.q));
        slot.down.push(i64::from(wind.r));
    }
    println!(
        "{:>4} {:>8} {:>7} {:>9} {:>7} {:>7} {:>7} {:>9}",
        "row", "latitude", "warmth", "air", "cloud", "windq", "windr", "airspread"
    );
    for (index, row) in rows.iter().enumerate() {
        println!(
            "{:>4} {:>8} {:>7} {:>9} {:>7} {:>7} {:>7} {:>9}",
            index,
            row.latitude,
            mean(&row.warmth),
            mean(&row.air),
            mean(&row.cloud),
            mean(&row.across),
            mean(&row.down),
            spread(&row.air),
        );
    }
}

/// The low end of the belt that the summary reports, in hundredths of a
/// degree of latitude.
const BELT_LOW: i32 = 30 * 100;

/// The high end of that belt.
const BELT_HIGH: i32 = 70 * 100;

/// Prints the zonal variability of the air over the mid-latitude belt.
///
/// **The belt is where a real field carries most of its travelling weather,
/// so it is the place to ask whether this one carries any.** The figure is
/// the mean absolute deviation of the air from the mean of its own row,
/// against the mean air of the belt. A field that holds a perfect band prints
/// nothing there.
fn say_belt(world: &World) {
    let field = world.weather();
    let lattice = field.lattice();
    let height = lattice.inner().height();
    let mut level: Vec<i64> = Vec::new();
    let mut deviation: Vec<i64> = Vec::new();
    for row in 0..height {
        let latitude = field.latitudes().of_row(row, height).abs();
        if latitude < BELT_LOW || latitude > BELT_HIGH {
            continue;
        }
        let air: Vec<i64> = lattice
            .inner_cells()
            .into_iter()
            .filter(|cell| lattice.inner_row_of(*cell) == row)
            .map(|cell| field.air_at(cell).0)
            .collect();
        if air.is_empty() {
            continue;
        }
        level.push(mean(&air));
        deviation.push(spread(&air));
    }
    let air = mean(&level);
    let apart = mean(&deviation);
    let share = if air == 0 { 0 } else { apart * 100 / air };
    println!(
        "belt rows {} air {air} zonal {apart} share {share}%",
        level.len()
    );
}

/// Prints the latitude and the depth of every standing storm.
fn say_storms(world: &World) {
    let field = world.weather();
    let lattice = field.lattice();
    let height = lattice.inner().height();
    let mut said: Vec<String> = Vec::new();
    for storm in field.cyclones() {
        let row = storm.eye().r.max(0) as u32;
        let inner = row
            .saturating_sub(lattice.ring())
            .min(height.saturating_sub(1));
        let latitude = field.latitudes().of_row(inner, height);
        said.push(format!("{latitude}/{}/{}", storm.depth, storm.radius));
    }
    println!(
        "storms {} as latitude/depth/radius {}",
        said.len(),
        said.join(" ")
    );
}

/// Returns the air of every cell of the whole lattice.
fn air_of(world: &World) -> Vec<i64> {
    world
        .weather()
        .air_plane()
        .iter()
        .map(|drops| drops.0)
        .collect()
}

/// Returns the plane less the mean of each row.
///
/// **The season and the belt are both latitude terms, and the row mean holds
/// the whole of them.** What is left is the part of the field that varies
/// along a row. A field forced only by the latitude and by the static ground
/// holds one such plane for ever, whatever the season does.
fn anomaly_of(plane: &[i64], width: usize) -> Vec<i64> {
    if width == 0 {
        return plane.to_vec();
    }
    let mut out = Vec::with_capacity(plane.len());
    for row in plane.chunks(width) {
        let middle = mean(row);
        out.extend(row.iter().map(|value| value - middle));
    }
    out
}

/// Returns the mean absolute difference between two planes of equal length.
fn apart(first: &[i64], second: &[i64]) -> i64 {
    if first.is_empty() || first.len() != second.len() {
        return 0;
    }
    first
        .iter()
        .zip(second.iter())
        .map(|(one, other)| (one - other).abs())
        .sum::<i64>()
        / first.len() as i64
}

/// The lags at which the probe compares the anomaly plane against itself.
const LAGS: [usize; 4] = [1, 64, 512, 2048];

fn main() {
    let extent = argument(1, 96) as u32;
    let seed = argument(2, 0x2f);
    let ticks = argument(3, 4096) as u32;
    let bits = argument(4, 0) as u32;
    let scale = WeatherScale::from_bits(bits).expect("the scale describes a lattice");
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
    let lattice = world.weather().lattice();
    println!(
        "extent {extent} seed {seed:#x} ticks {ticks} bits {bits} lattice {} by {} margin {}",
        lattice.inner().width(),
        lattice.inner().height(),
        lattice.ring()
    );
    let width = lattice.whole().width() as usize;
    let longest = LAGS.iter().copied().max().unwrap_or(1);
    let mut history: Vec<Vec<i64>> = Vec::new();
    let mut change: Vec<(u32, i64, i64, [i64; LAGS.len()], usize, u32)> = Vec::new();
    for tick in 1..=ticks {
        world.step(1).expect("the step must run");
        let now = anomaly_of(&air_of(&world), width);
        history.push(now.clone());
        if history.len() > longest + 1 {
            history.remove(0);
        }
        if !STOPS.contains(&tick) && tick != ticks {
            continue;
        }
        let mut lagged = [0i64; LAGS.len()];
        for (slot, lag) in lagged.iter_mut().zip(LAGS.iter()) {
            if history.len() > *lag {
                *slot = apart(&now, &history[history.len() - 1 - lag]);
            }
        }
        change.push((
            tick,
            mean(&air_of(&world)),
            apart(&now, &vec![0i64; now.len()]),
            lagged,
            world.weather().cyclones().len(),
            world.weather().cyclones_raised(),
        ));
    }
    println!(
        "{:>6} {:>10} {:>10} {:>8} {:>8} {:>8} {:>8} {:>7} {:>7}",
        "tick", "airmean", "spread", "lag1", "lag64", "lag512", "lag2048", "storms", "raised"
    );
    for (tick, level, deviation, lagged, standing, raised) in &change {
        println!(
            "{tick:>6} {level:>10} {deviation:>10} {:>8} {:>8} {:>8} {:>8} {standing:>7} {raised:>7}",
            lagged[0], lagged[1], lagged[2], lagged[3]
        );
    }
    say_storms(&world);
    say_belt(&world);
    say_rows(&world);
}
