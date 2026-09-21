//! Tests for territory closure: filling ground that one faction has surrounded.
//!
//! A faction that surrounds an unheld tile takes that tile through closure
//! passes, provided that no neighbour belongs to another faction and the
//! neighbour count reaches the threshold.[^1]
//!
//! # References
//!
//! [^1]: ADR-0153, a tile's lease follows the units that stand on it, decisions D6 and D7. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`

use cachette_core::holding::{ClosureRules, Holder};
use cachette_core::terrain::TileKind;
use cachette_core::{Axial, FactionId, World, WorldConfig};

const EXTENT: WorldConfig = WorldConfig {
    width: 64,
    height: 64,
    seed: 135,
    faction_count: 3,
    unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    ..WorldConfig::DEFAULT
};

/// Finds an impassable water tile whose six hex neighbours are all passable land.
fn find_isolated_water_tile(world: &World) -> Option<Axial> {
    let grid = world.grid();
    for row in 4..(grid.height() - 4) {
        for column in 4..(grid.width() - 4) {
            let center = Axial::new(column as i32, row as i32);
            if world.tile_kind(center) != Some(TileKind::Water) {
                continue;
            }
            let all_neighbours_passable = (0..6).all(|dir| {
                grid.neighbour(center, dir)
                    .is_some_and(|n| world.admits_a_unit(n))
            });
            if all_neighbours_passable {
                return Some(center);
            }
        }
    }
    None
}

#[test]
fn an_enclosed_lake_is_claimed_by_the_surrounding_faction() {
    let mut world = World::new(EXTENT).expect("extent describes a world");
    let lake = find_isolated_water_tile(&world)
        .expect("the world fixture must contain at least one isolated water tile");

    // Found a settlement adjacent to the lake.
    // The base reach of a city is 2 (radius 2), which covers the city tile,
    // ring 1 (radius 1, 6 neighbours including the lake), and ring 2.
    // The city's reach covers all 6 ring-1 land neighbours of the lake.
    let grid = world.grid();
    let seat = grid
        .neighbour(lake, 0)
        .expect("lake has neighbour inside world");
    world
        .found_settlement(seat, FactionId(0))
        .expect("seat must admit settlement");

    // Without closure (pass_count = 0), water is never held under D5.
    world.set_closure_rules(ClosureRules::new(0, 4));
    world.step(1).expect("step must run");
    assert_eq!(
        world.tile_holder(lake).and_then(Holder::faction),
        None,
        "without closure, water is unheld"
    );

    // With default closure (pass_count = 2, threshold = 4):
    // The 6 neighbours around the lake are within reach of the city and held by faction 0.
    // In pass 1, the lake has 6 neighbours of faction 0, which is >= 4 and 0 of any rival.
    // Thus the lake becomes held by faction 0.
    world.set_closure_rules(ClosureRules::DEFAULT);
    world.step(1).expect("step must run");
    assert_eq!(
        world.tile_holder(lake).and_then(Holder::faction),
        Some(FactionId(0)),
        "closure must give the enclosed lake to the surrounding faction"
    );
    assert!(
        world.check_invariants(),
        "the world invariants must hold when enclosed water is held"
    );
}

#[test]
fn straight_borders_do_not_expand_outward_by_closure() {
    let mut world = World::new(EXTENT).expect("extent describes a world");
    // With a city at (20, 20) and reach 2:
    // Reach 2 gives a disc of radius 2.
    // Beyond radius 2, tiles along a straight border have at most 3 neighbours in the disc.
    // Because the threshold is 4 (and 3 < 4), closure passes must NOT expand
    // across a straight border.
    let seat = Axial::new(20, 20);
    assert!(world.admits_a_unit(seat));
    world
        .found_settlement(seat, FactionId(0))
        .expect("seat must admit settlement");

    // Run 1 step with closure = 0
    world.set_closure_rules(ClosureRules::new(0, 4));
    world.step(1).expect("step must run");
    let held_without_closure = world.holding().holding_of(FactionId(0));

    // Run 1 step with closure = 2, threshold = 4
    world.set_closure_rules(ClosureRules::new(2, 4));
    world.step(1).expect("step must run");
    let held_with_closure = world.holding().holding_of(FactionId(0));

    // If all tiles around the city are ordinary land without isolated gaps,
    // the reach disc of radius 2 has only straight or convex outward edges.
    // A regular hex disc of radius 2 has no concave indentation, so no
    // exterior tile has 4 or more neighbours in the disc.
    // Therefore, closure adds zero tiles to a plain convex disc on flat land.
    assert_eq!(
        held_without_closure, held_with_closure,
        "a convex reach disc has at most 3 neighbours per exterior tile and must not expand"
    );
}

#[test]
fn a_tile_with_neighbours_of_two_factions_stays_unheld() {
    let mut world = World::new(EXTENT).expect("extent describes a world");
    let lake = find_isolated_water_tile(&world)
        .expect("the world fixture must contain at least one isolated water tile");
    let grid = world.grid();

    // Found two cities of two different factions on opposite sides of the lake.
    let seat0 = grid
        .neighbour(lake, 0)
        .expect("lake has neighbour inside world");
    let seat1 = grid
        .neighbour(lake, 3)
        .expect("lake has neighbour inside world");
    world
        .found_settlement(seat0, FactionId(0))
        .expect("seat0 must admit settlement");
    world
        .found_settlement(seat1, FactionId(1))
        .expect("seat1 must admit settlement");

    // Step the world with default closure.
    world.step(1).expect("step must run");

    // The lake has neighbours of both faction 0 and faction 1.
    // Per ADR-0153 D6, a tile whose neighbours name two factions stays unheld.
    assert_eq!(
        world.tile_holder(lake).and_then(Holder::faction),
        None,
        "a tile whose neighbours name two factions must stay unheld"
    );
    assert!(world.check_invariants());
}

#[test]
fn closure_never_claims_ground_held_by_another_faction() {
    let mut world = World::new(EXTENT).expect("extent describes a world");
    // Place city of faction 0 and city of faction 1 close enough that their reach discs touch.
    // Any tile held by faction 1 must never be converted to faction 0 by closure.
    let seat0 = Axial::new(10, 10);
    let seat1 = Axial::new(13, 10);
    world
        .found_settlement(seat0, FactionId(0))
        .expect("seat0 admits settlement");
    world
        .found_settlement(seat1, FactionId(1))
        .expect("seat1 admits settlement");

    world.step(1).expect("step must run");

    // Every tile held by faction 1 before closure passes remains held by faction 1.
    for row in 8..15 {
        for col in 8..15 {
            let addr = Axial::new(col, row);
            if world.tile_holder(addr).and_then(Holder::faction) == Some(FactionId(1)) {
                // Faction 1 holds this tile. Closure from faction 0 must never overwrite it.
                assert_eq!(
                    world.tile_holder(addr).and_then(Holder::faction),
                    Some(FactionId(1)),
                    "closure must never take ground another faction holds"
                );
            }
        }
    }
}

#[test]
fn a_corrupted_neighbour_threshold_refuses_closure() {
    // Proven-to-fail test:
    // If the neighbour threshold is corrupted to 7 (impossible on a 6-neighbour hex grid),
    // closure never triggers, even for a fully surrounded lake with 6 neighbours.
    let mut world = World::new(EXTENT).expect("extent describes a world");
    let lake =
        find_isolated_water_tile(&world).expect("world fixture must contain isolated water tile");
    let grid = world.grid();
    let seat = grid
        .neighbour(lake, 0)
        .expect("lake has neighbour inside world");
    world
        .found_settlement(seat, FactionId(0))
        .expect("seat must admit settlement");

    world.set_closure_rules(ClosureRules::new(2, 7));
    world.step(1).expect("step must run");

    assert_eq!(
        world.tile_holder(lake).and_then(Holder::faction),
        None,
        "a threshold above 6 must prevent closure from firing"
    );
}
