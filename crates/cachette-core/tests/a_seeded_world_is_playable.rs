//! The seeding of a world gives a playable world, or it names its refusal.
//!
//! **The demonstration draws a new seed on every run**, so a seed that gives
//! an unplayable world is a run a watcher sees and learns nothing from. The
//! tests below state what playable means and check it over fixed seeds.
//!
//! A seeded world is playable when all four statements below hold.
//!
//! 1. Every faction the caller asked for is seated.
//! 2. Every seat stands on ground that admits a unit.
//! 3. The ground around every seat holds the whole group that settles there.
//! 4. Two seats keep the minimum distance between them.
//!
//! **The seeding may refuse a faction, and the refusal is a named outcome.**
//! A world too small to hold the seats at that distance cannot seat them all,
//! and no draw of the sample changes that. The seeding then reports the
//! refusal for the faction it could not seat, and the caller draws another
//! seed or asks for fewer factions. The record refuses a sample that widens
//! until it succeeds, because that is a pass over every tile with extra
//! steps.[^1] [^2]
//!
//! **These tests drive the seeding the demonstration drives.** Each one builds
//! the world through the world constructor and calls the one seeding verb.
//! None of them founds a group by hand, and none of them turns a draw
//! off.[^3]
//!
//! **The seeds are not the typical world alone.** The list holds seeds that
//! finished no project in a measured run beside seeds that finished many, and
//! the small world below supplies the extreme that the demonstration extent
//! never reaches.[^4] [^5]
//!
//! # References
//!
//! [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D1. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
//! [^2]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decisions D1 and D2. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
//! [^3]: Testing rules, section 5. `.agents/rules/testing.md`
//! [^4]: Testing rules, section 2a. `.agents/rules/testing.md`
//! [^5]: Findings register, FND-544. `docs/FINDINGS.md`

use cachette_core::founding::MINIMUM_FOUNDING_DISTANCE;
use cachette_core::terrain::TileKind;
use cachette_core::{
    Axial, FoundingError, FoundingOutcome, World, WorldConfig, FOUNDING_GROUP_DEFAULT,
};

/// The extent of the world the demonstration builds.
const DEMONSTRATION_EXTENT: u32 = 256;

/// The factions the demonstration world holds.
const DEMONSTRATION_FACTIONS: u16 = 4;

/// The seeds that the demonstration test below takes.
///
/// **The list mixes the ends of a measured distribution.** A sweep of 200
/// seeds of this world ran 800 ticks each. Four seeds here finished no project
/// in that sweep, and four finished many. A list of seeds that all behave
/// alike would measure the list.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-544. `docs/FINDINGS.md`
const SEEDS: [u64; 8] = [20, 21, 77, 183, 2, 45, 118, 200];

/// Builds the world the demonstration builds, and seeds it.
fn seed_a_demonstration_world(seed: u64) -> Vec<FoundingOutcome> {
    let mut world = World::new(WorldConfig {
        width: DEMONSTRATION_EXTENT,
        height: DEMONSTRATION_EXTENT,
        seed,
        faction_count: DEMONSTRATION_FACTIONS,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    let outcomes = world.seed_world().expect("the world seeds once");
    check_every_seat(&world, &outcomes, seed);
    outcomes
}

/// Checks statements two, three and four over the seats of one world.
fn check_every_seat(world: &World, outcomes: &[FoundingOutcome], seed: u64) {
    let mut seats: Vec<Axial> = Vec::new();
    for outcome in outcomes {
        let Some(founding) = outcome.founding() else {
            continue;
        };
        let place = founding.place();
        let ground = world.terrain().kind(place);
        assert!(
            ground.is_some_and(TileKind::is_passable),
            "seed {seed:#x} seated faction {} on {ground:?}, which admits no unit",
            outcome.faction().0
        );
        let room = founding
            .survey()
            .chosen()
            .expect("a seated founding chose a place")
            .provision()
            .room;
        assert!(
            room >= FOUNDING_GROUP_DEFAULT,
            "seed {seed:#x} seated faction {} where the ground holds {room} of \
             a group of {FOUNDING_GROUP_DEFAULT}",
            outcome.faction().0
        );
        for held in &seats {
            assert!(
                held.distance(place) >= MINIMUM_FOUNDING_DISTANCE,
                "seed {seed:#x} put two seats {} apart, and the rule asks for \
                 {MINIMUM_FOUNDING_DISTANCE}",
                held.distance(place)
            );
        }
        seats.push(place);
    }
}

/// Every seed of the list seats every faction of the demonstration world.
///
/// **This is statement one, and the seeding meets it by construction at this
/// extent.** The world is far wider than the distance the seats keep, and the
/// sample the founding draws covers the whole of it, so a refusal here would
/// name a world that holds no ground for a group of two.
#[test]
fn every_seed_seats_every_faction_of_the_demonstration_world() {
    for seed in SEEDS {
        let outcomes = seed_a_demonstration_world(seed);
        let seated = outcomes
            .iter()
            .filter(|outcome| outcome.founding().is_some())
            .count();
        assert_eq!(
            seated, DEMONSTRATION_FACTIONS as usize,
            "seed {seed:#x} seated {seated} of {DEMONSTRATION_FACTIONS} factions"
        );
    }
}

/// Seeds a world of one extent and one seed, and checks every seat it made.
fn seed_a_small_world(side: u32, seed: u64) -> (World, Vec<FoundingOutcome>) {
    let mut world = World::new(WorldConfig {
        width: side,
        height: side,
        seed,
        faction_count: DEMONSTRATION_FACTIONS,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    let outcomes = world.seed_world().expect("the world seeds once");
    check_every_seat(&world, &outcomes, seed);
    (world, outcomes)
}

/// Returns how many outcomes seated a faction.
fn seated_count(outcomes: &[FoundingOutcome]) -> usize {
    outcomes
        .iter()
        .filter(|outcome| outcome.founding().is_some())
        .count()
}

/// Asserts that every outcome without a seat names its refusal.
fn every_refusal_is_named(outcomes: &[FoundingOutcome]) {
    for outcome in outcomes {
        if outcome.founding().is_some() {
            continue;
        }
        assert!(
            matches!(outcome.result(), Err(FoundingError::NoPlaceFound(_))),
            "the seeding refused faction {} without naming the refusal: {:?}",
            outcome.faction().0,
            outcome.result()
        );
    }
}

/// The side of the world whose ground is scarce.
const SCARCE_EXTENT: u32 = 32;

/// The seed of the scarce world below.
const SCARCE_SEED: u64 = 22;

/// The seats that the scarce world holds.
///
/// **The count is the assertion, and a count is what a weakened rule moves.**
/// A survey that took a place admitting no unit would waste one of its
/// candidates, and the founding would then refuse a faction this world can
/// seat. A looser statement passes with that defect in place, because the
/// refusal looks like the ground and not like the rule.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
const SCARCE_SEATS: usize = 3;

/// A world too small for four seats seats what it can and names each refusal.
///
/// **This is the extreme the demonstration extent never supplies.** Two seats
/// keep a fixed distance, this world holds few places at that distance, and
/// much of its ground admits no unit. The seeding seats what the world holds
/// and names the refusal for the rest.[^1]
///
/// # References
///
/// [^1]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decisions D1 and D2. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
#[test]
fn a_world_too_small_for_four_seats_names_every_refusal() {
    let (_, outcomes) = seed_a_small_world(SCARCE_EXTENT, SCARCE_SEED);
    let seated = seated_count(&outcomes);
    assert_eq!(
        seated, SCARCE_SEATS,
        "a world of side {SCARCE_EXTENT} at seed {SCARCE_SEED} seated \
         {seated} factions"
    );
    every_refusal_is_named(&outcomes);
}

/// A world with no admissible ground seats nobody, and it names every refusal.
///
/// **A run of such a world has nothing in it, and the seeding says so.** The
/// engine does not widen the sample and it does not draw again. The caller
/// reads the refusals and asks for another seed or for a wider world.[^1]
///
/// # References
///
/// [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D1. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
#[test]
fn a_world_with_no_place_for_a_group_seats_nobody_and_says_so() {
    let side = MINIMUM_FOUNDING_DISTANCE / 2;
    let (_, outcomes) = seed_a_small_world(side, 10);
    assert_eq!(
        seated_count(&outcomes),
        0,
        "a world of side {side} at this seed holds a place after all"
    );
    every_refusal_is_named(&outcomes);
}
