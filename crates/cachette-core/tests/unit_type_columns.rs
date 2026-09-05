//! A pass reads the capability column of the row a unit indexes, and zero
//! means cannot.
//!
//! The gather pass scales the tile rate by the gather rate of the type and
//! caps the load at the carry capacity. The build pass scales the builder
//! rate by the build rate of the type. A column at zero makes the unit take
//! nothing or add nothing, and the unit keeps its order.[^1]
//!
//! Every test here drives the step. None calls a pass directly.[^2]
//!
//! **The fixture is built for the extremes.** A row at zero and a row at one
//! sit on one island tile, so the two units differ in one column and in
//! nothing else. A fixture that used the worker row alone would supply no
//! extreme, and the assertion would measure the fixture.[^3]
//!
//! # References
//!
//! [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decisions D1 and D2. `docs/adrs/draft/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
//! [^2]: Testing rules, section 5. `.agents/rules/testing.md`
//! [^3]: Testing rules, section 2a. `.agents/rules/testing.md`

use cachette_core::choose::{self, ChoiceSchedule};
use cachette_core::resource::{Amount, ResourceKind};
use cachette_core::unit_type::{UnitTypeId, UnitTypeRow, WORKER_ROW};
use cachette_core::upgrade::UpgradeKind;
use cachette_core::{Axial, Entity, FactionId, Fix32, World, WorldConfig};

/// The extent of every fixture world.
const WIDTH: u32 = 192;
/// The number of rows of that extent.
const HEIGHT: u32 = 192;

/// The seed of the world that holds an island deposit.
///
/// An island is a tile whose every neighbour refuses a unit. A unit on it
/// never moves, so a test can put named units on one tile and know they are
/// still there when the pass runs.
const SEED: u64 = 102;

/// The number of ticks the longest test here runs.
const TICKS: u64 = 8;

/// The least stock an island deposit must carry for the fixture to use it.
///
/// The island of the fixture seed carries a small deposit, so the bound is
/// what the capacity tests need and not what a long run would take. The
/// every-tick assertion stops when the tile runs out.
const LEAST_STOCK: u32 = 6;

/// The thread counts that every stepping test runs at.
const THREAD_COUNTS: [usize; 3] = [1, 2, 12];

/// The row number of the type that cannot do the thing under test.
const CANNOT: u8 = 5;

/// The row number of the type that can do the thing under test, at scale
/// one.
const CAN: u8 = 6;

/// The row number of the type that can do the thing under test, at the
/// largest scale the fixture uses.
const MORE: u8 = 7;

/// Builds the world under test, with the choice held off the ticks it runs.
fn world() -> World {
    let mut world = World::new(WorldConfig {
        width: WIDTH,
        height: HEIGHT,
        seed: SEED,
        faction_count: 4,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    hold_the_choice(&mut world);
    world
}

/// Puts the choice far enough apart that it does not replace an order.
///
/// The choice pass writes the gather order of a unit whose level 1 cell
/// chooses on that frame, and that write replaces the order a caller gave.
/// The assertion states that no cell chooses inside the run.
fn hold_the_choice(world: &mut World) {
    let schedule =
        ChoiceSchedule::new(choose::PERIOD_LOG2_CEILING).expect("the exponent is inside the range");
    world
        .set_choice_schedule(schedule.period_log2())
        .expect("the exponent is inside the range");
    for cell in 0..world.pyramid().len() as u32 {
        for frame in 1..=TICKS {
            assert!(
                !schedule.chooses_now(cell, frame),
                "cell {cell} chooses on frame {frame}, so the choice replaces the order"
            );
        }
    }
}

/// Returns every address of the extent, in row-major order.
fn addresses() -> Vec<Axial> {
    let mut all = Vec::with_capacity((WIDTH * HEIGHT) as usize);
    for r in 0..HEIGHT {
        for q in 0..WIDTH {
            all.push(Axial::new(q as i32, r as i32));
        }
    }
    all
}

/// Returns an island tile that carries at least the given stock of a kind.
fn island_with_stock(world: &World, kind: ResourceKind, least: u32) -> Axial {
    addresses()
        .into_iter()
        .find(|address| {
            world.admits_a_unit(*address)
                && world.original_stock(*address, kind) >= Some(Amount(least))
                && world
                    .grid()
                    .neighbours(*address)
                    .iter()
                    .all(|side| side.is_none_or(|next| !world.admits_a_unit(next)))
        })
        .expect("the world must hold an island that carries the kind")
}

/// Returns an island tile.
fn island(world: &World) -> Axial {
    addresses()
        .into_iter()
        .find(|address| {
            world.admits_a_unit(*address)
                && world
                    .grid()
                    .neighbours(*address)
                    .iter()
                    .all(|side| side.is_none_or(|next| !world.admits_a_unit(next)))
        })
        .expect("the world must hold an island")
}

/// Writes one row and returns its type.
fn define(world: &mut World, number: u8, row: UnitTypeRow) -> UnitTypeId {
    world
        .define_unit_type(number, row)
        .expect("the row is inside the table");
    UnitTypeId::from_u8(number).expect("the number names a row")
}

/// Spawns one unit of a type on an address.
fn spawn(world: &mut World, address: Axial, unit_type: UnitTypeId) -> Entity {
    let unit = world
        .spawn_soldier(address, FactionId(0))
        .expect("the ground admits a unit");
    assert!(world.set_unit_type(unit, unit_type), "the unit is live");
    unit
}

/// Returns what a unit carries of one kind.
fn carried(world: &World, unit: Entity, kind: ResourceKind) -> u32 {
    world
        .soldier_carry(unit)
        .expect("the unit is live")
        .of(kind)
        .0
}

// ---------------------------------------------------------------------------
// The gather rate
// ---------------------------------------------------------------------------

#[test]
fn a_gather_rate_of_zero_takes_nothing_and_a_rate_of_one_takes_something() {
    // The two units stand on one tile with one order. They differ in the
    // gather rate and in nothing else, so a difference in the load is the
    // column reaching the pass. The rate-zero unit keeps its order, because
    // the pass reads the column and refuses nothing.
    for threads in THREAD_COUNTS {
        let mut field = world();
        let kind = ResourceKind::Wood;
        let address = island_with_stock(&field, kind, LEAST_STOCK);
        let cannot = define(
            &mut field,
            CANNOT,
            UnitTypeRow {
                gather_rate: Fix32::ZERO,
                ..WORKER_ROW
            },
        );
        let can = define(
            &mut field,
            CAN,
            UnitTypeRow {
                gather_rate: Fix32::ONE,
                ..WORKER_ROW
            },
        );
        let idle = spawn(&mut field, address, cannot);
        let taker = spawn(&mut field, address, can);
        assert!(field.order_gather(idle, kind));
        assert!(field.order_gather(taker, kind));

        for _ in 0..TICKS {
            field.step(threads).expect("the step must run");
            assert_eq!(
                field.gather_order(idle),
                Some(Some(kind)),
                "the unit that cannot gather keeps its order"
            );
        }

        assert_eq!(
            carried(&field, idle, kind),
            0,
            "a gather rate of zero must take nothing at {threads} threads"
        );
        assert!(
            carried(&field, taker, kind) > 0,
            "a gather rate of one must take something at {threads} threads"
        );
        assert!(
            field
                .gather_log()
                .iter()
                .all(|event| event.unit != idle.to_bits()),
            "a unit that took nothing must leave no take event"
        );
        assert!(field.check_invariants());
    }
}

#[test]
fn a_gather_rate_of_one_half_takes_half_the_tile_rate() {
    // The scale is a fixed-point multiply, and one half of the tile rate is
    // what the column means. The rate-one unit reads the tile rate itself,
    // so the test compares the two rather than naming the rate.
    let mut field = world();
    let kind = ResourceKind::Wood;
    let address = island_with_stock(&field, kind, LEAST_STOCK);
    let half = define(
        &mut field,
        CANNOT,
        UnitTypeRow {
            gather_rate: Fix32(Fix32::ONE.0 / 2),
            ..WORKER_ROW
        },
    );
    let whole = define(
        &mut field,
        CAN,
        UnitTypeRow {
            gather_rate: Fix32::ONE,
            ..WORKER_ROW
        },
    );
    let halved = spawn(&mut field, address, half);
    let full = spawn(&mut field, address, whole);
    assert!(field.order_gather(halved, kind));
    assert!(field.order_gather(full, kind));
    field.step(1).expect("the step must run");
    let taken_by_full = carried(&field, full, kind);
    assert!(
        taken_by_full >= 2,
        "the fixture must supply a rate above one"
    );
    assert_eq!(carried(&field, halved, kind), taken_by_full / 2);
}

// ---------------------------------------------------------------------------
// The carry capacity
// ---------------------------------------------------------------------------

#[test]
fn a_carry_capacity_of_zero_takes_nothing() {
    // The unit can gather, and it has nowhere to put what it takes. The cap
    // is read by the same pass as the rate, so a cap of zero and a rate of
    // zero look the same from outside and differ in which column is zero.
    let mut field = world();
    let kind = ResourceKind::Wood;
    let address = island_with_stock(&field, kind, LEAST_STOCK);
    let cannot = define(
        &mut field,
        CANNOT,
        UnitTypeRow {
            carry_capacity: 0,
            ..WORKER_ROW
        },
    );
    let unit = spawn(&mut field, address, cannot);
    assert!(field.order_gather(unit, kind));
    for _ in 0..TICKS {
        field.step(1).expect("the step must run");
    }
    assert_eq!(carried(&field, unit, kind), 0);
    assert!(field.gather_log().is_empty());
}

#[test]
fn a_carry_capacity_at_the_largest_value_caps_nothing_the_fixture_reaches() {
    // The largest value the column holds is the placeholder the worker
    // carries. A unit at it takes what an uncapped unit took: the tile rate
    // on every tick, until the tile runs out.
    let mut capped = world();
    let kind = ResourceKind::Wood;
    let address = island_with_stock(&capped, kind, LEAST_STOCK);
    let most = define(
        &mut capped,
        MORE,
        UnitTypeRow {
            carry_capacity: u32::MAX,
            ..WORKER_ROW
        },
    );
    let unit = spawn(&mut capped, address, most);
    assert!(capped.order_gather(unit, kind));
    let original = capped
        .original_stock(address, kind)
        .expect("the address is inside the world");
    let mut previous = 0u32;
    for _ in 0..TICKS {
        capped.step(1).expect("the step must run");
        let now = carried(&capped, unit, kind);
        let left = capped
            .tile_stock(address, kind)
            .expect("the address is inside the world");
        assert!(
            now > previous || left == Amount::ZERO,
            "the unit must take something on every tick while the tile holds any"
        );
        previous = now;
    }
    assert_eq!(
        previous, original.0,
        "the uncapped unit must take the whole deposit inside the run"
    );
}

#[test]
fn a_carry_capacity_between_the_extremes_is_the_most_the_unit_ever_holds() {
    // The cap is on the total load. The unit fills to the cap and then takes
    // nothing more, whatever the tile still holds.
    const CAP: u32 = 5;
    let mut field = world();
    let kind = ResourceKind::Wood;
    let address = island_with_stock(&field, kind, LEAST_STOCK);
    let small = define(
        &mut field,
        CAN,
        UnitTypeRow {
            carry_capacity: CAP,
            ..WORKER_ROW
        },
    );
    let unit = spawn(&mut field, address, small);
    assert!(field.order_gather(unit, kind));
    for _ in 0..TICKS {
        field.step(1).expect("the step must run");
        assert!(
            carried(&field, unit, kind) <= CAP,
            "the load rose above the carry capacity"
        );
    }
    assert_eq!(
        carried(&field, unit, kind),
        CAP,
        "the unit must fill to its capacity when the tile holds enough"
    );
    let original = field
        .original_stock(address, kind)
        .expect("the address is inside the world");
    let left = field
        .tile_stock(address, kind)
        .expect("the address is inside the world");
    assert_eq!(
        original.0 - left.0,
        CAP,
        "what the tile lost must equal what the unit holds"
    );
}

// ---------------------------------------------------------------------------
// The build rate
// ---------------------------------------------------------------------------

#[test]
fn a_build_rate_of_zero_adds_nothing_and_a_rate_of_one_adds_something() {
    // Two worlds, one builder each, on the same island. The builders differ
    // in the build rate and in nothing else.
    let mut idle_world = world();
    let address = island(&idle_world);
    let cannot = define(
        &mut idle_world,
        CANNOT,
        UnitTypeRow {
            build_rate: Fix32::ZERO,
            ..WORKER_ROW
        },
    );
    let idle = spawn(&mut idle_world, address, cannot);
    assert!(idle_world.order_build(idle, UpgradeKind::Road));
    for _ in 0..TICKS {
        idle_world.step(1).expect("the step must run");
        assert_eq!(
            idle_world.build_order(idle),
            Some(Some(UpgradeKind::Road)),
            "the unit that cannot build keeps its order"
        );
    }
    assert!(
        idle_world.upgrade_at(address).is_none(),
        "a build rate of zero must start no site"
    );

    let mut busy_world = world();
    let can = define(
        &mut busy_world,
        CAN,
        UnitTypeRow {
            build_rate: Fix32::ONE,
            ..WORKER_ROW
        },
    );
    let mason = spawn(&mut busy_world, address, can);
    assert!(busy_world.order_build(mason, UpgradeKind::Road));
    busy_world.step(1).expect("the step must run");
    let site = busy_world
        .upgrade_at(address)
        .expect("a build rate of one must start a site");
    assert!(site.progress.0 > 0, "a build rate of one must add work");
}

#[test]
fn a_build_rate_of_two_adds_twice_what_a_rate_of_one_adds() {
    // The scale is exact, so a rate of two on one tick equals a rate of one
    // on two ticks.
    let mut single = world();
    let address = island(&single);
    let one = define(
        &mut single,
        CAN,
        UnitTypeRow {
            build_rate: Fix32::ONE,
            ..WORKER_ROW
        },
    );
    let mason = spawn(&mut single, address, one);
    assert!(single.order_build(mason, UpgradeKind::Road));
    single.step(1).expect("the step must run");
    single.step(1).expect("the step must run");
    let two_ticks_at_one = single
        .upgrade_at(address)
        .expect("the unit built here")
        .progress;

    let mut double = world();
    let two = define(
        &mut double,
        MORE,
        UnitTypeRow {
            build_rate: Fix32::from_int(2),
            ..WORKER_ROW
        },
    );
    let mason = spawn(&mut double, address, two);
    assert!(double.order_build(mason, UpgradeKind::Road));
    double.step(1).expect("the step must run");
    let one_tick_at_two = double
        .upgrade_at(address)
        .expect("the unit built here")
        .progress;

    assert!(two_ticks_at_one.0 > 0);
    assert_eq!(one_tick_at_two, two_ticks_at_one);
}

// ---------------------------------------------------------------------------
// The default table
// ---------------------------------------------------------------------------

#[test]
fn a_new_world_is_built_with_the_default_table() {
    // The world holds the default table and not an empty one, so a unit that
    // nothing typed is a worker and gathers.
    let field = world();
    assert_eq!(
        *field.unit_types(),
        cachette_core::DEFAULT_UNIT_TYPE_TABLE,
        "the world must be built with the default table"
    );
    let worker = field.unit_types().row(cachette_core::DEFAULT_UNIT_TYPE);
    assert!(worker.gather_rate.0 > 0, "the default type must gather");
    assert!(worker.build_rate.0 > 0, "the default type must build");
    assert!(worker.carry_capacity > 0, "the default type must carry");
}
