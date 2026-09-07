//! A probe that measures one storm from its birth to its end.
//!
//! This is a diagnostic and not a test. It places one storm on a settled
//! field, runs the world until the storm dies, and reports the track, the
//! life, the wind and the rain under it, and what the field looked like
//! before and after.
//!
//! **The probe prints the width of a cell in kilometres.** A weather figure
//! taken from a world whose cell is wider than the process it measures says
//! nothing about that process, so a reader needs the pitch beside every
//! figure.[^1]
//!
//! Run it with `cargo run -p cachette-core --release --example cyclone_probe`.
//! The arguments are the extent, the seed, the weather
//! scale bits, the ticks to settle for, and the setting, which is `severe` or
//! `tropical`.
//!
//! # References
//!
//! [^1]: Findings register, FND-618. `docs/FINDINGS.md`

use cachette_core::weather::{cell_ground_of, ground_over_lattice};
use cachette_core::{
    Axial, CycloneSetting, TileIdx, WeatherScale, World, WorldConfig, LATITUDE_POLE,
};

/// The threads that one step runs on.
const THREADS: usize = 1;

/// The kilometres that one degree of latitude spans on the globe.
const KM_FOR_EACH_DEGREE: i64 = 111;

fn argument(position: usize, fallback: u64) -> u64 {
    std::env::args()
        .nth(position)
        .and_then(|text| match text.strip_prefix("0x") {
            Some(rest) => u64::from_str_radix(rest, 16).ok(),
            None => text.parse::<u64>().ok(),
        })
        .unwrap_or(fallback)
}

/// What the field looked like at one moment.
struct Picture {
    wind: i64,
    air: i64,
    ground: i64,
    peak_wind: i64,
}

fn picture(world: &World) -> Picture {
    let field = world.weather();
    let winds = field.wind_plane();
    let mut wind = 0i64;
    let mut peak_wind = 0i64;
    for one in winds {
        let speed = i64::from(one.speed());
        wind += speed;
        peak_wind = peak_wind.max(speed);
    }
    let count = winds.len().max(1) as i64;
    Picture {
        wind: wind / count,
        air: field.air_total().0,
        ground: field.ground_total().0,
        peak_wind,
    }
}

fn main() {
    let extent = argument(1, 256) as u32;
    let seed = argument(2, 0x2f);
    let bits = argument(3, 0) as u32;
    let settle = argument(4, 400);
    let name = std::env::args().nth(5).unwrap_or_else(|| "tropical".into());
    let setting = match name.as_str() {
        "severe" => CycloneSetting::SEVERE,
        _ => CycloneSetting::TROPICAL,
    };

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

    let lattice = world.weather_lattice();
    let layout = world.weather_layout();
    let cells = lattice.whole();
    let rows = i64::from(lattice.inner().height());
    let latitudes = world.weather().latitudes();
    let span = i64::from(2 * LATITUDE_POLE) / 100;
    let km = span * KM_FOR_EACH_DEGREE / rows.max(1);
    println!("world {extent} rows, seed {seed:#x}, scale bits {bits}, setting {name}");
    println!(
        "lattice {} by {} cells, world rows {rows}, latitude span {span} degrees",
        cells.width(),
        cells.height()
    );
    println!(
        "one cell spans about {km} km, and one tile spans about {} km",
        km / i64::from(layout.block_edge()).max(1)
    );
    let _ = latitudes;

    for _ in 0..settle {
        world.step(THREADS).expect("the step must run");
    }
    let before = picture(&world);
    println!(
        "before: mean wind {} of 48, peak wind {}, air {}, ground {}",
        before.wind, before.peak_wind, before.air, before.ground
    );

    // The storm goes on the warmest sea cell of the world, because that is
    // where a real one would stand. The probe reads the ground of every cell
    // once, in the way the field does.
    let under = ground_over_lattice(lattice, layout, world.terrain());
    let warmth = world.weather().warmth_plane().to_vec();
    let mut best = None;
    let mut best_heat = i32::MIN;
    for cell in 0..under.len() {
        let ground = under[cell];
        if ground.tiles() <= 0 || ground.open_tiles() * 2 > ground.tiles() {
            continue;
        }
        let Some(inner) = lattice.inner_of_whole(cell as u32) else {
            continue;
        };
        let _ = inner;
        let heat = warmth.get(cell).copied().unwrap_or(0);
        if heat > best_heat {
            best_heat = heat;
            best = Some(cell as u32);
        }
    }
    let cell = best.expect("the world holds a sea cell");
    let address = cells
        .address_of(TileIdx(cell))
        .expect("the cell is on the lattice");
    // A tile of that cell, which is what the authoring verb names.
    let inner = lattice
        .inner_address_of(cell)
        .expect("the cell covers the world");
    let place = Axial::new(
        inner.q * layout.block_edge() as i32,
        inner.r * layout.block_edge() as i32,
    );
    println!(
        "raising over cell {cell} at ({}, {}), tile ({}, {}), warmth {best_heat}",
        address.q, address.r, place.q, place.r
    );

    let raised = world
        .raise_cyclone(place, setting)
        .expect("the verb must place a storm");
    println!(
        "raised storm {} depth {} radius {} life {}",
        raised.id, raised.depth, raised.radius, raised.life
    );

    let start = raised.eye();
    let mut track = 0i64;
    let mut previous = start;
    let mut furthest = 0i64;
    let mut lived = 0u64;
    let mut peak_under = 0i64;
    let mut rain_under = 0i64;
    let mut deepest = raised.depth;
    let mut ambient_rain = 0i64;
    let mut under_cells = 0i64;
    let mut out_cells = 0i64;
    let mut wind_under = 0i64;
    let mut wind_out = 0i64;
    let mut steps = 0i64;
    loop {
        let ground_before: Vec<i64> = world
            .weather()
            .ground_plane()
            .iter()
            .map(|drops| drops.0)
            .collect();
        world.step(THREADS).expect("the step must run");
        steps += 1;
        let field = world.weather();
        let Some(storm) = field.cyclones().iter().find(|one| one.id == raised.id) else {
            break;
        };
        lived += 1;
        let eye = storm.eye();
        track += i64::from(previous.distance(eye));
        previous = eye;
        furthest = furthest.max(i64::from(start.distance(eye)));
        deepest = deepest.max(storm.depth);
        // The rain under the storm is what the ground of its footprint gained
        // this frame. Everything else is the ambient rain of the world.
        let after = field.ground_plane();
        for at in 0..after.len() {
            let gained = after[at].0 - ground_before.get(at).copied().unwrap_or(0);
            if gained <= 0 {
                continue;
            }
            if field.depression_at(at as u32) > 0 {
                rain_under += gained;
            } else {
                ambient_rain += gained;
            }
        }
        for at in 0..field.wind_plane().len() {
            let speed = i64::from(field.wind_plane()[at].speed());
            if field.depression_at(at as u32) > 0 {
                peak_under = peak_under.max(speed);
                wind_under += speed;
                under_cells += 1;
            } else {
                wind_out += speed;
                out_cells += 1;
            }
        }
        if steps > 4096 {
            break;
        }
    }
    println!(
        "the storm lived {lived} ticks, walked {track} cells about {} km, and reached {furthest} cells about {} km from where it started",
        track * km,
        furthest * km
    );
    println!("its deepest eye was {deepest}, and the fastest wind under it was {peak_under} of 48");
    println!(
        "it covered {under_cells} cell readings over {steps} ticks, of {} on the lattice",
        under_cells + out_cells
    );
    println!(
        "the rain under it ran {} drops for each cell and tick, against {} elsewhere",
        rain_under / under_cells.max(1),
        ambient_rain / out_cells.max(1)
    );
    println!(
        "the wind under it ran {} of 48, against {} elsewhere",
        wind_under / under_cells.max(1),
        wind_out / out_cells.max(1)
    );

    // The field must come back. A storm that leaves the ambient wind pinned
    // or wrecked is a defect, whatever it did while it stood.
    for _ in 0..settle {
        world.step(THREADS).expect("the step must run");
    }
    let after = picture(&world);
    println!(
        "after: mean wind {} of 48, peak wind {}, air {}, ground {}",
        after.wind, after.peak_wind, after.air, after.ground
    );
    println!(
        "the field carries {} storms, raised {} in all, and the water account {}",
        world.cyclones().len(),
        world.weather().cyclones_raised(),
        if world.weather().check_account() {
            "balances"
        } else {
            "does not balance"
        }
    );
    let _ = cell_ground_of;
}
