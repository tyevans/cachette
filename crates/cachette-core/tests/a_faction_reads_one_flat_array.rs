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
use cachette_core::{
    Advert, Axial, Entity, FactionId, Holder, SightRules, World, WorldConfig, ADVERT_OFFERS,
};

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
const CELL_FIELDS: [&str; 11] = [
    "cell_seen_now",
    "cell_seen_ever",
    "cell_tiles",
    "cell_open_tiles",
    "cell_own_units",
    "cell_other_units",
    "cell_own_held_tiles",
    "cell_other_held_tiles",
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
        field(&world, &after, "cell_own_units")[target_cell],
        1,
        "the scout counts as a unit of its own faction in the cell it stands in"
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
        field(&world, &after, "cell_own_units")[home_cell],
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

/// The cell counts separate the units of the reader from every other unit.
///
/// **The fixture puts a rival on a tile the watcher sees, and the assertion
/// is that the two counts differ.** A fixture with only the watcher's own
/// units would read the same array whether the reader split the count or
/// summed it, so it would measure nothing.[^1]
///
/// The counts are relative to the faction that reads. No position of the
/// array is indexed by a faction, because a field indexed by the faction
/// multiplies the world by the faction count.[^2]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
/// [^2]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D3. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
#[test]
fn the_cell_counts_tell_an_own_unit_from_another() {
    let mut world = a_still_world();
    let home = ground_near(&world, Axial::new(8, 8), 8);
    let watcher_cell = cell_of(&world, home);

    a_unit_at(&mut world, home, WATCHER);
    // Two rivals stand beside the watcher, so the other count is neither the
    // own count nor one. A count of one could come from either side.
    let beside = ground_near(&world, Axial::new(home.q + 1, home.r), 4);
    let also = ground_near(&world, Axial::new(home.q, home.r + 1), 4);
    assert_ne!(beside, home, "the fixture needs a second tile");
    assert_ne!(also, home, "the fixture needs a third tile");
    assert_ne!(beside, also, "the two rivals must stand apart");
    assert_eq!(
        cell_of(&world, beside),
        watcher_cell,
        "the first rival must stand in the cell the watcher sees"
    );
    assert_eq!(
        cell_of(&world, also),
        watcher_cell,
        "the second rival must stand in the cell the watcher sees"
    );
    a_unit_at(&mut world, beside, FactionId(1));
    a_unit_at(&mut world, also, FactionId(2));

    world.step(1).expect("the step runs");
    let values = observation_of(&world, WATCHER);

    assert_eq!(
        field(&world, &values, "cell_own_units")[watcher_cell],
        1,
        "the watcher counts its own unit and no other"
    );
    assert_eq!(
        field(&world, &values, "cell_other_units")[watcher_cell],
        2,
        "the watcher counts both rivals together and names neither"
    );
}

/// The held counts separate the ground of the reader from the ground of
/// another faction.
///
/// **The fixture asserts that both counts are above zero before it compares
/// them.** A cell in which nobody holds anything reads zero in both, and a
/// test over such a cell would pass whatever the reader did.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
#[test]
fn the_held_counts_tell_own_ground_from_other_ground() {
    let mut world = a_still_world();
    world.set_sight_rules(SightRules::new(64, 1, 16, 0));
    let mine = ground_near(&world, Axial::new(8, 8), 8);
    let theirs = ground_near(&world, Axial::new(20, 8), 8);
    let cell = cell_of(&world, mine);
    assert_eq!(
        cell_of(&world, theirs),
        cell,
        "the fixture needs both cities inside one cell"
    );

    a_unit_at(&mut world, mine, WATCHER);
    world
        .found_settlement(mine, WATCHER)
        .expect("the ground admits a city of the watcher");
    world
        .found_settlement(theirs, FactionId(1))
        .expect("the ground admits a city of the rival");
    world.step(1).expect("the step runs");

    let values = observation_of(&world, WATCHER);
    let own = field(&world, &values, "cell_own_held_tiles")[cell];
    let other = field(&world, &values, "cell_other_held_tiles")[cell];

    assert!(
        own > 0,
        "the fixture must give the watcher held ground inside the cell"
    );
    assert!(
        other > 0,
        "the fixture must give the rival held ground inside the same cell"
    );
    // **The comparison is against a count taken another way.** This walk
    // reads the holder of every tile of the cell the watcher sees now, from
    // the truth of the world. A test that compared the array against itself
    // would pass whatever the reader summed.
    let (truth_own, truth_other) = held_by_hand(&world, WATCHER, cell);
    assert_eq!(
        own, truth_own,
        "the own count is the ground the watcher holds"
    );
    assert_eq!(
        other, truth_other,
        "the other count is the ground every rival holds together"
    );
    assert_ne!(
        own, other,
        "the fixture must give the two sides different amounts of ground"
    );
    assert!(
        own + other <= field(&world, &values, "cell_seen_now")[cell],
        "the two counts never pass the tiles the watcher sees"
    );
}

/// Counts the held tiles of one cell by walking the world, for one faction
/// and for every other faction.
///
/// The walk reads the truth of the world and the sight rule of the faction,
/// and it takes the tiles the faction sees now. It visits the addresses in
/// ascending order, so two runs give one answer.
fn held_by_hand(world: &World, faction: FactionId, cell: usize) -> (i64, i64) {
    let (mut own, mut other) = (0i64, 0i64);
    for row in 0..world.grid().height() {
        for column in 0..world.grid().width() {
            let here = Axial::new(column as i32, row as i32);
            if cell_of(world, here) != cell || !world.faction_sees_now(faction, here) {
                continue;
            }
            match world.tile_holder(here).and_then(Holder::faction) {
                Some(holder) if holder == faction => own += 1,
                Some(_) => other += 1,
                None => {}
            }
        }
    }
    (own, other)
}

/// The layout version moves when the field set moves.
///
/// A stored weight file is a function of the field list, so a reader must be
/// able to tell a file written under one field set from a file written under
/// another.[^1]
///
/// # References
///
/// [^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, the consequences. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
#[test]
fn the_schema_reports_the_layout_version() {
    let world = a_still_world();
    assert_eq!(world.observation_schema().version(), OBSERVATION_VERSION);
    // **The version is pinned beside the field set, so neither moves alone.**
    // This once asserted that the version stood above one. That is a compile
    // time comparison of two constants: it is true whatever the fields do, so
    // it stated nothing and could never fail.
    //
    // The pair below can fail. Add or remove a field and the count moves, so
    // the assertion fails until the version moves with it, which is the whole
    // of the rule this test exists to hold.
    //
    // The pair catches a field the layout gained or lost. It cannot catch a
    // field that kept its length and changed its meaning, and the version
    // moves for that too. The addressing change that made this pair read
    // four is such a change: the field count did not move.
    assert_eq!(
        (OBSERVATION_VERSION, world.observation_schema().rows().len()),
        (5, 35),
        "the field set and the version must move together",
    );
}

// Every faction-indexed field is addressed relative to the reader.
//
// The two tests below read one world from two seats. A field addressed by a
// seat number would put a rival's quantities under the same position for one
// reader and under a different position for the other, and a policy that the
// league moves between seats would then read two things under one weight.[^1]
//
// [^1]: ADR-0193, a faction's observation names another faction by a position relative to the reader, decision D1. `docs/adrs/draft/adr-0193-an-observation-names-another-faction-by-a-position-relative-to-the-reader.md`

/// Builds a world of one faction count in which every faction-indexed
/// quantity differs from every other.
///
/// **The fixture asserts that it produced the case.** A world in which two
/// factions hold the same relation and post the same board would read the
/// same from either seat, whichever way the field is addressed, so the
/// assertions below would pass over a defect.[^1]
///
/// The relation of every ordered pair takes a value of its own, and the board
/// of every faction takes quantities of its own. The step runs first, so the
/// fog layers are filled by the pass that fills them and not by the test.
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
fn a_world_where_every_faction_differs(faction_count: u16) -> World {
    let config = WorldConfig {
        faction_count,
        ..WIDE
    };
    let mut world = World::new(config).expect("the configuration describes a world");
    world
        .set_choice_schedule(KEEP_STILL)
        .expect("the exponent is inside the range");
    world.set_sight_rules(SightRules::new(4, 1, 16, 0));

    let count = i32::from(faction_count);
    for from in 0..count {
        let camp = ground_near(&world, Axial::new(8 + from * 12, 8), 8);
        a_unit_at(&mut world, camp, FactionId(from as u16));
    }
    world.step(1).expect("the step runs");

    for from in 0..count {
        for to in 0..count {
            if from == to {
                continue;
            }
            let value = 1_000 * (from + 1) + (to + 1);
            assert!(
                world.set_relation(FactionId(from as u16), FactionId(to as u16), value),
                "the fixture writes the relation of every ordered pair"
            );
        }
        let rows: Vec<Advert> = (0..i32::from(world.board_rows()))
            .map(|row| Advert {
                good: ((from + row) % 3) as u8,
                wants: ADVERT_OFFERS,
                asking_good: ((from + row + 1) % 3) as u8,
                padding: 0,
                quantity: (100_000 * (from + 1) + row) as u32,
                asking_quantity: (200_000 * (from + 1) + row) as u32,
            })
            .collect();
        world
            .advertise(FactionId(from as u16), &rows)
            .expect("the board fits the bound and names a resource kind");
    }

    // The fixture asserts the distribution it needs: no two ordered pairs
    // hold one relation, and no two factions post one board quantity.
    let mut relations = Vec::new();
    let mut quantities = Vec::new();
    for from in 0..count {
        for to in 0..count {
            if from != to {
                relations.push(world.relation(FactionId(from as u16), FactionId(to as u16)));
            }
        }
        quantities.push(world.market(FactionId(from as u16))[0].quantity);
    }
    let mut sorted = relations.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        relations.len(),
        "the fixture needs a distinct relation for every ordered pair"
    );
    let mut sorted = quantities.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        quantities.len(),
        "the fixture needs a distinct board quantity for every faction"
    );
    world
}

/// The names of the fields that hold one block of board rows for each
/// faction.
const BOARD_FIELDS: [&str; 5] = [
    "board_good",
    "board_quantity",
    "board_wants",
    "board_asking_good",
    "board_asking_quantity",
];

/// Reads what one position of a board field holds for one faction.
fn advert_position(advert: &Advert, name: &str) -> i64 {
    match name {
        "board_good" => i64::from(advert.good),
        "board_quantity" => i64::from(advert.quantity),
        "board_wants" => i64::from(advert.wants),
        "board_asking_good" => i64::from(advert.asking_good),
        _ => i64::from(advert.asking_quantity),
    }
}

/// Returns the faction that one relative position of a faction-indexed field
/// names.
///
/// **The rule is written out here, and it is not taken from the engine.** A
/// test that read its expectation from the same mapping the writer uses would
/// agree with a broken mapping, and it would pass over the defect it exists to
/// catch.[^1]
///
/// The test states this rule, and it states no position of the array. A
/// position comes from the schema, in the way every other test here takes
/// one.[^2]
///
/// # References
///
/// [^1]: Testing rules, section 1. `.agents/rules/testing.md`
/// [^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
fn named_by(reader: FactionId, offset: u32, faction_count: u32) -> FactionId {
    FactionId(((u32::from(reader.0) + offset) % faction_count) as u16)
}

/// Asserts that every reader of one world finds one meaning at one relative
/// position.
///
/// The reader's own block comes first, and the rivals follow it in the
/// rotation the record states.
fn every_reader_agrees_on_the_relative_positions(faction_count: u16) {
    let world = a_world_where_every_faction_differs(faction_count);
    let count = u32::from(faction_count);
    let rows = u32::from(world.board_rows());

    let mut relation_by_reader = Vec::new();
    for seat in 0..faction_count {
        let reader = FactionId(seat);
        let values = observation_of(&world, reader);

        assert_eq!(
            field(&world, &values, "faction")[0],
            i64::from(seat),
            "the array names the faction that reads it"
        );

        let relation = field(&world, &values, "relation");
        assert_eq!(relation.len(), count as usize);
        for offset in 0..count {
            let other = named_by(reader, offset, count);
            assert_eq!(
                relation[offset as usize],
                i64::from(world.relation(reader, other).unwrap_or(0)),
                "reader {seat} holds the relation toward the faction {offset} seats after it"
            );
        }
        relation_by_reader.push(relation.to_vec());

        for name in BOARD_FIELDS {
            let block = field(&world, &values, name);
            assert_eq!(block.len(), (count * rows) as usize);
            for offset in 0..count {
                let other = named_by(reader, offset, count);
                let board = world.market(other);
                for row in 0..rows {
                    assert_eq!(
                        block[(offset * rows + row) as usize],
                        advert_position(&board[row as usize], name),
                        "reader {seat} holds the {name} of the faction {offset} seats after it"
                    );
                }
            }
            // The reader's own block comes first, whatever seat it holds.
            let own = world.market(reader);
            for row in 0..rows {
                assert_eq!(
                    block[row as usize],
                    advert_position(&own[row as usize], name),
                    "reader {seat} holds its own {name} in the first block"
                );
            }
        }
    }

    // Two readers of one world must not read one relation vector. A field
    // that answered the same for every seat would carry no relation at all,
    // and the assertions above would hold over it.
    for (seat, relation) in relation_by_reader.iter().enumerate().skip(1) {
        assert_ne!(
            relation, &relation_by_reader[0],
            "seat {seat} reads its own relations and not seat zero's"
        );
    }
}

/// Two seats of one world read one meaning at one relative position, at the
/// training faction count.
#[test]
fn two_seats_of_one_world_agree_on_the_relative_positions() {
    every_reader_agrees_on_the_relative_positions(3);
}

/// The rule holds at a faction count the training shape does not use.
///
/// A rule that only held at three factions would be a coincidence of the
/// shape the trainer runs.
#[test]
fn the_relative_positions_hold_at_five_factions() {
    every_reader_agrees_on_the_relative_positions(5);
}
