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
//! [^1]: PRD-0002, a developer watches the world run. `docs/product/shaped/prd-0002-a-developer-watches-the-world-run.md`
//! [^2]: ADR-0067, the viewer reads the world and never writes to it, decision D4. `docs/adrs/draft/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`

use std::num::NonZeroUsize;

use cachette_core::{Axial, FactionId, World, WorldConfig};
use cachette_view::{paint, Camera, Canvas, Lap, Metrics};
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
/// [^1]: PRD-0002, a developer watches the world run. `docs/product/shaped/prd-0002-a-developer-watches-the-world-run.md`
const DEMO: WorldConfig = WorldConfig {
    width: 40,
    height: 28,
    seed: 0x0cac_4e77_e5ee_d001,
    faction_count: 4,
};

/// The number of soldiers the demonstration spawns.
const SOLDIERS: u32 = 220;

/// The reason the demonstration stopped.
#[derive(Debug)]
enum DemoError {
    /// The world settings do not describe a world.
    World(cachette_core::WorldError),
    /// A step refused to run.
    Step(cachette_core::StepError),
    /// A soldier could not be placed.
    Soldier(cachette_core::SoldierError),
    /// The window could not open.
    Window(minifb::Error),
}

impl std::fmt::Display for DemoError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::World(error) => write!(formatter, "the world refused to build: {error}"),
            Self::Step(error) => write!(formatter, "the step refused to run: {error}"),
            Self::Soldier(error) => write!(formatter, "a soldier refused to spawn: {error}"),
            Self::Window(error) => write!(formatter, "the window refused to open: {error}"),
        }
    }
}

impl std::error::Error for DemoError {}

/// Fills the world with soldiers, spread over the tiles.
///
/// The placement is arithmetic on the tile count, so it is the same on every
/// run. The demonstration reproduces, which the product record requires.[^1]
///
/// # References
///
/// [^1]: PRD-0002, a developer watches the world run. `docs/product/shaped/prd-0002-a-developer-watches-the-world-run.md`
fn populate(world: &mut World) -> Result<(), DemoError> {
    let grid = world.grid();
    let factions = world.config().faction_count.max(1);
    for index in 0..SOLDIERS {
        let q = (index * 7 % grid.width()) as i32;
        let r = (index * 5 % grid.height()) as i32;
        let faction = FactionId((index % u32::from(factions)) as u16);
        world
            .spawn_soldier(Axial::new(q, r), faction)
            .map_err(DemoError::Soldier)?;
    }
    Ok(())
}

fn main() -> Result<(), DemoError> {
    let threads = std::thread::available_parallelism()
        .unwrap_or(NonZeroUsize::MIN)
        .get()
        .min(12);

    let mut world = World::new(DEMO).map_err(DemoError::World)?;
    populate(&mut world)?;

    let mut canvas = Canvas::new(WINDOW_WIDTH, WINDOW_HEIGHT);
    let camera = Camera::fitting(&world, &canvas);

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
        "cachette: {} by {} tiles, {} soldiers, {threads} threads",
        DEMO.width,
        DEMO.height,
        world.soldiers().len()
    );
    println!("close the window or press escape to stop");

    // The clock is read here and nowhere that decides anything. The engine
    // runs the same steps whatever these numbers say.
    let mut metrics = Metrics::start();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let at = Lap::start();
        world.step(threads).map_err(DemoError::Step)?;
        metrics.step(at.elapsed());

        let at = Lap::start();
        paint::draw(&world, camera, &mut canvas);
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
