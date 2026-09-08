//! A reader outside a step never meets a stale derived structure.
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
//! **The seeding founds a city for every faction, and it runs before the
//! first step.** It changes the unit arena in the same way, so the same rule
//! binds it: a verb that changes the structure leaves the world readable. One
//! rule holds both, so one file tests both. A second file over the same rule
//! would be one statement in two places.[^3]
//!
//! **Every caller-facing verb answers to that rule, and the second half of
//! this file drives each of them.** The control plane sends an action,
//! founds a city from a settler, or converts a set, between two steps. Each
//! of those changes the unit arena and reaches no barrier, so each restores
//! the structure before it returns. Four reports of one refusal came from a
//! reader that ran in that gap.[^4]
//!
//! **The refusal names its cause.** The reader answered nothing at all, and
//! the caller then had the fact of a refusal and none of the reason. One
//! test below reads what a stale structure says.[^4]
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
//! [^3]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
//! [^4]: Findings register, FND-644 and FND-647. `docs/FINDINGS.md`

use cachette_core::action::Verb;
use cachette_core::bridge::BridgeError;
use cachette_core::faction_view::FactionViewError;
use cachette_core::unit_type::SETTLER;
use cachette_core::{Axial, Entity, FactionId, World, WorldConfig};

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
    let outcomes = world.seed_world().expect("the world seeds once");
    // The seeding must seat somebody, or the world holds no unit and every
    // read below asks a bridge that nothing made stale. The fixture would
    // then measure itself.[^1]
    //
    // [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
    let seated = outcomes
        .iter()
        .filter(|outcome| outcome.is_seated())
        .count();
    assert!(
        seated > 0,
        "seed {seed}: the seeding seated no faction, so the world holds no unit"
    );
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
        // **The test reads before the first step as well as after each one.**
        // The seeding call founds a city for every faction, and it changed
        // the unit arena in the same way the controller does. The seeding
        // refreshes the bridge before it returns, so this read is the case
        // that proves it.
        if let Err(error) = draw(&world) {
            panic!("seed {seed}: the reader was refused before the first step: {error}");
        }
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

/// The people that the founding of the fixture settles.
const SETTLE_GROUP: u32 = 8;

/// The faction that acts in every test below.
const ACTOR: FactionId = FactionId(0);

/// Builds a small world of two factions, for the verb tests below.
fn small_config(seed: u64) -> WorldConfig {
    WorldConfig {
        width: 48,
        height: 48,
        seed,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    }
}

/// Returns an address that admits a unit, at or after the one asked for.
///
/// The terrain comes from the seed, so the address a test names may hold
/// water. The walk is over the columns of one row, in ascending order, so
/// two runs return one answer.
fn ground_on_row(world: &World, row: i32, from: i32) -> Axial {
    for column in from..world.grid().width() as i32 {
        let candidate = Axial::new(column, row);
        if world.admits_a_unit(candidate) {
            return candidate;
        }
    }
    panic!("the fixture found no ground that admits a unit on row {row}");
}

/// Builds a world that holds one settled faction and one live rival unit.
///
/// The world steps once after the founding, so the fog layers name the cells
/// the factions observe. **The reader walks the tiles of an observed cell
/// only**, so a fixture that never stepped would read no tile and would
/// reach no derived structure at all.
fn a_settled_world(seed: u64) -> World {
    let mut world = World::new(small_config(seed)).expect("the extent describes a world");
    let home = ground_on_row(&world, 10, 4);
    world
        .found_group_at(home, SETTLE_GROUP, ACTOR)
        .expect("the survey accepts the place of the fixture");
    let away = ground_on_row(&world, 30, 4);
    world
        .spawn_soldier(away, FactionId(1))
        .expect("the ground admits a unit");
    world.step(THREADS).expect("the step runs");
    let observation = world
        .faction_observation(ACTOR)
        .expect("a stepped world answers its own faction");
    assert!(
        observation.iter().any(|value| *value != 0),
        "the fixture must give the acting faction something to observe"
    );
    world
}

/// Puts a settler of the acting faction on ground far from every city.
fn a_settler(world: &mut World) -> Entity {
    let place = ground_on_row(world, 40, 4);
    let unit = world
        .spawn_soldier(place, ACTOR)
        .expect("the ground admits a unit");
    assert!(
        world.set_unit_type(unit, SETTLER),
        "the arena takes the settler row for a live unit"
    );
    world.step(THREADS).expect("the step runs");
    unit
}

/// The settling action leaves the observation of every faction readable.
///
/// This is the reported defect. A stored policy sent this action, and the
/// policy of the next faction read its observation before the next step.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-644. `docs/FINDINGS.md`
#[test]
fn a_settling_action_leaves_the_world_readable() {
    let mut world = a_settled_world(0x0cac_4e77_0472);
    a_settler(&mut world);
    let sites = world.settlements().len();

    let action = world
        .action_schema()
        .encode(Verb::Settle, &[])
        .expect("the settle verb declares no argument position");
    assert!(
        world.act(ACTOR, action),
        "the fixture must give the acting faction a settler that founds"
    );
    assert!(
        world.settlements().len() > sites,
        "the fixture must reach the case: the action founded a city"
    );

    // The reader answers, and it answers for the faction that acted and for
    // the faction that did not.
    for number in 0..world.faction_count() {
        world
            .faction_observation(FactionId(number))
            .unwrap_or_else(|error| {
                panic!("the world answers faction {number} after an action: {error}")
            });
    }
}

/// The founding verb leaves the world readable, and the action verb is not
/// the only path to it.
#[test]
fn the_founding_verb_leaves_the_world_readable() {
    let mut world = a_settled_world(0x0cac_4e77_0473);
    let settler = a_settler(&mut world);
    let sites = world.settlements().len();

    let outcomes = world.settle_set(&[settler]);
    assert!(
        outcomes
            .iter()
            .any(cachette_core::founding::SettleOutcome::founded),
        "the fixture must reach the case: the verb founded a city"
    );
    assert!(world.settlements().len() > sites);
    world
        .faction_observation(ACTOR)
        .expect("the world answers after the founding verb");
}

/// The conversion verb leaves the world readable.
#[test]
fn the_conversion_verb_leaves_the_world_readable() {
    let mut world = a_settled_world(0x0cac_4e77_0474);
    let taken = world
        .soldiers()
        .iter_faction(FactionId(1))
        .next()
        .expect("the fixture puts one unit in the hands of the rival");
    let before = world.population_of(ACTOR);

    world
        .convert_units(&[taken], ACTOR)
        .expect("the verb takes a live unit and a faction of this world");
    assert!(
        world.population_of(ACTOR) > before,
        "the fixture must reach the case: the verb changed the arena"
    );
    world
        .faction_observation(ACTOR)
        .expect("the world answers after the conversion verb");
}

/// A refusal names the cause, and a stale structure names both revisions.
///
/// The spawn primitive is not a caller-facing verb. It runs inside the step,
/// in a loop, so it leaves the structure to the barrier of the step. That
/// makes it the one honest way to produce a stale structure here, and the
/// test then reads what the refusal says.
///
/// **A refusal that named nothing sent four readers of one traceback after
/// four different causes.**[^1]
///
/// # References
///
/// [^1]: Findings register, FND-647. `docs/FINDINGS.md`
#[test]
fn a_refusal_names_the_revisions_of_a_stale_structure() {
    let mut world = a_settled_world(0x0cac_4e77_0475);
    let place = ground_on_row(&world, 12, 4);
    world
        .spawn_soldier(place, ACTOR)
        .expect("the ground admits a unit");

    let refusal = world
        .faction_observation(ACTOR)
        .expect_err("a structure the arena has moved past answers nothing");
    let FactionViewError::Bridge(BridgeError::Stale { built, current }) = refusal else {
        panic!("the refusal must name a stale structure, and it said {refusal}");
    };
    assert!(
        current > built,
        "the arena stands above the structure, and the refusal states both"
    );
    let said = refusal.to_string();
    assert!(
        said.contains(&built.to_string()) && said.contains(&current.to_string()),
        "the message names both revisions, and it said {said}"
    );

    // The public verb that restores the structure is what a caller of the
    // primitive is obliged to call, and the world answers again after it.
    world
        .rebuild_bridge(THREADS)
        .expect("the rebuild takes the arena of this world");
    world
        .faction_observation(ACTOR)
        .expect("the world answers once the structure describes the arena");
}
