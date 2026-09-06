//! Production and upkeep as rates attached to a site.
//!
//! The tests drive the engine and then read the store. A rate that only a
//! test applies proves that the arithmetic works and not that the step
//! reaches it.[^1]
//!
//! Each test states what the value depends on, and not only that the value
//! repeats. A rate that a run applied twice with the wrong period repeats
//! perfectly, so repetition proves nothing on its own.[^2]
//!
//! Every fixture asserts that it produced the case that it claims to test.
//! A store that never runs low passes a shortfall test that a real shortage
//! would fail.[^3]
//!
//! # The rate the pass applies is derived
//!
//! A test here sets a stored rate, and the pass applies the effective rate.
//! The pipeline scales the stored production rate by what the world holds,
//! and it adds a share of the store and a share of the ration of each
//! resident to the stored upkeep.[^4] A test that stated a store literal
//! would therefore state the whole pipeline, and it would fail on every
//! change to a weight that these tests do not govern.
//!
//! Each test below reads the effective rate from the engine and states the
//! schedule against it. The schedule is what these tests own: how often the
//! pass applies, which tick inside the period it applies on, and what the
//! pass does with a store that cannot hold the production or cannot pay the
//! bill.
//!
//! # References
//!
//! [^1]: Testing rules, drive the real caller. `.claude/rules/testing.md`
//! [^2]: Testing rules, section 2. `.claude/rules/testing.md`
//! [^3]: Testing rules, section 2a. `.claude/rules/testing.md`
//! [^4]: ADR-0062, production and upkeep are rates attached to a site, decisions D1 and D7. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`

use cachette_core::rates::{RateError, RateSchedule};
use cachette_core::sim_math;
use cachette_core::site::CommodityId;
use cachette_core::{Axial, Entity, FactionId, Fix32, World, WorldConfig};

/// The commodity that every test uses.
const GOOD: CommodityId = CommodityId(0);

/// A world that holds ground on every tile the fixtures need.
const CONFIG: WorldConfig = WorldConfig {
    width: 16,
    height: 16,
    seed: 0x0123_4567_89ab_cdef,
    faction_count: 2,
    unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
};

/// Builds a world and founds one settlement on the first tile.
fn one_site(period: u32, phase: u32) -> (World, Entity) {
    let mut world = World::new(CONFIG).expect("the extent must describe a world");
    world
        .set_economy_schedule(period, phase)
        .expect("the period is inside the range");
    let site = world
        .found_settlement(Axial::new(0, 0), FactionId(0))
        .expect("the tile is inside the world");
    (world, site)
}

/// Steps the world and asserts that the invariants hold at each frame.
fn run(world: &mut World, frames: u64, threads: usize) {
    for _ in 0..frames {
        world.step(threads).expect("the step must run");
        assert!(world.check_invariants(), "the world lost an invariant");
    }
}

/// Returns what the store of a site holds.
fn held(world: &World, site: Entity) -> Option<Fix32> {
    world
        .settlements()
        .store(site)
        .and_then(|store| store.quantity(GOOD))
}

/// Returns what one application of the production of a site pays.
///
/// The pass applies the effective rate, so the fixture reads that rate rather
/// than the stored one. The two differ by the pipeline, which these tests do
/// not govern.
fn earned_each_application(world: &World, site: Entity) -> Fix32 {
    let rate = world
        .effective_production_rate(site, GOOD)
        .expect("the site is live");
    world.economy_schedule().per_application(rate)
}

/// Returns what one application of the upkeep of a site charges.
fn owed_each_application(world: &World, site: Entity) -> Fix32 {
    let rate = world
        .effective_upkeep_rate(site, GOOD)
        .expect("the site is live");
    world.economy_schedule().per_application(rate)
}

/// Writes a store that stands a stated number of raw units below the bill
/// that the next application charges, and returns the bill.
///
/// The derived upkeep holds a share of the store itself, so the bill moves
/// when the store moves, and the store the fixture wants is a fixed point.
/// The loop finds it and the assertion below states that it did. A fixture
/// that missed the point would put the store somewhere other than the
/// boundary, and the boundary is the whole case.
fn store_below_the_bill(world: &mut World, site: Entity, below: i32) -> Fix32 {
    let mut written = Fix32::ZERO;
    for _ in 0..32 {
        world
            .set_settlement_store(site, GOOD, written)
            .expect("the commodity is inside the set");
        let bill = owed_each_application(world, site);
        let wanted = Fix32(bill.0 - below);
        if wanted == written {
            return bill;
        }
        written = wanted;
    }
    panic!("the fixture found no store that stands {below} raw units below the bill");
}

#[test]
fn a_site_produces_into_its_store_at_the_interval() {
    let (mut world, site) = one_site(2, 0);
    assert_eq!(
        world.set_production_rate(site, GOOD, Fix32::from_int(3)),
        Ok(true)
    );
    let earned = earned_each_application(&world, site);
    assert!(
        earned > Fix32::ZERO,
        "the fixture must earn something, or it tests nothing"
    );

    // The schedule applies at tick 2, at tick 4 and at tick 6. The first
    // frame therefore moves nothing.
    run(&mut world, 1, 1);
    assert_eq!(
        held(&world, site),
        Some(Fix32::ZERO),
        "tick one is not an application tick"
    );
    run(&mut world, 1, 1);
    assert_eq!(
        held(&world, site),
        Some(earned),
        "the first application pays one period of the rate"
    );

    // Six frames hold three applications. The store carries the derived
    // upkeep as well, so the count of what the pass paid in is the ledger.
    run(&mut world, 4, 1);
    assert_eq!(
        world.rate_ledger().produced[0].0,
        3 * i64::from(earned.0),
        "six frames must hold three applications"
    );
}

#[test]
fn a_store_rises_by_the_rate_multiplied_by_the_ticks_that_passed() {
    // The rate is what one tick earns. The period says how often the store
    // moves, and it does not say how much the source pays over time. Two
    // worlds on different periods must therefore agree after a whole number
    // of periods.
    //
    // **The reading is the ledger and not the store.** The upkeep now holds a
    // share of the store itself, and a share of a store is a drain that the
    // period discretises. Two worlds on different periods hold the store over
    // different spans, so they pay different amounts to keep it, and their
    // stores part. What each one earned does not depend on the period, and
    // that is the claim this test holds.
    let (mut fast, quick_site) = one_site(2, 0);
    let (mut slow, slow_site) = one_site(6, 0);
    assert_eq!(
        fast.set_production_rate(quick_site, GOOD, Fix32::from_int(5)),
        Ok(true)
    );
    assert_eq!(
        slow.set_production_rate(slow_site, GOOD, Fix32::from_int(5)),
        Ok(true)
    );
    let rate = fast
        .effective_production_rate(quick_site, GOOD)
        .expect("the site is live");
    assert_eq!(
        rate,
        slow.effective_production_rate(slow_site, GOOD)
            .expect("the site is live"),
        "the two worlds must hold one rate, or the totals below differ for \
         another reason"
    );

    run(&mut fast, 12, 1);
    run(&mut slow, 12, 1);
    let quick = fast.rate_ledger().produced[0].0;
    let sluggish = slow.rate_ledger().produced[0].0;
    assert_eq!(quick, i64::from(rate.0) * 12, "twelve ticks passed");
    assert_eq!(quick, sluggish);
    assert!(
        held(&fast, quick_site).expect("the site is live") > Fix32::ZERO,
        "the fixture must leave something in the store, or it tests nothing"
    );
}

#[test]
fn the_period_decides_how_often_the_store_moves() {
    // The test above proves that the totals agree over a whole number of
    // periods. This one proves that the period still reaches the behaviour,
    // by reading the store part way through the slower period.
    let (mut fast, quick_site) = one_site(2, 0);
    let (mut slow, slow_site) = one_site(6, 0);
    assert_eq!(
        fast.set_production_rate(quick_site, GOOD, Fix32::from_int(5)),
        Ok(true)
    );
    assert_eq!(
        slow.set_production_rate(slow_site, GOOD, Fix32::from_int(5)),
        Ok(true)
    );
    let quick_pays = earned_each_application(&fast, quick_site);
    run(&mut fast, 4, 1);
    run(&mut slow, 4, 1);
    assert_eq!(
        fast.rate_ledger().produced[0].0,
        2 * i64::from(quick_pays.0),
        "four frames hold two applications of the shorter period"
    );
    assert!(
        held(&fast, quick_site).expect("the site is live") > Fix32::ZERO,
        "the faster period must have moved the store"
    );
    assert_eq!(
        slow.rate_ledger().produced[0].0,
        0,
        "the slower period must not have applied yet"
    );
    assert_eq!(
        held(&slow, slow_site),
        Some(Fix32::ZERO),
        "the slower period must not have applied yet"
    );
}

#[test]
fn the_phase_decides_which_tick_inside_the_period_applies() {
    let (mut early, early_site) = one_site(4, 0);
    let (mut late, late_site) = one_site(4, 3);
    assert_eq!(
        early.set_production_rate(early_site, GOOD, Fix32::from_int(1)),
        Ok(true)
    );
    assert_eq!(
        late.set_production_rate(late_site, GOOD, Fix32::from_int(1)),
        Ok(true)
    );
    // The store of the late world starts empty, so the first application owes
    // nothing to keep it and the store afterwards is what the rate paid.
    let paid = earned_each_application(&late, late_site);
    assert!(
        paid > Fix32::ZERO,
        "the fixture must earn something, or it tests nothing"
    );

    // Three frames reach tick 3. The phase of zero applies at tick 4, so it
    // has not applied. The phase of three applies at tick 3, so it has.
    run(&mut early, 3, 1);
    run(&mut late, 3, 1);
    assert_eq!(held(&early, early_site), Some(Fix32::ZERO));
    assert_eq!(held(&late, late_site), Some(paid));
}

#[test]
fn a_site_pays_this_bill_from_these_earnings() {
    // Production runs before upkeep in one application. A site that earns
    // exactly what it owes therefore stays solvent, and it reports no
    // shortfall. The reverse order would make it insolvent every time.
    //
    // The upkeep the fixture stores is the rate the pipeline gives the
    // production, and not the rate the test stored. A site that earns its
    // bill holds nothing afterwards, so the derived share of the store adds
    // nothing on any later application and the two sides stay equal.
    let (mut world, site) = one_site(2, 0);
    assert_eq!(
        world.set_production_rate(site, GOOD, Fix32::from_int(4)),
        Ok(true)
    );
    let earns = world
        .effective_production_rate(site, GOOD)
        .expect("the site is live");
    assert_eq!(world.set_upkeep_rate(site, GOOD, earns), Ok(true));
    assert_eq!(
        world.effective_upkeep_rate(site, GOOD),
        Some(earns),
        "the fixture must owe exactly what it earns, or it tests nothing"
    );
    run(&mut world, 8, 1);
    assert_eq!(held(&world, site), Some(Fix32::ZERO));
    assert_eq!(
        world.rate_ledger().shortfall[0].0,
        0,
        "a site that earns what it owes must never fall short"
    );
    assert!(
        world.rate_ledger().spent[0].0 > 0,
        "the fixture must have spent something, or it tests nothing"
    );
}

#[test]
fn a_store_one_unit_short_stops_at_zero_and_reports_the_shortfall() {
    let (mut world, site) = one_site(2, 0);
    assert_eq!(
        world.set_upkeep_rate(site, GOOD, Fix32::from_int(1)),
        Ok(true)
    );
    // The store is one raw unit short of the bill. This is the boundary: a
    // store that could pay proves nothing about the case that cannot.
    let owed = store_below_the_bill(&mut world, site, 1);
    let short = Fix32(owed.0 - 1);
    assert_eq!(
        held(&world, site),
        Some(short),
        "the fixture must stand one raw unit below the bill"
    );

    run(&mut world, 2, 1);

    assert_eq!(
        held(&world, site),
        Some(Fix32::ZERO),
        "the store stops at zero and never goes below it"
    );
    let log = world.shortfall_log();
    assert_eq!(log.len(), 1, "the fixture must produce one shortfall");
    assert_eq!(log[0].amount, Fix32(1), "the shortfall is the one unit");
    assert_eq!(log[0].site, site.to_bits());
    assert_eq!(log[0].commodity, GOOD.0);
    assert_eq!(log[0].padding, [0; 2]);
    assert_eq!(world.rate_ledger().shortfall[0].0, 1);
    assert_eq!(world.rate_ledger().spent[0].0, i64::from(short.0));
}

#[test]
fn a_store_that_can_pay_reports_no_shortfall() {
    // The companion of the test above. A fixture that always fell short
    // would pass a shortfall test that never reached the paying case.
    let (mut world, site) = one_site(2, 0);
    assert_eq!(
        world.set_upkeep_rate(site, GOOD, Fix32::from_int(1)),
        Ok(true)
    );
    let owed = store_below_the_bill(&mut world, site, 0);
    assert_eq!(
        held(&world, site),
        Some(owed),
        "the fixture must hold exactly the bill"
    );
    assert!(owed > Fix32::ZERO, "the fixture must owe something");
    run(&mut world, 2, 1);
    assert!(world.shortfall_log().is_empty());
    assert_eq!(world.rate_ledger().shortfall[0].0, 0);
    assert_eq!(
        world.rate_ledger().spent[0].0,
        i64::from(owed.0),
        "the store paid the whole bill"
    );
}

#[test]
fn production_that_the_store_cannot_hold_becomes_a_spill() {
    let (mut world, site) = one_site(2, 0);
    // The store starts one raw unit below its ceiling, so one application
    // reaches the ceiling and almost all of it spills. A store that never
    // reached the ceiling would measure the fixture and not the kernel.
    assert_eq!(
        world.set_settlement_store(site, GOOD, Fix32(Fix32::MAX.0 - 1)),
        Ok(true)
    );
    assert_eq!(
        world.set_production_rate(site, GOOD, Fix32::from_int(100)),
        Ok(true)
    );
    // The bill a store of this size carries is the share of itself that the
    // pipeline charges to keep it. Production runs first, so the store
    // saturates at the ceiling and then pays that bill out of the ceiling.
    let offered = earned_each_application(&world, site);
    let bill = owed_each_application(&world, site);
    assert!(
        bill > Fix32::ZERO,
        "a store at the ceiling must owe something to keep"
    );
    run(&mut world, 2, 1);
    let after = held(&world, site).expect("the site is live");
    assert!(
        after > Fix32::ZERO,
        "the store saturated at its ceiling and never wrapped, but it holds \
         {after:?}"
    );
    assert_eq!(
        after,
        sim_math::sub(Fix32::MAX, bill),
        "the store reached the ceiling and paid the bill from it"
    );
    assert_eq!(
        world.rate_ledger().produced[0].0,
        1,
        "only the one unit that fitted landed"
    );
    assert!(
        world.rate_ledger().spilled[0].0 > 0,
        "the fixture must reach the ceiling, or it tests nothing"
    );
    assert_eq!(
        world.rate_ledger().produced[0].0 + world.rate_ledger().spilled[0].0,
        i64::from(offered.0),
        "what landed plus what spilled is what the rate offered"
    );
}

#[test]
fn the_table_refuses_a_rate_below_zero() {
    let (mut world, site) = one_site(2, 0);
    let below = Fix32(-1);
    assert_eq!(
        world.set_production_rate(site, GOOD, below),
        Err(RateError::RateBelowZero(below))
    );
    assert_eq!(
        world.set_upkeep_rate(site, GOOD, below),
        Err(RateError::RateBelowZero(below))
    );
    assert_eq!(world.production_rate(site, GOOD), Some(Fix32::ZERO));
}

#[test]
fn the_scaling_multiply_truncates_towards_negative_infinity() {
    // The direction is stated, and the test names it. A value below zero
    // rounds away from zero and a value above zero rounds towards it. Upkeep
    // is therefore a rate above zero that subtracts, and never a production
    // rate below zero: a rate below zero would lose one raw unit on every
    // application, for ever.
    let half = Fix32(1 << 15);
    assert_eq!(
        sim_math::mul(Fix32(1), half),
        Fix32::ZERO,
        "a value above zero rounds towards zero"
    );
    assert_eq!(
        sim_math::mul(Fix32(-1), half),
        Fix32(-1),
        "a value below zero rounds away from zero, which is the bias"
    );
    // The engine never meets that bias, because every rate it holds is at or
    // above zero, and a rate at or above zero cannot round below zero.
    let schedule = RateSchedule::new(7, 0).expect("the period is inside the range");
    assert_eq!(schedule.per_application(Fix32::ZERO), Fix32::ZERO);
    assert_eq!(schedule.per_application(Fix32(1)), Fix32(7));
}

#[test]
fn the_schedule_refuses_a_period_of_zero() {
    assert_eq!(RateSchedule::new(0, 0), None);
    let mut world = World::new(CONFIG).expect("the extent must describe a world");
    assert_eq!(
        world.set_economy_schedule(0, 0),
        Err(RateError::PeriodOutsideRange(0))
    );
    // The refusal left the schedule alone rather than half-written.
    assert_eq!(world.economy_schedule(), RateSchedule::DEFAULT);
}

#[test]
fn a_rate_does_not_outlive_the_site_that_earned_it() {
    let (mut world, site) = one_site(2, 0);
    assert_eq!(
        world.set_production_rate(site, GOOD, Fix32::from_int(9)),
        Ok(true)
    );
    let address = world.settlements().address(site).expect("the site is live");
    assert!(world.destroy_settlement(site));
    let heir = world
        .found_settlement(address, FactionId(0))
        .expect("the tile is free again");
    assert_eq!(world.production_rate(heir, GOOD), Some(Fix32::ZERO));
    run(&mut world, 4, 1);
    assert_eq!(
        world
            .settlements()
            .store(heir)
            .and_then(|store| store.quantity(GOOD)),
        Some(Fix32::ZERO),
        "the slot must not pay the successor of the site that earned"
    );
}

#[test]
fn a_lost_site_takes_its_holding_out_of_the_account() {
    let (mut world, site) = one_site(2, 0);
    assert_eq!(
        world.set_settlement_store(site, GOOD, Fix32::from_int(40)),
        Ok(true)
    );
    assert!(world.check_invariants());
    assert!(world.destroy_settlement(site));
    assert!(
        world.check_invariants(),
        "the account must fall with the holding that left"
    );
}

/// Founds sites over a world and gives each one a rate.
///
/// The pattern is fixed, so it is the same on every run and at every thread
/// count. It gives some sites more upkeep than production and gives others
/// the reverse, so one long run reaches both the paying case and the case
/// that falls short.
fn many_sites(world: &mut World) -> Vec<Entity> {
    let grid = world.grid();
    let mut sites = Vec::new();
    for index in 0..48u32 {
        let address = Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32);
        let site = world
            .found_settlement(address, FactionId((index % 2) as u16))
            .expect("the address is inside the world");
        // A rate of zero is a real rate, so part of the population earns
        // nothing and owes nothing.
        if index % 4 != 3 {
            world
                .set_production_rate(site, GOOD, Fix32::from_int((index % 5) as i16))
                .expect("the rate is at or above zero");
            world
                .set_upkeep_rate(site, GOOD, Fix32::from_int((index % 3) as i16))
                .expect("the rate is at or above zero");
        }
        sites.push(site);
    }
    sites
}

/// Runs a long economy and returns the totals it reached.
fn long_run(threads: usize) -> (i64, i64, i64, usize) {
    let mut world = World::new(CONFIG).expect("the extent must describe a world");
    world
        .set_economy_schedule(3, 1)
        .expect("the period is inside the range");
    let sites = many_sites(&mut world);
    let mut shortfalls = 0usize;
    for _ in 0..90 {
        world.step(threads).expect("the step must run");
        shortfalls += world.shortfall_log().len();
        // The conservation equality is inside the invariant check, so it runs
        // on every frame rather than once at the end.
        assert!(world.check_invariants(), "the world lost an invariant");
    }
    let mut held = 0i64;
    for site in &sites {
        held += i64::from(
            world
                .settlements()
                .store(*site)
                .and_then(|store| store.quantity(GOOD))
                .expect("the site is live")
                .0,
        );
    }
    let ledger = world.rate_ledger();
    // Growth is a third term, and the ration of the people it made is a
    // fourth. Growth takes food out of a store to make a person, and that
    // person then draws on the same store. The rate ledger holds neither act,
    // so a statement that left them out would fail the moment a site grew.
    let born = world.growth_ledger()[0].0;
    let drawn = world.draw_ledger().granted[0].0;
    (
        held + born + drawn,
        ledger.produced[0].0,
        ledger.spent[0].0,
        shortfalls,
    )
}

#[test]
fn what_a_site_produced_minus_what_it_spent_is_what_it_holds() {
    let (held, produced, spent, shortfalls) = long_run(1);
    assert_eq!(
        held,
        produced - spent,
        "the stores started empty, so the holding, plus what growth took, plus \
         what the cohorts drew, is the net of the ledger"
    );
    assert!(
        produced > 0,
        "the fixture must produce, or it tests nothing"
    );
    assert!(spent > 0, "the fixture must spend, or it tests nothing");
    assert!(
        shortfalls > 0,
        "the fixture must reach a site that cannot pay, or it never tests the shortfall"
    );
}


#[test]
fn the_totals_of_a_long_run_do_not_depend_on_the_thread_count() {
    let expected = long_run(1);
    for threads in [2usize, 12] {
        assert_eq!(
            long_run(threads),
            expected,
            "the run differs at {threads} threads"
        );
    }
}
