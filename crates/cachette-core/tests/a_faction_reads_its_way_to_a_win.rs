//! A faction reads how far it has walked the domination path, and it reads
//! the settlement it would march on.
//!
//! **The published progress of the domination path could not reach one.** The
//! share divided the seats a faction held by the seated factions, and the
//! reader that ends the game asks for the seats of the factions that are
//! still in the game. A faction that outlives three rivals of seven and then
//! takes the three seats that are left satisfies the reader while the share
//! reads four sevenths. A policy that climbed that share climbed toward a
//! value it could not reach, and across a measured league only the built-in
//! controller ever won by domination.[^1]
//!
//! **The second clause of that reader stays unpublished, and the fog is the
//! reason.** It ends the game for a faction that holds a unit while every
//! rival holds none. A share of it would state the unit count of a rival the
//! reader has never observed, and the layout publishes the units the reader
//! has seen and no other unit count.[^2]
//!
//! **The campaign objective was unpublished too.** The built-in controller
//! receives the nearest enemy settlement from a search over the whole board,
//! and a policy read no distance to an enemy settlement at all. The
//! positions this file tests publish the fogged reader, which is the reader
//! the legality answer of the campaign verb already uses, so a policy reads
//! no settlement it has never observed.[^2]
//!
//! Every test drives the world step, so each value comes from the observation
//! pass and not from a fixture.[^3]
//!
//! # References
//!
//! [^1]: The audit of the observation, sections 3.1 and 4.3. `docs/research/what-a-policy-cannot-see.md`
//! [^2]: PRD-0001, a faction sees only what it observes. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
//! [^3]: Testing Rules, section 5. `.agents/rules/testing.md`

use cachette_core::faction_observation::observation_schema;
use cachette_core::{Axial, Entity, FactionId, SightRules, TileKind, WinPath, World, WorldConfig};

/// The faction that reads in every fixture below.
const READER: FactionId = FactionId(0);

/// The faction whose seat the reader takes.
const RIVAL: FactionId = FactionId(1);

/// The faction that leaves the game, so that the seat requirement shrinks.
const LEAVER: FactionId = FactionId(2);

/// One in the width the array holds.
const ONE: i64 = 65536;

/// The people each founding settles.
const GROUP: u32 = 8;

/// A tick limit no fixture here reaches, so the territory reader stays quiet.
const FAR_LIMIT: u64 = 100_000;

/// The exponent that keeps a unit still, so a fixture holds its shape.
const KEEP_STILL: u32 = 12;

/// Builds a world of three seated factions whose territory reader never
/// fires.
fn a_still_world(extent: u32, seed: u64) -> World {
    let mut world = World::new(WorldConfig {
        width: extent,
        height: extent,
        seed,
        faction_count: 3,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent describes a world");
    world.set_tick_limit(FAR_LIMIT);
    world
        .set_choice_schedule(KEEP_STILL)
        .expect("the exponent is inside the range");
    world.set_sight_rules(SightRules::new(4, 1, 16, 0));
    world
}

/// Returns every address of a world, in row-major order.
fn addresses(world: &World) -> Vec<Axial> {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .collect()
}

/// Founds one group of a faction away from the places given, and returns the
/// place, which becomes the seat of that faction.
fn seat(world: &mut World, faction: FactionId, apart_from: &[Axial]) -> Axial {
    for address in addresses(world) {
        if apart_from
            .iter()
            .any(|place| (place.q - address.q).abs() < 12 && (place.r - address.r).abs() < 12)
        {
            continue;
        }
        if world.found_group_at(address, GROUP, faction).is_ok() {
            return address;
        }
    }
    panic!("faction {} finds no place to found", faction.0);
}

/// Reads the first position of one field of the observation of one faction.
fn scalar(world: &World, faction: FactionId, name: &str) -> i64 {
    let field = observation_schema()
        .row(name)
        .unwrap_or_else(|| panic!("the schema must declare the field `{name}`"));
    let array = world
        .faction_observation(faction)
        .expect("the number names a faction of the world");
    array[field.start as usize]
}

/// Removes every unit and every settlement of one faction, so that the
/// elimination pass takes it out of the game.
fn strip(world: &mut World, faction: FactionId) {
    let units: Vec<Entity> = world.soldiers().iter().collect();
    for unit in units {
        if world.soldiers().faction(unit) == Some(faction) {
            assert!(world.despawn_soldier(unit), "the unit is alive");
        }
    }
    let sites: Vec<Entity> = world.settlements().iter().collect();
    for site in sites {
        if world.settlement_faction(site) == Some(faction) {
            assert!(world.destroy_settlement(site), "the settlement stands");
        }
    }
}

/// The published domination progress reaches one on the tick the seat clause
/// fires.
///
/// **The requirement of the reader shrinks when a faction leaves the game,
/// and the old denominator did not.** The fixture seats three factions and
/// takes one out of the game, so the reader asks the reader-faction for two
/// seats of three. A share over the seated count reads two thirds at the
/// moment the game ends, which is the defect this test holds.
#[test]
fn the_domination_progress_reads_one_when_the_seat_clause_fires() {
    let mut world = a_still_world(48, 0x0004_1001);
    let mine = seat(&mut world, READER, &[]);
    let theirs = seat(&mut world, RIVAL, &[mine]);
    seat(&mut world, LEAVER, &[mine, theirs]);
    world.step(1).expect("the step runs");

    strip(&mut world, LEAVER);
    world.step(1).expect("the step runs");
    assert!(
        world.is_eliminated(LEAVER),
        "the fixture must take the third faction out of the game, or the \
         requirement does not shrink and the test measures nothing"
    );

    // The rival keeps one unit far from its seat, so the unit clause cannot
    // fire and only the seat clause can end the game. Its city goes, and the
    // reader founds its own city on the seat, because a faction holds the
    // ground its cities reach and a unit standing on a tile gives its
    // faction no claim on it.
    let refuge = addresses(&world)
        .into_iter()
        .find(|address| {
            world.admits_a_unit(*address)
                && [mine, theirs].iter().all(|place| {
                    (place.q - address.q).abs() >= 12 || (place.r - address.r).abs() >= 12
                })
        })
        .expect("the world holds open ground away from both seats");
    let survivors: Vec<Entity> = world.soldiers().iter().collect();
    for unit in survivors {
        if world.soldiers().faction(unit) == Some(RIVAL) {
            assert!(world.despawn_soldier(unit), "the unit is alive");
        }
    }
    world
        .spawn_soldier(refuge, RIVAL)
        .expect("the refuge admits a unit");
    let rival_city = world
        .settlements()
        .on_tile(theirs)
        .expect("the founding left a city on the seat of the rival");
    assert!(world.destroy_settlement(rival_city));
    world
        .found_settlement(theirs, READER)
        .expect("the seat admits a city");

    let mut progress_at_the_end = None;
    for _ in 0..200 {
        world.step(1).expect("the step runs");
        let progress = scalar(&world, READER, "domination_progress");
        if world.game_end().is_set() {
            progress_at_the_end = Some(progress);
            break;
        }
        assert!(
            progress < ONE,
            "the progress reads one while the game runs on"
        );
    }
    let end = world.game_end();
    assert_eq!(end.win_path(), Some(WinPath::Domination));
    assert_eq!(end.winner, READER);
    assert!(
        world.population_of(RIVAL) > 0,
        "the rival still lives, so the seat clause and not the unit clause fired"
    );
    assert_eq!(
        progress_at_the_end,
        Some(ONE),
        "the published progress must read one on the tick the win fires"
    );
}

/// A faction reads how many rivals of the game are left, and it reads that
/// from the seats and never from a rival unit count.
///
/// **The unit clause of the domination reader stays unpublished.** A share of
/// the rivals that hold no unit would state the unit count of a rival the
/// reader has never observed, and the layout publishes the units the reader
/// has seen and no other unit count. The test holds that rule: it strips one
/// rival of everything it has and requires the array to keep the seen unit
/// statistics as its only unit answer about a rival.
#[test]
fn the_seat_share_and_not_a_rival_unit_count_carries_the_domination_path() {
    let mut world = a_still_world(48, 0x0004_1002);
    let mine = seat(&mut world, READER, &[]);
    let theirs = seat(&mut world, RIVAL, &[mine]);
    seat(&mut world, LEAVER, &[mine, theirs]);
    world.step(1).expect("the step runs");
    let before = scalar(&world, READER, "domination_progress");

    strip(&mut world, LEAVER);
    world.step(1).expect("the step runs");
    assert!(
        world.is_eliminated(LEAVER),
        "the fixture must take the third faction out of the game"
    );
    assert!(
        !world.game_end().is_set(),
        "one rival still holds a seat and units, so no reader fires"
    );
    let after = scalar(&world, READER, "domination_progress");
    assert!(
        after > before,
        "the requirement of the reader shrank by one seat, so the progress \
         rose from {before} to {after}"
    );
    assert!(
        after < ONE,
        "one rival seat is still unheld, so the progress is below one"
    );
    assert!(
        observation_schema().row("rivals_without_units").is_none(),
        "no field states the unit count of a rival the reader has not seen"
    );
}
/// A faction reads no campaign objective while it has seen no enemy
/// settlement, and reads one when it has.
///
/// **The fogged reader is the one this position publishes.** The fixture puts
/// the two factions at war and out of sight of each other, so a position that
/// read the unfogged search would name a distance the reader may not know.
#[test]
fn the_campaign_objective_reads_the_settlements_the_faction_has_seen() {
    let mut world = a_still_world(96, 0x0004_1003);
    let mine = seat(&mut world, READER, &[]);
    let theirs = seat(&mut world, RIVAL, &[mine]);
    let edge = world.relation_rules().war_edge;
    assert!(
        world.set_relation(READER, RIVAL, edge - 1),
        "the two numbers name two factions of the world"
    );
    world.step(1).expect("the step runs");
    assert!(
        world.at_war(READER, RIVAL),
        "the fixture must put the pair in the war band"
    );

    let seen = world
        .grid()
        .index_of(theirs)
        .expect("the seat lies inside the world");
    assert!(
        !world.observation().has_seen(READER, seen),
        "the fixture must start the reader out of sight of the enemy \
         settlement, or the test cannot see the fog rule"
    );
    assert_eq!(
        scalar(&world, READER, "campaign_objective_flag"),
        0,
        "a faction that has seen no enemy settlement reads no objective"
    );
    assert_eq!(
        scalar(&world, READER, "campaign_objective_distance"),
        0,
        "the distance means nothing while the flag reads zero"
    );

    world
        .spawn_soldier(theirs, READER)
        .expect("the seat admits a unit");
    world.step(1).expect("the step runs");
    assert!(
        world.observation().has_seen(READER, seen),
        "a unit of the reader stands on the enemy settlement, so the reader \
         has seen it"
    );
    assert_eq!(
        scalar(&world, READER, "campaign_objective_flag"),
        ONE,
        "a faction that has seen an enemy settlement reads an objective"
    );
    assert!(
        scalar(&world, READER, "campaign_objective_distance") > 0,
        "the two seats stand apart, so the distance is above zero"
    );
    assert_eq!(
        scalar(&world, READER, "campaign_objective_relief"),
        0,
        "the objective is a settlement of the rival, so it is a take and \
         not a relief"
    );
}

/// Wet ground is not a hazard, and the hazard share of held ground says so.
///
/// **The field summed the burning held tiles and the wet held tiles.** A fire
/// takes units and wet ground yields more to a gatherer, so the field added a
/// harm to a benefit and called the sum a hazard.
#[test]
fn wet_held_ground_is_no_hazard() {
    let mut world = a_still_world(48, 0x0004_1004);
    let mine = seat(&mut world, READER, &[]);
    world.step(1).expect("the step runs");
    world
        .inflict_weather(READER, &[mine], cachette_core::weather::STRENGTH_CEILING)
        .expect("the faction holds this ground");
    world.step(1).expect("the step runs");
    world.step(1).expect("the step runs");

    assert_eq!(
        world.ground_is_wet(mine),
        Some(true),
        "the fixture must wet the ground the reader holds, or the test \
         measures nothing"
    );
    assert_eq!(
        world.tile_is_burning(mine),
        Some(false),
        "the fixture must leave the ground unburnt"
    );
    assert!(
        scalar(&world, READER, "water_share_held") > 0,
        "the water share of the held ground must see the wet ground"
    );
    assert_eq!(
        scalar(&world, READER, "held_under_hazard_share"),
        0,
        "wet ground is no hazard"
    );
    assert_eq!(
        scalar(&world, READER, "settlements_under_hazard_share"),
        0,
        "a settlement on wet ground is under no hazard"
    );
}

/// A fire on held ground reads as the hazard of that ground.
#[test]
fn a_fire_on_held_ground_reads_as_a_hazard() {
    // Only ground that carries fuel catches, and the fuel of this engine is
    // wood, so the fixture founds the city on a wooded tile.
    let mut world = a_still_world(48, 0x0004_1005);
    let wood = addresses(&world)
        .into_iter()
        .find(|address| {
            world.tile_kind(*address) == Some(TileKind::Forest)
                && world.found_group_at(*address, GROUP, READER).is_ok()
        })
        .expect("the world holds wood that admits a founding");
    world.step(1).expect("the step runs");

    let lit = world
        .grid()
        .index_of(wood)
        .expect("the seat lies inside the world");
    assert!(world.ignite(lit), "the wood under the city must catch");
    world.step(1).expect("the step runs");

    assert_eq!(
        world.tile_is_burning(wood),
        Some(true),
        "the fixture must leave the held ground on fire"
    );
    let hazard = scalar(&world, READER, "held_under_hazard_share");
    assert!(hazard > 0, "a fire on held ground is a hazard");
    assert_eq!(
        hazard,
        scalar(&world, READER, "fire_share_held"),
        "fire is the whole of the hazard, so the two positions agree"
    );
}
