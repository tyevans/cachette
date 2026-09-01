//! Consumption: the need of a unit, and the pooled draw by cohort.
//!
//! The tests here drive the engine. The world founds sites, gives them
//! units, and steps. A test that built the kernel and drove it directly
//! would prove that the kernel works and not that anything reaches it.[^1]
//!
//! One test calls the draw kernel on its own. It asserts the exactness of
//! the split, and the split is the return value of the kernel rather than a
//! state that the world keeps.
//!
//! The fixture is built to hold a site in deficit and a site in surplus.
//! Each test that needs those cases asserts that its fixture produced
//! them.[^2]
//!
//! # References
//!
//! [^1]: Testing rules, section 5. `.claude/rules/testing.md`
//! [^2]: Findings register, FND-051. `docs/FINDINGS.md`

use cachette_core::cohort::{self, CohortTable, NeedRule, COHORTS_PER_SITE, NEED_FULL};
use cachette_core::site::{CommodityId, SettlementArena};
use cachette_core::types::Accum;
use cachette_core::{Axial, Entity, FactionId, Fix32, Grid, World, WorldConfig};

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
/// twelve threads fills more than one output slot and the order of the
/// rationed log can differ.
const SITES: usize = 24;

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
    /// The sites that hold more than their people eat.
    surplus: Vec<Entity>,
    /// The sites that hold less than their people eat.
    deficit: Vec<Entity>,
    /// Every unit that the fixture spawned.
    units: Vec<Entity>,
}

/// Builds a world that holds a site in deficit and a site in surplus.
///
/// The world is not the world of the demonstration binary. That world is
/// chosen to look right, and it produces no place that runs out.[^1] This
/// one is built the other way round: every second site produces nothing and
/// starts with a store that its people empty in a few applications, and the
/// rest produce more than their people eat.
///
/// Each site takes units of both factions, so a site holds more than one
/// cohort and the split has something to divide.
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
fn build(world: &mut World) -> Fixture {
    world
        .set_economy_schedule(PERIOD, 0)
        .expect("the period is inside the range");
    let ground = open_ground(world);
    assert!(
        ground.len() > SITES * 4,
        "the world holds only {} open tiles",
        ground.len()
    );

    let mut fixture = Fixture {
        surplus: Vec::new(),
        deficit: Vec::new(),
        units: Vec::new(),
    };
    for index in 0..SITES {
        let place = ground[index * 3];
        let site = world
            .found_settlement(place, FactionId(0))
            .expect("the tile is free");
        // The headcounts differ between the two cohorts of a site, so a
        // split that ignored the headcount would still divide evenly and
        // the test would not see it.
        for ordinal in 0..3 {
            let unit = world
                .spawn_soldier(ground[index * 3 + 1], FactionId((ordinal % 2) as u16))
                .expect("the ground admits a unit");
            assert!(world.set_home_site(unit, Some(site)));
            fixture.units.push(unit);
        }
        if index % 2 == 0 {
            // A site that produces more than its people eat.
            world
                .set_production_rate(site, FOOD, Fix32::from_int(1))
                .expect("the rate is at or above zero");
            fixture.surplus.push(site);
        } else {
            // A site that produces nothing and starts with a small store.
            world
                .set_settlement_store(site, FOOD, Fix32(NEED_FULL.0 / 2))
                .expect("the commodity is in the set");
            fixture.deficit.push(site);
        }
    }
    assert!(!fixture.surplus.is_empty() && !fixture.deficit.is_empty());
    fixture
}

/// Returns what every live store holds of the food commodity.
fn stored(world: &World) -> Accum {
    let mut total = Accum(0);
    for site in world.settlements().iter() {
        let held = world
            .settlements()
            .store(site)
            .and_then(|store| store.quantity(FOOD))
            .expect("a live site holds a store");
        total = Accum(total.0 + i64::from(held.0));
    }
    total
}

#[test]
fn a_need_falls_to_zero_and_stops_there() {
    // The need of a unit at a site that cannot feed it falls by a
    // saturating subtract. A wrapping subtract would take the need through
    // zero and back to the top of the range, which is full satisfaction.
    let mut world = World::new(CONFIG).expect("the extent must describe a world");
    let fixture = build(&mut world);
    let hungry: Vec<Entity> = fixture
        .units
        .iter()
        .copied()
        .filter(|unit| {
            let home = world.soldiers().home(*unit).flatten();
            fixture
                .deficit
                .iter()
                .any(|site| world.settlements().slot_of(*site) == home)
        })
        .collect();
    assert!(!hungry.is_empty(), "the fixture must hold a hungry unit");

    for _ in 0..64 {
        world.step(4).expect("the step must run");
    }
    for unit in &hungry {
        let need = world.soldiers().need(*unit).expect("the unit lives");
        assert_eq!(
            need,
            Fix32::ZERO,
            "the need must stop at zero, and it must never wrap above it"
        );
        let deficit = world.soldiers().deficit(*unit).expect("the unit lives");
        assert!(
            deficit > Fix32::ZERO,
            "a unit at zero need must carry a deficit"
        );
    }
    assert!(world.check_invariants());
}

#[test]
fn a_fed_unit_holds_its_need_and_carries_no_deficit() {
    // The ration equals the decay, so a unit that receives its whole ration
    // holds its need. This is the case that proves the draw feeds anybody:
    // a test that only watched a need fall would pass with no draw at all.
    let mut world = World::new(CONFIG).expect("the extent must describe a world");
    let fixture = build(&mut world);
    let fed: Vec<Entity> = fixture
        .units
        .iter()
        .copied()
        .filter(|unit| {
            let home = world.soldiers().home(*unit).flatten();
            fixture
                .surplus
                .iter()
                .any(|site| world.settlements().slot_of(*site) == home)
        })
        .collect();
    assert!(!fed.is_empty(), "the fixture must hold a fed unit");

    for _ in 0..32 {
        world.step(4).expect("the step must run");
    }
    for unit in &fed {
        assert_eq!(
            world.soldiers().need(*unit),
            Some(NEED_FULL),
            "a unit that receives its whole ration holds its need"
        );
        assert_eq!(world.soldiers().deficit(*unit), Some(Fix32::ZERO));
    }
}

#[test]
fn every_headcount_sums_to_the_live_population() {
    let mut world = World::new(CONFIG).expect("the extent must describe a world");
    let fixture = build(&mut world);
    // A loss must move the count as well as a spawn.
    assert!(world.despawn_soldier(fixture.units[0]));
    for _ in 0..4 {
        world.step(3).expect("the step must run");
    }
    let cohorts = world.cohorts();
    let counted = cohorts.headcount_total().0 + i64::from(cohorts.unattached());
    assert_eq!(
        counted,
        i64::from(world.soldiers().len()),
        "the headcount of every cohort, plus the unattached, is the population"
    );
    assert!(counted > 0, "the fixture must hold units");
    assert!(world.check_invariants());
}

#[test]
fn the_store_falls_by_exactly_what_the_cohorts_received() {
    // Conservation over the world. What the stores held, plus what
    // production put in, minus what the cohorts took, is what they hold.
    // The balance is exact, because every term is a whole number in a
    // 64-bit accumulator.
    let mut world = World::new(CONFIG).expect("the extent must describe a world");
    build(&mut world);
    let opening = stored(&world);
    for _ in 0..24 {
        world.step(4).expect("the step must run");
        // The store account and the store column are two statements of one
        // total, and this check is what fails when they disagree.
        assert!(world.check_invariants());
    }
    let closing = stored(&world);
    let produced = world.rate_ledger().produced[0].0;
    let spent = world.rate_ledger().spent[0].0;
    let taken = world.draw_ledger().granted[0].0;
    assert!(taken > 0, "the fixture must reach a draw");
    assert_eq!(
        closing.0 - opening.0,
        produced - spent - taken,
        "the world must balance to zero"
    );
    assert!(
        world.draw_ledger().unmet[0].0 > 0,
        "the fixture must reach a store that could not serve"
    );
}

#[test]
fn a_store_that_divides_unevenly_splits_exactly() {
    // The kernel returns the split, and the world keeps no copy of it, so
    // this test calls the kernel. The store is one unit short of a number
    // that the demand divides, so the proportional share of every cohort
    // truncates and the remainder must go somewhere.
    let grid = Grid::new(4, 4).expect("a small extent describes a grid");
    let mut arena = SettlementArena::new(grid);
    let site = arena
        .found(Axial::new(0, 0), FactionId(0))
        .expect("the founding must succeed");
    // A store of an odd raw amount, against three units in two cohorts.
    let held = Fix32(100_001);
    arena
        .set_store(site, FOOD, held)
        .expect("the commodity is in the set");

    // Two cohorts at one site: one unit of faction zero, two of faction
    // one. The demand of the site is therefore three rations.
    let homes = [0u32, 0, 0];
    let factions = [FactionId(0), FactionId(1), FactionId(1)];
    let live = [1u8, 1, 1];
    let mut table = CohortTable::new();
    table.rebuild(&homes, &factions, &live, arena.slot_count());
    assert_eq!(table.headcount(0, FactionId(0)), Some(1));
    assert_eq!(table.headcount(0, FactionId(1)), Some(2));

    let ration = Fix32(NEED_FULL.0);
    let pass = cohort::draw(
        cachette_core::types::Tick(1),
        ration,
        FOOD,
        &table,
        arena.store_update(),
        1,
    )
    .expect("the draw must run");

    let demanded = i64::from(ration.0) * 3;
    assert!(
        i64::from(held.0) < demanded,
        "the store must be short, or nothing is split"
    );
    let granted: i64 = pass.shares.iter().map(|share| share.0).sum();
    assert_eq!(
        granted,
        i64::from(held.0),
        "the parts must sum to the whole, with no unit lost and none created"
    );
    assert_eq!(
        pass.ledger.granted[0].0,
        i64::from(held.0),
        "the ledger must agree with the shares"
    );
    // The store fell by exactly what the cohorts received. A split that
    // lost a unit would leave that unit in the store, and a split that
    // created one would take a unit that nobody held.
    assert_eq!(
        arena
            .store(site)
            .and_then(|store| store.quantity(FOOD))
            .map(|quantity| i64::from(quantity.0)),
        Some(i64::from(held.0) - granted)
    );
    // The split is proportional. The larger cohort takes about twice the
    // smaller one, and the remainder goes to the lower row.
    let small = pass.shares[cohort::row_index(0, 0)].0;
    let large = pass.shares[cohort::row_index(0, 1)].0;
    assert!(small > 0 && large > 0, "each cohort takes a share");
    assert_eq!(large - 2 * small, -1, "the remainder goes to the lower row");
    assert_eq!(pass.rationed.len(), 1, "the site could not serve");
}

/// Runs a fixture and returns what the equivalence test compares.
fn run(threads: usize, frames: u64) -> (Vec<u8>, u64, i64, i64) {
    let mut world = World::new(CONFIG).expect("the extent must describe a world");
    build(&mut world);
    let mut bytes = Vec::new();
    for _ in 0..frames {
        world.step(threads).expect("the step must run");
        bytes.extend_from_slice(world.rationed_log_bytes());
    }
    assert!(world.check_invariants());
    (
        bytes,
        world.state_hash().finish(),
        world.draw_ledger().granted[0].0,
        world.draw_ledger().unmet[0].0,
    )
}

#[test]
fn the_draw_is_identical_at_every_thread_count() {
    let expected = run(THREAD_COUNTS[0], 16);
    assert!(
        !expected.0.is_empty(),
        "the fixture must ration somebody, or the comparison reads two empty logs"
    );
    assert!(expected.2 > 0, "the fixture must reach a draw");
    for threads in &THREAD_COUNTS[1..] {
        let produced = run(*threads, 16);
        assert_eq!(
            produced.0, expected.0,
            "the rationed log differs at {threads} threads"
        );
        assert_eq!(
            produced.1, expected.1,
            "the state hash differs at {threads} threads"
        );
        assert_eq!(
            produced.2, expected.2,
            "the granted total differs at {threads} threads"
        );
        assert_eq!(
            produced.3, expected.3,
            "the unmet total differs at {threads} threads"
        );
    }
}

#[test]
fn a_place_that_produces_less_than_it_eats_runs_its_store_down() {
    let mut world = World::new(CONFIG).expect("the extent must describe a world");
    let fixture = build(&mut world);
    let poor = fixture.deficit[0];
    let rich = fixture.surplus[0];
    let mut poor_before = world
        .settlements()
        .store(poor)
        .and_then(|store| store.quantity(FOOD))
        .expect("the site lives");
    let mut rich_before = world
        .settlements()
        .store(rich)
        .and_then(|store| store.quantity(FOOD))
        .expect("the site lives");
    assert!(
        poor_before > Fix32::ZERO,
        "the poor site must start with a store"
    );

    let mut fell = 0;
    let mut rose = 0;
    for _ in 0..8 {
        // One whole period, so each pass sees exactly one application.
        for _ in 0..PERIOD {
            world.step(3).expect("the step must run");
        }
        let poor_now = world
            .settlements()
            .store(poor)
            .and_then(|store| store.quantity(FOOD))
            .expect("the site lives");
        let rich_now = world
            .settlements()
            .store(rich)
            .and_then(|store| store.quantity(FOOD))
            .expect("the site lives");
        if poor_now < poor_before {
            fell += 1;
        }
        if rich_now > rich_before {
            rose += 1;
        }
        assert!(poor_now >= Fix32::ZERO, "a store never falls below zero");
        // The store of a site cannot rise above the range of its type. That
        // is the bound, and it is a property of the store and not a budget.
        assert!(rich_now <= Fix32::MAX);
        poor_before = poor_now;
        rich_before = rich_now;
    }
    assert!(fell >= 1, "the poor site must run its store down");
    assert!(rose >= 4, "the rich site must accumulate");
    assert_eq!(
        world
            .settlements()
            .store(poor)
            .and_then(|store| store.quantity(FOOD)),
        Some(Fix32::ZERO),
        "the poor site must end at zero"
    );
}

#[test]
fn a_unit_of_a_lost_site_belongs_to_no_site() {
    // A home that named a lost slot would feed the settlement founded next
    // in that slot.
    let mut world = World::new(CONFIG).expect("the extent must describe a world");
    let fixture = build(&mut world);
    let site = fixture.surplus[0];
    let slot = world.settlements().slot_of(site);
    let members: Vec<Entity> = fixture
        .units
        .iter()
        .copied()
        .filter(|unit| world.soldiers().home(*unit).flatten() == slot)
        .collect();
    assert!(!members.is_empty(), "the site must hold units");
    assert!(world.destroy_settlement(site));
    for unit in &members {
        assert_eq!(world.soldiers().home(*unit), Some(None));
    }
    world.step(2).expect("the step must run");
    assert!(world.check_invariants());
}

#[test]
fn the_rule_refuses_a_rate_below_zero() {
    assert!(NeedRule::new(Fix32(-1), Fix32::ZERO, Fix32::ZERO, Fix32::ZERO).is_err());
    assert!(NeedRule::new(Fix32::ZERO, Fix32(-1), Fix32::ZERO, Fix32::ZERO).is_err());
    assert!(NeedRule::new(Fix32::ZERO, Fix32::ZERO, Fix32(-1), Fix32::ZERO).is_err());
    assert!(NeedRule::new(Fix32::ZERO, Fix32::ZERO, Fix32::ZERO, Fix32(-1)).is_err());
    assert!(NeedRule::new(Fix32::ZERO, Fix32::ZERO, Fix32::ZERO, Fix32::ZERO).is_ok());
}

#[test]
fn a_site_holds_one_cohort_for_each_faction() {
    // A site with one cohort has nothing to split, and the exactness rule
    // would then be a capability that nothing invokes.
    let mut table = CohortTable::new();
    table.rebuild(&[0, 0], &[FactionId(0), FactionId(1)], &[1, 1], 1);
    assert_eq!(table.rows().len(), COHORTS_PER_SITE);
    assert_eq!(table.headcount(0, FactionId(0)), Some(1));
    assert_eq!(table.headcount(0, FactionId(1)), Some(1));
    assert_eq!(table.headcount(0, FactionId(2)), Some(0));
    assert_eq!(table.headcount_total().0, 2);
    assert_eq!(table.unattached(), 0);
    assert!(table.check_invariants());
    assert!(table.describes(&[0, 0], &[FactionId(0), FactionId(1)], &[1, 1], 1));
    assert!(!table.describes(&[0, 0], &[FactionId(0), FactionId(0)], &[1, 1], 1));
}
