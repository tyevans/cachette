//! The military strength of a faction, of a tile, and of a cell under fog.
//!
//! The strength of one unit is the attack of its type plus the armour of its
//! type. A meeting reads the attack as the harm one group delivers and the
//! armour as the threshold that the attack must pass, so both columns raise
//! the worth of a unit.[^1] [^2]
//!
//! The tests drive the engine and then read the engine. A reader that no test
//! reaches through the public interface is inert.[^3]
//!
//! # References
//!
//! [^1]: ADR-0122, an attacker whose attack does not exceed the defender's armour contributes exactly zero, decision D1. `docs/adrs/draft/adr-0122-an-attacker-below-the-armour-contributes-exactly-zero.md`
//! [^2]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D1. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
//! [^3]: Testing rules, section 5. `.agents/rules/testing.md`

use cachette_core::types::FACTION_CEILING;
use cachette_core::{
    Accum, Admit, Axial, FactionId, Fix32, UnitTypeId, UnitTypeRow, World, WorldConfig,
    UNIT_TYPE_COUNT,
};

/// The attack of the strong type of the fixture, in whole units.
const STRONG_ATTACK: i16 = 7;
/// The armour of the strong type of the fixture, in whole units.
const STRONG_ARMOUR: i16 = 5;
/// The attack of the weak type of the fixture, in whole units.
const WEAK_ATTACK: i16 = 1;

/// Builds a world whose type table holds one strong row and one weak row.
///
/// **The two rows are far apart, and one of them carries no armour at all.**
/// A fixture whose rows were alike would pass an aggregate that dropped
/// either column, and a fixture in which every row carried armour would pass
/// a product of the two columns.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
fn world() -> World {
    let mut world = World::new(WorldConfig {
        width: 40,
        height: 40,
        seed: 11,
        faction_count: 3,
        unit_capacity: 256,
    })
    .expect("the extent must describe a world");
    world
        .define_unit_type(
            0,
            UnitTypeRow {
                attack: Fix32::from_int(STRONG_ATTACK),
                armour: Fix32::from_int(STRONG_ARMOUR),
                ..UnitTypeRow::NONE
            },
        )
        .expect("the row is legal");
    world
        .define_unit_type(
            1,
            UnitTypeRow {
                attack: Fix32::from_int(WEAK_ATTACK),
                armour: Fix32::ZERO,
                ..UnitTypeRow::NONE
            },
        )
        .expect("the row is legal");
    world
}

/// Returns the passable addresses of the fixture world, in index order.
///
/// The generated ground holds water, and no unit stands on water. A fixture
/// that named an address without asking the ground would refuse the spawn.
fn dry_ground(world: &World) -> Vec<Axial> {
    let mut dry = Vec::new();
    for row in 0..world.grid().height() {
        for column in 0..world.grid().width() {
            let here = Axial::new(column as i32, row as i32);
            if world
                .tile_kind(here)
                .is_some_and(cachette_core::TileKind::is_passable)
            {
                dry.push(here);
            }
        }
    }
    dry
}

/// Returns one passable address of the fixture world.
///
/// The ordinal walks the passable ground in index order, so two calls with
/// two ordinals name two different tiles.
fn dry(world: &World, ordinal: usize) -> Axial {
    let dry = dry_ground(world);
    assert!(
        dry.len() > ordinal,
        "the fixture world must hold more than {ordinal} passable tiles"
    );
    dry[ordinal]
}

/// Returns the type identity of a small number.
fn unit_type(number: u8) -> UnitTypeId {
    UnitTypeId::from_u8(number).expect("the number names a row of the table")
}

/// Returns the strength of one unit of the strong type, in raw fixed point.
fn strong_one() -> i64 {
    i64::from(Fix32::from_int(STRONG_ATTACK).0) + i64::from(Fix32::from_int(STRONG_ARMOUR).0)
}

#[test]
fn the_strength_of_a_faction_reads_both_columns_of_every_type_it_holds() {
    let mut world = world();
    let first = dry(&world, 0);
    let second = dry(&world, 1);
    let strong = world
        .spawn_soldier(first, FactionId(0))
        .expect("the spawn must succeed");
    let weak = world
        .spawn_soldier(second, FactionId(0))
        .expect("the spawn must succeed");
    assert!(world.set_unit_type(strong, unit_type(0)));
    assert!(world.set_unit_type(weak, unit_type(1)));
    world.step(1).expect("the step must run");

    let expected = strong_one() + i64::from(Fix32::from_int(WEAK_ATTACK).0);
    assert_eq!(world.faction_strength(FactionId(0)).0, expected);
    assert_eq!(world.faction_strength(FactionId(1)).0, 0);
}

#[test]
fn a_unit_that_holds_no_armour_still_carries_strength() {
    let mut world = world();
    let place = dry(&world, 0);
    let weak = world
        .spawn_soldier(place, FactionId(0))
        .expect("the spawn must succeed");
    assert!(world.set_unit_type(weak, unit_type(1)));
    world.step(1).expect("the step must run");
    assert!(
        world.faction_strength(FactionId(0)).0 > 0,
        "a product of the attack and the armour would read zero here"
    );
}

#[test]
fn the_strength_of_a_faction_moves_with_its_type_census() {
    let mut world = world();
    let place = dry(&world, 0);
    let unit = world
        .spawn_soldier(place, FactionId(0))
        .expect("the spawn must succeed");
    assert!(world.set_unit_type(unit, unit_type(1)));
    world.step(1).expect("the step must run");
    let weak = world.faction_strength(FactionId(0)).0;

    assert!(world.set_unit_type(unit, unit_type(0)));
    world.step(1).expect("the step must run");
    let strong = world.faction_strength(FactionId(0)).0;
    assert!(
        strong > weak,
        "a change of type must move the strength: {strong} against {weak}"
    );

    assert!(world.despawn_soldier(unit));
    world.step(1).expect("the step must run");
    assert_eq!(world.faction_strength(FactionId(0)).0, 0);
    assert!(world.check_invariants());
}

#[test]
fn the_type_census_of_every_faction_sums_to_its_population() {
    let mut world = world();
    let dry = dry_ground(&world);
    for (ordinal, place) in dry.iter().take(24).enumerate() {
        let unit = world
            .spawn_soldier(*place, FactionId((ordinal % 3) as u16))
            .expect("the spawn must succeed");
        assert!(world.set_unit_type(unit, unit_type((ordinal % 2) as u8)));
    }
    world.step(4).expect("the step must run");
    for number in 0..FACTION_CEILING {
        let faction = FactionId(number);
        let census = world.population_by_type(faction);
        assert_eq!(census.len(), UNIT_TYPE_COUNT);
        assert_eq!(
            census.iter().sum::<u32>(),
            world.population_of(faction),
            "the type census of faction {number} must sum to its population"
        );
    }
    assert!(world.check_invariants());
}

#[test]
fn a_tile_reports_the_strength_of_the_units_that_stand_on_it() {
    let mut world = world();
    let place = dry(&world, 8);
    let unit = world
        .spawn_soldier(place, FactionId(0))
        .expect("the spawn must succeed");
    assert!(world.set_unit_type(unit, unit_type(0)));
    world.rebuild_bridge(1).expect("the rebuild must run");
    let held = world
        .soldiers()
        .address(unit)
        .expect("the soldier is alive");
    assert_eq!(world.tile_strength(held), Ok(Accum(strong_one())));
    let dry = dry_ground(&world);
    let empty = *dry.last().expect("the fixture world holds passable ground");
    assert_ne!(empty, held);
    assert_eq!(world.tile_strength(empty), Ok(Accum(0)));
}

#[test]
fn a_faction_reads_no_rival_strength_on_ground_it_cannot_see() {
    let mut world = world();
    let dry = dry_ground(&world);
    let mine = dry[0];
    let far = *dry.last().expect("the fixture world holds passable ground");
    assert!(
        mine.distance(far) > u32::from(world.sight_rules().ceiling()),
        "the fixture must place the rival outside every sight the rules admit"
    );
    let watcher = world
        .spawn_soldier(mine, FactionId(0))
        .expect("the spawn must succeed");
    let rival = world
        .spawn_soldier(far, FactionId(1))
        .expect("the spawn must succeed");
    assert!(world.set_unit_type(watcher, unit_type(0)));
    assert!(world.set_unit_type(rival, unit_type(0)));
    world.step(1).expect("the step must run");

    let here = world
        .faction_summary_covering(FactionId(0), mine, Admit::SeenNow)
        .expect("the reader must answer for a cell the faction sees");
    assert!(
        here.own_strength().0 > 0,
        "the faction must read its own strength where it stands"
    );
    assert_eq!(
        here.other_strength().0,
        0,
        "no rival stands on the ground the faction watches"
    );

    let there = world
        .faction_summary_covering(FactionId(0), far, Admit::SeenNow)
        .expect("the reader must answer for a cell the faction cannot see");
    assert_eq!(
        there.other_strength().0,
        0,
        "a cell the faction cannot see must report no rival strength"
    );
    assert_eq!(there.admitted(), 0);
}

#[test]
fn the_strength_of_a_cell_is_the_sum_of_its_tiles_in_any_order() {
    let mut world = world();
    let dry = dry_ground(&world);
    let origin = dry[0];
    for (ordinal, place) in dry.iter().take(6).enumerate() {
        let unit = world
            .spawn_soldier(*place, FactionId(0))
            .expect("the spawn must succeed");
        assert!(world.set_unit_type(unit, unit_type((ordinal % 2) as u8)));
    }
    world.step(1).expect("the step must run");

    let cell = world
        .faction_summary_covering(FactionId(0), origin, Admit::SeenNow)
        .expect("the reader must answer");
    let layout = world.observation().layout();
    let block = layout
        .key_of(
            world
                .grid()
                .index_of(origin)
                .expect("the address is inside the world"),
        )
        .map(|key| layout.block_of_key(key))
        .expect("the address lies in a block");
    let mut tiles = Vec::new();
    for row in 0..world.grid().height() {
        for column in 0..world.grid().width() {
            let here = Axial::new(column as i32, row as i32);
            let Some(tile) = world.grid().index_of(here) else {
                continue;
            };
            let Some(key) = layout.key_of(tile) else {
                continue;
            };
            if layout.block_of_key(key) == block && world.faction_sees_now(FactionId(0), here) {
                tiles.push(here);
            }
        }
    }
    let mut ascending = 0i64;
    let mut descending = 0i64;
    for here in &tiles {
        ascending += world.tile_strength(*here).expect("the bridge is fresh").0;
    }
    for here in tiles.iter().rev() {
        descending += world.tile_strength(*here).expect("the bridge is fresh").0;
    }
    assert_eq!(
        ascending, descending,
        "the sum must not depend on the order"
    );
    assert_eq!(cell.own_strength().0, ascending);
    assert!(ascending > 0, "the fixture must place strength in the cell");
}
