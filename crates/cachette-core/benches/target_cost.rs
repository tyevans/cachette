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

use cachette_core::choose::PERIOD_LOG2_CEILING;
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
///
/// Every field is a parameter. A profile below states the default, and an
/// environment variable replaces it, so a run at another size or another
/// thread count is a setting rather than a change to this file.
struct Profile {
    /// The name that the preamble reports.
    name: String,
    /// The extents that the tile rows sweep.
    extents: Vec<Extent>,
    /// The thread counts that every row sweeps.
    threads: Vec<usize>,
    /// The extent that the unit rows hold fixed.
    unit_extent: Extent,
    /// The unit counts that the unit rows sweep.
    units: Vec<u32>,
    /// The extent and the unit count of the target scale row.
    target: (Extent, u32),
}

/// Reads a list of extents from the environment, as `WIDTHxHEIGHT` words.
fn extents_from(name: &str, fallback: &[(u32, u32)]) -> Vec<Extent> {
    if let Ok(text) = std::env::var(name) {
        let parsed: Vec<Extent> = text
            .split_whitespace()
            .filter_map(|word| {
                let (width, height) = word.split_once('x')?;
                Some(Extent {
                    width: width.parse().ok()?,
                    height: height.parse().ok()?,
                })
            })
            .collect();
        if !parsed.is_empty() {
            return parsed;
        }
    }
    fallback
        .iter()
        .map(|(width, height)| Extent {
            width: *width,
            height: *height,
        })
        .collect()
}

/// Reads a list of numbers from the environment, as decimal words.
fn numbers_from<T>(name: &str, fallback: &[T]) -> Vec<T>
where
    T: Copy + std::str::FromStr,
{
    if let Ok(text) = std::env::var(name) {
        let parsed: Vec<T> = text
            .split_whitespace()
            .filter_map(|word| word.parse().ok())
            .collect();
        if !parsed.is_empty() {
            return parsed;
        }
    }
    fallback.to_vec()
}

/// The small sweep. It exists so that a person can check the apparatus on a
/// development machine in under a minute, and it measures nothing that the
/// project may cite.
fn quick() -> Profile {
    Profile {
        name: "quick".to_owned(),
        extents: extents_from(EXTENTS_VAR, &[(64, 64), (256, 256), (512, 512)]),
        threads: numbers_from(THREADS_VAR, &[1, 2]),
        unit_extent: Extent {
            width: 512,
            height: 512,
        },
        units: numbers_from(UNITS_VAR, &[0, 1_000, 10_000]),
        target: (
            Extent {
                width: 512,
                height: 512,
            },
            10_000,
        ),
    }
}

/// The sweep that reaches the target scale of the project.
///
/// The last extent holds 16777216 tiles, and the target row places one
/// million units on it. Both are the figures the scale constants table
/// states.
fn full() -> Profile {
    Profile {
        name: "full".to_owned(),
        extents: extents_from(
            EXTENTS_VAR,
            &[
                (64, 64),
                (256, 256),
                (1024, 1024),
                (2048, 2048),
                (4096, 4096),
            ],
        ),
        threads: numbers_from(THREADS_VAR, &[1, 2, 4]),
        unit_extent: Extent {
            width: 2048,
            height: 2048,
        },
        units: numbers_from(UNITS_VAR, &[0, 10_000, 100_000, 1_000_000]),
        target: (
            Extent {
                width: 4096,
                height: 4096,
            },
            1_000_000,
        ),
    }
}

/// The variable that replaces the extents of a sweep.
const EXTENTS_VAR: &str = "CACHETTE_BENCH_EXTENTS";

/// The variable that replaces the thread counts of a sweep.
const THREADS_VAR: &str = "CACHETTE_BENCH_THREADS";

/// The variable that replaces the unit counts of a sweep.
const UNITS_VAR: &str = "CACHETTE_BENCH_UNITS";

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let first = arguments.first().map_or("", String::as_str);

    match first {
        // One measurement of resident memory, in a process of its own. The
        // parent below starts one child for each point, because a process
        // that has already built a large world does not give the allocator
        // back and would report the high mark of the run rather than the
        // cost of the world it holds.
        "memory-point" => memory_point(&arguments),
        "memory" => memory_sweep(&full()),
        "stages" => stage_rows(&arguments),
        "one" => one_point(&arguments),
        "full" => timing_sweep(&full()),
        _ => timing_sweep(&quick()),
    }
}

/// Runs the timing sweep and writes the table.
fn timing_sweep(profile: &Profile) {
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

/// Measures the parts of a frame that a caller can turn off from outside.
///
/// The engine holds no instrumentation, and this benchmark adds none. A stage
/// inside a step is not callable on its own, so the only honest way to price
/// one from outside is to run a frame with the stage switched off and take
/// the difference. Three switches exist on the public interface, and the rest
/// of the frame stays in one residual that this benchmark cannot divide.
///
/// The switches:
///
/// - **The bridge rebuild** is public, so it is measured directly rather than
///   by a difference. The step calls it three times in each frame.
/// - **The economy** is gated by a schedule. A period the frame never reaches
///   turns off the rate pass and the consumption pass together.
/// - **The choice** is gated by a second schedule, keyed on the level 1 cell.
///   The longest interval leaves almost no cell choosing in a frame. It does
///   not remove the walk over the live units, only the scoring inside it.
///
/// Each configuration builds its own world, because a schedule changes what a
/// frame does to the world and two configurations must start from the same
/// place.
fn stage_rows(arguments: &[String]) {
    let extent = extent_argument(arguments, 1);
    let units: u32 = arguments
        .get(2)
        .and_then(|word| word.parse().ok())
        .expect("the third argument must be a unit count");
    let threads: usize = arguments
        .get(3)
        .and_then(|word| word.parse().ok())
        .unwrap_or(1);

    println!("# stage rows. Each row is a whole frame under one switch");
    println!("# the difference between a row and `everything_on` is that stage");
    println!("bench\ttiles\tunits\tthreads\tsamples\tmin_ns\tmedian_ns\tmax_ns");

    // The switches. A period of 32767 is the largest the rate schedule takes,
    // and a frame count below it never reaches the phase, so the economy
    // never applies. The choice interval is the largest the schedule takes.
    let configurations: [(&str, Option<u32>, Option<u32>); 4] = [
        ("everything_on", None, None),
        ("economy_off", Some(32767), None),
        ("choice_off", None, Some(PERIOD_LOG2_CEILING)),
        (
            "economy_and_choice_off",
            Some(32767),
            Some(PERIOD_LOG2_CEILING),
        ),
    ];

    for (name, economy, choice) in configurations {
        let capacity = units.max(1024);
        let mut world =
            World::new(extent.config(capacity)).expect("the extent must describe a world");
        if let Some(period) = economy {
            world
                .set_economy_schedule(period, 1)
                .expect("the period must be inside the limit");
        }
        if let Some(period_log2) = choice {
            world
                .set_choice_schedule(period_log2)
                .expect("the interval must be inside the ceiling");
        }
        let placed = populate(&mut world, units);
        for _ in 0..WARMUP_FRAMES {
            world.step(threads).expect("the step must run");
        }
        let samples = samples_of(move || {
            let start = now();
            let log = world.step(threads).expect("the step must run");
            let elapsed = start.elapsed().as_nanos();
            std::hint::black_box(log.len());
            elapsed
        });
        report(name, extent.tiles(), placed, threads, &samples);
    }

    // The bridge rebuild is public, so it is priced directly. The step calls
    // it three times in a frame, so a frame pays three of these.
    let capacity = units.max(1024);
    let mut world = World::new(extent.config(capacity)).expect("the extent must describe a world");
    let placed = populate(&mut world, units);
    for _ in 0..WARMUP_FRAMES {
        world.step(threads).expect("the step must run");
    }
    let samples = samples_of(|| {
        let start = now();
        world.rebuild_bridge(threads).expect("the rebuild must run");
        start.elapsed().as_nanos()
    });
    report(
        "bridge_rebuild_once",
        extent.tiles(),
        placed,
        threads,
        &samples,
    );
}

/// Reads an extent argument, as `WIDTHxHEIGHT`.
fn extent_argument(arguments: &[String], index: usize) -> Extent {
    arguments
        .get(index)
        .and_then(|word| word.split_once('x'))
        .and_then(|(width, height)| {
            Some(Extent {
                width: width.parse().ok()?,
                height: height.parse().ok()?,
            })
        })
        .expect("the argument must be an extent, as WIDTHxHEIGHT")
}

/// Measures one timing point, so that a caller names the configuration.
fn one_point(arguments: &[String]) {
    let extent = extent_argument(arguments, 1);
    let units: u32 = arguments
        .get(2)
        .and_then(|word| word.parse().ok())
        .expect("the third argument must be a unit count");
    let threads: usize = arguments
        .get(3)
        .and_then(|word| word.parse().ok())
        .unwrap_or(1);
    println!("bench\ttiles\tunits\tthreads\tsamples\tmin_ns\tmedian_ns\tmax_ns");
    let samples = step_samples(extent.config(units.max(1024)), units, threads);
    report("step_one_point", extent.tiles(), units, threads, &samples);
}

/// Reads a size in kibibytes from the process status file.
///
/// The file is a Linux interface, and the target platform is Linux. A machine
/// that does not publish it reports zero, and a zero row says the run took no
/// memory figure rather than that the world took no memory.
fn status_kib(field: &str) -> u64 {
    let Ok(text) = std::fs::read_to_string("/proc/self/status") else {
        return 0;
    };
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix(field) {
            let digits: String = rest.chars().filter(char::is_ascii_digit).collect();
            return digits.parse().unwrap_or(0);
        }
    }
    0
}

/// Measures the resident memory of one world, and writes one row.
///
/// The process builds one world, places the units, runs two frames, and then
/// reads its own resident size. It measures one point and exits, so the
/// figure is the cost of this world and not the high mark of a sweep.
fn memory_point(arguments: &[String]) {
    let extent = extent_argument(arguments, 1);
    let units: u32 = arguments
        .get(2)
        .and_then(|word| word.parse().ok())
        .expect("the third argument must be a unit count");
    let threads: usize = arguments
        .get(3)
        .and_then(|word| word.parse().ok())
        .unwrap_or(1);

    let empty = status_kib("VmRSS:");
    let capacity = units.max(1024);
    let mut world = World::new(extent.config(capacity)).expect("the extent must describe a world");
    let placed = populate(&mut world, units);
    for _ in 0..WARMUP_FRAMES {
        world.step(threads).expect("the step must run");
    }
    let resident = status_kib("VmRSS:");
    let peak = status_kib("VmHWM:");
    // The world is read after the sizes are taken, so nothing above can drop
    // it early and report the memory of a world that no longer exists.
    let tiles = world.tile_count() as u64;
    println!(
        "memory\t{}\t{placed}\t{threads}\t{}\t{}\t{}",
        extent.tiles(),
        empty * 1024,
        resident * 1024,
        peak * 1024
    );
    assert_eq!(tiles, extent.tiles(), "the world must hold the extent");
    drop(world);
}

/// Starts one child for each point of the memory sweep and writes the table.
fn memory_sweep(profile: &Profile) {
    let binary = std::env::current_exe().expect("the benchmark must know its own path");
    preamble(profile);
    println!("bench\ttiles\tunits\tthreads\tempty_bytes\tresident_bytes\tpeak_bytes");

    let threads = profile.threads.first().copied().unwrap_or(1);
    for extent in &profile.extents {
        for units in &profile.units {
            // A world holds one unit for each open tile at most, so a point
            // that asks for more units than the extent can hold is skipped
            // rather than run against a smaller population under its label.
            if u64::from(*units) > extent.tiles() / 2 {
                continue;
            }
            let output = std::process::Command::new(&binary)
                .arg("memory-point")
                .arg(format!("{}x{}", extent.width, extent.height))
                .arg(units.to_string())
                .arg(threads.to_string())
                .output()
                .expect("the child must run");
            print!("{}", String::from_utf8_lossy(&output.stdout));
            if !output.status.success() {
                println!(
                    "# the point at {}x{} with {units} units failed",
                    extent.width, extent.height
                );
            }
        }
    }
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
    for extent in &profile.extents {
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
    for extent in &profile.extents {
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
    for extent in &profile.extents {
        for threads in &profile.threads {
            let samples = step_samples(extent.config(1024), 0, *threads);
            report("step_by_tiles", extent.tiles(), 0, *threads, &samples);
        }
    }
}

/// Measures one frame against the unit count, at a fixed extent.
fn step_by_units(profile: &Profile) {
    let extent = profile.unit_extent;
    for units in &profile.units {
        for threads in &profile.threads {
            let capacity = (*units).max(1024);
            let samples = step_samples(extent.config(capacity), *units, *threads);
            report("step_by_units", extent.tiles(), *units, *threads, &samples);
        }
    }
}

/// Measures one frame at the tile count and the unit count of the target.
fn target_row(profile: &Profile) {
    let (extent, units) = profile.target;
    for threads in &profile.threads {
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
