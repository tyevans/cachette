//! A wonder names the city whose ground holds it.
//!
//! The wonder lookup states each wonder of the world: its tile, its work
//! against the requirement, the faction that holds the tile, and the city
//! that the tile belongs to. A caller that wants to stop a wonder marches on
//! that city, so the lookup must name the right one.[^1]
//!
//! **The city is the nearest live city of the holder, and a tie goes to the
//! lower slot.** Each fixture below gives a wrong rule a different answer
//! from the right one, and it asserts that it did before it asserts the
//! answer.[^2]
//!
//! Every test drives the world step, so the holder comes from the holding
//! rewrite and the work comes from the builders.[^3]
//!
//! # References
//!
//! [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decisions D1 and D2. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
//! [^2]: Testing Rules, section 2a. `.agents/rules/testing.md`
//! [^3]: Testing Rules, section 5. `.agents/rules/testing.md`

use cachette_core::upgrade::UpgradeCategory;
use cachette_core::{Axial, Entity, FactionId, WonderSite, World, WorldConfig};

/// The faction that founds and builds in every fixture below.
const BUILDER: FactionId = FactionId(0);

/// The exponent that keeps a unit still.
const KEEP_STILL: u32 = 12;

/// The work this file asks a wonder for.
///
/// **The fixture states its own requirement and does not read the balance
/// value.** A small requirement lets a few builders move the work in a few
/// steps, and the work stays below the requirement, so no wonder finishes.
const FIXTURE_WONDER_WORK: u32 = 240;

/// The builders each fixture puts on the wonder tile.
const BUILDERS: u32 = 3;

/// The steps each fixture lets the builders work.
const WORK_STEPS: u32 = 4;

/// Builds a world in which no unit takes a movement intent and a wonder asks
/// for the fixture work.
fn a_still_world(seed: u64) -> World {
    let mut world = World::new(WorldConfig {
        width: 96,
        height: 96,
        seed,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        ..WorldConfig::DEFAULT
    })
    .expect("the configuration describes a world");
    world
        .set_choice_schedule(KEEP_STILL)
        .expect("the exponent is inside the range");
    assert!(
        world.set_wonder_work(FIXTURE_WONDER_WORK),
        "the default table holds a wonder row"
    );
    world
}

/// Returns an address that admits a unit, near the one asked for.
fn ground_near(world: &World, wanted: Axial, bound: i32) -> Axial {
    for ring in 0..=bound {
        for column in -ring..=ring {
            for row in -ring..=ring {
                if column.abs().max(row.abs()) != ring {
                    continue;
                }
                let candidate = Axial::new(wanted.q + column, wanted.r + row);
                if world.admits_a_unit(candidate) {
                    return candidate;
                }
            }
        }
    }
    panic!("the fixture found no ground that admits a unit near {wanted:?}");
}

/// Founds a city of the builder on one address, and returns it.
fn found_at(world: &mut World, place: Axial) -> Entity {
    world
        .spawn_soldier(place, BUILDER)
        .expect("the fixture places a unit on ground that admits one");
    world
        .found_settlement(place, BUILDER)
        .expect("the fixture founds a city on ground that admits one");
    world
        .settlement_on(place)
        .expect("the fixture just founded a city here")
}

/// Puts builders of the builder faction on one tile, orders each to build a
/// wonder, and lets them work.
fn build_a_wonder(world: &mut World, tile: Axial) {
    for _ in 0..BUILDERS {
        let unit = world
            .spawn_soldier(tile, BUILDER)
            .expect("the ground admits a unit");
        world
            .order_build(unit, UpgradeCategory::WONDER)
            .expect("the builder stands on ground its faction holds");
    }
    for _ in 0..WORK_STEPS {
        world.step(1).expect("the step runs");
    }
}

/// Returns the entry of the lookup for one tile.
fn the_wonder_on(world: &World, tile: Axial) -> WonderSite {
    let index = world
        .grid()
        .index_of(tile)
        .expect("the tile lies inside the world");
    world
        .wonder_sites()
        .into_iter()
        .find(|site| site.tile == index)
        .expect("the lookup names the tile the builders work on")
}

/// Returns the settlement slot of one city.
fn slot_of(world: &World, city: Entity) -> u32 {
    world
        .settlements()
        .slot_of(city)
        .expect("the city is alive")
}

/// The lookup names the city of the holder that stands nearest to the wonder.
///
/// **The fixture gives the far city the lower slot.** A lookup that named the
/// first city of the faction, or the city with the lower slot, would name the
/// far city, and the test asserts the near one.
#[test]
fn the_lookup_names_the_nearest_city_of_the_faction_that_holds_the_wonder() {
    let mut world = a_still_world(0x0a1d_5eed_0001_0001);
    let far_place = ground_near(&world, Axial::new(20, 48), 14);
    let far = found_at(&mut world, far_place);
    let near_place = ground_near(&world, Axial::new(64, 48), 14);
    let near = found_at(&mut world, near_place);
    world.step(1).expect("the step runs");
    build_a_wonder(&mut world, near_place);

    let site = the_wonder_on(&world, near_place);
    assert!(
        site.work > 0,
        "the fixture must put work on the wonder, and it reads {}",
        site.work
    );
    assert!(
        !site.is_finished(),
        "the fixture must leave the wonder unfinished"
    );
    assert_eq!(
        site.claim, 0,
        "a wonder under construction carries no claim"
    );
    assert_eq!(site.requirement, i64::from(FIXTURE_WONDER_WORK));
    assert_eq!(site.holder, Some(BUILDER), "the builders hold the ground");
    assert!(
        slot_of(&world, far) < slot_of(&world, near),
        "the fixture must give the far city the lower slot"
    );
    assert_eq!(
        site.settlement,
        Some(near),
        "the lookup names the city the wonder stands nearest to"
    );
}

/// Builds a world with two cities of the builder at one distance from a
/// wonder, founded in the order the caller asks, and returns the two cities
/// in that order.
fn two_cities_at_one_distance(west_first: bool) -> (World, Entity, Entity, Axial) {
    let mut world = a_still_world(0x0a1d_5eed_0002_0002);
    let spread = 3;
    let axes = [
        Axial::new(spread, 0),
        Axial::new(0, spread),
        Axial::new(spread, -spread),
    ];
    let (middle, axis) = (0..96)
        .flat_map(|row| (0..96).map(move |column| Axial::new(column, row)))
        .flat_map(|place| axes.iter().map(move |axis| (place, *axis)))
        .find(|(place, axis)| {
            [-1, 0, 1].iter().all(|side| {
                world.admits_a_unit(Axial::new(place.q + side * axis.q, place.r + side * axis.r))
            })
        })
        .expect("the world holds three open tiles on one hex axis");
    let west = Axial::new(middle.q - axis.q, middle.r - axis.r);
    let east = Axial::new(middle.q + axis.q, middle.r + axis.r);
    assert_eq!(middle.distance(west), middle.distance(east));
    let (first, second) = if west_first {
        (found_at(&mut world, west), found_at(&mut world, east))
    } else {
        (found_at(&mut world, east), found_at(&mut world, west))
    };
    world.step(1).expect("the step runs");
    build_a_wonder(&mut world, middle);
    (world, first, second, middle)
}

/// Two cities at one distance from a wonder resolve by the lower slot.
///
/// **Both cities reach the tile, and they stand at one distance from it.**
/// The test founds them in one order and then in the other. The city named
/// changes with the order, so the slot decides and not the side the city
/// stands on.
#[test]
fn a_tie_between_two_cities_at_one_distance_goes_to_the_lower_slot() {
    for west_first in [true, false] {
        let (world, first, second, middle) = two_cities_at_one_distance(west_first);
        let site = the_wonder_on(&world, middle);
        assert!(site.work > 0, "the fixture must put work on the wonder");
        assert_eq!(site.holder, Some(BUILDER), "the builders hold the ground");
        assert!(
            slot_of(&world, first) < slot_of(&world, second),
            "the city founded first takes the lower slot"
        );
        assert_eq!(
            site.settlement,
            Some(first),
            "the city with the lower slot wins the tie, founded west first: {west_first}"
        );
    }
}
