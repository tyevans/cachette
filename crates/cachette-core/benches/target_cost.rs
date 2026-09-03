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
use cachette_core::stage;
use cachette_core::{Axial, Entity, ExitField, FactionId, Fix32, Grid, World, WorldConfig};

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
        "stage-cost" => stage_cost_rows(&arguments),
        "placement" => placement_rows(&arguments),
        "collapse" => collapse_rows(&arguments),
        "exitfield" => exit_field_rows(&arguments),
        "memory-placement" => memory_placement(&arguments),
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

/// Prices every stage of a frame by name.
///
/// # Why this mode exists
///
/// The `stages` mode above prices a pass by running a whole frame with the
/// pass switched off and taking the difference. Three switches exist, so that
/// method left 62 percent of the cost of a unit in one residual that nothing
/// on the public interface could divide.[^1]
///
/// This mode reads a table that the step fills in as it runs. Every pass is
/// named, so nothing is left in a residual except the glue between the
/// passes, and this mode reports that glue as its own row.
///
/// # It needs a feature
///
/// The table is behind the `stage-cost` feature and is off by default. A run
/// without the feature reports zeros and says so in the preamble, so a reader
/// cannot mistake "this build does not measure" for "the frame cost nothing".
///
/// ```text
/// cargo bench --bench target_cost --features stage-cost -- stage-cost 4096x4096 1000000 12
/// ```
///
/// # What the columns mean
///
/// `frames` is how many frames the row averaged over. `entries` is how many
/// times the step opened the stage across those frames, and it should be
/// `frames` times what the stage declares. `total_ns` is the sum, and
/// `ns_for_each_frame` is the figure a register quotes. `takes_threads` is a
/// declaration in the source rather than a measurement: a stage declared
/// `false` that improves with the thread count means the declaration is
/// wrong.
///
/// `nested` says whether the row divides the row above it rather than adding
/// to the frame. The sum skips a nested row, because its time is already in
/// the stage it divides.
///
/// The last two rows are not stages. `all_stages` is the sum of the rows
/// above it, with the nested rows left out. `frame_wall` is the wall time of the same frames measured from
/// outside, and the difference between the two is the cost of the step that
/// no span covers, plus what the clock costs to read.
///
/// # References
///
/// [^1]: Target platform costs, where the unit cost goes. `docs/reference/graviton-costs.md`
fn stage_cost_rows(arguments: &[String]) {
    let extent = extent_argument(arguments, 1);
    let units: u32 = arguments
        .get(2)
        .and_then(|word| word.parse().ok())
        .expect("the third argument must be a unit count");
    let threads: usize = arguments
        .get(3)
        .and_then(|word| word.parse().ok())
        .unwrap_or(1);
    let scattered = arguments.get(4).map(String::as_str) == Some("scattered");

    /// How many frames one row averages over.
    ///
    /// The table is a sum, so a row here is an average and not a median. A
    /// single frame would carry whatever the operating system did during it.
    const FRAMES: usize = 9;

    println!("# stage cost rows. Each row is one named pass of a frame");
    println!("# recording\t{}", stage::is_recording());
    println!(
        "# placement\t{}",
        if scattered { "scattered" } else { "packed" }
    );
    println!("# frames\t{FRAMES}");
    println!("# transparent_huge_pages\t{}", huge_page_setting());
    println!(
        "stage\ttiles\tunits\tthreads\tframes\tentries\ttotal_ns\tns_for_each_frame\ttakes_threads\tnested"
    );

    let capacity = units.max(1024);
    let mut world = World::new(extent.config(capacity)).expect("the extent must describe a world");
    let placed = if scattered {
        populate_scattered(&mut world, units)
    } else {
        populate(&mut world, units)
    };
    for _ in 0..WARMUP_FRAMES {
        world.step(threads).expect("the step must run");
    }

    stage::reset();
    let start = now();
    for _ in 0..FRAMES {
        let log = world.step(threads).expect("the step must run");
        std::hint::black_box(log.len());
    }
    let wall = start.elapsed().as_nanos();
    let costs = stage::costs();

    let tiles = extent.tiles();
    let frames = FRAMES as u64;
    for stage in cachette_core::STAGES {
        let cost = costs.cost(*stage);
        println!(
            "{}\t{tiles}\t{placed}\t{threads}\t{FRAMES}\t{}\t{}\t{}\t{}\t{}",
            stage.name(),
            cost.entries,
            cost.nanos,
            cost.nanos / frames,
            stage.takes_threads(),
            stage.is_nested()
        );
    }
    let total = costs.total_nanos();
    println!(
        "all_stages\t{tiles}\t{placed}\t{threads}\t{FRAMES}\t{FRAMES}\t{total}\t{}\ttrue\tfalse",
        total / frames
    );
    let wall = u64::try_from(wall).unwrap_or(u64::MAX);
    println!(
        "frame_wall\t{tiles}\t{placed}\t{threads}\t{FRAMES}\t{FRAMES}\t{wall}\t{}\ttrue\tfalse",
        wall / frames
    );
    println!(
        "# anon_huge_pages_bytes\t{}",
        smaps_rollup_kib("AnonHugePages:") * 1024
    );
    println!("# resident_bytes\t{}", status_kib("VmRSS:") * 1024);
}

/// Reads how the kernel is set to give transparent huge pages.
///
/// The value is the whole line, with the current setting in square brackets.
/// A row that names a huge page setting is reproducible; a row that does not
/// is a figure about a machine nobody can name.
fn huge_page_setting() -> String {
    std::fs::read_to_string("/sys/kernel/mm/transparent_hugepage/enabled")
        .map_or_else(|_| "unreadable".to_owned(), |text| text.trim().to_owned())
}

/// Reads one field of the rolled up mapping summary of this process, in kB.
///
/// `AnonHugePages` is the part of the anonymous memory of the process that
/// sits on huge pages. It is the direct evidence that a huge page setting
/// reached this process, rather than the assumption that it did.
fn smaps_rollup_kib(field: &str) -> u64 {
    let Ok(text) = std::fs::read_to_string("/proc/self/smaps_rollup") else {
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

/// Places the units evenly over the whole world, and returns the count.
///
/// The other pattern fills the first open tiles it meets, which packs the
/// whole population into a band at one unit for each tile and leaves the rest
/// of the world empty. That is not the world the project describes: one
/// million units over 16777216 tiles is one unit for each seventeen tiles.
///
/// This pattern walks the world at a stride, so the units sit at the density
/// the target scale implies. It is the same on every run, because the stride
/// is arithmetic and which tiles hold water is a property of the seed.
fn populate_scattered(world: &mut World, units: u32) -> u32 {
    if units == 0 {
        return 0;
    }
    let grid = world.grid();
    let width = grid.width();
    let count = grid.tile_count();
    let stride = (count / units).max(1);
    let ceiling = u32::from(world.config().faction_count.max(1));
    let mut placed = 0u32;
    // The cursor never goes backwards, so no tile is offered twice, and it
    // never stops early at water. An earlier version searched one stride and
    // gave up, which placed 762599 of a requested 1000000 and made the
    // comparison it existed for a comparison of two unit counts.
    let mut cursor = 0u32;
    for step in 0..units {
        cursor = cursor.max(step.saturating_mul(stride));
        while cursor < count {
            let address = Axial::new((cursor % width) as i32, (cursor / width) as i32);
            cursor += 1;
            if !world.admits_a_unit(address) {
                continue;
            }
            let faction = FactionId((placed % ceiling) as u16);
            if world.spawn_soldier(address, faction).is_ok() {
                placed += 1;
                break;
            }
        }
        if cursor >= count {
            break;
        }
    }
    placed
}

/// Measures one frame under both placement patterns, back to back.
///
/// The two rows come from one process, on one machine, from one build, so the
/// difference between them is the placement and nothing else. A comparison
/// across two runs would carry the machine and the build as well.
///
/// No memory figure comes from this mode. The second world is built in a
/// process that has already built and dropped the first, so a resident size
/// read here would carry the first world with it. The memory mode measures
/// one point in a process of its own, and that is the figure to use.
///
/// This exists because a benchmark measures its fixture as much as its
/// subject. A population packed into a band gives every unit a neighbour,
/// concentrates the derived structure into few cells, and puts every
/// admission in contention. A population at the density the project states
/// does none of those.
fn placement_rows(arguments: &[String]) {
    let extent = extent_argument(arguments, 1);
    let units: u32 = arguments
        .get(2)
        .and_then(|word| word.parse().ok())
        .expect("the third argument must be a unit count");
    let threads: usize = arguments
        .get(3)
        .and_then(|word| word.parse().ok())
        .unwrap_or(1);

    println!("# the same frame under two placement patterns, in one process");
    println!("# packed: one unit for each open tile, from the first tile");
    println!("# scattered: the units spread over the whole world at a stride");
    println!("bench\ttiles\tunits\tthreads\tsamples\tmin_ns\tmedian_ns\tmax_ns");

    // The thread list is a parameter, so one run answers whether a result
    // holds at every thread count under both patterns rather than at one.
    let thread_counts = numbers_from(THREADS_VAR, &[threads]);
    for (name, scattered) in [("packed", false), ("scattered", true)] {
        for threads in thread_counts.iter().copied() {
            let capacity = units.max(1024);
            let mut world =
                World::new(extent.config(capacity)).expect("the extent must describe a world");
            let placed = if scattered {
                populate_scattered(&mut world, units)
            } else {
                populate(&mut world, units)
            };
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
    }
}

/// Counts how far the choice pass would collapse if it decided per cell.
///
/// One weight profile serves every unit alive, so two units in the same level
/// 1 cell with the same need score the same options and choose alike. If that
/// is common at scale, a pass that decided once for each distinct pair rather
/// than once for each unit would do far less work.
///
/// **The collapse factor is the live unit count divided by the number of
/// distinct pairs.** A factor of one means every unit is already unique in its
/// cell and there is nothing to collapse. A factor near the units for each
/// cell means the need adds almost no variety.
///
/// The need is a Q16.16 quantity, so it takes many values, and how coarse a
/// bucket may be before behaviour changes is a design decision rather than
/// one this benchmark makes. The rows therefore report several bucket widths
/// and let a record choose. A shift of zero is the exact need.
///
/// Everything here comes through the public crate interface: the live units,
/// the tile of a unit, the need of a unit, and the block layout that names
/// the cell. Nothing reaches inside the engine.
fn collapse_rows(arguments: &[String]) {
    let extent = extent_argument(arguments, 1);
    let units: u32 = arguments
        .get(2)
        .and_then(|word| word.parse().ok())
        .expect("the third argument must be a unit count");
    let threads: usize = arguments
        .get(3)
        .and_then(|word| word.parse().ok())
        .unwrap_or(1);

    // The fractional part of the fixed point scale holds sixteen bits, so a
    // shift of sixteen buckets the need to whole units and a shift of twelve
    // buckets it to sixteenths.
    const SHIFTS: [u32; 6] = [0, 8, 12, 14, 16, 20];

    println!("# the collapse of the choice pass, if it decided for each cell");
    println!("# a pair is one level 1 cell and one bucketed need");
    println!("# ratio_milli is the live units divided by the pairs, times 1000");
    println!(
        "bench\tplacement\tshift\tunits\tcells\tpairs\tratio_milli\tbiggest_cell\tmedian_cell"
    );

    for (placement, scattered) in [("packed", false), ("scattered", true)] {
        let capacity = units.max(1024);
        let mut world =
            World::new(extent.config(capacity)).expect("the extent must describe a world");
        let placed = if scattered {
            populate_scattered(&mut world, units)
        } else {
            populate(&mut world, units)
        };
        // The frames matter. A world that has never stepped holds one need
        // for every unit, and a count taken there would report the collapse
        // of the fixture rather than of a running world.
        for _ in 0..WARMUP_FRAMES {
            world.step(threads).expect("the step must run");
        }

        let soldiers = world.soldiers();
        let layout = world.bridge().layout();
        let live: Vec<Entity> = soldiers.iter().collect();

        // A cell and a bucketed need pack into one word, so the count is a
        // sort and a scan. No hash map takes part, so no iteration order
        // reaches the result.
        let mut cells: Vec<u32> = Vec::with_capacity(live.len());
        let mut needs: Vec<i32> = Vec::with_capacity(live.len());
        for unit in &live {
            let Some(tile) = soldiers.tile(*unit) else {
                continue;
            };
            let Some(key) = layout.key_of(tile) else {
                continue;
            };
            cells.push(layout.block_of_key(key));
            needs.push(soldiers.need(*unit).unwrap_or(Fix32::ZERO).0);
        }

        let mut occupancy = cells.clone();
        occupancy.sort_unstable();
        occupancy.dedup();
        let cell_count = occupancy.len();

        // The spread of the population over the cells. A mean would hide a
        // world in which a few cells hold everybody.
        let mut per_cell: Vec<u32> = Vec::new();
        {
            let mut sorted = cells.clone();
            sorted.sort_unstable();
            let mut run = 0u32;
            for window in 0..sorted.len() {
                run += 1;
                if window + 1 == sorted.len() || sorted[window] != sorted[window + 1] {
                    per_cell.push(run);
                    run = 0;
                }
            }
        }
        per_cell.sort_unstable();
        let biggest = per_cell.last().copied().unwrap_or(0);
        let median = per_cell.get(per_cell.len() / 2).copied().unwrap_or(0);

        // How many values the need column actually holds. If this is one,
        // the pair count is the cell count and the row below says nothing
        // about the need at all. That is a property of the fixture and it
        // has to be reported rather than inferred from the pair count.
        let mut distinct_needs = needs.clone();
        distinct_needs.sort_unstable();
        distinct_needs.dedup();
        let lowest = needs.iter().copied().min().unwrap_or(0);
        let highest = needs.iter().copied().max().unwrap_or(0);
        println!(
            "# {placement}_distinct_need_values\t{}\t lowest {lowest}\t highest {highest}",
            distinct_needs.len()
        );

        for shift in SHIFTS {
            let mut pairs: Vec<u64> = Vec::with_capacity(cells.len());
            for (cell, need) in cells.iter().zip(needs.iter()) {
                // The need is signed, so the shift is arithmetic and the
                // bucket is offset into the unsigned range before it packs.
                let bucket = ((need >> shift) as i64 - i64::from(i32::MIN >> shift)) as u64;
                pairs.push((u64::from(*cell) << 32) | (bucket & 0xffff_ffff));
            }
            pairs.sort_unstable();
            pairs.dedup();
            let distinct = pairs.len().max(1);
            let ratio_milli = (live.len() as u64 * 1000) / distinct as u64;
            println!(
                "collapse\t{placement}\t{shift}\t{placed}\t{cell_count}\t{}\t{ratio_milli}\t{biggest}\t{median}",
                pairs.len()
            );
        }
    }
}

/// Measures the exit field derivation and the level 1 rebuild that feeds it.
///
/// The exit field is the first thing in this engine built on the claim that
/// cost should follow the lattice rather than the population. So the question
/// is whether it scales like the tile pass, which improves with threads, or
/// floors like the unit passes, which stop.
///
/// **The derivation is measured directly rather than by a difference.** The
/// field, its constructor and the level it reads are all public, so a caller
/// outside the engine builds one and derives into it. Nothing is switched off
/// and nothing is subtracted.
///
/// The level 1 rebuild is measured beside it, because the step runs the two
/// together and the rebuild is the part that takes a thread count.
fn exit_field_rows(arguments: &[String]) {
    let extent = extent_argument(arguments, 1);
    let units: u32 = arguments
        .get(2)
        .and_then(|word| word.parse().ok())
        .expect("the third argument must be a unit count");
    let default_threads: usize = arguments
        .get(3)
        .and_then(|word| word.parse().ok())
        .unwrap_or(1);
    let thread_counts = numbers_from(THREADS_VAR, &[default_threads]);

    println!("# the exit field derivation, and the level 1 rebuild that feeds it");
    println!("# the units are scattered, at the density the scale constants imply");
    println!("bench\ttiles\tunits\tthreads\tsamples\tmin_ns\tmedian_ns\tmax_ns");

    let capacity = units.max(1024);
    let mut world = World::new(extent.config(capacity)).expect("the extent must describe a world");
    let placed = populate_scattered(&mut world, units);
    for _ in 0..WARMUP_FRAMES {
        world
            .step(*thread_counts.first().unwrap_or(&1))
            .expect("the step must run");
    }

    let layout = world.bridge().layout();
    let cells = Grid::new(layout.blocks_wide(), layout.blocks_high())
        .expect("the block lattice must describe a grid");
    println!("# level_1_cells\t{}", cells.tile_count());

    for threads in thread_counts.iter().copied() {
        // The level 1 rebuild takes a thread count, so it can scale.
        let samples = samples_of(|| {
            let start = now();
            world
                .rebuild_pyramid(threads)
                .expect("the rebuild must run");
            start.elapsed().as_nanos()
        });
        report("level_1_rebuild", extent.tiles(), placed, threads, &samples);

        // The derivation takes no thread count. It cannot scale, and the
        // thread column is here to show that it does not rather than to
        // suggest that it might.
        let mut field = ExitField::new(cells);
        let pyramid = world.pyramid();
        let samples = samples_of(|| {
            let start = now();
            field.derive(pyramid);
            let elapsed = start.elapsed().as_nanos();
            std::hint::black_box(field.exit(0, 0));
            elapsed
        });
        report(
            "exit_field_derive",
            extent.tiles(),
            placed,
            threads,
            &samples,
        );
    }
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
    let scattered = arguments.get(4).map(String::as_str) == Some("scattered");

    let empty = status_kib("VmRSS:");
    let capacity = units.max(1024);
    let mut world = World::new(extent.config(capacity)).expect("the extent must describe a world");
    let placed = if scattered {
        populate_scattered(&mut world, units)
    } else {
        populate(&mut world, units)
    };
    for _ in 0..WARMUP_FRAMES {
        world.step(threads).expect("the step must run");
    }
    let resident = status_kib("VmRSS:");
    let peak = status_kib("VmHWM:");
    // The world is read after the sizes are taken, so nothing above can drop
    // it early and report the memory of a world that no longer exists.
    let tiles = world.tile_count() as u64;
    let label = if scattered {
        "memory_scattered"
    } else {
        "memory_packed"
    };
    println!(
        "{label}\t{}\t{placed}\t{threads}\t{}\t{}\t{}",
        extent.tiles(),
        empty * 1024,
        resident * 1024,
        peak * 1024
    );
    assert_eq!(tiles, extent.tiles(), "the world must hold the extent");
    drop(world);
}

/// Measures the resident memory of one world under both placement patterns.
///
/// Each point runs in a process of its own, so neither reading carries the
/// other. That is the difference between this and the placement timing mode,
/// which measures two worlds in one process and therefore reports no memory.
fn memory_placement(arguments: &[String]) {
    let binary = std::env::current_exe().expect("the benchmark must know its own path");
    let extent = extent_argument(arguments, 1);
    let units = arguments
        .get(2)
        .cloned()
        .expect("the third argument must be a unit count");
    let threads = arguments.get(3).cloned().unwrap_or_else(|| "1".to_owned());

    println!("bench\ttiles\tunits\tthreads\tempty_bytes\tresident_bytes\tpeak_bytes");
    for pattern in ["packed", "scattered"] {
        let output = std::process::Command::new(&binary)
            .arg("memory-point")
            .arg(format!("{}x{}", extent.width, extent.height))
            .arg(&units)
            .arg(&threads)
            .arg(pattern)
            .output()
            .expect("the child must run");
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
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
