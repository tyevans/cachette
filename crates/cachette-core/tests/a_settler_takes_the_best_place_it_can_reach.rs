//! A faction sends its settlers at the best place they can reach.
//!
//! The rule has three parts. A candidate place must lie beyond the founding
//! distance from every city the faction holds. It must be eligible, which
//! means ground that admits a settlement with room for the group. Among what
//! passes those two, the faction takes the highest scoring place inside the
//! reach of a settler.
//!
//! **The reach is what keeps the settler alive.** A settler carries no home,
//! so nothing feeds it and it starves after a measured number of ticks. A
//! probe measured that lifetime, and the reach is half of it, because a
//! settler walks one tile in one tick and does not walk in a straight
//! line.[^1]
//!
//! These tests go through the public interface of the crate.[^2] They drive
//! the engine and read the answer, because the engine is what must apply the
//! rule.[^3]
//!
//! # References
//!
//! [^1]: The settler range probe. `crates/cachette-core/examples/settler_range_probe.rs`
//! [^2]: Testing rules, section 6. `.agents/rules/testing.md`
//! [^3]: Testing rules, section 5. `.agents/rules/testing.md`

use cachette_core::founding::MINIMUM_FOUNDING_DISTANCE;
use cachette_core::unit_type::SETTLER;
use cachette_core::{Axial, Entity, FactionId, World, WorldConfig};

/// The extent of the world the probe runs.
const EXTENT: u32 = 256;

/// The factions the world holds.
const FACTIONS: u16 = 4;

/// The threads each step runs on.
const THREADS: usize = 4;

/// The ticks each run makes.
const TICKS: u32 = 400;

/// The seeds the tests run.
///
/// The settlement probe reports a founding after the seeding on seven of the
/// first eight seeds. These three cover a run with one founding and a run
/// with several.
const SEEDS: [u64; 3] = [1, 2, 3];

/// Builds the world the settlement probe builds.
fn probe_world(seed: u64) -> World {
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

/// Returns the places one faction holds a city at, in slot order.
fn places_of(world: &World, faction: FactionId) -> Vec<Axial> {
    world
        .settlements()
        .iter()
        .filter(|site| world.settlements().faction(*site) == Some(faction))
        .filter_map(|site| world.settlements().address(site))
        .collect()
}

#[test]
fn a_target_lies_beyond_the_founding_distance_from_every_city_of_the_faction() {
    // **This is the first part of the rule, and it reuses the distance the
    // founding already keeps.** The survey takes the sites of the faction as
    // the places taken, so the target choice states no distance of its own.
    let mut world = probe_world(3);
    let mut checked = 0u32;
    for _ in 0..TICKS {
        world.step(THREADS).expect("the step runs");
        for index in 0..FACTIONS {
            let faction = FactionId(index);
            let Some(target) = world.settling_target_of(faction) else {
                continue;
            };
            for place in places_of(&world, faction) {
                assert!(
                    place.distance(target) >= MINIMUM_FOUNDING_DISTANCE,
                    "the target {target:?} is {} from the city at {place:?}, \
                     inside the founding distance {MINIMUM_FOUNDING_DISTANCE}",
                    place.distance(target)
                );
            }
            checked += 1;
        }
    }
    assert!(
        checked > 0,
        "no faction offered a target, so the test measured nothing"
    );
    println!("the test checked {checked} targets");
}

#[test]
fn a_target_lies_inside_the_reach_of_a_settler() {
    // **This is what stops the settler starving.** A world-wide sample offers
    // places on the far side of the world, and a settler dies before it
    // reaches one. The bound is the part of the rule that a run without it
    // would break, so this assertion fails under an unbounded choice.
    //
    // The fallback is the one case the bound does not cover. A faction whose
    // sample holds no eligible place inside the reach takes the nearest
    // eligible place instead, because a settler that never walks founds
    // nothing.
    let reach = World::settler_reach();
    let mut world = probe_world(3);
    let mut inside = 0u32;
    let mut fallback = 0u32;
    for _ in 0..TICKS {
        world.step(THREADS).expect("the step runs");
        for index in 0..FACTIONS {
            let faction = FactionId(index);
            let Some(target) = world.settling_target_of(faction) else {
                continue;
            };
            let places = places_of(&world, faction);
            if places.is_empty() {
                continue;
            }
            let near = places
                .iter()
                .map(|place| place.distance(target))
                .min()
                .expect("the faction holds a city");
            if near <= reach {
                inside += 1;
            } else {
                fallback += 1;
            }
        }
    }
    println!("targets inside the reach {inside}, targets from the fallback {fallback}");
    assert!(
        inside > 0,
        "no target lay inside the reach, so the bound never applied"
    );
    assert!(
        inside > fallback,
        "the fallback answered more often than the bound, so the bound is not the rule"
    );
}

#[test]
fn a_faction_takes_the_best_place_in_reach_and_not_the_nearest() {
    // **This is the third part of the rule, and it is the part the owner
    // asked for.** The engine offered a target. The nearest eligible place in
    // the same sample is the answer the rule this change replaced would have
    // given. The test asserts that the engine did not take it, on at least
    // one tick of at least one run.
    //
    // The two answers agree whenever the nearest eligible place is also the
    // best one, so the test counts the ticks they differ rather than
    // asserting on every tick.
    let reach = World::settler_reach();
    let mut differed = 0u32;
    let mut compared = 0u32;
    for seed in SEEDS {
        let mut world = probe_world(seed);
        for _ in 0..TICKS {
            world.step(THREADS).expect("the step runs");
            for index in 0..FACTIONS {
                let faction = FactionId(index);
                let Some(target) = world.settling_target_of(faction) else {
                    continue;
                };
                let places = places_of(&world, faction);
                if places.is_empty() {
                    continue;
                }
                let survey = world
                    .survey_founding_apart(1, faction, &places)
                    .expect("the survey runs");
                // The nearest eligible place inside the reach, which is what
                // the rule this change replaced would have chosen.
                let nearest = survey
                    .candidates()
                    .iter()
                    .filter(|candidate| candidate.is_eligible())
                    .filter_map(|candidate| {
                        let at = world.grid().address_of(candidate.tile())?;
                        let near = places.iter().map(|place| place.distance(at)).min()?;
                        (near <= reach).then_some((near, candidate.tile().0, at))
                    })
                    .min_by_key(|(near, tile, _)| (*near, *tile));
                let Some((_, _, nearest)) = nearest else {
                    continue;
                };
                compared += 1;
                if nearest != target {
                    differed += 1;
                }
            }
        }
    }
    println!("compared {compared} targets, the best differed from the nearest {differed} times");
    assert!(
        compared > 0,
        "no eligible place lay inside the reach, so the test measured nothing"
    );
    assert!(
        differed > 0,
        "the engine took the nearest place every time, so it is not taking the best"
    );
}

/// Returns the level 1 cell that covers one address.
fn cell_of(world: &World, address: Axial) -> u32 {
    let layout = world.pyramid().layout();
    let tile = world
        .grid()
        .index_of(address)
        .expect("the address is inside the world");
    layout.block_of_key(layout.key_of(tile).expect("the tile is inside the world"))
}

#[test]
fn a_settler_sent_at_a_tile_arrives_at_that_tile_and_founds_there() {
    // **This is the case that found the last mile.** The destination field
    // holds one direction for each level 1 cell, and a cell is thirty-two
    // tiles a side. A settler followed the gradient until it entered the cell
    // that held its target, and the field then had nothing left to say: the
    // reach of a seed cell is zero, so the settler fell back to the uniform
    // keyed draw and walked at random inside that cell until it starved.[^1]
    //
    // The test asserts the tile and not the cell, because the cell is not the
    // place the caller named. It reports the tick the settler entered the
    // cell and the tick it stood on the tile, so a reader sees the last mile
    // rather than taking the assertion on trust.
    //
    // The controller of the faction stands down. The order this test makes is
    // the order it measures, and the controller would re-aim the settler at a
    // place of its own choosing on any tick it drew the settle choice.
    //
    // [^1]: Findings register, FND-315. `docs/FINDINGS.md`
    let mut world = probe_world(1);
    let faction = FactionId(0);
    world.set_externally_controlled(faction, true);
    let places = places_of(&world, faction);
    assert!(!places.is_empty(), "the seeding must seat the faction");
    let seat = places[0];

    let target = world
        .settling_target_of(faction)
        .expect("the faction offers a target");
    assert!(
        cell_of(&world, target) != cell_of(&world, seat),
        "a target in the cell of the seat would be reached without the field"
    );

    let settler: Entity = world
        .spawn_soldier(seat, faction)
        .expect("the seat admits one unit");
    assert!(
        world.set_unit_type(settler, SETTLER),
        "the settler row is in the table"
    );
    // The plane is the settling plane of the faction, which is its number
    // raised by twice the faction count.
    let plane = 2 * FACTIONS + faction.0;
    world
        .send_units_to(&[settler], &[target], plane)
        .expect("the world holds the plane and the unit is live");

    let mut entered = None;
    let mut arrived = None;
    for tick in 0..TICKS {
        world.step(THREADS).expect("the step runs");
        let Some(at) = world.soldiers().address(settler) else {
            break;
        };
        if cell_of(&world, at) == cell_of(&world, target) && entered.is_none() {
            entered = Some(tick);
        }
        if at == target {
            arrived = Some(tick);
            break;
        }
    }
    println!(
        "the settler left {seat:?} for {target:?}, {} tiles away. \
         it entered the target cell at {entered:?} and stood on the target tile at {arrived:?}",
        seat.distance(target)
    );
    let arrived = arrived.expect(
        "the settler never stood on the tile it was sent to, so the last mile is still open",
    );
    assert!(
        world.soldiers().contains(settler),
        "the settler must reach its target alive"
    );

    let before = world.settlements().len();
    let outcomes = world.settle_set(&[settler]);
    let founding = outcomes[0]
        .founding()
        .expect("a settler on an eligible place founds");
    assert_eq!(
        founding.place(),
        target,
        "the city must stand on the tile the caller named"
    );
    assert_eq!(
        world.settlements().len(),
        before + 1,
        "the founding must add a site"
    );
    println!("the settler founded at {target:?} after arriving on tick {arrived}");
}
