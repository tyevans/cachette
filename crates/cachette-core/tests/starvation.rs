//! Starvation: the condition a shortage puts a unit in, and the end of a
//! unit that the shortage keeps.
//!
//! Every test here drives the engine. The world founds sites, gives them
//! units, and steps. A test that built the death plane and scanned it
//! directly would prove that the plane works and not that anything reaches
//! it.[^1]
//!
//! The fixture is built to starve some units and to feed others. Half of
//! its sites produce nothing and hold a store that empties, and half
//! produce more than their people eat. Each test that needs both cases
//! asserts that its fixture produced both. A fixture that starved every
//! unit would pass a test that ended every unit, and the defect it hides is
//! the one that ends a unit the shortage never touched.[^2]
//!
//! # References
//!
//! [^1]: Testing rules, section 5. `.claude/rules/testing.md`
//! [^2]: Testing rules, section 2a, and the findings register, FND-051. `.claude/rules/testing.md`

use cachette_core::cohort::{NeedCondition, NeedRule, NEED_FULL};
use cachette_core::resource::ResourceKind;
use cachette_core::site::CommodityId;
use cachette_core::{Axial, Entity, FactionId, Fix32, World, WorldConfig};

/// The commodity that a unit eats. The set holds one.
const FOOD: CommodityId = CommodityId(0);

/// The thread counts that the equivalence test runs at.
const THREAD_COUNTS: [usize; 3] = [1, 2, 12];

/// The world that every fixture below stands on.
const CONFIG: WorldConfig = WorldConfig {
    width: 48,
    height: 48,
    seed: 42,
    faction_count: 2,
};

/// The period of the economy in the fixtures.
///
/// A short period makes a run of a few frames reach several applications.
/// The period is a parameter of the schedule and not a constant of a
/// kernel.
const PERIOD: u32 = 2;

/// How many sites a fixture founds.
///
/// The count is above the thread count of the equivalence test, so a run at
/// twelve threads marks bits in more than one word of the death plane.
const SITES: usize = 24;

/// How many units a fixture gives to each site.
const PER_SITE: usize = 3;

/// The bound that ends a unit in a run of a few dozen frames.
///
/// The value is a parameter of the rule. A test states it, and no kernel
/// holds one.
const NEAR_BOUND: Fix32 = NEED_FULL;

/// A bound that no run of this file reaches.
///
/// A test that watches a deficit rise and fall needs the unit to stay
/// alive while it watches.
const FAR_BOUND: Fix32 = Fix32(NEED_FULL.0 * 1024);

/// How many frames a fixture runs for the death to arrive.
const FRAMES: usize = 48;

/// Returns the open ground of a world, in tile order.
fn open_ground(world: &World) -> Vec<Axial> {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| world.admits_a_unit(*address))
        .collect()
}

/// What one fixture built.
struct Fixture {
    /// The units that belong to a site which feeds them.
    fed: Vec<Entity>,
    /// The units that belong to a site which cannot feed them.
    hungry: Vec<Entity>,
}

/// Builds a world in which half the units starve and half do not.
///
/// The world is not the world of the demonstration binary. That world is
/// chosen to look right, and every unit in it eats.[^1] This one is built
/// the other way round: every second site produces nothing and starts with
/// a store that its people empty in a few applications, and the rest
/// produce more than their people eat.
///
/// The bound is an argument, because the bound is what the tests vary.
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
fn build(world: &mut World, bound: Fix32) -> Fixture {
    world
        .set_economy_schedule(PERIOD, 0)
        .expect("the period is inside the range");
    let rule = NeedRule::DEFAULT;
    world.set_need_rule(
        NeedRule::new(
            rule.decay(),
            rule.ration(),
            rule.threshold(),
            rule.recovery(),
            bound,
        )
        .expect("every rate is at or above zero"),
    );
    let ground = open_ground(world);
    assert!(
        ground.len() > SITES * 4,
        "the world holds only {} open tiles",
        ground.len()
    );

    let mut fixture = Fixture {
        fed: Vec::new(),
        hungry: Vec::new(),
    };
    for index in 0..SITES {
        let place = ground[index * 3];
        let site = world
            .found_settlement(place, FactionId(0))
            .expect("the tile is free");
        let mut members = Vec::new();
        for ordinal in 0..PER_SITE {
            let unit = world
                .spawn_soldier(ground[index * 3 + 1], FactionId((ordinal % 2) as u16))
                .expect("the ground admits a unit");
            assert!(world.set_home_site(unit, Some(site)));
            members.push(unit);
        }
        if index % 2 == 0 {
            world
                .set_production_rate(site, FOOD, Fix32::from_int(1))
                .expect("the rate is at or above zero");
            fixture.fed.append(&mut members);
        } else {
            world
                .set_settlement_store(site, FOOD, Fix32(NEED_FULL.0 / 2))
                .expect("the commodity is in the set");
            fixture.hungry.append(&mut members);
        }
    }
    assert!(!fixture.fed.is_empty() && !fixture.hungry.is_empty());
    fixture
}

#[test]
fn a_watcher_reads_the_condition_by_name() {
    // The condition is what a watcher reads. A watcher that read the
    // accumulator would hold the bound of the rule a second time.
    let mut world = World::new(CONFIG).expect("the extent must describe a world");
    let fixture = build(&mut world, FAR_BOUND);
    for unit in fixture.fed.iter().chain(&fixture.hungry) {
        assert_eq!(
            world.unit_condition(*unit),
            Some(NeedCondition::Fed),
            "a unit arrives fed"
        );
    }
    for _ in 0..16 {
        world.step(4).expect("the step must run");
    }
    for unit in &fixture.hungry {
        assert_eq!(
            world.unit_condition(*unit),
            Some(NeedCondition::Short),
            "a unit that failed its draw is in a condition a watcher can name"
        );
    }
    for unit in &fixture.fed {
        assert_eq!(world.unit_condition(*unit), Some(NeedCondition::Fed));
    }
    assert!(world.check_invariants());
}

#[test]
fn the_condition_gets_worse_while_the_shortage_lasts_and_recovers_when_it_ends() {
    // Both directions. A rule that only rose would pass a test that
    // watched a deficit grow, and a unit would then never recover.
    let mut world = World::new(CONFIG).expect("the extent must describe a world");
    let fixture = build(&mut world, FAR_BOUND);
    // The ration of this test is above the decay, so a unit that eats
    // climbs back over the threshold. Under the default rule the ration
    // equals the decay, a need that reached zero holds at zero, and the
    // deficit of a unit that eats again never falls.[^1]
    //
    // [^1]: Findings register, FND-089. `docs/FINDINGS.md`
    let rule = world.need_rule();
    world.set_need_rule(
        NeedRule::new(
            rule.decay(),
            Fix32(rule.decay().0 * 2),
            rule.threshold(),
            rule.recovery(),
            rule.bound(),
        )
        .expect("every rate is at or above zero"),
    );
    let watched = fixture.hungry[0];

    let mut worse = 0;
    let mut last = world.soldiers().deficit(watched).expect("the unit lives");
    for _ in 0..24 {
        world.step(4).expect("the step must run");
        let now = world.soldiers().deficit(watched).expect("the unit lives");
        if now > last {
            worse += 1;
        }
        last = now;
    }
    assert!(worse > 0, "the deficit never rose, so the shortage did not");
    assert_eq!(world.unit_condition(watched), Some(NeedCondition::Short));

    // The shortage ends. Every site that produced nothing now produces more
    // than its people eat.
    for site in world.settlements().iter().collect::<Vec<Entity>>() {
        world
            .set_production_rate(site, FOOD, Fix32::from_int(4))
            .expect("the rate is at or above zero");
    }
    let peak = last;
    let mut better = 0;
    // The recovery takes off a fixed amount at each application, so a
    // deficit that took a dozen applications to build takes more than a
    // dozen to clear. The count is what the rates of this test give, and
    // the test asserts the whole way back to fed.
    for _ in 0..240 {
        world.step(4).expect("the step must run");
        let now = world.soldiers().deficit(watched).expect("the unit lives");
        if now < last {
            better += 1;
        }
        last = now;
    }
    assert!(
        better > 0,
        "the deficit never fell after the shortage ended"
    );
    assert!(last < peak, "the deficit did not recover");
    assert_eq!(
        world.unit_condition(watched),
        Some(NeedCondition::Fed),
        "a unit that recovered its need carries no deficit"
    );
    assert!(world.check_invariants());
}

#[test]
fn a_shortage_that_lasts_long_enough_ends_the_unit() {
    let mut world = World::new(CONFIG).expect("the extent must describe a world");
    let fixture = build(&mut world, NEAR_BOUND);
    let mut ended = Vec::new();
    for _ in 0..FRAMES {
        world.step(4).expect("the step must run");
        ended.extend_from_slice(world.starved_log());
        assert!(world.check_invariants());
    }
    assert!(!ended.is_empty(), "the shortage ended nobody");
    for unit in &fixture.hungry {
        assert!(
            !world.soldiers().contains(*unit),
            "a unit the shortage starved must be gone"
        );
        assert!(
            ended.iter().any(|event| event.unit == unit.to_bits()),
            "the end of a unit must reach the log"
        );
    }
    // The fixture starves some units and not others. A fixture that starved
    // every unit would pass this test with a rule that ends every unit.
    for unit in &fixture.fed {
        assert!(
            world.soldiers().contains(*unit),
            "a unit that eats must survive"
        );
        assert_eq!(world.unit_condition(*unit), Some(NeedCondition::Fed));
    }
}

#[test]
fn the_bound_is_a_parameter_and_not_a_constant() {
    // The same fixture, the same frames, two bounds. A bound written into a
    // kernel would give the same answer twice.
    let mut near = World::new(CONFIG).expect("the extent must describe a world");
    let hungry = build(&mut near, NEAR_BOUND).hungry;
    let mut far = World::new(CONFIG).expect("the extent must describe a world");
    build(&mut far, FAR_BOUND);
    for _ in 0..FRAMES {
        near.step(4).expect("the step must run");
        far.step(4).expect("the step must run");
    }
    for unit in &hungry {
        assert!(!near.soldiers().contains(*unit), "the near bound must end");
        assert!(far.soldiers().contains(*unit), "the far bound must not end");
    }
}

#[test]
fn a_dead_identity_never_resolves_to_the_unit_spawned_next_in_its_slot() {
    // The generation advances when the engine frees the slot. A unit that
    // starves must never hand its identity to the unit spawned next in that
    // slot.
    let mut world = World::new(CONFIG).expect("the extent must describe a world");
    let fixture = build(&mut world, NEAR_BOUND);
    let ground = open_ground(&world);
    for _ in 0..FRAMES {
        world.step(4).expect("the step must run");
    }
    let dead = fixture.hungry[0];
    assert!(!world.soldiers().contains(dead));
    assert_eq!(world.unit_condition(dead), None);

    // The arena reuses a freed slot, so the next spawn takes one of the
    // slots the shortage freed. The test asserts that it took one, because
    // a spawn into a fresh slot would prove nothing.
    let mut reused = None;
    for address in ground.iter().take(fixture.hungry.len()) {
        let fresh = world
            .spawn_soldier(*address, FactionId(0))
            .expect("the ground admits a unit");
        if world.soldiers().slot_of(fresh) == Some(dead.index()) {
            reused = Some(fresh);
            break;
        }
    }
    let fresh = reused.expect("the arena must give a freed slot back");
    assert_ne!(
        fresh, dead,
        "the new unit took the identity of the dead one"
    );
    assert!(fresh.generation() > dead.generation());
    assert!(
        !world.soldiers().contains(dead),
        "the dead identity resolved to the unit spawned next in its slot"
    );
    assert_eq!(world.unit_condition(dead), None);
    assert_eq!(world.unit_condition(fresh), Some(NeedCondition::Fed));
}

#[test]
fn the_conservation_sum_balances_after_many_deaths() {
    // A determinism test cannot see a broken invariant: a rule that loses
    // the same load on every run repeats perfectly. The conservation sum is
    // what fails instead.[^1]
    //
    // [^1]: Findings register, FND-048. `docs/FINDINGS.md`
    let mut world = World::new(CONFIG).expect("the extent must describe a world");
    world
        .set_choice_schedule(0)
        .expect("the exponent is inside the range");
    let fixture = build(&mut world, NEAR_BOUND);
    for unit in fixture.fed.iter().chain(&fixture.hungry) {
        for kind in ResourceKind::ALL {
            world.order_gather(*unit, kind);
        }
    }

    // The units must carry something before they die, or the death accounts
    // for nothing and the test measures an empty sum.
    let mut carried_before = 0u64;
    for _ in 0..12 {
        world.step(4).expect("the step must run");
    }
    for unit in &fixture.hungry {
        if let Some(load) = world.soldier_carry(*unit) {
            for kind in ResourceKind::ALL {
                carried_before += u64::from(load.of(kind).0);
            }
        }
    }
    assert!(
        carried_before > 0,
        "no unit carried anything, so the fixture cannot test the account"
    );

    let mut deaths = 0;
    for _ in 0..FRAMES {
        world.step(4).expect("the step must run");
        deaths += world.starved_log().len();
        assert!(
            world.check_invariants(),
            "the conservation sum stopped balancing"
        );
    }
    assert!(deaths > 1, "the run ended only {deaths} units");
    let departed: u64 = world.departed_carry().iter().sum();
    assert!(
        departed >= carried_before,
        "what a dead unit carried left the world without a record"
    );
}

#[test]
fn the_same_seed_ends_the_same_units_in_the_same_order_at_every_thread_count() {
    // The plane is written in parallel and the scan of it is ordered. Two
    // runs that differ only in the thread count must end the same units, in
    // the same order, and leave the same world.
    let mut logs = Vec::new();
    let mut hashes = Vec::new();
    for threads in THREAD_COUNTS {
        let mut world = World::new(CONFIG).expect("the extent must describe a world");
        build(&mut world, NEAR_BOUND);
        let mut log = Vec::new();
        for _ in 0..FRAMES {
            world.step(threads).expect("the step must run");
            log.extend_from_slice(world.starved_log_bytes());
        }
        assert!(
            !log.is_empty(),
            "the run ended nobody, so it proves nothing"
        );
        logs.push(log);
        hashes.push(world.state_hash().finish());
    }
    for index in 1..logs.len() {
        assert_eq!(
            logs[0], logs[index],
            "the deaths differ between {} threads and {} threads",
            THREAD_COUNTS[0], THREAD_COUNTS[index]
        );
        assert_eq!(hashes[0], hashes[index], "the worlds differ");
    }
}
