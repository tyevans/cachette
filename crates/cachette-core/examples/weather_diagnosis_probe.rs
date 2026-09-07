//! A probe that measures the four complaints about the weather field.
//!
//! This is a diagnostic, not a test. It reports the column structure of the
//! temperature and the wind, the difference between the sea and the land, the
//! rotation of the wind field, and the distribution of the air.
//!
//! Run it with `cargo run -p cachette-core --release --example
//! weather_diagnosis_probe`. The arguments are the extent, the seed, the
//! ticks, and the weather scale.

use cachette_core::hex::{Axial, NEIGHBOURS, NEIGHBOUR_COUNT};
use cachette_core::{WeatherScale, World, WorldConfig};

fn argument(position: usize, fallback: u64) -> u64 {
    std::env::args()
        .nth(position)
        .and_then(|text| text.parse::<u64>().ok())
        .unwrap_or(fallback)
}

/// Returns the circulation of the wind around each cell, as a sum over the
/// six neighbours of the part of the neighbour's wind that runs along the
/// ring. A closed circulation gives a large value of one sign.
fn circulation(world: &World) -> Vec<i64> {
    let field = world.weather();
    let cells = field.cells();
    let wind = field.wind_plane();
    let mut out = vec![0i64; wind.len()];
    for (index, slot) in out.iter_mut().enumerate() {
        let Some(address) = cells.address_of(cachette_core::TileIdx(index as u32)) else {
            continue;
        };
        let mut sum = 0i64;
        for direction in 0..NEIGHBOUR_COUNT {
            let Some(neighbour) = cells.neighbour(address, direction) else {
                continue;
            };
            let Some(at) = cells.index_of(neighbour) else {
                continue;
            };
            // The tangent of the ring at this neighbour is the next step
            // round, so the circulation is the wind there along that step.
            let tangent = NEIGHBOURS[(direction + 2) % NEIGHBOUR_COUNT];
            let there = wind[at.0 as usize];
            sum += i64::from(there.q) * i64::from(tangent.q)
                + i64::from(there.r) * i64::from(tangent.r);
        }
        *slot = sum;
    }
    out
}

fn main() {
    let extent = argument(1, 256) as u32;
    let seed = argument(2, 0x2f);
    let ticks = argument(3, 400) as u32;
    let bits = argument(4, 5) as u32;
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

    println!("extent {extent} seed {seed:#x} ticks {ticks} scale bits {bits}");
    let cells = world.weather().cells();
    let (wide, high) = (cells.width(), cells.height());
    println!("lattice {wide} by {high} cells");
    println!(
        "season period {} ticks, pressure divisor {}, transport passes {}",
        cachette_core::weather::SEASON_PERIOD_TICKS,
        scale.pressure_divisor(),
        scale.transport_passes()
    );

    for tick in 1..=ticks {
        world.step(1).expect("the step must run");
        if tick % (ticks / 4).max(1) != 0 {
            continue;
        }
        let field = world.weather();
        let warmth = field.warmth_plane().to_vec();
        let wind = field.wind_plane().to_vec();
        let air = field.air_plane().to_vec();

        // The column profile of the temperature: the mean over each column,
        // and the largest jump between two neighbouring columns.
        let mut column_mean = vec![0i64; wide as usize];
        for column in 0..wide {
            let mut sum = 0i64;
            for row in 0..high {
                if let Some(at) = cells.index_of(Axial::new(column as i32, row as i32)) {
                    sum += i64::from(warmth[at.0 as usize]);
                }
            }
            column_mean[column as usize] = sum / i64::from(high);
        }
        let jump = column_mean
            .windows(2)
            .map(|pair| (pair[1] - pair[0]).abs())
            .max()
            .unwrap_or(0);
        // The same for rows, so a reader can tell a column artefact from a
        // field that simply varies.
        let mut row_mean = vec![0i64; high as usize];
        for row in 0..high {
            let mut sum = 0i64;
            for column in 0..wide {
                if let Some(at) = cells.index_of(Axial::new(column as i32, row as i32)) {
                    sum += i64::from(warmth[at.0 as usize]);
                }
            }
            row_mean[row as usize] = sum / i64::from(wide);
        }
        let row_jump = row_mean
            .windows(2)
            .map(|pair| (pair[1] - pair[0]).abs())
            .max()
            .unwrap_or(0);

        let curl = circulation(&world);
        let strongest = curl.iter().copied().map(i64::abs).max().unwrap_or(0);
        let mut sorted: Vec<i64> = curl.iter().copied().map(i64::abs).collect();
        sorted.sort_unstable();
        let at = |share: usize| sorted[sorted.len() * share / 100];
        let turning = curl.iter().filter(|value| value.abs() >= 4).count();
        let cyclonic = curl.iter().filter(|value| **value > 0).count();
        let mut air_sorted: Vec<i64> = air.iter().map(|drops| drops.0).collect();
        air_sorted.sort_unstable();
        let air_at = |share: usize| {
            if air_sorted.is_empty() {
                0
            } else {
                air_sorted[air_sorted.len() * share / 100]
            }
        };

        let air_total: i64 = air.iter().map(|drops| drops.0).sum();
        let air_high = air.iter().map(|drops| drops.0).max().unwrap_or(0);
        let air_cells = air.iter().filter(|drops| drops.0 > 0).count();
        let air_over_64 = air.iter().filter(|drops| drops.0 >= 64).count();
        let air_over_256 = air.iter().filter(|drops| drops.0 >= 256).count();
        let fastest = wind.iter().map(|w| w.speed()).max().unwrap_or(0);
        let still = wind.iter().filter(|w| w.is_still()).count();

        println!("--- tick {tick}");
        println!("  temperature row means (latitude) {row_mean:?}");
        println!("  largest column jump {jump}, largest row jump {row_jump}");
        println!(
            "  air total {air_total}, high {air_high}, cells with any {air_cells} of {}, over 64 {air_over_64}, over 256 {air_over_256}",
            air.len()
        );
        println!(
            "  fastest wind {fastest}, still cells {still} of {}",
            wind.len()
        );
        println!(
            "  circulation: median {} p90 {} p99 {} strongest {strongest}, turning {turning}, cyclonic {cyclonic}",
            at(50),
            at(90),
            at(99)
        );
        println!(
            "  air: median {} p90 {} p99 {} p999 {} high {air_high}",
            air_at(50),
            air_at(90),
            air_at(99),
            if air_sorted.is_empty() {
                0
            } else {
                air_sorted[air_sorted.len() * 999 / 1000]
            }
        );

        // The modal wind heading of each column and of each row. A watcher
        // of the wind overlay reads one heading as one colour, so a run of
        // one heading over many cells of one column is a vertical band.
        for (axis, along, across) in [("column", wide, high), ("row", high, wide)] {
            let mut modal = String::new();
            let mut flips = 0;
            let mut last = usize::MAX;
            for line in 0..along {
                let mut counts = [0u32; NEIGHBOUR_COUNT + 1];
                for other in 0..across {
                    let address = if axis == "column" {
                        Axial::new(line as i32, other as i32)
                    } else {
                        Axial::new(other as i32, line as i32)
                    };
                    if let Some(at) = cells.index_of(address) {
                        match wind[at.0 as usize].heading() {
                            None => counts[NEIGHBOUR_COUNT] += 1,
                            Some(heading) => counts[heading] += 1,
                        }
                    }
                }
                let best = (0..=NEIGHBOUR_COUNT).max_by_key(|k| counts[*k]).unwrap();
                let share = counts[best] * 100 / across.max(1);
                if best != last {
                    flips += 1;
                    last = best;
                }
                modal.push_str(&format!("{best}:{share} "));
            }
            println!("  modal heading of each {axis} ({flips} runs): {modal}");
        }
    }
}

// The probe above reports the field. The helper below reports the modal wind
// heading of each column, which is what a watcher of the wind overlay sees as
// a band of one colour.
