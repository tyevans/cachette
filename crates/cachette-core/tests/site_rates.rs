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
//! # References
//!
//! [^1]: Testing rules, drive the real caller. `.claude/rules/testing.md`
//! [^2]: Testing rules, section 2. `.claude/rules/testing.md`
//! [^3]: Testing rules, section 2a. `.claude/rules/testing.md`

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

#[test]
fn a_site_produces_into_its_store_at_the_interval() {
    let (mut world, site) = one_site(2, 0);
    assert_eq!(
        world.set_production_rate(site, GOOD, Fix32::from_int(3)),
        Ok(true)
    );
    // The schedule applies at tick 2 and at tick 4. Six frames therefore hold
    // three applications, and each one pays three units for each of the two
    // ticks in the period.
    run(&mut world, 6, 1);
    let held = world
        .settlements()
        .store(site)
        .and_then(|store| store.quantity(GOOD))
        .expect("the site is live");
    assert_eq!(held, Fix32::from_int(3 * 2 * 3));
}

#[test]
fn a_store_rises_by_the_rate_multiplied_by_the_ticks_that_passed() {
    // The rate is what one tick earns. The period says how often the store
    // moves, and it does not say how much the store moves over time. Two
    // worlds on different periods must therefore agree after a whole number
    // of periods.
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
    run(&mut fast, 12, 1);
    run(&mut slow, 12, 1);
    let quick = fast
        .settlements()
        .store(quick_site)
        .and_then(|store| store.quantity(GOOD));
    let sluggish = slow
        .settlements()
        .store(slow_site)
        .and_then(|store| store.quantity(GOOD));
    assert_eq!(quick, Some(Fix32::from_int(5 * 12)));
    assert_eq!(quick, sluggish);
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
    run(&mut fast, 4, 1);
    run(&mut slow, 4, 1);
    assert_eq!(
        fast.settlements()
            .store(quick_site)
            .and_then(|store| store.quantity(GOOD)),
        Some(Fix32::from_int(20))
    );
    assert_eq!(
        slow.settlements()
            .store(slow_site)
            .and_then(|store| store.quantity(GOOD)),
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
    // Three frames reach tick 3. The phase of zero applies at tick 4, so it
    // has not applied. The phase of three applies at tick 3, so it has.
    run(&mut early, 3, 1);
    run(&mut late, 3, 1);
    assert_eq!(
        early
            .settlements()
            .store(early_site)
            .and_then(|store| store.quantity(GOOD)),
        Some(Fix32::ZERO)
    );
    assert_eq!(
        late.settlements()
            .store(late_site)
            .and_then(|store| store.quantity(GOOD)),
        Some(Fix32::from_int(4))
    );
}

#[test]
fn a_site_pays_this_bill_from_these_earnings() {
    // Production runs before upkeep in one application. A site that earns
    // exactly what it owes therefore stays solvent, and it reports no
    // shortfall. The reverse order would make it insolvent every time.
    let (mut world, site) = one_site(2, 0);
    assert_eq!(
        world.set_production_rate(site, GOOD, Fix32::from_int(4)),
        Ok(true)
    );
    assert_eq!(
        world.set_upkeep_rate(site, GOOD, Fix32::from_int(4)),
        Ok(true)
    );
    run(&mut world, 8, 1);
    assert_eq!(
        world
            .settlements()
            .store(site)
            .and_then(|store| store.quantity(GOOD)),
        Some(Fix32::ZERO)
    );
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
    // The upkeep of one application is two units, because the rate is one
    // unit each tick and the period is two ticks.
    let owed = world.economy_schedule().per_application(Fix32::from_int(1));
    assert_eq!(owed, Fix32::from_int(2));
    assert_eq!(
        world.set_upkeep_rate(site, GOOD, Fix32::from_int(1)),
        Ok(true)
    );
    // The store is one raw unit short of the bill. This is the boundary: a
    // store that could pay proves nothing about the case that cannot.
    let short = Fix32(owed.0 - 1);
    assert_eq!(world.set_settlement_store(site, GOOD, short), Ok(true));

    run(&mut world, 2, 1);

    assert_eq!(
        world
            .settlements()
            .store(site)
            .and_then(|store| store.quantity(GOOD)),
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
    let owed = world.economy_schedule().per_application(Fix32::from_int(1));
    assert_eq!(
        world.set_upkeep_rate(site, GOOD, Fix32::from_int(1)),
        Ok(true)
    );
    assert_eq!(world.set_settlement_store(site, GOOD, owed), Ok(true));
    run(&mut world, 2, 1);
    assert!(world.shortfall_log().is_empty());
    assert_eq!(world.rate_ledger().shortfall[0].0, 0);
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
    run(&mut world, 2, 1);
    assert_eq!(
        world
            .settlements()
            .store(site)
            .and_then(|store| store.quantity(GOOD)),
        Some(Fix32::MAX),
        "the store saturates at its ceiling and never wraps"
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
    let scaled = world
        .economy_schedule()
        .per_application(Fix32::from_int(100));
    assert_eq!(
        world.rate_ledger().produced[0].0 + world.rate_ledger().spilled[0].0,
        i64::from(scaled.0),
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
    (held, ledger.produced[0].0, ledger.spent[0].0, shortfalls)
}

#[test]
fn what_a_site_produced_minus_what_it_spent_is_what_it_holds() {
    let (held, produced, spent, shortfalls) = long_run(1);
    assert_eq!(
        held,
        produced - spent,
        "the stores started empty, so the holding is the net of the ledger"
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
