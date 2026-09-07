//! A probe that measures whether the wind field holds a closed circulation,
//! and whether a storm forms, lives and travels.
//!
//! This is a diagnostic, not a test. The deflecting term turns the wind a
//! share of a sixth of a turn to one side. Nobody had measured whether that
//! produces a vortex a watcher would see, and a rotation term that produces
//! no visible vortex is not done.
//!
//! Run it with `cargo run -p cachette-core --release --example
//! weather_rotation_probe`. The arguments are the extent, the seed, the
//! ticks, and the weather scale.

use cachette_core::hex::{NEIGHBOURS, NEIGHBOUR_COUNT};
use cachette_core::weather::CLOUD_SHARE_WHOLE;
use cachette_core::{TileIdx, WeatherScale, World, WorldConfig};

fn argument(position: usize, fallback: u64) -> u64 {
    std::env::args()
        .nth(position)
        .and_then(|text| text.parse::<u64>().ok())
        .unwrap_or(fallback)
}

/// The largest circulation one cell can hold.
///
/// Each of the six neighbours contributes the part of its wind that runs
/// along the ring. That part reaches twice the speed when the wind points
/// exactly that way, so a ring of cells all running round at the speed
/// ceiling gives six times twice the ceiling.
fn circulation_ceiling() -> i64 {
    6 * 2 * i64::from(cachette_core::weather::SPEED_CEILING)
}

/// Returns the circulation of the wind round each cell.
///
/// A closed circulation gives a large value of one sign. The measure sums,
/// over the six neighbours, the part of that neighbour's wind that runs along
/// the ring at that point.
fn circulation(world: &World) -> Vec<i64> {
    let field = world.weather();
    let cells = field.cells();
    let wind = field.wind_plane();
    let mut out = vec![0i64; wind.len()];
    for (index, slot) in out.iter_mut().enumerate() {
        let Some(address) = cells.address_of(TileIdx(index as u32)) else {
            continue;
        };
        let mut sum = 0i64;
        for direction in 0..NEIGHBOUR_COUNT {
            let Some(beside) = cells.neighbour(address, direction) else {
                continue;
            };
            let Some(at) = cells.index_of(beside) else {
                continue;
            };
            // The tangent of the ring at this neighbour is the next step but
            // one round, and `along` gives the exact lattice projection.
            let tangent = NEIGHBOURS[(direction + 2) % NEIGHBOUR_COUNT];
            sum += i64::from(wind[at.0 as usize].along(tangent));
        }
        *slot = sum;
    }
    out
}

/// One storm the probe is tracking.
struct Tracked {
    /// The tick the storm first appeared on.
    born: u64,
    /// The centre of the storm when it first appeared, in lattice steps.
    from: (i64, i64),
    /// The centre of the storm now.
    at: (i64, i64),
    /// The ticks the storm has lived.
    ticks: u64,
    /// The greatest cell count the storm reached.
    widest: usize,
    /// Whether the storm was seen on the tick being read.
    seen: bool,
}

/// Returns the centre of each cluster of loaded cells, with its cell count.
///
/// A cell is loaded when its air stands at or above the given share of the
/// saturation mark. A cluster is a connected run of loaded cells. The walk is
/// over ascending cell index and the six directions in order, so the answer
/// does not depend on any thread.
fn clusters(world: &World, mark: i64, floor: usize) -> Vec<((i64, i64), usize)> {
    let field = world.weather();
    let cells = field.cells();
    let air = field.air_plane();
    let mut seen = vec![false; air.len()];
    let mut out = Vec::new();
    for start in 0..air.len() {
        if seen[start] || field.cloud_share_at(start as u32) < mark {
            continue;
        }
        let mut stack = vec![start];
        seen[start] = true;
        let mut count = 0usize;
        let (mut sum_q, mut sum_r) = (0i64, 0i64);
        while let Some(index) = stack.pop() {
            let Some(address) = cells.address_of(TileIdx(index as u32)) else {
                continue;
            };
            count += 1;
            sum_q += i64::from(address.q);
            sum_r += i64::from(address.r);
            for direction in 0..NEIGHBOUR_COUNT {
                let Some(beside) = cells.neighbour(address, direction) else {
                    continue;
                };
                let Some(at) = cells.index_of(beside) else {
                    continue;
                };
                let at = at.0 as usize;
                if seen[at] || air[at].0 < mark {
                    continue;
                }
                seen[at] = true;
                stack.push(at);
            }
        }
        if count >= floor {
            out.push(((sum_q / count as i64, sum_r / count as i64), count));
        }
    }
    out
}

fn distance(from: (i64, i64), to: (i64, i64)) -> i64 {
    let (dq, dr) = (to.0 - from.0, to.1 - from.1);
    (dq.abs() + dr.abs() + (dq + dr).abs()) / 2
}

fn main() {
    let extent = argument(1, 256) as u32;
    let seed = argument(2, 0x2f);
    let ticks = argument(3, 400);
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

    // A storm is a run of at least this many neighbouring cells whose sky
    // stands at or above three quarters full. The floor keeps a single wet
    // cell from counting as a storm.
    //
    // **The mark is a share of a sky and not a count of drops.** Warm air
    // holds a lot of water and cold air holds very little, so a count of
    // drops names a storm in the tropics and never names one at a high
    // latitude, whatever the sky there looks like.
    let mark = CLOUD_SHARE_WHOLE * 3 / 4;
    let floor = 6usize;
    // A tracked storm keeps its identity while a cluster appears within this
    // many cells of where it stood.
    let near = 4i64;

    let ceiling = circulation_ceiling();
    println!("extent {extent} seed {seed:#x} ticks {ticks} scale bits {bits}");
    println!("circulation ceiling {ceiling}, storm mark {mark} of a whole sky of {CLOUD_SHARE_WHOLE}, storm floor {floor} cells");

    let mut live: Vec<Tracked> = Vec::new();
    let mut done: Vec<Tracked> = Vec::new();
    let mut samples: Vec<(u64, i64, i64, i64, usize)> = Vec::new();

    for tick in 1..=ticks {
        world.step(1).expect("the step must run");

        let curl = circulation(&world);
        let mut sorted: Vec<i64> = curl.iter().copied().map(i64::abs).collect();
        sorted.sort_unstable();
        let at = |share: usize| {
            if sorted.is_empty() {
                0
            } else {
                sorted[(sorted.len() * share / 100).min(sorted.len() - 1)]
            }
        };
        // A cell is turning when its circulation reaches a quarter of what a
        // closed ring at the speed ceiling would give.
        let turning = curl
            .iter()
            .filter(|value| value.abs() * 4 >= ceiling)
            .count();

        for storm in &mut live {
            storm.seen = false;
        }
        for (centre, count) in clusters(&world, mark, floor) {
            let found = live
                .iter_mut()
                .filter(|storm| !storm.seen)
                .min_by_key(|storm| distance(storm.at, centre));
            match found {
                Some(storm) if distance(storm.at, centre) <= near => {
                    storm.at = centre;
                    storm.ticks += 1;
                    storm.widest = storm.widest.max(count);
                    storm.seen = true;
                }
                _ => live.push(Tracked {
                    born: tick,
                    from: centre,
                    at: centre,
                    ticks: 1,
                    widest: count,
                    seen: true,
                }),
            }
        }
        let (kept, lost): (Vec<Tracked>, Vec<Tracked>) =
            live.into_iter().partition(|storm| storm.seen);
        live = kept;
        done.extend(lost);

        if tick % (ticks / 8).max(1) == 0 {
            samples.push((tick, at(50), at(90), at(99), turning));
        }
    }
    done.extend(live);

    println!("  tick   curl p50   curl p90   curl p99   cells turning");
    for (tick, p50, p90, p99, turning) in samples {
        println!("  {tick:>4} {p50:>10} {p90:>10} {p99:>10} {turning:>15}");
    }

    // A storm that lived one tick is a cell that touched the mark and let go.
    // A storm is what outlives that.
    let storms: Vec<&Tracked> = done.iter().filter(|storm| storm.ticks > 1).collect();
    if storms.is_empty() {
        println!("no storm lived more than one tick");
        return;
    }
    let mut lives: Vec<u64> = storms.iter().map(|storm| storm.ticks).collect();
    lives.sort_unstable();
    let mut travels: Vec<i64> = storms
        .iter()
        .map(|storm| distance(storm.from, storm.at))
        .collect();
    travels.sort_unstable();
    let mut widths: Vec<usize> = storms.iter().map(|storm| storm.widest).collect();
    widths.sort_unstable();
    let longest = storms
        .iter()
        .max_by_key(|storm| storm.ticks)
        .expect("the list holds a storm");
    println!(
        "storms {} over {ticks} ticks, which is one every {} ticks",
        storms.len(),
        ticks / storms.len().max(1) as u64
    );
    println!(
        "  lifetime  median {} ticks, longest {} ticks",
        lives[lives.len() / 2],
        lives[lives.len() - 1]
    );
    println!(
        "  travel    median {} cells, furthest {} cells",
        travels[travels.len() / 2],
        travels[travels.len() - 1]
    );
    println!(
        "  width     median {} cells, widest {} cells",
        widths[widths.len() / 2],
        widths[widths.len() - 1]
    );
    println!(
        "  the longest lived storm was born on tick {}, lived {} ticks, and travelled {} cells",
        longest.born,
        longest.ticks,
        distance(longest.from, longest.at)
    );
}
