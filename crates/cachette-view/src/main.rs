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
use cachette_view::{Camera, Canvas, Lap, Metrics, Overlay};
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
/// The size is an input to a run. It is not the population the world is sized
/// for, and it is not a value any record or register holds.[^1]
///
/// # Why this number
///
/// **A site feeds exactly as many people as the food its survey measured.**
/// That is an identity and not an estimate. The founding sets the production
/// rate of a site to a sixteenth of the food the survey reached, and a person
/// draws a ration of a sixteenth of a full need on each application. The two
/// sixteenths cancel, so the people a site can carry is the number the
/// founding already prints.[^2] [^3]
///
/// At thirty, every founded site fed its whole group forever. Nobody went
/// short, no unit ever chose to forage, and no tile of the world was ever
/// gathered from. The demonstration showed a world in which the food layer
/// decided nothing.[^4]
///
/// **The size is chosen so that some ground cannot carry its group and other
/// ground can.** A watcher then sees both conditions at once, and the choice
/// a unit makes varies across the map instead of being the same everywhere.
/// A world where everybody is hungry says as little as one where nobody is.
///
/// The split follows the ground and not this number. The number only has to
/// fall inside the spread of what the four sites reach, and it sits at the
/// middle of that spread so that a small change in the ground does not push
/// every site to one side. **The run reports which sites cleared it**, so a
/// seed that loses the split says so instead of quietly going back to a world
/// where nothing is hungry.
///
/// # References
///
/// [^1]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
/// [^2]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
/// [^3]: Backlog item 0240, let the demonstration make a unit hungry. `docs/backlog/complete/0240-let-the-demonstration-make-a-unit-hungry.md`
/// [^4]: Findings register, FND-232. `docs/FINDINGS.md`
const GROUP: u32 = 48;

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
fn steer(camera: Camera, window: &Window, world: &World, canvas: &Canvas) -> Camera {
    let mut camera = camera;
    if window.is_key_down(Key::Minus) {
        camera = camera.zoomed_out(canvas);
    }
    if window.is_key_down(Key::Equal) {
        camera = camera.zoomed_in(canvas);
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

    camera.nudged(across, down, canvas).clamped(world, canvas)
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
    println!("hold tab to name the colours");
    println!("run `just inspect` for every number the window does not show");
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
        cachette_view::draw_frame(&world, camera, &metrics, &outcomes, overlay, &mut canvas)
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
