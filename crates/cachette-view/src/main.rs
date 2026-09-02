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
use cachette_view::{Camera, Canvas, Lap, Metrics};
use minifb::{Key, Window, WindowOptions};

/// The size of the window in pixels.
const WINDOW_WIDTH: usize = 960;
/// The size of the window in pixels.
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
};

/// The number of people the demonstration founds its run with.
///
/// The size is an input to a run. It is not the population the world is sized
/// for, and it is not a value any record or register holds.[^1]
///
/// # References
///
/// [^1]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
const GROUP: u32 = 30;

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
    /// The spatial structure no longer describes the world.
    Bridge(cachette_core::BridgeError),
}

impl std::fmt::Display for DemoError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::World(error) => write!(formatter, "the world refused to build: {error}"),
            Self::Step(error) => write!(formatter, "the step refused to run: {error}"),
            Self::Founding(error) => write!(formatter, "the run refused to begin: {error}"),
            Self::Window(error) => write!(formatter, "the window refused to open: {error}"),
            Self::Bridge(error) => write!(
                formatter,
                "the viewer refused to draw a world it could not read: {error}"
            ),
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
                seated += 1;
            }
            Err(error) => {
                println!("faction {faction} found no place: {error}");
                refusal = Some(*error);
            }
        }
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
fn steer(camera: Camera, window: &Window, world: &World, canvas: &Canvas) -> Camera {
    let mut camera = camera;
    if window.is_key_down(Key::Minus) {
        camera = camera.zoomed_out(canvas);
    }
    if window.is_key_down(Key::Equal) {
        camera = camera.zoomed_in(canvas);
    }

    // One press moves the view by one and a half tiles, in each direction
    // the person is holding.
    let mut across = 0.0;
    let mut down = 0.0;
    if window.is_key_down(Key::Left) || window.is_key_down(Key::A) {
        across -= 1.5;
    }
    if window.is_key_down(Key::Right) || window.is_key_down(Key::D) {
        across += 1.5;
    }
    if window.is_key_down(Key::Up) || window.is_key_down(Key::W) {
        down -= 1.5;
    }
    if window.is_key_down(Key::Down) || window.is_key_down(Key::S) {
        down += 1.5;
    }

    camera.stepped(across, down).clamped(world, canvas)
}

fn main() -> Result<(), DemoError> {
    let threads = std::thread::available_parallelism()
        .unwrap_or(NonZeroUsize::MIN)
        .get()
        .min(12);

    let mut world = World::new(DEMO).map_err(DemoError::World)?;
    let outcomes = found(&mut world)?;

    let mut canvas = Canvas::new(WINDOW_WIDTH, WINDOW_HEIGHT);
    // The world is larger than the window, so the camera shows a part of it
    // at a legible tile size. The person scrolls to see the rest. The camera
    // is the viewer's own value and never reaches the engine.
    // The group holds one small part of a large world, so a camera at the
    // corner would show an empty map. The view opens on the place that was
    // founded, and the person scrolls away from it.
    let mut camera = Camera::opening()
        .looking_at(opening_place(&outcomes), &canvas)
        .clamped(&world, &canvas);

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
    println!("close the window or press escape to stop");

    // The clock is read here and nowhere that decides anything. The engine
    // runs the same steps whatever these numbers say.
    let mut metrics = Metrics::start();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        camera = steer(camera, &window, &world, &canvas);

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
        cachette_view::draw_frame(&world, camera, &metrics, &outcomes, &mut canvas)
            .map_err(DemoError::Bridge)?;
        metrics.draw(at.elapsed());

        let at = Lap::start();
        window
            .update_with_buffer(canvas.pixels(), canvas.width(), canvas.height())
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
