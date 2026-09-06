//! A lodging raises the housing of the site beside it, and the site grows
//! again.
//!
//! **The fixture is built for the extreme and is not a copy of the
//! demonstration world.** The extreme these tests need is a site whose
//! residents exactly fill its housing, because that is the only state in
//! which a raise changes anything. The demonstration world supplies a site
//! with room for most of a run, and a fixture that copied it would measure
//! the fixture.[^1]
//!
//! Each test states its own housing and its own birth chance. The default
//! values are placeholders that the balance harness will change, and a test
//! that read them would measure the register.[^2]
//!
//! # References
//!
//! [^1]: Testing Rules, section 2a. `.agents/rules/testing.md`
//! [^2]: Balance register, the population. `docs/reference/balance.md`

use cachette_core::cohort::NeedRule;
use cachette_core::holding::ReachRules;
use cachette_core::rates::RateSchedule;
use cachette_core::site::CommodityId;
use cachette_core::terrain::TileKind;
use cachette_core::upgrade::{
    BuildRefusal, UpgradeCategory, UpgradeRow, LODGING_FIT, LODGING_LEVEL_1_WORK,
};
use cachette_core::{Axial, Entity, FactionId, Fix32, World, WorldConfig};

/// The extent of every world under test.
const WIDTH: u32 = 96;
/// The extent of every world under test.
const HEIGHT: u32 = 96;

/// The seed that these tests read.
const SEED: u64 = 0x0cac_4e77_10d6;

/// How many people the founding seats.
///
/// The group is small, so a test states its own housing and reaches the
/// extreme it needs rather than the one the founding gives.
const GROUP: u32 = 2;

/// The interval between two choices, as a power of two.
///
/// The interval is far above the ticks any test here takes, so no builder
/// chooses to walk away while a test runs.
const CHOICE_EXPONENT: u32 = 16;

/// The faction that owns every site and every builder here.
const OWNER: FactionId = FactionId(0);

/// The commodity that every fixture uses.
const GOOD: CommodityId = CommodityId(0);

/// The store that one birth costs here.
const FOOD: Fix32 = Fix32::ONE;

/// The deficit at which a unit ends here. Nothing reaches it, because the
/// need rule takes nothing.
const BOUND: Fix32 = Fix32::from_int(4);

/// How many ticks a build is given before a test gives up.
///
/// The work of the first level divided by the one builder that adds to it,
/// with room for the ticks the fixture spends settling.
const PATIENCE: u64 = (LODGING_LEVEL_1_WORK as u64) * 4;

/// Returns every address of the extent, in row-major order.
fn addresses() -> Vec<Axial> {
    let mut all = Vec::with_capacity((WIDTH * HEIGHT) as usize);
    for r in 0..HEIGHT {
        for q in 0..WIDTH {
            all.push(Axial::new(q as i32, r as i32));
        }
    }
    all
}

/// A site of the owning faction, and the tile beside it that a lodging goes
/// on.
struct Ground {
    /// The world under test.
    world: World,
    /// The settlement whose housing the tests read.
    site: Entity,
    /// The tile the settlement stands on.
    seat: Axial,
    /// The tile beside the settlement that fits a lodging.
    beside: Axial,
}

/// Builds a world with one site, and one tile beside it that fits a lodging.
///
/// The reach covers the world, so the held ground rule never decides a test
/// here.[^1]
///
/// # References
///
/// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D4. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
fn ground() -> Ground {
    let mut world = World::new(WorldConfig {
        width: WIDTH,
        height: HEIGHT,
        seed: SEED,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    // **The choice pass almost never fires here.** An exponent of zero makes
    // every unit choose on every tick, and a builder that chose to walk away
    // would make a test measure the walk rather than the raise.
    world
        .set_choice_schedule(CHOICE_EXPONENT)
        .expect("the exponent is inside the range");
    let reach = WIDTH + HEIGHT;
    world.set_reach_rules(ReachRules::new(reach, 1, reach));
    world.set_growth_schedule(RateSchedule::new(1, 0).expect("one is inside the range"));
    world.set_food_per_birth([FOOD]);
    world.set_housing_per_person(1);
    // The need rule takes nothing, so a unit never eats the store that these
    // tests read.
    world.set_need_rule(
        NeedRule::new(Fix32::ZERO, Fix32::ZERO, Fix32::ZERO, Fix32::ZERO, BOUND)
            .expect("no rate is below zero"),
    );
    world
        .set_economy_schedule(1, 0)
        .expect("one is inside the range");
    // The queue must not spend the people that growth makes.
    assert!(world.set_queue_bound(0), "zero is inside the block");
    // **The faction takes no controller.** The built-in controller sends an
    // idle unit toward the project nearest to it, and a builder that walked
    // away would make a test measure the walk rather than the raise. The
    // solver test below turns the controller back on, because the plan is
    // what that test reads.
    assert!(
        world.set_externally_controlled(OWNER, true),
        "the faction is in the world"
    );
    let (seat, beside) = seat_with_a_neighbour(&world);
    // **The founding records the seat of the faction.** The solver plans
    // around the seat, and a faction with no seat receives no evaluation, so
    // a fixture that only placed a settlement would leave the plan empty.
    let site = world
        .found_group_at(seat, GROUP, OWNER)
        .expect("the ground admits the group")
        .settlement();
    world.step(1).expect("the step must run");
    Ground {
        world,
        site,
        seat,
        beside,
    }
}

/// Finds a tile that admits a city and has a neighbour a lodging fits.
fn seat_with_a_neighbour(world: &World) -> (Axial, Axial) {
    for seat in addresses() {
        if !world.admits_a_unit(seat) {
            continue;
        }
        for side in world.grid().neighbours(seat).into_iter().flatten() {
            if world.admits_a_unit(side) && fits_a_lodging(world, side) {
                return (seat, side);
            }
        }
    }
    panic!("the world holds no seat with a neighbour that fits a lodging");
}

/// Reports whether the lodging row of the table fits the ground of one tile.
///
/// The test asks the table and never a ground kind by name.
fn fits_a_lodging(world: &World, address: Axial) -> bool {
    world
        .tile_kind(address)
        .is_some_and(|kind| LODGING_FIT & (1u32 << kind.to_u8()) != 0)
}

/// Puts one builder on a tile and orders it to raise a lodging there.
fn order_a_lodging(world: &mut World, address: Axial) -> Entity {
    let unit = world
        .spawn_soldier(address, OWNER)
        .expect("the ground admits a unit");
    world
        .zone_project(OWNER, address, UpgradeCategory::LODGING)
        .expect("the faction holds the ground");
    world
        .order_build(unit, UpgradeCategory::LODGING)
        .expect("the ground fits a lodging");
    unit
}

/// Steps until the level on a tile reaches one value, and returns the ticks.
///
/// **The order goes again on every tick.** The choice pass moves an idle
/// unit, and a builder that wandered off would make a test measure the walk
/// rather than the raise. A caller may order a build on every tick, so the
/// fixture does.
fn step_until_level(world: &mut World, unit: Entity, address: Axial, level: u8) -> u64 {
    for taken in 1..=PATIENCE {
        let here = world
            .soldiers_on(address)
            .map(|units| units.contains(&unit))
            .unwrap_or(false);
        if !here {
            world
                .place_soldier(unit, address)
                .expect("the ground admits the builder");
        }
        let _ = world.order_build(unit, UpgradeCategory::LODGING);
        world.step(1).expect("the step must run");
        if world.upgrade_level(address) >= level {
            return taken;
        }
    }
    panic!("the level {level} did not stand after {PATIENCE} ticks");
}

// ---------------------------------------------------------------------------
// The raise
// ---------------------------------------------------------------------------

/// A finished lodging raises the housing of the site beside it, by the
/// amount its row states.
///
/// The test reads the amount from the row of the table rather than writing a
/// number of its own, so it measures the composition and not the value.[^1]
///
/// # References
///
/// [^1]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D1. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
#[test]
fn a_finished_lodging_raises_the_housing_of_the_site_beside_it() {
    let Ground {
        mut world,
        site,
        beside,
        ..
    } = ground();
    // The store holds nothing, so growth never moves the numbers this test
    // reads.
    world.set_birth_chance(Fix32::ZERO);
    let before = world.site_housing(site).expect("the site is live");
    let unit = order_a_lodging(&mut world, beside);
    step_until_level(&mut world, unit, beside, 1);
    let first = world
        .upgrade_table()
        .row(UpgradeCategory::LODGING, 1)
        .expect("the default table holds the first level");
    assert_eq!(
        world.site_housing(site),
        Some(before + first.housing_change),
        "a finished lodging did not raise the housing of the site beside it"
    );

    // A second level raises it again, by the amount its own row states.
    step_until_level(&mut world, unit, beside, 2);
    let second = world
        .upgrade_table()
        .row(UpgradeCategory::LODGING, 2)
        .expect("the default table holds the second level");
    assert_eq!(
        world.site_housing(site),
        Some(before + first.housing_change + second.housing_change),
        "the second level of a lodging did not raise the housing again"
    );
}

/// A lodging that stands beside no settlement raises nothing.
///
/// The raise reaches a site on its own tile or one of the six beside it, and
/// a lodging further away houses nobody.
#[test]
fn a_lodging_far_from_every_site_raises_no_housing() {
    let Ground {
        mut world,
        site,
        seat,
        ..
    } = ground();
    world.set_birth_chance(Fix32::ZERO);
    let far = addresses()
        .into_iter()
        .find(|address| {
            world.admits_a_unit(*address)
                && fits_a_lodging(&world, *address)
                && seat.distance(*address) > 1
        })
        .expect("the world holds ground away from the seat");
    let before = world.site_housing(site).expect("the site is live");
    let unit = order_a_lodging(&mut world, far);
    step_until_level(&mut world, unit, far, 1);
    assert_eq!(
        world.site_housing(site),
        Some(before),
        "a lodging away from every site raised the housing of one"
    );
}

// ---------------------------------------------------------------------------
// The site grows again
// ---------------------------------------------------------------------------

/// A site whose residents fill its housing grows nobody, and it grows again
/// once a lodging finishes.
///
/// **The fixture states the extreme and asserts that it reached it.** The
/// housing is written to exactly the resident count, so the site has no free
/// place at the moment the test begins.[^1]
///
/// # References
///
/// [^1]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D2. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
#[test]
fn a_site_at_its_housing_grows_again_once_a_lodging_finishes() {
    let Ground {
        mut world,
        site,
        beside,
        ..
    } = ground();
    // The store pays for many births, so the store never decides this test.
    world
        .set_settlement_store(site, GOOD, Fix32::from_int(1000))
        .expect("the good is in the set");
    world.set_birth_chance(Fix32::ONE);
    let residents = world.site_residents(site).expect("the site is live");
    assert!(
        world.set_site_housing(site, residents),
        "the site must take the housing that fills it"
    );
    world.step(1).expect("the step must run");
    assert_eq!(
        world.site_free_places(site),
        Some(0),
        "the fixture must reach a site with no free place"
    );

    let stopped = world.site_residents(site).expect("the site is live");
    for _ in 0..PATIENCE {
        world.step(1).expect("the step must run");
    }
    assert_eq!(
        world.site_residents(site),
        Some(stopped),
        "a site with no free place grew somebody"
    );

    // The builder is a resident of nowhere, so it never fills the site it
    // builds beside.
    let unit = order_a_lodging(&mut world, beside);
    step_until_level(&mut world, unit, beside, 1);
    assert!(
        world.site_free_places(site).unwrap_or(0) > 0,
        "a finished lodging left the site with no free place"
    );
    for _ in 0..PATIENCE {
        world.step(1).expect("the step must run");
    }
    assert!(
        world.site_residents(site).expect("the site is live") > stopped,
        "a site whose housing rose did not grow again"
    );
}

// ---------------------------------------------------------------------------
// The state hash
// ---------------------------------------------------------------------------

/// The housing column of a lodging row enters the state hash.
///
/// The table decides what a later frame does, so two worlds built with
/// different tables never hash the same.[^1]
///
/// # References
///
/// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
#[test]
fn the_housing_of_a_lodging_row_enters_the_state_hash() {
    let Ground { mut world, .. } = ground();
    let before = world.state_hash();
    let row = world
        .upgrade_table()
        .row(UpgradeCategory::LODGING, 1)
        .expect("the default table holds the first level");
    world
        .define_upgrade_row(
            UpgradeCategory::LODGING.to_u8(),
            1,
            UpgradeRow {
                housing_change: row.housing_change + 1,
                ..row
            },
        )
        .expect("the category and the level are in the table");
    assert_ne!(
        before,
        world.state_hash(),
        "the housing column of a row is outside the state hash"
    );
}

/// The level of a lodging that stands enters the state hash.
///
/// A level the hash did not cover would let two worlds that hold different
/// lodgings hash the same and then diverge on the next tick.
#[test]
fn the_level_of_a_lodging_enters_the_state_hash() {
    let Ground {
        mut world, beside, ..
    } = ground();
    world.set_birth_chance(Fix32::ZERO);
    let unit = order_a_lodging(&mut world, beside);
    step_until_level(&mut world, unit, beside, 1);
    let first = world.state_hash();
    step_until_level(&mut world, unit, beside, 2);
    assert_ne!(
        first,
        world.state_hash(),
        "the level of a lodging is outside the state hash"
    );
}

// ---------------------------------------------------------------------------
// The ground fit
// ---------------------------------------------------------------------------

/// A lodging on ground the row does not fit is refused, and the refusal
/// names the category and the ground.
#[test]
fn a_lodging_on_ground_that_does_not_fit_is_refused() {
    let Ground { mut world, .. } = ground();
    let high = addresses()
        .into_iter()
        .find(|address| {
            world.tile_kind(*address) == Some(TileKind::Mountain) && world.admits_a_unit(*address)
        })
        .expect("the world holds high ground that admits a unit");
    assert!(
        !fits_a_lodging(&world, high),
        "the fixture must reach ground that a lodging does not fit"
    );
    let unit = world
        .spawn_soldier(high, OWNER)
        .expect("the ground admits a unit");
    let _ = world.zone_project(OWNER, high, UpgradeCategory::LODGING);
    assert_eq!(
        world.order_build(unit, UpgradeCategory::LODGING),
        Err(BuildRefusal::GroundDoesNotFit {
            category: UpgradeCategory::LODGING,
            ground: TileKind::Mountain,
        }),
        "a lodging on ground it does not fit was not refused"
    );
}

// ---------------------------------------------------------------------------
// The solver reaches the category
// ---------------------------------------------------------------------------

/// A faction whose every site is full comes to zone a lodging, and one whose
/// site has room does not.
///
/// **A category that nothing zones is inert.** The controller builds only
/// what the plan zones, so this test drives the step and reads the plan
/// rather than constructing a project of its own.[^1] [^2]
///
/// # References
///
/// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D2. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
/// [^2]: Recurring Defect Shapes, shape 3. `.agents/rules/recurring-defects.md`
#[test]
fn a_faction_with_no_free_place_comes_to_zone_a_lodging() {
    let Ground {
        mut world, site, ..
    } = ground();
    assert!(
        world.set_externally_controlled(OWNER, false),
        "the faction is in the world"
    );
    world.set_birth_chance(Fix32::ZERO);
    // A site with room. The solver has other work, so it zones no lodging.
    let residents = world.site_residents(site).expect("the site is live");
    assert!(
        world.set_site_housing(site, residents + 1),
        "the site must take a housing above its residents"
    );
    for _ in 0..PATIENCE {
        world.step(1).expect("the step must run");
    }
    assert!(
        world.site_free_places(site).unwrap_or(0) > 0,
        "the fixture must reach a site with a free place"
    );
    assert!(
        !plans_a_lodging(&world),
        "a faction with a free place zoned a lodging"
    );

    // The same site with no free place. Now the solver zones a lodging.
    assert!(
        world.set_site_housing(site, residents),
        "the site must take the housing that fills it"
    );
    for _ in 0..PATIENCE {
        world.step(1).expect("the step must run");
        if plans_a_lodging(&world) {
            break;
        }
    }
    assert_eq!(
        world.site_free_places(site),
        Some(0),
        "the fixture must reach a site with no free place"
    );
    assert!(
        plans_a_lodging(&world),
        "a faction with no free place zoned no lodging"
    );
}

/// Reports whether the plan of the owning faction zones a lodging.
fn plans_a_lodging(world: &World) -> bool {
    world
        .plan_of(OWNER)
        .iter()
        .any(|project| project.category == UpgradeCategory::LODGING)
}
