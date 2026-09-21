//! Tests for the theft prevention rule on foraging and harvesting.
//!
//! A unit cannot forage or harvest on another faction's held ground unless the
//! two factions are at war. Friendly foraging on another faction's ground is
//! theft and is refused.[^1]
//!
//! # References
//!
//! [^1]: ADR-0073, gathering is admitted by sort-then-admit against the tile. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`

use cachette_core::resource::{Amount, ResourceKind};
use cachette_core::{Axial, FactionId, World, WorldConfig};

const EXTENT: WorldConfig = WorldConfig {
    width: 64,
    height: 64,
    seed: 135,
    faction_count: 3,
    unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    ..WorldConfig::DEFAULT
};

/// Finds a tile held by the given faction that carries a resource deposit.
fn find_held_deposit(world: &World, faction: FactionId) -> (Axial, ResourceKind) {
    let grid = world.grid();
    for col in 0..grid.width() as i32 {
        for row in 0..grid.height() as i32 {
            let addr = Axial::new(col, row);
            if world.tile_holder(addr).and_then(|h| h.faction()) == Some(faction)
                && world.admits_a_unit(addr)
            {
                for kind in ResourceKind::ALL {
                    if world.original_stock(addr, kind) > Some(Amount::ZERO) {
                        return (addr, kind);
                    }
                }
            }
        }
    }
    panic!("fixture must have a deposit held by {faction:?}");
}

/// Finds an unheld tile that carries a resource deposit.
fn find_unheld_deposit(world: &World) -> (Axial, ResourceKind) {
    let grid = world.grid();
    for col in 0..grid.width() as i32 {
        for row in 0..grid.height() as i32 {
            let addr = Axial::new(col, row);
            if world.tile_holder(addr).and_then(|h| h.faction()).is_none()
                && world.admits_a_unit(addr)
            {
                for kind in ResourceKind::ALL {
                    if world.original_stock(addr, kind) > Some(Amount::ZERO) {
                        return (addr, kind);
                    }
                }
            }
        }
    }
    panic!("fixture must have an unheld deposit");
}

#[test]
fn friendly_units_cannot_forage_on_another_factions_held_ground() {
    let mut world = World::new(EXTENT).expect("extent describes a world");
    let seat = Axial::new(20, 20);
    assert!(world.admits_a_unit(seat));

    // Found city for faction 1.
    world
        .found_settlement(seat, FactionId(1))
        .expect("seat must admit settlement");
    world.step(1).expect("step must run");

    // Find a tile held by faction 1 that has a deposit.
    let (held_tile, kind) = find_held_deposit(&world, FactionId(1));

    // Faction 0 and Faction 1 are at peace by default.
    assert!(
        !world.at_war(FactionId(0), FactionId(1)),
        "factions must start at peace"
    );

    // Spawn a unit of faction 0 on faction 1's held ground.
    let foreigner = world
        .spawn_soldier(held_tile, FactionId(0))
        .expect("foreigner must spawn");

    // Order the friendly foreign unit to gather.
    assert!(world.order_gather(foreigner, kind));

    // Step the world.
    world.step(1).expect("step must run");

    // The unit must take nothing because gathering on another faction's ground
    // during peace is theft.
    assert!(
        world.gather_log().is_empty(),
        "no gather event must be emitted for friendly foraging on foreign land"
    );
    let carry = world
        .soldiers()
        .carry(foreigner)
        .expect("unit must exist")
        .total();
    assert_eq!(
        carry.0, 0,
        "friendly foreign unit must carry nothing after attempting theft"
    );
}

#[test]
fn units_can_forage_on_enemy_held_ground_during_war() {
    let mut world = World::new(EXTENT).expect("extent describes a world");
    let seat = Axial::new(20, 20);
    assert!(world.admits_a_unit(seat));

    // Found city for faction 1.
    world
        .found_settlement(seat, FactionId(1))
        .expect("seat must admit settlement");
    world.step(1).expect("step must run");

    let (held_tile, kind) = find_held_deposit(&world, FactionId(1));

    // Put faction 0 and faction 1 at war.
    assert!(world.set_relation(FactionId(0), FactionId(1), -20));
    assert!(world.set_relation(FactionId(1), FactionId(0), -20));
    assert!(
        world.at_war(FactionId(0), FactionId(1)),
        "factions must be at war"
    );

    // Spawn a unit of faction 0 on enemy held ground.
    let invader = world
        .spawn_soldier(held_tile, FactionId(0))
        .expect("invader must spawn");

    // Order the wartime invader to gather.
    assert!(world.order_gather(invader, kind));

    // Step the world.
    world.step(1).expect("step must run");

    // During war, foraging/pillaging on enemy ground is permitted.
    assert!(
        !world.gather_log().is_empty(),
        "gather event must be emitted for wartime foraging on enemy land"
    );
    let carry = world
        .soldiers()
        .carry(invader)
        .expect("unit must exist")
        .total();
    assert!(
        carry.0 > 0,
        "wartime invader must successfully gather resources on enemy land"
    );
}

#[test]
fn units_can_forage_on_own_held_ground_and_unheld_ground() {
    let mut world = World::new(EXTENT).expect("extent describes a world");
    let seat = Axial::new(20, 20);
    assert!(world.admits_a_unit(seat));

    world
        .found_settlement(seat, FactionId(1))
        .expect("seat must admit settlement");
    world.step(1).expect("step must run");

    let (own_tile, own_kind) = find_held_deposit(&world, FactionId(1));

    // Unit of faction 1 gathering on faction 1's own land.
    let own_unit = world
        .spawn_soldier(own_tile, FactionId(1))
        .expect("own unit must spawn");
    assert!(world.order_gather(own_unit, own_kind));

    // Unit of faction 0 gathering on unheld land.
    let (wild_tile, wild_kind) = find_unheld_deposit(&world);
    let wild_unit = world
        .spawn_soldier(wild_tile, FactionId(0))
        .expect("wild unit must spawn");
    assert!(world.order_gather(wild_unit, wild_kind));

    world.step(1).expect("step must run");

    assert!(
        world
            .soldiers()
            .carry(own_unit)
            .expect("unit must exist")
            .total()
            .0
            > 0,
        "unit must be able to forage on its own faction's ground"
    );
    assert!(
        world
            .soldiers()
            .carry(wild_unit)
            .expect("unit must exist")
            .total()
            .0
            > 0,
        "unit must be able to forage on unheld wild ground"
    );
}

#[test]
fn a_corrupted_peace_check_would_permit_theft() {
    // Proven-to-fail verification:
    // A peaceful unit on another faction's held ground must never gather anything.
    let mut world = World::new(EXTENT).expect("extent describes a world");
    let seat = Axial::new(20, 20);
    world.found_settlement(seat, FactionId(1)).unwrap();
    world.step(1).unwrap();

    let (held_tile, kind) = find_held_deposit(&world, FactionId(1));

    let foreigner = world.spawn_soldier(held_tile, FactionId(0)).unwrap();
    world.order_gather(foreigner, kind);
    world.step(1).unwrap();

    // Assert that the gather log has 0 entries from this unit.
    assert_eq!(
        world
            .gather_log()
            .iter()
            .filter(|r| r.unit == foreigner.to_bits())
            .count(),
        0,
        "theft must not be permitted"
    );
}
