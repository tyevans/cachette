//! A settler founds a city, and the controller settles new ground.
//!
//! A settler is a unit whose type row holds a settle column above zero. One
//! verb founds a settlement on the tile the unit stands on, for the faction
//! of the unit, and the founding spends the settler. The verb refuses a unit
//! whose settle column is zero, a tile any faction holds, a tile that carries
//! a settlement, ground that admits no unit, and a place inside the founding
//! distance of a city that stands.[^1] [^2]
//!
//! **Every fixture here is built for an extreme, not for the demonstration
//! world.** A settler at exactly the founding distance, a settler one tile
//! inside it, a settler on ground its own faction holds, and a world of one
//! city and one settler. A fixture copied from the demonstration binary
//! supplies no extreme, so the assertion would measure the fixture.[^3]
//!
//! The controller test drives the world step. The engine must invoke the
//! verb, and a capability that nothing invokes passes its own test and ships
//! inert.[^4]
//!
//! # References
//!
//! [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D5. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
//! [^2]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D1. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
//! [^3]: Testing rules, section 2a. `.agents/rules/testing.md`
//! [^4]: Testing rules, section 5. `.agents/rules/testing.md`

use cachette_core::founding::{SettleError, MINIMUM_FOUNDING_DISTANCE};
use cachette_core::holding::{Holder, ReachRules};
use cachette_core::unit_type::{SETTLER, SETTLER_ROW, SOLDIER, SOLDIER_ROW, WORKER};

// **Which unit type may found a city is a property of the shared table, so
// the check belongs at compile time and not inside one test.** A runtime
// assertion on two constants passes for every input the test could supply,
// so it measured nothing and the test read as though it had covered the
// rule.
const _: () = assert!(SOLDIER_ROW.settle_group == 0);
const _: () = assert!(SETTLER_ROW.settle_group > 0);
use cachette_core::{Axial, Entity, FactionId, World, WorldConfig};

/// The extent of the worlds below.
///
/// It is wide enough to hold two places at the founding distance and the
/// ground between them.
const EXTENT: u32 = 96;

/// Builds a world of the extent that no seeding has touched.
fn world(seed: u64) -> World {
    World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world")
}

/// Returns a world with one city, and the place that city stands on.
///
/// The search walks the seeds until it finds one whose ground admits a city
/// at the centre. The walk reads the world and draws nothing of its own, so
/// it adds no state to the fixture.
fn one_city(seed: u64) -> (World, Axial) {
    let centre = Axial::new((EXTENT / 2) as i32, (EXTENT / 2) as i32);
    for step in 0..64u64 {
        let mut field = world(seed + step);
        field.set_reach_rules(ReachRules::new(4, 4, 8));
        // The founding path records the seat of the faction. A settlement
        // founded by the address verb alone leaves the faction seatless, and
        // the game then ends on the first step, before any controller runs.
        if field.found_group_at(centre, 1, FactionId(0)).is_ok() {
            return (field, centre);
        }
    }
    panic!("no seed of the walk admits a city at the centre");
}

/// Puts a settler of a faction on a tile, and returns it.
///
/// The type is the settler row of the default table, so the test writes no
/// row of its own and reads the same table the engine does.
fn settler_at(field: &mut World, address: Axial, faction: FactionId) -> Entity {
    let unit = field
        .spawn_soldier(address, faction)
        .expect("the ground admits a unit");
    assert!(
        field.set_unit_type(unit, SETTLER),
        "the row is in the table"
    );
    unit
}

/// Returns the first passable tile at exactly this distance from a place.
fn ground_at_distance(field: &World, from: Axial, distance: u32) -> Axial {
    let grid = field.grid();
    for index in 0..grid.tile_count() {
        let address = Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32);
        if from.distance(address) == distance && field.admits_a_unit(address) {
            return address;
        }
    }
    panic!("the world holds no passable tile at the distance {distance}");
}

#[test]
fn a_settler_founds_a_city_and_the_founding_spends_it() {
    // The extreme is the exact founding distance. A rule that compared with
    // the wrong sense would refuse here, and a fixture placed far away would
    // never reach the comparison.
    let (mut field, city) = one_city(700);
    let place = ground_at_distance(&field, city, MINIMUM_FOUNDING_DISTANCE);
    let settler = settler_at(&mut field, place, FactionId(1));
    let before = field.settlements().len();
    let population = field.population_of(FactionId(1));

    let outcomes = field.settle_set(&[settler]);

    assert_eq!(outcomes.len(), 1, "the verb answers each unit of the set");
    let founding = outcomes[0]
        .founding()
        .expect("a settler at the founding distance founds");
    assert_eq!(
        founding.place(),
        place,
        "the city stands where the unit did"
    );
    assert_eq!(field.settlements().len(), before + 1);
    assert!(
        !field.soldiers().contains(settler),
        "the founding spends the settler"
    );
    assert_eq!(
        field.population_of(FactionId(1)),
        population,
        "the settle group takes the place of the settler"
    );
    assert!(field.check_invariants());
}

#[test]
fn a_settler_inside_the_founding_distance_is_refused() {
    // The extreme is one tile inside the distance. A rule that used a strict
    // comparison at the wrong end would found here.
    let (mut field, city) = one_city(700);
    let place = ground_at_distance(&field, city, MINIMUM_FOUNDING_DISTANCE - 1);
    let settler = settler_at(&mut field, place, FactionId(1));
    let before = field.settlements().len();

    let outcomes = field.settle_set(&[settler]);

    assert_eq!(
        outcomes[0].result(),
        &Err(SettleError::TooCloseToACity(place)),
        "the refusal names the distance and not the ground"
    );
    assert_eq!(
        field.settlements().len(),
        before,
        "a refusal changes nothing"
    );
    assert!(
        field.soldiers().contains(settler),
        "a refused settler keeps its life"
    );
}

#[test]
fn a_unit_whose_settle_column_is_zero_is_refused() {
    // The soldier row and the worker row both hold a settle column of zero,
    // and they differ in every other column. The verb must refuse both for
    // one reason.
    let (mut field, city) = one_city(700);
    let place = ground_at_distance(&field, city, MINIMUM_FOUNDING_DISTANCE);
    let worker = field
        .spawn_soldier(place, FactionId(1))
        .expect("the ground admits a unit");
    assert!(field.set_unit_type(worker, WORKER));
    let soldier = field
        .spawn_soldier(place, FactionId(1))
        .expect("the ground admits a unit");
    assert!(field.set_unit_type(soldier, SOLDIER));
    let before = field.settlements().len();

    let outcomes = field.settle_set(&[worker, soldier]);

    assert_eq!(
        outcomes[0].result(),
        &Err(SettleError::NotASettler(worker)),
        "the verb reads the settle column and not the type index"
    );
    assert_eq!(
        outcomes[1].result(),
        &Err(SettleError::NotASettler(soldier))
    );
    assert_eq!(
        field.settlements().len(),
        before,
        "a refused set founds nothing"
    );
}

#[test]
fn a_settler_on_ground_a_faction_holds_is_refused() {
    // The extreme is the settler's own ground. The reach of a city is below
    // the founding distance, so this refusal needs its own fixture: a
    // settler beside the city it came from.
    let (mut field, city) = one_city(700);
    field.step(2).expect("the step must run");
    let beside = ground_at_distance(&field, city, 1);
    let settler = settler_at(&mut field, beside, FactionId(0));

    let outcomes = field.settle_set(&[settler]);

    assert_eq!(
        outcomes[0].result(),
        &Err(SettleError::GroundIsHeld(beside)),
        "a settler founds only where nobody holds"
    );
}

#[test]
fn a_founded_city_holds_the_ground_around_it() {
    // The holding pass must give the new city its ground, so the test drives
    // the step rather than reading the verb.
    let (mut field, city) = one_city(700);
    let place = ground_at_distance(&field, city, MINIMUM_FOUNDING_DISTANCE);
    let settler = settler_at(&mut field, place, FactionId(1));
    field.step(2).expect("the step must run");
    let before = field.holding_of(FactionId(1));
    assert_eq!(before, 0, "the faction holds no ground before it founds");

    assert!(field.settle_set(&[settler])[0].founded());
    field.step(2).expect("the step must run");

    assert!(
        field.holding_of(FactionId(1)) > 0,
        "the new city must hold the ground around it"
    );
    assert_eq!(
        field.tile_holder(place).and_then(Holder::faction),
        Some(FactionId(1)),
        "the city holds the tile it stands on"
    );
    assert!(field.check_invariants());
}

#[test]
fn a_new_settlement_moves_the_state_hash() {
    // The settlement arena enters the whole-world hash. A founding that no
    // hash covered would let two worlds that differ diverge later with no
    // test between them.
    let (mut field, city) = one_city(700);
    let place = ground_at_distance(&field, city, MINIMUM_FOUNDING_DISTANCE);
    let settler = settler_at(&mut field, place, FactionId(1));
    let before = field.state_hash();

    assert!(field.settle_set(&[settler])[0].founded());

    assert_ne!(
        field.state_hash(),
        before,
        "a founded city must move the state hash"
    );
}

#[test]
fn the_controller_founds_a_city_through_the_verb() {
    // **This is the test the item turns on.** A settle verb that nothing
    // calls changes no settlement count, and every test above would still
    // pass. The engine must reach the verb on its own.
    let (mut field, city) = one_city(700);
    let place = ground_at_distance(&field, city, MINIMUM_FOUNDING_DISTANCE);
    // The controller evaluates a faction that holds a seat, so the faction
    // founds one city of its own first, far from the settler's place.
    let seat = ground_at_distance(&field, place, MINIMUM_FOUNDING_DISTANCE * 2);
    field
        .found_group_at(seat, 1, FactionId(1))
        .expect("the ground admits a group");
    settler_at(&mut field, place, FactionId(1));
    let before = field.settlements().len();

    // The settle draw is one keyed draw for each tick, so a run of ticks is
    // what reaches the yes answer. No test here asserts on time.
    for _ in 0..64 {
        field.step(1).expect("the step must run");
        if field.settlements().len() > before {
            assert!(field.check_invariants());
            return;
        }
    }
    panic!("the controller founded no city in 64 ticks, so the option is inert");
}

#[test]
fn the_refusal_reader_names_the_rule_that_refused_each_settler() {
    // **The verb answers one byte, and this reader is the reason.** A caller
    // that reads a refusal and no reason cannot tell a settler on held
    // ground from a settler that stands too near a city, and those two
    // states ask for different actions.
    //
    // The fixture supplies both extremes at once, and one settler the verb
    // accepts. A fixture of one settler would leave two arms of the reader
    // unmeasured.
    let (mut field, city) = one_city(700);
    field.step(2).expect("the step must run");
    let held = ground_at_distance(&field, city, 1);
    let near = ground_at_distance(&field, city, MINIMUM_FOUNDING_DISTANCE - 1);
    let far = ground_at_distance(&field, city, MINIMUM_FOUNDING_DISTANCE);
    settler_at(&mut field, held, FactionId(1));
    settler_at(&mut field, near, FactionId(1));
    settler_at(&mut field, far, FactionId(1));

    let names = field
        .settle_refusal_names(FactionId(1))
        .expect("the world holds the faction");

    assert_eq!(
        names.len(),
        3,
        "the reader answers one name for each settler and not one for each refusal"
    );
    assert!(
        names.contains(&"ground_is_held"),
        "the reader names the held ground refusal, and it gave {names:?}"
    );
    assert!(
        names.contains(&"too_close_to_a_city"),
        "the reader names the distance refusal, and it gave {names:?}"
    );
    assert!(
        names.contains(&"accepted"),
        "the reader names the settler the verb would take, and it gave {names:?}"
    );
    assert_eq!(
        field.settler_count(FactionId(1)),
        Some(3),
        "the settler count and the refusal reader read one set of settlers"
    );
}

#[test]
fn the_refusal_reader_and_the_settler_count_refuse_a_faction_the_world_lacks() {
    let (field, _) = one_city(700);
    let outside = FactionId(9);
    assert_eq!(field.settle_refusal_names(outside), None);
    assert_eq!(field.settler_count(outside), None);
}

#[test]
fn a_faction_with_no_settler_reads_an_empty_refusal_list() {
    // **This is the state the reader exists to separate.** A faction that
    // holds no settler and a faction whose settler stands on held ground both
    // read a refused settle verb, and only the reader tells them apart.
    let (field, _) = one_city(700);
    assert_eq!(field.settle_refusal_names(FactionId(1)), Some(Vec::new()));
    assert_eq!(field.settler_count(FactionId(1)), Some(0));
}
