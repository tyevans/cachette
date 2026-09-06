//! A probe that reports the season as a picture of the world.
//!
//! This is a diagnostic, not a test. For a set of latitudes from the equator
//! to the pole it reports the degrees the sun gives at midsummer, at
//! midwinter, and the difference between the two. It then counts the tiles
//! that see a swing above a stated amount.
//!
//! Run it with `cargo run -p cachette-core --release --example
//! weather_season_probe`. The arguments are the extent, the weather scale,
//! and the swing that counts as a real season.

use cachette_core::weather::{season_at, SEASON_PERIOD_TICKS};
use cachette_core::{Tick, WeatherScale};

fn argument(position: usize, fallback: u64) -> u64 {
    std::env::args()
        .nth(position)
        .and_then(|text| text.parse::<u64>().ok())
        .unwrap_or(fallback)
}

fn main() {
    let extent = argument(1, 256) as u32;
    let bits = argument(2, 0) as u32;
    let mark = argument(3, 60) as i32;
    let scale = WeatherScale::from_bits(bits).expect("the scale describes a lattice");
    let rows = extent / scale.side();
    // The sun stands at one limit a quarter of the way through the swing and
    // at the other three quarters of the way through it.
    let summer = Tick((SEASON_PERIOD_TICKS / 4) as u64);
    let winter = Tick((SEASON_PERIOD_TICKS * 3 / 4) as u64);

    println!("extent {extent} scale bits {bits}, {rows} rows, swing mark {mark}");
    println!("  row  latitude   midsummer  midwinter  swing  mean");
    let mut swinging = 0u32;
    let mut coldest_summer = i32::MAX;
    for row in 0..rows {
        let north = season_at(summer, row, rows, scale);
        let south = season_at(winter, row, rows, scale);
        let swing = (north - south).abs();
        if swing >= mark {
            swinging += 1;
        }
        // The pole is the last row. Its midsummer heat is what decides
        // whether a pole stays cold all year.
        if row + 1 == rows || row == 0 {
            coldest_summer = coldest_summer.min(north.max(south));
        }
        if rows <= 16 || row % (rows / 16) == 0 || row + 1 == rows {
            let latitude = row as i64 * scale.side_tiles() - i64::from(extent) / 2;
            println!(
                "  {row:>4} {latitude:>9}   {north:>9}  {south:>9} {swing:>6} {:>5}",
                (north + south) / 2
            );
        }
    }
    let tiles_for_each_row = i64::from(extent) * scale.side_tiles();
    println!(
        "rows with a swing of {mark} or more: {swinging} of {rows}, which is {} tiles of {}",
        i64::from(swinging) * tiles_for_each_row,
        i64::from(extent) * i64::from(extent)
    );
    println!("the warmer pole reaches {coldest_summer} at its own midsummer");
}
