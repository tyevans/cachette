//! The properties of a founding, over arbitrary seeds.
//!
//! One seed proves one world. These properties run over many, because a
//! founding reads the world the generator made and a fixture that names one
//! seed measures itself.[^1]
//!
//! Every world here is wider than the coarsest lattice spacing of the terrain
//! generator. A world narrower than that spacing sits inside one lattice cell
//! and holds one kind of ground.[^2]
//!
//! A seed whose sample holds no place that admits the group is a world these
//! properties cannot use. The property says so and returns, rather than
//! comparing two runs that founded nothing and finding them equal.[^2]
//!
//! # References
//!
//! [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
//! [^2]: Findings register, FND-054. `docs/FINDINGS.md`

use cachette_core::founding::{SAMPLE_SIZE, SURVEY_CEILING};
use cachette_core::terrain::TileKind;
use cachette_core::{FactionId, World, WorldConfig};
use proptest::prelude::*;

/// The extent of the fixture world.
///
/// The coarsest lattice of the generator spans sixty-four tiles, so this world
/// holds three lattice cells along each axis and therefore holds water as well
/// as open ground.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-054. `docs/FINDINGS.md`
const EXTENT: u32 = 192;

/// The size of the founding group. The size is an input to a run.
const GROUP: u32 = 30;

/// Builds a fixture world.
fn world_of(seed: u64) -> World {
    World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed,
        faction_count: 4,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world")
}

/// What one founding produced: the place, where its people stand, and the
/// state hash after a few frames.
type Run = ((i32, i32), Vec<(i32, i32)>, u64);

/// Runs one founding at a thread count and returns what it produced.
///
/// Returns `None` when the sample holds no place that admits the group.
fn run(seed: u64, threads: usize) -> Option<Run> {
    let mut world = world_of(seed);
    let founded = world.found_run(GROUP, FactionId(0)).ok()?;
    let place = (founded.place().q, founded.place().r);
    let mut standing: Vec<(i32, i32)> = founded
        .people()
        .iter()
        .map(|person| {
            let at = world.soldiers().address(*person).expect("the person lives");
            (at.q, at.r)
        })
        .collect();
    standing.sort_unstable();
    for _ in 0..3 {
        world.step(threads).expect("the step must run");
    }
    Some((place, standing, world.state_hash().finish()))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(12))]

    /// The same seed gives the same place and the same group at every thread
    /// count.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D5. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[test]
    fn one_seed_gives_one_run_at_every_thread_count(seed: u64) {
        let Some(one) = run(seed, 1) else {
            return Ok(());
        };
        prop_assert_eq!(Some(one.clone()), run(seed, 2));
        prop_assert_eq!(Some(one), run(seed, 12));
    }

    /// Every place the engine chooses answers the same test.
    ///
    /// The place stands on ground that admits a unit, it holds the whole
    /// group, it scores at least as much as every eligible place the sample
    /// refused, and the score in the report is the score of the counts in the
    /// report.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decisions D4 and D5. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    #[test]
    fn every_chosen_place_answers_the_same_test(seed: u64) {
        let world = world_of(seed);
        let survey = world.survey_founding(GROUP, FactionId(0)).expect("the survey must run");
        prop_assert_eq!(survey.drawn(), SAMPLE_SIZE);
        prop_assert!(survey.tiles_read() <= SURVEY_CEILING);
        let Some(chosen) = survey.chosen() else {
            return Ok(());
        };
        prop_assert!(chosen.is_eligible());
        prop_assert!(chosen.provision().room >= GROUP);
        prop_assert_eq!(
            world.tile_kind(chosen.address()).map(TileKind::is_passable),
            Some(true)
        );
        prop_assert_eq!(chosen.score(), chosen.provision().score());
        for other in survey.rejected() {
            if other.is_eligible() {
                prop_assert!(other.score().0 <= chosen.score().0);
            }
        }
    }
}
