//! A cohort that cannot feed everybody feeds a keyed subset whole.
//!
//! The pass used to divide the share of a cohort by its headcount, so every
//! unit gained the same amount. Every other input to a need is the same for
//! every unit of a cohort, so the units were numerically identical and they
//! crossed the death bound on one tick. A whole faction left the world
//! together.[^1]
//!
//! The pass now serves whole rations to as many units as the share covers, and
//! a keyed draw names which ones.[^2] A cohort therefore loses units until its
//! headcount reaches what its supply carries, and the survivors are fed.
//!
//! Every test here drives the step. None calls the consumption pass.[^3]
//!
//! **The fixture is built for these tests.** It does not copy the world of the
//! demonstration binary.[^4] It holds a site whose store covers a stated
//! fraction of what its people ask for, because the whole behaviour under test
//! is what happens at that fraction, and the demonstration world supplies
//! whatever the ground happened to generate.
//!
//! # References
//!
//! [^1]: Findings register, FND-318. `docs/FINDINGS.md`
//! [^2]: ADR-0106, a cohort serves whole rations to a keyed subset, never an equal share to everybody, decisions D1 and D2. `docs/adrs/draft/adr-0106-a-cohort-serves-whole-rations-to-a-keyed-subset.md`
//! [^3]: Testing rules, section 5. `.claude/rules/testing.md`
//! [^4]: Testing rules, section 2a. `.claude/rules/testing.md`

use cachette_core::cohort::{NeedRule, NEED_FULL};
use cachette_core::{Axial, CommodityId, Entity, FactionId, Fix32, World, WorldConfig};

/// The extent of every fixture world.
const EXTENT: u32 = 64;

/// The seed of every fixture world.
const SEED: u64 = 7;

/// The commodity that the ration draws against.
const FOOD: CommodityId = cachette_core::WORK_COMMODITY[0];

/// The number of units the fixture seats at one site.
const GROUP: u32 = 32;

/// The thread counts that the equivalence test runs at.
const THREAD_COUNTS: [usize; 3] = [1, 2, 12];

/// The rule the fixture runs. Every rate is stated here so that a test names
/// the arithmetic it depends on rather than reading it from a default.
fn rule() -> NeedRule {
    NeedRule::new(
        Fix32(NEED_FULL.0 / 4),
        Fix32(NEED_FULL.0 / 4),
        Fix32(NEED_FULL.0 / 2),
        Fix32(NEED_FULL.0 / 16),
        Fix32(NEED_FULL.0 * 2),
    )
    .expect("every rate is at or above zero")
}

/// Builds a world with one site, a group homed to it, and a store that covers
/// the given fraction of one application of the ration.
///
/// The store is written directly, so the test names the shortage rather than
/// arranging a ground that produces one.
fn short_cohort(fed: u32) -> (World, Entity, Vec<Entity>) {
    short_cohort_under(rule(), fed)
}

/// The rule the key tests run, which nobody starves under.
///
/// The key tests read which unit ate on each application, and a unit whose
/// need has already reached zero reads the same as a fed one, because both
/// hold what they held. The rates here are small enough that no unit reaches
/// zero inside the frames a key test drives, and the bound is out of reach so
/// nobody leaves the world while the test is counting.
fn gentle_rule() -> NeedRule {
    NeedRule::new(
        Fix32(NEED_FULL.0 / 64),
        Fix32(NEED_FULL.0 / 64),
        Fix32(NEED_FULL.0 / 2),
        Fix32(NEED_FULL.0 / 64),
        Fix32::MAX,
    )
    .expect("every rate is at or above zero")
}

/// Builds the fixture under a stated rule.
fn short_cohort_under(rule: NeedRule, fed: u32) -> (World, Entity, Vec<Entity>) {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: SEED,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    world.set_need_rule(rule);
    world
        .set_economy_schedule(1, 0)
        .expect("the period is inside the range");

    let place = open_tile(&world);
    let site = world
        .found_settlement(place, FactionId(0))
        .expect("the ground admits a settlement");
    let mut units = Vec::new();
    for _ in 0..GROUP {
        let unit = world
            .spawn_soldier(place, FactionId(0))
            .expect("the tile admits the unit");
        assert!(world.set_home_site(unit, Some(site)));
        units.push(unit);
    }
    // **The site earns whole rations for exactly `fed` of the group on each
    // tick, and it starts with none.** A store written once would empty after
    // one application and the whole cohort would starve, which measures a
    // world with no food rather than a world with too little.
    let supply = Fix32(rule.ration().0 * fed as i32);
    world
        .set_production_rate(site, FOOD, supply)
        .expect("the rate is at or above zero and the commodity is in the set");
    (world, site, units)
}

/// Returns the first address whose ground admits a unit.
fn open_tile(world: &World) -> Axial {
    let grid = world.grid();
    for index in 0..grid.tile_count() {
        let address = Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32);
        if world.admits_a_unit(address) {
            return address;
        }
    }
    panic!("the fixture holds no open ground");
}

/// Returns the need of every live unit of a group, in the order given.
fn needs(world: &World, units: &[Entity]) -> Vec<Fix32> {
    units
        .iter()
        .filter_map(|unit| world.soldiers().need(*unit))
        .collect()
}

/// Returns how many distinct values a list holds.
fn distinct(values: &[Fix32]) -> usize {
    let mut seen: Vec<Fix32> = Vec::new();
    for value in values {
        if !seen.contains(value) {
            seen.push(*value);
        }
    }
    seen.len()
}

#[test]
fn two_units_of_one_short_cohort_hold_different_needs() {
    // This is the whole claim. An equal split gives one value to every unit of
    // a cohort, and identical units cross the death bound together.[^1]
    //
    // [^1]: Findings register, FND-318. `docs/FINDINGS.md`
    let (mut world, _, units) = short_cohort(GROUP / 2);
    world.step(1).expect("the step must run");

    let after = needs(&world, &units);
    assert_eq!(after.len(), GROUP as usize, "the fixture lost a unit");
    assert!(
        distinct(&after) > 1,
        "every unit of the cohort holds one need, so the cohort still has the cliff: {:?}",
        after.first()
    );
}

#[test]
fn a_cohort_that_feeds_everybody_leaves_nobody_short() {
    // The contrast the test above needs. A cohort whose store covers every
    // ration serves every unit whole, so one value is the right answer there.
    let (mut world, _, units) = short_cohort(GROUP);
    world.step(1).expect("the step must run");

    let after = needs(&world, &units);
    assert_eq!(
        distinct(&after),
        1,
        "a cohort that fed everybody gave two answers: {after:?}"
    );
}

#[test]
fn a_short_cohort_loses_part_of_itself_and_keeps_the_rest() {
    let (mut world, _, units) = short_cohort(GROUP / 2);
    for _ in 0..400 {
        world.step(1).expect("the step must run");
    }

    let alive = units
        .iter()
        .filter(|unit| world.soldiers().need(**unit).is_some())
        .count();
    assert!(
        alive > 0,
        "the whole cohort died, so the cliff is still there"
    );
    assert!(
        alive < GROUP as usize,
        "the cohort lost nobody, so the fixture was never short"
    );
}

/// Returns the index of every unit that ate on the last application.
///
/// A unit that ate holds the need it had, because the ration equals the decay
/// in the rule the key tests use. Every other unit fell by the decay.
fn who_ate(before: &[Fix32], after: &[Fix32]) -> Vec<usize> {
    before
        .iter()
        .zip(after)
        .enumerate()
        .filter(|(_, (was, now))| now >= was)
        .map(|(index, _)| index)
        .collect()
}

#[test]
fn the_served_set_depends_on_the_frame() {
    // The draw is keyed on the frame, so the cohort serves a different set on
    // the next application. Without the frame in the key the same units would
    // eat every time, and the cohort would hold a caste rather than a
    // shortage.[^1]
    //
    // **The first version of this test passed with the frame taken out of the
    // key.** It compared each unit against the lowest need in the group, and
    // that floor moves for reasons that have nothing to do with the draw, so
    // two readings differed whatever the key held. The reading here is the set
    // of units whose need did not fall, which is the served set itself.[^2]
    //
    // [^1]: ADR-0106, a cohort serves whole rations to a keyed subset, never an equal share to everybody, decision D2. `docs/adrs/draft/adr-0106-a-cohort-serves-whole-rations-to-a-keyed-subset.md`
    // [^2]: Findings register, FND-318. `docs/FINDINGS.md`
    const FRAMES: usize = 12;
    // The site earns nothing, so the store holds exactly what the line below
    // writes into it and the draw is the only thing that decides who eats.
    let (mut world, site, units) = short_cohort_under(gentle_rule(), 0);
    let mut served = Vec::new();
    for _ in 0..FRAMES {
        // **The store is written before each frame rather than earned.** The
        // rate pass and the draw run on one schedule and the store carries
        // what a frame did not spend, so the supply of one frame is not
        // exactly the supply of the next. This test is about which units the
        // draw names, so the fixture holds the supply still.
        world
            .set_settlement_store(site, FOOD, gentle_rule().ration())
            .expect("the commodity is inside the set");
        let before = needs(&world, &units);
        world.step(1).expect("the step must run");
        let after = needs(&world, &units);
        let who = who_ate(&before, &after);
        assert!(!who.is_empty(), "the fixture served nobody on a frame");
        served.push(who);
    }
    assert!(
        served.iter().any(|who| *who != served[0]),
        "the same units ate on all {FRAMES} applications, so the frame is not \
         in the key: {served:?}"
    );
}

#[test]
fn a_cohort_serves_exactly_as_many_rations_as_its_share_covered() {
    // **This is the property a per-unit draw did not have.** Giving each unit
    // an independent chance makes the number that eats vary around the count
    // the store paid for, so a cohort whose share covered one ration served
    // two units on one application and none on another. The store had paid for
    // one, and the model created food.[^1] A rotation is a bijection, so the
    // count is exact.[^2]
    //
    // [^1]: Findings register, FND-318. `docs/FINDINGS.md`
    // [^2]: ADR-0106, a cohort serves whole rations to a keyed subset, never an equal share to everybody, decision D2. `docs/adrs/draft/adr-0106-a-cohort-serves-whole-rations-to-a-keyed-subset.md`
    const FRAMES: usize = 24;
    for covered in [1u32, 3, 7, GROUP / 2] {
        // The site earns nothing, so the store holds exactly what the line
        // below writes and the served count is a stated number.
        let (mut world, site, units) = short_cohort_under(gentle_rule(), 0);
        let supply = Fix32(gentle_rule().ration().0 * covered as i32);
        for frame in 0..FRAMES {
            world
                .set_settlement_store(site, FOOD, supply)
                .expect("the commodity is inside the set");
            let before = needs(&world, &units);
            world.step(1).expect("the step must run");
            let after = needs(&world, &units);
            assert_eq!(
                who_ate(&before, &after).len(),
                covered as usize,
                "a share covering {covered} rations served another number on \
                 frame {frame}"
            );
        }
    }
}

#[test]
fn a_short_cohort_gives_one_answer_at_every_thread_count() {
    let mut answers = Vec::new();
    for threads in THREAD_COUNTS {
        let (mut world, _, units) = short_cohort(GROUP / 2);
        for _ in 0..40 {
            world.step(threads).expect("the step must run");
        }
        let alive = units
            .iter()
            .filter(|unit| world.soldiers().need(**unit).is_some())
            .count();
        assert!(
            alive < GROUP as usize,
            "the run at {threads} threads lost nobody, so the comparison reads \
             worlds in which the shortage never bit"
        );
        answers.push((alive, world.state_hash()));
    }
    assert!(
        answers.windows(2).all(|pair| pair[0] == pair[1]),
        "the shortage depends on the thread count: {answers:?}"
    );
}

#[test]
fn every_account_balances_while_a_cohort_is_short() {
    let (mut world, _, _) = short_cohort(GROUP / 2);
    for _ in 0..80 {
        world.step(1).expect("the step must run");
        assert!(
            world.check_invariants(),
            "an account stopped balancing at tick {}",
            world.tick().0
        );
    }
}
