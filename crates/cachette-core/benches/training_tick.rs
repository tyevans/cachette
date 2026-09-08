//! What one tick of the training world costs, stage by stage.
//!
//! # Why this exists
//!
//! A profiling pass over the trainer found that the simulation holds between
//! 96 and 99 percent of the wall clock of a generation, and that one world
//! tick costs about 9.5 milliseconds on a development machine.[^1] Nothing
//! had ever measured inside that tick.
//!
//! The other benchmark measures a world that the benchmark populates
//! itself.[^2] That world holds no settlement, no holding and no controller,
//! so it does not run the passes that carry most of a training tick. This
//! benchmark builds the world the trainer builds: the same extent, the same
//! faction count, the same seeding, the same tick limit and the same
//! external seat.[^3]
//!
//! **Pin the process before you read a figure from this benchmark.** The
//! development machine holds two kinds of core, and the same tick costs 2.5
//! times as much on the smaller kind. A report holds that measurement and the
//! stage breakdown this benchmark produced.[^5]
//!
//! # Determinism
//!
//! Nothing here reaches the simulation. The stage table reads a clock and
//! writes two integers to a static, and no pass reads either.[^4] The
//! benchmark is not a test and no assertion in it reads a clock.
//!
//! # How to run it
//!
//! The table is behind the `stage-cost` feature and it is off by default. A
//! run without the feature reports zeros and says so in its preamble.
//!
//! ```text
//! cargo bench --bench training_tick --features stage-cost -- window 48x48 3 1 0 20
//! ```
//!
//! # References
//!
//! [^1]: Report 38, where the training time goes. `docs/research/reports/38-where-the-training-time-goes.md`
//! [^2]: The target cost benchmark. `crates/cachette-core/benches/target_cost.rs`
//! [^3]: The learner environment. `python/cachette/learn/env.py`
//! [^4]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
//! [^5]: Report 39, what one tick of the training world costs. `docs/research/reports/39-what-one-tick-of-the-training-world-costs.md`

use cachette_core::{stage, FactionId, World, WorldConfig, STAGES};

/// Reads the clock.
///
/// One lint forbids the clock across this workspace, because a simulation
/// that reads a clock gives an answer that depends on the load of the
/// machine. A benchmark is not a simulation and it takes no assertion from
/// this value.
#[allow(clippy::disallowed_methods)]
fn now() -> std::time::Instant {
    std::time::Instant::now()
}

/// What one run builds.
struct Shape {
    width: u32,
    height: u32,
    faction_count: u16,
    unit_capacity: u32,
    threads: usize,
    warmup: u64,
    frames: u64,
    seed: u64,
    tick_limit: u64,
}

impl Shape {
    /// Reads a shape from the arguments, and fills the rest from the trainer.
    ///
    /// The defaults are the ones the learner environment states.
    fn read(arguments: &[String]) -> Self {
        let (width, height) = arguments.get(1).map_or((48, 48), |word| {
            word.split_once('x')
                .map(|(left, right)| {
                    (
                        left.parse().expect("the width must be a number"),
                        right.parse().expect("the height must be a number"),
                    )
                })
                .expect("the extent must read as WIDTHxHEIGHT")
        });
        Self {
            width,
            height,
            faction_count: number(arguments, 2, 3),
            threads: number(arguments, 3, 1),
            warmup: number(arguments, 4, 0),
            frames: number(arguments, 5, 20),
            unit_capacity: number(arguments, 6, WorldConfig::TARGET_UNIT_POPULATION),
            seed: number(arguments, 7, 0),
            tick_limit: number(arguments, 8, 2500),
        }
    }

    /// Builds the world the trainer builds, and seeds it.
    fn build(&self) -> World {
        let mut world = World::new(WorldConfig {
            width: self.width,
            height: self.height,
            seed: self.seed,
            faction_count: self.faction_count,
            unit_capacity: self.unit_capacity,
        })
        .expect("the shape must describe a world");
        world.seed_world().expect("the world must seed");
        world.set_win_readers_enabled(true);
        world.set_tick_limit(self.tick_limit);
        world.set_externally_controlled(FactionId(0), true);
        world
    }

    /// Returns how many units live in the world.
    fn population(&self, world: &World) -> u32 {
        (0..self.faction_count)
            .map(|faction| world.population_of(FactionId(faction)))
            .sum()
    }
}

/// Reads one numeric argument, or gives back the default.
fn number<T: std::str::FromStr>(arguments: &[String], index: usize, fallback: T) -> T {
    arguments
        .get(index)
        .and_then(|word| word.parse().ok())
        .unwrap_or(fallback)
}

/// Writes the facts a reader needs before any figure below.
fn preamble(shape: &Shape) {
    println!("# recording\t{}", stage::is_recording());
    println!("# extent\t{}x{}", shape.width, shape.height);
    println!(
        "# tiles\t{}",
        u64::from(shape.width) * u64::from(shape.height)
    );
    println!("# faction_count\t{}", shape.faction_count);
    println!("# unit_capacity\t{}", shape.unit_capacity);
    println!("# threads\t{}", shape.threads);
    println!("# warmup_ticks\t{}", shape.warmup);
    println!("# frames\t{}", shape.frames);
    println!("# seed\t{}", shape.seed);
    println!("# tick_limit\t{}", shape.tick_limit);
    println!(
        "# available_parallelism\t{}",
        std::thread::available_parallelism().map_or(0, std::num::NonZero::get)
    );
}

/// Steps the world the given number of times and throws the result away.
fn run(world: &mut World, threads: usize, ticks: u64) {
    for _ in 0..ticks {
        let log = world.step(threads).expect("the step must run");
        std::hint::black_box(log.len());
    }
}

/// Prints the stage table for one window of frames.
fn window(arguments: &[String]) {
    let shape = Shape::read(arguments);
    preamble(&shape);
    let mut world = shape.build();
    run(&mut world, shape.threads, shape.warmup);
    let population = shape.population(&world);
    println!("# live_units\t{population}");

    stage::reset();
    let start = now();
    run(&mut world, shape.threads, shape.frames);
    let wall = u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX);
    let costs = stage::costs();

    println!("stage\tentries\ttotal_ns\tns_for_each_frame\ttakes_threads\tnested");
    for stage in STAGES {
        let cost = costs.cost(*stage);
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            stage.name(),
            cost.entries,
            cost.nanos,
            cost.nanos / shape.frames,
            stage.takes_threads(),
            stage.is_nested()
        );
    }
    let total = costs.total_nanos();
    println!(
        "all_stages\t{}\t{total}\t{}\ttrue\tfalse",
        shape.frames,
        total / shape.frames
    );
    println!(
        "frame_wall\t{}\t{wall}\t{}\ttrue\tfalse",
        shape.frames,
        wall / shape.frames
    );
    println!("# state_hash\t{:016x}", world.state_hash().finish());
}

/// Prints the wall time of one frame across an episode, in windows.
///
/// The trainer reported that throughput for each live world falls through an
/// episode. This mode walks one episode and prints what a frame costs in each
/// window, so that a reader sees where the rise happens rather than only that
/// it happened.
fn episode(arguments: &[String]) {
    let shape = Shape::read(arguments);
    preamble(&shape);
    let mut world = shape.build();
    println!("stage\tstart_tick\tlive_units\tns_for_each_frame");

    let windows = shape.warmup.max(1);
    for index in 0..windows {
        let start_tick = index * shape.frames;
        let population = shape.population(&world);
        stage::reset();
        let start = now();
        run(&mut world, shape.threads, shape.frames);
        let wall = u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX);
        let costs = stage::costs();
        for stage in STAGES {
            let cost = costs.cost(*stage);
            println!(
                "{}\t{start_tick}\t{population}\t{}",
                stage.name(),
                cost.nanos / shape.frames
            );
        }
        println!(
            "all_stages\t{start_tick}\t{population}\t{}",
            costs.total_nanos() / shape.frames
        );
        println!(
            "frame_wall\t{start_tick}\t{population}\t{}",
            wall / shape.frames
        );
    }
    println!("# state_hash\t{:016x}", world.state_hash().finish());
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.first().map_or("", String::as_str) {
        "episode" => episode(&arguments),
        _ => window(&arguments),
    }
}
