//! A site grows people from its store, and its housing bounds how many.
//!
//! **The first test drives the demonstration seeding and not the mechanism.**
//! A faction founds with a very small group, and a campaign raise asks for
//! more people than a faction founds with. Growth is the first link of that
//! chain, so a test that only drove the growth kernel would prove the
//! arithmetic and not the chain.[^1]
//!
//! **Each other fixture is built for the extreme it needs**, and each asserts
//! that it reached that extreme. The demonstration world supplies none of the
//! three extremes that growth lives at: a site at its housing, a site with an
//! empty store, and a site with one free place and more than one proposal in
//! one frame.[^2]
//!
//! Each fixture writes its own housing, food cost and chance. The default
//! values are placeholders that the balance harness will change, and a test
//! that read them would measure the register.[^3]
//!
//! # References
//!
//! [^1]: Testing rules, drive the real caller. `.agents/rules/testing.md`
//! [^2]: Testing rules, section 2a. `.agents/rules/testing.md`
//! [^3]: Balance register, the population. `docs/reference/balance.md`

use cachette_core::cohort::NeedRule;
use cachette_core::growth;
use cachette_core::rates::RateSchedule;
use cachette_core::sim_math;
use cachette_core::site::CommodityId;
use cachette_core::unit_type::WORKER;
use cachette_core::{
    Axial, Entity, FactionId, Fix32, World, WorldConfig, FOUNDING_GROUP_DEFAULT, SUBSYSTEM_CENSUS,
};

/// Returns one census count by name.
fn census(world: &World, name: &str) -> i64 {
    SUBSYSTEM_CENSUS
        .iter()
        .find(|row| row.name == name)
        .map(|row| (row.read)(world))
        .expect("the census holds the row")
}

/// The commodity that every fixture uses.
const GOOD: CommodityId = CommodityId(0);

/// The faction that owns every site in the built fixtures.
const OWNER: FactionId = FactionId(0);

/// The store that one birth costs in the built fixtures.
const FOOD: Fix32 = Fix32::ONE;

/// The deficit at which a unit ends in the built fixtures.
///
/// Nothing here reaches it, because the fixture rule takes nothing.
const BOUND: Fix32 = Fix32::from_int(4);

/// A chance that makes every proposal a birth.
///
/// The built fixtures state a certain chance, so an assertion about the
/// housing measures the housing and not the draw. One test states a chance
/// below one, and it is the test of the draw.
const CERTAIN: Fix32 = Fix32::ONE;

/// A world that holds ground on every tile the built fixtures need.
const CONFIG: WorldConfig = WorldConfig {
    // The coarsest lattice of the terrain generator spans sixty-four tiles.
    // A world narrower than that sits inside one lattice cell, so every tile
    // of it holds one kind of ground and a fixture then measures the
    // generator.[^4]
    //
    // [^4]: Findings register, FND-054. `docs/FINDINGS.md`
    width: 96,
    height: 96,
    seed: 0x0cac_4e77_0060,
    faction_count: 2,
    unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
};

/// Returns the first address of the world whose ground carries a unit.
///
/// The terrain is generated from the seed, so a fixed address is water at
/// some seeds. The fixture asks the world for ground rather than naming one.
fn open_ground(world: &World) -> Axial {
    for r in 0..world.grid().height() {
        for q in 0..world.grid().width() {
            let address = Axial::new(q as i32, r as i32);
            if world
                .tile_kind(address)
                .is_some_and(cachette_core::terrain::TileKind::is_passable)
            {
                return address;
            }
        }
    }
    panic!("the world holds no passable ground");
}

/// Builds a world that grows on every tick, and founds one site.
///
/// The caller states the store, the housing and the residents, so each
/// fixture states its own extreme.
fn one_site(stock: Fix32, housing: u32, residents: u32) -> (World, Entity) {
    let mut world = World::new(CONFIG).expect("the extent must describe a world");
    let place = open_ground(&world);
    world.set_growth_schedule(RateSchedule::new(1, 0).expect("one is inside the range"));
    world.set_food_per_birth([FOOD]);
    world.set_housing_per_person(1);
    // **The fixture states a need rule that takes nothing.** These tests
    // measure growth, and a unit that eats would take the store that growth
    // reads and would starve while the test ran. A fixture that let both act
    // would measure the two together.
    world.set_need_rule(
        NeedRule::new(Fix32::ZERO, Fix32::ZERO, Fix32::ZERO, Fix32::ZERO, BOUND)
            .expect("no rate is below zero"),
    );
    // The economy applies on every tick, so the cohort table is derived again
    // on every tick. The table is the derived resident count that growth
    // reads, and a caller that spawned a unit and did not step reads it
    // stale.
    world
        .set_economy_schedule(1, 0)
        .expect("one is inside the range");
    // The queue must not spend the people that growth makes, because these
    // fixtures measure growth alone.
    assert!(world.set_queue_bound(0), "zero is inside the block");
    let site = world
        .found_settlement(place, OWNER)
        .expect("the tile carries a settlement");
    assert!(world.set_site_housing(site, housing), "the site is live");
    world
        .set_settlement_store(site, GOOD, stock)
        .expect("the good is in the set");
    for _ in 0..residents {
        let unit = world
            .spawn_soldier(place, OWNER)
            .expect("the ground admits a unit");
        assert!(world.set_home_site(unit, Some(site)), "the site is live");
    }
    // One step with no chance of a birth settles the derived resident count
    // over the units the fixture spawned. The growth chance goes on after it,
    // so nothing grew before the test began.
    world.set_birth_chance(Fix32::ZERO);
    world.step(1).expect("the step must run");
    world.set_birth_chance(CERTAIN);
    assert!(
        world.cohorts_describe_the_units(),
        "the fixture must settle the derived resident count"
    );
    (world, site)
}

/// Returns the store that the grow stage of the next tick will read.
///
/// **The store column is not that value.** The rate pass runs before the grow
/// stage in the same tick, and it settles the store towards what the site
/// earns: it adds the production the world derives and takes the upkeep the
/// world derives.[^6] A site that holds the cost of one birth in its column
/// therefore holds a little less than that cost when the grow stage counts
/// what it can afford.
///
/// The two rates come from the readers the pass itself uses, so this states no
/// second copy of either derivation.[^7] The schedule of these fixtures
/// applies the rates on every tick with a period of one, so one rate is one
/// application.
///
/// # References
///
/// [^6]: ADR-0062, production and upkeep are rates attached to a site, decision D1. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
/// [^7]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
fn store_at_the_grow_stage(world: &World, site: Entity) -> Fix32 {
    let held = world
        .settlements()
        .store(site)
        .expect("the site is live")
        .quantity(GOOD)
        .expect("the good is in the set");
    let earned = world
        .effective_production_rate(site, GOOD)
        .expect("the site is live");
    let owed = world
        .effective_upkeep_rate(site, GOOD)
        .expect("the site is live");
    sim_math::sub(sim_math::add(held, earned), owed)
}

/// Builds a site whose store affords exactly one birth on the next tick.
///
/// **The fixture asserts that it reached that extreme.** The store is not the
/// cost of one birth, because the rate pass takes a share of what the store
/// holds before the grow stage reads it, and it does so once while this
/// function settles the resident count and once more on the tick the test
/// runs. A fixture that wrote the cost of one birth into the column would
/// hand the grow stage less than one birth, and the test would then measure
/// the fixture rather than the stage.[^8]
///
/// # References
///
/// [^8]: Testing rules, section 2a. `.agents/rules/testing.md`
fn one_birth_site(housing: u32, residents: u32) -> (World, Entity) {
    // The share that the rate pass leaves behind. Two applications stand
    // between the store this writes and the grow stage that reads it, so the
    // store starts above the cost of one birth by that share twice over.
    let kept = sim_math::sub(Fix32::ONE, cachette_core::effective::HOLDING_SHARE);
    let once = sim_math::div(FOOD, kept).expect("the share left behind is above zero");
    let stock = sim_math::div(once, kept).expect("the share left behind is above zero");
    let (world, site) = one_site(stock, housing, residents);
    let reaches = store_at_the_grow_stage(&world, site);
    assert_eq!(
        growth::proposals(&[reaches], &[FOOD]),
        1,
        "the fixture must afford exactly one birth when the grow stage reads it"
    );
    (world, site)
}

/// Steps the world and asserts that the invariants hold at each frame.
fn run(world: &mut World, frames: u64, threads: usize) {
    for _ in 0..frames {
        world.step(threads).expect("the step must run");
        assert!(world.check_invariants(), "the world lost an invariant");
    }
}

/// Returns the residents of a site by a full pass over the home column.
///
/// **This is not the reader under test.** A test that read a value back
/// through the function that produced it would stay green under that
/// function's own defect, so the assertions compare the reader against this
/// pass.
fn residents_by_a_full_pass(world: &World, site: Entity) -> u32 {
    let slot = world.settlements().slot_of(site);
    world
        .soldiers()
        .iter()
        .filter(|unit| world.soldiers().home(*unit) == Some(slot))
        .count() as u32
}

// ---------------------------------------------------------------------------
// The chain end to end: a faction the demonstration seeded reaches a cohort
// ---------------------------------------------------------------------------

/// The extent of the demonstration world that the run below seeds.
///
/// The world is wide enough to seat four foundings at the minimum distance,
/// and small enough that a run of many ticks stays inside a test suite. The
/// seeding is the seeding the demonstration runs, and no verb of this file
/// founds anything.
const RUN_EXTENT: u32 = 96;

/// The factions the run below seeds.
const RUN_FACTIONS: u16 = 4;

/// The ticks each run below is given.
///
/// The bound is half of the game horizon that the balance register holds on
/// the tick limit row, so that the rest of the loop still fits inside one
/// game: queue a settler, found a second city, queue soldiers, raise a cohort
/// and reach a decision.[^1]
///
/// # References
///
/// [^1]: Balance register, the game end, the tick limit row. `docs/reference/balance.md`
const RUN_TICKS: u32 = 2500;

/// The population a faction must reach.
///
/// The bar is the campaign cohort size, because a faction that cannot reach
/// it can raise no campaign, and domination can then never fire. The value is
/// the one the world is built with, and the run reads it rather than stating
/// a number of its own.[^1]
///
/// # References
///
/// [^1]: Balance register, the controller, the campaign cohort size row. `docs/reference/balance.md`
fn population_bar(world: &World) -> u32 {
    world.campaign_cohort_size()
}

/// The seeds that the run below takes.
///
/// **Growth is seed dependent, so one seed measures one world.** The list is
/// the list the road run takes, and no seed here was chosen for the answer it
/// gives.
const RUN_SEEDS: [u64; 8] = [
    0xf37f_d6bd_d8b6_4a3a,
    0xea33_5e28_791d_60da,
    0x58d0_24d9_7012_dd6a,
    0x0000_0000_0000_0001,
    0x0123_4567_89ab_cdef,
    0xdead_beef_cafe_f00d,
    0x0000_0000_0000_02a1,
    0xffff_ffff_ffff_ffff,
];

/// The seeds of the list above at which a faction must reach the bar.
const RUN_SEEDS_THAT_MUST_REACH: usize = 8;

/// Seeds a demonstration world, runs it, and reports the largest faction.
///
/// Returns the population the largest faction reached, the bar it had to
/// reach, and the tick it reached the bar at. The run stops at the tick the
/// bar is reached, so a run that closes early costs no more ticks.
fn run_a_demonstration_world(seed: u64) -> (u32, u32, Option<u32>) {
    let mut world = World::new(WorldConfig {
        width: RUN_EXTENT,
        height: RUN_EXTENT,
        seed,
        faction_count: RUN_FACTIONS,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    // The engine seeds itself, as the demonstration does. No verb of this
    // test founds anything, and no draw is turned off.
    let seated = world.seed_world().expect("the world seeds once");
    assert!(
        seated.iter().any(|outcome| outcome.founding().is_some()),
        "the seeding put no faction on the ground at seed {seed:#018x}"
    );
    let bar = population_bar(&world);
    let mut largest = 0u32;
    let mut reached_at = None;
    for tick in 1..=RUN_TICKS {
        world.step(1).expect("the step runs");
        let held = largest_faction(&world);
        largest = largest.max(held);
        if held >= bar {
            reached_at = Some(tick);
            break;
        }
    }
    assert!(world.check_invariants());
    (largest, bar, reached_at)
}

/// Returns the units of the largest faction.
fn largest_faction(world: &World) -> u32 {
    let mut largest = 0u32;
    for index in 0..RUN_FACTIONS {
        let faction = FactionId(index);
        let held = world
            .soldiers()
            .iter()
            .filter(|unit| world.soldiers().faction(*unit) == Some(faction))
            .count() as u32;
        largest = largest.max(held);
    }
    largest
}

/// A faction the demonstration seeded reaches a cohort inside the horizon.
///
/// **This test drives the engine and not the mechanism.** It seeds the world
/// the demonstration seeds, it calls no verb of its own, and it removes no
/// draw. A faction founds with the founding group of the balance register,
/// which is smaller than the campaign cohort size, so before growth existed
/// no faction could ever raise a cohort, no contest could kill, and
/// domination could never fire.[^1] [^2]
///
/// # References
///
/// [^1]: ADR-0157, a site's free places are its built housing less the residents the engine counts, the context. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
/// [^2]: Testing rules, drive the real caller. `.agents/rules/testing.md`
#[test]
fn the_demonstration_world_grows_a_faction_to_a_cohort() {
    let mut reached = 0usize;
    let mut report = String::new();
    for seed in RUN_SEEDS {
        let (largest, bar, at) = run_a_demonstration_world(seed);
        assert!(
            largest > FOUNDING_GROUP_DEFAULT,
            "the largest faction at seed {seed:#018x} holds {largest}, \
             which is no more than the founding group of {FOUNDING_GROUP_DEFAULT}"
        );
        if largest >= bar {
            reached += 1;
        }
        report.push_str(&format!(
            "\n  {seed:#018x}: largest faction {largest}, bar {bar}, reached at {at:?}"
        ));
    }
    println!("the population each seed reached:{report}");
    assert!(
        reached >= RUN_SEEDS_THAT_MUST_REACH,
        "a faction reached the cohort bar at {reached} of {} seeds in {RUN_TICKS} ticks, \
         and the bar is {RUN_SEEDS_THAT_MUST_REACH}:{report}",
        RUN_SEEDS.len()
    );
}

// ---------------------------------------------------------------------------
// The housing is a stop, not a slowdown
// ---------------------------------------------------------------------------

/// A site at its housing grows nobody, whatever its store holds.
///
/// The fixture gives the site a large store and no free place. It then raises
/// the housing and asserts that the population moves again, so the bound is
/// shown to be a stop and not a slowdown.
#[test]
fn a_site_at_its_housing_grows_nobody_and_grows_again_when_the_housing_rises() {
    let crowd = 6u32;
    let (mut world, site) = one_site(Fix32::from_int(1000), crowd, crowd);
    assert_eq!(
        world.site_free_places(site),
        Some(0),
        "the fixture must start with no free place"
    );
    let opening = residents_by_a_full_pass(&world, site);
    run(&mut world, 20, 1);
    assert_eq!(
        residents_by_a_full_pass(&world, site),
        opening,
        "a site at its housing grew somebody"
    );
    assert_eq!(world.births(), 0, "the census must report no birth");
    // The store must still be able to pay, or the housing is not what
    // refused. This is the assertion that stops the fixture from measuring
    // an empty store.
    assert!(
        world
            .settlements()
            .store(site)
            .and_then(|store| store.quantity(GOOD))
            .is_some_and(|held| held.0 >= FOOD.0),
        "the fixture must keep a store that could pay for a birth"
    );
    assert!(world.set_site_housing(site, crowd + 3), "the site is live");
    run(&mut world, 20, 1);
    assert_eq!(
        residents_by_a_full_pass(&world, site),
        crowd + 3,
        "a site with room did not grow to its new housing"
    );
}

/// A site with free places and an empty store grows nobody.
///
/// The fixture states no production, so the store stays empty. The site has
/// room, so the refusal is the store and not the housing.
#[test]
fn a_site_with_room_and_no_food_grows_nobody() {
    let (mut world, site) = one_site(Fix32::ZERO, 8, 2);
    assert!(
        world.site_free_places(site).is_some_and(|free| free > 0),
        "the fixture must start with a free place"
    );
    run(&mut world, 20, 1);
    assert_eq!(
        residents_by_a_full_pass(&world, site),
        2,
        "a site with no food grew somebody"
    );
    assert_eq!(world.births(), 0, "the census must report no birth");
    assert!(
        world.site_free_places(site).is_some_and(|free| free > 0),
        "the fixture must still hold a free place, or the housing refused"
    );
}

/// A site above its housing has no free place, and it loses nobody.
///
/// A population above the housing that holds it is a state of the world and
/// not a fault. The free places are zero rather than a value below zero.
#[test]
fn a_site_above_its_housing_reads_no_free_place() {
    let (world, site) = one_site(Fix32::from_int(1000), 2, 7);
    assert_eq!(world.site_residents(site), Some(7));
    assert_eq!(world.site_free_places(site), Some(0));
    assert_eq!(world.site_housing(site), Some(2));
}

// ---------------------------------------------------------------------------
// One birth is one person, and the derived count follows
// ---------------------------------------------------------------------------

/// One growth event adds exactly one person, and the resident count follows.
///
/// The fixture affords exactly one birth and holds exactly one free place, so
/// a stage that added two would fail on the count and a stage that added none
/// would fail on the census.
#[test]
fn one_growth_event_adds_one_person_and_the_reader_follows() {
    let (mut world, site) = one_birth_site(3, 2);
    run(&mut world, 1, 1);
    assert_eq!(world.births(), 1, "the census must report one birth");
    assert_eq!(
        residents_by_a_full_pass(&world, site),
        3,
        "the site must hold one more person"
    );
    assert_eq!(
        world.site_residents(site),
        Some(residents_by_a_full_pass(&world, site)),
        "the reader must equal a full pass over the home column"
    );
    assert_eq!(world.site_free_places(site), Some(0));
    // The store paid, so the next tick affords nothing.
    run(&mut world, 5, 1);
    assert_eq!(
        residents_by_a_full_pass(&world, site),
        3,
        "the store had nothing left to pay with"
    );
}

/// The census row counts the births of the run, and it never falls.
///
/// **The row and the per-tick reader are two questions.** The stage clears
/// the per-tick count before it acts, so a row that read it would fall to
/// zero on the first quiet tick and a reader could not tell a run that grew
/// nobody from a run that stopped growing.[^5]
///
/// The fixture affords one birth and then runs five quiet ticks, which is the
/// extreme the row lives at.
///
/// # References
///
/// [^5]: Findings register, FND-498. `docs/FINDINGS.md`
#[test]
fn the_census_counts_the_births_of_the_run_and_never_falls() {
    let (mut world, _site) = one_birth_site(3, 2);
    assert_eq!(census(&world, "births"), 0, "the fixture starts at zero");
    run(&mut world, 1, 1);
    assert_eq!(world.births(), 1, "the tick reader must report one birth");
    assert_eq!(census(&world, "births"), 1, "the row must count the birth");
    // The store paid for the one birth it could afford, so these ticks grow
    // nobody. The per-tick reader falls and the row does not.
    run(&mut world, 5, 1);
    assert_eq!(world.births(), 0, "the tick reader must fall");
    assert_eq!(
        census(&world, "births"),
        1,
        "the row must still say what the run grew"
    );
}

/// A grown person is a worker, because that is the type the spawn path gives.
#[test]
fn a_grown_person_is_a_worker() {
    let (mut world, site) = one_birth_site(3, 0);
    run(&mut world, 1, 1);
    assert_eq!(world.births(), 1);
    let slot = world.settlements().slot_of(site);
    let grown: Vec<Entity> = world
        .soldiers()
        .iter()
        .filter(|unit| world.soldiers().home(*unit) == Some(slot))
        .collect();
    assert_eq!(grown.len(), 1);
    assert_eq!(world.unit_type(grown[0]), Some(WORKER));
}

/// The rate never scales with the free places.
///
/// The fixture holds the store fixed and varies the free places above one.
/// The number of births must not move.
///
/// **The signature of the rate function is the stronger guard.** The rate
/// reads the store and the cost, and it takes no free places at all, so a
/// rate that scaled with them cannot be written without changing that
/// signature. This test was written first and a scaling defect was put back
/// under it. The test stayed green, because the rate equals what the store
/// affords and each birth pays, so no scaling can raise the count above what
/// the store holds. The report of the work states that reading.
#[test]
fn the_rate_never_scales_with_the_free_places() {
    let stock = Fix32::from_int(2);
    let mut counts = Vec::new();
    for housing in [4u32, 8, 32] {
        let (mut world, _) = one_site(stock, housing, 0);
        run(&mut world, 1, 1);
        counts.push(world.births());
    }
    assert!(counts[0] > 0, "the fixture must reach a birth");
    assert_eq!(
        counts[0], counts[1],
        "the rate followed the free places: {counts:?}"
    );
    assert_eq!(
        counts[1], counts[2],
        "the rate followed the free places: {counts:?}"
    );
}

/// A site with one free place and more than one proposal admits exactly one.
#[test]
fn one_free_place_admits_exactly_one_of_many_proposals() {
    let (mut world, site) = one_site(Fix32::from_int(4), 1, 0);
    // The store affords more than one birth, and the housing holds one
    // person. The refusal is therefore the free place and not the store.
    assert!(
        growth::proposals(&[Fix32::from_int(4)], &[FOOD]) > 1,
        "the fixture must propose more than once"
    );
    run(&mut world, 1, 1);
    assert_eq!(world.births(), 1, "more than one proposal was admitted");
    assert_eq!(residents_by_a_full_pass(&world, site), 1);
}

// ---------------------------------------------------------------------------
// The draw key
// ---------------------------------------------------------------------------

/// The birth draw depends on the tick, on the site and on the proposal.
///
/// A draw keyed on the wrong field draws the same wrong value on every
/// thread, on every run and on every machine, so both determinism tests pass
/// under the defect. Each assertion below changes one field of the key.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2. `.agents/rules/testing.md`
#[test]
fn the_birth_draw_depends_on_the_tick_the_site_and_the_proposal() {
    let seed = 0x0cac_4e77_0060u64;
    let half = Fix32(Fix32::ONE.0 / 2);
    let ticks: Vec<bool> = (0..64)
        .map(|tick| growth::proposal_takes(seed, tick, 0, 0, half))
        .collect();
    assert!(
        ticks.iter().any(|took| *took) && ticks.iter().any(|took| !*took),
        "the tick is not in the key"
    );
    let sites: Vec<bool> = (0..64)
        .map(|site| growth::proposal_takes(seed, 1, site, 0, half))
        .collect();
    assert!(
        sites.iter().any(|took| *took) && sites.iter().any(|took| !*took),
        "the site is not in the key"
    );
    // Two proposals of one site in one tick. A key that held the site alone
    // would give them the same answer at every tick.
    let mut differ = false;
    for tick in 0..64 {
        let first = growth::proposal_takes(seed, tick, 3, 0, half);
        let second = growth::proposal_takes(seed, tick, 3, 1, half);
        differ |= first != second;
    }
    assert!(differ, "the proposal ordinal is not in the key");
}

/// A chance below one refuses some proposals, so the draw is not inert.
#[test]
fn a_chance_below_one_refuses_some_proposals() {
    let (mut world, _) = one_site(Fix32::from_int(200), 200, 0);
    world.set_birth_chance(Fix32(Fix32::ONE.0 / 2));
    let mut grew = 0u32;
    let mut refused = 0u32;
    for _ in 0..40 {
        world.step(1).expect("the step must run");
        let born = world.births();
        grew += born;
        refused += growth::PROPOSAL_CEILING - born;
    }
    assert!(grew > 0, "the chance refused every proposal");
    assert!(refused > 0, "the chance admitted every proposal");
}

// ---------------------------------------------------------------------------
// The state hash and the thread count
// ---------------------------------------------------------------------------

/// The housing of a site enters the state hash.
///
/// The test changes the housing alone and asserts that the hash changes. A
/// value that the step reads on every tick and the hash does not cover lets
/// two different worlds hash the same and then diverge.[^1]
///
/// # References
///
/// [^1]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D6. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
#[test]
fn the_housing_enters_the_state_hash() {
    let (mut world, site) = one_site(Fix32::ZERO, 4, 1);
    let before = world.state_hash();
    assert!(world.set_site_housing(site, 5), "the site is live");
    let after = world.state_hash();
    assert_ne!(before, after, "the housing is outside the state hash");
}

/// Growth gives the same world at one, two and twelve threads.
///
/// The stage takes no thread count of its own, but the step around it does.
/// A world that grew differently under a thread count would give one binary
/// two answers.[^1]
///
/// # References
///
/// [^1]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
#[test]
fn growth_gives_one_answer_at_every_thread_count() {
    let mut hashes = Vec::new();
    let mut populations = Vec::new();
    for threads in [1usize, 2, 12] {
        let (mut world, site) = one_site(Fix32::from_int(64), 40, 2);
        world.set_birth_chance(Fix32(Fix32::ONE.0 / 2));
        run(&mut world, 30, threads);
        hashes.push(world.state_hash());
        populations.push(residents_by_a_full_pass(&world, site));
    }
    assert!(
        populations[0] > 2,
        "the fixture must grow, or the comparison is empty"
    );
    assert_eq!(hashes[0], hashes[1], "one thread and two disagree");
    assert_eq!(hashes[1], hashes[2], "two threads and twelve disagree");
    assert_eq!(populations[0], populations[1]);
    assert_eq!(populations[1], populations[2]);
}

// ---------------------------------------------------------------------------
// Nothing stores a second resident count
// ---------------------------------------------------------------------------

/// The resident reader equals a full pass over the home column, after a run
/// that assigned, grew and killed units.
#[test]
fn the_resident_reader_equals_a_full_pass_after_a_run() {
    let (mut world, site) = one_site(Fix32::from_int(20), 30, 4);
    run(&mut world, 40, 2);
    assert_eq!(
        world.site_residents(site),
        Some(residents_by_a_full_pass(&world, site)),
        "the reader and the column disagree"
    );
    assert!(
        world.cohorts_describe_the_units(),
        "the cohort table no longer describes the units"
    );
}

// ---------------------------------------------------------------------------
// A slot a death freed this frame takes a birth, and the identity is distinct
// ---------------------------------------------------------------------------

/// A slot that a death freed is reused, and the dead identity never resolves.
///
/// The fixture states a need rule that takes every unit down whatever the
/// store holds, and a store that pays for a birth on every tick. Deaths and
/// births therefore meet in one frame, which is the case the ordering of the
/// two stages exists for.[^1]
///
/// # References
///
/// [^1]: ADR-0082, the store sets the rate of a birth and the housing admits it, decision D4. `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md`
#[test]
fn a_slot_a_death_freed_is_reused_and_the_dead_identity_never_resolves() {
    let (mut world, _site) = one_site(Fix32::from_int(400), 40, 4);
    // A ration of nothing against a decay takes every unit down, whatever the
    // store holds. The bound is low, so a unit ends inside the run.
    world.set_need_rule(
        NeedRule::new(
            Fix32(Fix32::ONE.0 / 4),
            Fix32::ZERO,
            Fix32(Fix32::ONE.0 / 2),
            Fix32::ZERO,
            Fix32::ONE,
        )
        .expect("no rate is below zero"),
    );
    let mut seen: Vec<Entity> = world.soldiers().iter().collect();
    let mut deaths = 0usize;
    let mut births = 0u32;
    for _ in 0..40 {
        world.step(1).expect("the step must run");
        assert!(world.check_invariants());
        births += world.births();
        deaths += world.starved_log().len();
        for unit in world.soldiers().iter() {
            if !seen.contains(&unit) {
                seen.push(unit);
            }
        }
    }
    assert!(deaths > 0, "the fixture must reach a death");
    assert!(births > 0, "the fixture must reach a birth");
    // The arena opened fewer slots than the identities it handed out, so a
    // slot was reused. An arena that opened one slot for each identity would
    // never reach the case this test is about.
    assert!(
        world.soldiers().slot_count() < seen.len() as u32,
        "the arena opened one slot for each identity, so no slot was reused"
    );
    // No identity the run ended resolves any more, whatever took its slot.
    let live: Vec<Entity> = world.soldiers().iter().collect();
    for unit in &seen {
        if !live.contains(unit) {
            assert!(
                !world.soldiers().contains(*unit),
                "a dead identity still resolves"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// What the need rule decides about a population that grew
// ---------------------------------------------------------------------------

/// Under a rule whose ration is above the decay, a fed site holds what it
/// grew.
///
/// The fixture states its own rates rather than taking the default, because
/// the default sets the ration equal to the decay and a unit whose need
/// reaches zero then never climbs back.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-089. `docs/FINDINGS.md`
#[test]
fn a_fed_site_holds_the_population_it_grew() {
    let (mut world, site) = one_site(Fix32::from_int(4000), 24, 2);
    world.set_need_rule(
        NeedRule::new(
            Fix32(Fix32::ONE.0 / 32),
            Fix32(Fix32::ONE.0 / 16),
            Fix32(Fix32::ONE.0 / 2),
            Fix32(Fix32::ONE.0 / 16),
            Fix32::from_int(4),
        )
        .expect("no rate is below zero"),
    );
    let mut highest = 0u32;
    for _ in 0..80 {
        world.step(1).expect("the step must run");
        highest = highest.max(residents_by_a_full_pass(&world, site));
    }
    assert!(highest > 2, "the fixture must grow");
    assert_eq!(
        residents_by_a_full_pass(&world, site),
        highest,
        "a fed site lost what it grew"
    );
}

/// Under the default rule, a site that grows past what its store feeds loses
/// the units it grew.
///
/// The default rule sets the ration equal to the decay, so a unit whose need
/// reaches zero never climbs back and its deficit only rises.[^1] The site
/// here earns nothing, so every mouth growth adds empties the store sooner.
///
/// # References
///
/// [^1]: Findings register, FND-089. `docs/FINDINGS.md`
#[test]
fn under_the_default_rule_a_site_loses_what_it_grew() {
    let (mut world, site) = one_site(Fix32::from_int(30), 30, 2);
    world.set_need_rule(NeedRule::DEFAULT);
    let mut highest = 0u32;
    for _ in 0..400 {
        world.step(1).expect("the step must run");
        highest = highest.max(residents_by_a_full_pass(&world, site));
    }
    let closing = residents_by_a_full_pass(&world, site);
    assert!(highest > 2, "the fixture must grow before it declines");
    assert!(
        closing < highest,
        "the site held {closing} against a peak of {highest}, so nothing was lost"
    );
}
