//! The world reserves the unit columns at construction.
//!
//! The settings of the world name a unit reservation. The world reserves that
//! many entries in each unit column when it is built, it opens no more slots,
//! and a spawn past the reservation gets a typed refusal.[^1]
//!
//! The product record states the property these tests check. The storage the
//! world reserves is sized for the target population, it does not change
//! during a run, and a run does not stop to grow.[^2] The engine did the
//! opposite, and the findings register holds what was believed and what was
//! true.[^3]
//!
//! **Each test here has a defect that must break it.** A test that asserts a
//! capacity rather than an address stays green over a column that grows,
//! because the capacity a growing column reports is the capacity it happens
//! to have reached. The address of the first entry is the thing that moves
//! when a column reallocates, so the address is what these tests read.
//!
//! # References
//!
//! [^1]: ADR-0084, the world reserves the unit columns at construction. `docs/adrs/draft/adr-0084-the-world-reserves-the-unit-columns-at-construction.md`
//! [^2]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
//! [^3]: Findings register, FND-135. `docs/FINDINGS.md`

use cachette_core::{Axial, FactionId, FoundingError, SoldierError, World, WorldConfig};

/// The reservation that the fixtures name.
///
/// The number bounds the fixture and states nothing about the project. It is
/// small enough that a test fills it, because a world that never approaches
/// its reservation supplies no input that could fail either assertion.[^1]
///
/// # References
///
/// [^1]: Testing Rules, section 2a. `.claude/rules/testing.md`
const RESERVATION: u32 = 64;

/// The size of the group that the refused founding tries to seat.
///
/// The world that founds it reserves half of this, so the arena runs out of
/// slots part way through the group.
const GROUP: u32 = 8;

/// Builds a world that reserves `RESERVATION` unit slots.
fn world() -> World {
    World::new(WorldConfig {
        width: 32,
        height: 32,
        seed: 0x5eed_0000_0000_0084,
        faction_count: 1,
        unit_capacity: RESERVATION,
    })
    .expect("a small extent describes a world")
}

/// Returns an address that admits a unit.
///
/// A spawn refuses ground that admits no unit, and the terrain is generated
/// from the seed, so the test finds a tile rather than naming one.
fn passable(world: &World) -> Axial {
    for row in 0..world.grid().height() {
        for column in 0..world.grid().width() {
            let address = Axial::new(
                i32::try_from(column).expect("a small extent fits an axial"),
                i32::try_from(row).expect("a small extent fits an axial"),
            );
            if world
                .tile_kind(address)
                .is_some_and(cachette_core::TileKind::is_passable)
            {
                return address;
            }
        }
    }
    panic!("the generated world holds no passable tile");
}

/// Returns the address of the first entry of each unit column.
///
/// A column that reallocates gives its entries a new address. Nothing else
/// moves them, because the arena never compacts the slot index space.
fn column_addresses(world: &World) -> Vec<*const u8> {
    let arena = world.soldiers();
    vec![
        arena.tile_column().as_ptr().cast(),
        arena.faction_column().as_ptr().cast(),
        arena.carry_column().as_ptr().cast(),
        arena.order_column().as_ptr().cast(),
        arena.need_column().as_ptr().cast(),
        arena.deficit_column().as_ptr().cast(),
        arena.home_column().as_ptr().cast(),
        arena.intent_column().as_ptr().cast(),
        arena.live_column().as_ptr().cast(),
    ]
}

/// Spawns `count` units on one tile.
///
/// A spawn does not read the capacity of the ground, so one tile takes the
/// whole reservation.[^1] The test therefore measures the storage and not the
/// admission rule.
///
/// # References
///
/// [^1]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
fn fill(world: &mut World, count: u32) {
    let address = passable(world);
    for _ in 0..count {
        world
            .spawn_soldier(address, FactionId(0))
            .expect("a spawn inside the reservation must succeed");
    }
}

#[test]
fn the_settings_name_the_reservation_and_the_arena_takes_it() {
    let world = world();
    assert_eq!(world.config().unit_capacity, RESERVATION);
    assert_eq!(world.soldiers().capacity(), RESERVATION);
}

#[test]
fn a_default_world_reserves_the_target_population() {
    // The default names the answered target and no number of its own. A
    // second site that stated a default would read back correctly and change
    // nothing, which is the shape this project keeps meeting.
    let config = WorldConfig::default();
    assert_eq!(config.unit_capacity, WorldConfig::TARGET_UNIT_POPULATION);
    let world = World::new(config).expect("the default settings describe a world");
    assert_eq!(
        world.soldiers().capacity(),
        WorldConfig::TARGET_UNIT_POPULATION
    );
}

#[test]
fn a_spawn_past_the_reservation_is_refused() {
    let mut world = world();
    fill(&mut world, RESERVATION);
    let address = passable(&world);
    assert_eq!(
        world.spawn_soldier(address, FactionId(0)),
        Err(SoldierError::ArenaFull)
    );
    // The refusal changed nothing. A refused spawn is not a partial one.
    assert_eq!(world.soldiers().len(), RESERVATION);
    assert_eq!(world.soldiers().slot_count(), RESERVATION);
}

#[test]
fn a_death_lets_a_spawn_through_again() {
    // The reservation bounds the slots the arena opens, not the spawns a run
    // makes. A run that ends a unit and starts another stays inside it.
    let mut world = world();
    fill(&mut world, RESERVATION);
    let first = world
        .soldiers()
        .iter()
        .next()
        .expect("the world holds units");
    assert!(world.despawn_soldier(first));
    let address = passable(&world);
    assert!(world.spawn_soldier(address, FactionId(0)).is_ok());
}

#[test]
fn the_unit_columns_do_not_move_while_a_run_fills_them() {
    let mut world = world();
    let before = column_addresses(&world);
    fill(&mut world, RESERVATION);
    let after = column_addresses(&world);
    assert_eq!(
        before, after,
        "a unit column moved, so the arena reallocated under the run"
    );
}

#[test]
fn a_copy_of_a_world_keeps_the_reservation() {
    // A derived copy of a column allocates for what the column holds, not for
    // what it reserved. A copied world would then grow where the original
    // does not, and nothing would report it.
    let mut world = world();
    fill(&mut world, 1);
    let mut copy = world.clone();
    let before = column_addresses(&copy);
    fill(&mut copy, RESERVATION - 1);
    let after = column_addresses(&copy);
    assert_eq!(
        before, after,
        "a unit column of the copy moved, so the copy lost the reservation"
    );
    assert_eq!(copy.soldiers().capacity(), RESERVATION);
}

#[test]
fn a_founding_past_the_reservation_is_refused_and_leaves_nothing() {
    // The founding group is larger than the reservation, so the arena runs
    // out of slots part way through the group. The founding must report the
    // refusal and undo what it did.
    let mut world = World::new(WorldConfig {
        width: 32,
        height: 32,
        seed: 0x5eed_0000_0000_0084,
        faction_count: 1,
        unit_capacity: GROUP / 2,
    })
    .expect("a small extent describes a world");
    // A refused founding does not restore the state hash, and it must not be
    // asked to. The arena never compacts the slot index space and a
    // generation never rewinds, so the slots the founding opened stay open
    // and their generations stay advanced. That is the arena rule. What the
    // undo owes is that nothing lives and nothing stands.
    let outcome = world.found_run(GROUP, FactionId(0));
    assert_eq!(
        outcome.err(),
        Some(FoundingError::Person(SoldierError::ArenaFull))
    );
    assert_eq!(world.soldiers().len(), 0, "a refused founding left people");
    assert_eq!(
        world.settlements().len(),
        0,
        "a refused founding left a settlement standing"
    );
    assert!(
        world.check_invariants(),
        "a refused founding left the world outside its invariants"
    );
}

#[test]
fn a_refused_founding_frees_the_slots_it_opened() {
    // The undo despawns the people it seated, and a despawn frees a slot
    // rather than closing it. The world therefore keeps the slots the refused
    // founding opened, and the next founding reuses them. This is the arena
    // rule and not a defect: the arena never compacts the slot index space.
    // The test states it, so that a later reader does not read the open slots
    // as wreckage.
    let mut world = World::new(WorldConfig {
        width: 32,
        height: 32,
        seed: 0x5eed_0000_0000_0084,
        faction_count: 1,
        unit_capacity: GROUP / 2,
    })
    .expect("a small extent describes a world");
    assert!(world.found_run(GROUP, FactionId(0)).is_err());
    assert_eq!(world.soldiers().len(), 0);
    assert_eq!(
        world.soldiers().slot_count(),
        GROUP / 2,
        "the refused founding opened the whole reservation before it stopped"
    );
    // Every slot it opened is free, so the arena is empty and not full.
    assert!(world.soldiers().check_invariants());
}
