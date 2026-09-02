//! A household is derived from the dwelling slot, and nothing stores one.
//!
//! Every test here drives the world through its public interface. It founds
//! dwellings, gives units a home, moves one between dwellings, and reads the
//! household back. A test that read the home column and grouped it itself
//! would prove that grouping works and not that the engine answers the
//! question.[^1]
//!
//! The fixture is not the world of the demonstration binary. That world is
//! chosen to look right, and every dwelling in it holds the same handful of
//! people.[^2] This one is built to reach the ends of the distribution: one
//! dwelling holds several residents, one holds a single resident, one holds
//! nobody, and two units live nowhere at all. A grouping defect lives at
//! those ends, and a fixture of equal dwellings supplies none of them.
//!
//! # References
//!
//! [^1]: Testing rules, section 5. `.claude/rules/testing.md`
//! [^2]: Testing rules, section 2a. `.claude/rules/testing.md`

use cachette_core::cohort::{NeedRule, NEED_FULL};
use cachette_core::{Axial, Entity, FactionId, Fix32, World, WorldConfig};

/// The thread counts that the equivalence test runs at.
const THREAD_COUNTS: [usize; 3] = [1, 2, 12];

/// The world that every fixture below stands on.
const CONFIG: WorldConfig = WorldConfig {
    width: 32,
    height: 32,
    seed: 7,
    faction_count: 2,
    unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
};

/// How many units live in the crowded dwelling.
///
/// The count is above one, so a reader that returned the first match would
/// pass on the single dwelling and fail here.
const CROWD: usize = 5;

/// What one fixture built.
struct Fixture {
    /// The dwelling that holds several residents.
    crowded: Entity,
    /// The residents of the crowded dwelling, in the order they were made.
    crowd: Vec<Entity>,
    /// The dwelling that holds one resident.
    single: Entity,
    /// The one resident of the single dwelling.
    lodger: Entity,
    /// The dwelling that holds nobody.
    empty: Entity,
    /// The units that live nowhere.
    homeless: Vec<Entity>,
}

/// Returns the open ground of a world, in tile order.
fn open_ground(world: &World) -> Vec<Axial> {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| world.admits_a_unit(*address))
        .collect()
}

/// Builds a world whose dwellings hold different numbers of people.
///
/// The starvation bound is raised out of reach, because the subject here is
/// who lives where and not who dies. A unit that starved during a run would
/// leave a household by dying, and that would hide the question the test
/// asks.
fn build(world: &mut World) -> Fixture {
    let rule = NeedRule::DEFAULT;
    world.set_need_rule(
        NeedRule::new(
            rule.decay(),
            rule.ration(),
            rule.threshold(),
            rule.recovery(),
            Fix32(NEED_FULL.0 * 4096),
        )
        .expect("every rate is at or above zero"),
    );

    let ground = open_ground(world);
    assert!(
        ground.len() > CROWD + 8,
        "the world holds only {} open tiles",
        ground.len()
    );

    let mut place = ground.iter().copied();
    let crowded = world
        .found_settlement(place.next().expect("open ground"), FactionId(0))
        .expect("the tile is free");
    let single = world
        .found_settlement(place.next().expect("open ground"), FactionId(0))
        .expect("the tile is free");
    let empty = world
        .found_settlement(place.next().expect("open ground"), FactionId(1))
        .expect("the tile is free");

    // The residents are spread over two factions, so a reader that grouped by
    // faction as well as by dwelling would split this household and fail.
    let mut crowd = Vec::with_capacity(CROWD);
    for ordinal in 0..CROWD {
        let unit = world
            .spawn_soldier(
                place.next().expect("open ground"),
                FactionId((ordinal % 2) as u16),
            )
            .expect("the ground admits a unit");
        assert!(world.set_home_site(unit, Some(crowded)));
        crowd.push(unit);
    }

    let lodger = world
        .spawn_soldier(place.next().expect("open ground"), FactionId(0))
        .expect("the ground admits a unit");
    assert!(world.set_home_site(lodger, Some(single)));

    // The units that live nowhere are made after the residents, so their
    // slots sit above the residents' slots. A reader that ran off the end of
    // a household would reach them.
    let homeless: Vec<Entity> = (0..2)
        .map(|_| {
            world
                .spawn_soldier(place.next().expect("open ground"), FactionId(0))
                .expect("the ground admits a unit")
        })
        .collect();

    let fixture = Fixture {
        crowded,
        crowd,
        single,
        lodger,
        empty,
        homeless,
    };
    assert_eq!(
        fixture.crowd.len(),
        CROWD,
        "the crowded dwelling is crowded"
    );
    assert!(
        world
            .household_of(fixture.empty)
            .expect("the dwelling is live")
            .is_empty(),
        "the empty dwelling is empty"
    );
    fixture
}

/// Builds a world and its fixture together.
fn world_and_fixture() -> (World, Fixture) {
    let mut world = World::new(CONFIG).expect("the configuration describes a world");
    let fixture = build(&mut world);
    (world, fixture)
}

#[test]
fn a_watcher_reads_every_unit_that_lives_in_one_dwelling() {
    let (world, fixture) = world_and_fixture();

    let mut expected = fixture.crowd.clone();
    expected.sort_by_key(|unit| unit.index());
    assert_eq!(
        world.household_of(fixture.crowded).expect("live dwelling"),
        expected,
        "the crowded dwelling holds every unit that named it, and nobody else"
    );
    assert_eq!(
        world.household_of(fixture.single).expect("live dwelling"),
        vec![fixture.lodger],
        "the single dwelling holds one resident"
    );
}

#[test]
fn a_dwelling_that_nobody_lives_in_reads_as_an_empty_household() {
    let (world, fixture) = world_and_fixture();
    let members = world
        .household_of(fixture.empty)
        .expect("an empty dwelling is a live dwelling");
    assert!(
        members.is_empty(),
        "an empty household is an answer, not an error"
    );
}

#[test]
fn a_unit_that_lives_nowhere_is_in_no_household() {
    let (world, fixture) = world_and_fixture();
    for unit in &fixture.homeless {
        assert_eq!(
            world.dwelling_of(*unit),
            Some(None),
            "a unit that lives nowhere is a state, not an error"
        );
    }
    for dwelling in [fixture.crowded, fixture.single, fixture.empty] {
        let members = world.household_of(dwelling).expect("live dwelling");
        for unit in &fixture.homeless {
            assert!(
                !members.contains(unit),
                "units that live nowhere do not live together"
            );
        }
    }
}

#[test]
fn a_unit_that_takes_a_dwelling_of_its_own_leaves_the_one_it_was_in() {
    let (mut world, fixture) = world_and_fixture();
    let mover = fixture.crowd[2];
    assert_eq!(world.dwelling_of(mover), Some(Some(fixture.crowded)));

    assert!(world.set_home_site(mover, Some(fixture.empty)));

    // Both sides of the move. A rule that only added would pass the second
    // assertion on its own, and the first is what catches it.
    let left = world.household_of(fixture.crowded).expect("live dwelling");
    assert!(
        !left.contains(&mover),
        "the unit left the household it was in, by moving"
    );
    assert_eq!(
        left.len(),
        CROWD - 1,
        "the household it left lost exactly one member"
    );
    let joined = world.household_of(fixture.empty).expect("live dwelling");
    assert_eq!(
        joined,
        vec![mover],
        "the household it joined gained exactly that member"
    );
    assert_eq!(world.dwelling_of(mover), Some(Some(fixture.empty)));
}

#[test]
fn a_unit_that_gives_up_its_dwelling_leaves_every_household() {
    let (mut world, fixture) = world_and_fixture();
    assert!(world.set_home_site(fixture.lodger, None));

    assert_eq!(world.dwelling_of(fixture.lodger), Some(None));
    assert!(
        world
            .household_of(fixture.single)
            .expect("live dwelling")
            .is_empty(),
        "the dwelling it left holds nobody"
    );
    for dwelling in [fixture.crowded, fixture.empty] {
        assert!(
            !world
                .household_of(dwelling)
                .expect("live dwelling")
                .contains(&fixture.lodger),
            "a unit that gave up its dwelling joined no other household"
        );
    }
}

#[test]
fn a_household_is_readable_before_any_barrier_runs() {
    // Nothing stores a household, so nothing has to be rebuilt for one to be
    // read. The move below is answered by the next read, with no step and no
    // rebuild between them. A stored roster would need one.
    let (mut world, fixture) = world_and_fixture();
    let before = world.household_of(fixture.crowded).expect("live dwelling");
    assert!(world.set_home_site(fixture.lodger, Some(fixture.crowded)));
    let after = world.household_of(fixture.crowded).expect("live dwelling");

    // The assertion names the exact membership on both sides. A length that
    // grew by one would also be produced by a reader that already held the
    // lodger and gained somebody else.
    let mut expected = fixture.crowd.clone();
    expected.push(fixture.lodger);
    expected.sort_by_key(|unit| unit.index());
    assert_eq!(
        before, fixture.crowd,
        "the read before the write is the roster before the write"
    );
    assert_eq!(
        after, expected,
        "the read answers the write that came before it, with no barrier between them"
    );
}

#[test]
fn a_dead_identity_reads_as_nothing_and_never_as_a_roster() {
    let (mut world, fixture) = world_and_fixture();
    let ghost = fixture.crowd[0];
    assert!(world.despawn_soldier(ghost));
    assert_eq!(
        world.dwelling_of(ghost),
        None,
        "a dead unit resolves to nothing"
    );
    let members = world.household_of(fixture.crowded).expect("live dwelling");
    assert!(
        !members.contains(&ghost),
        "a dead unit lives nowhere at all"
    );

    // A buffer that already holds a roster must be cleared by the call that
    // refuses, or a caller reads the previous answer as this one.
    let mut buffer = world.household_of(fixture.crowded).expect("live dwelling");
    assert!(!buffer.is_empty());
    assert!(world.destroy_settlement(fixture.empty));
    assert!(
        !world.household_into(fixture.empty, &mut buffer),
        "a dead dwelling resolves to nothing"
    );
    assert!(
        buffer.is_empty(),
        "a refused call leaves no stale roster in the buffer"
    );
    assert_eq!(world.household_of(fixture.empty), None);
}

#[test]
fn the_members_come_back_in_one_order_at_every_thread_count() {
    let mut answers: Vec<Vec<Vec<Entity>>> = Vec::new();
    for threads in THREAD_COUNTS {
        let (mut world, fixture) = world_and_fixture();
        for _ in 0..4 {
            world.step(threads).expect("the step runs");
        }
        // The move happens after the run, so the roster the test compares is
        // one the run could have disturbed.
        assert!(world.set_home_site(fixture.crowd[1], Some(fixture.empty)));
        answers.push(
            [fixture.crowded, fixture.single, fixture.empty]
                .iter()
                .map(|dwelling| world.household_of(*dwelling).expect("live dwelling"))
                .collect(),
        );
    }
    let first = &answers[0];
    for (index, answer) in answers.iter().enumerate().skip(1) {
        assert_eq!(
            answer, first,
            "the run at {} threads gave a different household order from the run at {} threads",
            THREAD_COUNTS[index], THREAD_COUNTS[0]
        );
    }
    for household in first {
        let mut sorted = household.clone();
        sorted.sort_by_key(|unit| unit.index());
        assert_eq!(
            *household, sorted,
            "the members come back in ascending slot order, which no thread order reaches"
        );
    }
}
