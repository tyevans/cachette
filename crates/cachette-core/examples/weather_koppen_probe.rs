//! A probe that grades the weather against the Köppen climate classification.
//!
//! This is a diagnostic, not a test. The engine now reads the row axis of the
//! world as a latitude and imposes a banded circulation on the pressure, so
//! the map should hold an equatorial rain belt, a desert belt near thirty
//! degrees, and polar ice at each end. The Köppen scheme states its
//! boundaries as arithmetic, so it grades that claim in numbers rather than
//! by eye.[^1]
//!
//! Run it with `cargo run -p cachette-core --release --example
//! weather_koppen_probe`. The arguments are the extent, the seed, the weather
//! scale, the ticks the field settles for, and the ticks between two samples.
//!
//! # What the two axes are
//!
//! **The temperature axis is exact.** The field declares one linear map from
//! the warmth of a cell to a temperature in degrees Celsius, and the probe
//! reads that map. Nothing here restates it.
//!
//! **The precipitation axis is a proxy with a chosen scale.** The field holds
//! the water standing on the ground of a cell, and it does not count the rain
//! that fell. At rest the standing water balances what dries, so it rises and
//! falls with the rain. The probe therefore scales the mean standing water so
//! that the mean over the land of the world equals a stated annual figure,
//! and it grades the pattern rather than the amount. **So a Köppen letter
//! here says where a climate stands against the rest of this world, and it
//! does not say how many millimetres fall.**
//!
//! Every figure here is a whole number.
//!
//! # References
//!
//! [^1]: Research report 30, the published atmospheric math, section 8. `docs/research/reports/30-the-published-atmospheric-math.md`

use cachette_core::hex::Axial;
use cachette_core::{TileIdx, WeatherScale, World, WorldConfig};
use cachette_core::{LATITUDE_FINE, WARMTH_FINE, WARMTH_FLOOR};

/// The months that the probe divides one season period into.
const MONTHS: usize = 12;

/// The threads that one step runs on.
const THREADS: usize = 4;

/// The mean annual precipitation, in millimetres, that the probe gives the
/// land of the world.
///
/// **This is the normaliser of the precipitation axis and not a
/// measurement.** The field holds no rain gauge, so the probe fixes the mean
/// and reads the pattern around it. The figure is near the mean over the land
/// of the Earth.
const MEAN_ANNUAL_RAIN: i64 = 800;

/// The published share of the land of the Earth that each first-letter
/// Köppen class holds, in tenths of a percent, in the order A, B, C, D, E.
///
/// **Published.** The five shares come from the updated world map of the
/// Köppen-Geiger classification, and the register holds three of them
/// already.[^1] [^2]
///
/// # References
///
/// [^1]: Peel, Finlayson and McMahon, updated world map of the Koppen-Geiger climate classification, 2007. Hydrology and Earth System Sciences 11, 1633 to 1644.
/// [^2]: Findings register, FND-616. `docs/FINDINGS.md`
const PUBLISHED_SHARE_FINE: [i64; 5] = [190, 302, 134, 246, 128];

fn argument(position: usize, fallback: u64) -> u64 {
    std::env::args()
        .nth(position)
        .and_then(|text| match text.strip_prefix("0x") {
            Some(rest) => u64::from_str_radix(rest, 16).ok(),
            None => text.parse::<u64>().ok(),
        })
        .unwrap_or(fallback)
}

/// What one cell of the world gathered over one year.
#[derive(Clone)]
struct Record {
    /// The sum of the warmth of the cell in each month.
    warmth: [i64; MONTHS],
    /// The sum of the standing water of the cell in each month.
    water: [i64; MONTHS],
    /// The samples taken in each month.
    counted: [i64; MONTHS],
    /// The row of the world that the cell stands on.
    row: u32,
    /// Whether the cell holds more land than water.
    land: bool,
}

impl Record {
    fn new(row: u32, land: bool) -> Self {
        Self {
            warmth: [0; MONTHS],
            water: [0; MONTHS],
            counted: [0; MONTHS],
            row,
            land,
        }
    }

    /// Returns the temperature of one month, in whole degrees Celsius.
    fn degrees(&self, month: usize) -> i64 {
        let counted = self.counted[month].max(1);
        let warmth = self.warmth[month] / counted;
        (warmth * i64::from(WARMTH_FINE) + i64::from(WARMTH_FLOOR)) / i64::from(LATITUDE_FINE)
    }

    /// Returns the mean standing water of one month.
    fn water_of(&self, month: usize) -> i64 {
        self.water[month] / self.counted[month].max(1)
    }

    fn year_water(&self) -> i64 {
        (0..MONTHS).map(|month| self.water_of(month)).sum()
    }

    fn mean_degrees(&self) -> i64 {
        (0..MONTHS).map(|month| self.degrees(month)).sum::<i64>() / MONTHS as i64
    }
}

/// The Köppen group of one cell.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Group {
    Tropical,
    Desert,
    Steppe,
    Temperate,
    Continental,
    Tundra,
    IceCap,
}

impl Group {
    const ALL: [Self; 7] = [
        Self::Tropical,
        Self::Desert,
        Self::Steppe,
        Self::Temperate,
        Self::Continental,
        Self::Tundra,
        Self::IceCap,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Tropical => "A tropical",
            Self::Desert => "BW desert",
            Self::Steppe => "BS steppe",
            Self::Temperate => "C temperate",
            Self::Continental => "D continental",
            Self::Tundra => "ET tundra",
            Self::IceCap => "EF ice cap",
        }
    }
}

/// Grades one cell against the published Köppen tests.
///
/// The aridity threshold is `20 * MAT + K` millimetres. The high sun months
/// of a cell are the six months around its own summer, which the probe finds
/// from the temperature rather than from the calendar, so the two hemispheres
/// need no separate rule.
///
/// **The boundary between the temperate and the continental class is a
/// parameter, because the published scheme has two variants of it.** One puts
/// it at a coldest month of 0 degrees and the other at −3 degrees. The
/// difference is a convention and not a physical claim, so the probe reports
/// both rather than choosing.
fn grade(record: &Record, rain_for_each_drop: i64, whole: i64, boundary: i64) -> Group {
    let millimetres = |month: usize| record.water_of(month) * rain_for_each_drop / whole;
    let annual: i64 = (0..MONTHS).map(millimetres).sum();
    let mean = record.mean_degrees();
    let coldest = (0..MONTHS)
        .map(|month| record.degrees(month))
        .min()
        .unwrap_or(0);
    let warmest = (0..MONTHS)
        .map(|month| record.degrees(month))
        .max()
        .unwrap_or(0);

    // The six months around the warmest one.
    let peak = (0..MONTHS)
        .max_by_key(|month| record.degrees(*month))
        .unwrap_or(0);
    let high_sun: i64 = (0..6)
        .map(|step| millimetres((peak + MONTHS - 2 + step) % MONTHS))
        .sum();
    let share = if annual > 0 {
        high_sun * 100 / annual
    } else {
        50
    };
    let bonus = if share >= 70 {
        280
    } else if share >= 30 {
        140
    } else {
        0
    };
    let threshold = 20 * mean + bonus;
    if threshold > 0 && annual * 2 < threshold {
        return Group::Desert;
    }
    if threshold > 0 && annual < threshold {
        return Group::Steppe;
    }
    if coldest >= 18 {
        return Group::Tropical;
    }
    if warmest < 10 {
        if warmest < 0 {
            return Group::IceCap;
        }
        return Group::Tundra;
    }
    if coldest >= boundary {
        Group::Temperate
    } else {
        Group::Continental
    }
}

/// Prints the first-letter class table of one set of land cells, and returns
/// the total absolute error against the published shares, in tenths of a
/// point.
///
/// **The five first-letter classes gather the seven groups.** The arid class
/// takes the desert and the steppe, and the polar class takes the tundra and
/// the ice cap.
///
/// **The set normalises within itself.** So the error of a subset says what
/// that land grades against the published shares of the whole Earth, and it is
/// not a part of the error of a larger set. Two subsets do not add up to the
/// whole.
///
/// A reader who wants the figure that the register quotes must weight each
/// latitude band by the land Earth holds in it, and that weighting is not in
/// this tree.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-631. `docs/FINDINGS.md`
fn error_over(
    name: &str,
    members: &[&Record],
    rain_for_each_drop: i64,
    whole: i64,
    boundary: i64,
) -> i64 {
    let count = members.len().max(1);
    let mut totals = [0usize; 7];
    for record in members {
        let group = grade(record, rain_for_each_drop, whole, boundary);
        let at = Group::ALL
            .iter()
            .position(|other| *other == group)
            .unwrap_or(0);
        totals[at] += 1;
    }
    let held = [
        totals[0],
        totals[1] + totals[2],
        totals[3],
        totals[4],
        totals[5] + totals[6],
    ];
    let mut error = 0i64;
    println!();
    println!("over {name}, {} cells:", members.len());
    println!("  class     held    published    apart");
    for (at, published) in PUBLISHED_SHARE_FINE.iter().enumerate() {
        let share = (held[at] * 1000 / count) as i64;
        error += (share - published).abs();
        println!(
            "  {:<8} {:>4}.{:<3} {:>6}.{:<4} {:>4}.{}",
            ["A", "B", "C", "D", "E"][at],
            share / 10,
            share % 10,
            published / 10,
            published % 10,
            (share - published).abs() / 10,
            (share - published).abs() % 10
        );
    }
    println!(
        "  the class error here is {}.{} points",
        error / 10,
        error % 10
    );
    error
}

fn main() {
    let extent = argument(1, 128) as u32;
    let seed = argument(2, 0x2f);
    let bits = argument(3, 0) as u32;
    let settle = argument(4, 400);
    let step = argument(5, 8).max(1);
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

    let period = cachette_core::weather::SEASON_PERIOD_TICKS as u64;
    let lattice = world.weather().lattice();
    let inner = lattice.inner();
    let high = inner.height();
    let latitudes = world.weather().latitudes();

    // **The walk goes over the world and not over the whole lattice.** The
    // lattice carries a margin, and every plane is indexed by the whole
    // lattice, so a reader that takes a world address for a plane index reads
    // the wrong cell by the margin width.
    let mut records: Vec<Record> = Vec::with_capacity(inner.tile_count() as usize);
    let mut whole_of_inner: Vec<u32> = Vec::with_capacity(inner.tile_count() as usize);
    for index in 0..inner.tile_count() {
        let Some(address) = inner.address_of(TileIdx(index)) else {
            continue;
        };
        let Some(cell) = lattice.whole_of_inner(index) else {
            continue;
        };
        // The land share of the cell, read at the tiles it covers.
        let side = scale.side();
        let mut land = 0i64;
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
                tiles += 1;
                if tile.kind.is_passable() {
                    land += 1;
                }
            }
        }
        records.push(Record::new(
            address.r.max(0) as u32,
            tiles > 0 && land * 2 > tiles,
        ));
        whole_of_inner.push(cell);
    }

    // The argument of a step is the thread count. The probe runs one tick at
    // a time and samples every so many ticks.
    for _ in 0..settle {
        world.step(THREADS).expect("the step must run");
    }
    let start = world.tick().0;
    for elapsed in 0..period {
        world.step(THREADS).expect("the step must run");
        if elapsed % step != 0 {
            continue;
        }
        let now = world.tick().0 - start;
        let month = ((now * MONTHS as u64) / period).min(MONTHS as u64 - 1) as usize;
        let field = world.weather();
        for (index, record) in records.iter_mut().enumerate() {
            let cell = whole_of_inner[index];
            record.warmth[month] += i64::from(field.warmth_at(cell));
            record.water[month] += field.ground_at(cell).0;
            record.counted[month] += 1;
        }
    }

    // The normaliser of the precipitation axis. It makes the mean annual
    // figure over the land of the world equal the stated one.
    let land: Vec<&Record> = records.iter().filter(|record| record.land).collect();
    let total: i64 = land.iter().map(|record| record.year_water()).sum();
    let whole = (total / land.len().max(1) as i64).max(1);
    let rain_for_each_drop = MEAN_ANNUAL_RAIN;

    println!("extent {extent} seed {seed:#x} scale bits {bits} settle {settle} step {step}");
    println!(
        "world {} by {} cells, margin {}, latitude span {} degrees",
        inner.width(),
        high,
        lattice.ring(),
        latitudes.span() / LATITUDE_FINE
    );
    println!(
        "the mean land cell holds {whole} drops over a year, and the probe calls that \
         {MEAN_ANNUAL_RAIN} millimetres"
    );

    // **The land of a band is not a sample of the band.** A band that is
    // mostly open water holds its land in a few cells, and those cells sit
    // where the wind of a whole ocean converges. The land figure of such a
    // band says what its islands receive and not what its latitude receives.
    // So the probe reports both, on the same normaliser, and a band where the
    // two disagree is reporting its geography rather than its climate.
    {
        let bands = 12u32;
        println!();
        println!("=== the rain of a band over its land and over the whole of it ===");
        println!();
        println!(
            "  band  latitude  cells   land  land %   land mm   whole mm                land over whole"
        );
        for band in 0..bands {
            let members: Vec<&Record> = records
                .iter()
                .filter(|record| record.row * bands / high.max(1) == band)
                .collect();
            if members.is_empty() {
                continue;
            }
            let dry: Vec<&&Record> = members.iter().filter(|record| record.land).collect();
            let rain = |set: &[&&Record]| -> i64 {
                if set.is_empty() {
                    return 0;
                }
                set.iter()
                    .map(|record| record.year_water() * rain_for_each_drop / whole)
                    .sum::<i64>()
                    / set.len() as i64
            };
            let all: Vec<&&Record> = members.iter().collect();
            let land_mm = rain(&dry);
            let whole_mm = rain(&all);
            let middle = (band * high / bands + (band + 1) * high / bands) / 2;
            let latitude = i64::from(latitudes.of_row(middle, high)) / i64::from(LATITUDE_FINE);
            // The ratio is in hundredths, because this crate holds no
            // floating point number anywhere, probe or engine.[^1]
            //
            // [^1]: ADR-0002, simulated and aggregated state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
            let ratio = land_mm * 100 / whole_mm.max(1);
            println!(
                "  {band:4}  {latitude:8}  {:5}  {:5}  {:5}%  {land_mm:8}  {whole_mm:9}                   {:11}.{:02}",
                members.len(),
                dry.len(),
                dry.len() * 100 / members.len().max(1),
                ratio / 100,
                ratio % 100
            );
        }
    }

    // The two published boundaries between the temperate and the continental
    // class. The probe grades the same world under each.
    for (label, boundary) in [("0 C", 0i64), ("-3 C", -3i64)] {
        let bands = 12u32;
        println!();
        println!("=== the C and D boundary at a coldest month of {label} ===");
        println!();
        println!(
            "  band  latitude  cells   mean C  coldest C  warmest C   rain mm    \
         A  BW  BS   C   D  ET  EF"
        );
        let mut totals = [0usize; 7];
        for band in 0..bands {
            let members: Vec<&Record> = records
                .iter()
                .filter(|record| record.land && record.row * bands / high.max(1) == band)
                .collect();
            if members.is_empty() {
                continue;
            }
            let count = members.len() as i64;
            let mean: i64 = members
                .iter()
                .map(|record| record.mean_degrees())
                .sum::<i64>()
                / count;
            let coldest: i64 = members
                .iter()
                .map(|record| (0..MONTHS).map(|m| record.degrees(m)).min().unwrap_or(0))
                .sum::<i64>()
                / count;
            let warmest: i64 = members
                .iter()
                .map(|record| (0..MONTHS).map(|m| record.degrees(m)).max().unwrap_or(0))
                .sum::<i64>()
                / count;
            let rain: i64 = members
                .iter()
                .map(|record| record.year_water() * rain_for_each_drop / whole)
                .sum::<i64>()
                / count;
            let mut counts = [0usize; 7];
            for record in &members {
                let group = grade(record, rain_for_each_drop, whole, boundary);
                let at = Group::ALL
                    .iter()
                    .position(|other| *other == group)
                    .unwrap_or(0);
                counts[at] += 1;
                totals[at] += 1;
            }
            let middle = (band * high / bands + (band + 1) * high / bands) / 2;
            let latitude = i64::from(latitudes.of_row(middle, high)) / i64::from(LATITUDE_FINE);
            // The share of each class in this band, in whole percent, in the
            // order of `Group::ALL`. A dominant class hides where a band splits.
            let mut spread = String::new();
            for count in counts {
                spread.push_str(&format!("{:>4}", count * 100 / members.len()));
            }
            println!(
                "  {band:>4}  {latitude:>8}  {count:>5}  {mean:>7}  {coldest:>9}  {warmest:>9}  \
             {rain:>8}  {spread}"
            );
        }

        println!();
        println!("over every land cell of the world:");
        let land_count = land.len().max(1);
        for (at, group) in Group::ALL.iter().enumerate() {
            println!(
                "  {:<16} {:>6} cells, {:>3}%",
                group.name(),
                totals[at],
                totals[at] * 100 / land_count
            );
        }

        // **The class error, over the land of this world and not over the
        // land of Earth.** The five first-letter classes gather the seven
        // groups above: B takes the desert and the steppe, and E takes the
        // tundra and the ice cap. The error is the total absolute difference
        // against the published shares, in tenths of a point.
        //
        // A reader who wants the figure the register quotes must weight each
        // latitude band by the land Earth holds in it, and that weighting is
        // not in this tree. **This figure grades the land this world has**,
        // so it compares one run of this probe against another run of it and
        // it does not compare this world with Earth.[^4]
        //
        // [^4]: Findings register, FND-616 and FND-618. `docs/FINDINGS.md`
        let over_all = error_over(
            "the whole land of this world",
            &land,
            rain_for_each_drop,
            whole,
            boundary,
        );

        // **The error is reported over three sets of land, because a change to
        // one belt must not be read as a change to the model.** A term that
        // acts only where a cell freezes changes the polar set and leaves the
        // other one alone. A term that changes the model changes both.
        //
        // The three do not add up. Each set normalises within itself, so each
        // says what the land of that set grades against the published shares
        // and none of them is a part of another.
        let polar: Vec<&Record> = land
            .iter()
            .copied()
            .filter(|record| (latitudes.of_row(record.row, high) / LATITUDE_FINE).abs() >= 60)
            .collect();
        let rest: Vec<&Record> = land
            .iter()
            .copied()
            .filter(|record| (latitudes.of_row(record.row, high) / LATITUDE_FINE).abs() < 60)
            .collect();
        let cold = error_over(
            "the land poleward of 60 degrees",
            &polar,
            rain_for_each_drop,
            whole,
            boundary,
        );
        let warm = error_over(
            "the land equatorward of 60 degrees",
            &rest,
            rain_for_each_drop,
            whole,
            boundary,
        );
        println!();
        println!(
            "class error: whole {}.{}, poleward of 60 {}.{}, equatorward of 60 {}.{}",
            over_all / 10,
            over_all % 10,
            cold / 10,
            cold % 10,
            warm / 10,
            warm % 10
        );
        let _ = land_count;
    }
    // The two headline questions, in one line each.
    let band_of = |degrees: i32| -> Vec<&Record> {
        let low = degrees - 8;
        let high_end = degrees + 8;
        records
            .iter()
            .filter(|record| record.land)
            .filter(|record| {
                let at = latitudes.of_row(record.row, high) / LATITUDE_FINE;
                at >= low && at <= high_end
            })
            .collect()
    };
    let rain_of = |members: &[&Record]| -> i64 {
        if members.is_empty() {
            return 0;
        }
        members
            .iter()
            .map(|record| record.year_water() * rain_for_each_drop / whole)
            .sum::<i64>()
            / members.len() as i64
    };
    let equator = rain_of(&band_of(0));
    let north = rain_of(&band_of(30));
    let south = rain_of(&band_of(-30));
    let temperate = rain_of(&band_of(55));
    println!();
    println!(
        "rain over land, in the millimetres the normaliser gives: \
         equator {equator}, thirty north {north}, thirty south {south}, \
         fifty-five north {temperate}"
    );
    println!(
        "the subtropics take {}% of the equator, and the published belt of deserts \
         says this must be well under a hundred",
        north.max(south) * 100 / equator.max(1)
    );
}
