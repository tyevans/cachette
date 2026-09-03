//! The cost of a frame on the target platform.
//!
//! Every cost figure in this project is derived rather than measured, and one
//! blocker states that no measurement exists on the target platform.[^1] This
//! benchmark is the apparatus that takes such a measurement. It runs the
//! public crate interface and nothing else.
//!
//! # What it measures, and why each row earns its cost
//!
//! **The step, against the tile count.** The step scans every tile of the
//! world on every frame, so the tile count is one of the two axes of the
//! target scale. The engine runs at ten ticks for each second, which gives a
//! frame budget of one hundred milliseconds, and that budget is derived from
//! the tile edge and the march rate rather than measured.[^2] This row says
//! whether the budget is reachable.
//!
//! **The step, against the unit count.** The unit count is the other axis of
//! the target scale. The choice pass, the movement pass and admission all
//! walk the live units, so the two axes must be separated or neither figure
//! means anything.
//!
//! **Building a world, against the tile count.** The project states that
//! building a world visits no tile, because the tile value field generates a
//! value and stores only what a frame changed.[^3] A test proves that no tile
//! is visited. This row says what the claim is worth in time on the target.
//!
//! **The whole-world hash, against the tile count.** The golden state test
//! compares this value against a stored file on every frame it checks, and
//! the determinism rule of this project rests on it.[^4] The hash walks every
//! tile, so its cost decides whether that gate stays affordable at the target
//! extent.
//!
//! # What it does not measure
//!
//! The measured world holds no settlement. The rate pass, the consumption
//! pass and the position pass therefore do nothing, and the figures below are
//! a lower bound on the cost of a frame at the target scale rather than the
//! whole of it. The two passes that carry the target scale, which are the
//! tile pass and the movement of the units, are measured.
//!
//! # How to run it
//!
//! ```text
//! cargo bench --bench target_cost -- quick
//! cargo bench --bench target_cost -- full
//! ```
//!
//! The output is a tab separated table on the standard output. Every duration
//! is in nanoseconds, and every value is an integer. A line that starts with
//! a hash is a comment.
//!
//! A benchmark does not gate a merge, and no test in this project asserts on
//! time.[^5]
//!
//! # References
//!
//! [^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
//! [^2]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
//! [^3]: ADR-0068, terrain is generated from the seed and is never stored as a map. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
//! [^4]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
//! [^5]: Testing rules, section 3. `.claude/rules/testing.md`

use std::time::Instant;

use cachette_core::{Axial, FactionId, World, WorldConfig};

/// The seed that every world in this benchmark takes.
const SEED: u64 = 0x0123_4567_89ab_cdef;

/// The number of factions that every world in this benchmark holds.
const FACTIONS: u16 = 8;

/// The largest number of samples that one row takes.
const MAX_SAMPLES: usize = 9;

/// The smallest number of samples that one row takes.
const MIN_SAMPLES: usize = 3;

/// The time after which a row stops taking samples, in nanoseconds.
///
/// A row always takes the smallest number of samples first, so a
/// configuration that costs more than this budget still reports a figure.
const ROW_BUDGET_NS: u128 = 10_000_000_000;

/// The number of frames that a step row runs before it starts to measure.
const WARMUP_FRAMES: usize = 2;

/// Reads the clock.
///
/// One lint forbids the clock across this workspace, because a simulation
/// that reads a clock gives an answer that depends on the load of the
/// machine.[^1] A benchmark is the one caller that must read it: it produces
/// no simulated state, it enters no state hash, and it asserts nothing.[^2]
/// The allowance sits on this function alone, so the whole benchmark reads
/// the clock at one site and the lint still covers every other line of it.
///
/// # References
///
/// [^1]: ADR-0005, decision D1. `docs/adrs/REGISTRY.md`
/// [^2]: Testing rules, section 3. `.claude/rules/testing.md`
#[allow(clippy::disallowed_methods)]
fn now() -> Instant {
    Instant::now()
}

/// One extent of the world, as a width and a height in tiles.
#[derive(Clone, Copy)]
struct Extent {
    width: u32,
    height: u32,
}

impl Extent {
    const fn tiles(self) -> u64 {
        self.width as u64 * self.height as u64
    }

    const fn config(self, unit_capacity: u32) -> WorldConfig {
        WorldConfig {
            width: self.width,
            height: self.height,
            seed: SEED,
            faction_count: FACTIONS,
            unit_capacity,
        }
    }
}

/// The sweep that one run of the benchmark covers.
struct Profile {
    /// The name that the preamble reports.
    name: &'static str,
    /// The extents that the tile rows sweep.
    extents: &'static [Extent],
    /// The thread counts that every row sweeps.
    threads: &'static [usize],
    /// The extent that the unit rows hold fixed.
    unit_extent: Extent,
    /// The unit counts that the unit rows sweep.
    units: &'static [u32],
    /// The extent and the unit count of the target scale row.
    target: (Extent, u32),
}

/// The small sweep. It exists so that a person can check the apparatus on a
/// development machine in under a minute, and it measures nothing that the
/// project may cite.
const QUICK: Profile = Profile {
    name: "quick",
    extents: &[
        Extent {
            width: 64,
            height: 64,
        },
        Extent {
            width: 256,
            height: 256,
        },
        Extent {
            width: 512,
            height: 512,
        },
    ],
    threads: &[1, 2],
    unit_extent: Extent {
        width: 512,
        height: 512,
    },
    units: &[0, 1_000, 10_000],
    target: (
        Extent {
            width: 512,
            height: 512,
        },
        10_000,
    ),
};

/// The sweep that reaches the target scale of the project.
///
/// The last extent holds 16777216 tiles, and the target row places one
/// million units on it. Both are the figures the scale constants table
/// states.
const FULL: Profile = Profile {
    name: "full",
    extents: &[
        Extent {
            width: 64,
            height: 64,
        },
        Extent {
            width: 256,
            height: 256,
        },
        Extent {
            width: 1024,
            height: 1024,
        },
        Extent {
            width: 2048,
            height: 2048,
        },
        Extent {
            width: 4096,
            height: 4096,
        },
    ],
    threads: &[1, 2, 4],
    unit_extent: Extent {
        width: 2048,
        height: 2048,
    },
    units: &[0, 10_000, 100_000, 1_000_000],
    target: (
        Extent {
            width: 4096,
            height: 4096,
        },
        1_000_000,
    ),
};

fn main() {
    let argument = std::env::args().nth(1).unwrap_or_default();
    let profile = if argument == "full" { &FULL } else { &QUICK };

    preamble(profile);
    println!("bench\ttiles\tunits\tthreads\tsamples\tmin_ns\tmedian_ns\tmax_ns");

    build_rows(profile);
    hash_rows(profile);
    step_by_tiles(profile);
    step_by_units(profile);
    target_row(profile);
}

/// Writes the facts that a reader needs before any figure below it.
fn preamble(profile: &Profile) {
    let parallelism = std::thread::available_parallelism().map_or(0, std::num::NonZero::get);
    println!("# cachette target cost benchmark");
    println!("# profile\t{}", profile.name);
    println!("# target_triple\t{}", target_triple());
    println!("# available_parallelism\t{parallelism}");
    println!("# seed\t{SEED}");
    println!("# faction_count\t{FACTIONS}");
    println!("# settlements\t0");
    println!("# every duration is in nanoseconds");
    println!("# a thread count of zero means the call takes no thread count");
}

/// Returns the triple that this binary was built for.
///
/// The triple is not available as one string, so the parts are joined here.
fn target_triple() -> String {
    format!(
        "{}-{}-{}",
        std::env::consts::ARCH,
        std::env::consts::FAMILY,
        std::env::consts::OS
    )
}

/// Takes the samples of one row and returns them in ascending order.
fn samples_of(mut body: impl FnMut() -> u128) -> Vec<u128> {
    let mut samples = Vec::with_capacity(MAX_SAMPLES);
    let mut total: u128 = 0;
    while samples.len() < MAX_SAMPLES {
        let elapsed = body();
        total += elapsed;
        samples.push(elapsed);
        if samples.len() >= MIN_SAMPLES && total >= ROW_BUDGET_NS {
            break;
        }
    }
    samples.sort_unstable();
    samples
}

/// Writes one row of the table.
fn report(name: &str, tiles: u64, units: u32, threads: usize, samples: &[u128]) {
    let last = samples.len() - 1;
    println!(
        "{name}\t{tiles}\t{units}\t{threads}\t{}\t{}\t{}\t{}",
        samples.len(),
        samples[0],
        samples[samples.len() / 2],
        samples[last]
    );
}

/// Measures the construction of a world against the tile count.
///
/// The reservation is small and fixed, so the only quantity that moves across
/// these rows is the tile count. A flat column is what the generated tile
/// value field claims.
fn build_rows(profile: &Profile) {
    for extent in profile.extents {
        let config = extent.config(1024);
        let samples = samples_of(|| {
            let start = now();
            let world = World::new(config).expect("the extent must describe a world");
            let elapsed = start.elapsed().as_nanos();
            drop(world);
            elapsed
        });
        report("build", extent.tiles(), 0, 0, &samples);
    }

    // The reservation is paid once, at construction, and the default is the
    // whole target population. This row separates that cost from the tile
    // count above, which the rows above hold at a small fixed value.
    let (extent, _) = profile.target;
    let config = extent.config(WorldConfig::TARGET_UNIT_POPULATION);
    let samples = samples_of(|| {
        let start = now();
        let world = World::new(config).expect("the extent must describe a world");
        let elapsed = start.elapsed().as_nanos();
        drop(world);
        elapsed
    });
    report(
        "build_at_target_reservation",
        extent.tiles(),
        WorldConfig::TARGET_UNIT_POPULATION,
        0,
        &samples,
    );
}

/// Measures the whole-world hash against the tile count.
fn hash_rows(profile: &Profile) {
    for extent in profile.extents {
        let mut world = World::new(extent.config(1024)).expect("the extent must describe a world");
        world.step(1).expect("the step must run");
        let samples = samples_of(|| {
            let start = now();
            let hash = world.state_hash().finish();
            let elapsed = start.elapsed().as_nanos();
            std::hint::black_box(hash);
            elapsed
        });
        report("state_hash", extent.tiles(), 0, 0, &samples);
    }
}

/// Measures one frame against the tile count, at each thread count.
///
/// The world holds no unit, so this row is the tile pass alone.
fn step_by_tiles(profile: &Profile) {
    for extent in profile.extents {
        for threads in profile.threads {
            let samples = step_samples(extent.config(1024), 0, *threads);
            report("step_by_tiles", extent.tiles(), 0, *threads, &samples);
        }
    }
}

/// Measures one frame against the unit count, at a fixed extent.
fn step_by_units(profile: &Profile) {
    let extent = profile.unit_extent;
    for units in profile.units {
        for threads in profile.threads {
            let capacity = (*units).max(1024);
            let samples = step_samples(extent.config(capacity), *units, *threads);
            report("step_by_units", extent.tiles(), *units, *threads, &samples);
        }
    }
}

/// Measures one frame at the tile count and the unit count of the target.
fn target_row(profile: &Profile) {
    let (extent, units) = profile.target;
    for threads in profile.threads {
        let capacity = units.max(1024);
        let samples = step_samples(extent.config(capacity), units, *threads);
        report(
            "step_at_target_scale",
            extent.tiles(),
            units,
            *threads,
            &samples,
        );
    }
}

/// Builds a world, places the units, and returns the samples of one frame.
///
/// The setup is outside the measurement. The world advances between samples,
/// because a frame changes the world it runs on, so each sample is the next
/// frame of one run rather than a repeat of one frame.
fn step_samples(config: WorldConfig, units: u32, threads: usize) -> Vec<u128> {
    let mut world = World::new(config).expect("the extent must describe a world");
    let placed = populate(&mut world, units);
    assert_eq!(
        placed, units,
        "the extent must hold enough open ground for the units"
    );
    for _ in 0..WARMUP_FRAMES {
        world.step(threads).expect("the step must run");
    }
    samples_of(move || {
        let start = now();
        let log = world.step(threads).expect("the step must run");
        let elapsed = start.elapsed().as_nanos();
        std::hint::black_box(log.len());
        elapsed
    })
}

/// Places one unit on each of the first open tiles, and returns the count.
///
/// The pattern walks the tiles in index order and passes over ground that
/// admits no unit. It is the same on every run, because which tiles hold
/// water is a property of the seed.
fn populate(world: &mut World, units: u32) -> u32 {
    if units == 0 {
        return 0;
    }
    let grid = world.grid();
    let width = grid.width();
    let ceiling = u32::from(world.config().faction_count.max(1));
    let mut placed = 0u32;
    let mut index = 0u32;
    while placed < units && index < grid.tile_count() {
        let address = Axial::new((index % width) as i32, (index / width) as i32);
        index += 1;
        if !world.admits_a_unit(address) {
            continue;
        }
        let faction = FactionId((placed % ceiling) as u16);
        if world.spawn_soldier(address, faction).is_ok() {
            placed += 1;
        }
    }
    placed
}
