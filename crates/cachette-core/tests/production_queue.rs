//! A site builds a typed unit from a bounded queue its store pays for.
//!
//! Every test drives the step and then reads the world. A queue that only a
//! test advanced would prove that the arithmetic works and not that the step
//! reaches it.[^1]
//!
//! **Each fixture is built for the extreme it needs**, and each asserts that
//! it reached that extreme. A site whose store always pays never refuses, so
//! a refusal test built on a rich world would measure the fixture.[^2]
//!
//! Each fixture writes its own cost row. The default row holds placeholders
//! that the balance harness will change, and a test that read them would
//! measure the register.[^3]
//!
//! # References
//!
//! [^1]: Testing rules, drive the real caller. `.agents/rules/testing.md`
//! [^2]: Testing rules, section 2a. `.agents/rules/testing.md`
//! [^3]: Balance register, the production queue. `docs/reference/balance.md`

use cachette_core::production::{BuildCostRow, QueueOrder, QUEUE_BOUND};
use cachette_core::rates::RateSchedule;
use cachette_core::site::CommodityId;
use cachette_core::unit_type::{UnitTypeId, SOLDIER, WORKER};
use cachette_core::{Axial, Entity, FactionId, Fix32, World, WorldConfig};

/// The commodity that every fixture uses.
const GOOD: CommodityId = CommodityId(0);

/// The faction that owns every site in these fixtures.
const OWNER: FactionId = FactionId(0);

/// The work that a fixture entry needs. Two advances, so a test runs two
/// ticks and not eight.
const WORK: u32 = 2;

/// The residents that a fixture entry takes.
const PEOPLE: u32 = 1;

/// The goods that a finished fixture entry takes.
const GOODS: Fix32 = Fix32::from_int(3);

/// The goods that one advance of a fixture entry takes.
const CHARGE: Fix32 = Fix32::ONE;

/// A world that holds ground on every tile the fixtures need.
const CONFIG: WorldConfig = WorldConfig {
    width: 16,
    height: 16,
    seed: 0x0cac_4e77_0497,
    faction_count: 2,
    unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
};

/// Builds a world whose queue advances on every tick, and founds one site.
///
/// The store starts with the quantity the caller names, so a caller states
/// the extreme it wants: nothing, exactly one advance, or enough to finish.
fn one_site(stock: Fix32, residents: u32) -> (World, Entity, Vec<Entity>) {
    let mut world = World::new(CONFIG).expect("the extent must describe a world");
    world.set_queue_schedule(RateSchedule::new(1, 0).expect("one is inside the range"));
    assert!(
        world.set_queue_charge(GOOD, CHARGE),
        "the good is in the set"
    );
    world
        .define_build_cost(
            SOLDIER.0,
            BuildCostRow {
                work: WORK,
                people: PEOPLE,
                goods: [GOODS],
            },
        )
        .expect("the soldier row is inside the table");
    let site = world
        .found_settlement(Axial::new(0, 0), OWNER)
        .expect("the tile is inside the world");
    world
        .set_settlement_store(site, GOOD, stock)
        .expect("the good is in the set");
    let people = (0..residents)
        .map(|index| {
            let unit = world
                .spawn_soldier(Axial::new(0, 0), OWNER)
                .expect("the ground admits a unit");
            assert!(world.set_home_site(unit, Some(site)), "the site is live");
            assert_eq!(world.unit_type(unit), Some(WORKER), "index {index}");
            unit
        })
        .collect();
    (world, site, people)
}

/// Steps the world and asserts that the invariants hold at each frame.
fn run(world: &mut World, frames: u64, threads: usize) {
    for _ in 0..frames {
        world.step(threads).expect("the step must run");
        assert!(world.check_invariants(), "the world lost an invariant");
    }
}

/// Returns the live units whose home is the site, by type.
fn residents_by_type(world: &World, site: Entity, unit_type: UnitTypeId) -> usize {
    let slot = world.settlements().slot_of(site);
    world
        .soldiers()
        .iter()
        .filter(|unit| world.soldiers().home(*unit) == Some(slot))
        .filter(|unit| world.unit_type(*unit) == Some(unit_type))
        .count()
}

/// Returns every live unit whose home is the site.
fn residents(world: &World, site: Entity) -> usize {
    let slot = world.settlements().slot_of(site);
    world
        .soldiers()
        .iter()
        .filter(|unit| world.soldiers().home(*unit) == Some(slot))
        .count()
}

#[test]
fn a_queue_at_the_bound_of_its_world_refuses_another_entry() {
    // **The bound is a parameter of the world, and it sits below the width of
    // the stored block here.** A fixture that filled the block would be
    // refused by the block and not by the bound, and the bound could then be
    // removed with the test still green.
    let bound = 2usize;
    assert!(
        bound < QUEUE_BOUND,
        "the fixture must reach the bound first"
    );
    let (mut world, site, _) = one_site(Fix32::ZERO, 1);
    assert!(world.set_queue_bound(bound));
    for index in 0..bound {
        world
            .order_site_queue(OWNER, site, QueueOrder::Push(SOLDIER))
            .unwrap_or_else(|refusal| panic!("push {index} must be taken: {refusal}"));
    }
    let refusal = world
        .order_site_queue(OWNER, site, QueueOrder::Push(SOLDIER))
        .expect_err("the queue is at the bound of its world");
    assert_eq!(
        world.site_queue(site).map(<[_]>::len),
        Some(bound),
        "the refused push must change nothing: {refusal}"
    );
    assert_eq!(
        world.queue_refused_at_the_verb(),
        1,
        "the verb must count the refusal"
    );
}

#[test]
fn a_queue_that_fills_the_stored_block_refuses_another_entry() {
    let (mut world, site, _) = one_site(Fix32::ZERO, 1);
    for index in 0..QUEUE_BOUND {
        world
            .order_site_queue(OWNER, site, QueueOrder::Push(SOLDIER))
            .unwrap_or_else(|refusal| panic!("push {index} must be taken: {refusal}"));
    }
    let refusal = world
        .order_site_queue(OWNER, site, QueueOrder::Push(SOLDIER))
        .expect_err("the queue fills the stored block");
    assert_eq!(
        world.site_queue(site).map(<[_]>::len),
        Some(QUEUE_BOUND),
        "the refused push must change nothing: {refusal}"
    );
    assert_eq!(
        world.queue_refused_at_the_verb(),
        1,
        "the verb must count the refusal"
    );
}

#[test]
fn a_finished_entry_spends_a_resident_and_makes_a_unit_of_the_queued_type() {
    // Three residents, so the site can lose one and still hold a group.
    let (mut world, site, _) = one_site(Fix32::from_int(20), 3);
    world
        .order_site_queue(OWNER, site, QueueOrder::Push(SOLDIER))
        .expect("the queue is empty");
    let before = residents(&world, site);
    assert_eq!(before, 3, "the fixture must seat three residents");
    assert_eq!(residents_by_type(&world, site, SOLDIER), 0);

    run(&mut world, u64::from(WORK), 1);

    assert_eq!(
        world.queue_produced(),
        1,
        "the queue must have produced one"
    );
    assert_eq!(
        residents_by_type(&world, site, WORKER),
        2,
        "the site must have spent exactly one of the people it held"
    );
    assert_eq!(
        residents_by_type(&world, site, SOLDIER),
        1,
        "a unit of the queued type must stand at the site"
    );
    // **The queue never adds a person and never removes one on balance.** It
    // gives a person the site already held a type and a cost, and the unit it
    // makes lives at the site that built it.
    assert_eq!(residents(&world, site), before);
    assert!(
        world.site_queue(site).is_some_and(<[_]>::is_empty),
        "the finished entry must leave the queue"
    );
}

#[test]
fn a_finished_entry_with_no_resident_is_refused_and_counted() {
    // The extreme: a store that pays every charge and a site that houses
    // nobody. A fixture with residents never reaches this branch.
    let (mut world, site, _) = one_site(Fix32::from_int(20), 0);
    world
        .order_site_queue(OWNER, site, QueueOrder::Push(SOLDIER))
        .expect("the queue is empty");
    run(&mut world, u64::from(WORK), 1);

    assert_eq!(world.queue_refused_without_a_person(), 1);
    assert_eq!(world.queue_refused_without_goods(), 0, "counted apart");
    assert_eq!(world.queue_produced(), 0);
    // The entry stays at the front, and it costs nothing further.
    let queue = world.site_queue(site).expect("the site stands").to_vec();
    assert_eq!(queue.len(), 1);
    assert_eq!(queue[0].work, WORK, "the entry must hold its whole work");
    let held = world
        .settlements()
        .store(site)
        .and_then(|store| store.quantity(GOOD))
        .expect("the site is live");
    run(&mut world, 1, 1);
    let after = world
        .settlements()
        .store(site)
        .and_then(|store| store.quantity(GOOD))
        .expect("the site is live");
    assert_eq!(after, held, "a refused entry must not advance again");
}

#[test]
fn a_finished_entry_with_no_goods_is_refused_and_counted_apart() {
    // The extreme: a store that pays every advance and nothing more. The
    // charge is one for each of two advances, so the store holds two.
    let (mut world, site, _) = one_site(Fix32::from_int(2), 3);
    world
        .order_site_queue(OWNER, site, QueueOrder::Push(SOLDIER))
        .expect("the queue is empty");
    run(&mut world, u64::from(WORK), 1);

    assert_eq!(world.queue_refused_without_goods(), 1);
    assert_eq!(world.queue_refused_without_a_person(), 0, "counted apart");
    assert_eq!(world.queue_produced(), 0);
    assert_eq!(
        residents(&world, site),
        3,
        "a refused entry must spend nobody"
    );
    let queue = world.site_queue(site).expect("the site stands").to_vec();
    assert_eq!(queue.len(), 1, "the entry must stay at the front");
    assert_eq!(queue[0].work, WORK);

    // Put the goods in, and the same entry then finishes. This proves that
    // the refusal was the goods and not something else the fixture did.
    world
        .set_settlement_store(site, GOOD, Fix32::from_int(20))
        .expect("the good is in the set");
    run(&mut world, 1, 1);
    assert_eq!(world.queue_produced(), 1);
    assert_eq!(residents_by_type(&world, site, SOLDIER), 1);
}

#[test]
fn an_entry_does_not_advance_while_the_store_cannot_pay() {
    // The extreme: an empty store. A queue is never free.
    let (mut world, site, _) = one_site(Fix32::ZERO, 3);
    world
        .order_site_queue(OWNER, site, QueueOrder::Push(SOLDIER))
        .expect("the queue is empty");
    run(&mut world, 4, 1);
    let queue = world.site_queue(site).expect("the site stands").to_vec();
    assert_eq!(queue[0].work, 0, "an unpaid entry must not move");
    assert_eq!(world.queue_produced(), 0);

    // Put one charge in, and the entry then advances by exactly one.
    world
        .set_settlement_store(site, GOOD, CHARGE)
        .expect("the good is in the set");
    run(&mut world, 1, 1);
    let queue = world.site_queue(site).expect("the site stands").to_vec();
    assert_eq!(queue[0].work, 1, "one charge must buy one advance");
}

#[test]
fn the_front_entry_is_the_one_that_advances_and_the_order_never_changes() {
    let (mut world, site, _) = one_site(Fix32::from_int(40), 3);
    world
        .order_site_queue(OWNER, site, QueueOrder::Push(SOLDIER))
        .expect("the queue is empty");
    world
        .order_site_queue(OWNER, site, QueueOrder::Push(WORKER))
        .expect("the queue has room");
    run(&mut world, 1, 1);
    let queue = world.site_queue(site).expect("the site stands").to_vec();
    assert_eq!(queue[0].unit_type, SOLDIER, "the order is the push order");
    assert_eq!(queue[0].work, 1, "only the front entry advances");
    assert_eq!(queue[1].unit_type, WORKER);
    assert_eq!(queue[1].work, 0, "an entry behind the front holds no work");
}

#[test]
fn a_cleared_entry_leaves_the_order_of_the_entries_behind_it() {
    let (mut world, site, _) = one_site(Fix32::ZERO, 1);
    for unit_type in [SOLDIER, WORKER, SOLDIER] {
        world
            .order_site_queue(OWNER, site, QueueOrder::Push(unit_type))
            .expect("the queue has room");
    }
    world
        .order_site_queue(OWNER, site, QueueOrder::Clear(0))
        .expect("the position holds an entry");
    let queue = world.site_queue(site).expect("the site stands").to_vec();
    assert_eq!(queue.len(), 2);
    assert_eq!(queue[0].unit_type, WORKER);
    assert_eq!(queue[1].unit_type, SOLDIER);
    let refusal = world
        .order_site_queue(OWNER, site, QueueOrder::Clear(2))
        .expect_err("the position is empty");
    assert_eq!(
        world.site_queue(site).map(<[_]>::len),
        Some(2),
        "the refused clear must change nothing: {refusal}"
    );
}

#[test]
fn the_verb_refuses_a_site_of_another_faction_and_a_type_the_table_has_no_row_for() {
    let (mut world, site, _) = one_site(Fix32::ZERO, 1);
    world
        .order_site_queue(FactionId(1), site, QueueOrder::Push(SOLDIER))
        .expect_err("the site belongs to another faction");
    world
        .order_site_queue(OWNER, site, QueueOrder::Push(UnitTypeId(u8::MAX)))
        .expect_err("the number names no row");
    assert!(
        world.site_queue(site).is_some_and(<[_]>::is_empty),
        "a refused order changes nothing"
    );
    assert_eq!(world.queue_refused_at_the_verb(), 2);
}

#[test]
fn the_queue_enters_the_state_hash() {
    let (mut queued, site, _) = one_site(Fix32::from_int(20), 3);
    let (idle, _, _) = one_site(Fix32::from_int(20), 3);
    assert_eq!(
        queued.state_hash().finish(),
        idle.state_hash().finish(),
        "two worlds built the same way must hash the same"
    );
    queued
        .order_site_queue(OWNER, site, QueueOrder::Push(SOLDIER))
        .expect("the queue is empty");
    assert_ne!(
        queued.state_hash().finish(),
        idle.state_hash().finish(),
        "one queued entry must move the hash"
    );
}

#[test]
fn the_queue_gives_one_answer_at_every_thread_count() {
    // The queue is in flight at every thread count: the run crosses the tick
    // that finishes the entry and keeps going past it.
    let counts = [1usize, 2, 12];
    let mut answers = Vec::new();
    for threads in counts {
        let (mut world, site, _) = one_site(Fix32::from_int(40), 4);
        for _ in 0..3 {
            world
                .order_site_queue(OWNER, site, QueueOrder::Push(SOLDIER))
                .expect("the queue has room");
        }
        run(&mut world, 6, threads);
        assert!(
            residents_by_type(&world, site, SOLDIER) > 0,
            "the fixture must finish an entry, or it tests nothing"
        );
        answers.push((
            world.state_hash().finish(),
            world.event_log_bytes().to_vec(),
            residents_by_type(&world, site, SOLDIER),
        ));
    }
    for (index, answer) in answers.iter().enumerate().skip(1) {
        assert_eq!(
            *answer, answers[0],
            "the answer differs at {} threads",
            counts[index]
        );
    }
}

#[test]
fn the_built_in_controller_queues_through_the_same_verb() {
    // The test drives the engine. A test that called the verb would prove the
    // verb works and not that the controller reaches it.
    let mut world = World::new(WorldConfig {
        width: 64,
        height: 64,
        seed: 0x0cac_4e77_0472,
        faction_count: 4,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    let outcomes = world.seed_world().expect("a fresh world seeds once");
    assert!(
        outcomes.iter().any(|outcome| outcome.founding().is_some()),
        "the scenario must seat a faction, or the controller has nothing to do"
    );
    let mut queued = 0usize;
    for _ in 0..4 {
        world.step(1).expect("the step must run");
        let schema = world.action_schema();
        queued += world
            .controller_log()
            .iter()
            .filter(|command| schema.verb_of(command.action) == Some(cachette_core::Verb::Queue))
            .filter(|command| command.applied != 0)
            .count();
    }
    assert!(
        queued > 0,
        "the controller must reach the queue verb and be taken"
    );
    let sites: usize = world
        .settlements()
        .iter()
        .filter_map(|site| world.site_queue(site))
        .map(<[_]>::len)
        .sum();
    assert!(sites > 0, "a site must hold what the controller queued");
}

#[test]
fn a_bound_of_zero_turns_the_queue_off() {
    let (mut world, site, _) = one_site(Fix32::from_int(20), 3);
    assert!(world.set_queue_bound(0));
    world
        .order_site_queue(OWNER, site, QueueOrder::Push(SOLDIER))
        .expect_err("a bound of zero refuses every push");
    run(&mut world, 4, 1);
    assert_eq!(world.queue_produced(), 0);
    assert_eq!(residents(&world, site), 3);
}

/// Returns one count of the subsystem census by name.
fn census(world: &World, name: &str) -> i64 {
    world
        .subsystem_census()
        .into_iter()
        .find(|(row, _)| *row == name)
        .expect("the census holds the row")
        .1
}

#[test]
fn the_census_holds_the_queue_and_the_count_stays_after_the_tick_that_made_it() {
    // The extreme: a run that produces one unit early and then runs on. A
    // row that read the last tick would say zero here, and a reader would
    // take that for a queue that never built anything.
    let (mut world, site, _) = one_site(Fix32::from_int(20), 3);
    assert_eq!(census(&world, "queue_produced"), 0);
    world
        .order_site_queue(OWNER, site, QueueOrder::Push(SOLDIER))
        .expect("the queue is empty");

    run(&mut world, u64::from(WORK), 1);
    assert_eq!(census(&world, "queue_produced"), 1, "the row must rise");

    run(&mut world, 6, 1);
    assert_eq!(
        world.queue_produced(),
        0,
        "the fixture must reach a later tick that produced nothing"
    );
    assert_eq!(
        census(&world, "queue_produced"),
        1,
        "the census still says what the run made"
    );
}

#[test]
fn the_census_keeps_the_two_refusals_of_a_finished_entry_apart() {
    // The extreme: a store that pays every advance and cannot pay the
    // finished entry. The site holds its residents, so only one of the two
    // refusal rows may move.
    let (mut world, site, _) = one_site(Fix32::from_int(WORK as i16), 3);
    world
        .order_site_queue(OWNER, site, QueueOrder::Push(SOLDIER))
        .expect("the queue is empty");
    run(&mut world, u64::from(WORK), 1);

    assert_eq!(census(&world, "queue_produced"), 0);
    assert_eq!(census(&world, "queue_refused_without_goods"), 1);
    assert_eq!(census(&world, "queue_refused_without_a_person"), 0);

    // The verb refusal is a row of its own, and it counts an order that no
    // advance ever saw.
    assert_eq!(census(&world, "queue_refused_at_the_verb"), 0);
    world
        .order_site_queue(FactionId(1), site, QueueOrder::Push(SOLDIER))
        .expect_err("the site belongs to another faction");
    assert_eq!(census(&world, "queue_refused_at_the_verb"), 1);
    run(&mut world, 3, 1);
    assert_eq!(
        census(&world, "queue_refused_at_the_verb"),
        1,
        "the count survives the advance that empties the per-tick counter"
    );
}
