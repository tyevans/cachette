//! A faction reads one flat array, and the array says only what it observes.
//!
//! Every test here drives the world step. The step runs the observation pass
//! that fills the fog layers, so a test that filled a layer itself would
//! prove that the reader works and not that anything reaches it.[^1]
//!
//! **Every fixture asserts that it produced the case the test needs.** A
//! world chosen to look right supplies no extreme, so each fixture states the
//! distribution it needs and fails when the world does not give it.[^2]
//!
//! # What these tests protect
//!
//! The layout replaced a layout whose width followed the world shape and the
//! faction count. A policy trained against one shape could not read
//! another.[^3] Two tests here state that property: one compares the schema
//! across several world shapes and faction counts, and one compares the
//! values of two worlds of different size.
//!
//! The layout also publishes no raw count. Every position is a share, a
//! signed relation or a compressed magnitude, and each of the three is
//! bounded. One test builds a world that produces a value above the bound and
//! asserts that the bound holds there. **A test against a typical world would
//! never receive the input that fails.**[^2]
//!
//! # References
//!
//! [^1]: Testing rules, drive the real caller. `.agents/rules/testing.md`
//! [^2]: Testing rules, section 2a. `.agents/rules/testing.md`
//! [^3]: Findings register, FND-670. `docs/FINDINGS.md`

use cachette_core::faction_observation::{
    observation_schema, ObsField, ValueForm, ValueKind, OBSERVATION_VERSION,
};
use cachette_core::obs_ring::{ring_cell_counts, RING_STACK_CELLS, RING_STACK_CHANNELS};
use cachette_core::obs_ring_stack::RING_STACK_CHANNEL_NAMES;
use cachette_core::obs_token::TokenSet;
use cachette_core::sim_math;
use cachette_core::unit_type::SETTLER;
use cachette_core::{Axial, Entity, FactionId, SightRules, World, WorldConfig};

/// A world wide enough to hold ground that one faction never reaches.
const WIDE: WorldConfig = WorldConfig {
    width: 96,
    height: 96,
    seed: 0x00c0_ffee_0123_4567,
    faction_count: 3,
    unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
};

/// The faction that watches in every fixture below.
const WATCHER: FactionId = FactionId(0);

/// One unit of the fixed-point scale of this project.
const ONE: i64 = 65536;

/// The exponent that keeps a unit still.
///
/// A unit takes a movement intent at the interval its cell schedules. A long
/// interval stops a unit taking one inside a short test, so each test below
/// measures the reader and not the movement pass.
const KEEP_STILL: u32 = 12;

/// Builds a world in which no unit takes a movement intent.
fn a_still_world_of(config: WorldConfig) -> World {
    let mut world = World::new(config).expect("the configuration describes a world");
    world
        .set_choice_schedule(KEEP_STILL)
        .expect("the exponent is inside the range");
    world.set_sight_rules(SightRules::new(4, 1, 16, 0));
    world
}

/// Builds the wide world in which no unit takes a movement intent.
fn a_still_world() -> World {
    a_still_world_of(WIDE)
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

/// Returns the positions of one field of the array.
///
/// The reader takes the start and the length from the schema. **No test
/// states a position of its own**, because a second declaration of the layout
/// would state the wrong thing the moment a field moved.[^1]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
fn field<'a>(values: &'a [i64], name: &str) -> &'a [i64] {
    let row = observation_schema()
        .row(name)
        .unwrap_or_else(|| panic!("the schema declares a field named {name}"));
    let first = row.start as usize;
    &values[first..first + row.positions as usize]
}

/// Returns the one position of a field that holds one position.
fn one(values: &[i64], name: &str) -> i64 {
    let span = field(values, name);
    assert_eq!(span.len(), 1, "the field {name} holds one position");
    span[0]
}

/// Reads the observation of one faction.
fn observation_of(world: &World, faction: FactionId) -> Vec<i64> {
    world
        .faction_observation(faction)
        .expect("the number names a faction of this world")
}

/// The schema covers the array exactly, with no gap and no overlap.
///
/// **The count includes every reserved range.** A reserved block holds its
/// declared positions so that every later block starts where the design puts
/// it, and a builder that fills one moves nothing.[^1]
///
/// A caller decodes by arithmetic over the schema, so a start that did not
/// follow the field before it would put every later field in the wrong
/// place.[^2]
///
/// # References
///
/// [^1]: Research report 42, what a policy should be able to see, section 9. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
/// [^2]: ADR-0154, decision D1. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
#[test]
fn the_schema_covers_the_array_exactly() {
    let schema = observation_schema();
    let mut owner = vec![usize::MAX; schema.length() as usize];
    for (at, row) in schema.rows().iter().enumerate() {
        assert!(
            row.positions > 0,
            "the field {} holds a position",
            row.name()
        );
        for offset in 0..row.positions {
            let place = (row.start + offset) as usize;
            assert_eq!(
                owner[place],
                usize::MAX,
                "the position {place} belongs to the field {} and to another",
                row.name()
            );
            owner[place] = at;
        }
    }
    assert!(
        owner.iter().all(|held| *held != usize::MAX),
        "every position of the array belongs to a field"
    );
    assert_eq!(
        schema.length() as usize,
        observation_of(&a_still_world(), WATCHER).len(),
        "the array holds the length the schema declares"
    );
}

/// The width is one number for every world shape and every faction count.
///
/// **This is the property the layout exists to hold.** A field with one
/// position for each faction multiplied by the faction count, and a field
/// with one position for each cell of the block lattice multiplied by the
/// cell count. The width was a function of the width, the height and the
/// faction count, and a policy trained against one triple could not read
/// another.[^1]
///
/// The fixture asserts that the shapes it built really differ, so a fixture
/// that built one world five times would fail rather than pass.
///
/// # References
///
/// [^1]: Findings register, FND-670. `docs/FINDINGS.md`
#[test]
fn the_width_is_one_number_for_every_world_shape() {
    let declared = observation_schema();
    let shapes = [
        (24u32, 24u32, 2u16),
        (48, 48, 3),
        (96, 96, 7),
        (128, 64, 12),
    ];
    let mut tile_counts = Vec::new();
    for (width, height, factions) in shapes {
        let world = a_still_world_of(WorldConfig {
            width,
            height,
            faction_count: factions,
            ..WIDE
        });
        tile_counts.push(world.grid().tile_count());
        let values = observation_of(&world, WATCHER);
        assert_eq!(
            values.len(),
            declared.length() as usize,
            "the array of the {width} by {height} world at {factions} factions holds the declared length"
        );
        let schema = observation_schema();
        for (row, expected) in schema.rows().iter().zip(declared.rows()) {
            assert_eq!(
                (row.field, row.start, row.positions),
                (expected.field, expected.start, expected.positions),
                "the field {} starts at one position in every world",
                row.name()
            );
        }
    }
    tile_counts.dedup();
    assert!(
        tile_counts.len() > 1,
        "the fixture must build worlds of different tile counts"
    );
}

/// Every position of the array lies inside the bounds the schema declares.
///
/// The fixture runs three worlds: one with nothing in it, one with a faction
/// that holds units and no ground, and one with several factions that hold
/// ground and see each other. A world with one shape would leave most fields
/// at zero, and zero lies inside every bound.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
#[test]
fn every_position_lies_inside_its_declared_bounds() {
    let mut crowded = a_still_world();
    let camp = ground_near(&crowded, Axial::new(8, 8), 8);
    for step in 0..8i32 {
        let here = Axial::new(camp.q + step, camp.r);
        if crowded.admits_a_unit(here) {
            a_unit_at(&mut crowded, here, WATCHER);
            a_unit_at(&mut crowded, here, FactionId(1));
        }
    }
    crowded.step(1).expect("the step runs");

    let mut lonely = a_still_world();
    let post = ground_near(&lonely, Axial::new(60, 60), 8);
    a_unit_at(&mut lonely, post, WATCHER);
    a_unit_at(&mut lonely, post, WATCHER);
    lonely.step(1).expect("the step runs");

    let empty = a_still_world();
    let schema = observation_schema();
    for world in [&crowded, &lonely, &empty] {
        let values = observation_of(world, WATCHER);
        for row in schema.rows() {
            for offset in 0..row.positions {
                let value = values[(row.start + offset) as usize];
                assert!(
                    value >= row.low && value <= row.high,
                    "the field {} holds {value} at offset {offset}, outside {} to {}",
                    row.name(),
                    row.low,
                    row.high
                );
            }
        }
    }
}

/// A reserved field reads zero in every position.
///
/// **A reserved field is not a zero that states a real quantity of zero.**
/// The schema declares its bounds as zero and zero, so a reader tells the two
/// apart.
///
/// The three spatial blocks are built, so none of them is reserved. The test
/// derives that relationship from the schema rather than naming a count, so
/// a later revision that claims another reserve does not make it false.
/// The schema publishes one field for each token set, of the width the set
/// states.
///
/// **A set the field list forgets leaves the schema with no gap.** The fields
/// that follow move down, the array shrinks, and every check of coverage
/// still passes. The set list is the one declaration of the four sets, so
/// this test walks it and asks the schema for each one.
#[test]
fn the_schema_publishes_one_field_for_each_token_set() {
    let schema = observation_schema();
    for set in TokenSet::ALL {
        let row = schema
            .row(set.name())
            .unwrap_or_else(|| panic!("the schema publishes the token set {}", set.name()));
        assert_eq!(
            row.positions,
            set.slots(),
            "the field {} holds the positions the set states",
            set.name()
        );
        assert_eq!(
            row.channels().len() as u32,
            set.channels(),
            "the field {} names one channel for each channel of a token",
            set.name()
        );
        assert_eq!(
            row.space(),
            Some("token"),
            "the field {} lays its positions out as tokens",
            set.name()
        );
    }
    let published = schema
        .rows()
        .iter()
        .filter(|row| row.space() == Some("token"))
        .count();
    assert_eq!(
        published,
        TokenSet::ALL.len(),
        "the schema publishes no token field the set list does not name"
    );
}

/// The published ring geometry accounts for every position of the block.
#[test]
fn the_published_ring_geometry_fills_the_ring_stack() {
    let schema = observation_schema();
    let row = schema
        .row("ring_stack")
        .expect("the schema publishes the ring stack");
    let counts = schema.ring_cells();
    assert_eq!(
        counts.iter().sum::<u32>(),
        RING_STACK_CELLS,
        "the published ring cell counts sum to the cells of the frame"
    );
    assert_eq!(
        row.channels().len() as u32,
        RING_STACK_CHANNELS,
        "the field names one channel for each channel of a cell"
    );
    assert_eq!(
        counts.iter().sum::<u32>() * row.channels().len() as u32,
        row.positions,
        "the cells and the channels fill the block"
    );
    assert_eq!(
        counts.as_ref(),
        ring_cell_counts(),
        "the schema publishes the cell counts the frame derives"
    );
    assert!(
        row.channels().contains(&schema.spatial_gate()),
        "the gate the schema names is a channel of the ring stack"
    );
    assert_eq!(RING_STACK_CHANNEL_NAMES.len() as u32, RING_STACK_CHANNELS);
}

#[test]
fn a_reserved_field_reads_zero() {
    let mut world = a_still_world();
    let camp = ground_near(&world, Axial::new(8, 8), 8);
    a_unit_at(&mut world, camp, WATCHER);
    world.step(1).expect("the step runs");
    let values = observation_of(&world, WATCHER);
    let schema = observation_schema();
    let spatial = [
        ObsField::RingStack,
        ObsField::FrontierBySector,
        ObsField::TokenOwnSettlements,
        ObsField::TokenRivals,
        ObsField::TokenThreatClusters,
        ObsField::TokenCandidateSites,
    ];
    for field in spatial {
        assert!(
            !field.value_kind().is_reserved(),
            "the block {} is built, so the schema must not call it reserved",
            field.name()
        );
    }
    let mut reserved = 0u32;
    for row in schema.rows() {
        if !row.field.value_kind().is_reserved() {
            continue;
        }
        assert!(
            row.positions > 0,
            "the reserved field {} holds a position, so filling it moves no later field",
            row.name()
        );
        reserved += row.positions;
        assert_eq!(
            (row.low, row.high),
            (0, 0),
            "the reserved field {} declares zero bounds",
            row.name()
        );
        for offset in 0..row.positions {
            assert_eq!(
                values[(row.start + offset) as usize],
                0,
                "the reserved field {} reads zero at offset {offset}",
                row.name()
            );
        }
    }
    assert!(
        reserved > 0,
        "the layout holds a reserve, and this test proves the reserve reads zero"
    );
    assert!(
        reserved < schema.length(),
        "the array holds a position that is not reserved"
    );
}

/// A share holds its bound on a world that produces a value above it.
///
/// **The fixture builds the extreme rather than a typical world.** The units
/// for each held tile divide the unit count by eight times the held tile
/// count. A faction with units and no settlement holds no ground, so the
/// denominator is zero. Without the clamp inside the share the position would
/// read the unit count times one unit of the scale, which is above the bound
/// by that factor.
///
/// The fixture asserts that it produced the case: the faction holds no
/// ground, and it holds more than one unit. With one unit the unclamped value
/// would land exactly on the bound and the assertion would prove nothing.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
#[test]
fn a_share_holds_its_bound_at_the_extreme() {
    let mut world = a_still_world();
    let post = ground_near(&world, Axial::new(60, 60), 8);
    for step in 0..3i32 {
        let here = Axial::new(post.q + step, post.r);
        if world.admits_a_unit(here) {
            a_unit_at(&mut world, here, WATCHER);
        }
    }
    world.step(1).expect("the step runs");
    let values = observation_of(&world, WATCHER);

    assert_eq!(
        world.holding_of(WATCHER),
        0,
        "the fixture must give the faction no ground"
    );
    assert!(
        i64::from(world.population_of(WATCHER)) > 1,
        "the fixture must give the faction more than one unit"
    );
    assert_eq!(
        one(&values, "units_for_each_held_tile"),
        ONE,
        "the share stops at one unit of the scale"
    );
    assert_eq!(
        one(&values, "held_tiles"),
        0,
        "a faction that holds nothing reads nothing"
    );
}

/// Two worlds of different size agree on a value that follows no world size.
///
/// A compressed magnitude of an absolute quantity states the quantity and not
/// its share of the world, so two worlds that hold the same quantity read the
/// same value. A share of a world total states the share, so the same two
/// worlds read different values.
///
/// **The fixture asserts that the two worlds really differ in size.** A
/// fixture that built one world twice would agree on every position and prove
/// nothing.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
#[test]
fn two_worlds_of_different_size_agree_on_a_scale_free_value() {
    let build = |width: u32, height: u32| {
        let mut world = a_still_world_of(WorldConfig {
            width,
            height,
            ..WIDE
        });
        let camp = ground_near(&world, Axial::new(8, 8), 8);
        for step in 0..4i32 {
            let here = Axial::new(camp.q + step, camp.r);
            if world.admits_a_unit(here) {
                a_unit_at(&mut world, here, WATCHER);
            }
        }
        world.step(1).expect("the step runs");
        (
            i64::from(world.population_of(WATCHER)),
            observation_of(&world, WATCHER),
        )
    };
    let (small_units, small) = build(24, 24);
    let (large_units, large) = build(96, 96);

    assert_eq!(
        small_units, large_units,
        "the fixture must raise the same unit count in both worlds"
    );
    assert!(small_units > 0, "the fixture must raise a unit");
    assert_eq!(
        small.len(),
        large.len(),
        "the two worlds give arrays of one length"
    );
    assert_ne!(
        one(&small, "world_tiles"),
        one(&large, "world_tiles"),
        "the fixture must build two worlds of different size"
    );
    assert_eq!(
        one(&small, "live_units"),
        one(&large, "live_units"),
        "the unit count reads the same value whatever the world size"
    );
    assert_eq!(
        one(&small, "seated_faction_share"),
        one(&large, "seated_faction_share"),
        "the seated faction share reads the same value whatever the world size"
    );
    assert!(
        one(&small, "observed_share_world") > one(&large, "observed_share_world"),
        "the observed share of the world falls when the world grows"
    );
}

/// Two thread counts give one array.
///
/// The summary level rebuilds over as many threads as the caller states, and
/// the passes that build the array read the world that rebuild left. A
/// rebuild that took a thread completion order would give two answers
/// here.[^1]
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
        a_unit_at(&mut world, camp, FactionId(1));
        world.step(threads).expect("the step runs");
        let target = ground_near(&world, Axial::new(48, 48), 8);
        world
            .place_soldier(scout, target)
            .expect("the target ground admits a unit");
        world.step(threads).expect("the step runs");
        observation_of(&world, WATCHER)
    };
    let one_thread = build(1);
    let many_threads = build(12);
    assert_eq!(
        one_thread, many_threads,
        "the array of one thread and the array of twelve threads agree"
    );
}

/// A change in a place the faction never saw changes nothing in its array.
///
/// **This is the cost bound and the fog rule together.** The passes that build
/// the array walk the ground the faction observed, so a place it never
/// reached contributes nothing and costs nothing.[^1]
///
/// The test builds the same world twice and raises the rival in one of them.
/// It then requires the two arrays to agree at every position. **The
/// comparison needs no exclusion.** A field that moves with the tick, such as
/// the clock, the ticks that remain, the weather phase or a weather channel
/// of the ring stack, advances the same way in both worlds, so only the rival
/// can make a position differ. A test that compared one world before and
/// after a step would have to excuse every such field, and each excuse is a
/// position the fog rule stops covering.
///
/// The fixture asserts that the faction never saw the far ground, and that
/// the rival really rose. A fixture that changed a place the faction watches
/// would fail, and a fixture that changed nothing would pass without proving
/// anything.
///
/// # References
///
/// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D2. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
#[test]
fn a_place_the_faction_never_saw_changes_nothing() {
    let plain = a_watched_world(None);
    let rivalled = a_watched_world(Some(Axial::new(88, 88)));
    let schema = observation_schema();
    for row in schema.rows() {
        for offset in 0..row.positions {
            let place = (row.start + offset) as usize;
            assert_eq!(
                plain[place],
                rivalled[place],
                "the field {} moved after a change the faction cannot see",
                row.name()
            );
        }
    }
}

/// Builds the watcher world, and raises a rival near one address of it.
///
/// The rival rises after the first step, so the watcher has already filled
/// its fog layers when the rival appears. The fixture asserts that the
/// watcher never saw the ground it puts the rival on, because a rival on
/// watched ground would prove nothing about the fog rule.
fn a_watched_world(rival: Option<Axial>) -> Vec<i64> {
    let mut world = a_still_world();
    let camp = ground_near(&world, Axial::new(8, 8), 8);
    a_unit_at(&mut world, camp, WATCHER);
    world.step(1).expect("the step runs");
    if let Some(wanted) = rival {
        let far = ground_near(&world, wanted, 6);
        assert!(
            !world.faction_has_seen(WATCHER, far),
            "the fixture must name ground the watcher never saw"
        );
        let raised = a_unit_at(&mut world, far, FactionId(1));
        assert!(
            world.soldier_faction(raised).is_some(),
            "the fixture must raise the rival unit"
        );
    }
    world.step(1).expect("the step runs");
    observation_of(&world, WATCHER)
}

/// The confidence statistic tells an unobserved estimate from a real zero.
///
/// A power quantity of a rival is an estimate from the ground the faction
/// observed. Without the confidence a policy reads an inferred quantity and a
/// seen quantity as one fact, and it cannot learn to scout.[^1]
///
/// The fixture builds both cases: a faction that observed nothing, and a
/// faction that observes ground and finds no rival on it.
///
/// # References
///
/// [^1]: Research report 42, what a policy should be able to see, section 6.3. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
#[test]
fn the_confidence_tells_an_unobserved_estimate_from_a_zero() {
    let blind = a_still_world();
    assert_eq!(
        blind.faction_seen_ever(WATCHER),
        0,
        "the fixture must give the faction no observed ground"
    );
    let unseen = observation_of(&blind, WATCHER);
    let statistics = field(&unseen, "power_held_tiles");
    assert_eq!(
        statistics[statistics.len() - 1],
        0,
        "a faction that observed nothing reads no confidence"
    );

    let mut watching = a_still_world();
    let camp = ground_near(&watching, Axial::new(8, 8), 8);
    a_unit_at(&mut watching, camp, WATCHER);
    watching.step(1).expect("the step runs");
    assert!(
        watching.faction_seen_ever(WATCHER) > 0,
        "the fixture must give the faction observed ground"
    );
    let seen = observation_of(&watching, WATCHER);
    let statistics = field(&seen, "power_held_tiles");
    assert!(
        statistics[statistics.len() - 1] > 0,
        "a faction that observes ground reads a confidence above zero"
    );
}

/// No field of the layout names a seat.
///
/// A block indexed by seat teaches a policy a seat number, and a league seats
/// one policy in one seat for one game and in another seat for the next.[^1]
/// The standing of the rivals arrives as order statistics, whose width does
/// not follow the faction count.
///
/// The check is the width: a field whose length followed the faction count
/// would change length between two worlds of different faction counts, and
/// the schema is one schema.
///
/// # References
///
/// [^1]: Findings register, FND-647. `docs/FINDINGS.md`
#[test]
fn no_field_of_the_layout_follows_the_faction_count() {
    let two = a_still_world_of(WorldConfig {
        faction_count: 2,
        ..WIDE
    });
    let many = a_still_world_of(WorldConfig {
        faction_count: 12,
        ..WIDE
    });
    assert_ne!(
        two.faction_count(),
        many.faction_count(),
        "the fixture must build two worlds of different faction counts"
    );
    assert_eq!(
        observation_of(&two, WATCHER).len(),
        observation_of(&many, WATCHER).len(),
        "the array holds one length at two faction counts and at twelve"
    );
}

/// The schema reports the version of the layout.
///
/// A field added, removed, relengthened or rebounded changes the meaning of a
/// stored weight file, so a learner that loads a policy under another version
/// must stop.[^1]
///
/// # References
///
/// [^1]: ADR-0154, the consequences. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
#[test]
fn the_schema_reports_the_layout_version() {
    assert_eq!(
        observation_schema().version(),
        OBSERVATION_VERSION,
        "the schema carries the version of the layout"
    );
}

/// The exponents at which a compressed magnitude inverts exactly.
///
/// The compression divides a Q16.16 logarithm by the cap width and truncates,
/// so a general inversion carries a small relative error. The division leaves
/// no remainder where the cap width divides the scaled exponent, and at those
/// exponents the round trip is exact. The list gives such exponents, and the
/// helper asserts that each one is one of them rather than trusting this
/// comment.
const EXACT_EXPONENTS: [u32; 8] = [0, 5, 10, 15, 20, 25, 30, 35];

/// Returns the value form the schema publishes under one name.
fn form_named(name: &str) -> ValueForm {
    observation_schema()
        .value_forms()
        .into_iter()
        .find(|form| form.name == name)
        .unwrap_or_else(|| panic!("the schema publishes a value form named {name}"))
}

/// Compresses a count through the published parameters alone.
///
/// **The helper reads no engine function.** It states the forward map from
/// the offset and the divisor the schema publishes, so a disagreement between
/// the published parameters and the arithmetic the engine runs fails the test.
fn compress_through(form: &ValueForm, exponent: u32) -> i64 {
    let bits = i64::from(form.divisor_bits.expect("a magnitude publishes a divisor"));
    let scaled = i64::from(exponent) * form.unit;
    assert_eq!(
        scaled % bits,
        0,
        "the exponent {exponent} must divide exactly for an exact round trip"
    );
    scaled / bits
}

/// Recovers a count from a published value, through the published parameters
/// alone.
fn invert_through(form: &ValueForm, value: i64) -> i64 {
    let bits = i64::from(form.divisor_bits.expect("a magnitude publishes a divisor"));
    let base = i64::from(form.log_base.expect("a magnitude publishes a base"));
    let offset = form.log_offset.expect("a magnitude publishes an offset");
    let scaled = value * bits;
    assert_eq!(
        scaled % form.unit,
        0,
        "an exact inversion needs a value the unit divides"
    );
    let exponent = u32::try_from(scaled / form.unit).expect("the exponent is small");
    base.pow(exponent) - offset
}

/// Every field names a value form, and the form table says what the name
/// means.
///
/// **The form comes from the same declaration that decides how the field is
/// written.** The field list gives one kind for each field, the writer takes
/// its arithmetic from that kind, and the row publishes that same kind. The
/// bounds of the row therefore have to equal the bounds of the form, and this
/// test is what fails if the two ever come from different places.[^1]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
#[test]
fn every_field_publishes_a_value_form() {
    let schema = observation_schema();
    let forms = schema.value_forms();
    assert_eq!(
        forms.len(),
        ValueKind::ALL.len(),
        "the form table holds one entry for each value kind"
    );
    for row in schema.rows() {
        let named = forms
            .iter()
            .find(|form| form.name == row.form_name())
            .unwrap_or_else(|| {
                panic!(
                    "the field {} names the form {} and the table holds it",
                    row.name(),
                    row.form_name()
                )
            });
        assert_eq!(
            (named.low, named.high),
            (row.low, row.high),
            "the field {} and its form agree on the bounds",
            row.name()
        );
        assert_eq!(
            named.unit, ONE,
            "the form {} states one unit of the fixed-point scale",
            named.name
        );
    }
}

/// A form that divides by a whole the value does not carry is not invertible.
///
/// A share divides by a denominator the doc of each field names, and a signed
/// relation divides by the sum of the two magnitudes it compares. Neither
/// denominator travels with the value, so a reader cannot recover the
/// quantity from one position.
///
/// A statistic groups several forms over one quantity, so its positions do
/// not share one rule and it publishes that it is not uniform. **That is the
/// field class this layout cannot classify at the field level**, and the
/// schema says so rather than naming a form the positions do not all hold.
#[test]
fn a_form_states_whether_a_reader_can_invert_it() {
    let share = form_named("share");
    assert!(!share.invertible, "a share carries no denominator");
    assert_eq!(
        share.denominator,
        Some("per_field"),
        "a share divides by a whole the field names"
    );
    assert!(share.uniform, "every position of a share field is a share");

    let relation = form_named("relation");
    assert!(!relation.invertible, "a relation carries no denominator");
    assert_eq!(
        relation.denominator,
        Some("sum_of_magnitudes"),
        "a relation divides by the sum of the two magnitudes"
    );

    let statistic = form_named("statistic");
    assert!(
        !statistic.uniform,
        "a statistic groups several forms over one quantity"
    );
    assert!(
        !statistic.invertible,
        "a form whose positions differ cannot be inverted at the field level"
    );

    let reserved = form_named("reserved");
    assert!(!reserved.invertible, "a reserved field states no quantity");
    assert_eq!(
        (reserved.low, reserved.high),
        (0, 0),
        "a reserved field reads zero"
    );

    let magnitude = form_named("magnitude");
    assert!(
        magnitude.invertible,
        "a compressed magnitude carries every parameter its inversion needs"
    );
    assert_eq!(
        magnitude.denominator, None,
        "a compressed magnitude divides by no quantity of the world"
    );
    assert!(
        magnitude.uniform,
        "every position of a magnitude field is a magnitude"
    );
}

/// A known count compresses to the published value and inverts back to the
/// count.
///
/// **This is the round trip the published form exists for.** A reader outside
/// the engine holds no compression rule, so it must recover a count from the
/// published value and the published parameters alone. The forward map here
/// reads no engine function, and it therefore fails when the published
/// parameters and the arithmetic the engine runs disagree.
///
/// The fixture walks the whole range the cap admits rather than one middling
/// case. A count of zero is the low extreme, and the largest exponent reaches
/// thirty-four billion.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
#[test]
fn the_published_parameters_recover_a_known_count() {
    let form = form_named("magnitude");
    let base = i64::from(form.log_base.expect("a magnitude publishes a base"));
    let offset = form.log_offset.expect("a magnitude publishes an offset");
    let mut largest = 0i64;
    for exponent in EXACT_EXPONENTS {
        let count = base.pow(exponent) - offset;
        let expected = compress_through(&form, exponent);
        assert_eq!(
            i64::from(sim_math::compressed_magnitude(count).0),
            expected,
            "the engine compresses {count} to the value the parameters give"
        );
        assert_eq!(
            invert_through(&form, expected),
            count,
            "the published parameters recover {count} from its value"
        );
        largest = largest.max(count);
    }
    assert!(
        largest > 1_000_000_000,
        "the fixture must reach a count no world total would give by accident"
    );
    assert_eq!(
        i64::from(sim_math::compressed_magnitude(0).0),
        0,
        "an empty quantity reads empty, so its inversion reads empty"
    );
}

/// The engine publishes a count of the world, and the schema inverts it back.
///
/// The test drives the reader a caller drives, so it proves that the published
/// form reaches a real observation and not only the form table.[^1] The world
/// holds one tile fewer than a power of two, so the round trip is exact and
/// the assertion needs no tolerance.
///
/// # References
///
/// [^1]: Testing rules, section 5. `.agents/rules/testing.md`
#[test]
fn a_published_count_of_the_world_inverts_to_that_count() {
    let world = a_still_world_of(WorldConfig {
        width: 31,
        height: 33,
        ..WIDE
    });
    let tiles = i64::from(world.grid().tile_count());
    assert_eq!(
        tiles, 1023,
        "the fixture must hold one tile fewer than a power of two"
    );

    let values = observation_of(&world, WATCHER);
    let row = observation_schema()
        .row("world_tiles")
        .expect("the schema declares the world tile count");
    assert_eq!(
        row.form_name(),
        "magnitude",
        "the world tile count crosses as a compressed magnitude"
    );

    let published = one(&values, "world_tiles");
    assert_ne!(
        published, tiles,
        "the published value must not already be the count"
    );
    assert_eq!(
        invert_through(&row.form(), published),
        tiles,
        "the schema recovers the tile count of the world from what it published"
    );
}

/// The array publishes the settlers of the faction, and it separates that
/// count from the founding flag.
///
/// **A reader of the flag alone cannot tell a faction that holds no settler
/// from a faction whose settler stands on ground a city may not take.** The
/// two states ask for different actions: the first asks for a queued settler
/// and the second asks for a walk. The fixture therefore builds both, and it
/// asserts that the two positions differ between them.
///
/// The fixture places the settler on ground nobody holds, at a distance from
/// every city, so the flag reads one. A fixture that placed it beside a city
/// would leave the flag at zero in both states, and the assertion would then
/// measure the fixture.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
#[test]
fn the_array_publishes_the_settlers_of_the_faction() {
    let mut world = a_still_world();
    world.step(1).expect("the step runs");
    let empty = observation_of(&world, WATCHER);
    assert_eq!(
        one(&empty, "settlers"),
        0,
        "a faction with no settler publishes no settler"
    );
    assert_eq!(
        one(&empty, "may_found"),
        0,
        "a faction with no settler founds nothing"
    );

    let place = ground_near(&world, Axial::new(40, 40), 8);
    let settler = a_unit_at(&mut world, place, WATCHER);
    assert!(
        world.set_unit_type(settler, SETTLER),
        "the fixture retypes the unit"
    );
    world.step(1).expect("the step runs");
    let held = observation_of(&world, WATCHER);

    assert!(
        one(&held, "settlers") > 0,
        "the faction holds a settler, and the array must publish it"
    );
    assert_eq!(
        world.settler_count(WATCHER),
        Some(1),
        "the reader and the array read one set of settlers"
    );
    assert!(
        one(&held, "may_found") > 0,
        "the fixture placed the settler where the verb accepts it, so the \
         flag and the count both moved and neither alone carried the change"
    );
}

/// The reserve gave up one position for the settler count, and the whole
/// length stayed where it was.
///
/// **A field added anywhere but directly above the reserve moves the start of
/// every field after it, and every stored weight file then places its weights
/// on the wrong positions.** This asserts the shape of the claim rather than
/// the number of positions left, because the number changes with the next
/// claim and the shape does not.
#[test]
fn the_newest_claim_sits_directly_above_the_reserve() {
    let fields = ObsField::ALL;
    let last = fields.last().copied().expect("the layout holds a field");
    assert_eq!(
        last,
        ObsField::LayoutReserve,
        "the reserve is the last field of the layout"
    );
    let above = fields[fields.len() - 2];
    assert_eq!(
        above,
        ObsField::CampaignObjectiveRelief,
        "the campaign objective is the newest claim on the reserve"
    );
    assert!(
        !above.value_kind().is_reserved(),
        "a claim on the reserve reads a value, so it is no longer reserved"
    );
    assert!(
        !ObsField::Settlers.value_kind().is_reserved(),
        "the settler count reads a value, so it is no longer reserved"
    );
}
