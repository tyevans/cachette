//! The founding of a run.
//!
//! A run begins with a small group of people in a place the engine chose.
//! These tests go through the public interface of the crate.[^1]
//!
//! Every world here is wider than the coarsest lattice spacing of the terrain
//! generator. A world narrower than that spacing sits inside one lattice cell,
//! so every tile of it falls on the same side of the water threshold and the
//! world holds one kind of ground.[^2] A founding in such a world finds no
//! good place, and the test then measures the fixture rather than the
//! founding.
//!
//! # References
//!
//! [^1]: Testing rules, section 6. `.claude/rules/testing.md`
//! [^2]: Findings register, FND-054. `docs/FINDINGS.md`

use cachette_core::founding::{self, Provision, SAMPLE_SIZE, SURVEY_CEILING, SURVEY_RADIUS};
use cachette_core::resource::ResourceKind;
use cachette_core::terrain::TileKind;
use cachette_core::types::Accum;
use cachette_core::{Axial, FactionId, FoundingError, World, WorldConfig};

/// The extent of the ordinary fixture.
///
/// The coarsest lattice of the generator spans sixty-four tiles, so this world
/// holds three lattice cells along each axis. It therefore holds water as well
/// as open ground, and a founding in it has both a good place and a poor one
/// to choose between.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-054. `docs/FINDINGS.md`
const EXTENT: u32 = 192;

/// The size of the founding group.
///
/// The size is an input to a run. It is not the population the world is sized
/// for, and no record and no register holds this number.[^1]
///
/// # References
///
/// [^1]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
const GROUP: u32 = 30;

/// Builds a fixture world of the ordinary extent.
fn world_of(seed: u64) -> World {
    World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed,
        faction_count: 4,
    })
    .expect("the extent must describe a world")
}

/// Returns the food and the wood that a place can reach.
///
/// The test computes this for itself, from the public stock of each tile. It
/// does not read the score, because a test that asserted the score against
/// the score would assert nothing.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2. `.claude/rules/testing.md`
fn reachable_stock(world: &World, place: Axial) -> u64 {
    let mut total = 0u64;
    for address in founding::disc(world.grid(), place, SURVEY_RADIUS) {
        for kind in [ResourceKind::Food, ResourceKind::Wood] {
            total += u64::from(world.tile_stock(address, kind).map_or(0, |amount| amount.0));
        }
    }
    total
}

#[test]
fn a_run_begins_with_a_group_of_the_size_the_caller_gave() {
    let mut world = world_of(0x0cac_4e77_0061);
    assert_eq!(world.soldiers().len(), 0, "the world starts with nobody");

    let founded = world
        .found_run(GROUP, FactionId(1))
        .expect("a place exists");

    assert_eq!(founded.people().len(), GROUP as usize);
    assert_eq!(world.soldiers().len(), GROUP);
    assert_eq!(
        world.settlement_on(founded.place()),
        Some(founded.settlement())
    );
    for person in founded.people() {
        let at = world.soldiers().address(*person).expect("the person lives");
        assert!(
            at.distance(founded.place()) <= SURVEY_RADIUS,
            "a founder stands outside the place that was chosen"
        );
        assert_eq!(world.soldiers().faction(*person), Some(FactionId(1)));
    }
}

#[test]
fn a_group_of_a_different_size_is_a_different_run() {
    // The size is an input. Nothing in the engine holds it.
    for group in [1u32, 8, 30, 120] {
        let mut world = world_of(0x0cac_4e77_0061);
        let founded = world
            .found_run(group, FactionId(0))
            .expect("a place exists");
        assert_eq!(founded.people().len(), group as usize);
        assert_eq!(world.soldiers().len(), group);
    }
}

#[test]
fn the_founding_refuses_a_group_of_nobody() {
    let mut world = world_of(0x0cac_4e77_0061);
    assert_eq!(
        world.found_run(0, FactionId(0)),
        Err(FoundingError::EmptyGroup)
    );
    assert_eq!(world.soldiers().len(), 0, "a refusal founded nothing");
}

#[test]
fn the_founding_refuses_a_group_that_no_place_in_the_sample_admits() {
    // The disc holds thirty-seven tiles, and the ground of each holds a few
    // units, so a group far above that fits nowhere. The founding reports the
    // refusal rather than widening the sample.
    let mut world = world_of(0x0cac_4e77_0061);
    let huge = SURVEY_RADIUS * 1_000_000;
    assert_eq!(
        world.found_run(huge, FactionId(0)),
        Err(FoundingError::NoPlaceFound(SAMPLE_SIZE))
    );
    assert_eq!(world.soldiers().len(), 0, "a refusal founded nothing");
    assert_eq!(world.settlements().len(), 0, "a refusal seated nothing");
}

#[test]
fn the_survey_cost_does_not_grow_with_the_world() {
    // Four worlds whose tile counts span a factor of sixty-four. The tiles
    // the survey reads stay under one ceiling, and no world extent enters
    // that ceiling.[^1]
    //
    // [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D1. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    let mut counts = Vec::new();
    for extent in [128u32, 256, 512, 1024] {
        let world = World::new(WorldConfig {
            width: extent,
            height: extent,
            seed: 0x0cac_4e77_0061,
            faction_count: 4,
        })
        .expect("the extent must describe a world");
        let survey = world.survey_founding(GROUP).expect("the survey must run");
        assert_eq!(
            survey.drawn(),
            SAMPLE_SIZE,
            "the sample size followed the extent"
        );
        assert!(
            survey.tiles_read() <= SURVEY_CEILING,
            "a world of {extent} by {extent} read {} tiles, above the ceiling \
             of {SURVEY_CEILING}",
            survey.tiles_read()
        );
        counts.push(survey.tiles_read());
    }
    // The largest world holds sixty-four times the tiles of the smallest. A
    // survey that read them all would read sixty-four times as many here.
    let smallest = counts.iter().copied().min().expect("four worlds ran");
    let largest = counts.iter().copied().max().expect("four worlds ran");
    assert!(
        largest < smallest * 2,
        "the tiles read grew from {smallest} to {largest} as the world grew"
    );
}

#[test]
fn a_watcher_asks_why_the_place_was_chosen() {
    let world = world_of(0x0cac_4e77_0061);
    let survey = world.survey_founding(GROUP).expect("the survey must run");
    let chosen = survey.chosen().expect("a place exists");

    // The report is the output of the choice. Nothing recomputes a score to
    // answer a question about it, so no copy can disagree with the choice
    // that was made.[^1]
    //
    // [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D5. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    assert_eq!(chosen.score(), chosen.provision().score());
    assert!(chosen.is_eligible());
    assert!(chosen.provision().room >= GROUP);
    assert_eq!(
        world.tile_kind(chosen.address()).map(TileKind::is_passable),
        Some(true)
    );

    // The watcher compares the place against the places that were not
    // chosen, and every eligible one of them scores at most what it scored.
    let mut eligible = 0;
    for other in survey.rejected() {
        if other.is_eligible() {
            eligible += 1;
            assert!(
                other.score().0 <= chosen.score().0,
                "a rejected place scored above the chosen one"
            );
        }
    }
    assert!(eligible > 0, "the sample offered no alternative to compare");
}

#[test]
fn a_different_seed_gives_a_different_place_and_the_new_place_answers_the_same_test() {
    let mut places = Vec::new();
    for ordinal in 0..24u64 {
        let seed = 0x0cac_4e77_0061u64
            .wrapping_mul(ordinal.wrapping_add(1))
            .wrapping_add(ordinal);
        let mut world = world_of(seed);
        let founded = world
            .found_run(GROUP, FactionId(0))
            .unwrap_or_else(|error| panic!("seed {seed:#x} found no place: {error}"));
        let place = founded.place();

        // The new place answers the same test as the old one.
        assert_eq!(
            world.tile_kind(place).map(TileKind::is_passable),
            Some(true)
        );
        assert_eq!(founded.people().len(), GROUP as usize);
        assert!(
            founded
                .survey()
                .chosen()
                .expect("the founding chose")
                .provision()
                .room
                >= GROUP
        );
        places.push(place);
    }
    places.sort_unstable_by_key(|place| (place.q, place.r));
    places.dedup();
    assert!(
        places.len() > 20,
        "twenty-four seeds gave only {} distinct places",
        places.len()
    );
}

#[test]
fn two_places_that_score_the_same_resolve_by_the_tile_index() {
    // The tie is constructed, not hoped for. The test walks a window of the
    // world, finds two eligible places whose scores are equal, and asserts
    // that the lower tile index wins in both input orders.[^1]
    //
    // [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D4. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    let world = world_of(0x0cac_4e77_0061);
    let field = world.resources();

    let mut scored: Vec<(Accum, Axial)> = Vec::new();
    let mut tie: Option<(Axial, Axial)> = None;
    'search: for q in 8..72i32 {
        for r in 8..72i32 {
            let here = Axial::new(q, r);
            let survey =
                founding::survey_addresses(field, &[here], GROUP).expect("one address must survey");
            let candidate = survey.candidates()[0];
            if !candidate.is_eligible() {
                continue;
            }
            if let Some((_, earlier)) = scored
                .iter()
                .find(|(score, _)| *score == candidate.score())
                .copied()
            {
                tie = Some((earlier, here));
                break 'search;
            }
            scored.push((candidate.score(), here));
        }
    }
    let (first, second) = tie.expect("the window held no two places of one score");

    let low = world.grid().index_of(first).expect("inside the world");
    let high = world.grid().index_of(second).expect("inside the world");
    let (lower, _upper) = if low.0 < high.0 {
        (first, second)
    } else {
        (second, first)
    };

    for order in [[first, second], [second, first]] {
        let survey = founding::survey_addresses(field, &order, GROUP).expect("the survey must run");
        let chosen = survey.chosen().expect("both places admit the group");
        assert_eq!(
            chosen.address(),
            lower,
            "the tie resolved by the order the places arrived in, not by the \
             stable key"
        );
    }
}

#[test]
fn a_group_founded_in_a_poor_place_does_worse_than_one_founded_in_a_good_place() {
    let world = world_of(0x0cac_4e77_0061);
    let survey = world.survey_founding(GROUP).expect("the survey must run");
    let good = survey.chosen().expect("a place exists").address();
    let poor = survey
        .candidates()
        .iter()
        .rev()
        .find(|candidate| candidate.is_eligible())
        .expect("the sample held a second eligible place")
        .address();
    assert_ne!(good, poor, "the sample held one eligible place only");

    // The difference is asserted on a quantity the test computes for itself,
    // from the public stock of each tile. Asserting the score against the
    // score would assert nothing.[^1]
    //
    // [^1]: Testing rules, section 2. `.claude/rules/testing.md`
    let good_stock = reachable_stock(&world, good);
    let poor_stock = reachable_stock(&world, poor);
    assert!(
        good_stock > poor_stock,
        "the chosen place reaches {good_stock} of food and wood, and the \
         poor place reaches {poor_stock}"
    );

    // One pair is not enough. A score that read nothing the ground holds
    // still beats the worst eligible place, by luck of the sample, and the
    // pair above then measures the fixture rather than the rule.[^2] Rank
    // every eligible place by the stock the test computed, and require the
    // chosen one to sit in the best quarter.
    //
    // [^2]: Testing rules, section 2a. `.claude/rules/testing.md`
    let mut eligible: Vec<u64> = survey
        .candidates()
        .iter()
        .filter(|candidate| candidate.is_eligible())
        .map(|candidate| reachable_stock(&world, candidate.address()))
        .collect();
    assert!(
        eligible.len() >= 8,
        "the sample held only {} eligible places to rank",
        eligible.len()
    );
    eligible.sort_unstable();
    let quarter = eligible.len() - eligible.len() / 4;
    let threshold = eligible[quarter.min(eligible.len() - 1)];
    assert!(
        good_stock >= threshold,
        "the chosen place reaches {good_stock} of food and wood, which is \
         below the best quarter of the {} eligible places, at {threshold}. \
         The score does not read what the ground holds",
        eligible.len()
    );

    // Both foundings succeed. A poor place is a worse place, not an illegal
    // one, and a group founded in one is what a later rule acts on.
    let mut good_world = world_of(0x0cac_4e77_0061);
    let mut poor_world = world_of(0x0cac_4e77_0061);
    good_world
        .found_group_at(good, GROUP, FactionId(0))
        .expect("the good place admits the group");
    poor_world
        .found_group_at(poor, GROUP, FactionId(0))
        .expect("the poor place admits the group");
    assert_eq!(good_world.soldiers().len(), poor_world.soldiers().len());
}

#[test]
fn open_water_beside_a_place_raises_its_score() {
    // The water term is one of the properties the score reads, and a term
    // that nothing exercises ships inert.[^1] This test names two provisions
    // that differ in that term alone.
    //
    // [^1]: Recurring defect shapes, shape 3. `.claude/rules/recurring-defects.md`
    let dry = Provision {
        open_ground: 20,
        room: 160,
        water_edge: 0,
        ..Provision::default()
    };
    let shore = Provision {
        water_edge: 2,
        ..dry
    };
    assert!(
        shore.score().0 > dry.score().0,
        "open water beside a place did not change its score"
    );
}

#[test]
fn the_world_reserves_its_storage_once_and_does_not_grow_it_during_a_run() {
    let mut small = world_of(0x0cac_4e77_0061);
    let mut large = world_of(0x0cac_4e77_0061);

    let reserved = small.soldiers().capacity();
    small
        .found_run(GROUP, FactionId(0))
        .expect("a place exists");
    large
        .found_run(GROUP * 8, FactionId(0))
        .expect("a place exists");

    // The reserved storage is sized for the target, not for the group. A run
    // of thirty people and a run of two hundred and forty reserve the same.
    assert_eq!(small.soldiers().capacity(), reserved);
    assert_eq!(large.soldiers().capacity(), reserved);

    // The cost of a tick grows with the units that live, and the slots the
    // world holds do not change while it runs.
    let slots = small.soldiers().slot_count();
    assert_eq!(slots, GROUP, "the founding took more slots than it filled");
    for _ in 0..32 {
        small.step(2).expect("the step must run");
    }
    assert_eq!(
        small.soldiers().slot_count(),
        slots,
        "the world grew its storage during a run"
    );
    assert_eq!(small.soldiers().capacity(), reserved);
    assert_eq!(small.soldiers().len(), GROUP);
}

#[test]
fn the_same_seed_gives_the_same_place_and_the_same_group_at_every_thread_count() {
    let at = |threads: usize| {
        let mut world = world_of(0x0cac_4e77_0061);
        let founded = world
            .found_run(GROUP, FactionId(0))
            .expect("a place exists");
        let place = founded.place();
        let mut standing: Vec<(i32, i32)> = founded
            .people()
            .iter()
            .map(|person| {
                let at = world.soldiers().address(*person).expect("the person lives");
                (at.q, at.r)
            })
            .collect();
        standing.sort_unstable();
        for _ in 0..4 {
            world.step(threads).expect("the step must run");
        }
        (place, standing, world.state_hash().finish())
    };
    let one = at(1);
    assert_eq!(one, at(2), "two threads founded a different run");
    assert_eq!(one, at(12), "twelve threads founded a different run");
}

#[test]
fn the_column_draw_and_the_row_draw_are_two_draws() {
    // A key that gave the column and the row one draw index would put every
    // candidate on the diagonal of the world. Both determinism tests would
    // still pass, because the defect repeats on every run and at every thread
    // count. Only a test of the key itself sees it.[^1]
    //
    // [^1]: Testing rules, section 2. `.claude/rules/testing.md`
    let world = world_of(0x0cac_4e77_0061);
    let survey = world.survey_founding(GROUP).expect("the survey must run");

    let mut columns: Vec<i32> = survey
        .candidates()
        .iter()
        .map(|candidate| candidate.address().q)
        .collect();
    let mut rows: Vec<i32> = survey
        .candidates()
        .iter()
        .map(|candidate| candidate.address().r)
        .collect();
    columns.sort_unstable();
    columns.dedup();
    rows.sort_unstable();
    rows.dedup();

    assert!(
        rows.len() > SAMPLE_SIZE as usize / 2,
        "the sample covers only {} rows, so the row draw is not in the key",
        rows.len()
    );
    assert!(
        columns.len() > SAMPLE_SIZE as usize / 2,
        "the sample covers only {} columns",
        columns.len()
    );

    let on_the_diagonal = survey
        .candidates()
        .iter()
        .filter(|candidate| candidate.address().q == candidate.address().r)
        .count();
    assert!(
        on_the_diagonal < survey.considered() / 2,
        "the column and the row share a draw index"
    );
}

#[test]
fn the_seed_reaches_the_sample() {
    // The world seed is in the draw key, so a different seed gives a
    // different sample. A key that dropped it would give one sample for every
    // world, and every determinism test would still pass.
    let first = world_of(0x0cac_4e77_0061)
        .survey_founding(GROUP)
        .expect("the survey must run");
    let second = world_of(0x0cac_4e77_0062)
        .survey_founding(GROUP)
        .expect("the survey must run");
    let places = |survey: &founding::Survey| {
        let mut out: Vec<(i32, i32)> = survey
            .candidates()
            .iter()
            .map(|candidate| (candidate.address().q, candidate.address().r))
            .collect();
        out.sort_unstable();
        out
    };
    assert_ne!(
        places(&first),
        places(&second),
        "two seeds drew one sample, so the seed is not in the key"
    );
}
