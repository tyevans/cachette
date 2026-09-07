//! A probe that measures the air and the visible cloud against the latitude.
//!
//! This is a diagnostic, not a test. The owner watches the map at one cell
//! for each tile and reports that the inland north and the inland south never
//! hold cloud. This probe answers that in numbers. It splits the lattice into
//! bands of rows, and it reports, for each band, the temperature, the air,
//! the share of the sky that a watcher sees as cloud, and how many cells of
//! the band are overcast.
//!
//! It reports each band three ways: over every cell, over the land cells
//! alone, and over the inland cells alone. A band that holds cloud only over
//! its water is the shape the owner complains about, and the three columns
//! are what tell that shape from a band that is truly overcast.
//!
//! Run it with `cargo run -p cachette-core --release --example
//! weather_latitude_probe`. The arguments are the extent, the seed, the
//! ticks, the weather scale, and the cells from water that count as inland.
//!
//! Every figure here is an integer. A share is reported in 255ths, which is
//! the unit the overlay paints, so no step needs a float.

use cachette_core::hex::{Axial, NEIGHBOUR_COUNT};
use cachette_core::{TileIdx, WeatherScale, World, WorldConfig};

fn argument(position: usize, fallback: u64) -> u64 {
    // A seed reads better in hexadecimal, so the probe accepts both forms.
    std::env::args()
        .nth(position)
        .and_then(|text| match text.strip_prefix("0x") {
            Some(rest) => u64::from_str_radix(rest, 16).ok(),
            None => text.parse::<u64>().ok(),
        })
        .unwrap_or(fallback)
}

/// Returns the water tile count and the tile count of every weather cell.
fn ground_of(world: &World, extent: u32, bits: u32, count: usize) -> (Vec<i64>, Vec<i64>) {
    // **A world address names an inner cell, and the planes are indexed by
    // the whole lattice.** The margin sits between the two, so the walk goes
    // through the lattice rather than through the whole grid. Without this
    // step the probe reads the ground of one cell against the air of
    // another, and every band it prints is shifted by the margin.
    let lattice = world.weather().lattice();
    let inner = lattice.inner();
    let mut water = vec![0i64; count];
    let mut tiles = vec![0i64; count];
    for row in 0..extent {
        for column in 0..extent {
            let address = Axial::new(column as i32, row as i32);
            let Some(tile) = world.tile_terrain(address) else {
                continue;
            };
            let cell = Axial::new((column >> bits) as i32, (row >> bits) as i32);
            let Some(at) = inner.index_of(cell) else {
                continue;
            };
            let Some(whole) = lattice.whole_of_inner(at.0) else {
                continue;
            };
            tiles[whole as usize] += 1;
            if !tile.kind.is_passable() {
                water[whole as usize] += 1;
            }
        }
    }
    (water, tiles)
}

/// Returns the lattice distance from each cell to the nearest cell holding
/// water. A cell that holds water is at distance zero.
fn distance_from_water(world: &World, water: &[i64]) -> Vec<i32> {
    let cells = world.weather().cells();
    let mut out = vec![-1i32; water.len()];
    let mut front: Vec<u32> = Vec::new();
    for (index, count) in water.iter().enumerate() {
        if *count > 0 {
            out[index] = 0;
            front.push(index as u32);
        }
    }
    let mut step = 0i32;
    while !front.is_empty() {
        step += 1;
        let mut next = Vec::new();
        for index in front {
            let Some(address) = cells.address_of(TileIdx(index)) else {
                continue;
            };
            for direction in 0..NEIGHBOUR_COUNT {
                let Some(beside) = cells.neighbour(address, direction) else {
                    continue;
                };
                let Some(at) = cells.index_of(beside) else {
                    continue;
                };
                if out[at.0 as usize] >= 0 {
                    continue;
                }
                out[at.0 as usize] = step;
                next.push(at.0);
            }
        }
        front = next;
    }
    out
}

fn mean(values: &[i64]) -> i64 {
    if values.is_empty() {
        0
    } else {
        values.iter().sum::<i64>() / values.len() as i64
    }
}

/// The share of the sky that a watcher reads as cloud, above which a cell
/// counts as overcast. The unit is 255ths, so this is a little over half.
const OVERCAST: i64 = 140;

/// One row of the report.
struct Band {
    cells: usize,
    warmth: Vec<i64>,
    air: Vec<i64>,
    capacity: Vec<i64>,
    wet: Vec<i64>,
    cloud: Vec<i64>,
    overcast: usize,
}

impl Band {
    fn new() -> Self {
        Self {
            cells: 0,
            warmth: Vec::new(),
            air: Vec::new(),
            capacity: Vec::new(),
            wet: Vec::new(),
            cloud: Vec::new(),
            overcast: 0,
        }
    }

    fn add(&mut self, warmth: i64, air: i64, capacity: i64, wet: i64, cloud: i64) {
        self.cells += 1;
        self.warmth.push(warmth);
        self.air.push(air);
        self.capacity.push(capacity);
        self.wet.push(wet);
        self.cloud.push(cloud);
        if cloud >= OVERCAST {
            self.overcast += 1;
        }
    }

    fn say(&self) -> String {
        if self.cells == 0 {
            return format!(
                "{:>6} {:>6} {:>6} {:>6} {:>6} {:>6}",
                0, "-", "-", "-", "-", "-"
            );
        }
        format!(
            "{:>6} {:>6} {:>6} {:>6} {:>6} {:>5}%",
            self.cells,
            mean(&self.warmth),
            mean(&self.air),
            mean(&self.capacity),
            mean(&self.wet),
            self.overcast * 100 / self.cells,
        )
    }

    fn cloud_mean(&self) -> i64 {
        mean(&self.cloud)
    }
}

fn main() {
    let extent = argument(1, 128) as u32;
    let seed = argument(2, 0x2f);
    let ticks = argument(3, 300) as u32;
    let bits = argument(4, 0) as u32;
    let inland_from = argument(5, 4) as i32;
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

    let cells = world.weather().cells();
    let lattice = world.weather().lattice();
    let count = cells.tile_count() as usize;
    let (water, tiles) = ground_of(&world, extent, bits, count);
    let distance = distance_from_water(&world, &water);
    // The bands run over the rows of the world, not over the rows of the
    // whole lattice. The margin holds no latitude of its own.
    let high = world.weather().lattice().inner().height();
    // Ten bands of rows. Band zero is the north pole and band nine is the
    // south pole, because the season swings along the row axis.
    let bands = 10u32;

    println!("extent {extent} seed {seed:#x} ticks {ticks} scale bits {bits}");
    println!(
        "world {} by {} cells, margin {}, inland means {inland_from} cells or more from any water",
        lattice.inner().width(),
        high,
        lattice.ring()
    );

    for tick in 1..=ticks {
        world.step(1).expect("the step must run");
        if tick != ticks {
            continue;
        }
        let field = world.weather();
        let air = field.air_plane().to_vec();
        let warmth = field.warmth_plane().to_vec();
        let ceiling = field.air_ceiling().max(1);

        let mut all: Vec<Band> = (0..bands).map(|_| Band::new()).collect();
        let mut land: Vec<Band> = (0..bands).map(|_| Band::new()).collect();
        let mut inland: Vec<Band> = (0..bands).map(|_| Band::new()).collect();
        for index in 0..count {
            // The row is the row of the world. A ring cell has no world row,
            // and it holds no ground either, so the ground test drops it.
            let Some(address) = lattice.inner_address_of(index as u32) else {
                continue;
            };
            if tiles[index] == 0 {
                continue;
            }
            let row = address.r.max(0) as u32;
            let band = (row * bands / high.max(1)).min(bands - 1) as usize;
            let held = air.get(index).map_or(0, |drops| drops.0);
            let degrees = warmth.get(index).copied().unwrap_or(0).into();
            // The share of the sky that the overlay paints. The reader is the
            // field, so the probe never restates the ceiling itself.
            let cloud = field.cloud_share_at(index as u32);
            let capacity = field.capacity_at_cell(index as u32).0;
            let wet = field.ground_at(index as u32).0;
            all[band].add(degrees, held, capacity, wet, cloud);
            if water[index] * 2 < tiles[index] {
                land[band].add(degrees, held, capacity, wet, cloud);
            }
            if distance[index] >= inland_from {
                inland[band].add(degrees, held, capacity, wet, cloud);
            }
        }

        println!("--- tick {tick}, air ceiling {ceiling} drops");
        println!("     band |  cells warmth    air    cap ground  over |  cells warmth    air    cap ground  over |  cells warmth    air    cap ground  over");
        println!("          |               every cell                          land cells alone                        inland cells alone");
        for band in 0..bands as usize {
            println!(
                "  {band:>3} ({:>4}) | {} | {} | {}",
                all[band].cloud_mean(),
                all[band].say(),
                land[band].say(),
                inland[band].say(),
            );
        }
        println!("  the figure in brackets is the mean visible cloud of the band, in 255ths");

        // The two poles against the middle, which is the owner's complaint in
        // one line.
        let polar: Vec<i64> = all[0]
            .cloud
            .iter()
            .chain(all[bands as usize - 1].cloud.iter())
            .copied()
            .collect();
        let polar_inland: Vec<i64> = inland[0]
            .cloud
            .iter()
            .chain(inland[bands as usize - 1].cloud.iter())
            .copied()
            .collect();
        let middle: Vec<i64> = all[4]
            .cloud
            .iter()
            .chain(all[5].cloud.iter())
            .copied()
            .collect();
        println!(
            "  visible cloud, in 255ths: the two polar bands {}, their inland cells {}, the two middle bands {}",
            mean(&polar),
            mean(&polar_inland),
            mean(&middle),
        );
        let dark = polar_inland.iter().filter(|value| **value < 16).count();
        println!(
            "  inland polar cells under a sixteenth of the sky: {dark} of {}",
            polar_inland.len().max(1)
        );
    }
}
