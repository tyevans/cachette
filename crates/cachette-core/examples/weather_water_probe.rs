//! A probe that decomposes the water budget of the field by latitude.
//!
//! This is a diagnostic, not a test. One blocker records that the subtropics
//! receive about one percent of the rain the equator receives, and no
//! measurement says which term carries that.[^1] This probe reads the source,
//! the store and the sink of each latitude band so that a reader can tell
//! them apart.
//!
//! Run it with `cargo run -p cachette-core --release --example
//! weather_water_probe`. The arguments are the extent, the seed, the weather
//! scale and the ticks to settle for.
//!
//! # References
//!
//! [^1]: Blockers register, BLK-155. `docs/BLOCKERS.md`

use cachette_core::hex::Axial;
use cachette_core::{TileIdx, WeatherScale, World, WorldConfig};
use cachette_core::{LATITUDE_FINE, WARMTH_FINE, WARMTH_FLOOR};

/// The threads that one step runs on.
const THREADS: usize = 4;

/// The latitude bands the probe reports.
const BANDS: u32 = 12;

fn argument(position: usize, fallback: u64) -> u64 {
    std::env::args()
        .nth(position)
        .and_then(|text| match text.strip_prefix("0x") {
            Some(rest) => u64::from_str_radix(rest, 16).ok(),
            None => text.parse::<u64>().ok(),
        })
        .unwrap_or(fallback)
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
    let high = inner.height();
    let latitudes = world.weather().latitudes();
    let side = scale.side();

    // Each row of the table is one latitude band. The walk goes over the
    // world and reads every plane at the padded cell index, never at a world
    // address.[^1]
    //
    // [^1]: Findings register, FND-569. `docs/FINDINGS.md`
    let mut warmth = vec![0i64; BANDS as usize];
    let mut capacity = vec![0i64; BANDS as usize];
    let mut air = vec![0i64; BANDS as usize];
    let mut ground = vec![0i64; BANDS as usize];
    let mut open = vec![0i64; BANDS as usize];
    let mut tiles = vec![0i64; BANDS as usize];
    let mut counted = vec![0i64; BANDS as usize];
    // The same store, over the cells that hold more land than water. The
    // Köppen grade reads only these, and a band that is mostly ocean grades
    // on a small and unrepresentative sample of itself.
    let mut land_ground = vec![0i64; BANDS as usize];
    let mut land_air = vec![0i64; BANDS as usize];
    let mut land_counted = vec![0i64; BANDS as usize];

    let field = world.weather();
    for index in 0..inner.tile_count() {
        let Some(address) = inner.address_of(TileIdx(index)) else {
            continue;
        };
        let Some(cell) = lattice.whole_of_inner(index) else {
            continue;
        };
        let band = ((address.r.max(0) as u32) * BANDS / high.max(1)).min(BANDS - 1) as usize;
        warmth[band] += i64::from(field.warmth_at(cell));
        capacity[band] += field.capacity_at_cell(cell).0;
        air[band] += field.air_at(cell).0;
        ground[band] += field.ground_at(cell).0;
        counted[band] += 1;
        let mut here_land = 0i64;
        let mut here_tiles = 0i64;
        for row in 0..side {
            for column in 0..side {
                let at = Axial::new(
                    address.q * side as i32 + column as i32,
                    address.r * side as i32 + row as i32,
                );
                let Some(tile) = world.tile_terrain(at) else {
                    continue;
                };
                tiles[band] += 1;
                here_tiles += 1;
                if tile.kind.is_passable() {
                    here_land += 1;
                } else {
                    open[band] += 1;
                }
            }
        }
        if here_tiles > 0 && here_land * 2 > here_tiles {
            land_ground[band] += field.ground_at(cell).0;
            land_air[band] += field.air_at(cell).0;
            land_counted[band] += 1;
        }
    }

    println!("extent {extent} seed {seed:#x} scale bits {bits} settle {settle}");
    println!(
        "world {} by {} cells, margin {}, latitude span {} degrees",
        inner.width(),
        high,
        lattice.ring(),
        latitudes.span() / LATITUDE_FINE
    );
    println!();
    println!(
        "  band  latitude   cells  open%   mean C  capacity     air   air/cap%    ground  \
         ground/air%    land   land air  land ground"
    );
    for band in 0..BANDS as usize {
        if counted[band] == 0 {
            continue;
        }
        let n = counted[band];
        let degrees =
            (warmth[band] / n * i64::from(WARMTH_FINE) + i64::from(WARMTH_FLOOR))
                / i64::from(LATITUDE_FINE);
        let cap = capacity[band] / n;
        let vapour = air[band] / n;
        let wet = ground[band] / n;
        let middle = (band as u32 * high / BANDS + (band as u32 + 1) * high / BANDS) / 2;
        let latitude = i64::from(latitudes.of_row(middle, high)) / i64::from(LATITUDE_FINE);
        print!(
            "  {band:>4}  {latitude:>8}  {n:>6}  {:>5}  {degrees:>7}  {cap:>8}  {vapour:>6}  \
             {:>9}  {wet:>8}  {:>11}",
            open[band] * 100 / tiles[band].max(1),
            vapour * 100 / cap.max(1),
            wet * 100 / vapour.max(1)
        );
        let _ = ();
        let ln = land_counted[band].max(1);
        print!(
            "  {:>6}  {:>8}  {:>11}",
            land_counted[band],
            land_air[band] / ln,
            land_ground[band] / ln
        );
        println!();
    }
}
