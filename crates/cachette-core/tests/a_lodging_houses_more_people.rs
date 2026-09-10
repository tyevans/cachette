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
//! Each test states its own housing, its own birth chance and its own
//! lodging work. The default values are placeholders that the balance
//! harness will change, and a test that read them would measure the
//! register.[^2]
//!
//! **The sky of a world decides a long run, so the fixture keeps its runs
//! short and states what the sky must not do.** Every tile of a world passes
//! under rain and under storms. Rain and a storm wear a level that stands, a
//! builder pays the repair before it advances the level, and a storm ends a
//! unit that stands under it. A fixture that waited for a long build would
//! measure the weather, and a fixture that ordered a builder a storm had
//! ended would step a world that built nothing. Each loop here therefore
//! writes the state it asks about on every tick, and states that the builder
//! is alive on every tick.[^1]
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
    BuildRefusal, UpgradeCategory, UpgradeRow, BUILD_RATE, LODGING_FIT, UPGRADE_LEVEL_COUNT,
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

/// The work that each level of a lodging asks for in these tests.
///
/// **The fixture states the work, because a long build measures the sky.** A
/// level that stands wears under the rain and the storms of a world, and a
/// builder pays the repair before it advances the level. The count of ticks a
/// long build takes is therefore a property of the weather, and a test that
/// waited for it would measure the weather.
///
/// The value is a small multiple of the work one builder adds in a tick, so a
/// build still takes several ticks and still holds state between them, and it
/// is short enough that the wear of the window costs a builder nothing. The
/// fixture asserts both properties rather than restating them.
const LODGING_WORK: u32 = (BUILD_RATE as u32) * 8;

/// The margin on the ticks a build is given, above the work it asks for.
///
/// **This bounds a loop that leaves early. It is not a count of ticks that a
/// test takes.** A build of the stated work finishes in the ticks the work
/// asks for and takes none of this margin. The margin covers the ticks the
/// fixture spends settling and the odd tick a builder spends repairing what a
/// storm took.
const BUILD_MARGIN: u64 = 4;

/// How many ticks a window that must stay closed runs for.
///
/// **This is a count of applications of the growth stage.** The growth
/// schedule of the fixture applies the stage on every tick, so a site with no
/// free place refuses a birth this many times before a test reads it.
const CLOSED_WINDOW: u64 = 128;

/// How many ticks the solver is given to reach a category.
///
/// **This bounds a loop that leaves early in the second half of the solver
/// test, and it is the whole window in the first half.** The first half
/// asserts that the solver zones no lodging, so it runs the window out.
const SOLVER_WINDOW: u64 = 128;

/// How many ticks a growth is given before a test gives up.
///
/// **This bounds a loop that leaves early. It is not a count of ticks that a
/// test takes.** A site with a free place and a store that pays grows on the
/// first application of the growth stage, so a test that waits this long has
/// found a defect and not a slow world.
const GROWTH_PATIENCE: u64 = 1024;

/// How many births the store of a fixture pays for when a test fills it.
///
/// A site pays a share of its store to hold it, so a store fills above the
/// cost of the births one application makes. The margin is far above that
/// share and far below the ceiling of the fixed-point scale.
const STORE_MARGIN: i32 = 64;

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
        ..WorldConfig::DEFAULT
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
    state_the_lodging_work(&mut world);
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

/// Writes the work of every lodging level, and leaves every other column of
/// the row as the default table states it.
///
/// The housing column is what the raise tests read, so the fixture must not
/// touch it. Only the work moves, and only because the length of a build is
/// the one thing about a lodging that the weather decides.
///
/// **A build that finished in one tick would hold no state between two
/// ticks.** The engine asks for a work above the work one builder adds, and
/// this function asserts that the work the fixture states still meets that.
fn state_the_lodging_work(world: &mut World) {
    for level in 1..=UPGRADE_LEVEL_COUNT as u8 {
        let row = world
            .upgrade_table()
            .row(UpgradeCategory::LODGING, level)
            .expect("the default table holds every lodging level");
        world
            .define_upgrade_row(
                UpgradeCategory::LODGING.to_u8(),
                level,
                UpgradeRow {
                    work: LODGING_WORK,
                    ..row
                },
            )
            .expect("the category and the level are in the table");
    }
    assert!(
        i64::from(LODGING_WORK) > BUILD_RATE,
        "the stated work must take a builder more than one tick"
    );
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

/// Returns the ticks a build of one level is given before a test gives up.
///
/// The budget is the work that every level up to the target asks for, divided
/// by the work one builder adds in a tick, and multiplied by a margin. The
/// fixture reads the work from the table rather than restating it, so a
/// change to the work moves the budget with it.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
fn build_patience(world: &World, level: u8) -> u64 {
    let table = world.upgrade_table();
    let asked: i64 = (1..=level)
        .map(|step| table.work_at(UpgradeCategory::LODGING, step))
        .sum();
    let ticks = asked / BUILD_RATE;
    assert!(ticks > 0, "the stated work must ask a builder for a tick");
    (ticks as u64) * BUILD_MARGIN
}

/// Steps until the level on a tile reaches one value, and returns the ticks.
///
/// **The fixture states that the builder is alive.** A storm ends a unit, and
/// a fixture that ordered a builder the world had ended would step a world
/// that built nothing. It would then report a slow world, and the cause would
/// be a dead builder.
///
/// **The order goes again on every tick.** The choice pass moves an idle
/// unit, and a builder that wandered off would make a test measure the walk
/// rather than the raise. A caller may order a build on every tick, so the
/// fixture does.
fn step_until_level(world: &mut World, unit: Entity, address: Axial, level: u8) -> u64 {
    let budget = build_patience(world, level);
    for taken in 1..=budget {
        assert!(
            world.build_order(unit).is_some(),
            "the world ended the builder at tick {taken}, so the weather decided this test"
        );
        if !stands_on(world, unit, address) {
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
    panic!("the level {level} did not stand after {budget} ticks");
}

/// Reports whether one builder stands on one tile.
fn stands_on(world: &World, builder: Entity, address: Axial) -> bool {
    world
        .soldiers_on(address)
        .map(|units| units.contains(&builder))
        .unwrap_or(false)
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
    // **A row that housed nobody would pass the comparison below.** The test
    // reads the row rather than a number of its own, so it states here that
    // the row houses somebody.
    assert!(
        first.housing_change > 0,
        "the first level of a lodging houses nobody"
    );
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
/// **The fixture states the extreme and holds it on every tick.** The housing
/// is written to the residents the engine counts, so the site has no free
/// place at any tick of the closed window. A storm ends a resident, and a
/// housing written once would then stand above the residents and would open a
/// free place the test did not ask for.[^1]
///
/// **The store is refilled on every tick, and the test asserts that it
/// pays.** A site pays a share of its store to hold it, so a store written
/// once empties over a long window. A fixture that filled the store once
/// would end with an empty store, and the site would grow nobody because it
/// could not pay. The test would then read as a housing bound and would
/// measure the store.[^2]
///
/// # References
///
/// [^1]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D2. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
/// [^2]: Testing Rules, section 2a. `.agents/rules/testing.md`
#[test]
fn a_site_at_its_housing_grows_again_once_a_lodging_finishes() {
    let Ground {
        mut world,
        site,
        beside,
        ..
    } = ground();
    world.set_birth_chance(Fix32::ONE);
    fill_the_store(&mut world, site);
    close_the_site(&mut world, site);
    world.step(1).expect("the step must run");
    close_the_site(&mut world, site);
    assert_eq!(
        world.site_free_places(site),
        Some(0),
        "the fixture must reach a site with no free place"
    );

    // A site with no free place grows nobody, however long the run is and
    // however much the store holds.
    let stopped = world.site_residents(site).expect("the site is live");
    for _ in 0..CLOSED_WINDOW {
        fill_the_store(&mut world, site);
        close_the_site(&mut world, site);
        world.step(1).expect("the step must run");
        assert!(
            world.site_residents(site).expect("the site is live") <= stopped,
            "a site with no free place grew somebody"
        );
    }
    assert!(
        pays_for_a_birth(&world, site),
        "the fixture must still pay for a birth, or the store refused and not the housing"
    );

    // The builder is a resident of nowhere, so it never fills the site it
    // builds beside.
    let closed = world.site_housing(site).expect("the site is live");
    let held = world.site_residents(site).expect("the site is live");
    let unit = order_a_lodging(&mut world, beside);
    step_until_level(&mut world, unit, beside, 1);
    assert!(
        world.site_housing(site).expect("the site is live") > closed,
        "a finished lodging did not raise the housing of the site beside it"
    );
    // **The test drives the engine until the site grows, and not for a fixed
    // count of ticks.** A count taken from a work value or from a birth
    // chance would fail the next time somebody changes one of them, and the
    // test would then measure the balance register.
    //
    // The site may have grown already. Growth runs in the tick that finished
    // the level, so the loop reads the state before it steps again.
    let grew = world.site_residents(site).expect("the site is live") > held
        || step_until(&mut world, site, |world| {
            world.site_residents(site).expect("the site is live") > held
        });
    assert!(
        grew,
        "a site whose housing rose did not grow again inside {GROWTH_PATIENCE} ticks"
    );
}

/// Writes the housing that leaves a site with no free place.
///
/// The fixture writes it on every tick of a window that must stay closed, in
/// the way it refills the store on every tick. A storm ends a resident, and a
/// housing written once would stand above the residents the engine counts
/// from that moment on.
fn close_the_site(world: &mut World, site: Entity) {
    let residents = world.site_residents(site).expect("the site is live");
    assert!(
        world.set_site_housing(site, residents),
        "the site must take the housing that fills it"
    );
}

/// Writes the housing that leaves a site with a stated count of free places.
fn open_the_site(world: &mut World, site: Entity, places: u32) {
    let residents = world.site_residents(site).expect("the site is live");
    assert!(
        world.set_site_housing(site, residents + places),
        "the site must take a housing above its residents"
    );
}

/// Writes a store that pays for every birth the housing of a site admits.
///
/// The quantity is the proposals one application makes, times the cost of
/// one birth, times a margin for the share a site pays to hold its store.
/// The test reads the cost from the world and states no quantity of its
/// own.
fn fill_the_store(world: &mut World, site: Entity) {
    let cost = world.food_per_birth()[0];
    let full = Fix32(cost.0.saturating_mul(STORE_MARGIN));
    world
        .set_settlement_store(site, GOOD, full)
        .expect("the good is in the set");
}

/// Reports whether the store of a site pays for one birth.
fn pays_for_a_birth(world: &World, site: Entity) -> bool {
    let cost = world.food_per_birth()[0];
    world
        .settlements()
        .store(site)
        .and_then(|held| held.quantity(GOOD))
        .is_some_and(|held| held.0 >= cost.0)
}

/// Steps the world until a state arrives, and reports whether it arrived.
///
/// The store of the site is refilled on every tick, so the store never
/// decides what the caller reads.
fn step_until(world: &mut World, site: Entity, reached: impl Fn(&World) -> bool) -> bool {
    for _ in 0..GROWTH_PATIENCE {
        fill_the_store(world, site);
        world.step(1).expect("the step must run");
        if reached(world) {
            return true;
        }
    }
    false
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
/// **The fixture writes the housing on every tick.** A storm ends a resident,
/// so a housing written once stops describing the state the test asks the
/// solver about.[^3]
///
/// # References
///
/// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D2. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
/// [^2]: Recurring Defect Shapes, shape 3. `.agents/rules/recurring-defects.md`
/// [^3]: Testing Rules, section 2a. `.agents/rules/testing.md`
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
    for _ in 0..SOLVER_WINDOW {
        open_the_site(&mut world, site, 1);
        world.step(1).expect("the step must run");
    }
    open_the_site(&mut world, site, 1);
    assert!(
        world.site_free_places(site).unwrap_or(0) > 0,
        "the fixture must reach a site with a free place"
    );
    assert!(
        !plans_a_lodging(&world),
        "a faction with a free place zoned a lodging"
    );

    // The same site with no free place. Now the solver zones a lodging.
    for _ in 0..SOLVER_WINDOW {
        close_the_site(&mut world, site);
        world.step(1).expect("the step must run");
        if plans_a_lodging(&world) {
            break;
        }
    }
    close_the_site(&mut world, site);
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
