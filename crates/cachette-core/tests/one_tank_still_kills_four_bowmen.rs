//! One tank still kills four bowmen, and ten thousand bowmen also lose.
//!
//! This file holds the acceptance test that the project owner set for combat,
//! and the tests that prove the draw of the resolution is keyed on every field
//! it claims.
//!
//! **The threshold applies for each attacker type before anything is
//! aggregated.** An attacker type whose attack does not exceed the defender
//! type's armour contributes exactly zero, so a sum of zeroes stays zero at
//! any count.[^1] That is why ten thousand bowmen lose to one tank, and it
//! holds without a rate, a cap or a balance figure.
//!
//! **Contact is adjacency.** A unit reaches every unit of another faction on
//! its own tile and on the six tiles beside it. Admission refuses a step onto
//! a tile at its capacity, and it reads the capacity rather than the faction,
//! so a rule that fired only on co-occupation could never fire against a full
//! enemy tile.[^5]
//!
//! **The fixture produces the edge and not the typical case.** One world holds
//! a pair that cannot reach, one holds a pair that both reach, one holds a
//! tile filled past its capacity, and one holds two tiles that are two steps
//! apart and therefore reach nothing.[^2]
//!
//! **The worlds here are built so that no unit can move.** A world of one tile
//! has no neighbour inside the extent, and the wider worlds hold each tile
//! past its capacity, so admission refuses every step.[^3] A test whose units
//! walked away would measure the movement rule and not the contest.
//!
//! The tests see only the public crate API.[^4]
//!
//! # References
//!
//! [^1]: ADR-0122, an attacker whose attack does not exceed the defender's armour contributes exactly zero, decision D1. `docs/adrs/draft/adr-0122-an-attacker-below-the-armour-contributes-exactly-zero.md`
//! [^2]: Testing rules, section 2a. `.claude/rules/testing.md`
//! [^3]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
//! [^4]: Testing policy. `docs/TESTING.md`
//! [^5]: Findings register, FND-402. `docs/FINDINGS.md`

use cachette_core::contest::casualties;
use cachette_core::unit_type::{UnitTypeId, UnitTypeRow, WORKER_ROW};
use cachette_core::{Accum, Axial, Entity, FactionId, Fix32, World, WorldConfig};

/// Returns a worker row that fights with the given attack and armour.
///
/// The other columns are the worker's, so the row differs from the default
/// in the two columns the contest reads and in nothing else.
const fn fighter(attack: Fix32, armour: Fix32) -> UnitTypeRow {
    UnitTypeRow {
        attack,
        armour,
        ..WORKER_ROW
    }
}

/// The thread counts that every test in this file runs at.
///
/// A determinism claim proved at one thread count proves nothing.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 1. `.claude/rules/testing.md`
const THREAD_COUNTS: [usize; 3] = [1, 2, 12];

/// The seed of a world whose every tile admits a unit.
///
/// A world of water holds no unit at all, and the fixture would then measure
/// nothing. Each builder below asserts that its tiles admit a unit.
const LAND_SEED: u64 = 1;

/// The type number of the light unit. It reaches ordinary ground and it never
/// reaches the heavy unit.
const BOWMAN: u8 = 0;

/// The type number of the heavy unit. Its armour is above the attack of the
/// light unit, so the light unit contributes exactly zero against it.
const TANK: u8 = 1;

/// Puts the two factions of a fixture at war.
///
/// **The contest resolves a meeting only across a pair at war.** Every
/// fixture in this file is about the resolution and not about the gate, so
/// each one declares the war first. The edge is read from the world and not
/// restated here.[^1]
///
/// # References
///
/// [^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D4. `docs/adrs/draft/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
fn declare_war(world: &mut World) {
    let war = world.relation_rules().war_edge - 1;
    assert!(world.set_relation(FactionId(0), FactionId(1), war));
}

/// Builds a world of one tile, with the tank and bowman table filled.
///
/// The bowman delivers one whole casualty and carries no armour. The tank
/// delivers four and carries an armour above the attack of the bowman. The
/// values are content, and the caller states them here rather than the engine
/// holding them.
fn one_tile_world(unit_capacity: u32) -> World {
    let mut world = World::new(WorldConfig {
        width: 1,
        height: 1,
        seed: LAND_SEED,
        faction_count: 2,
        unit_capacity,
    })
    .expect("a world of one tile is a world");
    declare_war(&mut world);
    assert!(
        world.admits_a_unit(Axial::new(0, 0)),
        "the fixture needs ground that admits a unit"
    );
    world
        .define_unit_type(BOWMAN, fighter(Fix32::from_int(1), Fix32::ZERO))
        .expect("the bowman row is inside the table");
    world
        .define_unit_type(TANK, fighter(Fix32::from_int(4), Fix32::from_int(2)))
        .expect("the tank row is inside the table");
    world
}

/// Spawns units of one faction and one type on one address.
fn spawn(
    world: &mut World,
    address: Axial,
    faction: u16,
    unit_type: u8,
    count: u32,
) -> Vec<Entity> {
    let kind = UnitTypeId::from_u8(unit_type).expect("the number names a row of the table");
    (0..count)
        .map(|_| {
            let unit = world
                .spawn_soldier(address, FactionId(faction))
                .expect("the ground admits a unit and the faction is inside the world");
            assert!(world.set_unit_type(unit, kind), "the unit is alive");
            unit
        })
        .collect()
}

#[test]
fn one_tank_kills_four_bowmen_and_takes_nothing() {
    for threads in THREAD_COUNTS {
        let mut world = one_tile_world(64);
        let bowmen = spawn(&mut world, Axial::new(0, 0), 0, BOWMAN, 4);
        let tank = spawn(&mut world, Axial::new(0, 0), 1, TANK, 1);

        world.step(threads).expect("the step runs");

        assert_eq!(
            world.population_of(FactionId(0)),
            0,
            "the tank must end all four bowmen at {threads} threads"
        );
        assert_eq!(
            world.population_of(FactionId(1)),
            1,
            "four bowmen must reach the tank for exactly nothing at {threads} threads"
        );
        assert!(
            bowmen.iter().all(|unit| world.unit_type(*unit).is_none()),
            "every bowman is dead, so no identity of one resolves"
        );
        assert_eq!(
            world.unit_type(tank[0]),
            UnitTypeId::from_u8(TANK),
            "the tank still carries its type"
        );
        assert!(world.check_invariants());
    }
}

#[test]
fn ten_thousand_bowmen_lose_to_one_tank() {
    // **This is the case a typical fixture never supplies.** Four bowmen and
    // ten thousand bowmen must give the same answer against the tank, and only
    // a threshold applied before the aggregation does that. A spawn may
    // over-fill a tile, so the fixture reaches a crowd no admission would let
    // walk onto one tile.[^1]
    //
    // [^1]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
    const CROWD: u32 = 10_000;
    for threads in THREAD_COUNTS {
        let mut world = one_tile_world(CROWD + 16);
        spawn(&mut world, Axial::new(0, 0), 0, BOWMAN, CROWD);
        let tank = spawn(&mut world, Axial::new(0, 0), 1, TANK, 1);

        world.step(threads).expect("the step runs");

        assert_eq!(
            world.population_of(FactionId(1)),
            1,
            "a sum of zeroes must stay zero at {CROWD} attackers, at {threads} threads"
        );
        assert!(world.unit_type(tank[0]).is_some(), "the tank is alive");
        assert_eq!(
            world.population_of(FactionId(0)),
            CROWD - 4,
            "the tank ends exactly what its attack pays for"
        );
        assert!(world.check_invariants());
    }
}

#[test]
fn a_pair_that_both_reach_takes_losses_on_both_sides() {
    // The tank test covers a pair that cannot reach. This covers the other
    // edge: two types that each exceed the armour of the other, so both sides
    // lose units in one frame.
    for threads in THREAD_COUNTS {
        let mut world = one_tile_world(64);
        world
            .define_unit_type(BOWMAN, fighter(Fix32::from_int(2), Fix32::from_int(1)))
            .expect("the row is inside the table");
        world
            .define_unit_type(TANK, fighter(Fix32::from_int(3), Fix32::from_int(1)))
            .expect("the row is inside the table");
        spawn(&mut world, Axial::new(0, 0), 0, BOWMAN, 8);
        spawn(&mut world, Axial::new(0, 0), 1, TANK, 2);

        world.step(threads).expect("the step runs");

        // Two tanks deliver six casualties to eight bowmen. Eight bowmen
        // deliver sixteen to two tanks, and a tile cannot lose more units
        // than it holds.
        assert_eq!(world.population_of(FactionId(0)), 2);
        assert_eq!(world.population_of(FactionId(1)), 0);
        assert_eq!(
            world.fell_log().len(),
            8,
            "the log names every unit that fell at {threads} threads"
        );
        assert!(world.check_invariants());
    }
}

#[test]
fn a_tile_of_one_faction_resolves_nothing() {
    // A meeting needs two factions. A crowd of one faction must lose nobody,
    // however large it is and whatever the table says.
    let mut world = one_tile_world(64);
    spawn(&mut world, Axial::new(0, 0), 0, TANK, 16);
    world.step(4).expect("the step runs");
    assert_eq!(world.population_of(FactionId(0)), 16);
    assert!(world.fell_log().is_empty());
    assert!(world.check_invariants());
}

#[test]
fn the_frame_is_in_the_draw_key() {
    // **A draw keyed on the tile and not on the frame ends the same units for
    // ever, and both determinism tests pass while it does.**[^1] The fixture
    // makes the harm a fraction of one whole unit, so the remainder draw is
    // the only thing that decides whether anybody falls. If the frame left the
    // key, that draw would answer the same on every frame, and the count of
    // casualties would be the same on every frame.
    //
    // [^1]: Testing rules, section 2. `.claude/rules/testing.md`
    const FRAMES: usize = 24;
    let mut world = one_tile_world(128);
    // One half of a whole unit, in the project fixed-point scale. The whole
    // part of the harm is zero, so every casualty comes from the remainder.
    world
        .define_unit_type(BOWMAN, fighter(Fix32(Fix32::ONE.0 / 2), Fix32::ZERO))
        .expect("the row is inside the table");
    world
        .define_unit_type(TANK, fighter(Fix32::ZERO, Fix32::ZERO))
        .expect("the row is inside the table");
    spawn(&mut world, Axial::new(0, 0), 0, BOWMAN, 1);
    spawn(&mut world, Axial::new(0, 0), 1, TANK, 64);

    let mut counts = Vec::with_capacity(FRAMES);
    for _ in 0..FRAMES {
        world.step(4).expect("the step runs");
        counts.push(world.fell_log().len());
    }
    assert!(
        counts.contains(&0),
        "a frame must exist in which the remainder draws no casualty: {counts:?}"
    );
    assert!(
        counts.contains(&1),
        "a frame must exist in which the remainder draws one casualty: {counts:?}"
    );
    assert!(
        counts.iter().all(|count| *count <= 1),
        "half a whole unit of harm never ends two units: {counts:?}"
    );
    assert!(world.check_invariants());
}

#[test]
fn the_tile_is_in_the_draw_key() {
    // **Two tiles that hold the same groups must not answer alike.** If the
    // tile left the key, the two tiles would draw one value and their whole
    // sequences of casualty counts would be equal. The fixture runs both tiles
    // in one world, so the frame and the seed are the same for both, and only
    // the tile differs.
    //
    // **The two tiles are not neighbours, and the tile between them holds the
    // same units for both.** Contact is adjacency, so two neighbouring tiles
    // would fight each other and neither would hold the group the test means
    // to compare. A world of three tiles in a row puts one tile between them,
    // and each of the two sees exactly that one neighbour.
    //
    // Every tile holds far more units than the ground admits, so admission
    // refuses every step and no unit ever leaves the tile it was spawned
    // on.[^1]
    //
    // [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
    const FRAMES: usize = 24;
    const GARRISON: u32 = 40;
    let mut world = World::new(WorldConfig {
        width: 3,
        height: 1,
        seed: LAND_SEED,
        faction_count: 2,
        unit_capacity: 512,
    })
    .expect("a world of three tiles is a world");
    declare_war(&mut world);
    let left = Axial::new(0, 0);
    let middle = Axial::new(1, 0);
    let right = Axial::new(2, 0);
    for address in [left, middle, right] {
        assert!(
            world.admits_a_unit(address),
            "the fixture needs ground that admits a unit"
        );
    }
    // The bowman carries half a whole casualty of attack, so the harm it
    // delivers has no whole part at all and only the remainder draw decides
    // whether anybody falls. The tank carries no attack, so it never harms
    // the bowman beside it and the fixture keeps its shape.
    world
        .define_unit_type(BOWMAN, fighter(Fix32(Fix32::ONE.0 / 2), Fix32::ZERO))
        .expect("the row is inside the table");
    world
        .define_unit_type(TANK, fighter(Fix32::ZERO, Fix32::ZERO))
        .expect("the row is inside the table");
    for address in [left, right] {
        spawn(&mut world, address, 0, BOWMAN, 1);
        spawn(&mut world, address, 1, TANK, GARRISON);
    }
    // The middle tile holds one faction, so it adds the same term to both
    // sides and it never fights either of them.
    spawn(&mut world, middle, 1, TANK, GARRISON);

    let mut on_left = Vec::with_capacity(FRAMES);
    let mut on_right = Vec::with_capacity(FRAMES);
    for _ in 0..FRAMES {
        world.step(4).expect("the step runs");
        let fell_on = |address: Axial| {
            let tile = world
                .grid()
                .index_of(address)
                .expect("the address is inside the world");
            world
                .fell_log()
                .iter()
                .filter(|event| event.tile == tile)
                .count()
        };
        on_left.push(fell_on(left));
        on_right.push(fell_on(right));
    }
    assert!(
        on_left.iter().sum::<usize>() > 0,
        "the fixture resolved nothing on the left tile"
    );
    assert_ne!(
        on_left, on_right,
        "two tiles that share a frame and a seed must not draw alike"
    );
    assert!(world.check_invariants());
}

#[test]
fn a_unit_reaches_the_tile_beside_it() {
    // **Contact is adjacency.** Admission refuses a step onto a tile at its
    // capacity, and it reads the capacity rather than the faction, so an army
    // that filled a tile could never be entered. A rule that fired only on
    // co-occupation would never fire against the case a fight is about.[^1]
    //
    // The fixture puts each side on its own tile, beside the other, and
    // neither tile is full. Nothing enters anything, and the fight happens.
    //
    // [^1]: Findings register, FND-402. `docs/FINDINGS.md`
    let mut world = World::new(WorldConfig {
        width: 3,
        height: 1,
        seed: LAND_SEED,
        faction_count: 2,
        unit_capacity: 64,
    })
    .expect("a world of three tiles is a world");
    declare_war(&mut world);
    world
        .define_unit_type(BOWMAN, fighter(Fix32::from_int(1), Fix32::ZERO))
        .expect("the row is inside the table");
    world
        .define_unit_type(TANK, fighter(Fix32::from_int(4), Fix32::from_int(2)))
        .expect("the row is inside the table");
    spawn(&mut world, Axial::new(0, 0), 0, BOWMAN, 4);
    spawn(&mut world, Axial::new(1, 0), 1, TANK, 1);

    world.step(4).expect("the step runs");

    assert_eq!(
        world.population_of(FactionId(0)),
        0,
        "the tank reaches the tile beside it"
    );
    assert_eq!(
        world.population_of(FactionId(1)),
        1,
        "four bowmen on the tile beside the tank still reach it for nothing"
    );
    assert!(world.check_invariants());
}

#[test]
fn a_unit_reaches_no_tile_two_steps_away() {
    // Adjacency is the six neighbours and nothing further. A world of three
    // tiles in a row puts the two ends two steps apart, and neither reaches
    // the other.
    let mut world = World::new(WorldConfig {
        width: 3,
        height: 1,
        seed: LAND_SEED,
        faction_count: 2,
        unit_capacity: 64,
    })
    .expect("a world of three tiles is a world");
    declare_war(&mut world);
    world
        .define_unit_type(BOWMAN, fighter(Fix32::from_int(4), Fix32::ZERO))
        .expect("the row is inside the table");
    spawn(&mut world, Axial::new(0, 0), 0, BOWMAN, 4);
    spawn(&mut world, Axial::new(2, 0), 1, BOWMAN, 4);

    world.step(4).expect("the step runs");

    assert_eq!(world.population_of(FactionId(0)), 4);
    assert_eq!(world.population_of(FactionId(1)), 4);
    assert!(world.fell_log().is_empty());
    assert!(world.check_invariants());
}

#[test]
fn the_remainder_never_turns_no_harm_into_a_casualty() {
    // The remainder decides one casualty at most, and no draw below the scale
    // turns zero harm into one. The draw is a parameter here, so the test
    // covers the whole range of it rather than the one value a fixture reaches.
    for draw in [0u64, 1, 32_767, 65_535] {
        assert_eq!(casualties(Accum(0), 8, draw), 0);
        assert_eq!(casualties(Accum(-1), 8, draw), 0);
    }
    // A harm of one whole unit and no fraction ends exactly one unit, whatever
    // the remainder draw says.
    for draw in [0u64, 65_535] {
        assert_eq!(casualties(Accum(i64::from(Fix32::ONE.0)), 8, draw), 1);
    }
    // A tile never loses more units than it holds.
    assert_eq!(casualties(Accum(i64::from(Fix32::ONE.0) * 40), 8, 0), 8);
}
