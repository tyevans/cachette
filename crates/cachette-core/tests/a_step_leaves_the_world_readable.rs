//! A reader between two steps never meets a stale derived structure.
//!
//! The world holds a bridge from each tile to the units that stand on it. The
//! bridge counts the changes of the unit arena, and it refuses every answer
//! once that count moves past the count it was built from. A reader that
//! draws the world calls the bridge between two steps, so the bridge must
//! describe the arena at the end of every step.
//!
//! **The controller runs last, and it founds cities.** A founding spends a
//! settler and seats a settlement, and both change the unit arena. A step
//! that ends without refreshing the bridge after the controller therefore
//! leaves the world unreadable, and the next reader is refused.
//!
//! This test steps and reads in turn, in the way the viewer does. It reads
//! through the public interface of the crate.[^1] It drives the engine and
//! not the mechanism, because the engine is what must leave the world
//! consistent.[^2]
//!
//! # References
//!
//! [^1]: Testing rules, section 6. `.agents/rules/testing.md`
//! [^2]: Testing rules, section 5. `.agents/rules/testing.md`

use cachette_core::{Axial, FactionId, World, WorldConfig};

/// The extent of the world the viewer draws.
const EXTENT: u32 = 256;

/// The factions the world holds.
const FACTIONS: u16 = 4;

/// The threads each step runs on.
const THREADS: usize = 4;

/// The ticks each run of the test makes.
///
/// The owner met the refusal past tick five hundred of a windowed run. The
/// count here covers that, and the seeds below reach a founding far sooner.
const TICKS: u32 = 600;

/// The seeds the test runs.
///
/// Seed three founds five cities inside four hundred ticks, and it is the
/// seed that reaches a controller founding earliest. The other two seeds
/// found once each, so the test covers a world with one founding and a world
/// with several.
const SEEDS: [u64; 3] = [1, 3, 5];

/// Builds the world that the viewer builds.
fn viewer_world(seed: u64) -> World {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed,
        faction_count: FACTIONS,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    world.seed_world().expect("the world seeds once");
    world
}

/// Reads the world in the way the viewer reads it after a step.
///
/// The viewer asks how many units stand on the tile under the pointer. The
/// address is fixed, because the answer this test wants is whether the bridge
/// answers at all.
fn draw(world: &World) -> Result<usize, String> {
    world
        .soldier_count_on(Axial::new(1, 1))
        .map_err(|error| error.to_string())
}

#[test]
fn a_reader_between_two_steps_is_never_refused() {
    for seed in SEEDS {
        let mut world = viewer_world(seed);
        // **The test reads after each step and not before the first one.**
        // The seeding call founds a city for every faction, and it leaves the
        // bridge stale in the same way the controller did. That is a separate
        // fault of a separate verb, and this test states what a step
        // promises.
        for tick in 0..TICKS {
            world.step(THREADS).expect("the step runs");
            if let Err(error) = draw(&world) {
                let sites = world
                    .settlements()
                    .iter()
                    .filter(|site| world.settlements().faction(*site).is_some())
                    .count();
                panic!("seed {seed}, tick {tick}, {sites} sites: the reader was refused: {error}");
            }
        }
    }
}

#[test]
fn the_step_that_founds_a_city_still_leaves_the_world_readable() {
    // **This test names the tick the founding happened on.** The test above
    // would pass over a run that founds nothing, so it would measure a world
    // where the case never arises.[^1] This one refuses to pass unless a
    // founding happened after the seeding, and it reads the world on that
    // tick.
    //
    // [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
    let mut world = viewer_world(3);
    let sites_of = |world: &World| -> usize {
        world
            .settlements()
            .iter()
            .filter(|site| world.settlements().faction(*site).is_some())
            .count()
    };
    let start = sites_of(&world);
    let mut last = start;
    let mut foundings = 0u32;
    for tick in 0..TICKS {
        world.step(THREADS).expect("the step runs");
        let now = sites_of(&world);
        if now > last {
            foundings += 1;
            draw(&world).unwrap_or_else(|error| {
                panic!("tick {tick}: the reader was refused after a founding: {error}")
            });
        }
        last = now;
    }
    assert!(
        foundings > 0,
        "the run must found a city, or the test measures a world where the case never arises"
    );
    println!("the run founded {foundings} cities after the seeding");
}

#[test]
fn a_faction_that_never_settles_still_leaves_the_world_readable() {
    // A world whose factions are all controlled from outside emits no settle
    // command, so this run holds the case the two tests above do not: the
    // controller changes nothing structural and the step must still end
    // consistent.
    let mut world = viewer_world(3);
    for index in 0..FACTIONS {
        world.set_externally_controlled(FactionId(index), true);
    }
    for tick in 0..128 {
        world.step(THREADS).expect("the step runs");
        draw(&world).unwrap_or_else(|error| panic!("tick {tick}: the reader was refused: {error}"));
    }
}
