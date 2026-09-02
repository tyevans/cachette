//! A run founds one group for each faction.
//!
//! A run begins with one founding for each faction the world holds. The run
//! founds in ascending faction index, and each founding keeps a minimum
//! distance from every place a founding before it took.[^1] A founding that
//! finds no admissible place fails, and the foundings before it stand.
//!
//! These tests go through the public interface of the crate.[^2]
//!
//! Every world here is wider than the coarsest lattice spacing of the terrain
//! generator. A world narrower than that spacing sits inside one lattice cell,
//! so every tile of it holds one kind of ground and the test then measures the
//! fixture.[^3]
//!
//! # References
//!
//! [^1]: ADR-0076, a founding keeps a fixed distance from the foundings before it. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
//! [^2]: Testing rules, section 6. `.claude/rules/testing.md`
//! [^3]: Findings register, FND-054. `docs/FINDINGS.md`

use cachette_core::founding::{MINIMUM_FOUNDING_DISTANCE, SAMPLE_SIZE, SURVEY_CEILING};
use cachette_core::{Axial, FactionId, FoundingError, World, WorldConfig};

/// The extent of the ordinary fixture.
///
/// The coarsest lattice of the generator spans sixty-four tiles, so this world
/// holds three lattice cells along each axis. It therefore holds water as well
/// as open ground.[^1]
///
/// The extent also follows from the separation and from the faction count. The
/// world must be wide enough to seat every faction at the minimum distance,
/// and this one is many times that width.[^2]
///
/// # References
///
/// [^1]: Findings register, FND-054. `docs/FINDINGS.md`
/// [^2]: Backlog item 0094, the fixture extent. `docs/backlog/complete/0094-decide-how-many-groups-found-a-world.md`
const EXTENT: u32 = 192;

/// The number of factions the ordinary fixture holds.
const FACTIONS: u16 = 4;

/// The size of each founding group.
const GROUP: u32 = 30;

/// The extent of the crowded fixture.
///
/// The world holds one lattice cell and a half along each axis, so it still
/// holds two kinds of ground. It is small enough that a run of many factions
/// runs out of room, which is the case the refusal test needs.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
const CROWDED_EXTENT: u32 = 96;

/// The number of factions the crowded fixture holds.
///
/// The world cannot seat this many groups at the minimum distance, so the run
/// seats some factions and refuses the rest.
const CROWDED_FACTIONS: u16 = 32;

/// The seed of the crowded fixture.
const CROWDED_SEED: u64 = 0x0cac_4e77_0061;

/// The number of factions the close fixture holds.
///
/// Four foundings in a world of this extent land far apart by chance, so a
/// test of the distance over four factions measures the fixture and not the
/// rule.[^1] Eight foundings land close, so the assertion reaches the case
/// the rule governs.
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
const CLOSE_FACTIONS: u16 = 8;

/// Builds a fixture world of the ordinary extent.
fn world_of(seed: u64) -> World {
    world_with(seed, FACTIONS)
}

/// Builds a fixture world of the ordinary extent, with a faction count.
fn world_with(seed: u64, factions: u16) -> World {
    World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed,
        faction_count: factions,
    })
    .expect("the extent must describe a world")
}

/// Builds the crowded fixture, which cannot seat every faction.
fn crowded_world() -> World {
    World::new(WorldConfig {
        width: CROWDED_EXTENT,
        height: CROWDED_EXTENT,
        seed: CROWDED_SEED,
        faction_count: CROWDED_FACTIONS,
    })
    .expect("the extent must describe a world")
}

/// Returns the places a run seated, in the order the run founded them.
fn seated_places(world: &mut World) -> Vec<Axial> {
    world
        .found_run_for_every_faction(GROUP)
        .iter()
        .filter_map(|outcome| outcome.founding().map(cachette_core::Founding::place))
        .collect()
}

#[test]
fn a_run_founds_one_group_for_each_faction() {
    let mut world = world_of(0x0cac_4e77_0094);
    let outcomes = world.found_run_for_every_faction(GROUP);

    assert_eq!(
        outcomes.len(),
        FACTIONS as usize,
        "the run founded a number of groups the world does not hold"
    );
    for outcome in &outcomes {
        let founding = outcome
            .result()
            .as_ref()
            .expect("this world seats every faction");
        assert_eq!(founding.people().len(), GROUP as usize);
        for person in founding.people() {
            assert_eq!(
                world.soldiers().faction(*person),
                Some(outcome.faction()),
                "a founder answers to another faction"
            );
        }
    }
    assert_eq!(world.soldiers().len(), GROUP * u32::from(FACTIONS));
}

#[test]
fn the_run_founds_in_ascending_faction_index() {
    // The order is a property of the run. The call takes no order from the
    // caller, so no caller can give one faction the better place by naming it
    // first.[^1]
    //
    // [^1]: ADR-0004, iteration order is explicit, decision D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    let mut world = world_of(0x0cac_4e77_0094);
    let outcomes = world.found_run_for_every_faction(GROUP);

    let factions: Vec<u16> = outcomes.iter().map(|outcome| outcome.faction().0).collect();
    let ascending: Vec<u16> = (0..FACTIONS).collect();
    assert_eq!(
        factions, ascending,
        "the run founded the factions in another order"
    );

    // The first founding reads no place, so it takes the best place in its
    // sample. A run that founded in another order would give that place to
    // another faction.
    let first = outcomes[0]
        .founding()
        .expect("the lowest faction index founds first");
    let alone = world_of(0x0cac_4e77_0094)
        .survey_founding(GROUP, FactionId(0))
        .expect("the survey must run")
        .chosen()
        .expect("a place exists");
    assert_eq!(
        first.place(),
        alone.address(),
        "the faction that founded first is not the lowest faction index"
    );
}

#[test]
fn two_foundings_of_one_run_keep_the_minimum_distance() {
    // The fixture holds enough factions that two samples land close. A world
    // of four factions seats them far apart by chance, and the assertion then
    // measures the fixture.[^1]
    //
    // [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
    let mut world = world_with(0x0cac_4e77_0094, CLOSE_FACTIONS);
    let places = seated_places(&mut world);
    assert_eq!(
        places.len(),
        CLOSE_FACTIONS as usize,
        "the fixture refused a faction, so it does not reach the case"
    );

    for (ordinal, place) in places.iter().enumerate() {
        for other in &places[ordinal + 1..] {
            assert!(
                place.distance(*other) >= MINIMUM_FOUNDING_DISTANCE,
                "two foundings stand {} apart, and the rule asks for {}",
                place.distance(*other),
                MINIMUM_FOUNDING_DISTANCE
            );
        }
    }
}

#[test]
fn a_place_at_the_minimum_distance_stands_and_one_step_closer_is_refused() {
    // The restored defect that this test must catch is one step off the
    // boundary, so the test asserts at the boundary.[^1]
    //
    // [^1]: Findings register, FND-070. `docs/FINDINGS.md`
    let world = world_of(0x0cac_4e77_0094);
    let taken = world
        .survey_founding(GROUP, FactionId(0))
        .expect("the survey must run")
        .chosen()
        .expect("a place exists")
        .address();

    // Two places on one ray from the taken place, one at the minimum distance
    // and one a single step nearer. Both must admit the group when nothing is
    // taken, or the test would measure the ground rather than the rule.
    let at = |distance: u32| Axial::new(taken.q + distance as i32, taken.r);
    let far = at(MINIMUM_FOUNDING_DISTANCE);
    let near = at(MINIMUM_FOUNDING_DISTANCE - 1);
    assert_eq!(far.distance(taken), MINIMUM_FOUNDING_DISTANCE);
    assert_eq!(near.distance(taken), MINIMUM_FOUNDING_DISTANCE - 1);

    let alone = world
        .survey_places(&[far, near], GROUP, &[])
        .expect("the survey must run");
    for candidate in alone.candidates() {
        assert!(
            candidate.is_eligible(),
            "the ground at ({}, {}) refuses the group, so this fixture \
             measures the ground and not the separation",
            candidate.address().q,
            candidate.address().r
        );
    }

    let apart = world
        .survey_places(&[far, near], GROUP, &[taken])
        .expect("the survey must run");
    for candidate in apart.candidates() {
        let expected = candidate.address() == far;
        assert_eq!(
            candidate.is_separated(),
            expected,
            "the place at ({}, {}) reports the wrong separation",
            candidate.address().q,
            candidate.address().r
        );
        assert_eq!(candidate.is_eligible(), expected);
    }
    assert_eq!(
        apart.chosen().map(|candidate| candidate.address()),
        Some(far),
        "the founding took a place inside the minimum distance"
    );
}

#[test]
fn the_faction_is_in_the_draw_key() {
    // The candidate ordinal alone gives every faction one sample. Every
    // founding after the first would then read the places the first read, and
    // the separation rule would refuse them all. Both determinism tests pass
    // over that defect, because the sample repeats on every run and at every
    // thread count.[^1]
    //
    // The consequence is smaller than it looks, and only this test sees the
    // defect. A shared sample still seats every faction, because the sample
    // holds many places that stand far enough apart. The defect narrows the
    // pool that every founding after the first draws from, and no count and
    // no place says so.[^2]
    //
    // [^1]: Testing rules, section 2. `.claude/rules/testing.md`
    // [^2]: Findings register, FND-106. `docs/FINDINGS.md`
    let world = world_of(0x0cac_4e77_0094);
    let sample = |faction: u16| {
        let mut places: Vec<(i32, i32)> = world
            .survey_founding(GROUP, FactionId(faction))
            .expect("the survey must run")
            .candidates()
            .iter()
            .map(|candidate| (candidate.address().q, candidate.address().r))
            .collect();
        places.sort_unstable();
        places
    };
    let first = sample(0);
    for faction in 1..FACTIONS {
        assert_ne!(
            first,
            sample(faction),
            "faction {faction} drew the sample that faction 0 drew, so the \
             faction is not in the draw key"
        );
    }
}

#[test]
fn the_survey_cost_does_not_grow_with_the_world() {
    // The separation adds a comparison against the places taken. That
    // comparison grows with the faction count and not with the world extent,
    // so the bounded cost still holds.[^1]
    //
    // [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D1. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    let taken = [Axial::new(40, 40), Axial::new(70, 12)];
    let read = |extent: u32| {
        World::new(WorldConfig {
            width: extent,
            height: extent,
            seed: 0x0cac_4e77_0094,
            faction_count: FACTIONS,
        })
        .expect("the extent must describe a world")
        .survey_founding_apart(GROUP, FactionId(3), &taken)
        .expect("the survey must run")
        .tiles_read()
    };
    // The larger world holds two hundred and fifty-six times the tiles of
    // the smaller one. A survey that read them all would read that many more
    // tiles here. A world edge clips a disc, so the two counts differ a
    // little and neither follows the extent.
    let small = read(128);
    let large = read(2048);
    assert!(
        small <= SURVEY_CEILING && large <= SURVEY_CEILING,
        "a survey read {small} then {large} tiles, above the ceiling of \
         {SURVEY_CEILING}"
    );
    assert!(
        large < small * 2,
        "the tiles read grew from {small} to {large} as the world grew"
    );
}

#[test]
fn a_refused_faction_reports_its_refusal_and_the_foundings_before_it_stand() {
    let mut world = crowded_world();
    let outcomes = world.found_run_for_every_faction(GROUP);

    let seated: Vec<&cachette_core::FoundingOutcome> = outcomes
        .iter()
        .filter(|outcome| outcome.is_seated())
        .collect();
    let refused: Vec<&cachette_core::FoundingOutcome> = outcomes
        .iter()
        .filter(|outcome| !outcome.is_seated())
        .collect();
    assert!(
        !seated.is_empty() && !refused.is_empty(),
        "the crowded fixture seated {} of {} factions, so it does not reach \
         the case",
        seated.len(),
        outcomes.len()
    );

    // The refusal names the sample it drew, and it does not draw again.
    for outcome in &refused {
        assert_eq!(
            *outcome.result(),
            Err(FoundingError::NoPlaceFound(SAMPLE_SIZE)),
            "a refused faction reported another reason"
        );
    }

    // The foundings before the refusal stand. The run does not undo them.
    assert_eq!(
        world.soldiers().len(),
        GROUP * seated.len() as u32,
        "a refusal undid a founding that stood"
    );
    for outcome in &seated {
        let founding = outcome.founding().expect("this outcome is seated");
        assert_eq!(
            world.settlement_on(founding.place()),
            Some(founding.settlement()),
            "a settlement that stood is gone"
        );
    }
}

#[test]
fn the_same_seed_founds_the_same_run_at_every_thread_count() {
    let at = |threads: usize| {
        let mut world = world_of(0x0cac_4e77_0094);
        let places: Vec<(i32, i32)> = seated_places(&mut world)
            .iter()
            .map(|place| (place.q, place.r))
            .collect();
        for _ in 0..4 {
            world.step(threads).expect("the step must run");
        }
        (places, world.state_hash().finish())
    };
    let one = at(1);
    assert_eq!(one, at(2), "two threads founded a different run");
    assert_eq!(one, at(12), "twelve threads founded a different run");
}
