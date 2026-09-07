//! A faction reads one flat array, and the array says only what it observes.
//!
//! Every test here drives the world step. The step runs the observation pass
//! that fills the fog layers, so a test that filled a layer itself would
//! prove that the reader works and not that anything reaches it.[^1]
//!
//! **Every fixture asserts that it produced the case the test needs.** A
//! world chosen to look right supplies no extreme, so each fixture states
//! the distribution it needs and fails when the world does not give it.[^2]
//!
//! **The fog test below compares against the truth of the same cell.** The
//! fixture asserts that the whole-world summary of the far cell holds tiles
//! and height. A reader that leaked the truth would therefore write those
//! values into the array, and the assertion would fail. A test that compared
//! a zero against a zero would prove nothing.
//!
//! # References
//!
//! [^1]: Testing rules, drive the real caller. `.agents/rules/testing.md`
//! [^2]: Testing rules, section 2a. `.agents/rules/testing.md`

use cachette_core::faction_observation::OBSERVATION_VERSION;
use cachette_core::{Axial, Entity, FactionId, SightRules, World, WorldConfig};

/// A world wide enough to hold cells that one faction never reaches.
///
/// The block edge of the lattice is smaller than the world, so the world
/// covers more than one cell. A faction that camps in one corner of it can
/// never have seen the far corner.
const WIDE: WorldConfig = WorldConfig {
    width: 96,
    height: 96,
    seed: 0x00c0_ffee_0123_4567,
    faction_count: 3,
    unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
};

/// The faction that watches in every fixture below.
const WATCHER: FactionId = FactionId(0);

/// The exponent that keeps a unit still.
///
/// A unit takes a movement intent at the interval its cell schedules. A long
/// interval stops a unit taking one inside a short test, so each test below
/// measures the reader and not the movement pass.
const KEEP_STILL: u32 = 12;

/// Builds a world in which no unit takes a movement intent.
fn a_still_world() -> World {
    let mut world = World::new(WIDE).expect("the configuration describes a world");
    world
        .set_choice_schedule(KEEP_STILL)
        .expect("the exponent is inside the range");
    world.set_sight_rules(SightRules::new(4, 1, 16, 0));
    world
}

/// Returns an address that admits a unit, near the one asked for.
///
/// The terrain comes from the seed, so the tile a test names may hold water.
/// The search is a spiral over the rings around the address, and it takes the
/// first tile of the lowest ring. The order is fixed, so two runs return one
/// answer.
fn ground_near(world: &World, wanted: Axial, bound: i32) -> Axial {
    for ring in 0..=bound {
        for column in -ring..=ring {
            for row in -ring..=ring {
                if column.abs().max(row.abs()) != ring {
                    continue;
                }
                let candidate = Axial::new(wanted.q + column, wanted.r + row);
                if world.admits_a_unit(candidate) {
                    return candidate;
                }
            }
        }
    }
    panic!("the fixture found no ground that admits a unit near {wanted:?}");
}

/// Puts a unit of one faction on a tile and returns its identity.
fn a_unit_at(world: &mut World, address: Axial, faction: FactionId) -> Entity {
    world
        .spawn_soldier(address, faction)
        .expect("the fixture places a unit on ground that admits one")
}

/// Returns the cell of the lattice that covers an address.
fn cell_of(world: &World, address: Axial) -> usize {
    let layout = world.observation().layout();
    let tile = world
        .grid()
        .index_of(address)
        .expect("the address lies inside the world");
    let key = layout.key_of(tile).expect("the tile carries a key");
    layout.block_of_key(key) as usize
}

/// Returns the positions of one field of the array.
///
/// The reader takes the start and the length from the schema. **No test
/// states a position of its own**, because a second declaration of the layout
/// would state the wrong thing the moment a field moved.[^1]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
fn field<'a>(world: &World, values: &'a [i64], name: &str) -> &'a [i64] {
    let row = world
        .observation_schema()
        .row(name)
        .unwrap_or_else(|| panic!("the schema declares a field named {name}"));
    let first = row.start as usize;
    &values[first..first + row.positions as usize]
}

/// Reads the observation of the watcher.
fn observation_of(world: &World, faction: FactionId) -> Vec<i64> {
    world
        .faction_observation(faction)
        .expect("the faction names a faction of this world")
}

/// The names of the fields that carry one position for each cell.
const CELL_FIELDS: [&str; 9] = [
    "cell_seen_now",
    "cell_seen_ever",
    "cell_tiles",
    "cell_open_tiles",
    "cell_units",
    "cell_held_tiles",
    "cell_value_total",
    "cell_height_total",
    "cell_food_total",
];

/// The schema covers the array exactly, with no gap and no overlap.
///
/// A caller decodes by arithmetic over the schema, so a start that did not
/// follow the field before it would put every later field in the wrong
/// place.[^1]
///
/// # References
///
/// [^1]: ADR-0154, decision D1. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
#[test]
fn the_schema_covers_the_array_exactly() {
    let mut world = a_still_world();
    let camp = ground_near(&world, Axial::new(8, 8), 8);
    a_unit_at(&mut world, camp, WATCHER);
    world.step(1).expect("the step runs");

    let schema = world.observation_schema();
    let values = observation_of(&world, WATCHER);

    assert_eq!(
        schema.version(),
        OBSERVATION_VERSION,
        "the schema states the version of the layout"
    );
    assert_eq!(
        values.len(),
        schema.length() as usize,
        "the array holds the positions the schema declares"
    );
    assert!(
        !schema.rows().is_empty(),
        "the schema declares at least one field"
    );

    let mut next = 0u32;
    for row in schema.rows() {
        assert_eq!(
            row.start,
            next,
            "the field {} starts after the field before it",
            row.name()
        );
        assert!(
            row.positions > 0,
            "the field {} holds at least one position",
            row.name()
        );
        assert!(
            row.low <= row.high,
            "the field {} states a lower bound below its upper bound",
            row.name()
        );
        assert_eq!(row.width(), 8, "every position is an eight-byte integer");
        next += row.positions;
    }
    assert_eq!(
        next,
        schema.length(),
        "the fields fill the array and leave no gap"
    );
}

/// A cell the faction has never seen reads as nothing at all.
///
/// **This is the test the reader exists for.** The fixture asserts that the
/// far cell holds tiles and height in the truth, so a reader that reported
/// the truth would write those values here and fail both assertions.
#[test]
fn a_cell_the_faction_has_never_seen_reads_as_nothing() {
    let mut world = a_still_world();
    let camp = ground_near(&world, Axial::new(8, 8), 8);
    a_unit_at(&mut world, camp, WATCHER);
    world.step(1).expect("the step runs");

    let far = Axial::new(80, 80);
    let far_cell = cell_of(&world, far);
    let home_cell = cell_of(&world, camp);
    assert_ne!(
        far_cell, home_cell,
        "the fixture must put the far address in another cell"
    );

    // The fixture asserts the case. A cell of water alone would hold no
    // height and no open tile, and the test would then pass against a
    // reader that leaked the truth.
    let truth = world
        .summary_covering(far)
        .expect("the cell covers the address");
    assert!(
        truth.tiles() > 0,
        "the fixture must give the far cell tiles in the truth"
    );
    assert!(
        truth.height_total().0 > 0,
        "the fixture must give the far cell height in the truth"
    );

    let values = observation_of(&world, WATCHER);
    for name in CELL_FIELDS {
        if name == "cell_tiles" {
            continue;
        }
        let read = field(&world, &values, name)[far_cell];
        assert_eq!(
            read, 0,
            "the field {name} states nothing about a cell the faction never saw"
        );
    }

    // The tile count of a cell is a property of the lattice and not of the
    // faction, so it stands for every cell.
    assert_eq!(
        field(&world, &values, "cell_tiles")[far_cell],
        truth.tiles(),
        "every cell states how many tiles of the world it covers"
    );
    assert!(
        field(&world, &values, "cell_seen_ever")[home_cell] > 0,
        "the fixture must give the watcher sight of its own cell"
    );
}

/// A unit that walks into a cell opens that cell in the array.
///
/// The test drives the step, which is what runs the observation pass. It
/// reads the array before the walk and after it.
#[test]
fn a_unit_that_walks_into_a_cell_opens_it() {
    let mut world = a_still_world();
    let camp = ground_near(&world, Axial::new(8, 8), 8);
    let scout = a_unit_at(&mut world, camp, WATCHER);
    world.step(1).expect("the step runs");

    let target = ground_near(&world, Axial::new(48, 48), 8);
    let target_cell = cell_of(&world, target);
    let home_cell = cell_of(&world, camp);
    assert_ne!(
        target_cell, home_cell,
        "the fixture must send the scout into another cell"
    );

    let before = observation_of(&world, WATCHER);
    assert_eq!(
        field(&world, &before, "cell_seen_ever")[target_cell],
        0,
        "the fixture must start with the target cell unseen"
    );

    world
        .place_soldier(scout, target)
        .expect("the target ground admits a unit");
    world.step(1).expect("the step runs");
    let after = observation_of(&world, WATCHER);

    assert!(
        field(&world, &after, "cell_seen_now")[target_cell] > 0,
        "a cell the scout stands in reads as seen now"
    );
    assert!(
        field(&world, &after, "cell_open_tiles")[target_cell] > 0,
        "a cell the scout stands in states the ground it admits"
    );
    assert_eq!(
        field(&world, &after, "cell_units")[target_cell],
        1,
        "the scout counts in the cell it stands in"
    );

    // The cell the scout left keeps what the faction saw, and it loses the
    // present frame. A remembered tile adds the ground alone.
    assert_eq!(
        field(&world, &after, "cell_seen_now")[home_cell],
        0,
        "a cell the faction left reads as seen by nothing now"
    );
    assert_eq!(
        field(&world, &after, "cell_seen_ever")[home_cell],
        field(&world, &before, "cell_seen_ever")[home_cell],
        "a faction never forgets a place it saw"
    );
    assert_eq!(
        field(&world, &after, "cell_units")[home_cell],
        0,
        "a remembered cell reports no unit, whatever stands there now"
    );
    assert_eq!(
        field(&world, &after, "cell_height_total")[home_cell],
        field(&world, &before, "cell_height_total")[home_cell],
        "a remembered cell keeps the ground the faction saw"
    );
}

/// The array carries the quantity the wonder reader compares.
///
/// The standing of a faction reports the work toward a victory claim. The
/// wonder reader compares the claim itself, and the two are not the same
/// quantity.[^1] A learner that read the work alone could not see the thing
/// that ends its game.
///
/// # References
///
/// [^1]: Findings register, FND-568. `docs/FINDINGS.md`
#[test]
fn the_array_carries_the_claim_the_wonder_reader_compares() {
    let mut world = a_still_world();
    let camp = ground_near(&world, Axial::new(8, 8), 8);
    a_unit_at(&mut world, camp, WATCHER);
    world.step(1).expect("the step runs");

    let schema = world.observation_schema();
    let claim = schema
        .row("wonder_claim")
        .expect("the array carries the claim the wonder reader compares");
    let progress = schema
        .row("wonder_progress")
        .expect("the array carries the work toward a claim");
    assert_ne!(
        claim.start, progress.start,
        "the claim and the work are two positions, because they are two quantities"
    );
    assert_eq!(
        (claim.low, claim.high),
        (0, 1),
        "a victory claim holds zero or one, so the wonder path has no threshold"
    );

    let values = observation_of(&world, WATCHER);
    let standing = world.standing(WATCHER).expect("the faction stands");
    assert_eq!(
        field(&world, &values, "wonder_progress")[0],
        standing.wonder_progress,
        "the array reports the work the standing reports"
    );
    assert_eq!(
        field(&world, &values, "wonder_claim")[0],
        0,
        "no wonder stands in a world nobody built in"
    );
}

/// The length follows the world parameters and never the population.
#[test]
fn the_length_follows_the_world_and_not_the_population() {
    let mut world = a_still_world();
    let empty = world.observation_schema().length();
    assert_eq!(
        observation_of(&world, WATCHER).len(),
        empty as usize,
        "a world with no unit gives an array of the declared length"
    );

    let camp = ground_near(&world, Axial::new(8, 8), 8);
    for ring in 0..6i32 {
        for step in 0..6i32 {
            let address = Axial::new(camp.q + ring, camp.r + step);
            if world.admits_a_unit(address) {
                a_unit_at(&mut world, address, WATCHER);
            }
        }
    }
    world.step(1).expect("the step runs");
    assert!(
        world
            .standing(WATCHER)
            .expect("the faction stands")
            .live_units
            > 1,
        "the fixture must raise the population"
    );
    assert_eq!(
        world.observation_schema().length(),
        empty,
        "the declared length does not move when the population moves"
    );
    assert_eq!(
        observation_of(&world, WATCHER).len(),
        empty as usize,
        "the array does not grow with the population"
    );
}

/// Two thread counts give one array.
///
/// The summary level rebuilds over as many threads as the caller states, and
/// the array reads that level. A rebuild that took a thread completion order
/// would give two answers here.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
#[test]
fn two_thread_counts_give_one_array() {
    let build = |threads: usize| {
        let mut world = a_still_world();
        let camp = ground_near(&world, Axial::new(8, 8), 8);
        let scout = a_unit_at(&mut world, camp, WATCHER);
        world.step(threads).expect("the step runs");
        let target = ground_near(&world, Axial::new(48, 48), 8);
        world
            .place_soldier(scout, target)
            .expect("the target ground admits a unit");
        world.step(threads).expect("the step runs");
        observation_of(&world, WATCHER)
    };
    let one = build(1);
    let many = build(12);
    assert_eq!(
        one, many,
        "the array of one thread and the array of twelve threads agree"
    );
}
