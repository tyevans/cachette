//! Watch the world run.
//!
//! One command builds a world, steps the engine, and shows the result. The
//! thing that moves the entities is the engine that the tests exercise, not
//! a loop shaped like one.[^1]
//!
//! The loop steps and then draws, on one thread. The drawing rate and the
//! tick rate are therefore one number.[^2]
//!
//! # References
//!
//! [^1]: PRD-0002, a developer watches the world run. `docs/product/shipped/prd-0002-a-developer-watches-the-world-run.md`
//! [^2]: ADR-0067, the viewer reads the world and never writes to it, decision D4. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`

use std::num::NonZeroUsize;

use cachette_core::{World, WorldConfig};
use cachette_view::{fill_frame, Camera, FrameSize, Lap, Metrics, Overlay, Surface};
use minifb::{Key, Window, WindowOptions};

/// The size of the window in pixels.
const WINDOW_WIDTH: usize = 960;

/// The size of the window in pixels.
///
/// The window holds cards and not a panel, and the cards fit any window a
/// person opens. The height is therefore free again, and the map takes what
/// the cards do not.[^1]
///
/// # References
///
/// [^1]: Decisions register, DEC-084. `docs/DECISIONS.md`
const WINDOW_HEIGHT: usize = 720;

/// The world the demonstration builds.
///
/// The product record bounds the demonstration to a world small enough to
/// watch, so this is not the target scale and does not pretend to be.[^1]
///
/// # References
///
/// [^1]: PRD-0002, a developer watches the world run. `docs/product/shipped/prd-0002-a-developer-watches-the-world-run.md`
const DEMO: WorldConfig = WorldConfig {
    width: 640,
    height: 440,
    seed: 0x0cac_4e77_e5ee_d001,
    faction_count: 4,
    unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
};

/// The number of people the demonstration founds its run with.
///
/// **The core declares this number, and this binary reads it.** The window
/// once declared its own copy of forty-eight. The project owner set the core
/// default to two, that instruction reached the core and not this file, and
/// nothing failed when the two copies disagreed. A watcher then could not tell
/// which of two worlds the window showed.[^1]
///
/// A founding of forty-eight people into housing of sixteen leaves no free
/// place on the first tick, so no birth is possible for the whole run. The
/// window therefore showed a world whose growth was closed by construction,
/// and the Python demonstration showed a world whose growth was open.[^2]
///
/// The size is an input to a run. It is not the population the world is sized
/// for, and it is not a value any record or register holds.[^3]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1, redundant declaration sites. `.agents/rules/recurring-defects.md`
/// [^2]: Findings register, FND-548. `docs/FINDINGS.md`
/// [^3]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
const GROUP: u32 = cachette_core::FOUNDING_GROUP_DEFAULT;

/// The reason the demonstration stopped.
#[derive(Debug)]
enum DemoError {
    /// The world settings do not describe a world.
    World(cachette_core::WorldError),
    /// A step refused to run.
    Step(cachette_core::StepError),
    /// The run could not be founded.
    Founding(cachette_core::FoundingError),
    /// The window could not open.
    Window(minifb::Error),
    /// The frame refused to fill.
    Frame(cachette_view::FrameError),
}

impl std::fmt::Display for DemoError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::World(error) => write!(formatter, "the world refused to build: {error}"),
            Self::Step(error) => write!(formatter, "the step refused to run: {error}"),
            Self::Founding(error) => write!(formatter, "the run refused to begin: {error}"),
            Self::Window(error) => write!(formatter, "the window refused to open: {error}"),
            Self::Frame(error) => write!(formatter, "the viewer refused to fill a frame: {error}"),
        }
    }
}

impl std::error::Error for DemoError {}

/// Founds the run and says why the engine chose each place.
///
/// The demonstration begins as a group somebody could grow, not as a full
/// world of units doing nothing in particular.[^1] The engine chooses each
/// place from a bounded sample of the world, so the choice costs the same
/// whatever the extent is.[^2]
///
/// The run founds one group for each faction, in ascending faction index, and
/// each founding keeps a minimum distance from the foundings before it.[^3] A
/// faction that finds no admissible place is refused, and the demonstration
/// says so and runs on. It stops only when no faction was seated, because a
/// world with nobody in it shows nothing.
///
/// # References
///
/// [^1]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
/// [^2]: ADR-0075, the founding choice reads a bounded sample of the world, decision D1. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
/// [^3]: ADR-0076, a founding keeps a fixed distance from the foundings before it. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
fn found(world: &mut World) -> Result<Vec<cachette_core::FoundingOutcome>, DemoError> {
    let outcomes = world.found_run_for_every_faction(GROUP);
    let mut seated = 0usize;
    let mut carried = 0usize;
    let mut refusal = None;
    for outcome in &outcomes {
        let faction = outcome.faction().0;
        match outcome.result() {
            Ok(founded) => {
                let chosen = founded.survey().chosen().expect("the founding chose");
                let reached = chosen.provision();
                println!(
                    "faction {faction} founded at ({}, {}) with {} people, \
chosen from {} places",
                    founded.place().q,
                    founded.place().r,
                    founded.people().len(),
                    founded.survey().considered()
                );
                println!(
                    "  it reaches {} food, {} wood and {} stone, over {} open tiles, \
with {} of open water beside it",
                    reached.food.0,
                    reached.wood.0,
                    reached.stone.0,
                    reached.open_ground,
                    reached.water_edge
                );
                // The food the survey reached is the number of people the site
                // can carry, because the production rate and the ration are
                // both a sixteenth and the two cancel. The line says whether
                // this ground carries this group, so a seed that feeds every
                // group says so on the way past rather than quietly showing a
                // world in which nothing is ever hungry.[^4]
                //
                // [^4]: Findings register, FND-232. `docs/FINDINGS.md`
                if reached.food.0 >= GROUP {
                    carried += 1;
                    println!("  this ground carries its group of {GROUP}");
                } else {
                    println!(
                        "  this ground carries {} of its group of {GROUP}, and the rest go short",
                        reached.food.0
                    );
                }
                seated += 1;
            }
            Err(error) => {
                println!("faction {faction} found no place: {error}");
                refusal = Some(*error);
            }
        }
    }
    // The demonstration is a fixture, and a fixture that produces one
    // condition everywhere measures itself. This says which way the run came
    // out rather than assuming the split that the group size was chosen
    // for.[^5]
    //
    // [^5]: Testing rules, section 2a. `.claude/rules/testing.md`
    if seated > 0 && (carried == 0 || carried == seated) {
        println!(
            "note: every seated group is {}, so this run shows one condition and not two",
            if carried == 0 { "short" } else { "fed" }
        );
    }
    match seated {
        0 => Err(DemoError::Founding(
            refusal.unwrap_or(cachette_core::FoundingError::EmptyGroup),
        )),
        _ => Ok(outcomes),
    }
}

/// Returns the place the window opens on.
///
/// The group holds one small part of a large world, so a camera at the corner
/// shows empty ground. The window opens on the first place that was founded.
///
/// # Panics
///
/// Panics when no faction founded. The caller stops before this on that
/// outcome, so reaching here is a programming error in this binary.
fn opening_place(outcomes: &[cachette_core::FoundingOutcome]) -> cachette_core::Axial {
    outcomes
        .iter()
        .find_map(|outcome| outcome.founding().map(cachette_core::Founding::place))
        .expect("the run seated at least one faction")
}

/// Reads the keyboard and returns the camera the person asked for.
///
/// The camera, the scroll position and the zoom belong to the viewer. None
/// of them is pushed into the world.[^1]
///
/// # References
///
/// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
fn steer(camera: Camera, window: &Window, world: &World, size: &FrameSize) -> Camera {
    let mut camera = camera;
    if window.is_key_down(Key::Minus) {
        camera = camera.zoomed_out(size);
    }
    if window.is_key_down(Key::Equal) {
        camera = camera.zoomed_in(size);
    }

    // One press moves the view by one step, in each direction the person is
    // holding. The size of a step is a share of the window and not a count of
    // tiles, so a press covers the same part of the picture at every zoom.[^2]
    //
    // [^2]: Findings register, FND-209. `docs/FINDINGS.md`
    let mut across = 0.0;
    let mut down = 0.0;
    if window.is_key_down(Key::Left) || window.is_key_down(Key::A) {
        across -= 1.0;
    }
    if window.is_key_down(Key::Right) || window.is_key_down(Key::D) {
        across += 1.0;
    }
    if window.is_key_down(Key::Up) || window.is_key_down(Key::W) {
        down -= 1.0;
    }
    if window.is_key_down(Key::Down) || window.is_key_down(Key::S) {
        down += 1.0;
    }

    camera.nudged(across, down, size).clamped(world, size)
}

fn main() -> Result<(), DemoError> {
    let threads = std::thread::available_parallelism()
        .unwrap_or(NonZeroUsize::MIN)
        .get()
        .min(12);

    let mut world = World::new(DEMO).map_err(DemoError::World)?;
    let outcomes = found(&mut world)?;

    // **The binary owns the pixels, in the same way the control plane does.**
    // Both front ends reach the drawing through one command, and neither
    // draws anything itself. Two presenters over one renderer is what stops
    // the two disagreeing about the world.[^4]
    //
    // [^4]: ADR-0094, the caller owns the camera and the pixels, decision D5. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
    let mut pixels = vec![0_u32; WINDOW_WIDTH * WINDOW_HEIGHT];
    let size = FrameSize::new(WINDOW_WIDTH, WINDOW_HEIGHT);
    // The world is larger than the window, so the camera shows a part of it
    // at a legible tile size. The person scrolls to see the rest. The camera
    // is the viewer's own value and never reaches the engine.
    // The group holds one small part of a large world, so a camera at the
    // corner would show an empty map. The view opens on the place that was
    // founded, and the person scrolls away from it.
    let mut camera = Camera::opening()
        .looking_at(opening_place(&outcomes), &size)
        .clamped(&world, &size);

    let mut window = Window::new(
        "cachette — watch the world run",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        WindowOptions::default(),
    )
    .map_err(DemoError::Window)?;

    // The window limits its own update rate. The engine steps once for each
    // drawn frame, so the two rates are one number.
    window.set_target_fps(30);

    println!(
        "cachette: {} by {} tiles, {} people, {threads} threads",
        DEMO.width,
        DEMO.height,
        world.soldiers().len()
    );
    println!("arrow keys or WASD scroll, minus and equals zoom");
    println!("hold tab to name the colours");
    println!("run `just inspect` for every number the window does not show");
    println!("close the window or press escape to stop");

    // The clock is read here and nowhere that decides anything. The engine
    // runs the same steps whatever these numbers say.
    let mut metrics = Metrics::start();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        camera = steer(camera, &window, &world, &size);

        let at = Lap::start();
        world.step(threads).map_err(DemoError::Step)?;
        metrics.step(at.elapsed());

        let at = Lap::start();
        // The viewer refuses a world whose spatial structure no longer
        // describes it, rather than drawing a picture without its soldiers.
        // The step rebuilds that structure at the barrier, so this cannot
        // happen here, and a refusal means the loop changed.
        // The frame is the world and the panel that says what it holds. The
        // panel reads the counts of the pass that just ran, so the two belong
        // in one call and the tests drive that call.
        // The binary owns the founding report and lends it to the panel. The
        // world keeps no copy of it.
        // The window draws the cards. The whole panel goes to a rendered
        // picture, which one command produces and which no window height
        // cuts.[^3] The key holds no state: the keyboard says whether the
        // watcher wants the reference layer, and the answer lives for one
        // frame.
        //
        // [^3]: Decisions register, DEC-084. `docs/DECISIONS.md`
        let overlay = Overlay::Glass {
            reference: window.is_key_down(Key::Tab),
        };
        let surface =
            Surface::new(WINDOW_WIDTH, WINDOW_HEIGHT, &mut pixels).map_err(DemoError::Frame)?;
        fill_frame(&world, camera, &metrics, &outcomes, overlay, surface)
            .map_err(DemoError::Frame)?;
        metrics.draw(at.elapsed());

        let at = Lap::start();
        window
            .update_with_buffer(&pixels, WINDOW_WIDTH, WINDOW_HEIGHT)
            .map_err(DemoError::Window)?;
        metrics.show(at.elapsed());
    }

    println!(
        "stopped at tick {}, state hash {}",
        world.tick().0,
        world.state_hash()
    );
    metrics.report(
        world.grid().tile_count(),
        world.soldiers().len() as usize,
        threads,
    );
    Ok(())
}
