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
    unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    ..WorldConfig::DEFAULT
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

/// Answers whether the storm log of the last step names the unit.
///
/// A storm ends a unit that stands in the open, and it obeys no rule of the
/// need pass.[^1] Every test below drops the units this answers for, and no
/// others. A test that dropped every unit the world no longer holds would pass
/// against a run in which the shortage ended them all, and it would then
/// measure the fixture.
///
/// # References
///
/// [^1]: Findings register, FND-725. `docs/FINDINGS.md`
fn a_storm_took(world: &World, unit: Entity) -> bool {
    world
        .units_lost_to_storms()
        .iter()
        .any(|lost| lost.unit == unit.to_bits())
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

/// The condition is what a watcher reads.
///
/// A watcher that read the accumulator would hold the bound of the rule a
/// second time.
///
/// **A storm ends a unit that stands in the open.** The run collects the storm
/// log of every step and drops the units it names. It counts what it read on
/// each side, and it refuses a count of zero, so a run that lost a whole side
/// fails rather than passes.
#[test]
fn a_watcher_reads_the_condition_by_name() {
    let mut world = World::new(CONFIG).expect("the extent must describe a world");
    let fixture = build(&mut world, FAR_BOUND);
    for unit in fixture.fed.iter().chain(&fixture.hungry) {
        assert_eq!(
            world.unit_condition(*unit),
            Some(NeedCondition::Fed),
            "a unit arrives fed"
        );
    }
    let mut taken: Vec<u64> = Vec::new();
    for _ in 0..16 {
        world.step(4).expect("the step must run");
        taken.extend(world.units_lost_to_storms().iter().map(|lost| lost.unit));
    }

    let mut short_read = 0usize;
    for unit in &fixture.hungry {
        if taken.contains(&unit.to_bits()) {
            continue;
        }
        assert_eq!(
            world.unit_condition(*unit),
            Some(NeedCondition::Short),
            "a unit that failed its draw is in a condition a watcher can name"
        );
        short_read += 1;
    }
    assert!(
        short_read > 0,
        "the storms took every hungry unit, so the run read no condition"
    );

    let mut fed_read = 0usize;
    for unit in &fixture.fed {
        if taken.contains(&unit.to_bits()) {
            continue;
        }
        assert_eq!(world.unit_condition(*unit), Some(NeedCondition::Fed));
        fed_read += 1;
    }
    assert!(
        fed_read > 0,
        "the storms took every fed unit, so the run read no condition"
    );
    assert!(world.check_invariants());
}

/// Both directions of the rule.
///
/// A rule that only rose would pass a test that watched a deficit grow, and a
/// unit would then never recover.
///
/// **The run watches the whole hungry set and not one unit.** A storm ends a
/// unit that stands in the open, and this run steps the world for hundreds of
/// ticks.[^1] The run drops a unit on the tick the storm log names it, and it
/// refuses to go on with an empty set. It reads the deficit of a unit it holds
/// on every tick, so a unit that leaves the world without the storm log naming
/// it fails on that tick and names the tick.
///
/// # References
///
/// [^1]: Findings register, FND-725. `docs/FINDINGS.md`
#[test]
fn the_condition_gets_worse_while_the_shortage_lasts_and_recovers_when_it_ends() {
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
    let mut watched: Vec<(Entity, Fix32)> = fixture
        .hungry
        .iter()
        .map(|unit| {
            (
                *unit,
                world.soldiers().deficit(*unit).expect("the unit lives"),
            )
        })
        .collect();

    let mut worse = 0;
    for tick in 0..24 {
        world.step(4).expect("the step must run");
        watched.retain(|(unit, _)| !a_storm_took(&world, *unit));
        assert!(
            !watched.is_empty(),
            "the storms took every hungry unit by tick {tick}, so the run watches nothing"
        );
        for (unit, last) in &mut watched {
            let now = world
                .soldiers()
                .deficit(*unit)
                .unwrap_or_else(|| panic!("unit {unit:?} left the world on tick {tick}"));
            if now > *last {
                worse += 1;
            }
            *last = now;
        }
    }
    assert!(worse > 0, "the deficit never rose, so the shortage did not");
    for (unit, _) in &watched {
        assert_eq!(world.unit_condition(*unit), Some(NeedCondition::Short));
    }

    // The shortage ends. Every site that produced nothing now produces more
    // than its people eat.
    for site in world.settlements().iter().collect::<Vec<Entity>>() {
        world
            .set_production_rate(site, FOOD, Fix32::from_int(4))
            .expect("the rate is at or above zero");
    }
    let peak = watched.clone();
    let mut better = 0;
    // The recovery takes off a fixed amount at each application, so a
    // deficit that took a dozen applications to build takes more than a
    // dozen to clear. The count is what the rates of this test give, and
    // the test asserts the whole way back to fed.
    for tick in 0..240 {
        world.step(4).expect("the step must run");
        watched.retain(|(unit, _)| !a_storm_took(&world, *unit));
        assert!(
            !watched.is_empty(),
            "the storms took every hungry unit by tick {tick} of the recovery"
        );
        for (unit, last) in &mut watched {
            let now = world
                .soldiers()
                .deficit(*unit)
                .unwrap_or_else(|| panic!("unit {unit:?} left the world on tick {tick}"));
            if now < *last {
                better += 1;
            }
            *last = now;
        }
    }
    assert!(
        better > 0,
        "the deficit never fell after the shortage ended"
    );
    for (unit, last) in &watched {
        let top = peak
            .iter()
            .find(|(other, _)| other == unit)
            .expect("a watched unit was watched before the shortage ended")
            .1;
        assert!(last < &top, "the deficit of unit {unit:?} did not recover");
        assert_eq!(
            world.unit_condition(*unit),
            Some(NeedCondition::Fed),
            "a unit that recovered its need carries no deficit"
        );
    }
    assert!(world.check_invariants());
}

/// A shortage that lasts long enough ends the unit.
///
/// **The fixture starves some units and not others.** A fixture that starved
/// every unit would pass this test with a rule that ends every unit. A storm
/// also ends a unit that stands in the open, so the run collects the storm log
/// of every step and drops the units it names, and no others. It counts what it
/// read on each side and refuses a count of zero.
#[test]
fn a_shortage_that_lasts_long_enough_ends_the_unit() {
    let mut world = World::new(CONFIG).expect("the extent must describe a world");
    let fixture = build(&mut world, NEAR_BOUND);
    let mut ended = Vec::new();
    let mut taken: Vec<u64> = Vec::new();
    for _ in 0..FRAMES {
        world.step(4).expect("the step must run");
        ended.extend_from_slice(world.starved_log());
        taken.extend(world.units_lost_to_storms().iter().map(|lost| lost.unit));
        assert!(world.check_invariants());
    }
    assert!(!ended.is_empty(), "the shortage ended nobody");

    let mut starved_read = 0usize;
    for unit in &fixture.hungry {
        if taken.contains(&unit.to_bits()) {
            continue;
        }
        assert!(
            !world.soldiers().contains(*unit),
            "a unit the shortage starved must be gone"
        );
        assert!(
            ended.iter().any(|event| event.unit == unit.to_bits()),
            "the end of a unit must reach the log"
        );
        starved_read += 1;
    }
    assert!(
        starved_read > 0,
        "the storms took every hungry unit, so the run read no starvation"
    );

    let mut fed_read = 0usize;
    for unit in &fixture.fed {
        if taken.contains(&unit.to_bits()) {
            continue;
        }
        assert!(
            world.soldiers().contains(*unit),
            "a unit that eats must survive"
        );
        assert_eq!(world.unit_condition(*unit), Some(NeedCondition::Fed));
        fed_read += 1;
    }
    assert!(
        fed_read > 0,
        "the storms took every fed unit, so the run read no survivor"
    );
}

/// The same fixture, the same frames, two bounds.
///
/// A bound written into a kernel would give the same answer twice.
///
/// **A storm ends a unit in either world, and the two worlds need not lose the
/// same units.** The run collects both storm logs and skips a unit that either
/// log names. It counts the units it compared, and it refuses a count of zero.
#[test]
fn the_bound_is_a_parameter_and_not_a_constant() {
    let mut near = World::new(CONFIG).expect("the extent must describe a world");
    let hungry = build(&mut near, NEAR_BOUND).hungry;
    let mut far = World::new(CONFIG).expect("the extent must describe a world");
    build(&mut far, FAR_BOUND);
    let mut taken: Vec<u64> = Vec::new();
    for _ in 0..FRAMES {
        near.step(4).expect("the step must run");
        far.step(4).expect("the step must run");
        taken.extend(near.units_lost_to_storms().iter().map(|lost| lost.unit));
        taken.extend(far.units_lost_to_storms().iter().map(|lost| lost.unit));
    }
    let mut compared = 0usize;
    for unit in &hungry {
        if taken.contains(&unit.to_bits()) {
            continue;
        }
        assert!(!near.soldiers().contains(*unit), "the near bound must end");
        assert!(far.soldiers().contains(*unit), "the far bound must not end");
        compared += 1;
    }
    assert!(
        compared > 0,
        "the storms took every hungry unit, so the run compared no bound"
    );
}

#[test]
fn a_dead_identity_never_resolves_to_the_unit_spawned_next_in_its_slot() {
    // The generation advances when the engine frees the slot. A unit that
    // starves must never hand its identity to the unit spawned next in that
    // slot.
    let mut world = World::new(CONFIG).expect("the extent must describe a world");
    // **The fixture turns growth off.** This test is about the arena and the
    // shortage. A world that grew would take the slots the shortage freed
    // before the spawn below could, and the test would then measure growth.
    world.set_birth_chance(Fix32::ZERO);
    let fixture = build(&mut world, NEAR_BOUND);
    let ground = open_ground(&world);
    let mut starved = Vec::new();
    for _ in 0..FRAMES {
        world.step(4).expect("the step must run");
        starved.extend_from_slice(world.starved_log());
    }
    let starved: Vec<u64> = starved.iter().map(|event| event.unit).collect();
    let dead = *fixture
        .hungry
        .iter()
        .find(|unit| starved.contains(&unit.to_bits()))
        .expect("the shortage must end a hungry unit, and a storm death proves nothing here");
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
