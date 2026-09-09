//! A throwaway probe that reads the sky of each cell over time.
//!
//! This is a diagnostic, not a test. The project owner reports that a place
//! never has a clear day and then a cloudy one, and that the sky is never
//! broken. This probe answers the first claim in numbers.
//!
//! It settles the world, then samples the cloud share of every cell once each
//! tick over a window. For each cell it holds the least share, the largest
//! share, and the ticks the cell spent clear, broken and overcast. It then
//! reports how the range is distributed over the cells, and how many cells
//! ever crossed from clear to overcast.
//!
//! It also prints the whole series for four cells, one for each of four
//! latitudes, so that a reader can see the shape rather than a summary.
//!
//! Run it with `cargo run -p cachette-core --release --example
//! weather_cell_probe`. The arguments are the extent, the seed, the ticks to
//! settle, the ticks to sample, and the weather scale in bits.
//!
//! Every figure is an integer.

use cachette_core::weather::CLOUD_SHARE_WHOLE;
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

/// The share below which the viewer paints bare paper. It is the cloud floor
/// of the demonstration renderer, carried here so that the probe reports what
/// a watcher sees and not what the field holds.
const CLEAR_MARK: i64 = 18 * CLOUD_SHARE_WHOLE / 100;

/// The share at which the sky reads as one whole overcast.
const OVERCAST_MARK: i64 = 90 * CLOUD_SHARE_WHOLE / 100;

/// The latitudes the probe follows one cell at, in hundredths of a degree.
const FOLLOWED: [i32; 4] = [0, 30 * 100, 60 * 100, -35 * 100];

/// What the probe holds for one cell over the window.
#[derive(Clone)]
struct Cell {
    least: i64,
    largest: i64,
    total: i64,
    clear_ticks: u32,
    broken_ticks: u32,
    overcast_ticks: u32,
    at_the_mark: u32,
}

impl Cell {
    fn new() -> Self {
        Self {
            least: CLOUD_SHARE_WHOLE + 1,
            largest: -1,
            total: 0,
            clear_ticks: 0,
            broken_ticks: 0,
            overcast_ticks: 0,
            at_the_mark: 0,
        }
    }

    fn take(&mut self, share: i64, at_the_mark: bool) {
        self.least = self.least.min(share);
        self.largest = self.largest.max(share);
        self.total += share;
        if share < CLEAR_MARK {
            self.clear_ticks += 1;
        } else if share >= OVERCAST_MARK {
            self.overcast_ticks += 1;
        } else {
            self.broken_ticks += 1;
        }
        if at_the_mark {
            self.at_the_mark += 1;
        }
    }

    fn range(&self) -> i64 {
        (self.largest - self.least).max(0)
    }
}

/// Returns the deciles of a run of numbers.
fn deciles(values: &mut [i64]) -> Vec<i64> {
    values.sort_unstable();
    (1..10)
        .map(|part| {
            let at = (values.len() * part / 10).min(values.len().saturating_sub(1));
            values.get(at).copied().unwrap_or(0)
        })
        .collect()
}

fn main() {
    let extent = argument(1, 96) as u32;
    let seed = argument(2, 0x2f);
    let settle = argument(3, 400) as u32;
    let window = argument(4, 400) as u32;
    let bits = argument(5, 0) as u32;
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

    let cells: Vec<u32> = world.weather().lattice().inner_cells();
    let height = world.weather().lattice().inner().height();
    println!(
        "extent {extent} seed {seed:#x} settle {settle} window {window} bits {bits} cells {}",
        cells.len()
    );

    // The cell the probe follows at each latitude. It is the cell of that row
    // nearest the middle column, so the choice is fixed and not a draw.
    let middle = world.weather().lattice().inner().width() / 2;
    let followed: Vec<(i32, u32)> = FOLLOWED
        .iter()
        .map(|asked| {
            let row = (0..height)
                .min_by_key(|row| (world.weather().latitudes().of_row(*row, height) - asked).abs())
                .unwrap_or(0);
            let cell = cells
                .iter()
                .copied()
                .find(|cell| {
                    let field = world.weather();
                    field.lattice().inner_row_of(*cell) == row
                        && field
                            .lattice()
                            .whole()
                            .address_of(cachette_core::TileIdx(*cell))
                            .map(|address| address.q)
                            .unwrap_or(-1)
                            == (middle + world.weather().lattice().ring()) as i32
                })
                .unwrap_or(cells[0]);
            (*asked, cell)
        })
        .collect();

    for _ in 0..settle {
        world.step(1).expect("the settle must run");
    }

    let mut held = vec![Cell::new(); cells.len()];
    let mut series: Vec<Vec<i64>> = vec![Vec::new(); followed.len()];
    for _ in 0..window {
        world.step(1).expect("the step must run");
        let field = world.weather();
        for (slot, cell) in cells.iter().enumerate() {
            let share = field.cloud_share_at(*cell);
            let at_the_mark = field.air_at(*cell).0 >= field.capacity_at_cell(*cell).0;
            held[slot].take(share, at_the_mark);
        }
        for (slot, (_, cell)) in followed.iter().enumerate() {
            series[slot].push(field.cloud_share_at(*cell));
        }
    }

    let total = held.len() as i64;
    let mut ranges: Vec<i64> = held.iter().map(Cell::range).collect();
    let ever_clear = held.iter().filter(|cell| cell.clear_ticks > 0).count();
    let ever_overcast = held.iter().filter(|cell| cell.overcast_ticks > 0).count();
    let both = held
        .iter()
        .filter(|cell| cell.clear_ticks > 0 && cell.overcast_ticks > 0)
        .count();
    let always_overcast = held
        .iter()
        .filter(|cell| cell.overcast_ticks == window)
        .count();
    let always_clear = held
        .iter()
        .filter(|cell| cell.clear_ticks == window)
        .count();
    let mostly_broken = held
        .iter()
        .filter(|cell| cell.broken_ticks * 2 > window)
        .count();
    let pinned = held
        .iter()
        .filter(|cell| cell.at_the_mark == window)
        .count();

    println!("a whole sky is {CLOUD_SHARE_WHOLE}, clear is under {CLEAR_MARK}, overcast is {OVERCAST_MARK} and above");
    println!("cells {total}");
    println!("  ever clear            {ever_clear}");
    println!("  ever overcast         {ever_overcast}");
    println!("  both, at some tick    {both}");
    println!("  overcast every tick   {always_overcast}");
    println!("  clear every tick      {always_clear}");
    println!("  broken over half      {mostly_broken}");
    println!("  at the mark every tick {pinned}");
    let parts = deciles(&mut ranges);
    println!(
        "  range over the window, deciles: {}",
        parts
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    );

    for (slot, (asked, cell)) in followed.iter().enumerate() {
        let run = &series[slot];
        let least = run.iter().copied().min().unwrap_or(0);
        let largest = run.iter().copied().max().unwrap_or(0);
        let mean = if run.is_empty() {
            0
        } else {
            run.iter().sum::<i64>() / run.len() as i64
        };
        println!("cell {cell} at latitude {asked}: least {least} mean {mean} largest {largest}");
        let shown: Vec<String> = run
            .iter()
            .step_by((window as usize / 40).max(1))
            .map(|value| value.to_string())
            .collect();
        println!("  {}", shown.join(" "));
    }
}
