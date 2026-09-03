//! A unit that stands on the tile of its home site gives its load to the
//! store of that site.
//!
//! **The resource loop had no sink.** A unit gathered into a carry column and
//! no verb moved that load into a store, so what a settlement held depended
//! only on the rate the founding set from the survey. The ground the units
//! stood on did not change it, and gathering could not feed anybody.[^1]
//!
//! Every test here drives the step. None calls the delivery pass.[^2]
//!
//! **The fixture is built for these tests.** It does not copy the world of the
//! demonstration binary, because that world is chosen to look right and not to
//! produce an extreme.[^3] It holds a site standing on ground that carries
//! food, a site whose store is one unit below its ceiling, a unit at home and
//! a unit away from it.
//!
//! # References
//!
//! [^1]: ADR-0062, production and upkeep are rates attached to a site, decision D2. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
//! [^2]: Testing rules, section 5. `.claude/rules/testing.md`
//! [^3]: Testing rules, section 2a. `.claude/rules/testing.md`

use cachette_core::cohort::NeedRule;
use cachette_core::resource::ResourceKind;
use cachette_core::{Axial, CommodityId, Entity, FactionId, Fix32, World, WorldConfig};

/// The extent of every fixture world.
const EXTENT: u32 = 256;

/// The seed of every fixture world.
///
/// Each test asserts the property of the ground that it depends on, so a
/// change to the generator fails the fixture rather than the assertion.
const SEED: u64 = 7;

/// The commodity that food fills. It is read from the declared map, so this
/// file holds no second copy of the number.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-191. `docs/FINDINGS.md`
const FOOD: CommodityId = cachette_core::WORK_COMMODITY[0];

/// The thread counts that the equivalence test runs at.
const THREAD_COUNTS: [usize; 3] = [1, 2, 12];

/// The frames a test drives before it reads the result.
const FRAMES: u64 = 6;

/// Builds a world that neither feeds nor kills anybody.
///
/// The consumption pass draws from the same store that a delivery fills, so a
/// fixture that let it run would measure the two against each other. The rule
/// below asks for nothing and ends nobody, so the store holds what the
/// delivery put in it.
fn world() -> World {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: SEED,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    let rule = NeedRule::new(
        Fix32::ZERO,
        Fix32::ZERO,
        Fix32::ZERO,
        Fix32::ZERO,
        Fix32::MAX,
    )
    .expect("every rate is at or above zero");
    world.set_need_rule(rule);
    world
}

/// Returns the first address whose ground admits a unit and carries food.
///
/// The scan runs in index order and takes the first address that fits, so the
/// answer is fixed and does not depend on how a caller walked the world.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
fn a_tile_that_carries_food(world: &World) -> Axial {
    let grid = world.grid();
    for index in 0..grid.tile_count() {
        let address = Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32);
        if !world.admits_a_unit(address) {
            continue;
        }
        if world
            .tile_stock(address, ResourceKind::Food)
            .is_some_and(|stock| stock.0 > 0)
        {
            return address;
        }
    }
    panic!("the fixture holds no open tile that carries food");
}

/// Returns the food that one site holds.
fn store_of(world: &World, site: Entity) -> Fix32 {
    world
        .settlements()
        .store(site)
        .expect("the site is live")
        .quantity(FOOD)
        .expect("the commodity is inside the set")
}

/// Returns what one unit carries of food.
fn carry_of(world: &World, unit: Entity) -> u32 {
    world
        .soldiers()
        .carry(unit)
        .expect("the unit is live")
        .of(ResourceKind::Food)
        .0
}

/// Seats a site on a food tile and a unit on that same tile, homed to it.
///
/// The site stands where the food is, so the unit gathers and stands at home
/// on one tile. A fixture that put them apart would test the movement as well
/// as the delivery.
fn site_and_unit_on_one_tile(world: &mut World) -> (Entity, Entity, Axial) {
    let place = a_tile_that_carries_food(world);
    let site = world
        .found_settlement(place, FactionId(0))
        .expect("the ground admits a settlement");
    let unit = world
        .spawn_soldier(place, FactionId(0))
        .expect("the ground admits the unit");
    assert!(
        world.set_home_site(unit, Some(site)),
        "the unit takes a home"
    );
    assert!(
        world.order_gather(unit, ResourceKind::Food),
        "the order lands"
    );
    (site, unit, place)
}

/// Fills the carry of a unit without letting it deliver.
///
/// **The gather resolve runs before the delivery in one frame**, so a unit
/// that stands at home ends every frame with an empty carry. A test that
/// needs a load in hand therefore takes the home away while it gathers, and
/// gives it back afterwards.
fn load_without_delivering(world: &mut World, unit: Entity, site: Entity, frames: u64) -> u32 {
    assert!(
        world.set_home_site(unit, None),
        "the unit gives up its home"
    );
    for _ in 0..frames {
        world.order_gather(unit, ResourceKind::Food);
        world.step(1).expect("the step must run");
    }
    assert!(
        world.set_home_site(unit, Some(site)),
        "the unit takes its home back"
    );
    let carried = carry_of(world, unit);
    assert!(carried > 0, "the fixture put no load on the unit");
    carried
}

#[test]
fn a_unit_at_home_gives_its_load_to_the_store() {
    let mut world = world();
    let (site, unit, _) = site_and_unit_on_one_tile(&mut world);
    let before = store_of(&world, site);

    for _ in 0..FRAMES {
        world.step(1).expect("the step must run");
        world.order_gather(unit, ResourceKind::Food);
    }

    let after = store_of(&world, site);
    assert!(
        after > before,
        "the store did not rise: {before:?} to {after:?}"
    );
    assert!(
        world.delivered_carry()[ResourceKind::Food.index()] > 0,
        "the world recorded no delivery"
    );
}

#[test]
fn the_store_rises_by_exactly_what_the_carry_lost() {
    // This is the test the register asks for against every value the work
    // writes into state: change the value, and the reading must change by the
    // same amount.[^1]
    //
    // [^1]: Decisions register, DEC-074. `docs/DECISIONS.md`
    let mut world = world();
    let (site, unit, _) = site_and_unit_on_one_tile(&mut world);

    let carried = load_without_delivering(&mut world, unit, site, 1);
    let held = store_of(&world, site);

    // The unit takes no new order, so the only quantity that moves is the one
    // it already holds.
    world.step(1).expect("the step must run");

    let moved = i64::from(store_of(&world, site).0) - i64::from(held.0);
    let lost = i64::from(carried) - i64::from(carry_of(&world, unit));
    // Without this the assertion below reads zero against zero and holds for
    // a world in which nothing was delivered at all.[^2]
    //
    // [^2]: Testing rules, section 2a. `.claude/rules/testing.md`
    assert!(
        moved > 0,
        "the fixture delivered nothing, so the equality is empty"
    );
    assert_eq!(
        moved,
        lost * i64::from(Fix32::ONE.0),
        "the store rose by an amount the carry did not lose"
    );
}

#[test]
fn a_unit_away_from_its_home_delivers_nothing() {
    let mut world = world();
    let (site, unit, place) = site_and_unit_on_one_tile(&mut world);
    load_without_delivering(&mut world, unit, site, 1);

    // The unit steps off the tile of its site. The site keeps its store.
    let away = world
        .grid()
        .neighbour(place, 0)
        .filter(|target| world.admits_a_unit(*target))
        .expect("the fixture holds an open neighbour");
    assert!(
        world
            .place_soldier(unit, away)
            .expect("the ground admits it"),
        "the unit moved"
    );
    let held = store_of(&world, site);
    let carried = carry_of(&world, unit);

    world.step(1).expect("the step must run");

    assert_eq!(
        store_of(&world, site),
        held,
        "a unit away from home delivered"
    );
    assert_eq!(
        carry_of(&world, unit),
        carried,
        "a unit away from home lost a load"
    );
}

#[test]
fn a_unit_with_no_home_delivers_nothing() {
    let mut world = world();
    let (site, unit, _) = site_and_unit_on_one_tile(&mut world);
    load_without_delivering(&mut world, unit, site, 1);

    assert!(
        world.set_home_site(unit, None),
        "the unit gives up its home"
    );
    let held = store_of(&world, site);
    let carried = carry_of(&world, unit);

    world.step(1).expect("the step must run");

    assert_eq!(
        store_of(&world, site),
        held,
        "a unit with no home delivered"
    );
    assert_eq!(
        carry_of(&world, unit),
        carried,
        "a unit with no home lost a load"
    );
}

#[test]
fn a_full_store_takes_what_it_can_and_the_unit_keeps_the_rest() {
    let mut world = world();
    let (site, unit, _) = site_and_unit_on_one_tile(&mut world);
    let carried = load_without_delivering(&mut world, unit, site, 4);
    assert!(carried > 1, "the fixture put too small a load on the unit");

    // The store has room for one whole unit and no more.
    let ceiling = Fix32(Fix32::MAX.0 - Fix32::ONE.0);
    world
        .set_settlement_store(site, FOOD, ceiling)
        .expect("the commodity is inside the set");

    world.step(1).expect("the step must run");

    assert_eq!(
        carry_of(&world, unit),
        carried - 1,
        "the unit gave more or less than the room the store had"
    );
    assert_eq!(
        store_of(&world, site),
        Fix32(ceiling.0 + Fix32::ONE.0),
        "the store took more or less than its room"
    );
}

#[test]
fn every_account_balances_across_a_delivery() {
    let mut world = world();
    let (_, unit, _) = site_and_unit_on_one_tile(&mut world);
    for _ in 0..FRAMES {
        world.step(1).expect("the step must run");
        world.order_gather(unit, ResourceKind::Food);
        assert!(
            world.check_invariants(),
            "an account stopped balancing at tick {}",
            world.tick().0
        );
    }
    assert!(
        world.delivered_carry()[ResourceKind::Food.index()] > 0,
        "the run delivered nothing, so the assertion above saw no delivery"
    );
}

#[test]
fn a_delivery_gives_one_answer_at_every_thread_count() {
    let mut answers = Vec::new();
    for threads in THREAD_COUNTS {
        let mut world = world();
        let (site, unit, _) = site_and_unit_on_one_tile(&mut world);
        for _ in 0..FRAMES {
            world.step(threads).expect("the step must run");
            world.order_gather(unit, ResourceKind::Food);
        }
        assert!(
            world.delivered_carry()[ResourceKind::Food.index()] > 0,
            "the run at {threads} threads delivered nothing, so the comparison \
             below reads worlds in which the pass never ran"
        );
        answers.push((
            store_of(&world, site),
            carry_of(&world, unit),
            world.state_hash(),
        ));
    }
    assert!(
        answers.windows(2).all(|pair| pair[0] == pair[1]),
        "the delivery depends on the thread count: {answers:?}"
    );
}
