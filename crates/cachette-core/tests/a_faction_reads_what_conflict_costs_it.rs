//! A faction reads the strength of its army, and it reads the units that
//! changed hands.
//!
//! The engine simulated conflict and published only the losing side of it, and
//! that side was unwritten. The military strength read zero in every position,
//! so no trained policy could see the army it had. The units a rival's belief
//! took were not declared at all, so a faction that shrank by conversion read
//! a smaller unit count and no cause for it.[^1]
//!
//! **A field that reads a constant zero cannot be told from a real zero.** A
//! reward term over one trains against a constant for as long as the run
//! lasts, and nothing fails. The tests below therefore assert what each value
//! depends on, and not only that it repeats: a strength that ignored the
//! armour column and a strength that ignored the headcount both read as
//! plausible numbers.[^2]
//!
//! **The military strength carries a provisional definition.** No record
//! states how the attack column and the armour column of a unit type combine,
//! and a blocker holds that question. These tests pin the properties the
//! definition must keep and no figure of it.[^3]
//!
//! The tests see only the public crate interface.[^4]
//!
//! # References
//!
//! [^1]: Findings register, FND-694. `docs/FINDINGS.md`
//! [^2]: Testing rules, section 2. `.agents/rules/testing.md`
//! [^3]: Blockers register, BLK-158. `docs/BLOCKERS.md`
//! [^4]: Testing rules, section 6. `.agents/rules/testing.md`

use cachette_core::cohort::NeedRule;
use cachette_core::event_memory::MemoryKind;
use cachette_core::faction_observation::{observation_schema, FieldRow, ObsField};
use cachette_core::unit_type::{UnitTypeId, UnitTypeRow, WORKER_ROW};
use cachette_core::{Axial, Entity, FactionId, Fix32, TileKind, World, WorldConfig};

/// The seed of a world whose one tile admits a unit.
const LAND_SEED: u64 = 1;

/// The type number of the unit that carries an attack and no armour.
const STRIKER: u8 = 0;

/// The type number of the unit that carries an armour and no attack.
///
/// **The pair is the fixture the strength definition needs.** A strength that
/// read the attack column alone would call this unit worthless, and a strength
/// that read the armour column alone would call the striker worthless. One
/// type of each shape therefore reaches both halves of the sum.
const WALL: u8 = 1;

/// Returns a worker row that fights with the given attack and armour.
const fn fighter(attack: Fix32, armour: Fix32) -> UnitTypeRow {
    UnitTypeRow {
        attack,
        armour,
        ..WORKER_ROW
    }
}

/// Builds a world of one tile with the two fighting types defined.
///
/// **Nobody in this fixture goes hungry.** A unit here belongs to no site, so
/// it draws from no store, and the default need rule would end every unit
/// after a few hundred steps. The need rule below is a fixture choice, and no
/// test reads a value of it.
fn arena(unit_capacity: u32) -> World {
    let mut world = World::new(WorldConfig {
        width: 1,
        height: 1,
        seed: LAND_SEED,
        faction_count: 3,
        unit_capacity,
    })
    .expect("a world of one tile is a world");
    assert!(
        world.admits_a_unit(Axial::new(0, 0)),
        "the fixture needs ground that admits a unit"
    );
    world
        .define_unit_type(STRIKER, fighter(Fix32::from_int(3), Fix32::ZERO))
        .expect("the striker row is inside the table");
    world
        .define_unit_type(WALL, fighter(Fix32::ZERO, Fix32::from_int(3)))
        .expect("the wall row is inside the table");
    let never_hungry = NeedRule::new(
        Fix32::ZERO,
        Fix32::ZERO,
        Fix32::ZERO,
        Fix32::ZERO,
        Fix32::from_int(1),
    )
    .expect("no rate of the rule is below zero");
    world.set_need_rule(never_hungry);
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

/// Returns the row of one field of the observation schema.
fn row(name: &str) -> FieldRow {
    observation_schema()
        .row(name)
        .unwrap_or_else(|| panic!("the schema must declare the field `{name}`"))
}

/// Reads the first position of one field of the observation of one faction.
fn scalar(world: &World, faction: u16, name: &str) -> i64 {
    let field = row(name);
    let array = world
        .faction_observation(FactionId(faction))
        .expect("the number names a faction of the world");
    array[field.start as usize]
}

/// Reads one kind of one field that names a cause, for one faction.
///
/// **A field that names a cause holds one position for each kind that has
/// one, and not one for each kind.** A reader that indexed it by the position
/// of the kind would read another field for every kind above the first few,
/// and the number it read would look like a share.
fn blamed(world: &World, faction: u16, name: &str, kind: MemoryKind) -> i64 {
    let field = row(name);
    let slot = kind
        .blame_position()
        .expect("the kind names a cause, so it holds a position in this field");
    let array = world
        .faction_observation(FactionId(faction))
        .expect("the number names a faction of the world");
    array[field.start as usize + slot]
}

/// Reads one kind of one memory field of the observation of one faction.
fn memory(world: &World, faction: u16, name: &str, kind: MemoryKind) -> i64 {
    let field = row(name);
    let array = world
        .faction_observation(FactionId(faction))
        .expect("the number names a faction of the world");
    array[field.start as usize + kind.position()]
}

/// The strength rises with the headcount, so it is not a per-unit constant.
#[test]
fn the_strength_of_a_faction_follows_how_many_units_it_holds() {
    let mut world = arena(64);
    spawn(&mut world, Axial::new(0, 0), 0, STRIKER, 1);
    let one = scalar(&world, 0, "military_strength");
    assert!(one > 0, "one striker is a strength above nothing");
    spawn(&mut world, Axial::new(0, 0), 0, STRIKER, 7);
    let eight = scalar(&world, 0, "military_strength");
    assert!(
        eight > one,
        "eight strikers must read stronger than one, and read {eight} against {one}"
    );
}

/// The strength reads the armour column as well as the attack column.
///
/// **This is the assertion that a sum of one column alone would fail.** A unit
/// of no attack holds ground, and a strength that scored it at nothing would
/// rank a wall below an empty field.
#[test]
fn the_strength_of_a_faction_reads_both_columns_of_the_type_table() {
    let mut world = arena(64);
    spawn(&mut world, Axial::new(0, 0), 0, WALL, 4);
    let walls = scalar(&world, 0, "military_strength");
    assert!(
        walls > 0,
        "a unit that carries armour and no attack is not worth nothing"
    );

    let mut striking = arena(64);
    spawn(&mut striking, Axial::new(0, 0), 0, STRIKER, 4);
    let strikers = scalar(&striking, 0, "military_strength");
    assert_eq!(
        walls, strikers,
        "the two columns of the fixture are equal, so the two armies rank equal"
    );
}

/// The strength for each unit separates a large weak army from a small strong
/// one.
#[test]
fn the_strength_for_each_unit_separates_a_weak_crowd_from_a_strong_few() {
    let mut crowd = arena(64);
    spawn(&mut crowd, Axial::new(0, 0), 0, STRIKER, 8);
    crowd
        .define_unit_type(STRIKER, fighter(Fix32::from_int(1), Fix32::ZERO))
        .expect("the striker row is inside the table");

    let mut few = arena(64);
    spawn(&mut few, Axial::new(0, 0), 0, STRIKER, 2);
    few.define_unit_type(STRIKER, fighter(Fix32::from_int(8), Fix32::ZERO))
        .expect("the striker row is inside the table");

    assert!(
        scalar(&few, 0, "military_strength") > scalar(&crowd, 0, "military_strength"),
        "two units of eight outweigh eight units of one"
    );
    assert!(
        scalar(&few, 0, "strength_for_each_unit") > scalar(&crowd, 0, "strength_for_each_unit"),
        "the per-unit reading must rank the strong few above the weak crowd"
    );
}

/// A faction reads how many rival seats it holds and how many exist.
///
/// **A domination win asks for a count and not a share.** The domination block
/// publishes the progress as a share of the seated factions, and a share
/// cannot say that one seat is left.
#[test]
fn a_faction_reads_the_rival_seats_it_holds_and_the_rival_seats_that_exist() {
    let mut world = arena(64);
    spawn(&mut world, Axial::new(0, 0), 0, STRIKER, 1);
    world.step(1).expect("one thread is a thread count");

    assert!(
        scalar(&world, 0, "rival_seats") > 0,
        "a world of three seats holds rival seats"
    );
    assert_eq!(
        scalar(&world, 0, "rival_seats_held"),
        0,
        "a faction that took no rival seat reads none"
    );
    assert_eq!(
        scalar(&world, 0, "rival_seats"),
        scalar(&world, 1, "rival_seats"),
        "the rival seat count follows the world and not the reader"
    );
}

/// A faction reads the strength of a rival it can see, and reads nothing of a
/// rival it cannot.
///
/// **The statistics of a power quantity need one value for each seat.** The
/// own value is exact and the value of a rival is an estimate from the ground
/// the faction sees this frame, so a rival standing in sight moves the gap and
/// a rival out of sight does not.[^1]
///
/// # References
///
/// [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D2. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
#[test]
fn the_strength_statistics_read_a_rival_the_faction_can_see() {
    let alone = {
        let mut world = arena(64);
        spawn(&mut world, Axial::new(0, 0), 0, STRIKER, 4);
        world.step(1).expect("one thread is a thread count");
        strength_gap(&world)
    };

    let matched = {
        let mut world = arena(64);
        spawn(&mut world, Axial::new(0, 0), 0, STRIKER, 4);
        spawn(&mut world, Axial::new(0, 0), 1, STRIKER, 4);
        world.step(1).expect("one thread is a thread count");
        strength_gap(&world)
    };

    assert!(
        alone > matched,
        "a faction alone in sight must lead by more than one that faces an \
         equal army, and it read {alone} against {matched}"
    );
    assert_eq!(
        matched, 0,
        "two equal armies in sight of each other read no gap"
    );
}

/// Returns the gap statistic of the strength of the first faction.
///
/// The gap is the third of the seven statistics of a power quantity, and the
/// statistics run in the order the layout declares.
fn strength_gap(world: &World) -> i64 {
    let field = row("power_strength");
    let array = world
        .faction_observation(FactionId(0))
        .expect("the number names a faction of the world");
    array[field.start as usize + 2]
}

/// A faction that loses a unit to another faction's belief reads the loss, and
/// the faction that gains it reads the gain.
///
/// **The test drives the verb a control plane calls.** The history takes its
/// arrivals inside the one site that changes the faction of a unit, so the
/// field pass and the verb both reach it. A test that recorded an arrival by
/// hand would prove nothing about whether either route arrives.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 5. `.agents/rules/testing.md`
#[test]
fn a_conversion_reaches_the_history_of_both_factions() {
    let mut world = arena(64);
    let units = spawn(&mut world, Axial::new(0, 0), 0, STRIKER, 4);
    world.step(1).expect("one thread is a thread count");
    assert_eq!(
        memory(&world, 0, "memory_recent", MemoryKind::OwnUnitsConverted),
        0,
        "nothing has changed hands yet"
    );

    world
        .convert_units(&units, FactionId(1))
        .expect("the units are alive and the faction is seated");
    world.step(1).expect("one thread is a thread count");

    assert!(
        memory(&world, 0, "memory_recent", MemoryKind::OwnUnitsConverted) > 0,
        "the faction that lost the units reads the loss"
    );
    assert!(
        memory(&world, 1, "memory_recent", MemoryKind::RivalUnitsConverted) > 0,
        "the faction that gained the units reads the gain"
    );
    assert_eq!(
        memory(&world, 0, "memory_recent", MemoryKind::RivalUnitsConverted),
        0,
        "the faction that lost the units gained nothing"
    );
    assert!(
        blamed(
            &world,
            0,
            "memory_worst_rival",
            MemoryKind::OwnUnitsConverted
        ) > 0,
        "the history names the faction that took the units"
    );
}

/// A conversion is not a meeting and not a shortage, so it moves no other
/// counter.
#[test]
fn a_conversion_moves_no_counter_that_names_another_cause() {
    let mut world = arena(64);
    let units = spawn(&mut world, Axial::new(0, 0), 0, STRIKER, 4);
    world
        .convert_units(&units, FactionId(1))
        .expect("the units are alive and the faction is seated");
    world.step(1).expect("one thread is a thread count");

    for kind in [
        MemoryKind::OwnUnitsFelled,
        MemoryKind::OwnUnitsStarved,
        MemoryKind::OwnUnitsBurned,
    ] {
        assert_eq!(
            memory(&world, 0, "memory_recent", kind),
            0,
            "a conversion must not read as a {} loss",
            kind.name()
        );
    }
}

/// The resident count and the live unit count are one quantity while every
/// unit is homed, and they part when a unit is homed nowhere.
///
/// **A reader must not weigh the two as two terms of one objective.** The
/// engine holds no person apart from a unit, so a settlement counts the units
/// whose home names it. This test states the identity, so that a change to it
/// fails here rather than in a training run.[^1] [^2]
///
/// # References
///
/// [^1]: Findings register, FND-695. `docs/FINDINGS.md`
/// [^2]: Blockers register, BLK-159. `docs/BLOCKERS.md`
#[test]
fn the_resident_count_counts_units_and_not_a_separate_people() {
    let mut world = arena(64);
    spawn(&mut world, Axial::new(0, 0), 0, STRIKER, 4);
    world.step(1).expect("one thread is a thread count");

    assert!(
        scalar(&world, 0, "live_units") > 0,
        "the fixture must hold an army"
    );
    assert_eq!(
        scalar(&world, 0, "population"),
        0,
        "a unit that no settlement is the home of is nobody's resident"
    );
    assert_eq!(
        scalar(&world, 0, "population"),
        scalar(&world, 0, "population_for_each_settlement"),
        "a faction of no settlement divides by one, so the two agree"
    );
}

/// Every field the layout holds back reads zero in every position.
///
/// **This is the other half of the assertion the writer makes.** The writer
/// asserts that every field it leaves alone declares the reserved form. This
/// asserts that no field of the reserved form is written, so a field cannot
/// carry a bound of zero and a value that is not zero.
#[test]
fn a_field_the_layout_holds_back_reads_zero_in_a_live_world() {
    let mut world = arena(64);
    spawn(&mut world, Axial::new(0, 0), 0, STRIKER, 4);
    spawn(&mut world, Axial::new(0, 0), 1, WALL, 4);
    world.step(1).expect("one thread is a thread count");
    let array = world
        .faction_observation(FactionId(0))
        .expect("the number names a faction of the world");

    let schema = observation_schema();
    let mut held_back = 0usize;
    for field in schema.rows() {
        if !field.field.value_kind().is_reserved() {
            continue;
        }
        held_back += 1;
        let first = field.start as usize;
        let span = &array[first..first + field.positions as usize];
        assert!(
            span.iter().all(|value| *value == 0),
            "the field `{}` declares bounds of zero and reads {span:?}",
            field.field.name()
        );
    }
    assert!(
        held_back > 0,
        "the layout declares no reserved field, so this test asserts nothing"
    );
}

/// The extent of the world the fire fixture builds.
const FOREST_EXTENT: u32 = 96;

/// The seed of the world the fire fixture builds.
const FOREST_SEED: u64 = 20_260_907;

/// The largest number of ticks the fire fixture waits for a unit to burn.
///
/// **The bound is what makes the fixture assertion able to fail.** A fixture
/// that burned nobody would run to this bound and fail there rather than
/// hanging.
const PATIENCE: u32 = 400;

/// Returns the address whose neighbourhood holds the most forest.
///
/// **The fixture must supply the extreme and not the typical case.** A fire
/// lit on one lonely forest tile burns out in a few ticks and reaches nobody,
/// so a crowd standing on it would survive and the test would measure the
/// fixture.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
fn thickest_wood(world: &World) -> Axial {
    let reach = 3i32;
    let mut best = Axial::new(0, 0);
    let mut most = -1i32;
    for r in 0..FOREST_EXTENT as i32 {
        for q in 0..FOREST_EXTENT as i32 {
            let here = Axial::new(q, r);
            if world.tile_kind(here) != Some(TileKind::Forest) {
                continue;
            }
            let mut count = 0i32;
            for dr in -reach..=reach {
                for dq in -reach..=reach {
                    if world.tile_kind(Axial::new(q + dq, r + dr)) == Some(TileKind::Forest) {
                        count += 1;
                    }
                }
            }
            if count > most {
                most = count;
                best = here;
            }
        }
    }
    assert!(most > 8, "the world holds no wood worth burning");
    best
}

/// Builds a world of wood, stands a crowd in the densest part of it, and sets
/// that part alight.
///
/// The world holds no city, so nothing but the fire ends a unit.
fn burning_wood() -> World {
    let mut world = World::new(WorldConfig {
        width: FOREST_EXTENT,
        height: FOREST_EXTENT,
        seed: FOREST_SEED,
        faction_count: 2,
        unit_capacity: 4096,
    })
    .expect("the extent must describe a world");
    let middle = thickest_wood(&world);
    for r in -2i32..=2 {
        for q in -2i32..=2 {
            let here = Axial::new(middle.q + q, middle.r + r);
            if world.tile_kind(here).is_none() {
                continue;
            }
            for _ in 0..3 {
                if world.spawn_soldier(here, FactionId(0)).is_err() {
                    break;
                }
            }
        }
    }
    let tile = world
        .grid()
        .index_of(middle)
        .expect("the address lies inside the world");
    assert!(world.ignite(tile), "the densest wood must catch");
    world
}

/// The two positions of one quantity read the same number.
///
/// The tiles a faction lost, the settlements it lost and the units a hazard
/// took are each declared twice: once in the block that names the quantity and
/// once in the memory block. One function answers both, and this asserts that
/// no revision moved one and left the other.[^1]
///
/// **The fixture reaches the case.** A fire ends units of the crowd, so the
/// hazard counter is above zero when the comparison runs, and an equality
/// between two zeroes would prove nothing.[^2]
///
/// # References
///
/// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
/// [^2]: Testing rules, section 2a. `.agents/rules/testing.md`
#[test]
fn a_quantity_the_layout_publishes_twice_reads_the_same_in_both_places() {
    let mut world = burning_wood();
    let mut burnt = 0i64;
    for _ in 0..PATIENCE {
        world.step(1).expect("one thread is a thread count");
        burnt = scalar(&world, 0, "units_lost_to_hazard");
        if burnt > 0 {
            break;
        }
    }
    assert!(
        burnt > 0,
        "the fixture must burn somebody, so that the comparison below is live"
    );

    for (field, kind) in [
        (ObsField::TilesLost, MemoryKind::OwnGroundLost),
        (ObsField::SettlementsLost, MemoryKind::OwnSitesLost),
        (ObsField::UnitsLostToHazard, MemoryKind::OwnUnitsBurned),
    ] {
        assert_eq!(
            scalar(&world, 0, field.name()),
            memory(&world, 0, "memory_recent", kind),
            "the field `{}` and the memory of `{}` are one quantity",
            field.name(),
            kind.name()
        );
    }
}
